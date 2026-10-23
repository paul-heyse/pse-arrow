---
title: "Plan 51b: Projects, revisions and temporal context"
status: complete
date: 2026-10-22
adrs: [ADR-0217]
review_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
scenario_sources: [docs/functional_reviews/surrealkv-surrealmx-functional-extensions-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/51-engineering-workspaces-and-evidence.md
doc_topics: [current-work, design, preparation, storage]
---

# Plan 51b: Projects, revisions and temporal context

## Resulting workflow

**Implemented:** Rust, Python and CLI users can create a project, fork a named branch from an exact
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

Persistent project/branch/draft interaction now composes those existing local sibling and study
mechanisms around their original identities. Do not replace local admission
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

## Implemented realization

**Implemented:** registry-owned project navigation, immutable draft versions, savepoints, correction
records and preparation documents generate native installation and strict Rust/Python contracts.
Metadata selection is indexed and bounded. Branch names are presentation; branch keys and source
problem ownership remain stable. Frozen project selections bind store, project, exact source and
physical identity and reopen after head/name changes.

Raw draft heads advance monotonically and select immutable versions. Savepoints retain original
versions; restore creates a new generation. Larger malformed bodies retain original sealed artifact
intents. Explicit local admission stores version-linked diagnostics without moving a branch. Typed
`ProjectDraftDelta` recovery authenticates the exact store/base and original local token before
constructing the existing SourceDelta. Accepted immutable publication receipts settle before later
mutable head/draft reads. Superseded direct document-publication and inline CLI save paths were
removed. Normal service opening never initializes or resets a database; regenerated early-design
data needs no historical migration.

Logical merge uses existing declaration/document/fit identities, exact ancestry and original source
CAS. Same-object conflicts need typed choices. Source offsets alone do not create logical conflicts.
The derived whole-package checksum is excluded from logical manifest conflicts; actual manifest
fields remain exact. The existing loader recomputes that checksum only from complete original
byte inventories. The private admitted composition proof preserves complete original text only when its declaration
and fit preimage matches; otherwise preview reports unavailable composed text. The public IR/text
guard remains intact. Full original inputs and extra merge parents remain rooted in ancestry.
Undo creates a compensating revision; it does not rewrite old studies.

Historical context has signed UTC microsecond instants, half-open intervals, a separate project
journal cut and transitive correction ancestry. Both original authored and physical sources are
protected and reopened through existing owners. Actual study/worker/point, fit/horizon, lazy
occurrence, replay and review consumers retain the selected record. Changed scientific edits clear
inherited context unless original source/physical identities still match. This is the minimum
context bridge required by PJ05, not SC01–SC03's wider dataset/calibration/parameter publication.

Passive discovery snapshots at most 32 resident descriptors without cache gets, recency or counter
changes, and pages 32 exact canonical product headers with an issuer/problem/revision-bound cursor.
Current existing owners establish compatibility where possible; body-only/persisted evidence is
unchecked for receiving eligibility. No recipe reconstruction, native handle or provider transfer is
performed. The public preparation wire mechanically projects the existing binding variants.

Rust, Python and dedicated `pse-project` adapters share these owners. The CLI retains original typed
intents and one allowance, creates one runtime/pool and drains it before disconnecting. Deep public
scientific continuations use heap frames charged to the existing runtime pool; stack limits and OS
memory policy were not raised to conceal oversized frames. No backend, library pin or scientific
solver mechanism was replaced.

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

## Verification and handoff

**Tested, 2026-10-22:** targeted Linux controls use the selected SurrealDB3.3.4/MX1.0 service,
private initialized fixture databases and a zero-failure baseline. Arrow-consuming closures use
explicit scope-verified force-validation; Python uses the rebuilt correctness extension. Native
fit testing selects only the pinned solver environment, not a new full-native bootstrap.

| Command / selected scope | Final result |
| --- | --- |
| `scripts/pse-env -- just codegen` | Passed all five generation targets; strict Rust/Python documents, native declarations and schema outputs published through generators |
| `scripts/pse-env --store -- just unit-package pse-operations 'test(canonical_projects::tests::) \| test(canonical_contexts::tests::) \| test(passive_product_headers)' --features pse-operations/canonical-tests` | 13 passed, zero failed; navigation/version/visibility, source CAS and original receipts, context corrections/cuts/physical roots, bounded header pagination |
| `scripts/pse-env --store -- just unit-package pse-runtime 'test(project_comparison::) \| test(project_large_draft) \| test(project_admitted_draft_run) \| test(project_context_recorded_study) \| test(passive_preparation_discovery) \| test(passive_metadata_inspection)' --features pse-runtime/canonical-tests` | 16 passed, zero failed; parsed independent/overlapping edits, original text and derived package checksums, foreign-issuer key collisions, exact merge/undo replay, malformed/large custody and cancellation, actual one-occurrence recorded science, original context/lane attribution and passive inspection |
| `scripts/pse-env -- just py-sync-correctness`, then `scripts/pse-env --store -- .venv/bin/python -m scripts.python_tests python/pse/tests/test_projects.py python/pse/tests/test_project_contexts.py python/pse/tests/test_preparation_discovery.py python/pse/tests/test_project_cli.py -m component -q -n 4 --basetemp FRESH_OWNED_DIRECTORY` | Rebuilt correctness extension; 5 passed, zero failed. Exercises real Python custody/admission/publication, version/savepoint recovery, frozen selections, historical reopen, passive discovery and actual executable selection/reopen across processes |
| `scripts/pse-env --native=solver --store -- .venv/bin/python -m scripts.arrow_validation test -p pse-runtime --lib --locked --features canonical-tests,solver-ipopt,force-validate -- workflow::fitting::oracle::lm_inquiry::fitting_residual_lm_public_result_reopens_actual_method_termination_and_profile --exact` | 1 passed, zero failed; actual two-parameter/four-observation LM fit, stored method/status/profile, selected historical context and unchanged receipt after a later correction |
| `scripts/pse-env --native=solver -- .venv/bin/python -m scripts.arrow_validation check --keep-going -p pse-runtime --all-targets --locked --features canonical-tests,solver-ipopt` | Passed all-target affected compile, zero errors/warnings |
| `scripts/pse-env -- just project-build` | Dedicated executable rebuilt against repaired core; actual argv exercised by the Python component above |
| `scripts/pse-env -- just ready` | Passed environment and configured deployment checks; unrelated historical unsupported-profile resource cleanup remains deferred, with those resources untouched |

The final Runtime control invocation is `2eb900cb336e4e9d96675f3de6318ed1`
(Nextest `810f59bc-9419-405c-ab50-95f24fc6287a`); storage is
`2e8efda2090a4cbf92fb96b8cc8ada81`; the final public five-component cut is
`d946351a48c54058869fcbed5513b68a`. Earlier native context and real CLI-handler controls
also passed (3 and 1 respectively); they do not broaden the final scopes above.

**Not run:** full hygiene, whole-series integration/scientific qualification, performance
campaigns or other platforms. These remain Q01–Q04 in 51g. **Measured:** no performance
improvement is claimed; passive reuse evidence is not a throughput result.

**Tested (mechanical publication):** `scripts/pse-env -- just turn-end` passed ADR indexing and
formatting; `scripts/pse-env -- just docs` published 614 chapters and scoped search. Neither is
product qualification. All three Plan51b implementation worktrees were merged with their authored
changes already integrated; merge resolution retained the qualified main implementations and
produced no additional tree changes. The drained worktrees and merged branches were removed.
The supplied untracked library reference remains untouched.

## Outcome

**Implemented:** PJ01–PJ06 compose persistent project navigation, recoverable raw versions and
local admission, logical revision comparison, explicit guarded merge/compensating undo, historical
context and passive preparation discovery. Existing scientific admission, publication, retained
source/artifact owners and reuse eligibility remain authoritative. The minimum actual study/fit
context bridge is implemented; SC01–SC03's wider research catalogue remains separate scope.

**Tested:** bounded independent implementation review identified wrong raw-artifact ownership,
missing exact draft-base authentication, foreign-source revision-key collisions, unparsed merged
text claims and retained-worker context reuse. Each was corrected at its owning mechanism, and
static rechecks found no remaining blocker in the repaired boundaries. Functional controls exposed
oversized nested public continuations, a draft-head API consumer left on the old shape, and false
manifest conflicts caused by a derived whole-bundle checksum. Charged heap continuations, explicit
immutable version access and existing-loader checksum composition corrected those roots. The
initial 11-pass/4-fail Runtime bundle became 16-pass/zero-fail after repairs and the added foreign
control. The public Python journeys passed after real native stack failures were repaired; no
thread-stack increase or acceptance weakening was used. The strengthened fit fixture initially used
synthetic physical state and a pre-fork project generation; authentic physical documents and a
fresh descriptor for the newly issued context command corrected those fixture errors. The actual
fit receipt is reread after correction. Three compiler-reported redundant test
qualifications were removed. An affected native compile was initially invoked without its solver
environment and rejected the workstation's old system Ipopt; the explicit pinned native route is
the required command above, not a dependency downgrade or ignored failure.

Deliberate choices: UTC instants rather than a second time vocabulary; immutable versions and
compensating publication rather than history rewinds; conservative original-text availability and
unchecked persisted eligibility rather than invented equivalence/reuse claims. Complete project
exchange, scientific closure release, collaboration and whole-series qualification remain at their
existing 51c–51g owners. No backend, dependency pin, OS memory policy or solver was changed.

Enduring meaning is at blueprint §14.3/§20.6 with proposed ADR-0217; operational use is in the
[substrate guide](../dev/surreal-substrate.md#project-and-derived-evidence-operations).
The coordinator owns package and opportunity disposition. Keep this completed packet while
51c–51g depend on its contracts and targeted evidence; retire it through Q04. Full series
completion, ADR acceptance and quantified benefit are not claimed.
