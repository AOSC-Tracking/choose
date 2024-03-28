use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ParseError {
    ParseIntError(std::num::ParseIntError),
    ParseRangeError(crate::error::ParseRangeError),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseIntError(e) => f.write_str(&e.to_string()),
            ParseError::ParseRangeError(e) => f.write_str(&e.to_string()),
        }
    }
}

impl Error for ParseError {}
