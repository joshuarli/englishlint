//! Stable rule identifiers, metadata, and checker dispatch.

use std::fmt;
use std::str::FromStr;

#[repr(usize)]
#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleId {
    SentenceLength = 0,
    Contraction = 1,
    BannedModal = 2,
    LatinAbbreviation = 4,
    FillerWord = 5,
    TrailingCondition = 6,
    ComplexTense = 7,
    IngClause = 8,
    TerminologyRotation = 9,
    LongNounChain = 10,
    ParagraphLength = 11,
    MultipleInstructions = 12,
    PassiveVoice = 13,
    AmericanSpelling = 14,
}

impl RuleId {
    pub const ALL: [Self; 14] = [
        Self::SentenceLength,
        Self::Contraction,
        Self::BannedModal,
        Self::LatinAbbreviation,
        Self::FillerWord,
        Self::TrailingCondition,
        Self::ComplexTense,
        Self::IngClause,
        Self::TerminologyRotation,
        Self::LongNounChain,
        Self::ParagraphLength,
        Self::MultipleInstructions,
        Self::PassiveVoice,
        Self::AmericanSpelling,
    ];

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|id| id.to_string() == value.trim().to_ascii_uppercase())
    }

    pub(crate) fn definition(self) -> &'static RuleDefinition {
        catalog()
            .iter()
            .find(|definition| definition.metadata.id == self)
            .expect("every rule has a definition")
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ENG{:03}", *self as usize + 1)
    }
}

impl FromStr for RuleId {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value).ok_or(())
    }
}

pub type Checker = fn(
    &crate::analysis::Analysis,
    &crate::document::Document,
    &crate::config::Config,
    &mut Vec<crate::diagnostic::Diagnostic>,
);

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rule {
    pub id: RuleId,
    pub title: &'static str,
    pub description: &'static str,
    pub suggestion: &'static str,
    pub heuristic: bool,
}

pub struct RuleDefinition {
    pub metadata: Rule,
    pub check: Checker,
}

macro_rules! rule {
    ($id:ident, $title:expr, $description:expr, $suggestion:expr, $heuristic:expr, $check:path) => {
        RuleDefinition {
            metadata: Rule {
                id: RuleId::$id,
                title: $title,
                description: $description,
                suggestion: $suggestion,
                heuristic: $heuristic,
            },
            check: $check,
        }
    };
}

static DEFINITIONS: [RuleDefinition; 14] = [
    rule!(
        SentenceLength,
        "Sentence length",
        "A sentence exceeds the configured word limit.",
        "Split the sentence into two or more sentences.",
        false,
        crate::lint::checks::sentence_length::check
    ),
    rule!(
        Contraction,
        "Contraction",
        "A contraction appears in visible prose.",
        "Write the complete form.",
        false,
        crate::lint::checks::contractions::check
    ),
    rule!(
        BannedModal,
        "Banned modal",
        "The prose uses should, would, may, might, or could.",
        "Use must or can when that is the intended meaning.",
        false,
        crate::lint::checks::modals::check
    ),
    rule!(
        LatinAbbreviation,
        "Latin abbreviation",
        "The prose uses e.g., i.e., or etc.",
        "Use plain English or name the remaining items.",
        false,
        crate::lint::checks::latin::check
    ),
    rule!(
        FillerWord,
        "Filler or vague wording",
        "The prose uses filler, vague, or AI-style wording.",
        "Delete the filler or state a measurable fact.",
        false,
        crate::lint::checks::filler::check
    ),
    rule!(
        TrailingCondition,
        "Trailing condition",
        "A procedural condition appears after the command.",
        "Put the condition before the command.",
        true,
        crate::lint::checks::trailing_condition::check
    ),
    rule!(
        ComplexTense,
        "Complex or perfect tense",
        "The prose appears to use a complex or perfect tense.",
        "Use a simple tense.",
        true,
        crate::lint::checks::tenses::check
    ),
    rule!(
        IngClause,
        "Dangling -ing clause",
        "A comma is followed by a likely participial -ing clause.",
        "Write the action as a separate sentence.",
        true,
        crate::lint::checks::ing::check
    ),
    rule!(
        TerminologyRotation,
        "Terminology rotation",
        "Multiple terms appear for one configured concept.",
        "Choose one term and add it to the project glossary.",
        true,
        crate::lint::checks::terminology::check
    ),
    rule!(
        LongNounChain,
        "Long noun chain",
        "A likely technical noun chain contains at least four words.",
        "Break the noun chain with a preposition.",
        true,
        crate::lint::checks::noun_chains::check
    ),
    rule!(
        ParagraphLength,
        "Paragraph length",
        "A paragraph contains more than six sentences.",
        "Split the paragraph around its topics.",
        false,
        crate::lint::checks::paragraphs::check
    ),
    rule!(
        MultipleInstructions,
        "Multiple instructions",
        "A procedural sentence appears to contain multiple instructions.",
        "Write one instruction per sentence.",
        true,
        crate::lint::checks::multiple_instructions::check
    ),
    rule!(
        PassiveVoice,
        "Likely passive voice",
        "The prose appears to use a passive construction.",
        "Name the actor and use an active verb.",
        true,
        crate::lint::checks::passive::check
    ),
    rule!(
        AmericanSpelling,
        "American spelling",
        "The prose uses a British spelling variant.",
        "Use the American spelling.",
        true,
        crate::lint::checks::spelling::check
    ),
];

pub fn catalog() -> &'static [RuleDefinition; 14] {
    &DEFINITIONS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_and_definitions_are_one_to_one() {
        for (index, id) in RuleId::ALL.iter().enumerate() {
            assert_eq!(RuleId::parse(&id.to_string()), Some(*id));
            assert_eq!(catalog()[index].metadata.id, *id);
            assert!(!catalog()[index].metadata.title.is_empty());
        }
        assert!(RuleId::parse("ENG999").is_none());
    }
}
