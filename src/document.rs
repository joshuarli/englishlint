use crate::config::Config;
use crate::directives::DirectiveSet;
use crate::source::{SourceFile, Span};
use crate::text::TextMode;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug)]
pub struct ProseSpan {
    pub span: Span,
    pub mode: TextMode,
    pub block: usize,
    segment: usize,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub source: SourceFile,
    pub prose: Vec<ProseSpan>,
    pub protected: Vec<Span>,
    pub directives: DirectiveSet,
}

impl Document {
    pub fn parse(path: impl Into<std::path::PathBuf>, source: String, config: &Config) -> Self {
        let source_file = SourceFile::new(path, source);
        let directives = DirectiveSet::parse(&source_file.text);
        let mut prose = Vec::new();
        let mut protected = Vec::new();
        let mut mode = config.default_mode;
        let mut protected_depth = 0usize;
        let mut link_depth = 0usize;
        let mut image_depth = 0usize;
        let mut heading_depth = 0usize;
        let mut heading_text = String::new();
        let mut block = 0usize;
        let mut segment = 0usize;
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        for (event, range) in Parser::new_ext(&source_file.text, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { .. }) => {
                    block += 1;
                    heading_depth += 1;
                    heading_text.clear();
                    segment += 1;
                }
                Event::End(TagEnd::Heading(_)) => {
                    heading_depth = heading_depth.saturating_sub(1);
                    if heading_depth == 0 {
                        mode = heading_mode(&heading_text, config);
                        heading_text.clear();
                    }
                    segment += 1;
                }
                Event::Start(Tag::Paragraph) => {
                    block += 1;
                    segment += 1;
                }
                Event::Start(Tag::Item) | Event::Start(Tag::TableCell) => {
                    block += 1;
                    segment += 1;
                }
                Event::Start(Tag::CodeBlock(_)) | Event::Start(Tag::HtmlBlock) => {
                    protected.push(Span::new(range.start, range.end));
                    protected_depth += 1;
                    segment += 1;
                }
                Event::End(TagEnd::CodeBlock) | Event::End(TagEnd::HtmlBlock) => {
                    protected_depth = protected_depth.saturating_sub(1);
                    segment += 1;
                }
                Event::Start(Tag::Link { .. }) => {
                    link_depth += 1;
                    segment += 1;
                }
                Event::End(TagEnd::Link) => {
                    link_depth = link_depth.saturating_sub(1);
                    segment += 1;
                }
                Event::Start(Tag::Image { .. }) => {
                    protected.push(Span::new(range.start, range.end));
                    image_depth += 1;
                    segment += 1;
                }
                Event::End(TagEnd::Image) => {
                    image_depth = image_depth.saturating_sub(1);
                    segment += 1;
                }
                Event::Text(text) if protected_depth == 0 => {
                    if heading_depth > 0 {
                        heading_text.push_str(&text);
                    } else if image_depth == 0 {
                        let span_mode =
                            mode_at(&source_file.text, range.start, mode, config.default_mode);
                        add_visible_text(
                            &mut prose,
                            &source_file.text,
                            range.start,
                            range.end,
                            span_mode,
                            block,
                            segment,
                        );
                    }
                }
                Event::Code(_)
                | Event::InlineHtml(_)
                | Event::Html(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_) => {
                    protected.push(Span::new(range.start, range.end));
                    segment += 1;
                }
                _ => {}
            }
        }

        let prose = merge_prose(prose);
        Self {
            source: source_file,
            prose,
            protected,
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
            .filter(|span| !self.is_protected(*span))
            .collect()
    }

    pub fn is_protected(&self, span: Span) -> bool {
        self.protected
            .iter()
            .any(|protected| protected.start < span.end && span.start < protected.end)
    }
}

fn add_visible_text(
    prose: &mut Vec<ProseSpan>,
    source: &str,
    start: usize,
    end: usize,
    mode: TextMode,
    block: usize,
    segment: usize,
) {
    let text = &source[start..end];
    let lower = text.to_ascii_lowercase();
    let mut cursor = 0usize;
    let mut piece = segment;
    while cursor < text.len() {
        let next_url = lower[cursor..]
            .find("http://")
            .map(|offset| cursor + offset)
            .or_else(|| {
                lower[cursor..]
                    .find("https://")
                    .map(|offset| cursor + offset)
            });
        let Some(url_start) = next_url else {
            add_span(prose, start + cursor, end, mode, block, piece);
            break;
        };
        add_span(prose, start + cursor, start + url_start, mode, block, piece);
        let url_end = text[url_start..]
            .find(|character: char| {
                character.is_whitespace() || [')', '>', '"', '\''].contains(&character)
            })
            .map(|offset| url_start + offset)
            .unwrap_or(text.len());
        cursor = url_end.max(url_start + 1);
        piece = piece.saturating_add(1);
    }
}

fn add_span(
    prose: &mut Vec<ProseSpan>,
    start: usize,
    end: usize,
    mode: TextMode,
    block: usize,
    segment: usize,
) {
    if start < end {
        prose.push(ProseSpan {
            span: Span::new(start, end),
            mode,
            block,
            segment,
        });
    }
}

fn merge_prose(mut spans: Vec<ProseSpan>) -> Vec<ProseSpan> {
    let mut merged = Vec::new();
    for span in spans.drain(..) {
        let can_merge = merged.last().is_some_and(|previous: &ProseSpan| {
            previous.mode == span.mode
                && previous.block == span.block
                && previous.segment == span.segment
        });
        if can_merge {
            merged.last_mut().unwrap().span.end = span.span.end;
        } else {
            merged.push(span);
        }
    }
    merged
}

fn mode_at(source: &str, offset: usize, current: TextMode, default: TextMode) -> TextMode {
    let prefix = &source[..offset.min(source.len())];
    let procedural = prefix.rfind("<!-- englishlint: procedural");
    let descriptive = prefix.rfind("<!-- englishlint: descriptive");
    match (procedural, descriptive) {
        (Some(procedural), Some(descriptive)) if procedural > descriptive => TextMode::Procedural,
        (Some(_), None) => TextMode::Procedural,
        (None, Some(_)) => TextMode::Descriptive,
        _ if prefix.trim().is_empty() => default,
        _ => current,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pulldown_excludes_code_images_and_plain_urls_but_keeps_link_text() {
        let document = Document::parse("test.md", "# Install\n\nRun the **service**.\n\n```\nshould\n```\n\n[should](https://example.com/should) ![may](image.png)\n\nRead https://example.com/should before you continue.\n".into(), &Config::default());
        let visible: String = document
            .prose
            .iter()
            .filter(|span| !document.is_protected(span.span))
            .map(|span| document.source.text_for(span.span))
            .collect();
        assert!(visible.contains("Run the **service**."));
        assert!(visible.contains("should"));
        assert!(visible.contains("Read "));
        assert!(!visible.contains("https://"));
        assert!(document
            .protected
            .iter()
            .any(|span| document.source.text_for(*span).contains("should")));
    }
    #[test]
    fn preserves_prose_source_offsets() {
        let document = Document::parse("test.md", "A café works.\n".into(), &Config::default());
        let span = document.prose.first().unwrap().span;
        assert_eq!(document.source.text_for(span), "A café works.");
    }
    #[test]
    fn keeps_text_events_in_one_formatted_sentence() {
        let document = Document::parse(
            "test.md",
            "Use **one** setting, then save it.\n".into(),
            &Config::default(),
        );
        assert_eq!(document.prose.len(), 1);
        assert_eq!(
            document.source.text_for(document.prose[0].span),
            "Use **one** setting, then save it."
        );
    }
}
