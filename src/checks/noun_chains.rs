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
    let stop = [
        "a", "an", "the", "of", "for", "on", "in", "to", "with", "and", "or", "that", "is", "are",
        "can", "will", "must", "from", "by", "as", "at",
    ];
    for window in analysis.words.windows(5) {
        let between = &document.source.text[window[0].span.end.0..window[4].span.start.0];
        if ["a", "an", "the"].contains(&window[0].text.to_ascii_lowercase().as_str())
            && !between.contains(['.', '!', '?', ':', ',', '`', '[', ']'])
            && window[1..]
                .iter()
                .all(|word| !stop.contains(&word.text.to_ascii_lowercase().as_str()))
        {
            let span = Span::new(window[1].span.start.0, window[4].span.end.0);
            super::emit(
                out,
                document,
                config,
                RuleId::LongNounChain,
                span,
                format!(
                    "technical noun chain has at least four words: '{}'",
                    super::quote(
                        document
                            .source
                            .text_for(Span::new(window[0].span.start.0, window[4].span.end.0))
                    )
                ),
                "Break the noun chain with a preposition such as 'of' or 'for'.",
            );
            break;
        }
    }
}
