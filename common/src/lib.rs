//! Shared helpers for every tool in this workspace.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::ExitCode;

/// Opens a file for buffered reading. `-` means stdin.
pub fn open_input(path: &Path) -> io::Result<Box<dyn BufRead>> {
    if path == Path::new("-") {
        Ok(Box::new(io::stdin().lock()))
    } else if path.is_dir() {
        Err(io::Error::other("is a directory"))
    } else {
        Ok(Box::new(BufReader::new(File::open(path)?)))
    }
}

/// Runs a tool's main logic and turns the result into an exit code.
///
/// Errors are printed as `name: message`. A closed pipe (for example
/// `meow big.log | head`) is a normal way to stop, so it exits quietly.
pub fn run(name: &str, f: impl FnOnce() -> anyhow::Result<ExitCode>) -> ExitCode {
    match f() {
        Ok(code) => code,
        Err(e) if is_broken_pipe(&e) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{name}: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn is_broken_pipe(e: &anyhow::Error) -> bool {
    e.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|io| io.kind() == io::ErrorKind::BrokenPipe)
    })
}
