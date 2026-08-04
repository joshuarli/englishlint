use crate::analysis::Analysis;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;

pub(crate) fn check(
    _analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for (term, replacement) in [
        ("colour", "color"),
        ("behaviour", "behavior"),
        ("organisation", "organization"),
        ("optimise", "optimize"),
        ("initialise", "initialize"),
        ("centre", "center"),
        ("analyse", "analyze"),
        ("licence", "license"),
    ] {
        for span in document.occurrences(term) {
            super::emit(
                out,
                document,
                config,
                RuleId::AmericanSpelling,
                span,
                format!("use American spelling '{}'", term),
                format!("Use '{}'.", replacement),
            );
        }
    }
}
