//! Functions for reading and writing various file formats.

pub mod cube;
pub mod spi1d;

#[derive(Debug)]
pub enum ReadError {
    IoErr(std::io::Error),
    FormatErr,
}

impl From<std::io::Error> for ReadError {
    fn from(error: std::io::Error) -> Self {
        ReadError::IoErr(error)
    }
}

impl From<std::num::ParseFloatError> for ReadError {
    fn from(_error: std::num::ParseFloatError) -> ReadError {
        ReadError::FormatErr
    }
}

impl From<std::num::ParseIntError> for ReadError {
    fn from(_error: std::num::ParseIntError) -> ReadError {
        ReadError::FormatErr
    }
}
