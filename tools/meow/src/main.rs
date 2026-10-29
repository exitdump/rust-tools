mod highlight;
mod markdown;
mod plain;
mod pretty;

use std::io::{self, BufWriter, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

/// Print files beautifully: Markdown is rendered, code is highlighted,
/// and text gets line numbers and color. Piped output stays plain, like `cat`.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Files to print. `-` or no file means stdin.
    #[arg(default_value = "-")]
    files: Vec<PathBuf>,

    /// When to use pretty output
    #[arg(long, value_enum, default_value_t = When::Auto)]
    color: When,

    /// Plain output, exactly like `cat` (same as --color never)
    #[arg(short, long)]
    plain: bool,

    /// Language to use instead of guessing, e.g. `md`, `rust`, `json`
    #[arg(short, long)]
    language: Option<String>,

    /// Color theme for code
    #[arg(long, default_value = "base16-ocean.dark")]
    theme: String,

    /// Output width in columns (default: terminal width)
    #[arg(long)]
    width: Option<usize>,

    /// Number all lines (plain output)
    #[arg(short = 'n', long)]
    number: bool,

    /// Number only non-blank lines, wins over -n (plain output)
    #[arg(short = 'b', long)]
    number_nonblank: bool,

    /// Squeeze repeated blank lines into one (plain output)
    #[arg(short = 's', long)]
    squeeze_blank: bool,

    /// Show `$` at the end of each line (plain output)
    #[arg(short = 'E', long)]
    show_ends: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum When {
    /// Pretty in a terminal, plain when piped or when NO_COLOR is set
    Auto,
    Always,
    Never,
}

impl Args {
    /// Any classic `cat` flag asks for plain output.
    fn has_cat_flags(&self) -> bool {
        self.number || self.number_nonblank || self.squeeze_blank || self.show_ends
    }

    fn wants_pretty(&self) -> bool {
        if self.plain || self.has_cat_flags() {
            return false;
        }
        match self.color {
            When::Always => true,
            When::Never => false,
            When::Auto => io::stdout().is_terminal() && !anstyle_query::no_color(),
        }
    }
}

fn main() -> ExitCode {
    common::run("meow", || {
        let args = Args::parse();
        let mut pretty = if args.wants_pretty() {
            Some(pretty::Printer::new(&args)?)
        } else {
            None
        };
        let mut state = plain::State::default();
        let mut out = BufWriter::new(io::stdout().lock());
        let mut code = ExitCode::SUCCESS;

        for path in &args.files {
            // A missing file is reported, and the rest still get printed.
            let input = match common::open_input(path) {
                Ok(input) => input,
                Err(e) => {
                    out.flush()?;
                    eprintln!("meow: {}: {e}", path.display());
                    code = ExitCode::FAILURE;
                    continue;
                }
            };
            match &mut pretty {
                Some(printer) => printer.print(path, input, &mut out)?,
                None => plain::print_file(input, &args, &mut state, &mut out)?,
            }
        }

        out.flush()?;
        Ok(code)
    })
}
