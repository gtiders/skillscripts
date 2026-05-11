# skillscripts

Fast script search and skill retrieval CLI.

## What It Is

`skillscripts` (alias `sks`) is a local-first command-line tool with two core capabilities:

**Fast Script Search**:
- You have scripts scattered everywhere and can't find them
- You want to quickly locate a script by functionality
- You need a lightweight script manager

**Fast Skill Retrieval**:
- You are an AI Agent developer managing a skill library
- You want to quickly retrieve reusable skills
- You need to provide tool-calling capabilities for Agents

## Core Features

### Dual-Mode Search

The same command serves two scenarios:

**Script Search Mode**:
- Quickly locate script files by functionality
- Lightweight script management without complex databases
- Instant results with fuzzy matching

**Skill Retrieval Mode**:
- Retrieve reusable skills for AI Agents
- Output YAML ready for tool calling
- Manage skill libraries for Agent development

Both modes share the same output format (YAML with name, description, tags, path), making it easy to use scripts as Agent tools.

### Instant Scanning

- Parallel file scanning, millisecond response
- Automatic encoding detection, skip binary files
- gitignore rules support

### Smart Matching

- Fuzzy matching on `name`, `tags`, `description`
- Search priority is `name > tags > description`
- Paths excluded from search to reduce noise

### YAML Header

Any script becomes searchable by adding a YAML header:

```python
# ---
# name: resize_image
# description: Resize image using PIL
# tags: [image, python]
# ---
from PIL import Image
```

This header serves dual purposes:
- **For script search**: Provides metadata for quick identification
- **For skill retrieval**: Defines tool interface for Agent invocation

Supported comment styles: `#`, `//`, `--`, `%`, `/**`, etc.

### Interactive Selection

skim-based TUI interface with live preview, suitable for both script selection and skill browsing.
- Picker list shows `name ✨ tags ✨ description`
- Picker filtering uses all three fields together
- Preview uses One Dark-style foreground coloring for YAML

## Installation

From release:
- https://github.com/gtiders/skillscripts/releases/latest

From source:

```bash
cargo install --path .
```

## Quick Start

```bash
# Initialize config
sks init

# Search scripts/skills
sks search image

# List all scripts/skills
sks list

# Interactive selection
sks pick

# Resolve a task_id to a path
sks task 902
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Create config file. Use `--local` for project-level config. |
| `config` | View current configuration. |
| `search <query>` | Fuzzy search, output YAML. |
| `list` | List all scripts/skills, output YAML. |
| `pick` | Interactive TUI selector with YAML preview. |
| `task <id>` | Print only the path for a specific `task_id`. |

## Output Format

`search` and `list` output YAML:

```yaml
- name: resize_image
  tags:
    - image
    - python
  description: Resize image using PIL
  path: ./scripts/resize_image.py
```

## Configuration

Config file locations:
- Global: `~/.config/skillscripts/skillscripts.yaml`
- Local: `./skillscripts.yaml` (project-level, merged with global)

### Configuration Example

```yaml
scan_paths:
  - skills
  - ./scripts
  - ~/projects/utils
ignore_patterns:
  - target
  - .git
  - node_modules
max_file_size: 1MB
search_limit: 10
report_parse_errors: true
```

`sks config` prints four sections so you can inspect the full resolution chain:

- Built-in defaults
- Global config file
- Local config file
- Effective merged config

### Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `scan_paths` | Paths to scan | `["."]` |
| `ignore_patterns` | Patterns to ignore | `[]` |
| `max_file_size` | Maximum file size | `1MB` |
| `search_limit` | Search result limit | `5` |
| `report_parse_errors` | Report parse errors | `false` |
| `copy_to_clipboard_on_pick` | Copy path to clipboard after pick | `false` |

## YAML Header Specification

### Required Fields

| Field | Description |
|-------|-------------|
| `name` | Script/skill name |
| `description` | Script/skill description |

### Optional Fields

| Field | Description |
|-------|-------------|
| `task_id` | Optional globally unique integer identifier for `sks task <id>`. |
| `tags` | Tag list |
| `args` | Parameter definitions |
| `version` | Version number |
| `command_template` | Command template |

### `task_id` Rules

- `task_id` is optional.
- If present, it must be unique across all scanned skills.
- Duplicate `task_id` values cause the scan to fail.
- `sks task <id>` prints only the matched path to `stdout`.
- If no skill matches, `sks task <id>` exits non-zero and prints the error to `stderr`.

### Examples

**Python**:
```python
# ---
# task_id: 902
# name: disk_check
# description: Check disk usage
# tags: [ops, monitoring]
# args:
#   path:
#     type: string
#     description: Target path
#     required: false
# ---
import shutil
shutil.disk_usage(path)
```

**Shell**:
```bash
#!/bin/bash
# ---
# task_id: 1201
# name: git_log
# description: Show recent commits
# tags: [git, vcs]
# ---
git log --oneline -10
```

**JavaScript**:
```javascript
// ---
// name: fetch_data
// description: Fetch remote data
// tags: [http, async]
// ---
const response = await fetch(url);
```

## License

MIT
