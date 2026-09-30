# bookmark-lint

A linter for browser bookmark exports. Every major browser (Chrome,
Firefox, Safari, Edge) can export your bookmarks as an HTML file in the
Netscape Bookmark File Format -- a `<DL>` tree of `<DT><A HREF="...">`
links, unchanged since the mid-90s. After a decade of saving things you'll
never look at again, that file accumulates dead weight: links with no
title, empty `HREF` attributes left behind by broken sync tools, the same
page bookmarked three times under three different folders.

`bookmark-lint` reads that export and reports problems the way a compiler
reports problems: with a message, a file:line:col, and a caret pointing at
the exact character that's wrong. "Something's wrong in your bookmarks
file" is useless when the file has four thousand links in it.

## Exporting your bookmarks

- Chrome / Edge: open `chrome://bookmarks`, click the three-dot menu, choose
  "Export bookmarks".
- Firefox: open the Library (`Ctrl+Shift+O` / `Cmd+Shift+B`), "Import and
  Backup" -> "Export Bookmarks to HTML".
- Safari: File -> Export Bookmarks.

## Usage

```
cargo run --release -- bookmarks.html
```

Given a file like this:

```html
<DL><p>
    <DT><A HREF="https://example.com/">Example</A>
    <DT><A HREF="">Untitled Broken Link</A>
    <DT><A>No href at all</A>
    <DT><A HREF="https://example.com/"></A>
</DL><p>
```

it reports:

```
error: href attribute is empty
  --> bookmarks.html:3:18
   |
3  |     <DT><A HREF="">Untitled Broken Link</A>
   |                  ^

error: bookmark link has no href attribute
  --> bookmarks.html:4:12
   |
4  |     <DT><A>No href at all</A>
   |            ^^

warning: duplicate bookmark url (first seen at line 2)
  --> bookmarks.html:5:18
   |
5  |     <DT><A HREF="https://example.com/"></A>
   |                  ^^^^^^^^^^^^^^^^^^^^^

warning: bookmark has no title
  --> bookmarks.html:5:41
   |
5  |     <DT><A HREF="https://example.com/"></A>
   |                                         ^

4 findings in bookmarks.html
```

Exit code is `0` if the file has no findings, `1` if it does, `2` on a
usage or I/O error -- so it's usable as a CI check on a bookmarks export you
keep in version control.

## How it works

There's no HTML parser dependency and no dependencies at all -- just the
standard library. `src/parser.rs` walks the file character by character
tracking line and column as it goes, picking out `<A ...>...</A>` tags and
recording the exact position of each attribute value and title. `src/lint.rs`
runs a handful of checks over the resulting links; `src/diagnostic.rs`
renders findings in the compiler-style format shown above.

## Status

Early. The lint rules so far: missing `href`, empty `href`, duplicate URLs,
missing titles, and `javascript:` bookmarklet links. See the issue tracker (or the roadmap in commit
history) for what's planned next.
