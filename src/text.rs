#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextMode {
    Descriptive,
    Procedural,
}

impl TextMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "descriptive" | "description" => Some(Self::Descriptive),
            "procedural" | "procedure" | "instructions" => Some(Self::Procedural),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Descriptive => "descriptive",
            Self::Procedural => "procedural",
        }
    }
}
