//! Stable rule identifiers and the public rule catalog.

use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleId {
    SentenceLength,
    Contraction,
    BannedModal,
    Semicolon,
    LatinAbbreviation,
    FillerWord,
    TrailingCondition,
    ComplexTense,
    IngClause,
    TerminologyRotation,
    LongNounChain,
    ParagraphLength,
    MultipleInstructions,
    PassiveVoice,
    AmericanSpelling,
}

impl RuleId {
    pub const ALL: [Self; 15] = [
        Self::SentenceLength,
        Self::Contraction,
        Self::BannedModal,
        Self::Semicolon,
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
        match value.trim().to_ascii_uppercase().as_str() {
            "ENG001" => Some(Self::SentenceLength),
            "ENG002" => Some(Self::Contraction),
            "ENG003" => Some(Self::BannedModal),
            "ENG004" => Some(Self::Semicolon),
            "ENG005" => Some(Self::LatinAbbreviation),
            "ENG006" => Some(Self::FillerWord),
            "ENG007" => Some(Self::TrailingCondition),
            "ENG008" => Some(Self::ComplexTense),
            "ENG009" => Some(Self::IngClause),
            "ENG010" => Some(Self::TerminologyRotation),
            "ENG011" => Some(Self::LongNounChain),
            "ENG012" => Some(Self::ParagraphLength),
            "ENG013" => Some(Self::MultipleInstructions),
            "ENG014" => Some(Self::PassiveVoice),
            "ENG015" => Some(Self::AmericanSpelling),
            _ => None,
        }
    }

    pub fn metadata(self) -> Rule {
        match self {
            Self::SentenceLength => Rule::new(
                self,
                "Sentence length",
                "A sentence exceeds the configured word limit.",
                "Split the sentence into two or more sentences.",
                false,
            ),
            Self::Contraction => Rule::new(
                self,
                "Contraction",
                "A contraction appears in visible prose.",
                "Write the complete form.",
                false,
            ),
            Self::BannedModal => Rule::new(
                self,
                "Banned modal",
                "The prose uses should, would, may, might, or could.",
                "Use must or can when that is the intended meaning.",
                false,
            ),
            Self::Semicolon => Rule::new(
                self,
                "Semicolon",
                "A semicolon joins independent sentences.",
                "Replace the semicolon with a period.",
                false,
            ),
            Self::LatinAbbreviation => Rule::new(
                self,
                "Latin abbreviation",
                "The prose uses e.g., i.e., or etc.",
                "Use plain English or name the remaining items.",
                false,
            ),
            Self::FillerWord => Rule::new(
                self,
                "Filler or vague wording",
                "The prose uses filler, vague, or AI-style wording.",
                "Delete the filler or state a measurable fact.",
                false,
            ),
            Self::TrailingCondition => Rule::new(
                self,
                "Trailing condition",
                "A procedural condition appears after the command.",
                "Put the condition before the command.",
                true,
            ),
            Self::ComplexTense => Rule::new(
                self,
                "Complex or perfect tense",
                "The prose appears to use a complex or perfect tense.",
                "Use a simple tense.",
                true,
            ),
            Self::IngClause => Rule::new(
                self,
                "Dangling -ing clause",
                "A comma is followed by a likely participial -ing clause.",
                "Write the action as a separate sentence.",
                true,
            ),
            Self::TerminologyRotation => Rule::new(
                self,
                "Terminology rotation",
                "Multiple terms appear for one configured concept.",
                "Choose one term and add it to the project glossary.",
                true,
            ),
            Self::LongNounChain => Rule::new(
                self,
                "Long noun chain",
                "A likely technical noun chain contains at least four words.",
                "Break the noun chain with a preposition.",
                true,
            ),
            Self::ParagraphLength => Rule::new(
                self,
                "Paragraph length",
                "A paragraph contains more than six sentences.",
                "Split the paragraph around its topics.",
                false,
            ),
            Self::MultipleInstructions => Rule::new(
                self,
                "Multiple instructions",
                "A procedural sentence appears to contain multiple instructions.",
                "Write one instruction per sentence.",
                true,
            ),
            Self::PassiveVoice => Rule::new(
                self,
                "Likely passive voice",
                "The prose appears to use a passive construction.",
                "Name the actor and use an active verb.",
                true,
            ),
            Self::AmericanSpelling => Rule::new(
                self,
                "American spelling",
                "The prose uses a British spelling variant.",
                "Use the American spelling.",
                true,
            ),
        }
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rule {
    pub id: RuleId,
    pub title: &'static str,
    pub description: &'static str,
    pub suggestion: &'static str,
    pub heuristic: bool,
}

impl Rule {
    const fn new(
        id: RuleId,
        title: &'static str,
        description: &'static str,
        suggestion: &'static str,
        heuristic: bool,
    ) -> Self {
        Self {
            id,
            title,
            description,
            suggestion,
            heuristic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_round_trip_and_metadata_is_complete() {
        for (index, id) in RuleId::ALL.iter().enumerate() {
            let text = id.to_string();
            assert_eq!(RuleId::parse(&text), Some(*id));
            assert_eq!(*id as usize, index);
            assert!(!id.metadata().title.is_empty());
        }
        assert!(RuleId::parse("ENG999").is_none());
    }
}
