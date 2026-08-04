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
    let actions = [
        "run",
        "set",
        "open",
        "close",
        "make",
        "check",
        "verify",
        "confirm",
        "validate",
        "ensure",
        "install",
        "configure",
        "create",
        "delete",
        "remove",
        "read",
        "write",
        "add",
        "enable",
        "disable",
        "copy",
        "update",
        "select",
        "enter",
        "use",
        "press",
        "replace",
        "save",
        "increase",
        "decrease",
    ];
    for sentence in analysis
        .sentences
        .iter()
        .filter(|sentence| sentence.mode == TextMode::Procedural)
    {
        let text = document.source.text_for(sentence.span);
        let action_count = sentence
            .words
            .iter()
            .filter(|word| actions.contains(&word.text.to_ascii_lowercase().as_str()))
            .count();
        if action_count > 1 && (text.contains(" and ") || text.contains(',')) {
            super::emit(
                out,
                document,
                config,
                RuleId::MultipleInstructions,
                sentence.span,
                format!(
                    "sentence contains multiple instructions: '{}'",
                    super::quote(text)
                ),
                "Write one instruction per sentence.",
            );
        }
    }
}
