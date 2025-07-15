use derive_more::From;
use log::warn;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RemoveKind, RenameMode},
};
use std::{
    env::{self, Args},
    ffi::OsStr,
    fs,
    path::PathBuf,
    process::Command,
    sync::{LazyLock, mpsc},
};

static ARGS: LazyLock<Vec<String>> = LazyLock::new(|| env::args().collect());

static FIGURES_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = ARGS.get(1).expect("You must provide a path to watch");
    let path = PathBuf::from(path);

    if !path.exists() {
        panic!("directory {:?} does not exist", path);
    }
    if !path.is_dir() {
        panic!("{:?} is not a directory", path);
    }
    path
});

static EXT_SVG: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("svg"));

const INKSCAPE_BIN_ARG: &str = "--inkscape-bin";

const INKSCAPE_BIN: LazyLock<String> = LazyLock::new(|| {
    let args_iter = (&*ARGS).iter();

    for arg in args_iter {
        if arg == INKSCAPE_BIN_ARG {}
    }
});
const INKSCAPE_EXPORT_ARG: &str = "--export-filename";
const INKSCAPE_EXPORT_LATEX_ARG: &str = "--export-latex";

#[derive(From, Debug)]
enum AppError {
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

fn main() {
    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).expect("Failed to initialize watcher");

    watcher
        .watch(&*FIGURES_DIR, RecursiveMode::Recursive)
        .expect(format!("Failed to watch directory {:?}", FIGURES_DIR).as_str());

    for event_result in rx {
        match event_result {
            Ok(event) => handle_event(event),
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}

fn handle_event(event: Event) {
    let Some(path) = event.paths.get(0) else {
        return;
    };

    if !is_svg_file(path) {
        return;
    }

    match event.kind {
        EventKind::Create(_) | EventKind::Modify(ModifyKind::Data(_)) => {
            if let Err(e) = convert_svg_to_pdf_and_tex(path) {
                eprintln!("Failed to convert SVG to PDF/PDF_TEX: {}", e);
            };
        }
        EventKind::Remove(RemoveKind::File) => {
            if let Err(e) = remove_generated_files(path) {
                eprintln!("Cleanup error: {}", e);
            }
        }
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            if let Some(new_path) = event.paths.get(1) {
                if let Err(e) = rename_generated_files(path, new_path) {
                    eprintln!("Rename error: {}", e);
                }
            }
        }

        _ => {}
    };
}

fn is_svg_file(path: &PathBuf) -> bool {
    path.extension() == Some(&EXT_SVG) && !path.is_dir()
}

struct GeneratedPdfAndTex {
    pub pdf: PathBuf,
    pub pdf_tex: PathBuf,
}

impl GeneratedPdfAndTex {
    pub fn from_svg(svg_path: &PathBuf) -> Result<Self, AppError> {
        let stem = svg_path
            .file_stem()
            .ok_or(AppError::FailedToRetrieveFileStem)?;
        let stem = stem
            .to_str()
            .ok_or(AppError::InvalidUTF8(Some(svg_path.clone())))?;

        let pdf = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(format!(".{}.pdf", stem));
        let pdf_latex = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(format!(".{}.pdf_tex", stem));

        Ok(GeneratedPdfAndTex {
            pdf,
            pdf_tex: pdf_latex,
        })
    }
}

fn convert_svg_to_pdf_and_tex(svg_path: &PathBuf) -> Result<bool, AppError> {
    let GeneratedPdfAndTex { pdf, .. } = GeneratedPdfAndTex::from_svg(svg_path)?;

    let pdf_str = pdf.to_str().ok_or(None)?;

    let svg_path_str = svg_path.to_str().ok_or(Some(svg_path.clone()))?;

    let output = Command::new(INKSCAPE_CMD)
        .args([
            INKSCAPE_EXPORT_ARG,
            pdf_str,
            INKSCAPE_EXPORT_LATEX_ARG,
            svg_path_str,
        ])
        .output()?;

    if output.status.success() {
        Ok(true)
    } else {
        Err(AppError::InkscapeError(output.stderr))
    }
}

fn remove_generated_files(svg_path: &PathBuf) -> Result<(), AppError> {
    let pdf_and_tex = GeneratedPdfAndTex::from_svg(svg_path)?;
    if pdf_and_tex.pdf.exists() {
        fs::remove_file(pdf_and_tex.pdf)?;
    } else {
        warn!("does not exist: {:?}", pdf_and_tex.pdf);
    }
    if pdf_and_tex.pdf_tex.exists() {
        fs::remove_file(pdf_and_tex.pdf_tex)?;
    } else {
        warn!("does not exist: {:?}", pdf_and_tex.pdf_tex);
    }
    Ok(())
}

fn rename_generated_files(svg_path_old: &PathBuf, svg_path_new: &PathBuf) -> Result<(), AppError> {
    let pdf_and_tex_old = GeneratedPdfAndTex::from_svg(svg_path_old)?;
    let pdf_and_tex_new = GeneratedPdfAndTex::from_svg(svg_path_new)?;
    if pdf_and_tex_old.pdf.exists() {
        fs::rename(pdf_and_tex_old.pdf, pdf_and_tex_new.pdf)?;
    } else {
        warn!("does not exist: {:?}", pdf_and_tex_old.pdf);
    }
    if pdf_and_tex_old.pdf_tex.exists() {
        fs::rename(pdf_and_tex_old.pdf_tex, pdf_and_tex_new.pdf_tex)?;
    } else {
        warn!("does not exist: {:?}", pdf_and_tex_old.pdf_tex);
    }
    Ok(())
}
