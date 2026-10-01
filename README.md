# squigit-parser

A Rust workspace with placeholder document, audio, and video parsers. The dev
CLI routes by file extension and prints a category. It does not open or parse
files, so the supplied path does not need to exist yet.

## Layout

- `parser-rs/`: facade that routes paths to the appropriate crate.
- `crates/parser-pdf/`: PDF and Microsoft Office placeholders.
- `crates/parser-aud/`: audio placeholder.
- `crates/parser-vid/`: video placeholder.
- `xtask/`: repository tasks and the `dev` CLI.
- `.cargo/config.toml`: the `cargo xtask` alias.
- `rust-toolchain.toml`: stable Rust with Rustfmt and Clippy.

## Development

Install Rust using Rustup and run commands from this repository. Rustup reads
`rust-toolchain.toml` and selects the required toolchain and components.

```sh
cargo xtask doctor
cargo xtask fmt
cargo xtask fmt --all
cargo xtask dev path/to/file.mov
```

`doctor` checks required repository files, Git, Rust tooling, formatting, and
`cargo check --workspace`. `fmt` formats existing Rust files with staged,
unstaged, or untracked Git changes. `--all` formats every workspace crate.
Changed-file formatting does not traverse into unchanged child modules.

The dev CLI accepts exactly one path. Quote paths containing spaces. It can
also be run directly:

```sh
cargo run --package xtask --bin dev -- "path/to/my document.docx"
```

## Placeholder output

| File category | Examples | Output |
| --- | --- | --- |
| PDF / Microsoft Office | `.pdf`, `.docx`, `.pptx`, `.xlsx` | `this is a doc file` |
| Audio | `.mp3`, `.wav`, `.flac`, `.m4a` | `this is a aud file` |
| Video | `.mov`, `.mp4`, `.mkv`, `.webm` | `this is a vid file` |

Extensions are case-insensitive. The complete extension lists are in
`parser-rs/src/lib.rs`. Missing or unsupported extensions produce an error and
a nonzero exit status. No test suite or coverage requirement is configured.
