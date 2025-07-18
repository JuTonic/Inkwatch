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
    io,
    path::{Path, PathBuf},
    process::Command,
    sync::{LazyLock, mpsc},
};

mod args;
mod error;
mod walk_dirs;

use walk_dirs::walk_dirs;

static EXT_SVG: LazyLock<&OsStr> = LazyLock::new(|| OsStr::new("svg"));
const INKSCAPE_EXPORT_ARG: &str = "--export-filename";
const INKSCAPE_EXPORT_LATEX_ARG: &str = "--export-latex";

fn main() {
    let _ = &*CONFIG;

    dbg!(regenerate_all_aux_files());

    let (tx, rx) = mpsc::channel::<Result<Event, notify::Error>>();

    let mut watcher: RecommendedWatcher =
        notify::recommended_watcher(tx).expect("Failed to initialize watcher");

    watcher
        .watch(&*CONFIG.watch_dir, RecursiveMode::Recursive)
        .expect(format!("Failed to watch directory {:?}", &*CONFIG.watch_dir).as_str());

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
        EventKind::Create(_)
        | EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
            if let Err(e) = convert_svg_to_pdf_and_tex(path) {
                eprintln!("Failed to convert SVG to PDF/PDF_TEX: {}", e);
            };
        }
        EventKind::Remove(RemoveKind::File)
        | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
            if let Err(e) = remove_generated_files(path) {
                eprintln!("Cleanup error: {}", e);
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

const HIDE_PREFIX: char = '.';
const PDF_EXT: &str = "pdf";
const PDF_TEX_EXT: &str = "pdf_tex";
impl GeneratedPdfAndTex {
    pub fn from_svg(svg_path: &PathBuf) -> Result<Self, AppError> {
        let stem = svg_path
            .file_stem()
            .ok_or(AppError::FailedToRetrieveFileStem)?;
        let stem = stem
            .to_str()
            .ok_or(AppError::InvalidUTF8(Some(svg_path.clone())))?;

        let mut pdf_filename = format!("{}.{}", stem, PDF_EXT);
        let mut pdf_tex_filename = format!("{}.{}", stem, PDF_TEX_EXT);

        if CONFIG.hide_aux_files {
            pdf_filename.insert(0, HIDE_PREFIX);
            pdf_tex_filename.insert(0, HIDE_PREFIX);
        }

        let pdf = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(pdf_filename);
        let pdf_tex = svg_path
            .parent()
            .ok_or(AppError::FailedToRetrieveParentDir)?
            .join(pdf_tex_filename);

        Ok(GeneratedPdfAndTex { pdf, pdf_tex })
    }
}

fn convert_svg_to_pdf_and_tex(svg_path: &PathBuf) -> Result<bool, AppError> {
    let GeneratedPdfAndTex { pdf, .. } = GeneratedPdfAndTex::from_svg(svg_path)?;

    let pdf_str = pdf.to_str().ok_or(None)?;

    let svg_path_str = svg_path.to_str().ok_or(Some(svg_path.clone()))?;

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

fn regenerate_all_aux_files() -> Result<(), AppError> {
    for res in walk_dirs(CONFIG.watch_dir.as_path())? {
        let path = res?.path();

        if is_svg_file(&path) {
            convert_svg_to_pdf_and_tex(&path.to_path_buf())?;
        }
    }

    return Ok(());
}
