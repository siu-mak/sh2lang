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

    pub fn format_diagnostic(&self, file: &str, base: Option<&std::path::Path>, msg: &str, span: Span) -> String {
        let (start_line, start_col) = self.line_col(span.start);
        let (end_line, _) = self.line_col(span.end);
        let snippet = self.line_snippet(start_line);
        
        let mut arrow_col = start_col;

        if start_line != end_line {
            // Multi-line adjustment: skip leading whitespace if we point to it
            if let Some(first_non_ws) = snippet.chars().position(|c| !c.is_whitespace()) {
                 let first_non_ws_col = first_non_ws + 1;
                 if start_col < first_non_ws_col {
                     arrow_col = first_non_ws_col;
                 }
            }
        }

        let mut arrow = String::new();
        // Indent to column
        for _ in 0..(arrow_col - 1) {
            arrow.push(' ');
        }

        if start_line == end_line {
            let len = max(1, span.end - span.start);
            arrow.push('^');
            if len > 1 {
                for _ in 0..(len - 1) {
                    arrow.push('~');
                }
            }
        } else {
            // Multi-line: just point at start
            arrow.push('^');
        }

        let display_file = crate::diag_path::display_path(file, base);

        format!(
            "{}:{}:{}: {}\n{}\n{}",
            display_file, start_line, start_col, msg, snippet, arrow
        )
    }
}
