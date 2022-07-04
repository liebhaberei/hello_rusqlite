use hello_rusqlite::data::database::Database;
use std::process;

fn main() {
    let db = Database::open().unwrap_or_else(|e| {
        eprintln!("database error: {}", e);
        process::exit(1);
    });

    println!("database connected.");
}
