use derive_more::From;
use log::warn;
use std::{env, ffi::OsStr, fmt, fs, io, path::PathBuf};

#[derive(From, Debug)]
pub enum WhichError {
    VarError(env::VarError),
    Io(io::Error),
}

impl fmt::Display for WhichError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarError(e) => write!(f, "Environment variable error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

const PATH: &str = "PATH";
pub fn which(file_name: &str) -> Result<Option<PathBuf>, WhichError> {
    let file_path = PathBuf::from(file_name);
    if file_path.is_file() {
        return Ok(Some(file_path));
    }

    let paths = env::var(PATH)?;

    for path_dir in std::env::split_paths(paths.as_str()) {
        for result in fs::read_dir(path_dir)? {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() && path.file_name() == Some(OsStr::new(file_name)) {
                        return Ok(Some(path));
                    }
                }
                Err(err) => {
                    warn!("Failed to read file in path: {err}");
                }
            }
        }
    }

    Ok(None)
}
