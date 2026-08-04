use crate::document::Document;
use crate::source::Span;
use crate::text::TextMode;

#[derive(Clone, Debug)]
pub struct Word {
    pub span: Span,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Sentence {
    pub span: Span,
    pub mode: TextMode,
    pub words: Vec<Word>,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub span: Span,
    pub sentences: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Analysis {
    pub sentences: Vec<Sentence>,
    pub paragraphs: Vec<Paragraph>,
    pub words: Vec<Word>,
}

impl Analysis {
    pub fn build(document: &Document) -> Self {
        let mut sentences = Vec::new();
        let mut all_words = Vec::new();
        for prose in &document.prose {
            let text = &document.source.text[prose.span.start.0..prose.span.end.0];
            let mut start = None;
            let mut previous = None;
            for (offset, character) in text.char_indices() {
                if start.is_none() && !character.is_whitespace() {
                    start = Some(offset);
                }
                let boundary = ".!?:".contains(character)
                    && text[offset + character.len_utf8()..]
                        .chars()
                        .next()
                        .is_none_or(char::is_whitespace);
                if boundary {
                    if let Some(begin) = start.take() {
                        let end = offset + character.len_utf8();
                        let span = Span::new(prose.span.start.0 + begin, prose.span.start.0 + end);
                        let words = tokenize(
                            &document.source.text[span.start.0..span.end.0],
                            span.start.0,
                        );
                        all_words.extend(words.iter().cloned());
                        sentences.push(Sentence {
                            span,
                            mode: prose.mode,
                            words,
                        });
                    }
                }
                previous = Some(offset);
            }
            if let Some(begin) = start {
                let end = text.len();
                if previous.is_some() {
                    let span = Span::new(prose.span.start.0 + begin, prose.span.start.0 + end);
                    let words = tokenize(
                        &document.source.text[span.start.0..span.end.0],
                        span.start.0,
                    );
                    all_words.extend(words.iter().cloned());
                    sentences.push(Sentence {
                        span,
                        mode: prose.mode,
                        words,
                    });
                }
            }
        }
        sentences.sort_by_key(|sentence| sentence.span.start);
        let mut paragraphs = Vec::new();
        let mut current = Vec::new();
        for (index, sentence) in sentences.iter().enumerate() {
            let before = &document.source.text[current
                .last()
                .map(|last: &usize| sentences[*last].span.end.0)
                .unwrap_or(sentence.span.start.0)
                ..sentence.span.start.0];
            if !current.is_empty() && (before.contains("\n\n") || before.contains("\n#")) {
                paragraphs.push(paragraph(&sentences, &current));
                current.clear();
            }
            current.push(index);
        }
        if !current.is_empty() {
            paragraphs.push(paragraph(&sentences, &current));
        }
        Self {
            sentences,
            paragraphs,
            words: all_words,
        }
    }
}

fn paragraph(sentences: &[Sentence], indices: &[usize]) -> Paragraph {
    Paragraph {
        span: Span::new(
            sentences[*indices.first().unwrap()].span.start.0,
            sentences[*indices.last().unwrap()].span.end.0,
        ),
        sentences: indices.to_vec(),
    }
}

pub fn tokenize(text: &str, base: usize) -> Vec<Word> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut output = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if !chars[index].1.is_alphanumeric() {
            index += 1;
            continue;
        }
        let start = chars[index].0;
        index += 1;
        while index < chars.len() {
            let character = chars[index].1;
            let previous = chars[index - 1].1;
            let next = chars.get(index + 1).map(|(_, c)| *c);
            if character.is_alphanumeric() {
                index += 1;
                continue;
            }
            if ['\'', '-', '.'].contains(&character)
                && next.is_some_and(char::is_alphanumeric)
                && (previous.is_alphanumeric() || previous == '.')
            {
                index += 1;
                continue;
            }
            break;
        }
        let end = chars
            .get(index)
            .map(|(offset, _)| *offset)
            .unwrap_or(text.len());
        output.push(Word {
            span: Span::new(base + start, base + end),
            text: text[start..end].to_string(),
        });
    }
    output
}

pub fn occurrences(text: &str, phrase: &str, base: usize) -> Vec<Span> {
    let lower = text.to_ascii_lowercase();
    let needle = phrase.to_ascii_lowercase();
    let mut result = Vec::new();
    let mut from = 0;
    while let Some(relative) = lower[from..].find(&needle) {
        let at = from + relative;
        let end = at + needle.trim_end().len();
        let before = lower[..at].chars().last();
        let after = lower[end..].chars().next();
        if before.is_none_or(|c| !c.is_alphanumeric()) && after.is_none_or(|c| !c.is_alphanumeric())
        {
            result.push(Span::new(base + at, base + end));
        }
        from = at + needle.len();
    }
    result
}
