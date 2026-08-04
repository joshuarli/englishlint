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
        ("e.g.", "for example"),
        ("i.e.", "that is"),
        ("etc.", "name the remaining items"),
    ] {
        for span in document.occurrences(term) {
            super::emit(
                out,
                document,
                config,
                RuleId::LatinAbbreviation,
                span,
                format!("avoid Latin abbreviation '{}'", term),
                format!("Use '{}'.", replacement),
            );
        }
    }
}
