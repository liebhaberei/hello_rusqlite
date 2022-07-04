use rusqlite::{params, Connection};
use std::error;

use crate::data::{Address, Error, Person};

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
            [],
        )?;

        self.connection.execute(
            "CREATE TABLE IF NOT EXISTS person (
                id INTEGER PRIMARY KEY,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                mobile TEXT,
                address INTEGER,
                FOREIGN KEY(address) REFERENCES address(id)
            );",
            [],
        )?;

        Ok(())
    }

    fn reset(&self) -> Result<(), Box<dyn error::Error>> {
        self.connection.execute("DROP TABLE address;", [])?;
        self.connection.execute("DROP TABLE person;", [])?;

        self.initialize()?;

        Ok(())
    }

    fn is_valid(s: &str) -> bool {
        s.chars().all(char::is_alphabetic)
    }

    fn insert_person(&self, person: &mut Person) -> Result<i32, Box<dyn error::Error>> {
        match &mut person.address {
            Some(address) => {
                match person.address_id {
                    Some(_) => {
                        self.update_address(address)?;
                    }
                    None => {
                        self.insert_address(address)?;
                    }
                };
                match &person.mobile {
                    Some(mobile) => self.connection.execute(
                        "INSERT INTO person (first_name, last_name, mobile, address) VALUES (?1, ?2, ?3, ?4)",
                        params![person.first_name, person.last_name, mobile, address.id],
                    )?,
                    None => self.connection.execute(
                        "INSERT INTO person (first_name, last_name, address) VALUES (?1, ?2, ?3)",
                        params![person.first_name, person.last_name, address.id],
                    )?,
                }
            }
            None => match &person.mobile {
                Some(mobile) => self.connection.execute(
                    "INSERT INTO person (first_name, last_name, mobile) VALUES (?1, ?2, ?3)",
                    params![person.first_name, person.last_name, mobile],
                )?,
                None => self.connection.execute(
                    "INSERT INTO person (first_name, last_name) VALUES (?1, ?2)",
                    params![person.first_name, person.last_name],
                )?,
            },
        };

        let mut stmt = self.connection.prepare("SELECT last_insert_rowid()")?;
        let mut id = stmt.query([])?;
        match id.next()? {
            Some(id) => {
                let id = id.get(0)?;
                person.id = id;
                Ok(id)
            }
            None => Err(Box::new(Error::NoId)),
        }
    }

    fn get_person_by_id(&self, id: i32) -> Result<Person, Box<dyn error::Error>> {
        let mut stmt = self.connection.prepare(
            "SELECT id, first_name, last_name, mobile, address FROM person WHERE id = ?1",
        )?;
        let mut id = stmt.query_map(params![id], |row| {
            Ok(Person {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                mobile: row.get(3)?,
                address_id: row.get(4)?,
                address: None,
            })
        })?;
        match id.next() {
            Some(person) => {
                let mut person = person?;
                if let Some(id) = person.address_id {
                    let address = self.get_address_by_id(id)?;
                    person.address = Some(address);
                }
                Ok(person)
            }
            None => Err(Box::new(Error::NotFound)),
        }
    }

    fn insert_address(&self, address: &mut Address) -> Result<i32, Box<dyn error::Error>> {
        match &address.phone {
            Some(phone) => self.connection.execute(
                "INSERT INTO address (street, zip, city, phone) VALUES (?1, ?2, ?3, ?4)",
                params![address.street, address.zip, address.city, phone],
            )?,
            None => self.connection.execute(
                "INSERT INTO address (street, zip, city) VALUES (?1, ?2, ?3)",
                params![address.street, address.zip, address.city],
            )?,
        };

        let mut stmt = self.connection.prepare("SELECT last_insert_rowid()")?;
        let mut id = stmt.query([])?;
        match id.next()? {
            Some(id) => {
                let id = id.get(0)?;
                address.id = id;
                Ok(id)
            }
            None => Err(Box::new(Error::NoId)),
        }
    }

    fn get_address_by_id(&self, id: i32) -> Result<Address, Box<dyn error::Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, street, zip, city, phone FROM address WHERE id = ?1")?;
        let mut id = stmt.query_map(params![id], |row| {
            Ok(Address {
                id: row.get(0)?,
                street: row.get(1)?,
                zip: row.get(2)?,
                city: row.get(3)?,
                phone: row.get(4)?,
            })
        })?;
        match id.next() {
            Some(a) => Ok(a?),
            None => Err(Box::new(Error::NotFound)),
        }
    }

    fn get_addresses(&self) -> Result<Vec<Address>, Box<dyn error::Error>> {
        let mut stmt = self
            .connection
            .prepare("SELECT id, street, zip, city, phone FROM address")?;
        let mut addresses = stmt.query_map([], |row| {
            Ok(Address {
                id: row.get(0)?,
                street: row.get(1)?,
                zip: row.get(2)?,
                city: row.get(3)?,
                phone: row.get(4)?,
            })
        })?;

        let mut list: Vec<Address> = Vec::new();
        for address in addresses {
            let address = address?;
            list.push(address);
        }

        Ok(list)
    }

    fn update_address(&self, address: &Address) -> Result<(), Box<dyn error::Error>> {
        match &address.phone {
            Some(phone) => self.connection.execute(
                "UPDATE address SET street = ?1, zip = ?2, city = ?3, phone = ?4 WHERE id = ?5",
                params![address.street, address.zip, address.city, phone, address.id],
            )?,
            None => self.connection.execute(
                "UPDATE address SET street = ?1, zip = ?2, city = ?3, phone = NULL WHERE id = ?4",
                params![address.street, address.zip, address.city, address.id],
            )?,
        };

        Ok(())
    }

    fn delete_address(&self, id: i32) -> Result<(), Box<dyn error::Error>> {
        self.connection
            .execute("DELETE FROM address WHERE ?1", params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use crate::data::{Address, Person};
    use std::error;

    #[test]
    fn insert_person() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut person = Person {
            id: -1,
            first_name: String::from("Max"),
            last_name: String::from("Mustermann"),
            mobile: None,
            address_id: None,
            address: None,
        };

        let id = db.insert_person(&mut person)?;
        assert!(id >= 0);
        assert_eq!(id, person.id);

        Ok(())
    }

    #[test]
    fn insert_person_mobile() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut person = Person {
            id: -1,
            first_name: String::from("Max"),
            last_name: String::from("Mustermann"),
            mobile: Some(String::from("0123456789")),
            address_id: None,
            address: None,
        };

        let id = db.insert_person(&mut person)?;
        assert!(id >= 0);
        assert_eq!(id, person.id);

        Ok(())
    }

    #[test]
    fn insert_person_address() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 1"),
            zip: String::from("00000"),
            city: String::from("Musterstadt"),
            phone: None,
        };

        let mut person = Person {
            id: -1,
            first_name: String::from("Max"),
            last_name: String::from("Mustermann"),
            mobile: None,
            address_id: None,
            address: Some(address),
        };

        let id = db.insert_person(&mut person)?;
        assert!(id >= 0);
        assert_eq!(id, person.id);

        Ok(())
    }

    #[test]
    fn get_person_by_id() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 1"),
            zip: String::from("00000"),
            city: String::from("Musterstadt"),
            phone: None,
        };

        let mut person = Person {
            id: -1,
            first_name: String::from("Max"),
            last_name: String::from("Mustermann"),
            mobile: None,
            address_id: None,
            address: Some(address),
        };

        let id = db.insert_person(&mut person)?;
        assert!(id >= 0);
        assert_eq!(id, person.id);

        let person2 = db.get_person_by_id(id)?;
        assert_eq!(person, person2);

        Ok(())
    }

    #[test]
    fn get_person_by_id_no_address() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut person = Person {
            id: -1,
            first_name: String::from("Max"),
            last_name: String::from("Mustermann"),
            mobile: None,
            address_id: None,
            address: None,
        };

        let id = db.insert_person(&mut person)?;
        assert!(id >= 0);
        assert_eq!(id, person.id);

        let person2 = db.get_person_by_id(id)?;
        assert_eq!(person, person2);

        Ok(())
    }

    #[test]
    fn insert_address() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 1"),
            zip: String::from("00000"),
            city: String::from("Musterstadt"),
            phone: None,
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        Ok(())
    }

    #[test]
    fn insert_address_phone() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 2"),
            zip: String::from("00000"),
            city: String::from("Musterstadt"),
            phone: Some(String::from("01234567890")),
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        Ok(())
    }

    #[test]
    fn get_address_by_id() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 3"),
            zip: String::from("12345"),
            city: String::from("Meinestadt"),
            phone: Some(String::from("098765431")),
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        let address2 = db.get_address_by_id(id)?;

        assert_eq!(address, address2);

        Ok(())
    }

    #[test]
    fn update_address() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 4"),
            zip: String::from("54321"),
            city: String::from("Mycity"),
            phone: None,
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        let address2 = Address {
            street: String::from("Musterstraße 5"),
            zip: String::from("65432"),
            city: String::from("Anothercity"),
            phone: Some(String::from("099998888")),
            ..address
        };

        db.update_address(&address2)?;

        let address3 = db.get_address_by_id(id)?;

        assert_eq!(address2, address3);

        Ok(())
    }

    #[test]
    fn get_addresses_empty() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;
        db.reset()?;

        let addresses = db.get_addresses()?;

        assert!(addresses.len() == 0);

        Ok(())
    }

    #[test]
    fn get_addresses() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;
        db.reset()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 4"),
            zip: String::from("54321"),
            city: String::from("Mycity"),
            phone: None,
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        let mut address2 = Address {
            street: String::from("Musterstraße 5"),
            zip: String::from("65432"),
            city: String::from("Anothercity"),
            phone: Some(String::from("099998888")),
            ..address
        };

        let id = db.insert_address(&mut address2)?;
        assert!(id >= 0);
        assert_eq!(id, address2.id);

        let addresses = db.get_addresses()?;

        assert!(addresses.len() == 2);
        assert_eq!(addresses[0], address);
        assert_eq!(addresses[1], address2);

        Ok(())
    }

    #[test]
    fn delete_address() -> Result<(), Box<dyn error::Error>> {
        let db = Database::open()?;

        let mut address = Address {
            id: -1,
            street: String::from("Musterstraße 4"),
            zip: String::from("54321"),
            city: String::from("Mycity"),
            phone: None,
        };

        let id = db.insert_address(&mut address)?;
        assert!(id >= 0);
        assert_eq!(id, address.id);

        db.delete_address(id)?;

        match db.get_address_by_id(id) {
            Ok(_) => panic!("address wasn't deleted!"),
            Err(_) => Ok(()),
        }
    }
}
