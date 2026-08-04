use crate::analysis::Analysis;
use crate::source::Span;

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
    for prose in &document.prose {
        for (offset, character) in document.source.text_for(prose.span).char_indices() {
            if character == ';' {
                let span = Span::new(prose.span.start.0 + offset, prose.span.start.0 + offset + 1);
                super::emit(
                    out,
                    document,
                    config,
                    RuleId::Semicolon,
                    span,
                    "semicolon joins two sentences",
                    "Replace the semicolon with a period.",
                );
            }
        }
    }
}
