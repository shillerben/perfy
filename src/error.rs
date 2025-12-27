use std::io;
use std::fmt::{Debug, Display};

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error { message: message.to_string() }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::new(&e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;