use std::error::Error;
use std::io;
use std::fmt;
use std::result::Result;

use image::error::ImageError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    ImageError(ImageError),
    IOError(io::Error),
    BadImage(String),
    ArgError(String),
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::ImageError(err) =>
                err.fmt(f),
            AppError::IOError(err) =>
                err.fmt(f),
            AppError::BadImage(what) =>
                write!(f, "Unusable image: {}", what),
            AppError::ArgError(what) =>
                write!(f, "Bad argument: {}", what)
        }
    }
}

impl From<ImageError> for AppError {
    fn from(error: ImageError) -> Self {
        Self::ImageError(error)
    }
}
impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::IOError(error)
    }
}
