//! Plain output, byte for byte like `cat`, with its classic flags.

use std::io::{self, BufRead, Write};

use crate::Args;

/// Carried across files, so numbering continues like in `cat`.
#[derive(Default)]
pub struct State {
    line_no: u64,
    prev_blank: bool,
}

pub fn print_file(
    mut input: impl BufRead,
    args: &Args,
    state: &mut State,
    out: &mut impl Write,
) -> io::Result<()> {
    if !args.has_cat_flags() {
        io::copy(&mut input, out)?;
        return Ok(());
    }

    // Read raw bytes, so non-UTF-8 files still print.
    let mut line = Vec::new();
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            return Ok(());
        }
        let (body, ending) = split_ending(&line);
        let blank = body.is_empty();

        if args.squeeze_blank && blank && state.prev_blank {
            continue;
        }
        state.prev_blank = blank;

        let numbered = if args.number_nonblank {
            !blank
        } else {
            args.number
        };
        if numbered {
            state.line_no += 1;
            write!(out, "{:>6}\t", state.line_no)?;
        }

        out.write_all(body)?;
        if args.show_ends {
            out.write_all(b"$")?;
        }
        out.write_all(ending)?;
    }
}

/// Splits a line into its text and its ending (`\n`, `\r\n` or nothing).
fn split_ending(line: &[u8]) -> (&[u8], &[u8]) {
    let len = if line.ends_with(b"\r\n") {
        2
    } else if line.ends_with(b"\n") {
        1
    } else {
        0
    };
    line.split_at(line.len() - len)
}
