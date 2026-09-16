mod diagnostic;
mod lint;
mod parser;

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let path = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("usage: bookmark-lint <bookmarks.html>");
            return ExitCode::from(2);
        }
    };

    let source = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read {path}: {e}");
            return ExitCode::from(2);
        }
    };

    let links = parser::parse(&source);
    let diagnostics = lint::run(&links);

    if diagnostics.is_empty() {
        println!("{path}: no findings ({} bookmarks checked)", links.len());
        return ExitCode::SUCCESS;
    }

    for d in &diagnostics {
        eprintln!("{}\n", d.render(&path, &source));
    }

    eprintln!(
        "{} finding{} in {path}",
        diagnostics.len(),
        if diagnostics.len() == 1 { "" } else { "s" }
    );

    ExitCode::FAILURE
}
