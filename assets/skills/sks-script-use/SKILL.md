---
name: sks-script-use
description: Discover and reuse registered sks scripts before writing one-off code or shell commands. Use for concrete tasks that could be performed by a script or local automation, including calculations, conversions, file or data processing, content generation, validation, and build or development workflows, even when the user does not explicitly mention sks, local scripts, or existing tools.
---

# Use an sks script

1. For an executable task, run `sks search "<capability query>"` once before writing or running ad-hoc code or shell commands. Use a concise capability query and preserve the user's domain terms.
2. Treat every request to use a script as an explicit search trigger, regardless of task size.
3. Select matches by `comment` and `tags`. Read the file at the result's `path` when arguments or behavior are unclear; do not invent arguments.
4. Run a selected script with `sks run <name> [args...]` when execution is available and appropriate. The name is case-sensitive and must be copied exactly from the result.
5. If the registered command cannot run here or the script needs a task-specific change, use the `.sks/<filename>` copy reported by a `sks run` attempt. Edit that copy or run it directly with a suitable interpreter and arguments. `sks run` executes the registered source, and another run replaces the copy. Do not run a command only to obtain its copy if executing it could have unwanted effects.
6. If no useful match exists, continue with another approach. Do not repeat equivalent searches.

Skip discovery for purely conceptual discussion that requires no execution. Do not force a script when no registered script matches the task.
