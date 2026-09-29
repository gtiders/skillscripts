# Repository Guidelines

## Project Structure

Runtime code is organized by responsibility:

- `src/main.rs` — process entry point and top-level error reporting
- `src/cli.rs` — CLI parsing, command dispatch, and YAML output
- `src/registry.rs` — YAML loading, imports, validation, path resolution, and registry models
- `src/run_command.rs` — `sks run <name> [args...]`, snapshots, command expansion, and execution
- `src/picker.rs` — interactive card picker and syntax-highlighted preview
- `src/search.rs` — fuzzy script search and ranking
- `src/skill.rs` — `sks skill use/create` output
- `src/update.rs` — GitHub Release lookup, target selection, checksum verification, and replacement
- `src/init.rs` — configuration and Agent Skill initialization
- `src/portable_path.rs` — platform-independent configuration paths
- `assets/skills/sks-script-use/SKILL.md` — guidance for discovering and running scripts
- `assets/skills/sks-script-create/SKILL.md` — guidance for writing and registering scripts
- `tests/cli.rs` — integration tests and shared test helpers

Keep build output in `target/`; it is not versioned.

## Configuration and Naming

The global configuration is `~/.config/sks/sks.yaml`. Script registrations use `name`, `path`, `command`, optional `comment`, and optional `tags` fields.

Names must match the Python-style ASCII rule `[A-Za-z_][A-Za-z0-9_]*`, are case-sensitive, and are globally unique. Paths must be relative Unix-style paths. Commands must contain the `{{path}}` placeholder. Do not reintroduce numeric IDs, aliases, fuzzy execution, or compatibility parsing for the old `id` field.

`sks run <name> [args...]` matches names exactly, passes through all remaining arguments, and copies the registered source to `.sks/<filename>` in the current working directory before execution. The copy is replaced when the filename already exists.

## Development Commands

```bash
cargo build
cargo run -- --help
cargo run -- list
cargo test
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

Use `cargo run -- list` with the real global configuration only for local manual checks. Tests should use temporary configuration through the existing `TestEnv` helper.

## Coding Style

Use standard Rust formatting and four-space indentation. Use `snake_case` for functions, modules, and tests; `UpperCamelCase` for types; and concise names that describe one responsibility. Prefer direct data flow and small functions. Avoid introducing deep application-layer nesting without a concrete need.

## Testing

Group tests by user-visible behavior. Cover successful and failing cases for:

- valid and invalid script names
- duplicate names and configuration errors
- list and picker YAML output
- exact `run` matching, argument passthrough, path replacement, and `.sks` snapshots
- CLI search results and ranking
- skill command output
- update checks where network-independent tests are possible

Use `assert_cmd`, `predicates`, and temporary directories. Run the full validation set before submitting changes:

```bash
cargo fmt --all
cargo test
cargo clippy --all-targets --all-features -- -D warnings
git diff --check
```

## Commits and Pull Requests

Use short imperative commit subjects, commonly prefixed with `fix:`, `refactor:`, `chore:`, or `release:`. Keep each commit limited to one logical change.

Pull requests should include a concise behavior summary and the relevant `cargo test` and Clippy results. Include screenshots only when changing the picker interface.
