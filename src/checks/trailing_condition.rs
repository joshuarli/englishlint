use crate::analysis::{occurrences, Analysis};
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;
use crate::text::TextMode;

pub(crate) fn check(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for sentence in analysis
        .sentences
        .iter()
        .filter(|sentence| sentence.mode == TextMode::Procedural)
    {
        let text = document.source.text_for(sentence.span);
        for condition in ["if", "when"] {
            let found = occurrences(text, condition, sentence.span.start.0);
            let first_word_end = text
                .char_indices()
                .find(|(_, character)| character.is_whitespace())
                .map(|(index, _)| index)
                .unwrap_or(text.len());
            if found.len() > 1
                || found
                    .first()
                    .is_some_and(|span| span.start.0 > sentence.span.start.0 + first_word_end)
            {
                let span = found.last().copied().unwrap_or(sentence.span);
                super::emit(
                    out,
                    document,
                    config,
                    RuleId::TrailingCondition,
                    span,
                    format!(
                        "put the '{}' condition before the command: '{}'",
                        condition,
                        super::quote(text)
                    ),
                    format!(
                        "Start the sentence with '{}', then write the command.",
                        condition
                    ),
                );
            }
        }
    }
}
