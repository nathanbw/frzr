use sqlite::Value;
use sqlite::State;

fn main() {
    let connection = sqlite::open("frzr.db").unwrap();
    connection
        .execute(
            "
        CREATE TABLE IF NOT EXISTS schema_version (id INTEGER PRIMARY KEY ASC, version INTEGER);
        ",
        )
        .unwrap();

    println!("Making the statement");
    let mut statement = connection
        .prepare("SELECT * FROM schema_version ORDER BY version DESC LIMIT 1")
        .unwrap();

    // println!("Calling bind on the statement");
    // let result = match statement.bind(1, 1) {
    //     Ok(t) => t,
    //     Err(e) => {
    //         println!("Result:\t{:?}", e);
    //         ()
    //     },
    // };
    // Now, I think result either holds the thing we want, or an empty tuple?

    println!("starting the loop");
    let mut latest_version_in_db = 0;
    while let State::Row = statement.next().unwrap() {
        println!("Got something, about to try to read it:");
        latest_version_in_db = statement.read::<i64>(1).unwrap();
    }
    if latest_version_in_db == 0 {
        println!("DB not initialized yet, let's go ahead and start it at version 1");
        // Run may grow to include other statistics about the run, like number of files processed
        connection
            .execute("
        CREATE TABLE IF NOT EXISTS run (
        id INTEGER PRIMARY KEY ASC,
        start_time datetime NOT NULL,
        finish_time datetime
        );")
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
            .execute("
        CREATE TABLE IF NOT EXISTS file_entry (
        id INTEGER PRIMARY KEY ASC,
        file_name BLOB,
        file_hash STRING,
        run_id INTEGER,
        FOREIGN KEY(run_id) REFERENCES run(id)
        );")
            .unwrap();
        connection
            .execute("INSERT INTO schema_version (version) VALUES (1);")
            .unwrap();
        latest_version_in_db = 1;
    }
    println!("We got version {} of the schema, ready to go!", latest_version_in_db);

    // We can put some values in the DB:
    let fileName = "frzr.db";
    let hash = "DEADBEEF";
    let mut statement = connection
        .prepare("INSERT INTO file_entry (file_name, file_hash) VALUES (?, ?);")
        .unwrap();

    statement.bind(1, fileName.as_bytes()).unwrap();
    statement.bind(2, hash).unwrap();

    while let State::Row = statement.next().unwrap() {}

    // And let's try querying out a file by its name:
    let mut statement = connection
        .prepare("SELECT * FROM file_entry WHERE file_name = ?")
        .unwrap();

    statement.bind(1, "frzr.db".as_bytes()).unwrap();

    while let State::Row = statement.next().unwrap() {
        println!("id = {}", statement.read::<i64>(0).unwrap());
        // Interesting; why can I query frzr.db as a string?
        println!("file_name = {}", statement.read::<String>(1).unwrap());
        println!("file_hash = {}", statement.read::<String>(2).unwrap());
    }
}
