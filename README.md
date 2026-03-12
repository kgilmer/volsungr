# Volsungr

A command-line tool that searches for the latest version of a crate that is compatible with a specified version of the Rust toolchain.

## Status

This tool is functional for its designed purpose.

# Build / Install

Use the Rust tooling to build and install:

```bash
git clone https://github.com/kgilmer/volsungr.git
cd volsungr/
cargo install --path .
```

# Usage

```bash
usage: volsungr --rustc-version <version> [--package <name>] [--directory <path>] [--brief]
```

## Options

| Flag | Short | Description |
|------|-------|-------------|
| `--rustc-version` | `-r` | Target rustc version (e.g., 1.70.0 or v1.70.0) **(required)** |
| `--package` | `-p` | Package name to query (can be specified multiple times) |
| `--directory` | `-d` | Directory containing a Cargo.toml file to extract dependencies from |
| `--brief` | `-b` | Suppress warnings and only print package name and compatible version |

**Note:** Either `--package` or `--directory` must be provided, but not both.

## Examples

### Search for a specific package

Search for the most recent version of `toml_datetime` that is compatible with version 1.63 of the Rust toolchain:

```bash
$ volsungr -r 1.63.0 -p toml_datetime
For rustc version "1.63.0", the latest compatible version: toml_datetime = "0.6.1"
```

### Extract dependencies from a Cargo.toml

Query all dependencies from a project's Cargo.toml:

```bash
$ volsungr -r 1.30.0 -d ../my-project
For rustc version "1.30.0", the latest compatible version: anyhow = "1.0.44"
For rustc version "1.30.0", the latest compatible version: serde = "1.0.152"
For rustc version "1.30.0", the latest compatible version: tokio = "0.2.25"
```

### Brief output mode

Use the `--brief` flag to suppress warnings and only print package names with compatible versions:

```bash
$ volsungr -r 1.63.0 -p toml_datetime --brief
toml_datetime = "0.6.1"

$ volsungr -r 1.30.0 -d ../my-project --brief
anyhow = "1.0.44"
serde = "1.0.152"
tokio = "0.2.25"
```

### Multiple packages

Query multiple packages at once:

```bash
$ volsungr -r 1.70.0 -p serde -p tokio
For rustc version "1.70.0", the latest compatible version: serde = "1.0.193"
For rustc version "1.70.0", the latest compatible version: tokio = "1.35.0"
```

### No compatible version found

When no compatible version exists:

```bash
$ volsungr -r 1.30.0 -p some-modern-crate
No versions of some-modern-crate are compatible with rustc version "1.30.0" (checked versions ["2.0.0", "2.1.0"])
```

With `--brief`:

```bash
$ volsungr -r 1.30.0 -p some-modern-crate --brief
some-modern-crate = "<none>"