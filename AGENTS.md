# Repository Guidelines

## Project Structure & Module Organization

`squigit-parser` is a Rust workspace. `parser-rs/` is the public facade;
`crates/parser-pdf/` and `crates/parser-vid/` implement PDF/Microsoft Office and
video parsing. `crates/parser-common/` owns collage encoding, output directories,
and manifests. Each library starts in its crate's `src/lib.rs`.
`xtask/src/main.rs` implements repository tasks, and `xtask/src/bin/dev.rs` is
the development CLI. `.cargo/config.toml` defines the Cargo alias; `rust-toolchain.toml`
selects stable Rust with Rustfmt and Clippy. No test suite is configured.

## Build, Test, and Development Commands

Run commands from the repository with Rustup installed:

- `cargo xtask doctor`: check repository files, tooling, formatting, and build.
- `cargo xtask fmt`: format staged, unstaged, and untracked Rust files.
- `cargo xtask fmt --all`: format all workspace crates.
- `cargo xtask dev <path> --from <start> --to <end> --output <directory>`: run
  through the facade; video also requires `--jump <milliseconds>`.
- `cargo check --workspace`: compile-check every crate.
- `cargo clippy --workspace -- -D warnings`: check for lint warnings.

See `README.md` for tool arguments and output formats. No test command is configured.

## Coding Style & Naming Conventions

Use Rust 2024, four-space indentation, and Rustfmt's standard formatting.
Use `snake_case` for functions and modules, `PascalCase` for types, and
`SCREAMING_SNAKE_CASE` for constants. Cargo package names use hyphens; Rust imports
use underscores, such as `parser_rs`. Keep dispatch in the facade and
category-specific behavior in its parser crate. Share layout and output handling
in `parser-common`; keep provider requests outside this local parsing workspace.

For Markdown, use ATX headings (`## Section`), backticks around commands and
paths, and short, actionable paragraphs.

## Testing Guidelines

No testing framework, coverage threshold, or test naming convention is
established. Do not add tests unless explicitly requested. When tests are
requested, use descriptive case names identifying the input and expected
behavior, and document the command needed to run them.

## Commit & Pull Request Guidelines

Use concise, imperative commit subjects with a change-type prefix, such as
`docs: add contributor guide` or `feat: parse identifiers`.

Keep pull requests focused. Describe the change, its purpose, and validation
performed; link relevant issues when available. Report unavailable checks
accurately.

## Agent-Specific Instructions

- Never use subagents.
- Do not add tests unless the user asks for them.
- During refactors, do not retain legacy code to support old formats.
