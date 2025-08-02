use args::CONFIG;
use error::AppError;
use log::warn;
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RemoveKind, RenameMode},
};
use std::{
    ffi::OsStr,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    process::Command,
    sync::{LazyLock, mpsc},
};

mod args;
mod error;
mod walk_dirs;
mod which;

use rayon::{prelude::*, spawn};
use walk_dirs::walk_dirs;

static EXT_SVG: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("svg"));
const INKSCAPE_EXPORT_ARG: &str = "--export-filename";
const INKSCAPE_EXPORT_LATEX_ARG: &str = "--export-latex";

fn main() {
    let _ = &*CONFIG;

    if CONFIG.regenerate {
        if let Err(e) = regenerate_all_aux_files() {
            // TODO: Make a more meaningful error handling
            warn!("{e}");
        }
    }

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).expect("Failed to initialize watcher");

    let recursive_mode = if CONFIG.recursively {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher
        .watch(&CONFIG.watch_dir, recursive_mode)
        .unwrap_or_else(|_| panic!("Failed to watch directory {}", &CONFIG.watch_dir.display()));

    for event_result in rx {
        match event_result {
            Ok(event) => spawn(move || handle_event(&event)),
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }
}

fn handle_event(event: &Event) {
    let Some(path) = event.paths.first() else {
        return;
    };

    if !is_svg_file(path) {
        return;
    }

    match event.kind {
        EventKind::Create(_)
        | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Name(RenameMode::To)) => {
            if let Err(e) = convert_svg_to_pdf_and_tex(path) {
                eprintln!("Failed to convert SVG to PDF/PDF_TEX: {e}");
            }
        }
        EventKind::Remove(RemoveKind::File)
        | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            if let Err(e) = remove_generated_files(path) {
                eprintln!("Cleanup error: {e}");
            }
        }
        _ => {}
    }
}

fn is_svg_file(path: &Path) -> bool {
    path.extension() == Some(&EXT_SVG) && !path.is_dir()
}

struct AuxFiles {
    pub pdf: PathBuf,
    pub pdf_tex: PathBuf,
}

const PDF_EXT: &str = "pdf";
const PDF_TEX_EXT: &str = "pdf_tex";
impl AuxFiles {
    pub fn from_svg(svg_path: &Path) -> Result<Self, AppError> {
        let stem = svg_path
            .file_stem()
            .ok_or(AppError::FailedToRetrieveFileStem)?;
        let stem = stem
            .to_str()
            .ok_or_else(|| AppError::InvalidUTF8(Some(svg_path.to_path_buf())))?;

        let pdf_filename = format!("{}{}.{}", CONFIG.aux_prefix, stem, PDF_EXT);
        let pdf_tex_filename = format!("{}{}.{}", CONFIG.aux_prefix, stem, PDF_TEX_EXT);

        let pdf = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(pdf_filename);
        let pdf_tex = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(pdf_tex_filename);

        Ok(Self { pdf, pdf_tex })
    }
}

fn convert_svg_to_pdf_and_tex(svg_path: &Path) -> Result<bool, AppError> {
    let AuxFiles { pdf, .. } = AuxFiles::from_svg(svg_path)?;

    let pdf_str = pdf.to_str().ok_or(None)?;

    let svg_path_str = svg_path
        .to_str()
        .ok_or_else(|| Some(svg_path.to_path_buf()))?;

    let output = Command::new(&*CONFIG.inkscape_path)
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

fn remove_generated_files(svg_path: &Path) -> Result<(), AppError> {
    let pdf_and_tex = AuxFiles::from_svg(svg_path)?;
    if pdf_and_tex.pdf.exists() {
        fs::remove_file(pdf_and_tex.pdf)?;
    } else {
        warn!("does not exist: {}", pdf_and_tex.pdf.display());
    }
    if pdf_and_tex.pdf_tex.exists() {
        fs::remove_file(pdf_and_tex.pdf_tex)?;
    } else {
        warn!("does not exist: {}", pdf_and_tex.pdf_tex.display());
    }
    Ok(())
}

fn regenerate_all_aux_files() -> Result<(), AppError> {
    let watch_dir = CONFIG.watch_dir.as_path();

    let entries: Vec<DirEntry> = if CONFIG.recursively {
        walk_dirs(watch_dir)?.filter_map(Result::ok).collect()
    } else {
        fs::read_dir(watch_dir)?.filter_map(Result::ok).collect()
    };

    entries.par_iter().for_each(|entry| {
        let path = entry.path();

        // TODO: Handle errors
        if is_svg_file(&path) {
            let _ = convert_svg_to_pdf_and_tex(&path);
        }
    });

    Ok(())
}
