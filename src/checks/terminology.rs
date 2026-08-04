use crate::analysis::Analysis;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;

pub(crate) fn check(
    _analysis: &Analysis,
    document: &Document,
    config: &Config,
    out: &mut Vec<Diagnostic>,
) {
    for (group, terms) in [
        (
            "check",
            &["check", "verify", "confirm", "validate", "ensure"] as &[&str],
        ),
        (
            "config",
            &["config", "configuration", "settings", "options"],
        ),
        ("delete", &["delete", "remove", "drop", "destroy"]),
    ] {
        let mut occurrences_found = Vec::new();
        for term in terms {
            for span in document.occurrences(term) {
                occurrences_found.push((span, *term));
            }
        }
        occurrences_found.sort_by_key(|(span, _)| span.start);
        let distinct: std::collections::HashSet<&str> =
            occurrences_found.iter().map(|(_, term)| *term).collect();
        if distinct.len() > 1 {
            let preferred = config
                .glossary
                .get(group)
                .map(String::as_str)
                .unwrap_or(occurrences_found[0].1);
            for (span, term) in occurrences_found {
                if term != preferred {
                    let suggestion = if config.glossary.contains_key(group) {
                        format!("Use '{}' for this concept.", preferred)
                    } else {
                        format!("Choose one term; set [glossary] {} = <preferred term> in englishlint.ini.", group)
                    };
                    super::emit(
                        out,
                        document,
                        config,
                        RuleId::TerminologyRotation,
                        span,
                        format!(
                            "terminology rotation: '{}' and '{}' name the same concept",
                            term, preferred
                        ),
                        suggestion,
                    );
                }
            }
        }
    }
}
