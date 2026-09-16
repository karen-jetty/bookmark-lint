// Minimal scanner for the Netscape Bookmark File Format (what every major
// browser exports). We don't need a general HTML parser: bookmark exports
// have one shape, `<A HREF="...">Title</A>`, repeated inside nested `<DL>`
// blocks. Walking the file char by char and tracking line/col ourselves is
// what lets diagnostics point at the exact byte a problem came from.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

pub struct Link {
    pub href: Option<String>,
    pub href_pos: Pos,
    pub title: String,
    pub title_pos: Pos,
}

struct Scanner {
    chars: Vec<char>,
    idx: usize,
    line: usize,
    col: usize,
}

impl Scanner {
    fn new(source: &str) -> Self {
        Scanner {
            chars: source.chars().collect(),
            idx: 0,
            line: 1,
            col: 1,
        }
    }

    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.idx).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.idx + offset).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.idx += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn eof(&self) -> bool {
        self.idx >= self.chars.len()
    }
}

pub fn parse(source: &str) -> Vec<Link> {
    let mut scanner = Scanner::new(source);
    let mut links = Vec::new();

    while !scanner.eof() {
        if is_anchor_open(&scanner) {
            scanner.bump(); // '<'
            scanner.bump(); // 'A'
            let (href, href_pos) = parse_attrs(&mut scanner);
            let title_pos = scanner.pos();
            let title = read_until_close_tag(&mut scanner);
            links.push(Link {
                href,
                href_pos,
                title,
                title_pos,
            });
        } else {
            scanner.bump();
        }
    }

    links
}

fn is_anchor_open(scanner: &Scanner) -> bool {
    let c0 = scanner.peek();
    let c1 = scanner.peek_at(1);
    let c2 = scanner.peek_at(2);
    matches!(c0, Some('<'))
        && matches!(c1, Some('a') | Some('A'))
        && matches!(c2, Some(c) if c.is_whitespace() || c == '>')
}

// Consumes attributes up to and including the closing '>'. Returns the
// HREF value (None if the attribute was never present) and the position
// of the first character of that value -- or, when HREF is missing, the
// position right after the tag name, so a "no href" diagnostic still has
// somewhere sensible to point.
fn parse_attrs(scanner: &mut Scanner) -> (Option<String>, Pos) {
    let fallback_pos = scanner.pos();
    let mut href: Option<String> = None;
    let mut href_pos = fallback_pos;

    loop {
        skip_whitespace(scanner);
        match scanner.peek() {
            None => break,
            Some('>') => {
                scanner.bump();
                break;
            }
            Some(_) => {
                let name = read_attr_name(scanner);
                if name.is_empty() {
                    // Stray character (e.g. a bare '=') in malformed markup.
                    // Skip it so we always make forward progress.
                    scanner.bump();
                    continue;
                }

                skip_whitespace(scanner);
                let mut value = String::new();
                let mut value_pos = scanner.pos();

                if scanner.peek() == Some('=') {
                    scanner.bump();
                    skip_whitespace(scanner);
                    match scanner.peek() {
                        Some(q @ ('"' | '\'')) => {
                            scanner.bump();
                            value_pos = scanner.pos();
                            while let Some(c) = scanner.peek() {
                                if c == q {
                                    break;
                                }
                                value.push(c);
                                scanner.bump();
                            }
                            scanner.bump(); // closing quote
                        }
                        _ => {
                            value_pos = scanner.pos();
                            while let Some(c) = scanner.peek() {
                                if c.is_whitespace() || c == '>' {
                                    break;
                                }
                                value.push(c);
                                scanner.bump();
                            }
                        }
                    }
                }

                if name.eq_ignore_ascii_case("href") {
                    href = Some(value);
                    href_pos = value_pos;
                }
            }
        }
    }

    (href, href_pos)
}

fn read_attr_name(scanner: &mut Scanner) -> String {
    let mut name = String::new();
    while let Some(c) = scanner.peek() {
        if c.is_whitespace() || c == '=' || c == '>' {
            break;
        }
        name.push(c);
        scanner.bump();
    }
    name
}

fn skip_whitespace(scanner: &mut Scanner) {
    while let Some(c) = scanner.peek() {
        if c.is_whitespace() {
            scanner.bump();
        } else {
            break;
        }
    }
}

// Bookmark titles are plain text between `<A ...>` and `</A>`; we stop at
// the next '<' rather than matching "</A>" literally so a malformed or
// missing close tag doesn't send us scanning off past the intended link.
fn read_until_close_tag(scanner: &mut Scanner) -> String {
    let mut text = String::new();
    while let Some(c) = scanner.peek() {
        if c == '<' {
            break;
        }
        text.push(c);
        scanner.bump();
    }
    text.trim().to_string()
}
