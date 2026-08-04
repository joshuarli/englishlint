use crate::analysis::tokenize;
use crate::analysis::{occurrences, Analysis};
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
    for phrase in ["is ", "are ", "was ", "were ", "be ", "been "] {
        for sentence in &analysis.sentences {
            for span in occurrences(
                document.source.text_for(sentence.span),
                phrase,
                sentence.span.start.0,
            ) {
                let tail = &document.source.text[span.end.0..sentence.span.end.0];
                if let Some(word) = tokenize(tail, span.end.0).first() {
                    if word.text.ends_with("ed") {
                        let construction = document
                            .source
                            .text_for(Span::new(span.start.0, word.span.end.0));
                        super::emit(
                            out,
                            document,
                            config,
                            RuleId::PassiveVoice,
                            span,
                            format!("likely passive construction '{}'", construction),
                            "Name the actor and use an active verb.",
                        );
                    }
                }
            }
        }
    }
}
