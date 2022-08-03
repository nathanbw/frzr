use sqlite::State;
use sqlite::Value;

fn main() {
    // First example from the docs
    let connection = sqlite::open(":memory:").unwrap();

    connection
        .execute(
            "
        CREATE TABLE users (name TEXT, age INTEGER);
        INSERT INTO users VALUES ('Alice', 42);
        INSERT INTO users VALUES ('Bob', 69);
        ",
        )
        .unwrap();

    connection
        .iterate("SELECT * FROM users WHERE age > 50", |pairs| {
            for &(column, value) in pairs.iter() {
                println!("{} = {}", column, value.unwrap());
            }
            true
        })
        .unwrap();
    // End first example from the docs

    // Second example from the docs
    let mut statement = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap();

    statement.bind(1, 50).unwrap();

    while let State::Row = statement.next().unwrap() {
        println!("name = {}", statement.read::<String>(0).unwrap());
        println!("age = {}", statement.read::<i64>(1).unwrap());
    }
    // End second example from the docs

    // Third example from the docs
    let mut cursor = connection
        .prepare("SELECT * FROM users WHERE age > ?")
        .unwrap()
        .into_cursor();

    cursor.bind(&[Value::Integer(50)]).unwrap();

    while let Some(row) = cursor.next().unwrap() {
        println!("name = {}", row[0].as_string().unwrap());
        println!("age = {}", row[1].as_integer().unwrap());
    }
    // End third example from the docs
}
