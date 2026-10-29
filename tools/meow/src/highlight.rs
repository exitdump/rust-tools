//! Syntax highlighting for code, and light highlighting for plain text.

use std::fmt::Write;
use std::path::Path;
use std::sync::LazyLock;

use anstyle::{AnsiColor, Style};
use anyhow::{Result, bail};
use regex::Regex;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

pub struct Highlighter {
    syntaxes: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new(theme: &str) -> Result<Self> {
        let mut themes = ThemeSet::load_defaults().themes;
        let Some(theme) = themes.remove(theme) else {
            let names: Vec<_> = themes.keys().map(String::as_str).collect();
            bail!("unknown theme `{theme}`, try one of: {}", names.join(", "));
        };
        Ok(Self {
            syntaxes: SyntaxSet::load_defaults_newlines(),
            theme,
        })
    }

    /// Finds a syntax by name or extension, like `rust`, `rs` or `json`.
    pub fn by_token(&self, token: &str) -> Option<&SyntaxReference> {
        if token.is_empty() {
            return None;
        }
        self.syntaxes
            .find_syntax_by_token(token)
            .filter(|s| s.name != "Plain Text")
    }

    /// Guesses a file's syntax from its name, then from its first line (`#!/bin/sh`).
    pub fn by_path(&self, path: &Path, first_line: &str) -> Option<&SyntaxReference> {
        let by_name = |name: Option<&std::ffi::OsStr>| {
            name.and_then(|n| n.to_str())
                .and_then(|n| self.syntaxes.find_syntax_by_extension(n))
        };
        by_name(path.extension())
            .or_else(|| by_name(path.file_name()))
            .or_else(|| self.syntaxes.find_syntax_by_first_line(first_line))
            .filter(|s| s.name != "Plain Text")
    }

    /// Highlights code into terminal-colored lines, without line endings.
    pub fn highlight(&self, syntax: &SyntaxReference, code: &str) -> Vec<String> {
        let mut lines = HighlightLines::new(syntax, &self.theme);
        LinesWithEndings::from(code)
            .map(|line| {
                let colored = match lines.highlight_line(line, &self.syntaxes) {
                    Ok(ranges) => as_24_bit_terminal_escaped(&ranges, false),
                    Err(_) => line.to_owned(),
                };
                format!("{}\x1b[0m", colored.trim_end_matches(['\r', '\n']))
            })
            .collect()
    }
}

/// Text patterns worth a color. The first alternative that matches wins.
static TEXT_PATTERNS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
          (?P<url>https?://[^\s<>"'`)\]]+)
        | (?P<email>[\w.+-]+@[\w-]+(?:\.[\w-]+)+)
        | (?P<error>\b(?:ERROR|ERR|FATAL|FAIL|FAILED|FAILURE|PANIC|CRITICAL)\b)
        | (?P<warn>\b(?:WARN|WARNING|DEPRECATED)\b)
        | (?P<ok>\b(?:INFO|OK|SUCCESS|DONE|PASS|PASSED)\b)
        | (?P<debug>\b(?:DEBUG|TRACE)\b)
        | (?P<todo>\b(?:TODO|FIXME|NOTE|HACK|XXX)\b)
        | (?P<time>\b\d{4}-\d{2}-\d{2}(?:[T\ ]\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?)?\b
                  |\b\d{1,2}:\d{2}(?::\d{2})?\b)
        | (?P<string>"[^"\n]*")
        | (?P<number>\b\d+(?:\.\d+)?\b)
        "#,
    )
    .expect("text patterns are valid")
});

const TEXT_STYLES: [(&str, Style); 10] = [
    ("url", AnsiColor::BrightBlue.on_default().underline()),
    ("email", AnsiColor::Cyan.on_default()),
    ("error", AnsiColor::BrightRed.on_default().bold()),
    ("warn", AnsiColor::BrightYellow.on_default().bold()),
    ("ok", AnsiColor::BrightGreen.on_default().bold()),
    ("debug", Style::new().dimmed()),
    ("todo", AnsiColor::BrightMagenta.on_default().bold()),
    ("time", AnsiColor::Cyan.on_default()),
    ("string", AnsiColor::Green.on_default()),
    ("number", AnsiColor::Yellow.on_default()),
];

/// Colors URLs, log levels, TODOs, dates, strings and numbers in a line of text.
pub fn text_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut last = 0;
    for caps in TEXT_PATTERNS.captures_iter(line) {
        let Some(whole) = caps.get(0) else { continue };
        let style = TEXT_STYLES
            .iter()
            .find(|(name, _)| caps.name(name).is_some())
            .map_or(Style::new(), |&(_, style)| style);
        out.push_str(&line[last..whole.start()]);
        let _ = write!(out, "{style}{}{style:#}", whole.as_str());
        last = whole.end();
    }
    out.push_str(&line[last..]);
    out
}
