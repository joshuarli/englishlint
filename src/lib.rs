pub mod cli;
pub mod output;

pub mod error;

mod analysis;
pub mod config;
pub mod diagnostic;
mod directives;
mod document;
mod lint;
pub mod rules;
pub mod source;
mod text;

use config::Config;
use diagnostic::Diagnostic;
use document::Document;
use error::LintError;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

pub struct LintedFile {
    pub source: source::SourceFile,
    pub diagnostics: Vec<diagnostic::Diagnostic>,
}

pub fn lint_text(
    path: impl Into<std::path::PathBuf>,
    text: impl Into<String>,
    config: &Config,
) -> (source::SourceFile, Vec<Diagnostic>) {
    let document = Document::parse(path, text.into(), config);
    let diagnostics = lint::lint_document(&document, config);
    (document.source, diagnostics)
}

pub fn lint_directory(root: &Path, config: &Config) -> Result<Vec<LintedFile>, LintError> {
    let mut paths = Vec::new();
    for entry in WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .standard_filters(false)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(false)
        .parents(true)
        .require_git(false)
        .build()
    {
        let entry = entry.map_err(|error| LintError::Walk(error.to_string()))?;
        if entry.file_type().is_some_and(|kind| kind.is_file())
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            paths.push(entry.path().to_path_buf());
        }
    }
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let metadata = fs::metadata(&path).map_err(|error| LintError::Io {
                path: path.clone(),
                source: error,
            })?;
            if metadata.len() > config.max_file_bytes as u64 {
                return Err(LintError::Config(crate::error::ConfigError::FileTooLarge {
                    path: path.clone(),
                    bytes: metadata.len(),
                    limit: config.max_file_bytes,
                }));
            }
            let source = fs::read_to_string(&path).map_err(|error| LintError::Io {
                path: path.clone(),
                source: error,
            })?;
            let (source, diagnostics) = lint_text(path, source, config);
            Ok(LintedFile {
                source,
                diagnostics,
            })
        })
        .collect()
}
