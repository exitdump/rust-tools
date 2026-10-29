# rust-tools

[![CI](https://github.com/exitdump/rust-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/exitdump/rust-tools/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/exitdump/rust-tools)](https://github.com/exitdump/rust-tools/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Small, fast command-line tools written in Rust. Each tool is **one file**: no runtime, nothing else to install. They work on **Windows, macOS and Linux**.

🌐 **Website:** https://exitdump.github.io/rust-tools/

## Tools

| Tool | Like | What it does |
|---|---|---|
| [`meow`](tools/meow) | `cat` | Prints files beautifully: rendered Markdown, highlighted code, colored logs |

More tools are on the way. [Want to build one?](CONTRIBUTING.md)

## Download

Pick your system, unzip, and run. These links always point to the latest release.

| System | meow |
|---|---|
| Windows (x64) | [meow-windows-x64.zip](https://github.com/exitdump/rust-tools/releases/latest/download/meow-windows-x64.zip) |
| macOS (Apple Silicon) | [meow-macos-arm64.tar.gz](https://github.com/exitdump/rust-tools/releases/latest/download/meow-macos-arm64.tar.gz) |
| macOS (Intel) | [meow-macos-x64.tar.gz](https://github.com/exitdump/rust-tools/releases/latest/download/meow-macos-x64.tar.gz) |
| Linux (x64) | [meow-linux-x64.tar.gz](https://github.com/exitdump/rust-tools/releases/latest/download/meow-linux-x64.tar.gz) |

All files, plus `SHA256SUMS.txt` checksums, are on the [Releases page](https://github.com/exitdump/rust-tools/releases).

## Install from the terminal

Each example installs `meow`. For another tool, change the name.

**Windows** (PowerShell):

```powershell
iwr https://github.com/exitdump/rust-tools/releases/latest/download/meow-windows-x64.zip -OutFile meow.zip
Expand-Archive meow.zip -DestinationPath "$HOME\bin" -Force
$p = [Environment]::GetEnvironmentVariable("Path", "User"); [Environment]::SetEnvironmentVariable("Path", "$p;$HOME\bin", "User")
```

Then open a new terminal and run `meow --help`.

**macOS** (use `macos-x64` on Intel Macs):

```sh
curl -fsSL https://github.com/exitdump/rust-tools/releases/latest/download/meow-macos-arm64.tar.gz | tar xz meow
sudo mv meow /usr/local/bin/
```

**Linux:**

```sh
curl -fsSL https://github.com/exitdump/rust-tools/releases/latest/download/meow-linux-x64.tar.gz | tar xz meow
sudo mv meow /usr/local/bin/
```

**With Rust installed** (any system):

```sh
cargo install --git https://github.com/exitdump/rust-tools meow
```

> **macOS says the app can't be checked?** Files downloaded in a browser get a quarantine flag. Remove it with `xattr -d com.apple.quarantine meow`.

## Build from source

```sh
git clone https://github.com/exitdump/rust-tools
cd rust-tools
cargo build --release   # binaries land in target/release/
cargo test              # run every tool's tests
```

## Contributing

New tools, bug fixes and doc improvements are all welcome. [CONTRIBUTING.md](CONTRIBUTING.md) walks you through it step by step, even if it's your first Rust project.

## License

[MIT](LICENSE) © Jahid Hasan ([@exitdump](https://github.com/exitdump))
