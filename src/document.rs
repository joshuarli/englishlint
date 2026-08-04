use crate::config::Config;
use crate::directives::DirectiveSet;
use crate::source::{SourceFile, Span};
use crate::text::TextMode;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

#[derive(Clone, Debug)]
pub struct ProseSpan {
    pub span: Span,
    pub mode: TextMode,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub source: SourceFile,
    pub prose: Vec<ProseSpan>,
    pub directives: DirectiveSet,
}

impl Document {
    pub fn parse(path: impl Into<std::path::PathBuf>, source: String, config: &Config) -> Self {
        let source_file = SourceFile::new(path, source);
        let directives = DirectiveSet::parse(&source_file.text);
        let mut prose = Vec::new();
        let mut mode = config.default_mode;
        let mut protected_depth = 0usize;
        let mut image_depth = 0usize;
        let mut link_depth = 0usize;
        let mut active_span: Option<(usize, TextMode)> = None;
        let mut heading_depth = 0usize;
        let mut heading_text = String::new();
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);

        for (event, range) in Parser::new_ext(&source_file.text, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { .. }) => {
                    finish_active(&mut prose, &mut active_span, range.start, mode);
                    heading_depth += 1;
                    heading_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    heading_depth = heading_depth.saturating_sub(1);
                    if heading_depth == 0 {
                        mode = heading_mode(&heading_text, config);
                        heading_text.clear();
                    }
                }
                Event::Start(Tag::CodeBlock(_)) | Event::Start(Tag::HtmlBlock) => {
                    finish_active(&mut prose, &mut active_span, range.start, mode);
                    protected_depth += 1;
                }
                Event::End(TagEnd::CodeBlock) | Event::End(TagEnd::HtmlBlock) => {
                    protected_depth = protected_depth.saturating_sub(1);
                }
                Event::Start(Tag::Image { .. }) => {
                    finish_active(&mut prose, &mut active_span, range.start, mode);
                    image_depth += 1;
                }
                Event::End(TagEnd::Image) => image_depth = image_depth.saturating_sub(1),
                Event::Start(Tag::Link { .. }) => {
                    finish_active(&mut prose, &mut active_span, range.start, mode);
                    link_depth += 1;
                }
                Event::End(TagEnd::Link) => link_depth = link_depth.saturating_sub(1),
                Event::Start(Tag::Paragraph)
                    if protected_depth == 0 && image_depth == 0 && link_depth == 0 => {}
                Event::End(TagEnd::Paragraph)
                    if protected_depth == 0 && image_depth == 0 && link_depth == 0 =>
                {
                    finish_active(&mut prose, &mut active_span, range.end, mode);
                }
                Event::Text(text) if protected_depth == 0 => {
                    if heading_depth > 0 {
                        heading_text.push_str(&text);
                    } else if image_depth == 0 && link_depth == 0 {
                        mode = mode_at(&source_file.text, range.start, mode, config.default_mode);
                        if active_span.is_none() {
                            active_span = Some((range.start, mode));
                        }
                    }
                }
                Event::Code(_)
                | Event::InlineHtml(_)
                | Event::Html(_)
                | Event::InlineMath(_)
                | Event::DisplayMath(_) => {}
                _ => {}
            }
        }
        finish_active(&mut prose, &mut active_span, source_file.text.len(), mode);
        Self {
            source: source_file,
            prose,
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

fn finish_active(
    prose: &mut Vec<ProseSpan>,
    active: &mut Option<(usize, TextMode)>,
    end: usize,
    mode: TextMode,
) {
    if let Some((start, started_mode)) = active.take() {
        let end = end.saturating_sub(1).max(start);
        if start < end {
            prose.push(ProseSpan {
                span: Span::new(start, end),
                mode: started_mode,
            });
        }
    }
    let _ = mode;
}

fn mode_at(source: &str, offset: usize, current: TextMode, default: TextMode) -> TextMode {
    let prefix = &source[..offset.min(source.len())];
    let procedural = prefix.rfind("<!-- englishlint: procedural");
    let descriptive = prefix.rfind("<!-- englishlint: descriptive");
    match (procedural, descriptive) {
        (Some(procedural), Some(descriptive)) if procedural > descriptive => TextMode::Procedural,
        (Some(_), None) => TextMode::Procedural,
        (None, Some(_)) => TextMode::Descriptive,
        _ => {
            if prefix.trim().is_empty() {
                default
            } else {
                current
            }
        }
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
    fn pulldown_excludes_code_headings_links_and_images() {
        let document = Document::parse("test.md", "# Install\n\nRun the service.\n\n```\nshould\n```\n\n[should](https://example.com/should) ![may](image.png)\n".into(), &Config::default());
        let visible: String = document
            .prose
            .iter()
            .map(|span| document.source.text_for(span.span))
            .collect();
        assert!(visible.contains("Run the service."));
        assert!(!visible.contains("should\n```"));
        assert!(!visible.contains("https://"));
    }

    #[test]
    fn preserves_prose_source_offsets() {
        let document = Document::parse("test.md", "A café works.\n".into(), &Config::default());
        let span = document.prose.first().unwrap().span;
        assert_eq!(document.source.text_for(span), "A café works.");
    }
}
