use crate::analysis::Analysis;
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
    for sentence in &analysis.sentences {
        let limit = match sentence.mode {
            TextMode::Descriptive => config.descriptive_limit,
            TextMode::Procedural => config.procedural_limit,
        };
        if sentence.words.len() > limit {
            super::emit(
                out,
                document,
                config,
                RuleId::SentenceLength,
                sentence.span,
                format!(
                    "{} sentence has {} words; maximum is {} for {} text: '{}'",
                    sentence.mode.name(),
                    sentence.words.len(),
                    limit,
                    sentence.mode.name(),
                    super::quote(document.source.text_for(sentence.span))
                ),
                "Split the sentence into two or more sentences.",
            );
        }
    }
}
