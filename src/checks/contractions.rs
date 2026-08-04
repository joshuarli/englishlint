use crate::analysis::Analysis;
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
    let special = [
        "it's", "that's", "what's", "who's", "there's", "here's", "let's",
    ];
    for word in &analysis.words {
        let lower = word.text.to_ascii_lowercase();
        if (lower.contains('\'') && !lower.ends_with("'s")) || special.contains(&lower.as_str()) {
            super::emit(
                out,
                document,
                config,
                RuleId::Contraction,
                word.span,
                format!("contraction '{}' is not allowed", word.text),
                "Write the complete form.",
            );
        }
    }
}
