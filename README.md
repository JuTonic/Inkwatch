# Inktex-watch

**Inktex-watch** is a background CLI tool that watches a directory for changes to SVG files and automatically exports them to PDF and LaTeX overlay files using [Inkscape](https://inkscape.org). It is designed to be used alongside the [inktex.nvim](https://github.com/JuTonic/inktex.nvim) plugin to automate figure workflows in LaTeX documents.

## Usage

To start watching the `./figures` directory for changes, simply run:

```bash
inktex-watch ./figures
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
inktex-watch [DIR] (OPTIONS)

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

Run:
```
cargo install inktex-watch
```

Or build locally:

```bash
git clone https://github.com/JuTonic/inktex-watch.git
cd inktex-watch
cargo build --release
```
The binary will be created at `./target/release/inktex-watch`.

To install system-wide run:

```bash
cargo install --path .
```
Make sure that `~/.cargo/bin` is in your `PATH`
