use crate::parser::Pos;

pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    fn label(&self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub pos: Pos,
    pub len: usize,
}

impl Diagnostic {
    // Renders a rustc-style block: the message, a `-->` location line, and
    // the source line itself with a caret under the exact span. Guessing
    // "somewhere in this file" is not good enough for a linter meant to be
    // run against a bookmarks.html with a few thousand links in it.
    pub fn render(&self, path: &str, source: &str) -> String {
        let line_text = source
            .lines()
            .nth(self.pos.line.saturating_sub(1))
            .unwrap_or("");
        let gutter = self.pos.line.to_string();
        let pad = " ".repeat(gutter.len());
        let caret_pad = " ".repeat(self.pos.col.saturating_sub(1));
        let carets = "^".repeat(self.len.max(1));

        format!(
            "{level}: {msg}\n{pad} --> {path}:{line}:{col}\n{pad} |\n{gutter} | {text}\n{pad} | {caret_pad}{carets}",
            level = self.severity.label(),
            msg = self.message,
            pad = pad,
            path = path,
            line = self.pos.line,
            col = self.pos.col,
            gutter = gutter,
            text = line_text,
            carets = carets,
        )
    }
}
