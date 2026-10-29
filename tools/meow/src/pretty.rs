//! Pretty output: a header per file, then rendered Markdown,
//! highlighted code, or colored text with line numbers.

use std::io::{BufRead, Write};
use std::path::Path;

use anstyle::{AnsiColor, Style};
use anyhow::Result;
use syntect::parsing::SyntaxReference;
use unicode_width::UnicodeWidthStr;

use crate::Args;
use crate::highlight::{self, Highlighter};
use crate::markdown;

const DIM: Style = Style::new().dimmed();
const NAME: Style = AnsiColor::BrightWhite.on_default().bold();

/// How a file's contents get shown.
enum Kind<'a> {
    Markdown,
    Code(&'a SyntaxReference),
    Text,
}

pub struct Printer {
    hl: Highlighter,
    language: Option<String>,
    width: usize,
    printed_any: bool,
}

impl Printer {
    pub fn new(args: &Args) -> Result<Self> {
        // Older Windows consoles need color support switched on first.
        #[cfg(windows)]
        anstyle_query::windows::enable_ansi_colors();

        let width = args
            .width
            .or_else(|| terminal_size::terminal_size().map(|(w, _)| usize::from(w.0)))
            .unwrap_or(80)
            .clamp(20, 120);
        Ok(Self {
            hl: Highlighter::new(&args.theme)?,
            language: args.language.clone(),
            width,
            printed_any: false,
        })
    }

    pub fn print(
        &mut self,
        path: &Path,
        mut input: impl BufRead,
        out: &mut impl Write,
    ) -> Result<()> {
        let mut bytes = Vec::new();
        input.read_to_end(&mut bytes)?;
        if self.printed_any {
            writeln!(out)?;
        }
        self.printed_any = true;

        let name = if path == Path::new("-") {
            "stdin".to_owned()
        } else {
            path.display().to_string()
        };
        let size = human_size(bytes.len());

        if bytes.iter().take(8000).any(|&b| b == 0) {
            self.header(out, &name, &format!("binary · {size}"))?;
            writeln!(
                out,
                "{DIM}  binary file not shown, use --plain to print it raw{DIM:#}"
            )?;
            return self.footer(out);
        }

        let text = String::from_utf8_lossy(&bytes);
        let kind = self.kind(path, text.lines().next().unwrap_or(""));
        let label = match &kind {
            Kind::Markdown => "Markdown",
            Kind::Code(syntax) => syntax.name.as_str(),
            Kind::Text => "Text",
        };
        let count = text.lines().count();
        let lines = if count == 1 { "line" } else { "lines" };
        self.header(out, &name, &format!("{label} · {count} {lines} · {size}"))?;

        match kind {
            Kind::Markdown => {
                // Some breathing room inside the header and footer
                writeln!(out)?;
                markdown::render(&text, &self.hl, self.width, out)?;
                writeln!(out)?;
            }
            Kind::Code(syntax) => self.numbered(out, &self.hl.highlight(syntax, &text))?,
            Kind::Text => {
                let lines: Vec<String> = text.lines().map(highlight::text_line).collect();
                self.numbered(out, &lines)?;
            }
        }
        self.footer(out)
    }

    fn kind(&self, path: &Path, first_line: &str) -> Kind<'_> {
        if let Some(lang) = &self.language {
            return match lang.to_lowercase().as_str() {
                "md" | "markdown" => Kind::Markdown,
                other => self.hl.by_token(other).map_or(Kind::Text, Kind::Code),
            };
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(ext.as_str(), "md" | "markdown" | "mdx") {
            return Kind::Markdown;
        }
        self.hl
            .by_path(path, first_line)
            .map_or(Kind::Text, Kind::Code)
    }

    /// Prints lines behind a gutter of line numbers.
    fn numbered(&self, out: &mut impl Write, lines: &[String]) -> Result<()> {
        let digits = lines.len().to_string().len().max(3);
        for (i, line) in lines.iter().enumerate() {
            writeln!(out, "{DIM}{:>digits$} │{DIM:#} {line}", i + 1)?;
        }
        Ok(())
    }

    /// `╭─ name ──────────── Rust · 12 lines · 340 B ─`
    fn header(&self, out: &mut impl Write, name: &str, meta: &str) -> Result<()> {
        let fill = self
            .width
            .saturating_sub(name.width() + meta.width() + 7)
            .max(1);
        writeln!(
            out,
            "{DIM}╭─{DIM:#} {NAME}{name}{NAME:#} {DIM}{} {meta} ─{DIM:#}",
            "─".repeat(fill)
        )?;
        Ok(())
    }

    fn footer(&self, out: &mut impl Write) -> Result<()> {
        writeln!(out, "{DIM}╰{}{DIM:#}", "─".repeat(self.width - 1))?;
        Ok(())
    }
}

fn human_size(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    let size = bytes as f64;
    if size < KB {
        format!("{bytes} B")
    } else if size < KB * KB {
        format!("{:.1} KB", size / KB)
    } else {
        format!("{:.1} MB", size / (KB * KB))
    }
}
