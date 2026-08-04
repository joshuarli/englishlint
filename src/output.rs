use crate::diagnostic::Diagnostic;
use crate::source::SourceFile;
use std::fmt::Write;
use std::path::Path;

pub fn render(source: &SourceFile, root: &Path, diagnostic: &Diagnostic) -> String {
    let location = diagnostic.location(source);
    let mut output = String::new();
    writeln!(
        output,
        "{}:{}:{}: {} [{}] {}",
        source.display_path(root),
        location.line,
        location.column,
        diagnostic.rule,
        severity_name(diagnostic.severity),
        diagnostic.message
    )
    .unwrap();
    if let Some(suggestion) = &diagnostic.suggestion {
        writeln!(output, "  suggestion: {}", suggestion.message).unwrap();
    }
    output
}

fn severity_name(severity: crate::diagnostic::Severity) -> &'static str {
    match severity {
        crate::diagnostic::Severity::Error => "error",
        crate::diagnostic::Severity::Warning => "warning",
        crate::diagnostic::Severity::Info => "info",
    }
}
