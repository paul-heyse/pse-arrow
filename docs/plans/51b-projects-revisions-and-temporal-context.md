---
title: "Plan 51b: Projects, revisions and temporal context"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, preparation, storage]
---

# Plan 51b: Projects, revisions and temporal context

## Resulting workflow

**Proposed:** Rust, Python and CLI users can create a project, fork a named branch from an exact
admitted source closure, save incomplete work, compare revisions, explicitly publish/merge and
undo without erasing accepted science. Historical context and preparation discovery make the
same work explainable. O01–O03/O11 and their state are owned by the
[coordinator](51-engineering-workspaces-and-evidence.md#opportunity-dispositions).

A project organizes existing authored definitions, physical source, studies and references.
It is not another model language, mutable solver session or persistence of a whole Salsa database.
Presentation labels, bookmarks and user notes do not affect scientific identity. A writable branch
owns its canonical source-problem/head; a read-only bookmark can name any exact revision. A fork
copies/adopts a protected immutable source closure through ordinary source admission under the
new problem owner. Merely attaching two labels to the same CAS head would not create branches.

## Existing behavior and extension seams

**Interface-checked:** Python `Runtime.modeling_from_documents`, `modeling_package` and
`modeling_revision` already obtain admitted modeling packages. `ModelingPackage` exposes immutable
`revision_id`, `canonical_revision`, `canonical_problem`, `with_declarations`, `with_delta`,
`reload_from_parent` and source tables. Runtime `workflow/modeling/documents.rs::SourceDelta`
specifies an exact local parent and add/replace/remove declaration, document and fit edits;
omissions inherit. `canonical/delta.rs` produces immutable local revisions, while durable
`canonical.rs::admit_revision` and `pse-operations` canonical edits use source-head CAS and
operation settlement. RPDS roots and owned bytes support local sibling revisions already.

Local sibling construction and study variants are not yet a persistent project/branch/draft UX.
Build that missing composition around the existing identities. Do not replace local admission
with a database transaction per keystroke, persist live evaluators or introduce an additional
native working collection. The existing source owner remains the only place that can admit
executable revisions; raw drafts are recoverable editing material.

## Interaction and history contracts

A draft records its branch, exact admitted base, draft generation and original text/typed edits.
Saving invalid text succeeds as draft storage and retains admission diagnostics separately.
It cannot acquire an executable revision identity. Parsing/admission happens locally, with
success producing the existing immutable package; explicit publish checks the branch head and
draft generation and uses ordinary durable source admission. Do not hold a transaction open
across human edits or compilation. An interrupted save/publish settles its original operation.

Comparison has at least source-only, value and structural distinctions. Resolve original
logical object identifiers and source locations through admitted declarations; report unsupported
or unmapped relationships explicitly. A renamed presentation label is not automatically a new
physical model, and a changed physical parameter is not hidden as textual noise. Keep numerical
or symbolic equivalence claims within actual existing library evidence. A useful comparison
returns compact changes first and fetches selected source/detail on demand.

Project relationship navigation can expose declared source dependencies and provenance. Keep
those edges distinct from equation dependency, structural analysis and solver execution order;
project graph organization supplies none of those scientific conclusions.

Three-way merge compares base, target head and proposed revision by logical object. Nonoverlapping
object edits may compose automatically. Two changed versions of one object produce a typed conflict,
even when a text algorithm could pick a line. Do not infer a physically meaningful choice from
units, timestamps or "latest wins". User resolutions become a SourceDelta against the current
exact parent and pass ordinary admission. Head movement after inspection rejects publication and
requires a fresh comparison; it never publishes the old decision against a new parent.

Undo publishes a compensating revision or selects an explicit earlier revision for future work.
It does not delete revisions, rewind recorded runs or silently alter a prior study. Draft
savepoint deletion, project soft deletion and scientific closure release have distinct lifecycles.
Retention controls in 51e explain which history remains available after each explicit operation.

For historical engineering/calibration context, store explicit immutable records with an effective
interval, recorded source revision/cut, correction ancestry and declared scope. Acquisition time,
model integration time and catalogue order stay distinct. An effective-time query at a later
recorded cut may select a backdated correction; an exact old run still points to the original
record. This is not native MX time travel or a generic bitemporal wrapper around all fields.
The existing [MX transaction/snapshot semantics](https://github.com/surrealdb/surrealmx/blob/f67f00cb94f4fb4582090cd86b3b6993f8e25300/src/tx.rs)
support short atomic selections, not indefinite historical storage by holding snapshots open.

## Work packages

<a id="PJ01"></a>
### PJ01 — Persistent project and branch navigation

Consume ST01's project/source declaration slice and ST02's owned mutations. Add create/list/open,
fork, rename/bookmark and exact branch/revision selection through Rust workflow, Python facade
and the existing CLI command organization. Return actual project, source problem/revision and
physical closure references so later study creation consumes exact state rather than a mutable
project label. Use native indexed membership queries; opening the project must not prepare every
model or read every result file.

Acceptance: reopen a project, fork one exact package, edit each branch independently and retrieve
the original bookmark after head movement. Foreign-authority references and duplicate/conflicting
names fail at their owner. One public command and Python workflow exercise persistence through
the chosen service. Preserve current store-independent local modeling operations.

<a id="PJ02"></a>
### PJ02 — Durable drafts and recoverable local admission

Save/load/list/delete raw drafts and savepoints with exact base and expected generation. Integrate
with `ModelingPackage.with_delta`/`with_declarations` for an explicit admit action. Keep diagnostics
and failed raw text recoverable without treating them as source truth. Keep only bounded draft
metadata inline; give larger draft bytes opaque raw-artifact custody using existing publication/
protection mechanics. That custody accepts malformed bytes and grants no scientific source
identity. Scientific source closure/admission begins only with the explicit admit operation.
Expose save versus admitted versus published state clearly in all public results.

Acceptance: save malformed text, reopen it after process exit, fail admission without losing text,
fix it and admit locally. Concurrent draft saves yield one winner/typed generation conflict.
Exercise a grouped draft edit with partial discard/savepoint recovery, then cancel a solve launched
from an admitted immutable draft: cancellation must not corrupt the saved draft or undo committed
source effects. Head movement does not silently rebase. No database outage is reported as a
durable save success.
Delete temporary draft sidecars/prototype save APIs when their consumers move.

<a id="PJ03"></a>
### PJ03 — Semantic revision comparison

Consume PJ01's exact selection and current admitted source/physical descriptions. Implement
compact typed comparison plus selected source/value/structure detail, usable before execution
and by later merge/result comparison. Reuse existing source identities and library-owned
mathematical descriptions; avoid bespoke equation canonicalization or compiling every revision
just to display source changes. Python and CLI consume the same typed comparison.

Acceptance: independent authored fixtures distinguish value-only edits, equation/connectivity
changes, presentation changes and unsupported mappings. The expected categorization comes from
known authored changes, not merely re-running the comparison algorithm. Wrong physical/source
context cannot appear as an exact comparison. Remove replaced text-only comparison shortcuts.

<a id="PJ04"></a>
### PJ04 — Publication, explicit merge and undo

Use PJ02's admitted proposal and PJ03's object comparison with ST02's exact head CAS. Implement
explicit publish and three-way conflict resolution, recording source ancestry and command identity.
Fork/import and merge share the source owner; do not add a second acceptance path. Undo produces
an ordinary admitted compensating revision and retains old study/result references.

Acceptance: nonoverlapping object edits merge; overlapping edits require an explicit resolution;
wrong-parent, head-change and uncertain-response cases neither lose work nor publish twice.
After undo, an old run reconstructs its original revision and a new study selects the new head.
Delete any automatic text/last-writer-wins path introduced during development.

<a id="PJ05"></a>
### PJ05 — Historical context and correction selection

Share ST01's source/time declaration slice with SC01/SC02. Add effective-time plus recorded-cut
selectors, correction ancestry and soft-deletion visibility to project context and calibration
navigation. Pin selected records into study/fit provenance through their actual source consumers.
Reject ambiguous overlapping context where the declared selection rule cannot choose uniquely;
do not guess from wall-clock insertion order.

Acceptance: insert a backdated correction and soft deletion, query before/after effective and
recorded cuts and retain an unchanged exact old run. Released historical closure is explicitly
unavailable rather than reconstructed from current values. No universal history service or native
engine-version feature is required.

<a id="PJ06"></a>
### PJ06 — Compatible preparation discovery

Expose the existing preparation artifact census, compatibility/eligibility decisions and consumed
reuse explanations for a selected exact model/physical/provider context. Reuse current cache and
eligibility owners; passive discovery performs no solver run, native compilation or provider
transfer. Show compatible, incompatible and unavailable/unknown reasons with real evidence,
without promising persisted live handles, a whole Salsa image or cross-provider reuse.

Acceptance: value-only and structural edits expose their different actual reuse behavior without
invented invalidation causes; a known compatible artifact is discoverable; a changed structural/
provider context
is rejected or unavailable by the original eligibility rules; an absent explanation is unknown,
not a fabricated cache-miss cause. Exercise the public project inspection consumer. No parallel
cache/catalogue survives where the current owner already supplies the fact.

## Verification, evolution and handoff

**Proposed:** targeted Rust/Python/CLI tests establish the package behaviors; 51g owns composed
qualification. A headless user can fork/edit/study/compare now; a later graphical editor adds
presentation and calls these same operations. A second dataset or solver provider changes its
declared source/context, not project storage or merge policy. A future concurrent collaborator
must still resolve exact-parent conflicts; 51f owns whether live synchronization is worth adding.

Current checkpoint: authored target only. The first usable slice is PJ01/PJ02 with ST02, not a
complete editor or preparation-persistence migration. Package progress remains solely in the
coordinator; no project, history or reuse implementation is claimed.
