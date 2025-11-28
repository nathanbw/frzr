use std::fs;
use std::fs::DirEntry;
use std::io;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

use sha2::{Digest, Sha256};

use sqlite::Connection;
use sqlite::State;

use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;

use std::ffi::OsString;

// TODO: Example how to import `args` from clap so we can pass options to the "verbs" of frzr:
// use clap::{arg, Command};
use clap::{Command};

fn cli() -> Command<'static> {
    Command::new("frzr")
        .about("A bitrot detector")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            //TODO maybe add `--start-over` argument to `init` for blowing away .frzr and starting
            //     fresh?
            Command::new("init").about("Initialize the frzr db for the current directory"),
        )
        .subcommand(
            // TODO dump could probably take a run id and dump its checksums while defaulting to
            //      the latest
            Command::new("dump").about("Dump the latest run's checksums in `shasum` format"),
        )
        .subcommand(
            // TODO status could take a run id, too?
            // TODO will status ever show info about a currently-running `frzr` process?
            // TODO should this be `stats` instead of `status`?
            Command::new("status").about("Show the status of the latest run"),
        )
        .subcommand(
            // TODO Will check also print out if there are differences in what is in the DB vs what
            //      is on the disk?
            // TODO Maybe the default behavior will be to walk starting at CWD, but you can pass
            //      --full to do a full scan regardless of CWD
            Command::new("check")
                .about("Walk the filesystem, starting at CWD, compute and store checksums"),
        )
    //TODO Remove below commented-out stanza -- leaving for learning:
    // .subcommand(
    //     Command::new("stash")
    //         .args_conflicts_with_subcommands(true)
    //         .args(push_args())
    //         .subcommand(Command::new("push").args(push_args()))
    //         .subcommand(Command::new("pop").arg(arg!([STASH])))
    //         .subcommand(Command::new("apply").arg(arg!([STASH]))),
    // )
}

// fn push_args() -> Vec<clap::Arg<'static>> {
//     vec![arg!(-m --message <MESSAGE>).required(false)]
// }

// TODO: How to not crash when stdout is closed?  Answer: There are
//      two paths: register our own handler for SIGPIPE, or just
//      handle the broken pipe error that's returned as standard
//
//      I think the best path forward is just to use the write macros
//      and handle the standard broken pipe error
fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            // TODO: do we need to match `sub_matches`; here `_`?
            init();
        }
        Some(("dump", _)) => {
            dump();
        }
        Some(("check", _)) => {
            check();
        }
        Some(("stash", sub_matches)) => {
            let stash_command = sub_matches.subcommand().unwrap_or(("push", sub_matches));
            match stash_command {
                ("apply", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Applying {:?}", stash);
                }
                ("pop", sub_matches) => {
                    let stash = sub_matches.get_one::<String>("STASH");
                    println!("Popping {:?}", stash);
                }
                ("push", sub_matches) => {
                    let message = sub_matches.get_one::<String>("message");
                    println!("Pushing {:?}", message);
                }
                (name, _) => {
                    unreachable!("Unsupported subcommand `{}`", name)
                }
            }
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Calling out to {:?} with {:?}", ext, args);
        }
        _ => unreachable!(), // If all subcommands are defined above, anything else is unreachabe!()
    }
}

fn dump() {
    let db = match open_and_initialize_db() {
        Ok(db) => db,
        Err(e) => {
            println!(
                "There was a problem opening or initializing the DB: {:?}",
                e
            );
            exit(1);
        }
    };
    let mut current_run_id = 0;
    let mut statement = db
        .prepare("SELECT id FROM run ORDER BY id DESC LIMIT 1;")
        .unwrap();
    while State::Row == statement.next().unwrap() {
        current_run_id = statement.read::<i64>(0).unwrap();
    }
    if current_run_id == 0 {
        // TODO Return early; no runs in the DB yet
    }
    let mut statement = db
        .prepare("SELECT file_name, file_hash FROM file_entry WHERE run_id = ?;")
        .unwrap();
    statement.bind(1, current_run_id).unwrap();
    while State::Row == statement.next().unwrap() {
        let cur_file_name_vec = statement.read::<Vec<u8>>(0).unwrap();
        let cur_file_name = OsString::from_vec(cur_file_name_vec);
        let file_name_path = Path::new(&cur_file_name);

        let cur_file_hash = statement.read::<String>(1).unwrap();
        // TODO: print nasty filenames better
        println!("{}  {}", cur_file_hash, file_name_path.display());
    }
}

fn init() {
    // TODO: This is how I expect init to work:
    //       1. Check if .frzr directory exists. If it does, bail with message
    //       2. Create .frzr directory in the current directory
    //       3. Initialize the DB there
    let db_path = Path::new("./.frzr/");
    let path_exists = match db_path.try_exists() {
        Ok(exists) => exists,
        Err(e) => {
            // TODO Maybe print to stderr here:
            println!("Error checking for existing .frzr directory; not taking any action!");
            println!("Error was: {:?}", e);
            exit(1);
        }
    };
    if path_exists {
        // TODO Maybe print to stderr here:
        println!(".frzr directory already exists; not taking any action!");
        exit(1);
    };
    match std::fs::create_dir(db_path) {
        Ok(ok_val) => ok_val,
        Err(e) => {
            // TODO Maybe print to stderr here:
            println!("Error creating .frzr directory! Error was: {:?}", e);
            exit(1);
        }
    };
    // FUTURE: Maybe return the schema version as well as the connection?
    match open_and_initialize_db() {
        Ok(db) => db,
        Err(e) => {
            println!(
                "There was a problem opening or initializing the DB: {:?}",
                e
            );
            exit(1);
        }
    };
    // If we get here, then the db is open and ready for business
}

fn check() {
    // TODO: Everywhere but `init` ought to probably use a different function to get the DB
    //       Another possibility would be a param to `open_and_initialize_db` for whether to
    //       create the DB vs just open and verify schema or something
    let db = match open_and_initialize_db() {
        Ok(db) => db,
        Err(e) => {
            println!(
                "There was a problem opening or initializing the DB: {:?}",
                e
            );
            exit(1);
        }
    };
    // Now, let's iterate over all the files
    let path_buf: PathBuf = PathBuf::from(".");

    // TODO: We should have a .frzrignore, or maybe take as a CLI arg?
    let mut ignore_paths: Vec<PathBuf> = Vec::new();
    //TODO: Definitely should not ignore `.git` by default:
    ignore_paths.push(PathBuf::from("./.git"));
    //TODO: Definitely should not ignore `target` by default:
    ignore_paths.push(PathBuf::from("./target"));
    ignore_paths.push(PathBuf::from("./.frzr"));
    let filenames = match give_me_the_files(path_buf, ignore_paths) {
        Ok(filenames) => filenames,
        Err(e) => {
            println!("There was a problem recursing the filesystem: {:?}", e);
            exit(1);
        }
    };
    let file_iter = filenames.iter();
    let mut is_this_the_first_file = true;
    let mut current_run_id = 0;
    for filename in file_iter {
        let file_hash = match compute_the_hash(filename) {
            Ok(file_hash) => file_hash,
            Err(e) => {
                // TODO what should we do here? If we have started a run, should we write it out?
                println!("There was a problem computing a hash: {:?}", e);
                exit(1);
            }
        };
        if is_this_the_first_file {
            is_this_the_first_file = false;
            match db.execute("INSERT INTO run (start_time) VALUES (CURRENT_TIMESTAMP);") {
                Ok(_) => (), // TODO use the function/map that does this prettier
                Err(e) => {
                    // TODO what should we do here? Maybe we should
                    //      have a scheme where we copy the DB, write to
                    //      the copy, and then move it back into place on
                    //      success only
                    println!("There was a problem computing a hash: {:?}", e);
                    exit(1);
                }
            };
            let mut statement = db
                .prepare("SELECT id FROM run ORDER BY id DESC LIMIT 1;")
                .unwrap();
            while State::Row == statement.next().unwrap() {
                current_run_id = statement.read::<i64>(0).unwrap();
            }
        }

        let mut statement = db
            .prepare(
                "\
                        INSERT INTO file_entry (run_id, file_name, file_hash) VALUES (?, ?, ?);\
                    ",
            )
            .unwrap();

        statement.bind(1, current_run_id).unwrap();
        // TODO: Why does ? not work, and I have to use unwrap() instead? The error was:
        //       Cannot use the `?` operator in a function that returns `()`
        let filename_bytes = filename.as_os_str().as_bytes();
        statement.bind(2, filename_bytes).unwrap();
        statement.bind(3, file_hash.as_bytes()).unwrap();
        statement.next().unwrap();
        println!("filename: {:?}", filename);
    }
    // If we haven't crashed yet, then the run exists in the DB, the file_entry rows exist in
    // the db, and the run can be marked finished
    let mut statement = db
        .prepare(
            "\
                    UPDATE run set end_time = CURRENT_TIMESTAMP WHERE id = ?;\
                ",
        )
        .unwrap();
    statement.bind(1, current_run_id).unwrap();
    statement.next().unwrap();
    println!("Inserting the end_time timestamp");
    println!("Reached the end of the check() function");
}

fn compute_the_hash(file: &PathBuf) -> Result<String, io::Error> {
    let mut the_file = fs::File::open(file)?;

    let mut hasher = Sha256::new();

    let mut buf: [u8; 4096] = [0; 4096]; // Read 4k at a time
    loop {
        let num_bytes_read = the_file.read(&mut buf)?;
        if num_bytes_read == 0 {
            break;
        }
        // read bytes from the file, pass them to the hasher:
        hasher.update(&buf[..num_bytes_read]);
    }

    let result: String = format!("{:x}", hasher.finalize());
    Ok(result)
}

fn give_me_the_files(
    path_string: PathBuf,
    ignore_paths: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, io::Error> {
    let mut all_the_files: Vec<PathBuf> = Vec::new();
    let visited_dirs: Vec<DirEntry> = Vec::new();
    let result = dir_walk_recurser(path_string, visited_dirs, &mut all_the_files, &ignore_paths);
    match result {
        Ok(_) => Ok(all_the_files),
        Err(e) => Err(e),
    }
}

// TODO: is visited_dirs actually doing anything?
fn dir_walk_recurser(
    path_string: PathBuf,
    mut visited_dirs: Vec<DirEntry>,
    out_files: &mut Vec<PathBuf>,
    ignore_paths: &Vec<PathBuf>,
) -> Result<Vec<DirEntry>, io::Error> {
    // Return early if you see the .frzr/ directory:
    if ignore_paths.contains(&path_string) {
        println!("Skipping ignored path: {:?}", path_string);
        return Ok(visited_dirs);
    }
    let dir_iter = match fs::read_dir(path_string) {
        Ok(rd) => rd,
        Err(e) => {
            println!("An error occurred: {}", e);
            return Err(e);
        }
    };
    for entry in dir_iter {
        let entry = match entry {
            Ok(de) => de,
            Err(e) => {
                println!("An error occurred: {}", e);
                return Err(e);
            }
        };
        // println!("{:?}", entry);
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                println!("An error occurred: {}", e);
                return Err(e);
            }
        };
        let path_name = entry.path();
        if file_type.is_dir() {
            // add this to the list and recurse
            visited_dirs.push(entry);
            visited_dirs = match dir_walk_recurser(path_name, visited_dirs, out_files, ignore_paths)
            {
                Ok(vd) => vd,
                Err(e) => {
                    println!("An error occurred: {}", e);
                    return Err(e);
                }
            };
        } else {
            println!("File:\t{:?}", path_name);
            out_files.push(path_name);
        }
    }
    Ok(visited_dirs)
}

fn open_and_initialize_db() -> Result<Connection, sqlite::Error> {
    // TODO: do I need the opening "./" on these path strings? (check everywhere, not just here)
    let connection = sqlite::open("./.frzr/frzr.db")?;
    // If you need to test what happens when this function returns an error, uncomment this:
    // connection.execute("CREATE TABLE CREATE TABLE")?;
    connection.execute(
        "
        CREATE TABLE IF NOT EXISTS schema_version (id INTEGER PRIMARY KEY ASC, version INTEGER);
        ",
    )?;

    let mut latest_version_in_db = 0;
    {
        // TODO Why is this scope necessary?
        let mut statement =
            connection.prepare("SELECT * FROM schema_version ORDER BY version DESC LIMIT 1;")?;
        while let State::Row = statement.next()? {
            latest_version_in_db = statement.read::<i64>(1)?;
        }
    }
    if latest_version_in_db == 0 {
        // Run may grow to include other statistics about the run, like number of files processed
        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS run (
                id INTEGER PRIMARY KEY ASC NOT NULL,
                start_time datetime NOT NULL,
                end_time datetime
                );
            ",
        )?;

        // I'm trying the filename as BLOB instead of string because, at least for Linux,
        // not all valid paths are strings in any single encoding
        // file_name in this case means "relative path to the file, including the filename"

        // Instead of using time, datetime, or date for file_entry, I think I should do run_id
        // A run should be inserted at the top of any checksum calculation, should have a start
        // and an end, and then can be referenced from file_entry. This might be useful

        // Should file_hash be a blob, also? Probably easier to to select on if it is a string
        // That raises the question for file_name, too. Not sure about types here
        connection.execute(
            "
            CREATE TABLE IF NOT EXISTS file_entry (
                id INTEGER PRIMARY KEY ASC NOT NULL,
                file_name BLOB,
                file_hash STRING,
                run_id INTEGER NOT NULL,
                FOREIGN KEY(run_id) REFERENCES run(id)
                );
            ",
        )?;
        latest_version_in_db = 1;
        let mut statement =
            connection.prepare("INSERT INTO schema_version (version) VALUES (?);")?;
        statement.bind(1, latest_version_in_db)?;
        statement.next()?;
    }
    Ok(connection)
}
