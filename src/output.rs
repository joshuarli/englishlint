use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::RuleId;
use crate::source::SourceFile;
use std::fmt::Write;
use std::path::Path;

pub fn render_file(source: &SourceFile, root: &Path, diagnostics: &[Diagnostic]) -> String {
    let mut output = String::new();
    writeln!(output, "{}:", source.display_path(root)).unwrap();
    for diagnostic in diagnostics {
        let location = diagnostic.location(source);
        writeln!(
            output,
            "  {}:{}:{}",
            location.line, location.column, diagnostic.rule
        )
        .unwrap();
    }
    output
}

pub fn render_rule_summary(rules: &[(RuleId, Severity)]) -> String {
    let mut output = String::from("englishlint: rule summary\n");
    for (rule, severity) in rules {
        let metadata = &rule.definition().metadata;
        writeln!(
            output,
            "  {} [{}] {}: {} Suggestion: {}",
            rule,
            severity_name(*severity),
            metadata.title,
            metadata.description,
            metadata.suggestion
        )
        .unwrap();
    }
    output
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}
