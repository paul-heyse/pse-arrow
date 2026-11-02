# Library discovery pilot evidence

This account records the 2026-11-02 local pilot for the
[evidence workflow](../dev/library-evidence.md). The workflow and native commands are
**Implemented**. The briefings' benefit and adoption priorities remain **Proposed**;
the pilot does not measure improved agent decisions or qualify production integrations.

## Resolved references

The native census delivered 68,645 resolved written-token observations from 1,766
workspace-consumer files. It used a frozen `git archive` of
`ad69066e76eabbe610c809d87690bc840e35a9c7`, including that revision's original Cargo.lock,
at `/tmp/pse-library-source-pqhhz1em`. The selected world was the default feature selection,
Linux x86-64 target, non-test context, with build scripts/proc-macro execution disabled.
It is not a compiler-unit-equivalent or all-configuration census.

| Family | Resolved observations | Example that corrects a simplistic adoption hypothesis |
| --- | ---: | --- |
| DataFusion/Arrow | 59,490 | Result selection already uses pruning; admission already calls memory-reservation growth. |
| Tokio | 7,526 | Canonical result-release tracking already uses `TaskTracker` tokens. |
| Symbolica | 1,629 | Math preparation already uses `evaluator_multiple` and exported instructions. |

The full original response is retained at
`~/.local/share/pse-arrow/library-evidence/2026-11-02/reference-census/references.json`, native invocation
`01a19e61-f947-76f4-aae6-cf817e5d48f2`, SHA-256
`7f57355df2f18b2d84c01a0ee7c9547940d86296bf9608626222893b96ac0b14`.
Its scoped observation and unprojected view are complete, with zero unavailable requested
packages and zero unmapped resolved definitions. Counts include imports, types and generated
workspace code. They do not establish runtime frequency, value, or absence of unobserved use.
Unsupported host/proc-macro profiles and the other original limitations remain in the response.
The separately prepared analyzer session was stopped and its original performing owner drained.
The same durable directory retains the original reference metadata, selection and snapshot
selection receipt; the temporary project can be reconstructed from the recorded source revision.

## API facts

Explicit acquisition and export use the live checkout's locked package graph, separately from
the frozen reference project. Each `api-N/manifest.json` identifies the exact package, selected
documentation features/target, original artifact and table digests. Tables retain declarations,
public paths, traits, implementations, type occurrences and external items. A facade's external
item does not silently become a complete declaration from its defining crate.

The final bundle is retained at
`~/.local/share/pse-arrow/library-evidence/2026-11-02/acquired/`.
Its manifest SHA-256 is
`2bd2456453c21137be4157f9e9056aa8ac4f90a70606a301af1493901676fb15`.
**Tested:** all 294 produced table files had matching digests and row counts, totaling
699,669 JSON object records across 49 package exports. Records are facts, not unique capabilities.

| Family | Selected packages | Exported packages |
| --- | ---: | ---: |
| DataFusion/Arrow | 47 | 46 |
| Symbolica | 1 | 1 |
| Tokio | 2 | 2 |

The `just library-evidence` run used `--acquire`, the original frozen `--reference-project`
and explicit `--reference-capture` reuse. It returned exit 1 with two failed steps against
a zero target, retaining `capture_complete: false`. The reference response's digest remained
identical to the original census.

`datafusion-macros@55.1.0` remains a coverage gap: the API owner refuses its
host/proc-macro documentation-root profile with "Cargo did not plan exactly one documentation
root for the addressed package/library/target". Both acquisition and export failures are retained;
subsequent independent packages continue. This establishes unavailable qualified documentation
in this context, not absence of the library's macro capabilities.

The acquired Symbolica tables supplied a fresh-reader navigation exercise: the public path for
`Atom::evaluator_multiple` led to its declaration, `AtomCore` import requirement and
`EvaluatorBuilder` return type. `export_instructions` led through its output type to the
documented `constant_functions` and recursive `sub_evaluators` fields. That revealed the
target-precision placeholder obligation now retained in the evaluator contract. It establishes
usable navigation in this artifact, not execution correctness or measured retrieval improvement.

## Interpretation and independent reading

The [DataFusion/Arrow](libraries/datafusion/briefing.md), [Tokio](libraries/tokio/briefing.md)
and [Symbolica](libraries/symbolica/briefing.md) briefings combine representative source reads,
broad library context and the separately dated census. Their technical accounts preserve
residual obligations: pruning still needs residual filtering; cancellation, admission closure
and task drain are distinct; optimized evaluator instructions do not replace the authored
mathematical contract. A fresh independent reader traced those distinctions and existing-use
counterexamples. This is qualitative process evidence, not a measured comparison.

## Reproduction and evidence limits

Recreate the frozen project from the named revision, retain its original lockfile, explicitly
prepare the native analysis tool and session for that project, then run the configured families
with `--reference-project`. Source paths and context identities change with the new project
location; identical response bytes are not promised. To deepen the original reference evidence
with API acquisition later, the operating guide describes explicit `--reference-capture` reuse.
That retains historical evidence without asserting a refreshed source inspection.

Earlier attempts remain in the caller-owned `/tmp/pse-library-pilots-20261102*` bundles.
They include source-staleness refusals; freezing the reference project preserved that guard.
Implementation fixes addressed an upstream database attachment conflict, excessive full-index
materialization, the default RPC receive ceiling, and an overly broad partial-completion status.
An early batch's successful passive API lookups were **unprepared**
and produced no tables. Those lookups are not successful API exports; the exporter now refuses
that state and the bundle checks the requested manifest and table files before declaring success.
No failed or interrupted run is promoted to qualified evidence.

The first explicit acquisition batch produced some real tables but also exposed compiler
selection through an unqualified `rustc` invoked from dependency directories. It was interrupted;
its captures and produced tables are retained under
`~/.local/share/pse-arrow/library-evidence/2026-11-02/interrupted-acquisition/`.
API acquisition now uses the existing checkout compiler-selection owner for both `RUSTC`
and `RUSTDOC`, including subsequent context checks, without selecting a different toolchain.

## Verification

**Tested**, zero failures against a zero target: 12 selected native/API tests and five bundle
tests, using the pinned checkout toolchain. The native selection includes reference resolution,
export tables, CLI selectors, compiler selection outside the checkout, transport filtering and
real documentation-closure acquisition. Exact commands:

```bash
scripts/pse test pse-host --lib --features native-hir \
  -E 'test(dependency_usage) | test(api::export) | test(bulk_export) | test(api_extraction_keeps_checkout_compilers) | test(extraction_removes_reexec) | test(acquired_cargo_documentation_closure)'
scripts/pse test xtask --bin xtask -E 'test(library_evidence)'
```

`scripts/pse check pse-host --all-targets --features native-hir` passed. Scoped clippy checks
for `pse-host --all-targets --features native-hir` and `xtask --all-targets --features
force-validate`, through `scripts/pse-env -- scripts/cargo-bin/cargo-pse-correctness clippy`,
passed with `-- -D warnings`. `just ready` passed skill synchronization and environment checks.
`just docs` passed, publishing 1,025 chapters with search in the local build output.
Overview links follow the working tree's uncommitted move into `analytical_infrastructure/`;
those existing documentation moves remain outside this change's commit ownership.

The earlier `just hygiene` run reported seven failed recipes against a zero target: ADR
front-matter/index checks, due-register dates, unrelated document typos, generated CLI drift,
and both clippy modes' findings in concurrent production workflow code. The task-owned CLI
drift was regenerated, and `just codegen-check` passed afterward. Other findings were left to
their owners; this is not whole-workspace or product qualification. Formatting was scoped to
owned source files to preserve concurrent production edits; the workspace-wide mutating
`just turn-end` formatter was not run over those edits.
