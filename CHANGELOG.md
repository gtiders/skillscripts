# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.0.10] - 2026-06-16

### Changed

- Simplified the codebase into a flat `cli` / `picker` / `registry` layout.
- Extracted `run` parsing and execution into a dedicated module.
- Removed legacy scanning, metadata parsing, and old command layers that no longer match the registry-driven design.
- Collapsed integration tests into a single `tests/cli.rs` file.
- Trimmed unused dependencies, obsolete test fixtures, and stale source directories.
- Updated the release workflow to build and publish only the `sks` binary.

## [1.0.8] - 2026-05-05

### Changed

- Repositioned project as dual-mode tool: fast script search and skill retrieval for AI Agents
- Removed bundled skills directory (`skills/`) - users now manage their own script collections
- Updated README to emphasize dual-mode capabilities with clear use cases
- Simplified documentation by removing architecture, dependencies, and development sections
- Changed all JSON output to YAML for better consistency and Agent tool integration
- Removed `tool_name` field from skill metadata
- Renamed `sync` command to instant scanning with in-memory caching
- Removed `comfy-table` and `syntect` dependencies for lighter binary

### Fixed

- Improved configuration merging for scan paths with proper defaults
- Enhanced error reporting with optional parse error display

## [1.0.5] - 2026-03-27

### Added

- New `sks config` command to print three config snapshots: default, current-directory local, and effective merged config.
- New integration test coverage for config snapshot output (`cli_config_show`).

### Changed

- Reworked bundled skills into standardized `SKILL.md` format under:
  - `skills/sks_builder/SKILL.md`
  - `skills/sks_usager/SKILL.md`
- README and README_zh were rewritten and aligned to emphasize:
  - fast skill retrieval (`skill find`) for agent loops
  - deep agent/runtime JSON integration
  - script-based skills beyond Markdown-only systems
- Repository docs were consolidated into README files (removed standalone `docs/` pages).

### Fixed

- Release workflow packaging verification now checks current bundled skill paths (`skills/sks_builder/SKILL.md`, `skills/sks_usager/SKILL.md`) so tag-triggered releases pass asset validation.

## [1.0.4] - 2026-03-24

### Fixed

- `scan_paths` now expands `~` and `~/...` to the user home directory when loading config.
- `sync` now warns (without failing) when a configured scan path does not exist.
- Added i18n messages for missing scan path warnings in `en` and `zh-CN`.

## [1.0.3] - 2026-03-24

### Fixed

- Release workflow now packages the entire `skills/` directory (recursive) instead of file-level includes only.
- Added directory-level verification (`test -d skills`) before archive upload.

## [1.0.2] - 2026-03-24

### Fixed

- Release packaging now explicitly includes `README.md`, `README_zh.md`, `LICENSE`, and bundled `skills/*` files.
- Added packaging-asset verification step in release workflow to prevent missing files in future release archives.

## [1.0.1] - 2026-03-24

### Added

- Project-level `skills/` pack with one-skill-per-folder layout for easier runtime portability
- Extended integration docs for `scan_paths` and system-prompt-first tool routing
- Additional docs pages for positioning, script skillization, runtime integration, and packaging

### Changed

- `init` global seeding now copies skill files recursively from bundled/current `skills/`
- Unified default config/cache paths to `~/.config/sks` and `~/.cache/sks`
- README and README_zh restructured around product positioning, pain points, and runtime copy flow

### Added

- Layered project structure split into `app`, `domain`, `infra`, and `presentation`
- Stable, legal, deduplicated tool identifiers for `search --json`
- Agent-ready JSON output with TTY-aware syntax highlighting
- Structured integration test suite based on `assert_cmd`, `predicates`, and `tempfile`
- Bilingual documentation with `README.md` and `README_zh.md`

### Changed

- Polished CLI help text and user-facing copy across CLI, TUI, and localized messages
- Refined error messages to be more human-friendly and action-oriented
- Split CLI output, sync pipeline, and TUI internals into smaller focused modules

### Fixed

- Preserved plain JSON output in non-TTY environments for Agent and pipeline compatibility
- Improved fallback handling for missing or corrupted local indexes
- Hardened strict-mode sync behavior around invalid headers and duplicate `tool_name` values
