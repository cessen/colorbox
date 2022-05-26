//! Functions for reading and writing various file formats.

pub mod cube_iridas;
pub mod cube_resolve;
pub mod spi1d;

fn filter_non_finite(n: f32) -> f32 {
    if n.is_finite() {
        n
    } else {
        0.0
    }
}

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
