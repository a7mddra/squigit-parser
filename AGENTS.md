# Repository Guidelines

## Project Structure & Module Organization

`squigit-parser` is a Rust workspace. `parser-rs/` is the public facade;
`crates/parser-pdf/`, `crates/parser-aud/`, and `crates/parser-vid/` contain
category-specific placeholders. Each library lives in its crate's `src/lib.rs`.
`xtask/src/main.rs` implements repository tasks, and `xtask/src/bin/dev.rs` is
the development CLI. `.cargo/config.toml` defines the Cargo alias; `rust-toolchain.toml`
selects stable Rust with Rustfmt and Clippy. No tests or assets exist yet.

## Build, Test, and Development Commands

Run commands from the repository with Rustup installed:

- `cargo xtask doctor`: check repository files, tooling, formatting, and build.
- `cargo xtask fmt`: format staged, unstaged, and untracked Rust files.
- `cargo xtask fmt --all`: format all workspace crates.
- `cargo xtask dev path/to/file.mov`: run the CLI through the facade.
- `cargo check --workspace`: compile-check every crate.
- `cargo clippy --workspace -- -D warnings`: check for lint warnings.

See `README.md` for usage and placeholder output. No test command is configured.

## Coding Style & Naming Conventions

Use Rust 2024, four-space indentation, and Rustfmt's standard formatting.
Use `snake_case` for functions and modules, `PascalCase` for types, and
`SCREAMING_SNAKE_CASE` for constants. Cargo package names use hyphens; Rust imports
use underscores, such as `parser_rs`. Keep dispatch in the facade and
category-specific behavior in its parser crate. The current implementations
only print categories; do not add parsing logic without a request.

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
