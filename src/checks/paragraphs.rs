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
    for paragraph in &analysis.paragraphs {
        if paragraph.sentences.len() > 6 {
            super::emit(
                out,
                document,
                config,
                RuleId::ParagraphLength,
                paragraph.span,
                format!(
                    "paragraph has {} sentences; maximum is 6",
                    paragraph.sentences.len()
                ),
                "Split the paragraph around its topics.",
            );
        }
    }
}
