use std::fs;
use std::fs::DirEntry;
use std::io::Error;
use std::io::Read;
use std::path::PathBuf;

use sha2::{Digest, Sha256};
use sha2::digest::generic_array;

use sqlite::Connection;
use sqlite::State;
use sqlite::Value;

use generic_array::GenericArray;

use std::fmt::UpperHex;
use std::os::unix::ffi::OsStrExt;

// TODO: How to not crash when stdout is closed?
fn main() {
    // TODO: return the schema version as well as the connection
    // TODO: Maybe return Option or Result from open_and_initialize_db?
    let mut db = open_and_initialize_db();
    // If we haven't crashed yet, then the db is open and ready for business
    // Now, let's iterate over all the files
    let path_buf: PathBuf = PathBuf::from("src");
    // TODO: Error handling
    let filenames = give_me_the_files(path_buf).unwrap();
    let file_iter = filenames.iter();
    let mut is_this_the_first_file = true;
    let mut current_run_id = 0;
    for filename in file_iter {
        let file_hash = compute_the_hash(filename).unwrap();
        { // TODO: Verify that this scope is needed, and if so, find out why
            if is_this_the_first_file {
                is_this_the_first_file = false;
                { // TODO: Verify that this scope is needed, and if so, find out why
                    db.execute("INSERT INTO run (start_time) VALUES (CURRENT_TIMESTAMP);").unwrap();
                    let mut statement = db
                        .prepare("SELECT id FROM run ORDER BY id DESC LIMIT 1;")
                        .unwrap();
                    while State::Row == statement.next().unwrap() {
                        current_run_id = statement.read::<i64>(0).unwrap();
                    }
                }
            }

            let mut statement = db
                .prepare("\
                    INSERT INTO file_entry (run_id, file_name, file_hash) VALUES (?, ?, ?);\
                ").unwrap();

            statement.bind(1, current_run_id).unwrap();
            // TODO: I want to store the PathBuf in sqlite as raw binary data
            //       1. Is this doing that?
            //       2. Why is it so hard?
            let why_is_this_copy_needed = PathBuf::from(filename);
            let why_is_this_temporary_variable_needed = why_is_this_copy_needed.into_os_string();
            let filename_str = why_is_this_temporary_variable_needed.as_bytes();
            statement.bind(2, filename_str).unwrap();
            // TODO: This feels like a useless roundtrip -- this already was raw bytes, no?
            statement.bind(3, file_hash.as_bytes()).unwrap();
            statement.next().unwrap();
        }
        // If we haven't crashed yet, then the run exists in the DB, the file_entry rows exist in
        // the db, and the run can be marked finished
        { // TODO: Verify that this scope is needed, and if so, find out why
            let mut statement = db
                .prepare("\
                    UPDATE run set end_time = CURRENT_TIMESTAMP WHERE id = ?;\
                ").unwrap();
            statement.bind(1, current_run_id).unwrap();
            statement.next().unwrap();
        }
    }
}

fn compute_the_hash(file: &PathBuf) -> Result<String, Error> {
    // random.dat created with: dd if=/dev/random of=random.dat bs=1M count=4
    // This program was tested during development with: cargo run && sha256sum random.dat
    // and visually comparing the results
    let mut the_file = match fs::File::open(file) {
        Ok(file) => file,
        Err(e) => {
            println!("An error occurred: {}", e);
            return Err(e);
        }
    };

    let mut hasher = Sha256::new();

    let mut buf: [u8; 4096] = [0; 4096]; // Read 4k at a time
    loop {
        let num_bytes_read = match the_file.read(&mut buf) {
            Ok(thing) => thing,
            Err(e) => {
                println!("An error occurred: {}", e);
                return Err(e);
            }
        };
        if num_bytes_read == 0 {
            break;
        }
        // read bytes from the file, pass them to the hasher:
        hasher.update(&buf[..num_bytes_read]);
    }

    let result : String = format!("{:x}", hasher.finalize());
    Ok(result)
}

fn give_me_the_files(path_string: PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut all_the_files: Vec<PathBuf> = Vec::new();
    let visited_dirs: Vec<DirEntry> = Vec::new();
    let result = dir_walk_recurser(path_string, visited_dirs, &mut all_the_files);
    match result {
        Ok(_) => Ok(all_the_files),
        Err(e) => Err(e),
    }
}

fn dir_walk_recurser(
    path_string: PathBuf,
    mut visited_dirs: Vec<DirEntry>,
    out_files: &mut Vec<PathBuf>,
) -> Result<Vec<DirEntry>, Error> {
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
            visited_dirs = match dir_walk_recurser(path_name, visited_dirs, out_files) {
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

fn open_and_initialize_db() -> Connection {
    let connection = sqlite::open("frzr.db").unwrap();
    connection
        .execute(
            "
        CREATE TABLE IF NOT EXISTS schema_version (id INTEGER PRIMARY KEY ASC, version INTEGER);
        ",
        )
        .unwrap();

    let mut latest_version_in_db = 0;
    {
        let mut statement = connection
            .prepare("SELECT * FROM schema_version ORDER BY version DESC LIMIT 1;")
            .unwrap();
        while let State::Row = statement.next().unwrap() {
            latest_version_in_db = statement.read::<i64>(1).unwrap();
        }
    }
    if latest_version_in_db == 0 {
        // Run may grow to include other statistics about the run, like number of files processed
        connection
            .execute(
                "
            CREATE TABLE IF NOT EXISTS run (
                id INTEGER PRIMARY KEY ASC NOT NULL,
                start_time datetime NOT NULL,
                end_time datetime
                );
            ",
            )
            .unwrap();

        // I'm trying the filename as BLOB instead of string because, at least for Linux,
        // not all valid paths are strings in any single encoding
        // file_name in this case means "relative path to the file, including the filename"

        // Instead of using time, datetime, or date for file_entry, I think I should do run_id
        // A run should be inserted at the top of any checksum calculation, should have a start
        // and an end, and then can be referenced from file_entry. This might be useful

        // Should file_hash be a blob, also? Probably easier to to select on if it is a string
        // That raises the question for file_name, too. Not sure about types here
        connection
            .execute(
                "
            CREATE TABLE IF NOT EXISTS file_entry (
                id INTEGER PRIMARY KEY ASC NOT NULL,
                file_name BLOB,
                file_hash STRING,
                run_id INTEGER NOT NULL,
                FOREIGN KEY(run_id) REFERENCES run(id)
                );
            ",
            )
            .unwrap();
        connection
            .execute("INSERT INTO schema_version (version) VALUES (1);")
            .unwrap();
        latest_version_in_db = 1;
    }
    connection
}
