use rusqlite::Connection;
use std::error;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open() -> Result<Database, Box<dyn error::Error>> {
        let connection = Connection::open("kontakte.db")?;

        let db = Database { connection };
        db.initialize()?;

        Ok(db)
    }

    fn initialize(&self) -> Result<(), Box<dyn error::Error>> {
        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS address (
                id INTEGER PRIMARY KEY,
                street TEXT NOT NULL,
                zip TEXT NOT NULL,
                city TEXT NOT NULL,
                phone TEXT
            );",
            []
        )?;

        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS person (
                id INTEGER PRIMARY KEY,
                fist_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                mobile TEXT,
                address INTEGER,
                FOREIGN KEY(address) REFERENCES address(id)
            );",
            []
        )?;

        Ok(())
    }
}
