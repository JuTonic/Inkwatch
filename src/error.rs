use std::path::PathBuf;

use derive_more::From;

#[derive(From, Debug)]
pub enum AppError {
    InvalidUTF8(Option<PathBuf>),
    Io(std::io::Error),
    FailedToRetrieveFileStem,
    FailedToRetrieveParentDir,
    InkscapeError(Vec<u8>),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::InvalidUTF8(Some(path)) => {
                write!(f, "Invalid UTF-8 sequence in file: {:?}", path)
            }
            AppError::InvalidUTF8(None) => {
                write!(f, "Invalid UTF-8 sequence in an unknown file")
            }
            AppError::Io(err) => {
                write!(f, "I/O error: {}", err)
            }
            AppError::FailedToRetrieveFileStem => {
                write!(f, "Failed to retrieve file stem from path")
            }
            AppError::FailedToRetrieveParentDir => {
                write!(f, "Failed to retrieve parent directory from path")
            }
            AppError::InkscapeError(output) => {
                // Attempt to convert output to UTF-8 if possible
                match std::str::from_utf8(output) {
                    Ok(s) => write!(f, "Inkscape error: {}", s.trim()),
                    Err(_) => write!(f, "Inkscape error with non-UTF8 output: {:?}", output),
                }
            }
        }
    }
}
