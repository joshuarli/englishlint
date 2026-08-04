use crate::analysis::Analysis;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;
use crate::source::Span;

pub(crate) fn check(
    analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for sentence in &analysis.sentences {
        let text = document.source.text_for(sentence.span);
        for (offset, character) in text.char_indices() {
            if character == ',' {
                let tail = text[offset + 1..].trim_start();
                if let Some(word) = tail.split_whitespace().next() {
                    if word.ends_with("ing") && word != "including" {
                        let span = Span::new(
                            sentence.span.start.0 + offset,
                            sentence.span.start.0 + offset + 1,
                        );
                        super::emit(
                            out,
                            document,
                            config,
                            RuleId::IngClause,
                            span,
                            format!("dangling '-ing' clause '{}...'", super::quote(tail)),
                            "Write the action as a separate sentence.",
                        );
                    }
                }
            }
        }
    }
}
