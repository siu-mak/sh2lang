use std::cmp::{max, min};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }

    pub fn merge(self, other: Span) -> Self {
        Span {
            start: min(self.start, other.start),
            end: max(self.end, other.end),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceMap {
    src: String,
    line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new(src: String) -> Self {
        let mut line_starts = vec![0];
        for (i, c) in src.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }
        SourceMap { src, line_starts }
    }

    pub fn len(&self) -> usize {
        self.src.len()
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn line_col(&self, pos: usize) -> (usize, usize) {
        let line_idx = self
            .line_starts
            .binary_search(&pos)
            .unwrap_or_else(|x| x - 1);
        let col = pos - self.line_starts[line_idx] + 1;
        (line_idx + 1, col)
    }

    pub fn line_snippet(&self, line: usize) -> &str {
        if line < 1 || line > self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[line - 1];
        let end = if line == self.line_starts.len() {
            self.src.len()
        } else {
            self.line_starts[line] - 1 // Exclude newline
        };
        &self.src[start..end]
    }

    pub fn format_diagnostic(&self, file: &str, msg: &str, span: Span) -> String {
        let (line, col) = self.line_col(span.start);
        let snippet = self.line_snippet(line);
        let mut arrow = String::new();
        for _ in 0..(col - 1) {
            arrow.push(' ');
        }
        let len = max(1, span.end - span.start);
        for _ in 0..len {
            arrow.push('^');
        }

        format!(
            "error: {}\n--> {}:{}:{}\n |\n | {}\n | {}",
            msg, file, line, col, snippet, arrow
        )
    }
}
