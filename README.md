# sks

Registry-based launcher for local scripts. `sks` loads explicit YAML registrations, lists and searches them, runs a selected script, through the CLI.

## Features

- Exact execution by Python-style ASCII name: `[A-Za-z_][A-Za-z0-9_]*`.
- YAML registry with imports, descriptions, and tags.
- Interactive picker with source preview and syntax highlighting.
- Ranked CLI search with YAML output and source paths.
- Pre-execution snapshot to `.sks/<filename>` in the current directory.
- Self-update from the latest GitHub Release with target-aware asset selection and checksum verification.
- Built-in instructions for script use and script creation.

## Requirements

- Rust 1.85 or newer when building from source.
- Python or another runtime required by each registered script.

## Installation

Build and install from source:

```bash
git clone https://github.com/gtiders/skillscripts.git
cd skillscripts
cargo install --path .
```

Initialize the global configuration:

```bash
sks init
```

The configuration directory is `~/.config/sks`. `init` creates `sks.yaml`, an empty `scripts.yaml`, and the `sks-script-use` and `sks-script-create` Agent Skills under `~/.agents/skills`.

## Usage

```bash
sks list
sks search "markdown pdf"
sks pick
sks run <name> [args...]
sks skill use
sks skill create
sks update
sks update --check
sks update --force
```

`run` matches `name` exactly. All arguments after the name are appended to the registered command. Before execution, the source file is copied to `.sks/<filename>` in the current directory; an existing file with the same name is replaced. The copy is made even if the command exits with an error.

`list` prints the effective registry as YAML. `pick` opens a card list showing each registration's name, path, command, comment, and tags beside a syntax-highlighted source preview. Search is fuzzy, and matched characters are highlighted. Use Up/Down or click a card to select it; Up/Down wraps between the first and last cards, while the card list ignores mouse scrolling. Use Enter to print the selected registration and Esc to cancel. Alt+Up/Down scrolls a card that is taller than the terminal. Source code wraps to the preview width; use the mouse wheel over the preview or Ctrl+D/Ctrl+B to scroll it. Card content is cached for the current query, theme, and width; preview loading and wrapping run in the background.

Picker and preview colors come from the same [Chromata](https://docs.rs/chromata/) Base24 theme. The default is `Catppuccin Frappe`. Run `sks themes` to list all available names. Set `picker.theme` in `~/.config/sks/sks.yaml` to change the theme. Example:

```yaml
picker:
  theme: Catppuccin Frappe
```

`update` queries the GitHub latest Release, selects the asset matching the binary's compiled Rust target (including GNU or musl), verifies `checksums.txt`, and replaces the current executable. `--check` does not install; `--force` installs even when the version comparison is inconclusive.

### Search

Run `sks search "<query>"` to get ranked matches as YAML. Each result includes its registered name, resolved source path, command, and any comment and tags. Search is read-only; use the exact name with `sks run <name> [args...]` to execute a match. `--limit N` changes the default of 5 results, and `--tag TAG` adds a repeatable ranking hint. No match prints `[]`.

## Configuration

Global file: `~/.config/sks/sks.yaml`

```yaml
imports:
  - scripts.yaml
  - imports/tools.yaml

scripts: []
```

Script registration:

```yaml
scripts:
  - name: ase_to_xyz
    path: tools/ase2xyz.py
    command: python {{path}}
    comment: Convert ASE-readable structure files to extended XYZ
    tags: [ase, structure, extxyz, conversion]
```

Rules:

- `name` is required, case-sensitive, and globally unique.
- A name must match `[A-Za-z_][A-Za-z0-9_]*`. Empty, Unicode, numeric-leading, dotted, dashed, slashed, and spaced names are invalid.
- `path` must be a relative Unix-style path. It is resolved relative to the YAML file that defines it.
- Only the global file may declare `imports`; imported files cannot import another file.
- `command` must contain `{{path}}`. The placeholder is replaced with the resolved script path.
- `comment` and `tags` are optional. Tags are used as search ranking hints.

## Common Issues

### `Global config not found`

Run `sks init`, then add registrations to `~/.config/sks/sks.yaml` or an imported YAML file.

### `invalid script name`

Rename the registration to an ASCII Python-style identifier, for example `convert_csv` or `_internal`.

### `unknown script name`

Run `sks list` and use the exact registered name. Matching is case-sensitive and does not use fuzzy guessing.

### `command` validation fails

Add `{{path}}` to the command. For example: `python {{path}}`.

### `sks update` cannot find an asset

The release must provide an archive for the binary's compiled target and a matching `checksums.txt`. Check the network connection and the available assets on the latest GitHub Release.
