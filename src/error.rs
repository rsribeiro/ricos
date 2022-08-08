use core::{fmt, num::ParseIntError};
#[cfg(feature="acpi-feat")]
use aml::AmlError as AmlCrateError;
#[cfg(feature="acpi-feat")]
use acpi::AcpiError as AcpiCrateError;

#[derive(Debug)]
pub enum Error {
    WrongNumberOfArguments(u8),
    ColorArgumentExpected,
    NumericArgumentExpected,
    InvalidCommand,
    ColorParseError,
    #[cfg(feature="acpi-feat")]
    AcpiError(AcpiCrateError),
    #[cfg(feature="acpi-feat")]
    AmlError(AmlCrateError)
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error::NumericArgumentExpected
    }
}

#[cfg(feature="acpi-feat")]
impl From<AcpiCrateError> for Error {
    fn from(err: AcpiCrateError) -> Self {
        Error::AcpiError(err)
    }
}
#[cfg(feature="acpi-feat")]
impl From<AmlCrateError> for Error {
    fn from(err: AmlCrateError) -> Self {
        Error::AmlError(err)
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
            #[cfg(feature="acpi-feat")]
            Self::AcpiError(err) => write!(f, "AcpiError: {:?}", err),
            #[cfg(feature="acpi-feat")]
            Self::AmlError(_err) => write!(f, "AmlError")
        }
    }
}
