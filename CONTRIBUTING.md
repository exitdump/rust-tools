# Contributing

Thanks for helping! You don't need to be a Rust expert: small fixes and doc improvements matter as much as new tools.

## Ways to help

- 🐛 **Report a bug:** [open an issue](https://github.com/exitdump/rust-tools/issues/new) with the command you ran, what you expected, and what happened. Say which system you're on.
- 💡 **Suggest a tool:** open an issue saying what it would do and which Unix tool it's like (if any).
- 📝 **Improve the docs:** typos, unclear steps, missing examples, all welcome.
- 🛠️ **Build a tool:** follow the steps below.

## 1. Set up (5 minutes)

1. Install Rust from [rustup.rs](https://rustup.rs). On Windows, choose the default (MSVC) option and let it install the Visual Studio C++ Build Tools.
2. [Fork the repo](https://github.com/exitdump/rust-tools/fork), then clone your fork:

   ```sh
   git clone https://github.com/<your-name>/rust-tools
   cd rust-tools
   ```

3. Check that everything works:

   ```sh
   cargo test
   ```

## 2. Find your way around

```
rust-tools/
├── common/            shared helpers every tool uses
├── tools/
│   └── meow/          one folder per tool
│       ├── src/main.rs
│       ├── tests/cli.rs
│       └── README.md
├── site/              the website (plain HTML, no build step)
└── .github/workflows/ CI and release automation
```

Run a tool while you work on it:

```sh
cargo run -p meow -- README.md
```

Everything after `--` goes to the tool.

## 3. Add a new tool

Say you're building `count`, a friendly `wc`.

**Create it:**

```sh
cargo new tools/count
```

**Use the shared settings.** Replace `tools/count/Cargo.toml` with:

```toml
[package]
name = "count"
description = "Count lines, words and characters, like wc"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
common.workspace = true
```

**Start from this `src/main.rs`:**

```rust
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Count lines, words and characters, like `wc`.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Files to read. `-` or no file means stdin.
    #[arg(default_value = "-")]
    files: Vec<PathBuf>,
}

fn main() -> ExitCode {
    common::run("count", || {
        let args = Args::parse();
        for path in &args.files {
            let input = common::open_input(path)?;
            // ... your tool's work here
        }
        Ok(ExitCode::SUCCESS)
    })
}
```

`common::run` prints errors nicely and exits quietly when the output pipe closes (`count big.txt | head`). `common::open_input` reads a file, or stdin for `-`.

**Add tests** in `tools/count/tests/cli.rs`. Copy the helpers at the top of [`tools/meow/tests/cli.rs`](tools/meow/tests/cli.rs); they run your tool like a user would.

**Add it to the list** in two places:

- the **Tools** table in [README.md](README.md)
- the `TOOLS` list in [site/tools.js](site/tools.js) (copy the `meow` entry)

Also add a short `tools/count/README.md`, like meow's.

That's it. The release workflow builds every folder in `tools/` automatically, for every system.

### What makes a good tool here

- **Works on Windows, macOS and Linux.** Use `Path`/`PathBuf` instead of joining strings with `/`.
- **Plays well with pipes.** Read stdin when there's no file. Keep output plain (no colors) when it isn't a terminal.
- **Has `--help`.** `clap` gives you this from the doc comments.
- **Uses exit codes.** `0` when it worked, `1` when something failed.
- **Stays small.** Add a dependency only when it really saves work.

## 4. Check before you push

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs the same checks on Windows, macOS and Linux for every pull request.

## 5. Open a pull request

```sh
git checkout -b add-count
git add .
git commit -m "Add count, a friendly wc"
git push -u origin add-count
```

Then open a pull request on GitHub. Say what the change does and how you tested it. Screenshots of terminal output are great.

## Website

The site is plain HTML in [`site/`](site). Open `site/index.html` in a browser to preview it. It's published automatically when `site/` changes on `main`.

## Releasing (maintainers)

1. Bump `version` in the root `Cargo.toml` and run `cargo build` so `Cargo.lock` updates.
2. Commit, then tag and push:

   ```sh
   git tag v0.2.0
   git push origin main v0.2.0
   ```

3. The **Release** workflow builds every tool for every system and publishes a GitHub Release. The website's download links always point to the latest one.
