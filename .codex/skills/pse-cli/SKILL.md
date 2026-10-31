---
name: pse-cli
description: Select and use pse-arrow's native CLI for source and Rust code intelligence, pinned API and compiler investigations, correctness tests, managed runs and result inspection. Use while investigating or implementing in this checkout; choose the capability that answers the current question.
---

# Use the native CLI

Prefer `pse` when its capability fits: it preserves the checkout's actual correctness
selection, source-built fixtures, native process ownership and original results. Raw Cargo,
nextest, search tools and specialized recipes remain available; use them when they better
answer the question, retaining the repository's environment and validation obligations.

The native environment puts the checkout's `scripts/` on `PATH`, so `pse` resolves to
`scripts/pse`. Without activation, use that explicit launcher. Prepare a missing host with
`scripts/bootstrap-host`; help and reads do not prepare it. Use `pse commands FAMILY` or
`pse help COMMAND` for current parser-owned syntax. Do not guess options or response fields.

## Choose by question

| Need | Start with |
|---|---|
| Understand written source, inspect staged/committed bytes, or preview a structural change | `outline`, `show`, syntax `code query` / `code preview` |
| Resolve an entity, callers, implementations, candidate APIs or an edit in its Rust context | Explicit `code session prepare`, then a selected semantic `code` command |
| Learn the exact locked library API, dependency inclusion or selected compiler units | `api`, `dep`, `dep units`; API acquisition is explicit |
| Inspect an original compiler failure or investigate native compiler behavior | `compiler diagnostics`; explicit `compiler generate` / `compiler probe` for new evidence |
| Choose revealing correctness tests | `changes` / `affected`, then `tests` to build/list or `test` to execute the selected population; `check` for compilation |
| Supervise work or recover its original outcome | `run`, `history`, `wait`, `result`, bounded `log`; `cancel` settles owned work |
| Inspect an active calculation through its product owner | Explicit `observe prepare`, then `discover`, `schema`, `query` or `follow` |

For code, API and compiler investigations, read only the relevant section of
[code-intelligence.md](references/code-intelligence.md). It explains when deeper capabilities
are useful and the evidence boundaries of each answer. The
[environment guide](../../../docs/dev/agent-environment.md) owns preparation, placement,
run controls and detailed command examples; the
[qualification guide](../../../docs/dev/validation-assessment.md) owns what checks establish.

## Keep context proportional

Start with the smallest useful scope and a modest `--budget`. Inspect returned identity,
coverage, limitations and next actions before expanding. `--fields` selects payload detail;
it does not remove provenance or turn an incomplete answer into a complete one. Use
`--format json` for structured work, `human` for reading, or `ndjson` for normalized records.
Use the selected command's `pse schema` view when its returned shape matters. Follow returned
continuations and guarded actions unchanged rather than reconstructing their identities.

Preparation, compilation and execution are explicit effects. Source reads need no analyzer;
API reads need their acquired artifact; semantic queries need their selected prepared session.
An unavailable provider remains unavailable. Related tests and possible callers are candidates;
edit commands produce previews. Apply an inspected proposal through normal authorized edits,
then run the revealing checks; there is no CLI edit-apply operation.
