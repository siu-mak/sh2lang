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
pub struct Diagnostic {
    pub msg: String,
    pub span: Span,
    pub sm: Option<SourceMap>,
    pub file: Option<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn format(&self, base: Option<&std::path::Path>) -> String {
        let main = if let (Some(sm), Some(file)) = (&self.sm, &self.file) {
            sm.format_diagnostic(file, base, &self.msg, self.span)
        } else {
            format!("error: {}", self.msg)
        };
        match &self.help {
            Some(help) => format!("{}\nhelp: {}", main, help),
            None => main,
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
        
        let line_start = self.line_starts[line_idx];
        // Calculate column by counting characters from line start to pos
        let col = if pos >= line_start {
             self.src[line_start..min(pos, self.src.len())].chars().count() + 1
        } else {
             1
        };
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
        if start > end { return ""; }
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
            // Calculate length in characters
            let line_start = self.line_starts[start_line - 1];
            // Safe slicing
            let start_clamp = max(line_start, span.start);
            let end_clamp = min(self.src.len(), span.end);
            
            // Only measure length if valid range on this line
            let len = if end_clamp > start_clamp {
                self.src[start_clamp..end_clamp].chars().count()
            } else {
                0 
            };
            
            let len = max(1, len);
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
