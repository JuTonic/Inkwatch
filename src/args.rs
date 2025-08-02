use crate::which::{WhichError, which};
use derive_more::From;
use std::{env, path::PathBuf, process, sync::LazyLock};

const INKSCAPE_BIN: &str = "inkscape";

#[derive(Debug, From)]
pub enum ParseError {
    WatchPathIsNotDirectory(PathBuf),
    InkscapePathIsNotFile(PathBuf),
    PathDoesNotExist(PathBuf),
    NotInPath(String),
    #[from]
    WhichError(WhichError),
    MissingArgument(String),
    UnexpectedArgument(String),

    Help,
    Version,
}

pub struct Config {
    pub watch_dir: PathBuf,
    pub inkscape_path: PathBuf,
    pub aux_prefix: String,
    pub regenerate: bool,
    pub recursively: bool,
}

enum PossibleArgs {
    Help,
    Version,
    InkscapePath,
    AuxPrefix,
    DoNotRegenerate,
    NotRecursively,
    Other,
}

impl PossibleArgs {
    pub fn from(arg: &str) -> Self {
        match arg {
            "--help" | "-h" => Self::Help,
            "--version" | "-v" => Self::Version,
            "--inkscape-path" => Self::InkscapePath,
            "--aux-prefix" => Self::AuxPrefix,
            "--do-not-regenerate" => Self::DoNotRegenerate,
            "--not-recursively" => Self::NotRecursively,
            _ => Self::Other,
        }
    }

    pub fn is_not_an_arg(str: &str) -> bool {
        matches!(Self::from(str), Self::Other)
    }
}

impl Config {
    pub fn parse() -> Result<Self, ParseError> {
        let mut args = env::args().skip(1);
        let first = args
            .next()
            .ok_or_else(|| ParseError::MissingArgument("DIR".to_string()))?;

        match PossibleArgs::from(&first) {
            PossibleArgs::Other => {}
            PossibleArgs::Version => return Err(ParseError::Version),
            _ => return Err(ParseError::Help),
        }

        let watch_dir = check_if_dir(PathBuf::from(&first))?;

        let mut inkscape_path = None;
        let mut aux_prefix = String::new();
        let mut regenerate = true;
        let mut recursively = true;

        while let Some(arg) = args.next() {
            match PossibleArgs::from(&arg) {
                PossibleArgs::Help | PossibleArgs::Version => return Err(ParseError::Help),
                PossibleArgs::InkscapePath => {
                    inkscape_path = if let Some(path) = args.next() {
                        if PossibleArgs::is_not_an_arg(&path) {
                            Some(check_if_file(PathBuf::from(path))?)
                        } else {
                            return Err(ParseError::UnexpectedArgument(
                                "--inkscape-path -> [PATH] <-".to_string(),
                            ));
                        }
                    } else {
                        return Err(ParseError::MissingArgument(
                            "--inkscape-path -> [PATH] <-".to_string(),
                        ));
                    }
                }
                PossibleArgs::AuxPrefix => {
                    aux_prefix = if let Some(prefix) = args.next() {
                        if PossibleArgs::is_not_an_arg(&prefix) {
                            prefix
                        } else {
                            return Err(ParseError::UnexpectedArgument(
                                "--aux-prefix -> [PREFIX] <-".to_string(),
                            ));
                        }
                    } else {
                        return Err(ParseError::MissingArgument(
                            "--aux-prefix -> [PREFIX] <-".to_string(),
                        ));
                    }
                }
                PossibleArgs::DoNotRegenerate => regenerate = false,
                PossibleArgs::NotRecursively => recursively = false,
                PossibleArgs::Other => {
                    return Err(ParseError::UnexpectedArgument(format!(
                        "Not a valid argument -> {arg} <-"
                    )));
                }
            }
        }

        let inkscape_path = if let Some(p) = inkscape_path {
            p
        } else {
            let opt = which(INKSCAPE_BIN)?;
            opt.ok_or_else(|| ParseError::NotInPath(INKSCAPE_BIN.to_string()))?
        };

        Ok(Self {
            watch_dir,
            inkscape_path,
            aux_prefix,
            regenerate,
            recursively,
        })
    }
}

fn check_if_dir(path: PathBuf) -> Result<PathBuf, ParseError> {
    if !path.exists() {
        return Err(ParseError::PathDoesNotExist(path));
    }
    if !path.is_dir() {
        return Err(ParseError::WatchPathIsNotDirectory(path));
    }
    Ok(path)
}

// NOTE: does not check executability is it meant to be checked by plugin
fn check_if_file(path: PathBuf) -> Result<PathBuf, ParseError> {
    if !path.exists() {
        return Err(ParseError::PathDoesNotExist(path));
    }
    if !path.is_file() {
        return Err(ParseError::InkscapePathIsNotFile(path));
    }
    Ok(path)
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &str = env!("CARGO_BIN_NAME");
pub static CONFIG: LazyLock<Config> = LazyLock::new(|| match Config::parse() {
    Ok(cfg) => cfg,
    Err(e) => {
        match e {
            ParseError::Help => print_help(),
            ParseError::Version => {
                print_version();
                process::exit(0);
            }
            ParseError::WatchPathIsNotDirectory(path) => {
                eprintln!("Error: watch path is not a directory: {}", path.display());
            }
            ParseError::InkscapePathIsNotFile(path) => {
                eprintln!("Error: Inkscape path is not a file: {}", path.display());
            }
            ParseError::PathDoesNotExist(path) => {
                eprintln!("Error: path does not exist: {}", path.display());
            }
            ParseError::NotInPath(bin) => {
                eprintln!("Error: '{bin}' not found in $PATH");
            }
            ParseError::WhichError(err) => {
                eprintln!("Error while locating '{INKSCAPE_BIN}': {err}");
            }
            ParseError::MissingArgument(arg) => {
                eprintln!("{arg}");
                // print_help();
            }
            ParseError::UnexpectedArgument(arg) => {
                eprintln!("Unexpected argument: {arg}");
                // print_help();
            }
        }

        process::exit(1);
    }
});

// TODO: add normal help message
fn print_help() {
    eprintln!(
        "
{BIN_NAME} [DIR] [OPTIONS]

DIR:
  directory to watch for changes

OPTIONS: 
  --help or -h            Print this message
  --version or -v         Print version
  --inkscape-path [PATH]  Set path to inkscape binary
  --aux-prefix [STR]      Set prefix added to pdf and pdf_tex files
  --do-not-regenerate     Do not regenerate pdf and pdf_tex files at first run
  --not-recursively       Do not watch for changes in subdirectories"
    );
}

fn print_version() {
    println!("{BIN_NAME} {VERSION}");
}
