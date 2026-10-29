use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

/// Writes a test file into Cargo's temp dir for integration tests.
fn fixture(name: &str, content: &str) -> String {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&path, content).unwrap();
    path.to_str().unwrap().to_owned()
}

fn meow(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_meow"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // meow may exit before reading stdin (e.g. on a bad flag), so a failed write is fine.
    let _ = child.stdin.take().unwrap().write_all(stdin.as_bytes());
    child.wait_with_output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

/// Runs meow with pretty output forced on, at a fixed width.
fn pretty(args: &[&str], stdin: &str) -> String {
    let mut all = vec!["--color", "always", "--width", "60"];
    all.extend_from_slice(args);
    let out = meow(&all, stdin);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    stdout(&out)
}

/// Drops ANSI color codes, leaving what a person would read.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[test]
fn prints_file_unchanged() {
    let file = fixture("plain.txt", "a\nb\n");
    let out = meow(&[&file], "");
    assert!(out.status.success());
    assert_eq!(stdout(&out), "a\nb\n");
}

#[test]
fn reads_stdin_without_files() {
    assert_eq!(stdout(&meow(&[], "hi\n")), "hi\n");
    assert_eq!(stdout(&meow(&["-"], "hi\n")), "hi\n");
}

#[test]
fn numbers_all_lines() {
    let out = meow(&["-n"], "a\n\nb\n");
    assert_eq!(stdout(&out), "     1\ta\n     2\t\n     3\tb\n");
}

#[test]
fn numbers_only_nonblank_lines() {
    let out = meow(&["-b"], "a\n\nb\n");
    assert_eq!(stdout(&out), "     1\ta\n\n     2\tb\n");
}

#[test]
fn squeezes_blank_lines() {
    let out = meow(&["-s"], "a\n\n\n\nb\n");
    assert_eq!(stdout(&out), "a\n\nb\n");
}

#[test]
fn show_ends_keeps_crlf() {
    let out = meow(&["-E"], "a\r\nb");
    assert_eq!(stdout(&out), "a$\r\nb$");
}

#[test]
fn numbering_continues_across_files() {
    let one = fixture("one.txt", "a\n");
    let two = fixture("two.txt", "b\n");
    let out = meow(&["-n", &one, &two], "");
    assert_eq!(stdout(&out), "     1\ta\n     2\tb\n");
}

#[test]
fn missing_file_fails_but_prints_the_rest() {
    let good = fixture("good.txt", "ok\n");
    let out = meow(&["no-such-file.txt", &good], "");
    assert!(!out.status.success());
    assert_eq!(stdout(&out), "ok\n");
    assert!(String::from_utf8_lossy(&out.stderr).contains("no-such-file.txt"));
}

#[test]
fn directory_is_an_error() {
    let out = meow(&[env!("CARGO_TARGET_TMPDIR")], "");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("is a directory"));
}

#[test]
fn handles_bangla_text() {
    let out = meow(&["-n"], "আমি বাংলায় গান গাই\n");
    assert_eq!(stdout(&out), "     1\tআমি বাংলায় গান গাই\n");
}

#[test]
fn piped_output_stays_plain() {
    // Tests run with stdout piped, so auto mode must not add colors.
    assert_eq!(stdout(&meow(&[], "# hi\n")), "# hi\n");
}

#[test]
fn plain_flag_wins_over_color() {
    assert_eq!(
        stdout(&meow(&["--color", "always", "-p"], "# hi\n")),
        "# hi\n"
    );
}

#[test]
fn renders_markdown() {
    let raw = pretty(
        &["-l", "md"],
        "# Title\n\nHello **world** and `code`.\n\n- one\n- two\n\n> quoted\n",
    );
    let text = strip_ansi(&raw);
    assert!(raw.contains("\x1b["), "expected colors");
    assert!(text.contains("Title\n━━━━━\n"));
    assert!(text.contains("Hello world and code."));
    assert!(text.contains("  • one\n  • two\n"));
    assert!(text.contains("│ quoted"));
    assert!(!text.contains("**") && !text.contains("# Title"));
}

#[test]
fn detects_markdown_by_extension() {
    let file = fixture("notes.md", "## Plan\n");
    let text = strip_ansi(&pretty(&[&file], ""));
    assert!(text.contains("Markdown · 1 line"));
    assert!(text.contains("Plan\n────\n"));
}

#[test]
fn renders_markdown_tables() {
    let text = strip_ansi(&pretty(
        &["-l", "md"],
        "| Name | Age |\n|------|----:|\n| Rahim | 7 |\n",
    ));
    assert!(text.contains("│ Name  │ Age │"));
    assert!(text.contains("│ Rahim │   7 │"));
}

#[test]
fn wraps_wide_tables_to_fit() {
    let long = "a very long description that will not fit in one table row";
    let text = strip_ansi(&pretty(
        &["-l", "md"],
        &format!("| Tool | Info |\n|---|---|\n| meow | {long} |\n"),
    ));
    for line in text.lines() {
        assert!(line.chars().count() <= 60, "too wide: {line:?}");
    }
    assert!(text.contains("│ meow │ a very long"));
}

#[test]
fn renders_task_lists_and_links() {
    let text = strip_ansi(&pretty(
        &["-l", "md"],
        "- [x] done\n- [ ] todo\n\nSee [docs](https://example.com).\n",
    ));
    assert!(text.contains("☑ done"));
    assert!(text.contains("☐ todo"));
    assert!(text.contains("docs (https://example.com)"));
}

#[test]
fn highlights_markdown_code_blocks() {
    let raw = pretty(&["-l", "md"], "```rust\nfn main() {}\n```\n");
    assert!(raw.contains("\x1b[38;2;"), "expected 24-bit colors");
    let text = strip_ansi(&raw);
    assert!(text.contains("╭─ rust ─"));
    assert!(text.contains("│ fn main() {}"));
}

#[test]
fn wraps_long_paragraphs() {
    let text = strip_ansi(&pretty(&["-l", "md"], &"word ".repeat(40)));
    for line in text.lines() {
        assert!(line.chars().count() <= 60, "too wide: {line:?}");
    }
    assert!(text.lines().filter(|l| l.starts_with("word")).count() > 1);
}

#[test]
fn highlights_code_by_extension() {
    let file = fixture("hello.rs", "fn main() {}\n");
    let raw = pretty(&[&file], "");
    assert!(raw.contains("\x1b[38;2;"), "expected 24-bit colors");
    let text = strip_ansi(&raw);
    assert!(text.contains("Rust · 1 line"));
    assert!(text.contains("  1 │ fn main() {}"));
}

#[test]
fn colors_log_levels_in_text() {
    let file = fixture("app.log", "ERROR disk full at https://example.com\n");
    let raw = pretty(&[&file], "");
    assert!(raw.contains("ERROR\x1b[0m"), "expected ERROR to be colored");
    let text = strip_ansi(&raw);
    assert!(text.contains("Text · 1 line"));
    assert!(text.contains("  1 │ ERROR disk full at https://example.com"));
}

#[test]
fn hides_binary_files() {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("blob.bin");
    std::fs::write(&path, [0u8, 1, 2, 3]).unwrap();
    let text = strip_ansi(&pretty(&[path.to_str().unwrap()], ""));
    assert!(text.contains("binary file not shown"));
}

#[test]
fn unknown_theme_is_an_error() {
    let out = meow(&["--color", "always", "--theme", "nope"], "x");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown theme"));
}
