# sks

Minimal registry-driven script launcher and picker CLI.

## Overview

`sks` reads a single global config file at `~/.config/sks/sks.yaml`.  
Scripts are registered explicitly; the tool does not scan directories or parse script headers.

Each registered script has only three fields:

- `id`
- `path`
- `command`

`path` is resolved relative to the YAML file that defines it. Only the global config may declare `imports`.

## Config Format

Global config:

```yaml
imports:
  - lang/python.yaml

scripts:
  - id: 1
    path: scripts/hello.py
    command: python {{path}}
```

Imported config:

```yaml
scripts:
  - id: 2
    path: tools/build.py
    command: python {{path}}
```

Rules:

- only relative paths are allowed
- imported files cannot declare `imports`
- `id` must be globally unique
- `command` must contain `{{path}}`

## Commands

```bash
sks init
sks list
sks pick
sks run 1 foo --bar baz
```

- `init` creates `~/.config/sks/sks.yaml`
- `list` outputs all registered scripts as YAML
- `pick` opens the interactive picker with a table-style list and syntax-highlighted file preview
- `run <id> [args...]` replaces `{{path}}` in `command` and appends all remaining args

## Picker

`pick` shows three columns:

- `ID`
- `PATH`
- `COMMAND`

The preview pane renders the full script file with embedded `syntect` highlighting. The current default theme is GitHub Dark, with preview background handled by skim.

## Run Semantics

`run` is intentionally simple:

```bash
sks run 12 input.txt --mode fast
```

This means:

1. find script `id: 12`
2. replace `{{path}}` in `command`
3. append `input.txt --mode fast` to the command

`run` treats everything after `<id>` as passthrough arguments. It does not keep its own option parsing layer.

## Install

From source:

```bash
cargo install --path .
```
