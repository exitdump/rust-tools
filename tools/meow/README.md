# meow

`cat`, but nice to read. In a terminal, `meow` shows each file in the best way for its type:

- **Markdown** (`.md`) is rendered: colored headings, bullet lists, task lists, quotes, boxed tables and highlighted code blocks, wrapped to your terminal width.
- **Code** gets syntax highlighting and line numbers (Rust, Python, JavaScript, JSON, YAML, HTML, shell and many more).
- **Text and logs** get line numbers, plus colors for `ERROR` / `WARN` / `INFO`, `TODO`, URLs, dates, numbers and quoted strings.
- Each file gets a header with its name, type, line count and size. Binary files aren't dumped to the screen.

When its output is piped (`meow file | grep x`), `meow` prints the plain bytes exactly like `cat`, so scripts keep working.

## Install

See [Download](../../README.md#download) in the main README.

## Usage

```sh
meow README.md               # rendered Markdown
meow src/main.rs app.log     # highlighted code, then a colored log
meow -l json < data.txt      # choose the language yourself
meow -p notes.md             # plain output, like cat
meow -n -s file.txt          # classic cat flags still work (plain output)
```

## Options

| Flag | What it does |
|---|---|
| `-p`, `--plain` | Plain output, like `cat` |
| `--color auto\|always\|never` | When to use pretty output. `auto` (default) is pretty in a terminal, plain when piped or when `NO_COLOR` is set |
| `-l`, `--language <LANG>` | Language instead of guessing: `md`, `rust`, `json`, `py`, … |
| `--theme <NAME>` | Code color theme (default `base16-ocean.dark`). Pass a wrong name to see the list |
| `--width <N>` | Output width (default: terminal width, up to 120) |
| `-n`, `-b`, `-s`, `-E` | Classic `cat` flags: number lines, number non-blank lines, squeeze blank lines, show line ends. These switch to plain output |

## Known limits

- TypeScript and TOML aren't in the built-in language list yet, so they're shown as colored text.
- Very long code lines aren't wrapped by `meow`; your terminal wraps them.
