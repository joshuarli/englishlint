pub mod analysis;
pub mod config;
pub mod diagnostic;
pub mod directives;
pub mod document;
pub mod lint;
pub mod rules;
pub mod source;
pub mod text;

use config::Config;
use document::Document;
use ignore::WalkBuilder;
use std::fs;
use std::path::Path;

pub struct LintedFile {
    pub source: source::SourceFile,
    pub diagnostics: Vec<diagnostic::Diagnostic>,
}

pub fn lint_directory(root: &Path, config: &Config) -> Result<Vec<LintedFile>, String> {
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
        let entry = entry.map_err(|error| format!("walk error: {error}"))?;
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
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let document = Document::parse(path, source, config);
            let diagnostics = lint::lint_document(&document, config);
            Ok(LintedFile {
                source: document.source,
                diagnostics,
            })
        })
        .collect()
}
