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
    let terms = [
        ("simply", "delete it or state the fact"),
        ("just", "delete it or state the fact"),
        ("seamlessly", "delete it or describe the behavior"),
        ("effortlessly", "delete it or describe the behavior"),
        ("robust", "give the measurable property"),
        ("powerful", "give the measurable property"),
        ("comprehensive", "name what it includes"),
        ("leverage", "use"),
        ("utilize", "use"),
        ("in order to", "to"),
        ("prior to", "before"),
        ("facilitate", "help"),
        ("functionality", "function or feature"),
        ("streamline", "make simpler"),
        ("performant", "give a measurement"),
        ("crucial", "state the fact"),
        ("pivotal", "state the fact"),
        ("delve", "examine"),
        ("myriad", "many"),
        ("plethora", "many"),
        ("etc", "name the items"),
        ("ensure", "make sure that"),
        ("out of the box", "by default"),
    ];
    for word in &analysis.words {
        let lower = word.text.to_ascii_lowercase();
        for (term, replacement) in terms {
            if lower == term && !config.ignored_words.contains(term) {
                super::emit(
                    out,
                    document,
                    config,
                    RuleId::FillerWord,
                    word.span,
                    format!("replace filler or vague word '{}'", word.text),
                    format!("Use '{}'.", replacement),
                );
            }
        }
    }
}
