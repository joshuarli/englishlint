use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;
use crate::source::Span;

pub(crate) mod contractions;
pub(crate) mod filler;
pub(crate) mod ing;
pub(crate) mod latin;
pub(crate) mod modals;
pub(crate) mod multiple_instructions;
pub(crate) mod noun_chains;
pub(crate) mod paragraphs;
pub(crate) mod passive;
pub(crate) mod semicolons;
pub(crate) mod sentence_length;
pub(crate) mod spelling;
pub(crate) mod tenses;
pub(crate) mod terminology;
pub(crate) mod trailing_condition;

pub(crate) fn emit(
    out: &mut Vec<Diagnostic>,
    document: &Document,
    config: &Config,
    rule: RuleId,
    span: Span,
    message: impl Into<String>,
    suggestion: impl Into<String>,
) {
    let line = document.source.location(span.start).line;
    if !document.directives.suppresses(rule, line, config) {
        let mut diagnostic = Diagnostic::new(rule, span, message, suggestion);
        if rule.definition().metadata.heuristic {
            diagnostic.severity = crate::diagnostic::Severity::Warning;
        }
        out.push(diagnostic);
    }
}

pub(crate) fn quote(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() > 100 {
        format!("{}…", compact.chars().take(97).collect::<String>())
    } else {
        compact
    }
}
