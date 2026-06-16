# AGENTS.md Progressive Disclosure Guide

Use `AGENTS.md` as a small, stable entry point for coding agents. Put details in focused docs that agents can open only when the task needs them.

## Keep AGENTS.md Small

Include only information that is useful for almost every task in this repository:

- A short project description.
- Non-default build/runtime requirements.
- Essential build, test, and smoke-check commands.
- Cross-cutting rules that prevent project-specific mistakes.
- Links to focused docs for diagrams, operations, conventions, or recent memory.

Avoid long file inventories, generated script catalogs, copied article text, and broad style advice that belongs in topic docs.

## Prefer Stable Guidance

Documentation loaded into every agent session should not depend on volatile implementation details. Describe capabilities and architectural boundaries before exact paths. When exact paths matter, keep them in the narrowest relevant topic doc and phrase them as current entry points rather than permanent structure.

Stable examples for this repository:

- "`init-wrapper` is an early-boot PID 1 wrapper; do not run mount/pivot smoke checks on a production rootfs."
- "The root replacement syscall flow lives in `src/main.rs`; low-level syscall wrappers live in `src/unix.rs`."
- "Diagram source and rendering rules live in `docs/diagrams/README.md`."

Stale-prone examples:

- Exhaustive source trees that duplicate `find src` output.
- Cross-project examples from unrelated repositories or benchmark projects.
- Exact dependency patch versions outside lockfiles or package manifests.
- Copied command lists that drift from `Cargo.toml`, Dockerfiles, or project scripts.

## Organize By Scope

Use progressive disclosure by scope:

| Location | Use For |
|----------|---------|
| Root `AGENTS.md` | Repository-wide essentials and links |
| Nested `AGENTS.md` | Area-specific essentials if a subdirectory grows its own workflow |
| Topic docs | Architecture, boot smoke checks, diagrams, conventions |
| Memory docs | Recent decisions and session context, not stable rules |

Nested instruction files are tool-dependent. Keep equivalent files synchronized when a tool uses a different instruction filename, such as `CLAUDE.md` pointing to `AGENTS.md`.

## Maintenance Checklist

When editing agent-facing docs:

- Remove contradictions instead of adding exceptions.
- Move detailed rules to the most specific topic doc.
- Replace exhaustive structure lists with stable categories when possible.
- Keep recent history in memory files and durable policy in topic docs.
- Delete copied article content, CTAs, and references that do not guide work in this repository.
- Verify that every linked local path exists.
