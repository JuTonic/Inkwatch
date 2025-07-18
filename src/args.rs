use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;

use std::env;

const INKSCAPE_BIN: &str = "inkscape";

#[derive(Parser)]
#[command(
    name = "inkfigd",
    author,
    version,
    about = "Watches a directory for .svg changes and exports via Inkscape to pdf_tex"
)]
struct Args {
    /// Directory to watch for SVG files
    #[arg(value_name = "WATCH_DIR")]
    watch_dir: PathBuf,

    /// Path to Inkscape binary
    #[arg(long, value_name = "PATH")]
    inkscape_path: Option<PathBuf>,

    #[arg(long)]
    hide_aux_files: bool,
}

pub struct Config {
    pub watch_dir: PathBuf,
    pub inkscape_path: PathBuf,
    pub hide_aux_files: bool,
}

fn check_if_dir(path: PathBuf) -> Result<PathBuf, ValidateError> {
    if !path.exists() {
        return Err(ValidateError::DoesNotExist(path));
    }
    if !path.is_dir() {
        return Err(ValidateError::IsNotDirectory(path));
    }
    Ok(path)
}

fn check_if_file(path: PathBuf) -> Result<PathBuf, ValidateError> {
    if !path.exists() {
        return Err(ValidateError::DoesNotExist(path));
    }
    if !path.is_file() {
        return Err(ValidateError::IsNotFile(path));
    }
    Ok(path)
}

impl Args {
    fn validate(self) -> Result<Config, ValidateError> {
        let watch_dir = check_if_dir(self.watch_dir)?;

        let inkscape_bin = self.inkscape_path;
        let inkscape_bin = if let Some(path) = inkscape_bin {
            check_if_file(path)?
        } else {
            match which::which(INKSCAPE_BIN) {
                Ok(path) => path,
                Err(e) => return Err(ValidateError::NotInPath(e)),
            }
        };

        Ok(Config {
            watch_dir,
            inkscape_path: inkscape_bin,
            hide_aux_files: self.hide_aux_files,
        })
    }
}

#[derive(Debug)]
enum ValidateError {
    IsNotDirectory(PathBuf),
    IsNotFile(PathBuf),
    DoesNotExist(PathBuf),
    NotInPath(which::Error),
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Args::parse()
        .validate()
        .expect("failed to validate arguments")
});
