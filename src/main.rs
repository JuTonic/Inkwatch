use derive_more::From;
use inotify::{EventMask, Inotify, WatchMask};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

static WATCHING_DIR: LazyLock<String> = LazyLock::new(|| {
    let mut args = env::args();
    args.next();

    let raw_path = args.next().expect("You must provide a pth to watch");

    let path = Path::new(&raw_path);

    if !path.exists() {
        panic!("directory {} does not exist", raw_path);
    }

    if !path.is_dir() {
        panic!("{} is not a directory", raw_path);
    }

    raw_path
});
static WATCHING_DIR_PATHBUF: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(WATCHING_DIR.as_str()));

static SVG: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("svg"));
static PDF: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("pdf"));
static PDF_LATEX: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("pdf_latex"));

fn main() {
    let mut inotify = Inotify::init().expect("Error while initializing inotify instance");

    inotify
        .watches()
        .add(
            WATCHING_DIR.as_str(),
            WatchMask::CREATE | WatchMask::MODIFY | WatchMask::MOVE | WatchMask::DELETE,
        )
        .expect("Failed to add file watch");

    let mut buffer = [0u8; 1024];

    loop {
        let events = inotify.read_events_blocking(&mut buffer).unwrap();

        for event in events {
            let path = to_absolute_path(event.name.unwrap());

            if !is_svg_file(&path) {
                continue;
            }

            let _ = match event.mask {
                EventMask::CREATE => {
                    println!("created!");
                    svg_to_latex_pdf(&path).expect("failed :(");
                }
                EventMask::MOVED_FROM => println!("file moved"),
                _ => {}
            };
        }
    }
}

fn to_absolute_path(file_path: &OsStr) -> PathBuf {
    let file_path = PathBuf::from(file_path);

    WATCHING_DIR_PATHBUF.join(file_path)
}

fn is_svg_file(file_path: &PathBuf) -> bool {
    return file_path.is_file() && file_path.extension() == Some(&SVG);
}

#[derive(From, Debug)]
enum AppError {
    InvalidUTF8(Option<PathBuf>),
    Io(std::io::Error),
}

fn svg_to_hidden_pdf_path(path: &PathBuf) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;

    let hidden_pdf_filename = format!(".{stem}.pdf");

    let hidden_pdf_path = path.parent()?.join(hidden_pdf_filename);

    Some(hidden_pdf_path)
}

const INKSCAPE_CMD: &'static str = "inkscape";
const INKSCAPE_EXPORT_AS_PDF_ARG: &'static str = "--export-filename";
const INKSCAPE_EXPORT_AS_LATEX_PDF_ARG: &'static str = "--export-latex";

fn svg_to_latex_pdf(path: &PathBuf) -> Result<bool, AppError> {
    let hidden_pdf_path = svg_to_hidden_pdf_path(path).ok_or(Some(path.clone()))?;

    let hidden_pdf_path_str = hidden_pdf_path.to_str().ok_or(None)?;

    let path_str = path.to_str().ok_or(Some(path.clone()))?;

    let output = Command::new(INKSCAPE_CMD)
        .args([
            INKSCAPE_EXPORT_AS_PDF_ARG,
            hidden_pdf_path_str,
            INKSCAPE_EXPORT_AS_LATEX_PDF_ARG,
            path_str,
        ])
        .output()?;

    Ok(output.status.success())
}
