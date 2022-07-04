use rusqlite::{params, Connection};
use std::error;
use std::fmt;

pub mod database;

#[derive(Debug)]
enum Error {
    InvalidData,
    NoId,
    NotFound,
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidData => write!(f, "InvalidData"),
            Error::NoId => write!(f, "NoID"),
            Error::NotFound => write!(f, "NotFound"),
        }
    }
}

#[derive(Debug)]
pub struct Address {
    id: i32,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub phone: Option<String>,
}

impl PartialEq for Address {
    fn eq(&self, other: &Address) -> bool {
        self.id == other.id
            && self.street == other.street
            && self.zip == other.zip
            && self.city == other.city
            && self.phone == other.phone
    }
}

#[derive(Debug)]
pub struct Person {
    id: i32,
    pub first_name: String,
    pub last_name: String,
    pub mobile: Option<String>,
    address_id: Option<i32>,
    pub address: Option<Address>,
}

impl PartialEq for Person {
    fn eq(&self, other: &Person) -> bool {
        self.id == other.id
            && self.first_name == other.first_name
            && self.last_name == other.last_name
            && self.mobile == other.mobile
            && self.address == other.address
    }
}
