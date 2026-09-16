use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::{Link, Pos};
use std::collections::HashMap;

pub fn run(links: &[Link]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen_urls: HashMap<String, Pos> = HashMap::new();

    for link in links {
        match &link.href {
            None => diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: "bookmark link has no href attribute".to_string(),
                pos: link.href_pos,
                len: 2,
            }),
            Some(url) if url.trim().is_empty() => diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: "href attribute is empty".to_string(),
                pos: link.href_pos,
                len: 1,
            }),
            Some(url) => {
                if let Some(first_pos) = seen_urls.get(url) {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: format!(
                            "duplicate bookmark url (first seen at line {})",
                            first_pos.line
                        ),
                        pos: link.href_pos,
                        len: url.chars().count().max(1),
                    });
                } else {
                    seen_urls.insert(url.clone(), link.href_pos);
                }
            }
        }

        if link.title.is_empty() {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: "bookmark has no title".to_string(),
                pos: link.title_pos,
                len: 1,
            });
        }
    }

    diagnostics.sort_by_key(|d| (d.pos.line, d.pos.col));
    diagnostics
}
