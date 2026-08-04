#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

use crate::rules::RuleId;
use crate::source::{Location, SourceFile, Span};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<Replacement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Replacement {
    pub span: Span,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub rule: RuleId,
    pub severity: Severity,
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
            severity: Severity::Error,
            span,
            message: message.into(),
            suggestion: Some(Suggestion {
                message: suggestion.into(),
                replacement: None,
            }),
        }
    }

    pub fn location(&self, source: &SourceFile) -> Location {
        source.location(self.span.start)
    }
}
