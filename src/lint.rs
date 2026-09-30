use crate::diagnostic::{Diagnostic, Severity};
use crate::parser::{Link, Pos};
use std::collections::HashMap;

const JAVASCRIPT_SCHEME: &str = "javascript:";

// Browsers strip leading whitespace from a URL before reading the scheme,
// and schemes are case-insensitive, so " JavaScript:..." still runs.
fn is_javascript_url(url: &str) -> bool {
    let trimmed = url.trim_start();
    trimmed
        .get(..JAVASCRIPT_SCHEME.len())
        .is_some_and(|s| s.eq_ignore_ascii_case(JAVASCRIPT_SCHEME))
}

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
                if is_javascript_url(url) {
                    // Only underline the scheme: bookmarklets are often
                    // hundreds of characters and can span several lines,
                    // which would make the caret row useless.
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        message: "javascript: bookmarklet link".to_string(),
                        pos: link.href_pos,
                        len: JAVASCRIPT_SCHEME.len(),
                    });
                }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn messages(source: &str) -> Vec<String> {
        run(&parse(source)).into_iter().map(|d| d.message).collect()
    }

    #[test]
    fn flags_bookmarklet() {
        let found = messages(r#"<DT><A HREF="javascript:alert(1)">Alert</A>"#);
        assert_eq!(found, vec!["javascript: bookmarklet link"]);
    }

    #[test]
    fn scheme_match_ignores_case_and_leading_space() {
        let found = messages(r#"<DT><A HREF="  JavaScript:void(0)">Noop</A>"#);
        assert_eq!(found, vec!["javascript: bookmarklet link"]);
    }

    #[test]
    fn ordinary_links_are_not_flagged() {
        let found = messages(
            r#"<DT><A HREF="https://example.com/javascript:">Docs</A>
<DT><A HREF="java">Short</A>"#,
        );
        assert!(found.is_empty());
    }

    #[test]
    fn non_ascii_href_does_not_panic() {
        assert!(messages("<DT><A HREF=\"\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\">Accents</A>").is_empty());
    }

    #[test]
    fn bookmarklet_caret_covers_only_the_scheme() {
        let links = parse(r#"<A HREF="javascript:alert(1)">x</A>"#);
        let d = run(&links);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].len, 11);
        assert_eq!(d[0].pos.col, 10);
    }
}
