---
name: sks-script-create
description: Create, register, update, and validate reusable local scripts managed by sks. Use when writing automation for the user, adding a script to the sks YAML registry, choosing useful script tags and descriptions, or troubleshooting an sks registration.
---

# Create an sks script

1. Run `sks search "<capability query>"` once before creating a script. Reuse an existing match when it covers the task.
2. Choose the YAML registry and a nearby scripts directory.
3. Write one focused script with explicit arguments, useful errors, and no embedded secrets.
4. Register a unique `name`, a relative Unix-style `path` using `/`, a `command` containing `{{path}}`, a concise `comment`, and 2–5 useful `tags`.
5. Validate the complete registry with `sks list`.
6. Run `sks run <name> [args...]` with representative inputs to test the registration and script together.

Names must match `[A-Za-z_][A-Za-z0-9_]*`; they are case-sensitive. Do not use absolute paths or backslashes in registry YAML. Inspect the existing registry before choosing a name. If arguments are unclear, inspect the source instead of inventing them.
