---
name: sks-script-create
description: Create, register, update, and validate reusable local scripts managed by sks. Use for changes to registered scripts, their command-line interfaces, or their YAML registrations.
---

# Create or update an sks script

## Confirm the interface

- Before creating a script, run `sks search "<capability query>"` once and reuse a suitable match. For an existing script, inspect its registered source and registry entry.
- Before introducing or changing arguments or behavior, present a concrete proposal: arguments, defaults, input formats, outputs and filenames, overwrite policy, and side effects. Obtain the user's agreement unless their request or an earlier approved proposal already specifies these choices.
- An approved common interface can cover a batch of scripts. Proceed within that approval without asking again for each script or routine implementation detail.
- Internal rewrites that preserve the approved interface and behavior can proceed under the user's existing authorization. Keep CLI help consistent with the implementation; do not require separate specification documents.

## Keep scripts focused

- Prefer positional arguments for core input files and `-o/--output` for an optional output path. Require explicit input paths unless the user has approved default filenames or discovery behavior.
- Minimize parameters. Avoid rarely useful presentation options, duplicate controls, and compatibility aliases unless requested. State frame numbering, inclusive/exclusive endpoints, and selection behavior explicitly; preserve approved semantics.
- Preserve scientific calculations during standardization: units, tensor ordering, filtering, statistical definitions, and data-to-mode correspondence. Report a suspected calculation error with evidence and a proposed correction before changing scientific behavior.
- When a configuration template command is useful and approved, print only valid configuration to stdout. Send diagnostics to stderr, and verify any advertised redirection example against the actual runner's output.

## Scientific plotting defaults

For this user's scientific plotting scripts, use these defaults unless the user specifies otherwise:

- Produce one PNG, overwrite an existing file at the chosen output path, and create its parent directory when needed. Propose additional CSV, PDF, or other outputs explicitly.
- Use the current `plot_vasp_aimd` style as the reference: serif fonts, inward ticks, white background with light grid, colorblind-friendly colors, and fixed 600 dpi. Inspect the reference implementation rather than guessing its style. Adapt figure dimensions to keep labels readable.
- Do not expose DPI and other appearance controls by default. Retain controls that affect approved data selection or analysis.
- Shared style or parsing helpers are allowed. Deploy required helpers alongside the script and verify imports through the registered command; include them in subsequent synchronization.

## Register concisely

- Choose the appropriate category YAML and nearby script directory. Preserve an existing script's name, path, and command unless a change is approved.
- Names must match `[A-Za-z_][A-Za-z0-9_]*` and are case-sensitive. Use a relative Unix-style registry path and a command containing `{{path}}`.
- Write a short comment describing the main purpose. Usually use one or two necessary tags, including the category; avoid listing every input format, dependency, property, or implementation detail as a tag.

## Validate and hand off

- Run `sks list` after registration changes. Check argument parsing and the behavior affected by the edit with proportionate, meaningful verification.
- When the user has authorized execution and the input/output locations are clear, run `sks run <name> [args...]` with actual or representative input and inspect the resulting output. Do not run a script merely to obtain its `.sks` copy if execution could overwrite user results or cause other unapproved effects.
- For plots, inspect the generated image for clipped labels, overlapping elements, and readable legends. Report which inputs and selections were used and what verification actually passed.
- When chezmoi synchronization is requested, add the exact changed target files and required new helpers. Check the relevant status/diff afterward. Exclude unrelated changes and generated caches or run outputs. Report any automatic commit created by the user's chezmoi configuration.
