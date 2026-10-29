# rust-tools

Small command-line tools written in Rust. Each one is a single binary that runs on Linux, macOS and Windows.

## Tools

| Tool | Like | What it does |
|---|---|---|
| [`meow`](tools/meow) | `cat` | Print files beautifully: rendered Markdown, highlighted code, colored logs |

## meow

In a terminal, `meow` makes files easy to read:

- **Markdown** (`.md`) is rendered: colored headings, bullet lists, task lists, quotes, boxed tables and highlighted code blocks, wrapped to your terminal width.
- **Code** gets syntax highlighting and line numbers (Rust, Python, JS, JSON, YAML, shell and many more).
- **Text and logs** get line numbers, plus colors for `ERROR`/`WARN`/`INFO`, `TODO`, URLs, dates, numbers and quoted strings.
- Each file gets a header with its name, type, line count and size. Binary files aren't dumped to the screen.

When piped (`meow file | grep x`), it prints plain bytes exactly like `cat`, so scripts keep working.

```sh
meow README.md               # rendered Markdown
meow src/main.rs app.log     # highlighted code, colored log
meow -l json < data.txt      # choose the language yourself
meow -p notes.md             # plain output, like cat
meow -n -s file.txt          # classic cat flags still work (plain output)
```

| Flag | What it does |
|---|---|
| `-p`, `--plain` | Plain output, like `cat` |
| `--color auto\|always\|never` | When to use pretty output (`auto` respects `NO_COLOR`) |
| `-l`, `--language` | Language instead of guessing: `md`, `rust`, `json`, … |
| `--theme` | Code color theme (default `base16-ocean.dark`) |
| `--width` | Output width (default: terminal width, up to 120) |
| `-n`, `-b`, `-s`, `-E` | Classic `cat` flags: number lines, number non-blank, squeeze blanks, show ends |

## Install

```sh
cargo install --git https://github.com/exitdump/rust-tools meow
```

## Build from source

```sh
cargo build --release   # binaries land in target/release/
cargo test              # run every tool's tests
```

## Layout

```
common/        shared helpers (stdin handling, errors, broken pipes)
tools/<name>/  one folder per tool
```

## License

MIT
