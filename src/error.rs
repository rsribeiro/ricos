use core::{fmt, num::ParseIntError};

#[derive(Debug)]
pub enum Error {
    WrongNumberOfArguments(u8),
    ColorArgumentExpected,
    NumericArgumentExpected,
    InvalidCommand,
    ColorParseError
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error::NumericArgumentExpected
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongNumberOfArguments(n_args) => write!(f, "{} arguments expected.", n_args),
            Self::ColorArgumentExpected => write!(f, "Color name arguments expected."),
            Self::NumericArgumentExpected => write!(f, "Numeric arguments expected."),
            Self::InvalidCommand => write!(f, "Invalid command."),
            Self::ColorParseError => write!(f, "Error parsing color."),
        }
    }
}