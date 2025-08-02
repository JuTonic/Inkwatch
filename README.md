# Inkwatch

**Inkwatch** is a background CLI tool that watches a directory for changes to SVG files and automatically exports them to PDF and LaTeX overlay files using [Inkscape](https://inkscape.org). It is designed to be used alongside the [inkfig.nvim](https://github.com/JuTonic/inkfig.nvim) plugin to automate figure workflows in LaTeX documents.

## Usage

To start watching the `./figures` directory for changes, simply run:

```bash
inkscape ./figures
```

Whenever you create or save a file like:
```bash
./figures/picture.svg
```
inkwatch will automatically generate overlay files:
```bash
./figures/picture.pdf
./figures/picture.pdf_tex
```
If you delete `picture.svg`, the corresponding `.pdf` and `.pdf_tex` files will be deleted.

Avaliable options are:

```
inkwatch [DIR] (OPTIONS)

[DIR] - REQUIRED
  directory to watch for changes

(OPTIONS)
  --help or -h            Print this message
  --version or -v         Print version
  --inkscape-path [PATH]  Set path to inkscape binary
  --aux-prefix [STR]      Set prefix added to pdf and pdf_tex files
  --do-not-regenerate     Do not regenerate pdf and pdf_tex files at first run
  --not-recursively       Do not watch for changes in subdirectories
```

## Build

Prerequisites:
- Rust toolchain (see [rustup](https://rustup.rs/))

Clone the repo and build it:

```bash
git clone https://github.com/JuTonic/inkwatch.git
cd inkwatch
cargo build --release
```
The binary will be created at `./target/release/inkwatch`.

To install system-wide run:

```bash
cargo install --path .
```
Make sure that `~/.cargo/bin` is in your `PATH`
