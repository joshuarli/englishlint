use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteOffset(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: ByteOffset,
    pub end: ByteOffset,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end, "span start must not exceed span end");
        Self {
            start: ByteOffset(start),
            end: ByteOffset(end),
        }
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug)]
pub struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts = vec![0];
        for (offset, character) in text.char_indices() {
            if character == '\n' {
                starts.push(offset + 1);
            }
        }
        Self { starts }
    }

    pub fn location(&self, text: &str, offset: ByteOffset) -> Location {
        let index = match self.starts.binary_search(&offset.0) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        Location {
            line: index + 1,
            column: text[self.starts[index]..offset.0.min(text.len())]
                .chars()
                .count()
                + 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub lines: LineIndex,
}

impl SourceFile {
    pub fn new(path: impl Into<PathBuf>, text: String) -> Self {
        let lines = LineIndex::new(&text);
        Self {
            path: path.into(),
            text,
            lines,
        }
    }

    pub fn location(&self, offset: ByteOffset) -> Location {
        self.lines.location(&self.text, offset)
    }

    pub fn text_for(&self, span: Span) -> &str {
        &self.text[span.start.0.min(self.text.len())..span.end.0.min(self.text.len())]
    }

    pub fn display_path(&self, root: &Path) -> String {
        self.path
            .strip_prefix(root)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_maps_unicode_offsets_to_columns() {
        let source = "élan\nsecond";
        let index = LineIndex::new(source);
        assert_eq!(
            index.location(source, ByteOffset(0)),
            Location { line: 1, column: 1 }
        );
        assert_eq!(
            index.location(source, ByteOffset("é".len())),
            Location { line: 1, column: 2 }
        );
        assert_eq!(
            index.location(source, ByteOffset(6)),
            Location { line: 2, column: 1 }
        );
    }

    #[test]
    fn spans_are_ordered_and_empty_spans_are_valid() {
        assert!(Span::new(2, 2).is_empty());
        assert!(Span::new(2, 4).start < Span::new(3, 4).start);
    }
}
