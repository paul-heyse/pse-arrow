---
title: "Plan 51d: Development and research evidence"
status: complete
date: 2026-10-22
adrs: [ADR-0219]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, diagnostics, validation, performance]
---

# Plan 51d: Development and research evidence

## Useful behavior and authority

**Implemented:** ask precise questions over selected authentic test, diagnostic and performance
histories: which declared tests failed, which crates/contexts they belong to, what is unreported,
which diagnostic artifacts actually relate to a failure and which measurements form a comparable
population. Supply sources explicitly for offline analysis; import compact projections into the
separate tooling database for repeated cross-run investigation. O06–O08 state remains at the
[coordinator](51-engineering-workspaces-and-evidence.md#opportunity-dispositions).

The persistent index is selected, but offline query comes first and remains independently useful.
Neither is a new terminal composer, host invocation/resource registry, scheduler, generic query
catalogue or MCP platform. Completed Plan50 stays complete; this series adds derived capability.
Canonical science, product assertions, tool execution and host observations retain different
owners and outcome meanings. A missing source or unknown association remains queryable as unknown.

## Source adapters and query model

**Interface-checked:** start with `scripts/resource_store.py`, `test_resources.py`, `test_run.py`,
`run_inspection.py`, `validation.py`, `validation_receipts.py`, `validation_scope.py`,
`case_measure.py` and `active_diagnostics.py`, plus their declared source codecs and CLI consumers.
FileStore terminal/inventory and host resource records remain independently readable when the
canonical service, native product or compiler is unavailable. Do not import their authority into
`pse-operations` merely because a queryable derived schema is useful.

Each adapter uses the original owner reader/validation and retains source kind, authority,
codec version, original identity, snapshot/cut/digest, capture context, completeness and actual
links. Original snapshots establish facts at that cut, not a new live lease. Invalid artifacts
produce typed invalid/unavailable rows or an explicit refused source; they never become inferred
success from a log string. Declare any stored projection once through the existing registry;
preserve host-only declarations/readers at their original owner.

Test identity uses declared Rust binary/test IDs or Python node IDs with source/build/features,
test inventory and selected scope. Preserve selected, reported, unreported, skipped, reused,
transferred and command-failure states. A command failure does not imply every selected test
failed, and a missing record does not pass. Crate/module mapping comes from actual inventories
and declaration context, not path regex guesses. A filter result carries its denominator and
coverage rather than presenting a percentage over an unexplained subset.

The offline operation receives explicit source paths/identities or a validated copied closure,
uses typed adapters to form selected Arrow relations and applies DataFusion filters/joins/grouping.
Use the existing native tooling/xtask organization for a bounded built query consumer, with thin
Python/CLI adapters where needed. Preserve existing product-independent inspection if the native
query capability is unavailable. A query must not crawl host storage, start a server, build a
product extension or execute tests just to inspect supplied records. Prefer finite operation-
shaped questions over arbitrary SQL exposed as a new public security or compatibility contract.

For persistent discovery, import compact identities, outcome/coverage/context and declared links,
not blanket raw logs, profiler files or all scientific payloads. Idempotence identifies the
source authority, codec, original identity and snapshot digest. Changed snapshots remain distinct
versions; do not overwrite prior outcomes as if they were one event. Stage/reconcile a generation,
then activate it atomically. Queries select a generation/source cut and state stale, partial,
missing or unavailable coverage. Index loss is rebuildable; source mutation/retention is not.

## Diagnostics and measurements

Only declared links connect a diagnostic action, invocation, process, provider, run or attempt.
A standalone diagnostic remains valid without an artificial parent invocation. Show diagnostic
tool outcome separately from target execution and scientific outcome. Sampled CPU/RSS/queue
observations can explain what was seen, not establish causality or a numerical defect. Never
join by nearest timestamp when an exact association was not recorded.

Raw detail is lazy: original owner protection for live artifacts, explicit selected copies for
offline packs. An index row is not permission to reopen an old absolute path or pin all payloads
forever. Frozen summaries remain usable when detail is released; missing detail is explicit.
51e owns release explanation and selected closure copying, not another evidence retention system.

Performance populations preserve actual workload/input, revision/build/profile/feature/native
provider/ISA/capture settings, repeat identity and sample membership where supplied. Separate
engine/compile/startup/execution/capture costs and censored/failed samples as the source allows.
`case_measure` currently cannot provide Criterion sample membership for an invocation; retain
that unavailability instead of fabricating a join. A slow timeout is not a fast missing sample.
Comparable-cohort summaries are useful without automatically asserting regression, flakiness or
causality. New speed/capacity claims require a named measurement, baseline and conditions.

## Work packages

<a id="EV01"></a>
### EV01 — Authentic source facts and coverage

Implement typed selected-source adapters and a shared projection contract for actual test,
terminal/inventory, assessment/context, diagnostic and measurement sources. Retain actual source
versions and independent owner read paths. Define source-snapshot coverage and honest outcome/
denominator semantics before the persistent schema. ST01 contributes only stored projection
declarations; offline host readers need no canonical-store dependency.

Acceptance: authentic complete-failing, interrupted/unreported, skipped/reused and command-failure
fixtures produce distinct facts with declared inventory mapping. Corrupt/foreign/unknown codec
sources cannot masquerade as valid empty success. Expected populations come from explicit source
facts, not the aggregation implementation. Delete parallel heuristic log parsers if replaced.

<a id="EV02"></a>
### EV02 — Supplied-only offline query

Consume EV01 to expose a bounded evidence query CLI and reusable typed query surface: filter test
outcomes by crate/context, group declared failure populations, inspect missing coverage and follow
actual diagnostic references. Build selected Arrow batches and use DataFusion for relational work;
avoid one subprocess per row and repeated full parsing for each aggregate in one operation.
Keep source selection, query work and raw-detail opening separate.

Acceptance: query only the supplied authentic closed sources with the server unavailable; get
the correct failed/unreported populations and denominator, then select one protected/copied detail.
No unrelated directories are traversed and no product test/build is initiated. Unknown optional
native query capability is diagnosed through the existing independent inspection path. This
package is usable before ST03 or any persistent evidence database exists.

<a id="EV03"></a>
### EV03 — Incremental persistent discovery

Consume ST03's separate database/generation contract and EV01/EV02 semantics. Implement explicit
import/query/status/rebuild/drop commands, versioned source membership and native indexes over
the repeated filters. Initial import is explicitly requested; no background universal scanner
or source-owner mutation is implied. Reuse source facts between import and offline analysis,
without creating a second schema or allowing the index to certify terminal completion.

Acceptance: import one complete-failing and one interrupted source twice; counts do not inflate.
An interrupted import never exposes an active incomplete generation. Changed/missing sources and
stale cuts are visible; offline and indexed queries agree for the same selected source cut.
Dropping the index preserves all authoritative resources and service-unavailable inspection.

<a id="EV04"></a>
### EV04 — Linked and standalone diagnostic investigation

Integrate existing diagnostic actions/artifact readers with EV01–EV03, accepting only actual
owner-supplied links. Expose tool/target/scientific outcomes separately and selected protected raw
detail. Connect LC04's storage-health observations through real authority/process references,
with unknown correspondence explicit. Use the current diagnostic capture mechanisms; no new
debugger or mandatory always-on profiler is selected.

Acceptance: investigate one exactly linked failing test/run and one standalone action; unrelated
nearby timestamps remain unrelated. Release an artifact and confirm summary availability with
truthful missing-detail status, not stale path access. Show a diagnostic tool failure without
converting it into a target/scientific failure. Delete obsolete ad hoc detail traversal only
where the new consumer has actually replaced it.

<a id="EV05"></a>
### EV05 — Comparable performance histories

Add explicit cohort selection and summaries over authentic measurement populations using the
existing workload/context records. Surface comparability exclusions and unknown inputs, actual
sample membership, censored outcomes and capture overhead where known. Use selected columnar
aggregation for distributions; compact native metadata indexes locate cohorts without raw-log
ingestion. Preserve unsupported joins, especially Criterion-to-invocation membership, as absent.

Acceptance: deliberately varied build/provider/workload/capture fixtures are not merged silently;
duplicate imports do not duplicate samples; failed/censored runs remain visible. A comparison
with unknown sample association reports its actual scope. If a performance improvement is claimed,
run the named representative before/after measurement under its declared conditions; otherwise
complete the capability without a quantitative benefit claim.

## Verification and evolution

**Implemented:** EV01–EV05 are complete at targeted acceptance on 2026-10-23. The
[coordinator](51-engineering-workspaces-and-evidence.md#package-state-and-execution-route)
owns package/opportunity state. Source owners remain independent; a new runner or diagnostic
producer supplies authentic owner facts and explicit associations rather than a heuristic log
parser. LC04 remains responsible for storage-health collection. This work accepts its declared
references when supplied, without inventing process/scientific correspondence.

**Tested:** Linux targeted functional acceptance below used pinned tools and a zero-failure
baseline. Arrow-consuming native builds explicitly verified force-validation. The shared native
command is recorded once in [51c verification](51c-scientific-data-studies-and-results.md#verification-and-handoff).
It includes all evidence query, canonical import/cut and existing evidence-capture controls in
the four selected packages, alongside the scientific/workbench controls; its result is not a
whole-product or full-series qualification.

| Command / selected scope | Final result |
| --- | --- |
| Shared Rust acceptance linked above | 70 passed, zero failed, 1,839 excluded by the explicit selection; actual native fit/publication/downstream, source/replay, cohort/observer, evidence and CLI controls across 10 binaries (Nextest `83280600-0939-478c-8157-c53fa09c321b`) |
| `scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.test_evidence_sources scripts.tests.test_evidence_projection scripts.tests.test_evidence_store scripts.tests.test_active_diagnostics` | 48 passed, zero failed; authentic owner adapters, declared inventories, interrupted/unreported/skipped/reused/transferred populations, foreign codec/copy refusal, independent lifecycle/detail and CLI dispatch |
| `scripts/pse-env -- just evidence-build` | Passed explicit optional native executable build; no query performs this build implicitly |
| `PSE_EVIDENCE_TEST_TOOL="$PWD/target/debug/pse-evidence" scripts/pse-env -- .venv/bin/python -m unittest scripts.tests.evidence_native` | 3 passed, zero failed; actual executable over supplied authentic invocation, diagnostic and measurement artifacts with service/license/activity context unavailable, no unrelated traversal or implicit product build, exact links, released detail and actual samples |
| `scripts/pse-env -- just codegen` | Passed original registry/generator publication, including native evidence schema and selected query contracts |
| `scripts/pse-env -- just family-check` | Passed single pinned Arrow/Parquet/object_store/DataFusion universe after wiring the optional native query edge |

The native query controls independently specify failed/unreported denominators and context
populations, preserve separate diagnostic tool/target/scientific outcomes, require exact declared
links and exclude unrelated timestamp neighbors. Canonical controls cover scoped roles,
staged/activated generation visibility, duplicate import, changed snapshots, explicit generation
cuts/history, supplied freshness observations, offline/indexed parity and derived-index loss
without source mutation. Measurement fixtures vary actual build/provider/workload/capture context,
retain failed/censored samples and report unavailable membership instead of inventing it.

**Not run:** whole-series integration, full hygiene and whole-series manual governance/native/
powerset qualification, additional operating systems or fresh performance campaigns. Mechanical
ADR/formatting and 616-chapter documentation publication passed; [51c mechanical publication](51c-scientific-data-studies-and-results.md#verification-and-handoff)
records the format-only Ruff diagnostics that remain for Q03, without a clean lint claim. Q01–Q04 in 51g own
those boundaries. **Measured:** no quantitative speed, regression or capacity claim is made;
authentic sample ingestion is a functional capability, not a benchmark of this implementation.

## Outcome

### What was built

**Implemented:** typed authentic invocation, assessment, diagnostic, measurement and host adapters
extend the existing evidence family with original source identity/codec/digest, declared inventory
and compact contexts/links. Explicit outcome and coverage populations remain distinct. Source
validation uses the original owner; unsupported or corrupt input cannot become empty success.
Actual Rust binary/test and Python node identities use declared mapping without guessed crates.

**Implemented:** a supplied-only built native DataFusion query tool answers finite evidence
questions with selected Arrow relations, context filters, distributions, coverage and exact
association joins. Thin Python/CLI dispatch runs before service/activity configuration and retains
independent snapshot/detail inspection when the optional native executable is absent. Queries
neither scan arbitrary storage nor initialize a product, start a service or compile implicitly.

**Implemented:** the existing isolated staged/activated tooling index reuses the same facts for
explicit import/query/status/rebuild/drop and selected generation cuts. Duplicate snapshots
deduplicate, changed versions require explicit history, and freshness is matching/stale/missing
only against supplied observations; otherwise results describe their imported cut. The index
confers no terminal, raw-artifact, resource-release or scientific authority.

**Implemented:** linked/standalone diagnostic questions preserve separate outcomes and original
artifact references. Detail selects original protected artifacts or validated supplied copies,
with optional digest binding; released detail does not erase the summary or reopen an old path.
Performance questions expose actual context, sample membership, distributions, reported summaries,
failed/censored samples and known capture overhead independently. Genuine Criterion samples are
accepted only through validated source data; unavailable invocation membership stays unavailable.

### A mistake made and corrected

**Tested:** initial native queries lacked the engine catalog/schema because the shared engine
factory deliberately disables a default namespace. The selected query consumer now registers its
own namespace and tables instead of changing global engine policy. Independent review exposed
context filtering dropping target/scientific/artifact facts and selected diagnostic queries
including unrelated links. All action facts now retain their original context; joins retain only
the exact declared referring invocation. The two-context fixture supplies all fourteen declared
context fields rather than treating omitted fields as comparable.

The initial combined 43-pass/7-fail Rust run included these query initialization failures and
separate scientific fixture failures. The former were corrected at the query owner; later final
acceptance supersedes those failures rather than recording a nonzero baseline. No nearest-time
join, default success or lint suppression was introduced to make acceptance pass.

### Deviations from the plan, deliberate

**Implemented:** finite operation-shaped questions use the existing EngineFactory/DataFusion
rather than a generic SQL platform. Original process-cost/measurement reports and unmeasured smoke
records are supported at their actual scope; nested preparation/admission reports are not silently
flattened into complete performance populations. Optional explicit invocation sample files must
validate actual owner membership. Unknown context or Criterion association remains explicit.
LC04 storage-health collection and LC01/LC03 portable closure/release are later-owned consumers,
not duplicated here. No fresh benchmark was required because no performance benefit is claimed.

Enduring authority and query semantics live at blueprint §24.1 and proposed ADR-0219; operator
commands and current limits are in the [substrate guide](../dev/surreal-substrate.md#research-versions-study-workbench-and-selected-evidence).
The evidence implementation worktree was merged with qualified main repairs preserved, then the
clean merged worktree and branch were removed. Retain this completed companion while later
51e–51g work consumes it; Q04 owns aggregate retirement and existing-plan reconciliation. Full
Plan51 qualification and ADR acceptance remain separate.
