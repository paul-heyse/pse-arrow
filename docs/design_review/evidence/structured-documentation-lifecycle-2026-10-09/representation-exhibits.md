# E. Representation exhibits: one review, one plan and one evidence bundle under four candidates

**Role and baseline.** This is supporting evidence for the formal design review of the
repository's reference-content system. It is not a verdict and picks no winner.

**Baseline.** The live dirty `main` at `4c24721e691187e1a5b28398b29722fbde671da8`, observed
2026-10-09/10. The efficiency review, its evidence folder, Plan 32, Plan 33,
`docs/lifecycle.toml` and the lifecycle scripts are untracked. Many other files have
concurrent uncommitted edits.

**Inputs.** The content model and consumer journeys come from `content-model-and-journeys.md`,
mechanisms and edit costs from `mechanisms-and-consumers.md`, and evidence lifecycle from
`evidence-reuse-and-collection.md`. Library facts come from `library-alternatives.md`.

**Runs.** Executed runs are in `probes/README.md`: P1 (markdown-it-py 4.2.0 +
mdit-py-plugins 0.6.1), P2 (mdBook 0.5.4 over the staged live corpus, the staging-vs-preprocessor
comparison and Pagefind 1.5.2) and the `git merge-file` exhibit. *Tested* and *Measured* labels
below refer to those runs and their stated conditions. The GitHub-rendering statements reuse the
earlier, disclosed synthetic-fixture call to GitHub's renderer
(`library-alternatives.md` §0); no remote renderer was called in this assignment.

**Maintainer constraint applied throughout.** Every candidate keeps finding, packet and
checkpoint prose as prose. None splits cause, consequence and correction into typed fields.
The structure carries only what the skills already name: element kind, identity, state,
relations and placement.

## Exhibit material (verbatim)

Copied by a scratch excerpt script (not retained) from the live, dirty `main` checkout at `4c24721e` (2026-10-09/10).
The review, its evidence folder and Plan 33 are untracked. Blocks are byte-exact apart from the fences.

### X1. Efficiency review (primary)

**Front matter**

````markdown
---
title: Efficiency principles across the simulator codebase
date: 2026-10-09
tier: design
purpose: target
status: review
standard: Core 3.4
profile: Process Simulator 1.5
baseline: 4c24721e691187e1a5b28398b29722fbde671da8
---
````

**§1 field table: header and the five requested rows**

````markdown
| Field | Assessment |
|---|---|
| Standard | Core/template 3.4, Efficiency Heuristics 1.0, Process Simulator principles/review additions 1.5, selected pse-arrow binding |
| Baseline | `4c24721e691187e1a5b28398b29722fbde671da8`; concurrent graph/hash implementation was committed during inquiry startup and is included |
| Architectural fitness | Fails G9; concrete violations of applicable foundations |
| Behavioral/semantic adequacy | Revise for qualification-applicability defects; whole-simulator scientific acceptance is not established |
| Disposition | Existing [Plan 28](../../plans/28-surrealdb-unified-substrate.md) remains the coordination/disposition owner if these findings are adopted; suggested work owners below are prospective |
````

**Finding F02 (full)**

````markdown
### <a id="f02"></a>F02 — Qualification applicability omits consumed inputs

`validation_scope.RUST_INPUTS`, inherited by Python product scope, omits `test_run.py`, `test_resources.py`, `host_admission.py`, `surreal_server.py` and `benches/`. Supported native/tool selectors including `UNO_DIR`, `PETSC_DIR`, `SUITESPARSE_INCLUDE_DIR`, `CC`, `CXX` and `CMAKE_TOOLCHAIN_FILE` are absent from its environment projection.

The omitted scripts govern execution, fixture identity, placement, service admission or cleanup. Native configuration consumes the omitted selectors. `_reuse_guarded` compares the projected `input_identity`; the complete outer snapshot cannot repair that comparison. `verify_native` rehashes paths from the old capture, not a newly selected configuration's closure. See [scope](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation_scope.py), [reuse](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation_receipts.py), [native configuration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_operation.py) and [native cache inputs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_cache.py).

**Tested projection behavior:** the retained pinned Python 3.14.7 probe changed synthetic file digests/environment values passed to the actual function. Both product scopes ignored the five tested omitted paths and six selectors; crate source, `native_tests.py` and `IPOPT_DIR` positive controls changed identity. This tests projection only. Actual successful-receipt reuse was not exercised.

Consequently, a prior successful observation can satisfy the unchanged-input comparison after relevant orchestration/configuration changes. This violates AP-04/AP-05, DP-09/DP-22 and G3/G6/G7. It does not prove that a scientific result was numerically wrong.

**Correction.** Restore conservative consumed-input coverage, include effective supported native/tool configuration and bump the scope interpretation version. Including execution scripts broadly is a viable first correction; a narrower complete closure is also valid. Preserve historical receipts under their original interpretation. Do not relabel them under the expanded definition.

**Verification.** Retain positive controls, detect each omitted consumed input, and demonstrate refusal of unchanged-input reuse on an actual successful report after a relevant change. Reviewed transfer remains explicitly historical. Old captured-library hashes cannot substitute for current configuration association.

````

**RC05 row, with its table header**

````markdown
| ID | Current rule or contract | Proposed impact | Dependent finding; if retained |
|---|---|---|---|
| <a id="rc05"></a>RC05 | Implemented input-scope version 2 and its projected consumed-input definition | Expand conservative coverage and advance interpretation version; preserve old receipts as historical | F02; this corrects implementation to the intended complete-input contract. Keeping incomplete scope cannot support the unchanged-input claim |
````

**"Adoption handoff — 2026-10-09", first paragraph**

````markdown
### Adoption handoff — 2026-10-09

After this review, the maintainer selected a standalone remediation workstream rather than
the proposed Plan 28 distribution. [Plan 33](../../plans/33-efficiency-principles-remediation.md)
is the sole current disposition, implementation and affected-qualification owner for F01–F11
and the six opportunities in §8. Existing Plan 28 scientific/campaign obligations remain
with their owners; they do not gate Plan 33's completion.
````

**Evidence README entry for the probe (`efficiency-principles-codebase-2026-10-09/README.md`)**

````markdown
## Executed input-projection probe

**Tested:** the [probe](input-scope-probe.py) calls the actual
`scripts.validation_scope.input_identity` with synthetic before/after file digests and
environment values. It changes no repository files and starts no compiler, solver or service.

```bash
scripts/pse-env --resource-class light -- .venv/bin/python -B docs/design_review/evidence/efficiency-principles-codebase-2026-10-09/input-scope-probe.py
```

On the reviewed baseline, Python 3.14.7, input-scope version 2, the command exited 0.
The [retained output](input-scope-probe.json) uses `true` when the changed input changes
the projected identity. Five omitted source paths and six native/tool environment selectors
were undetected in both product scopes; crate source, `native_tests.py` and `IPOPT_DIR`
positive controls were detected in both scopes. Baseline: every consumed input should change
applicability when changed; the omissions violate that expectation. This is not a demonstration
of actual successful-receipt reuse or a scientific-result error.

The first invocation exited 125 before execution because host admission had no capacity.
`just ready` then passed its skills-sync and doctor steps against the zero-failure baseline;
the ordinary retry succeeded. No admission bypass was used. An earlier reviewer diagnostic
outside the pinned wrapper is superseded by this retained pinned execution.

````

**`input-scope-probe.json`: top-level keys, scalar values shown**

````json
{
 "environment": "<dict of 7>",
 "input_scope_version": 2,
 "method": "synthetic before/after values passed to actual input_identity",
 "paths": "<dict of 7>",
 "python": "3.14.7",
 "schema": "review-input-projection-probe-v1"
}
````

### X2. Plan 33 (primary)

**Front matter**

````markdown
---
title: Efficiency principles remediation
status: in-progress
date: 2026-10-09
adrs: [ADR-0169, ADR-0170, ADR-0171, ADR-0172]
review_sources: [../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md]
scenario_sources: [../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#4-representative-journeys-and-change-locality]
---
````

**RC05 in "Confirmed rule changes and adoption routes" (section lead, table header, row)**

````markdown
## Confirmed rule changes and adoption routes

All decisions below were confirmed by the maintainer on 2026-10-09. Review RC identifiers
refer to the efficiency review, not the distinct Graph/hash RC identifiers in Plan 28.
Confirmation selects the target; required architecture/ADR adoption precedes dependent
implementation. Accepted ADRs are not rewritten in place.

| Rule impact | Confirmed direction | Route and dependency |
|---|---|---|
| [RC05](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc05), as amended | Expand conservative consumed-input coverage and advance the scope interpretation. Obtain fresh qualification; old internal receipts need no preservation machinery and cannot be relabelled as current passes. | EFF01 corrects the scope/reuse implementation. Existing requested-effects and complete-input contracts remain; use a design amendment only for an actual contract change. |
````

**EFF01 packet row, with the section lead and header**

````markdown
## Implementation packets and dependencies

The table is the sole packet-progress owner. Detail below supplies target behavior and
revealing acceptance, not a second status ledger. Packet status below records execution progress; implementation
has not started. A working prerequisite means implemented and exercised behavior, not merely
an agreed interface or another plan's historical pass.

| Packet | Required input | Delivery and targeted acceptance | Progress |
|---|---|---|---|
| <a id="eff01"></a>EFF01 — Complete applicability (F02) | Existing complete-input intent; independent of other remedies. | Expand scope/effective configuration and bump interpretation. Detect every reviewed omission with positive controls; exercise actual unchanged-input refusal using a newly successful bounded report after a relevant change. Remove obsolete scope/reuse assumptions. | Implemented; focused successful-reuse/configuration controls pass; affected integration remains. |
````

**EI01 investigation row, with its header**

````markdown
| Packet | Question, evidence and decision boundary | Integration and acceptance |
|---|---|---|
| <a id="ei01"></a>EI01 — Selected gather | Can selected forecasting/scratch follow touched source extent instead of the full source while remaining conservative for duplicate multiplicity and physical aliases? Inspect current checked-take ownership and representative nested/variable-width layouts; do not reopen the repaired historical quadratic bug as an unfixed diagnosis. | Adopt selected forecasting/scratch where sound, otherwise identify the exact alias/layout limitation. Integrate at checked take, with bounds/nulls, duplicates, empty selection, reservation refusal and escaped ownership controls. |
````

**F02 disposition row, with the section lead and header**

````markdown
## Finding dispositions

This is the sole disposition owner for the efficiency review's F01–F11. Scenario references
are the review's existing S01–S07, not a new registry. Scheduling is not resolution.

| Finding | Scenario | Disposition | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
| [F02](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f02) | S01 | Scheduled | EFF01 | Complete input/configuration sensitivity and actual refusal of newly stale successful reuse. |
````

**"Current checkpoint", first paragraph**

````markdown
## Current checkpoint

Execution is in progress. EFF01 binds the scripts/configuration/benchmark closure and
effective native configuration under input interpretation 3. EFF02 integrates explicit
execution effects, actual Cargo target/host Arrow validation, package-declared native
opt-ins and exact build-metadata propagation. Focused tooling controls pass.
````

### X3. Plan 28: colliding finding IDs (secondary)

**Plan 28 *Finding dispositions*: the completion-audit F01 row**

````markdown
## Finding dispositions
…
| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_plan-28-completion_2026-10-06.md#f01) | S01/S03/S07 | Resolved | A3; C2/C4 | [A Outcome](28a-canonical-substrate-and-revisions.md#outcome-recorded-after-implementation): native generic lifecycle-root refusal, history mutation and protected reclamation controls pass; enclosing E3 remains separate. |
````

**Plan 28 *Enhancement-review dispositions*: Enhancement F01**

````markdown
### Enhancement-review dispositions
…
| Finding or obligation | Review scenario | Disposition | Work owner | Required evidence or settling decision |
|---|---|---|---|---|
| [Enhancement F01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#f01) | S01/S06 | Implemented; focused controls pass, E3/E4 pending | C5; N2/N4; E3 | Actual stored/default study basis reuse, every binding/seed role checked, distinct occurrences, changed premises and creation cancellation. |
````

**Plan 28 *Parallel-execution review dispositions*: Parallel F01**

````markdown
### Parallel-execution review dispositions
…
| Finding | Review scenarios | Disposition | Work owner | Required evidence or question |
|---|---|---|---|---|
| [Parallel F01](../design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md#f01) | S01/S03/S06 | In progress; focused managed Rust controls pass, Python/assembled E3 acceptance pending | A4; B6; T7; E3 | Reliable sixteen complete cases and original report/result meaning; protected reclamation and uncertain-acknowledgment safety. Select the correction with adequate operation attribution; RC01 is accepted conditionally; A4 selects the actual route. |
````

**Plan 28 *Graph and hashing finding dispositions*: Graph/hash F01**

````markdown
### Graph and hashing finding dispositions
…
| Finding or premise | Review scenarios | Disposition | Work/decision owner | Required evidence or question |
|---|---|---|---|---|
| [Graph/hash F01](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f01) | S01/S02 | Resolved, 2026-10-09 | 28j/J3; [28e acceptance](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance) | Retained canonical inventories/indexed settlement replace repeated discovery. Compiler inventory controls, final native runtime many-body/exact-basis attribution/released-root controls, public Python retention and actual portable restart passed with current effects and owner charges preserved. |
````

### X4. Reopened-target chain: WCL-F01 (secondary)

**WCL review: WCL-F01 heading and first paragraph**

````markdown
### <a id="f01"></a>WCL-F01 — Lifecycle meaning is not consistently executable

**Interface-checked:** `scripts/docs.py::metadata` extracts title/status, while
`scripts/adr.py::split_front_matter` provides a limited flat parser. Document families
contain useful existing metadata, but review ownership/provenance/retirement conditions
are uneven: only 30 of 99 design-review Markdown files have any front matter, and only
four of the 50 principal reviews have a status field. Review decisions are often prose;
`disposition_owner` and `disposition-owner` also vary. Lack of a review status is not
itself a defect; absence of a consistent interpretation contract is.
````

**Target review: front matter and decision line**

````markdown
---
title: Workspace content lifecycle — scoped target assessment
date: 2026-10-09
tier: design
purpose: target
standard: Core 3.4
decision: accept
disposition_owner: docs/plans/32-workspace-content-lifecycle.md
---
…
**Overall decision: Accept.** No new material findings or additional rule impacts prevent dependent implementation. The strongest evidence is the explicit ownership/failure design grounded in the inspected support mechanisms and the bounded historical-input evidence. Runtime correctness, corpus compatibility, publication behavior and disposal safety require the execution evidence already owned by Plan 32.
````

**Plan 32: WCL-F01 disposition row, with header**

````markdown
| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| [WCL-F01](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#f01) | S02/S03/S07 | open | WC00/WC02, with WC07 adoption | Shared interpretation and real consumer migration; body/native-state/ownership preservation |
````

**Handoff: "Later discussion: why the document target needs revision", first paragraph**

````markdown
## Later discussion: why the document target needs revision

The maintainer challenged the external tags/bindings paired with largely unchanged Markdown
as a dual system. Simply moving those selectors into front matter would still leave tools
reconstructing meaning from presentation tables. The proposed correction must unify the
**authored document**, not merely move its sidecar metadata into the same physical file.
````

### X5. Evidence bundle (canonical-selection-2026-10-06)

**README.md (complete)**

````markdown
# Canonical selection query plans

**Interface-checked:** the installed `pse.substrate.v1` schema on SurrealDB 3.3.0
was inspected with `EXPLAIN FULL` for bounded scalar output, dense output, result
block range, analysis-edge and study-frontier selectors. The
[captured plans](query-plans.json) retain the complete query text and native response.

All five shapes use `IndexScan` on the intended scoped composite index:
`result_cell_rows`, `result_output_range`, `result_block_page`, `analysis_edges`
and `study_candidates`. Ordering follows the index; none requires a `Sort` or
table scan. Result-range overlap and optional scientific predicates remain filters
within the selected result/output/partition prefix. The study frontier uses
study, settled and assigned state plus its ordinal range in the index itself.

The application tables were empty. This establishes the selected physical paths
under these concrete parameters, without a cardinality, concurrent capacity or
timing claim. Functional controls exercise admitted values and protected reads;
Plan 28e owns assembled workload qualification and measurements.
````

**`query-plans.json`: top-level keys and plan names**

````json
{
 "server": "SurrealDB 3.3.0",
 "interpretation": "pse.substrate.v1",
 "scope": "Installed target schema; empty application tables; concrete selector shapes; no scale or speed conclusion",
 "plans": {
  "scalar-output": [
   "query",
   "response"
  ],
  "dense-output": [
   "query",
   "response"
  ],
  "block-range": [
   "query",
   "response"
  ],
  "analysis-edges": [
   "query",
   "response"
  ],
  "study-frontier": [
   "query",
   "response"
  ]
 }
}
````


## 1. Facts the exhibits establish before any candidate

| # | Fact (source) | Label |
|---|---|---|
| E1 | **F02's state is split across places.** The F02 disposition says `Scheduled` (X2). Its owner EFF01 says `Implemented; focused … controls pass; affected integration remains` (X2). The packet section lead still says `implementation has not started` (X2). The checkpoint says EFF01 `binds … under input interpretation 3` (X2). That is four statements of one element's state in one file, and nothing relates them. | Observed (file text) |
| E2 | **RC05's confirmation is written in five places.** Plan 33's purpose paragraph ("confirmed the review's RC01–RC05 directions"), Plan 33's rule-change section lead (X2), the review's *Adoption handoff* (second paragraph: "The maintainer confirmed RC01–RC05"), `docs/design_review/README.md` ("Its confirmed design-phase directions…") and `docs/plans/README.md` ("incorporates the confirmed rule changes"). The review's §1 *Disposition* row and its §10 owner table still name Plan 28 (X1). | Observed |
| E3 | **Plan 28 has four rows whose finding ID is `F01`** (X3), each linking a different review's `#f01`. Only a prose prefix distinguishes them ("Enhancement", "Parallel", "Graph/hash"), and the completion-audit row has none. The four tables have four different header sets. | Observed |
| E4 | **The WCL-F01 chain spans four documents** (X4). The review raised it; the target review accepted the design (`decision: accept`); Plan 32's disposition stays `open`; and the handoff then records that the accepted target "needs revision". No field records that the accepted target was reopened. The handoff's own `doc_*` front matter is the only machine-readable link. | Observed |
| E5 | **The probe's recorded conditions no longer hold.** `input-scope-probe.json` records `input_scope_version: 2` and imports production `scripts.validation_scope`. The working tree is at `INPUT_SCOPE_VERSION = 3` (C §3). The README describes the run in prose. | Observed |
| E6 | **The canonical-selection answer's conditions have moved on** (X5): `interpretation: pse.substrate.v1` and `server: SurrealDB 3.3.0` with upstream SDK. The current schema interpretation is `pse.substrate.v3` (`crates/pse-schema/src/catalog/substrate.rs:14`, `crates/pse-operations/src/generated/surreal.rs:14`). SurrealDB is `=3.3.0` but now `path = "vendor/surrealdb"`, a locally patched SDK (C §2). No producer script is retained. No Markdown links the bundle; it survives through `docs/site.toml [[asset_bundles]]` and a `docs/lifecycle.toml` README exception. | Observed |
| E7 | **Turning off mdBook definition lists changes nothing in the current corpus:** 298 pages, 0 differ, 0 `<dl>` (P2 step 1). | Tested |

## 2. The four candidates, each in its strongest honest form

### (i) Baseline as-is

Markdown with free YAML front matter. Element tables are found externally by exact heading text
and header list. Each plan needs the following stanzas, verbatim from the live files:

````toml
# docs/lifecycle.toml (Plan 33)
[[exceptions]]
path = "docs/plans/33-efficiency-principles-remediation.md"
[exceptions.values]
doc_owner = "docs/plans/33-efficiency-principles-remediation.md"

[[scope_tables]]
path = "docs/plans/33-efficiency-principles-remediation.md"
name = "implementation-packets-and-dependencies"
heading = "Implementation packets and dependencies"
headers = [
  "Packet",
  "Required input",
  "Delivery and targeted acceptance",
  "Progress",
]
kind = "packet"
id_column = 0
status_profile = "packet"
id_pattern = '^([A-Z]+[-]?[0-9]+)\b'
status_column = 3

[[scope_tables]]
path = "docs/plans/33-efficiency-principles-remediation.md"
name = "bounded-investigation-packets"
heading = "Bounded investigation packets"
headers = [
  "Packet",
  "Question, evidence and decision boundary",
  "Integration and acceptance",
]
kind = "packet"
id_column = 0
status_profile = "packet"
id_pattern = '^([A-Z]+[-]?[0-9]+)\b'

[[scope_tables]]
path = "docs/plans/33-efficiency-principles-remediation.md"
name = "finding-dispositions"
heading = "Finding dispositions"
headers = [
  "Finding",
  "Scenario",
  "Disposition",
  "Decision/work owner",
  "Evidence or completion condition",
]
kind = "finding"
id_column = 0
status_profile = "finding"
status_column = 2
owner_column = 3

# docs/site.toml
[publication]
current_work = [
  "plans/28*.md",
  "plans/32-workspace-content-lifecycle.md",
  "plans/33-efficiency-principles-remediation.md",
]
[[asset_bundles]]
name = "efficiency-principles-codebase-2026-10-09"
root = "design_review/evidence/efficiency-principles-codebase-2026-10-09"
files = ["input-scope-probe.json", "input-scope-probe.py"]
````

**State.** State comes from `[status_profiles.packet|finding]` regex lists (B §2). For
example, `Implemented; …; affected integration remains.` maps to *open* because of the word
"remains".

### (ii) Simplest viable: no new block syntax

These are conventions over existing Markdown, with one checker and generated views.

- **Front matter per family** comes from a small schema declared once (`library-alternatives.md`
  §D). Examples: a review gets `decision`, `standard`, `baseline` and `role:
  principal|supporting`; a plan gets `status`, `review_sources` and `adrs`. Key spelling is
  closed, so `disposition_owner` vs `disposition-owner` cannot recur.
- **Identity** is the document path plus an explicit anchor. The corpus already writes
  `<a id="f02"></a>`, `<a id="rc05"></a>` and `<a id="eff01"></a>`. No new global IDs are
  introduced: the qualified identity of Parallel F01 is
  `design_review_parallel-execution-architecture_2026-10-08.md#f01`, which a view can print.
- **Elements and state.**
  - An element is a table row whose first cell starts with an explicit anchor, or with a link
    to `…#anchor` for a disposition.
  - State is the leading code-span token in the column headed **`Status`**, drawn from a
    closed per-family vocabulary, followed by free prose. Example: `` `implemented` focused …
    controls pass; affected integration remains. ``
  - Elements are found by those rules, not by heading text, so no external binding is needed.
- **Relations** are written once, on the side that changes:
  - plan → review: `review_sources` plus each disposition's link;
  - disposition → packet: a `[EFF01](#eff01)` link in the owner cell;
  - the review is never edited on adoption.
- **Derived views** (generated or queried): open items, "adopted by / disposition of each
  finding" on the review page, plan and review indexes, current work, and evidence staleness.

Exact slice: `probes/merge/S1-status-vs-new-row/ii-simple/base.md`.

### (iii) Typed-block Markdown: the P1-workable syntax

`::: <family> <title> {#id key=value …}` … `:::`.

- **Attribute placement.** Attributes go in the fence info rather than on a separate attrs
  line: P1 #6 showed attrs lines attach across blank lines and need an upward scan, whereas
  info attributes sit on `map[0]`, the line an edit tool targets.
- **Families** are those the skills name: finding, scenario, rule-impact, disposition,
  packet, investigation, rule-change and evidence.
- **Nesting** uses a longer outer fence. A validator refuses `close.markup == ""` (P1 #4).
- **Bodies** are unchanged prose, including headings. Field lists are not used (P2: they
  garble in mdBook).
- **Rendering.** The staging step rewrites each block to `<div id class data-*>` (P2: Tested
  identical to the preprocessor route). Summary tables such as the packet table become derived
  views.

Example, the live F02 prose unchanged inside a block. Body abbreviated here; the full text is
X1.

````markdown
::: finding Qualification applicability omits consumed inputs {#f02}
`validation_scope.RUST_INPUTS`, inherited by Python product scope, omits `test_run.py`, …

**Correction.** Restore conservative consumed-input coverage, …

**Verification.** Retain positive controls, …
:::
````

Exact plan slice: `probes/merge/S1-status-vs-new-row/iii-typed/base.md`. A disposition looks
like this; the quoted value is required because a bare value cannot contain `#` (P1 #7):

````markdown
::: disposition F02 {#d-f02 finding="review:efficiency-principles-codebase#f02" scenario=S01 state=scheduled owner=EFF01}
Complete input/configuration sensitivity and actual refusal of newly stale successful reuse.
:::
````

### (iv) Canonical YAML records with Markdown block scalars, plus prose Markdown

**Strongest honest form.** A plan's or review's *records* live in a sibling
`<doc>.records.yaml`: packets, investigations, dispositions, rule changes, findings' identity,
title and state. Long-form narrative such as purpose, foundations, design and checkpoint stays
in the `.md`. Staging merges the two into one published page.

A wholly-YAML plan is a weaker variant. It would put about 400 lines of prose at 4–6 spaces of
indentation.

Exact slice: `probes/merge/S1-status-vs-new-row/iv-canonical/base.yaml`:

````yaml
packets:
  - id: EFF01
    title: Complete applicability (F02)
    state: implemented
    required_input: |
      Existing complete-input intent; independent of other remedies.
    delivery: |
      Expand scope/effective configuration and bump interpretation. Detect every reviewed omission …
    progress: |
      focused successful-reuse/configuration controls pass; affected integration remains.
dispositions:
  - finding: review:efficiency-principles-codebase#f02
    scenario: S01
    state: scheduled
    owner: EFF01
    condition: |
      Complete input/configuration sensitivity and actual refusal of newly stale successful reuse.
````

ruamel.yaml 0.19.1 round-trip edits are byte-stable only with the file's indent declared
(`library-alternatives.md` §0, Measured).

## 3. Walk-throughs

### (a) F02 → resolved with evidence

| Candidate | Edited bytes | What else must change; what notices inconsistency |
|---|---|---|
| (i) | `\| S01 \| Scheduled \| EFF01 \|` → `\| S01 \| Resolved, 2026-10-10 \| EFF01 \|`, plus evidence prose in the last cell | EFF01's Progress cell, the packet section lead ("has not started") and the checkpoint paragraph by hand. The regex maps "Resolved" → complete. Nothing relates F02 to EFF01's state, and a `Resolved` disposition with an open owner packet is not flagged (B §2). |
| (ii) | `` `scheduled` `` → `` `resolved` `` in the Status cell; evidence stays prose in the last cell | The owner cell is a typed link (`#eff01`), so the checker can flag "resolved disposition, owner not `complete`". The packet lead sentence is still free prose that can drift. |
| (iii) | One line: `state=scheduled` → `state=resolved` on the fence line (P1 #11: Tested single-line splice, re-parse identical apart from that token) | Same owner check through the `owner=` attribute. Evidence prose goes in the body. |
| (iv) | One line: `state: scheduled` → `state: resolved` | Same owner check. Evidence prose goes in `condition:` at the right indentation. |

The review file is unchanged in every candidate. In (ii)–(iv) the review page's "F02 →
resolved (Plan 33)" line is a derived view. On GitHub raw view it is absent; the reader
follows the plan link.

### (b) RC05 confirmation recorded exactly once

Today it is written in five places (E2), plus the stale proposed-owner table in the review.

| Candidate | Single write | Derived | Limits |
|---|---|---|---|
| (i) | Not enforced. The plan's section lead dates all rows at once. | Nothing | The review's *Adoption handoff* exists because nothing else shows adoption on the review. |
| (ii) | Plan 33 rule-change row: `` `confirmed` 2026-10-09 `` in its Status cell, with "as amended" and direction prose unchanged | "Rule impacts → confirmation" on the review page and in indexes | The maintainer's superseding *design direction* is prose and belongs in Plan 33's *Design-phase direction*, where it already is. The handoff's restatement becomes unnecessary rather than forbidden. |
| (iii) | `::: rule-change {#rc05-adoption rule="review:efficiency-principles-codebase#rc05" state=confirmed date=2026-10-09}` with the direction prose as body | Same | Same |
| (iv) | `rule_changes: - rule: review:…#rc05 / state: confirmed / date: 2026-10-09 / direction: \|` | Same | Same |

Index prose (`docs/design_review/README.md`, `docs/plans/README.md`) can still restate it in
any candidate. "Once" holds only if those indexes become generated or stop carrying status.

### (c) Rename a heading or reorder a table

| Candidate | Effect |
|---|---|
| (i) | Renaming "Implementation packets and dependencies" or the "Progress" header requires editing the `[[scope_tables]]` `heading`/`headers` (and the `*_column` indexes if columns move). Otherwise `document_lifecycle scope` raises "must select exactly one native table" and reports no item at all. No recipe or CI runs it (B §6.1). Slug-based inbound links such as `#11-rule-impacts-and-disposition` break silently. |
| (ii) | Headings are free, because elements are found by anchor. Columns can be reordered freely because state is found by the `Status` header. Renaming the `Status` header itself is caught by the checker (anchored rows without a Status column). Explicit anchors keep links stable. |
| (iii) | Headings are free. There is no authored summary table, so tables are derived. Block order is free. |
| (iv) | Key order is free. Headings exist only in the rendered projection. |

### (d) Add a packet

| Candidate | Edit | Also |
|---|---|---|
| (i) | Add a row. A new table needs a new `[[scope_tables]]` stanza; the `just plan` skeleton's header matches no binding (B §1). | `current_work` for a new plan |
| (ii) | Add a row with an anchor and a state token | Nothing; the checker validates ID uniqueness and token vocabulary |
| (iii) | Add a block | Same checks, plus fence balance |
| (iv) | Add a list item at the right indentation | Same checks, plus YAML validity |

### (e) Two agents edit the same plan concurrently (*Tested*, `git merge-file`, git 2.43.0)

| Scenario | (i) | (ii) | (iii) | (iv) |
|---|---|---|---|---|
| S1: EFF01 status ∥ new EI07 row at the end of the investigations | clean | clean | clean | clean |
| S2: new EI07 ∥ new EI08 at the same place | conflict | conflict | conflict | conflict |
| S3: EFF00 status ∥ EFF01 status (adjacent elements) | **conflict** | **conflict** | clean | clean |
| S4: EFF01 progress ∥ EFF01 delivery (same element) | **conflict** | **conflict** | clean | clean |

The cause is physical: one table row is one line, so edits to the same or adjacent rows always
touch. The S2 conflict is trivial in every encoding: keep both blocks or rows.

### (f) Retire the review after its findings close

| Candidate | What survives | Hand work |
|---|---|---|
| (i) | Per ADR-0096 and `docs/dev/documentation.md`, the file is deleted and its meaning is moved to owners. Inbound links are repaired to permalinks: Plan 33's `review_sources`, `scenario_sources` and eleven distinct `#fNN` links (19 references to the review in all; if Plan 33 is still retained); the review fields of ADR-0169–0172 (`git:<commit>:<path>`); and both READMEs' prose. `retire-plan` covers plans only, and its blocker never clears (B §1). | The `lifecycle.toml` README exception for the evidence folder, the `site.toml` asset bundle and the index prose |
| (ii) | Identity `path#f02` survives as a permalink after repair. The checker lists all inbound references, including typed links, before deletion. | Link repair. This is mechanical because the inbound set is computable, but it is still an edit per referrer. |
| (iii)/(iv) | Typed relations written as logical IDs (`review:efficiency-principles-codebase#f02`) survive textually. A resolver must map the slug to its last path and commit, either from `git log --diff-filter=D` or from a retained ledger. | Clickable Markdown links in prose still need repair. In (iii) the attribute is not clickable on GitHub. |

### (g) The evidence bundle goes stale after a pin, toolchain or schema change

**Re-expression (common to (ii)–(iv)).** Only the location of these keys differs between
candidates: README front matter in (ii), an `evidence` block in (iii), a `.records.yaml` in
(iv).

````yaml
question: Do the five canonical-selection query shapes select the intended scoped composite index without Sort or table scan?
answer: All five use IndexScan on result_cell_rows, result_output_range, result_block_page, analysis_edges, study_candidates (Interface-checked; empty tables; no cardinality or timing claim).
conditions:
  surrealdb: {version: 3.3.0, source: upstream SDK}      # now =3.3.0 at path vendor/surrealdb (patched)
  schema_interpretation: pse.substrate.v1                 # now pse.substrate.v3
  data: empty application tables
producer: none retained (the EXPLAIN FULL statements are in query-plans.json "query" fields)
consumers: [Plan 28e (named by the README; 28e does not link back)]
````

| Candidate | What detects staleness | What refresh or delete touches |
|---|---|---|
| (i) | Nothing. Conditions are prose (X5), and C §3 found no comparison of recorded conditions against current ones anywhere. | Refresh: README and JSON. Delete: the folder, the `site.toml [[asset_bundles]]` entry, the `publication.asset_bundles` name and the `lifecycle.toml` exception. |
| (ii)–(iv) | A shared checker resolves each condition key against its current source and lists the mismatches. The keys and sources here: `schema_interpretation` against the `INTERPRETATION` constant; `surrealdb` against `Cargo.lock` plus the dependency source (the `path = vendor/…` override). The same keys serve `input-scope-probe.json`: `input_scope_version` against `INPUT_SCOPE_VERSION`, and `python` against the toolchain. **A version-only key would miss the vendored patch** (C §2). The condition vocabulary needs source identity, not only version numbers. | Refresh: rerun the producer (when declared) and rewrite conditions and answer. Delete: the folder, plus asset selection if that stays a separate `site.toml` list. If attachments are selected from the bundle's own record, the delete touches one folder. |

The detection machinery and its condition resolvers do not depend on the representation. The
representation decides only where the keys are written and how an agent edits them.

### (h) Rendering on GitHub raw view and mdBook 0.5.4

| Candidate | GitHub raw (reasoned from the earlier synthetic render) | mdBook (P2, *Tested*) |
|---|---|---|
| (i) | Tables render | Tables render |
| (ii) | Tables render. State tokens show as `code`. | Same. Derived views must be generated before mdBook runs, in staging. |
| (iii) | Fence and `{…}` lines show as literal paragraph text; bodies render. State and relations are readable only as attribute text. The authored file has no packet summary table unless a generated view is committed (generated-path governance). | Needs the staging rewrite or a preprocessor (they were byte-identical). Without one, default definition lists garble the page (route C). Field lists inside blocks garble unless converted (route A). Div ids are present, and Pagefind records them as anchors, but filters stay page-level. |
| (iv) | The `.records.yaml` shows as a YAML code view: prose inside block scalars is not rendered as Markdown. The `.md` lacks the records. | Needs a staging projection that renders records to Markdown. Not run. |

## 4. Per-candidate summary

| | (i) baseline | (ii) simplest viable | (iii) typed blocks | (iv) YAML records |
|---|---|---|---|---|
| **Mechanical decisions left to the authoring agent** | Front-matter keys and spelling; heading and table header text; ID prefixes and whether to anchor; status wording; where to restate confirmations; evidence layout and README; whether to add lifecycle exceptions or bindings, `current_work` and asset-bundle entries | Prose, headings, non-state columns, which elements exist and which state token applies. The system decides anchor form, state column name and vocabulary, family keys and relation direction. | Prose, element order, headings inside bodies, fence length for nesting, and quoting attribute values that contain `#`. The system decides family names, attribute keys and vocabularies. | Prose inside block scalars and its indentation. The system decides all structure. |
| **Views that become derived** | ADR index (already); lifecycle scope projection (regex, approximate) | Open items; review adoption and disposition view; plan and review indexes; current work; evidence staleness; ID-collision report (E3) | The same, plus summary tables (packet and disposition tables) | The same, plus the whole published page |
| **Still hand-maintained** | `lifecycle.toml` (48 exceptions, 48 bindings, 6 relationships, 22 legacy entries), `site.toml` current work and asset bundles, index prose, and agreement among packet cell, disposition cell, lead sentence and checkpoint (E1) | Checkpoint and lead-sentence prose (can still contradict tokens); index prose if retained; link repair at retirement | Checkpoint prose; the staging transform (a small owned component); field-list or other syntax not covered by the transform; generated views committed for GitHub if wanted | Checkpoint prose; projection code; indentation; the split between the `.md` and `.records.yaml` of one document |
| **Edit form for agents (exact-string replacement)** | One table row per element, and long cells: a unique match needs long strings | Same as (i) | One fence line per element's state and relations; prose lines separate | One line per field; prose lines carry a fixed indent |
| **Concurrent edits (e)** | Adjacent or same-row edits conflict | Same as (i) | Only same-place insertions conflict | Same as (iii) |

## 5. Uncertainties and limits

- **Merge results are line-layout facts on one slice.** A (ii) variant that writes long tables
  as one element per line, such as lists, would behave like (iii)/(iv) in S3/S4, at the cost
  of losing the table.
- **The (iv) projection and a combined `.md` + `.records.yaml` page were not built.** Nor was
  ruamel round-tripping of the generated slice; only the earlier §0 fixture was run.
- **Not run:** the derived views, checkers and condition resolvers are described, not
  implemented.
- **GitHub raw-view statements for (iii) and (iv) are reasoned,** from the earlier synthetic
  render and from GFM, not re-rendered.
- **The P2 staging comparison covers one fixture page.** The definition-list result covers the
  whole staged corpus.
