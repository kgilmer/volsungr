# Volsungr

A command-line tool that searches for the latest version of a crate that is compatible with a specified version of the Rust toolchain.

## Status

This tool is functional for it's designed purpose.

# Build / Install

Use the Rust tooling to build and install:

```
git clone https://github.com/kgilmer/volsungr.git
cd volsungr/
cargo install --path .
```

# Usage

```
usage: volsungr <target rustc version> <package name>
```

Example; search for the most recent version of `toml_datetime` that is compatible with version 1.63 of the Rust toolchain.

```
$ volsungr 1.63.0 toml_datetime
Searching toml_datetime versions compatible with rust 1.63.0...
Latest compatible version of toml_datetime = "0.6.1"
```
