use crate::analysis::{occurrences, Analysis};
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;

pub(crate) fn check(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for (bad, replacement) in [
        ("should", "must"),
        ("would", "rewrite the hypothetical condition"),
        ("may", "can"),
        ("might", "can"),
        ("could", "can"),
    ] {
        for sentence in &analysis.sentences {
            let text = document.source.text_for(sentence.span);
            for span in occurrences(text, bad, sentence.span.start.0) {
                let shown = document.source.text_for(span);
                super::emit(
                    out,
                    document,
                    config,
                    RuleId::BannedModal,
                    span,
                    format!(
                        "avoid modal '{}'; use '{}' when that is the intended meaning",
                        shown, replacement
                    ),
                    format!(
                        "Replace '{}' with '{}', or state the condition directly.",
                        shown, replacement
                    ),
                );
            }
        }
    }
}
