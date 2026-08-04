use crate::analysis::Analysis;
use crate::config::Config;
use crate::diagnostic::Diagnostic;
use crate::document::Document;
use crate::rules::RuleId;

#[path = "checks/mod.rs"]
pub(crate) mod checks;

pub fn lint_document(document: &Document, config: &Config) -> Vec<Diagnostic> {
    let analysis = Analysis::build(document);
    let mut diagnostics = Vec::new();
    for definition in crate::rules::catalog() {
        (definition.check)(&analysis, document, config, &mut diagnostics);
    }
    diagnostics.sort_by_key(|diagnostic| (diagnostic.span.start, diagnostic.rule));
    diagnostics
}

#[allow(dead_code)]
fn _rule_catalog_is_exhaustive() {
    for definition in RuleId::ALL.iter().map(|id| id.definition()) {
        let _ = definition.metadata;
    }
}
