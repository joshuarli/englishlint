use crate::rules::RuleId;
use crate::source::{Location, SourceFile, Span};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Suggestion {
    Message(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub rule: RuleId,
    pub span: Span,
    pub message: String,
    pub suggestion: Option<Suggestion>,
}

impl Diagnostic {
    pub fn new(
        rule: RuleId,
        span: Span,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            span,
            message: message.into(),
            suggestion: Some(Suggestion::Message(suggestion.into())),
        }
    }

    pub fn location(&self, source: &SourceFile) -> Location {
        source.location(self.span.start)
    }
}
