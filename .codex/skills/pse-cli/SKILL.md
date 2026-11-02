---
name: pse-cli
description: Consider pse-arrow's advanced Rust code intelligence for resolved relationships, types, guarded edit previews, pinned APIs and compiler evidence; select native correctness, managed execution and retained-result capabilities when useful. Routine file/text searches and source reads use standard tools.
---

# Choose tools by the question

Use standard search and reading tools for routine file discovery, text matching and source
inspection. `rg`, `rg --files` and ordinary source readers are usually sufficient.

Consider native code intelligence when resolved identity, inferred types, relationships,
structured API information, guarded edit previews or original compiler evidence could answer
the question more directly or reliably. These capabilities are options for engineering judgment,
not a required sequence or a universal preference. Weigh the expected information gain against
preparation cost and tools already available; a prepared session or acquired artifact can make
a focused query inexpensive. Stop when the evidence answers the question.

Native correctness selection, source-built fixtures, managed execution and retained outcomes
have separate operational value. Retain the repository's environment and validation obligations
when choosing between native commands, raw Cargo/nextest and specialized recipes.

The native environment puts the checkout's `scripts/` on `PATH`, so `pse` resolves to
`scripts/pse`. Without activation, use that explicit launcher. Prepare a missing host with
`scripts/bootstrap-host`; help and reads do not prepare it. Use `pse COMMAND --help` for concise
syntax, `pse help COMMAND` for structured parser metadata, or `pse commands FAMILY` for discovery.
Do not guess options or response fields.

## Advanced code intelligence to consider

For optional review/planning examples and their evidence limits, see
[engineering inquiries](references/engineering-inquiries.md). The calling process retains
its own judgment and publication boundary.

| Question where deeper evidence could help | Capability to consider |
|---|---|
| Which references, callers or implementations belong to this actual entity, including aliases? | `inspect entity --view refs`, `calls` or `implementations` in an explicitly prepared semantic session |
| What type, receiver adjustment, trait predicate or API candidate applies here? | Entity views `explain`, `hints`, `predicate` and `candidates`; `compatibility` compares types and `visibility` examines access in a selected context |
| Where did macro-generated behavior come from? | Entity views `macro`, `derive-origin` and `format-origin`, subject to backend support |
| Would a refactor preserve the intended identity and syntax? | `propose rename`, `file-rename`, `assist`, `ssr`, `syntax` or `grammar` constructs guarded previews; none applies edits or proves compilation |
| What does the exact locked dependency expose, and how do its APIs connect or differ? | `search api` / `inspect api` query acquired signatures, bounds, implementations, reexports and artifact comparisons |
| Why is a dependency or feature included, and in which target/host/build units? | `inspect dependency` / `inspect dependency-units` distinguish authored links, metadata and compiler-unit context |
| What explains a compiler failure, capture, suspension, cleanup or concrete layout? | Read `inspect diagnostic` / `inspect compiler-facts` / `inspect compiler-artifact`; explicitly acquire new evidence with `compiler generate` or an authored probe when warranted |

Read only the relevant section of [code-intelligence.md](references/code-intelligence.md)
for preparation, examples and evidence limits. Native search/read commands remain available
when structural captures, interpreted source facts or guarded references add value.

## Native execution and results

| Need | Capability |
|---|---|
| Choose revealing correctness tests | `inspect changes` / `inspect affected` and `inspect test-selection`; `inspect tests` reads retained populations; `tests` builds/lists and `test` executes |
| Supervise work or recover its original outcome | `run`, `history`, `wait`, `inspect invocation`, bounded `inspect log`; `cancel` settles owned work |
| Inspect an active calculation through its product owner | Explicit `observe prepare`, then `discover`, `schema`, `query` or `follow` |

The [environment guide](../../../docs/dev/agent-environment.md) owns preparation, placement,
run controls and detailed command examples; the
[qualification guide](../../../docs/dev/validation-assessment.md) owns what checks establish.

## Keep context proportional

Keep queries focused on the useful entity, range or artifact. Inspect returned identity,
coverage and limitations before expanding; bounded results do not establish exhaustive absence.
Search options, output formats and request configuration are in the code-intelligence reference.

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
