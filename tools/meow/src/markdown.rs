//! Renders Markdown in the terminal: colored headings, bullet lists, quotes,
//! boxed tables and highlighted code blocks, wrapped to the output width.

use std::io::{self, Write};
use std::mem;

use anstyle::{Ansi256Color, AnsiColor, Color, Style};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use unicode_width::UnicodeWidthStr;

use crate::highlight::Highlighter;

const DIM: Style = Style::new().dimmed();
const QUOTE_BAR: Style = AnsiColor::Green.on_default();
const BULLET: Style = AnsiColor::Cyan.on_default().bold();
const INLINE_CODE: Style = AnsiColor::BrightYellow
    .on_default()
    .bg_color(Some(Color::Ansi256(Ansi256Color(236))));
const LINK: Color = Color::Ansi(AnsiColor::BrightBlue);
const TABLE_HEAD: Style = Style::new().bold();
const DONE: Style = AnsiColor::Green.on_default();

pub fn render(src: &str, hl: &Highlighter, width: usize, out: &mut impl Write) -> io::Result<()> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut renderer = Renderer::new(hl, width);
    for event in Parser::new_ext(src, options) {
        renderer.event(event);
    }
    renderer.flush();
    out.write_all(renderer.out.as_bytes())
}

/// Left margin added by a list item or a quote.
struct Indent {
    /// Printed on every line after the first.
    rest: String,
    width: usize,
    /// Printed once, on the item's first line (the bullet or number).
    marker: Option<String>,
}

struct Table {
    aligns: Vec<Alignment>,
    rows: Vec<Vec<String>>,
    cell: String,
}

struct Renderer<'a> {
    hl: &'a Highlighter,
    width: usize,
    out: String,
    /// Inline text of the current block, waiting to be wrapped.
    spans: Vec<(String, Style)>,
    bold: usize,
    italic: usize,
    strike: usize,
    /// Open links: target URL and where their text starts in `spans`.
    links: Vec<(String, usize)>,
    heading: Option<HeadingLevel>,
    indents: Vec<Indent>,
    /// Open lists, with the next number for ordered ones.
    lists: Vec<Option<u64>>,
    /// Open code block: language and collected text.
    code: Option<(String, String)>,
    table: Option<Table>,
    /// A blank line goes before the next block.
    gap: bool,
}

impl<'a> Renderer<'a> {
    fn new(hl: &'a Highlighter, width: usize) -> Self {
        Self {
            hl,
            width,
            out: String::new(),
            spans: Vec::new(),
            bold: 0,
            italic: 0,
            strike: 0,
            links: Vec::new(),
            heading: None,
            indents: Vec::new(),
            lists: Vec::new(),
            code: None,
            table: None,
            gap: false,
        }
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.push(&text, self.style()),
            Event::Code(code) => self.push(&code, INLINE_CODE),
            Event::Html(html) | Event::InlineHtml(html) => self.push(&html, DIM),
            Event::SoftBreak => self.push(" ", Style::new()),
            Event::HardBreak => self.push("\n", Style::new()),
            Event::Rule => self.rule(),
            Event::TaskListMarker(true) => self.push("☑ ", DONE),
            Event::TaskListMarker(false) => self.push("☐ ", DIM),
            Event::FootnoteReference(name) => self.push(&format!("[^{name}]"), DIM),
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => self.flush(),
            Tag::Heading { level, .. } => {
                self.flush();
                self.heading = Some(level);
            }
            Tag::BlockQuote(_) => {
                self.flush();
                self.gap_line();
                self.indents.push(Indent {
                    rest: format!("{QUOTE_BAR}│{QUOTE_BAR:#} "),
                    width: 2,
                    marker: None,
                });
            }
            Tag::CodeBlock(kind) => {
                self.flush();
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split([' ', ',']).next().unwrap_or("").to_owned()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                self.code = Some((lang, String::new()));
            }
            Tag::List(start) => {
                self.flush();
                self.lists.push(start);
            }
            Tag::Item => self.item(),
            Tag::Emphasis => self.italic += 1,
            Tag::Strong => self.bold += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Link { dest_url, .. } => self.links.push((dest_url.to_string(), self.spans.len())),
            Tag::Image { .. } => {
                self.push("[image: ", DIM);
                self.italic += 1;
            }
            Tag::Table(aligns) => {
                self.flush();
                self.table = Some(Table {
                    aligns,
                    rows: Vec::new(),
                    cell: String::new(),
                });
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(table) = &mut self.table {
                    table.rows.push(Vec::new());
                }
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.flush();
                self.gap = true;
            }
            TagEnd::Heading(level) => {
                self.heading_block(level);
                self.heading = None;
                self.gap = true;
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                self.indents.pop();
                self.gap = true;
            }
            TagEnd::CodeBlock => {
                if let Some((lang, body)) = self.code.take() {
                    self.code_block(&lang, &body);
                }
                self.gap = true;
            }
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
                if self.lists.is_empty() {
                    self.gap = true;
                }
            }
            TagEnd::Item => {
                self.flush();
                self.indents.pop();
            }
            TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
            TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Link => self.end_link(),
            TagEnd::Image => {
                self.italic = self.italic.saturating_sub(1);
                self.push("]", DIM);
            }
            TagEnd::TableCell => {
                if let Some(table) = &mut self.table {
                    let cell = mem::take(&mut table.cell).trim().to_owned();
                    if let Some(row) = table.rows.last_mut() {
                        row.push(cell);
                    }
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.table_block(&table);
                }
                self.gap = true;
            }
            _ => {}
        }
    }

    /// Sends text to whatever is open: a code block, a table cell, or the current block.
    fn push(&mut self, text: &str, style: Style) {
        if let Some((_, body)) = &mut self.code {
            body.push_str(text);
        } else if let Some(table) = &mut self.table {
            table.cell.push_str(if text == "\n" { " " } else { text });
        } else {
            self.spans.push((text.to_owned(), style));
        }
    }

    fn style(&self) -> Style {
        let mut style = self.heading.map_or(Style::new(), heading_style);
        if self.bold > 0 {
            style = style.bold();
        }
        if self.italic > 0 {
            style = style.italic();
        }
        if self.strike > 0 {
            style = style.strikethrough();
        }
        if !self.links.is_empty() {
            style = style.fg_color(Some(LINK)).underline();
        }
        style
    }

    fn item(&mut self) {
        self.flush();
        let depth = self.lists.len();
        let bullet = match self.lists.last_mut() {
            Some(Some(next)) => {
                let number = format!("{next}.");
                *next += 1;
                number
            }
            _ => ["•", "◦", "▪"][depth.saturating_sub(1) % 3].to_owned(),
        };
        let lead = if depth <= 1 { "  " } else { "" };
        let width = lead.len() + bullet.width() + 1;
        self.indents.push(Indent {
            rest: " ".repeat(width),
            width,
            marker: Some(format!("{lead}{BULLET}{bullet}{BULLET:#} ")),
        });
    }

    /// Shows a link's URL after its text, unless the text already is the URL.
    fn end_link(&mut self) {
        let Some((url, start)) = self.links.pop() else {
            return;
        };
        if self.table.is_some() || url.is_empty() || url.starts_with('#') {
            return;
        }
        let text: String = self
            .spans
            .get(start..)
            .unwrap_or_default()
            .iter()
            .map(|(t, _)| t.as_str())
            .collect();
        if text != url {
            self.push(&format!(" ({url})"), DIM);
        }
    }

    /// Wraps and prints the current block's text.
    fn flush(&mut self) {
        if self.spans.iter().all(|(text, _)| text.trim().is_empty()) {
            self.spans.clear();
            return;
        }
        let spans = mem::take(&mut self.spans);
        self.gap_line();
        for line in wrap(&spans, self.avail()) {
            self.line(&line);
        }
    }

    fn heading_block(&mut self, level: HeadingLevel) {
        let mut spans = mem::take(&mut self.spans);
        if spans.iter().all(|(text, _)| text.trim().is_empty()) {
            return;
        }
        let style = heading_style(level);
        if level == HeadingLevel::H3 {
            spans.insert(0, ("▸ ".to_owned(), style));
        }
        self.gap_line();
        let avail = self.avail();
        for line in wrap(&spans, avail) {
            self.line(&line);
        }
        let underline = match level {
            HeadingLevel::H1 => "━",
            HeadingLevel::H2 => "─",
            _ => return,
        };
        let text_width = spans
            .iter()
            .map(|(text, _)| text.width())
            .sum::<usize>()
            .min(avail);
        self.line(&format!("{style}{}{style:#}", underline.repeat(text_width)));
    }

    fn code_block(&mut self, lang: &str, body: &str) {
        self.gap_line();
        let avail = self.avail();
        let label = if lang.is_empty() {
            String::new()
        } else {
            format!(" {lang} ")
        };
        let top = "─".repeat(avail.saturating_sub(2 + label.width()));
        self.line(&format!("{DIM}╭─{label}{top}{DIM:#}"));
        let lines = match self.hl.by_token(lang) {
            Some(syntax) => self.hl.highlight(syntax, body),
            None => body.lines().map(str::to_owned).collect(),
        };
        for code in lines {
            self.line(&format!("{DIM}│{DIM:#} {code}"));
        }
        self.line(&format!(
            "{DIM}╰{}{DIM:#}",
            "─".repeat(avail.saturating_sub(1))
        ));
    }

    fn table_block(&mut self, table: &Table) {
        let cols = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        if cols == 0 {
            return;
        }
        self.gap_line();
        let mut widths = vec![0; cols];
        for row in &table.rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.width());
            }
        }
        // Too wide for the screen: narrow the widest column until it fits,
        // and let its text wrap onto more lines.
        let avail = self.avail();
        while widths.iter().sum::<usize>() + 3 * cols + 1 > avail {
            let Some(widest) = widths.iter_mut().max() else {
                break;
            };
            if *widest <= 6 {
                break;
            }
            *widest -= 1;
        }
        let border = |left: &str, mid: &str, right: &str| {
            let parts: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
            format!("{DIM}{left}{}{right}{DIM:#}", parts.join(mid))
        };
        let (top, sep, bottom) = (
            border("┌", "┬", "┐"),
            border("├", "┼", "┤"),
            border("└", "┴", "┘"),
        );

        self.line(&top);
        for (r, row) in table.rows.iter().enumerate() {
            let style = if r == 0 { TABLE_HEAD } else { Style::new() };
            let cells: Vec<Vec<String>> = widths
                .iter()
                .enumerate()
                .map(|(i, &width)| {
                    let cell = row.get(i).map_or("", String::as_str);
                    wrap(&[(cell.to_owned(), Style::new())], width)
                })
                .collect();
            let height = cells.iter().map(Vec::len).max().unwrap_or(0).max(1);

            for l in 0..height {
                let mut text = format!("{DIM}│{DIM:#}");
                for (i, width) in widths.iter().enumerate() {
                    let part = cells[i].get(l).map_or("", String::as_str);
                    let pad = width.saturating_sub(part.width());
                    let (left, right) = match table.aligns.get(i) {
                        Some(Alignment::Right) => (pad, 0),
                        Some(Alignment::Center) => (pad / 2, pad - pad / 2),
                        _ => (0, pad),
                    };
                    text.push_str(&format!(
                        " {}{style}{part}{style:#}{} {DIM}│{DIM:#}",
                        " ".repeat(left),
                        " ".repeat(right)
                    ));
                }
                self.line(&text);
            }
            if r == 0 && table.rows.len() > 1 {
                self.line(&sep);
            }
        }
        self.line(&bottom);
    }

    fn rule(&mut self) {
        self.flush();
        self.gap_line();
        let width = self.avail();
        self.line(&format!("{DIM}{}{DIM:#}", "─".repeat(width)));
        self.gap = true;
    }

    /// Prints one line with the margins of open lists and quotes.
    fn line(&mut self, text: &str) {
        for indent in &mut self.indents {
            match indent.marker.take() {
                Some(marker) => self.out.push_str(&marker),
                None => self.out.push_str(&indent.rest),
            }
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Prints the blank line between blocks, if one is due.
    fn gap_line(&mut self) {
        if mem::take(&mut self.gap) && !self.out.is_empty() {
            let margin: String = self.indents.iter().map(|i| i.rest.as_str()).collect();
            self.out.push_str(margin.trim_end());
            self.out.push('\n');
        }
    }

    /// Columns left for text after margins.
    fn avail(&self) -> usize {
        let margins: usize = self.indents.iter().map(|i| i.width).sum();
        self.width.saturating_sub(margins).max(20)
    }
}

fn heading_style(level: HeadingLevel) -> Style {
    match level {
        HeadingLevel::H1 => AnsiColor::BrightMagenta.on_default().bold(),
        HeadingLevel::H2 => AnsiColor::BrightCyan.on_default().bold(),
        HeadingLevel::H3 => AnsiColor::BrightYellow.on_default().bold(),
        _ => Style::new().bold(),
    }
}

/// Word-wraps styled spans to `width` columns. A `"\n"` span forces a break.
fn wrap(spans: &[(String, Style)], width: usize) -> Vec<String> {
    let mut wrapper = Wrapper {
        width,
        ..Wrapper::default()
    };
    for (text, style) in spans {
        if text == "\n" {
            wrapper.end_word();
            wrapper.break_line();
            continue;
        }
        let mut word = String::new();
        for ch in text.chars() {
            if ch.is_whitespace() {
                wrapper.add(&word, *style);
                word.clear();
                wrapper.end_word();
                wrapper.space = true;
            } else {
                word.push(ch);
            }
        }
        wrapper.add(&word, *style);
    }
    wrapper.finish()
}

#[derive(Default)]
struct Wrapper {
    width: usize,
    lines: Vec<String>,
    line: String,
    line_width: usize,
    /// The word being built; it can mix styles, as in `**bold**text`.
    word: String,
    word_width: usize,
    /// Whitespace came before the current word.
    space: bool,
}

impl Wrapper {
    fn add(&mut self, text: &str, style: Style) {
        if text.is_empty() {
            return;
        }
        self.word_width += text.width();
        self.word.push_str(&format!("{style}{text}{style:#}"));
    }

    fn end_word(&mut self) {
        if self.word.is_empty() {
            return;
        }
        let gap = usize::from(self.space && self.line_width > 0);
        if self.line_width > 0 && self.line_width + gap + self.word_width > self.width {
            self.break_line();
        } else if gap == 1 {
            self.line.push(' ');
            self.line_width += 1;
        }
        self.line.push_str(&self.word);
        self.line_width += self.word_width;
        self.word.clear();
        self.word_width = 0;
        self.space = false;
    }

    fn break_line(&mut self) {
        self.lines.push(mem::take(&mut self.line));
        self.line_width = 0;
    }

    fn finish(mut self) -> Vec<String> {
        self.end_word();
        if !self.line.is_empty() {
            self.lines.push(self.line);
        }
        self.lines
    }
}
