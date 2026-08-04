use crate::config::Config;
use crate::directives::DirectiveSet;
use crate::source::{SourceFile, Span};
use crate::text::TextMode;

#[derive(Clone, Debug)]
pub struct ProseSpan {
    pub span: Span,
    pub mode: TextMode,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub source: SourceFile,
    pub prose: Vec<ProseSpan>,
    pub headings: Vec<Span>,
    pub directives: DirectiveSet,
}

impl Document {
    pub fn parse(path: impl Into<std::path::PathBuf>, source: String, config: &Config) -> Self {
        let source_file = SourceFile::new(path, source);
        let directives = DirectiveSet::parse(&source_file.text);
        let mut prose = Vec::new();
        let mut headings = Vec::new();
        let mut mode = config.default_mode;
        let mut front_matter = false;
        let mut fence: Option<char> = None;
        let mut offset = 0;

        for raw_line in source_file.text.split_inclusive('\n') {
            let start = offset;
            offset += raw_line.len();
            let end = offset;
            let line = raw_line.trim_end_matches(['\n', '\r']);
            let trimmed = line.trim();

            if start == 0 && trimmed == "---" {
                front_matter = true;
                continue;
            }
            if front_matter {
                if trimmed == "---" {
                    front_matter = false;
                }
                continue;
            }
            if let Some(marker) = fence {
                if trimmed.starts_with(marker) {
                    fence = None;
                }
                continue;
            }
            if trimmed.starts_with("```") {
                fence = Some('`');
                continue;
            }
            if trimmed.starts_with("~~~") {
                fence = Some('~');
                continue;
            }
            if trimmed.starts_with('#') {
                headings.push(Span::new(start, end));
                mode = heading_mode(trimmed.trim_start_matches('#').trim(), config);
                continue;
            }

            let lower = trimmed.to_ascii_lowercase();
            if lower.contains("<!-- englishlint: procedural") {
                mode = TextMode::Procedural;
            }
            if lower.contains("<!-- englishlint: descriptive") {
                mode = TextMode::Descriptive;
            }
            if trimmed.is_empty() || trimmed.starts_with("<!--") {
                continue;
            }
            if trimmed.starts_with('>') && trimmed.len() == 1 {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.contains("]: ") {
                continue;
            }

            let mut visible_start = start + line.len() - line.trim_start().len();
            if let Some(marker_end) =
                list_marker_end(&source_file.text[visible_start..end.min(source_file.text.len())])
            {
                visible_start += marker_end;
            }
            if visible_start < end {
                for span in visible_segments(&source_file.text, visible_start, end) {
                    if !span.is_empty() {
                        prose.push(ProseSpan { span, mode });
                    }
                }
            }
        }

        if offset < source_file.text.len() {
            let line = &source_file.text[offset..];
            if !line.trim().is_empty() {
                prose.push(ProseSpan {
                    span: Span::new(offset, source_file.text.len()),
                    mode,
                });
            }
        }

        Self {
            source: source_file,
            prose,
            headings,
            directives,
        }
    }

    pub fn occurrences(&self, phrase: &str) -> Vec<Span> {
        self.prose
            .iter()
            .flat_map(|prose| {
                crate::analysis::occurrences(
                    self.source.text_for(prose.span),
                    phrase,
                    prose.span.start.0,
                )
            })
            .collect()
    }
}

fn heading_mode(heading: &str, config: &Config) -> TextMode {
    let heading = heading.to_ascii_lowercase();
    if config
        .procedural_headings
        .iter()
        .any(|candidate| heading.contains(candidate))
    {
        TextMode::Procedural
    } else {
        TextMode::Descriptive
    }
}

fn visible_segments(source: &str, start: usize, end: usize) -> Vec<Span> {
    let text = &source[start..end];
    let mut hidden = Vec::new();
    let mut backtick = None;
    for (offset, character) in text.char_indices() {
        if character == '`' {
            if let Some(begin) = backtick.take() {
                hidden.push((begin, offset + 1));
            } else {
                backtick = Some(offset);
            }
        }
    }
    let lower = text.to_ascii_lowercase();
    let mut cursor = 0;
    while let Some(relative) = lower[cursor..]
        .find("http://")
        .or_else(|| lower[cursor..].find("https://"))
    {
        let begin = cursor + relative;
        let finish = text[begin..]
            .find(|character: char| {
                character.is_whitespace() || [')', '>', '"'].contains(&character)
            })
            .map(|offset| begin + offset)
            .unwrap_or(text.len());
        hidden.push((begin, finish));
        cursor = finish.max(begin + 1);
        if cursor >= text.len() {
            break;
        }
    }
    let mut link_start = None;
    let mut depth = 0;
    for (offset, character) in text.char_indices() {
        if link_start.is_none() && character == ']' && text[offset + 1..].starts_with('(') {
            link_start = Some(offset + 1);
            depth = 0;
        } else if let Some(begin) = link_start {
            if character == '(' {
                depth += 1;
            }
            if character == ')' {
                if depth == 0 {
                    hidden.push((begin, offset + 1));
                    link_start = None;
                } else {
                    depth -= 1;
                }
            }
        }
    }
    hidden.sort_unstable();
    let mut segments = Vec::new();
    let mut cursor = 0;
    for (hidden_start, hidden_end) in hidden {
        let hidden_start = hidden_start.min(text.len());
        let hidden_end = hidden_end.min(text.len());
        if hidden_start > cursor {
            segments.push(Span::new(start + cursor, start + hidden_start));
        }
        cursor = cursor.max(hidden_end);
    }
    if cursor < text.len() {
        segments.push(Span::new(start + cursor, end));
    }
    segments
}

fn list_marker_end(line: &str) -> Option<usize> {
    if line.starts_with(['-', '*', '+']) && line.as_bytes().get(1) == Some(&b' ') {
        Some(2)
    } else {
        let digits = line.bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && line[digits..].starts_with(". ") {
            Some(digits + 2)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_code_front_matter_and_headings_but_keeps_list_prose() {
        let document = Document::parse(
            "test.md",
            "---\ntitle: should\n---\n# Install\n\n- Run the service.\n```\nshould\n```\n".into(),
            &Config::default(),
        );
        assert_eq!(document.prose.len(), 1);
        assert_eq!(document.prose[0].mode, TextMode::Procedural);
        assert_eq!(
            document.source.text_for(document.prose[0].span).trim(),
            "Run the service."
        );
    }
}
