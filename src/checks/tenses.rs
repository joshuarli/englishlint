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
    for phrase in ["has been", "have been", "had been"] {
        for sentence in &analysis.sentences {
            for span in occurrences(
                document.source.text_for(sentence.span),
                phrase,
                sentence.span.start.0,
            ) {
                super::emit(
                    out,
                    document,
                    config,
                    RuleId::ComplexTense,
                    span,
                    format!("complex or perfect tense starts with '{}'", phrase),
                    "Use a simple tense.",
                );
            }
        }
    }
}
