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
| Find files, text, Rust syntax or declarations across source/configuration | `search` with an explicit mode and useful path/package scope; default first 20 arrivals |
| Read known source, index/revision bytes, a declaration or addressed evidence | `inspect --source FILE`, `inspect outline`, `inspect declaration`, or a returned result reference |
| Resolve an entity, callers, implementations, annotations or candidate APIs in Rust context | Explicit `code session prepare`, then `search symbol --semantic` or `inspect entity --view ...` |
| Construct a checked edit preview | `propose --result REF`, `propose syntax` for native capture templates, or a native semantic transformation under `propose` |
| Learn locked APIs, dependency inclusion or selected compiler units | `search api`, `inspect dependency` / `inspect dependency-units`; acquisition stays explicit |
| Inspect an original compiler failure or investigate native compiler behavior | `inspect diagnostic` / `inspect compiler-artifact`; explicit `compiler generate` / `compiler probe` for new evidence |
| Choose revealing correctness tests | `inspect changes` / `inspect affected` and `inspect test-selection`; `inspect tests` reads retained populations; `tests` builds/lists and `test` executes |
| Supervise work or recover its original outcome | `run`, `history`, `wait`, `inspect invocation`, bounded `inspect log`; `cancel` settles owned work |
| Inspect an active calculation through its product owner | Explicit `observe prepare`, then `discover`, `schema`, `query` or `follow` |

For code, API and compiler investigations, read only the relevant section of
[code-intelligence.md](references/code-intelligence.md). It explains when deeper capabilities
are useful and the evidence boundaries of each answer. The
[environment guide](../../../docs/dev/agent-environment.md) owns preparation, placement,
run controls and detailed command examples; the
[qualification guide](../../../docs/dev/validation-assessment.md) owns what checks establish.

## Keep context proportional

Start with the smallest useful scope and a modest `--limit` for search or `--budget` for detailed inspection. Inspect returned identity,
coverage, limitations and next actions before expanding. `--fields` selects payload detail;
it does not remove provenance or turn an incomplete answer into a complete one. Use
`--format json` for structured work, `human` for reading, or `ndjson` for normalized records.
Search defaults to fast arrival order; exact counts and exhaustive examination are explicit
demands. Use stable ordering for guarded continuation pages. First results and short output
do not establish absence or exact omitted totals. Search flags, typed JSON requests and saved
queries share one validator; use `search --explain` to inspect the effective selection and
configuration provenance without starting providers. NDJSON emits complete live records and
a final coverage summary; missing finality means incomplete delivery.

Use the selected command's `pse schema` view when its returned shape matters. Follow returned
continuations and guarded actions unchanged rather than reconstructing their identities.

Preparation, compilation and execution are explicit effects. Source reads need no analyzer;
API reads need their acquired artifact; semantic queries need their selected prepared session.
An unavailable provider remains unavailable. Related tests and possible callers are candidates;
edit commands produce previews. Apply an inspected proposal through normal authorized edits,
then run the revealing checks; there is no CLI edit-apply operation.

Use `inspect test-selection --from-result REF` to validate a working-source witness and
select its current containing package through the native test-selection owner. Target,
feature, platform and filter options retain their normal meanings. This is a current test
candidate, not a dependency-impact or coverage proof; `inspect affected` supplies the
separate conservative changed-source view. Fresh populations and execution remain `tests`
and `test`.
