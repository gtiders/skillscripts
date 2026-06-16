# Repository Guidelines

## Project Structure & Module Organization
The codebase is intentionally flat. Runtime logic lives in four source files:

- `src/main.rs` for process entry and top-level error reporting
- `src/cli.rs` for command dispatch and YAML output
- `src/run_command.rs` for `sks run <id> [args...]`
- `src/picker.rs` for the skim UI and syntect preview rendering
- `src/registry.rs` for config loading, validation, path resolution, and registry models

Integration tests live in `tests/`. Shared helpers are in `tests/support/mod.rs`. Keep build output in `target/` out of version control.

## Build, Test, and Development Commands
- `cargo build` builds the binary in debug mode.
- `cargo run -- --help` shows the current CLI surface.
- `cargo run -- list` prints registered scripts from the real global config.
- `cargo test` runs the full integration test suite.
- `cargo clippy --all-targets --all-features -- -D warnings` enforces a clean lint baseline.
- `cargo fmt --all` formats the repository.

## Coding Style & Naming Conventions
Use standard Rust formatting with 4-space indentation. Prefer small functions and direct data flow over extra abstraction layers. Follow existing naming:

- `snake_case` for functions, modules, and tests
- `UpperCamelCase` for types
- concise module names that reflect one responsibility

Avoid rebuilding deep `app/domain/infra` style nesting unless the project genuinely grows into it.

## Testing Guidelines
Group tests by behavior, not by internal module names. Current suites are:

- `tests/init_cli.rs`
- `tests/list_pick_cli.rs`
- `tests/registry_validation_cli.rs`
- `tests/run_cli.rs`

Use `assert_cmd`, `predicates`, and temp directories through `TestEnv`. Cover both happy paths and validation failures.

## Commit & Pull Request Guidelines
Recent history uses short imperative subjects, often with prefixes like `fix:`, `refactor:`, `chore:`, or `release:`. Keep each commit scoped to one logical change.

PRs should include:

- a concise behavior summary
- relevant command output for `cargo test` and `cargo clippy`
- screenshots only when the picker UI changes
