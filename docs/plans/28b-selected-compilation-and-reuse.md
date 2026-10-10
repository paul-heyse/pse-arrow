---
title: Selected compilation and durable reuse
status: in-progress
date: 2026-10-05
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md, docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md, docs/design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md, docs/design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28b: Selected compilation and durable reuse

## Responsibility and basis

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns the selected scientific
compilation boundary, dependency-complete durable descriptions and native preparation reuse.
[28a](28a-canonical-substrate-and-revisions.md) supplies immutable object selections and
guarded revisions; [28c](28c-durable-execution-and-studies.md) consumes admitted descriptions
to execute runs. Database queries perform structural selection and dependency operations;
existing scientific kernels retain physical interpretation, inference and finite specialization.
Current shared decisions and finding dispositions belong to the coordinator.

The [preparation integration](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
adds B7 after confirmed rule decisions. [28i](28i-runtime-validity-and-interruption.md) owns
receiving validity/interruption and [28j](28j-pure-preparation-and-publication.md) owns the
pure basis/effect contract. This companion adopts their working contracts at selected
preparation and reconstruction; the coordinator owns PA01–PA04 dispositions.

The affected foundations are `pse-compiler::workspace`, `pse-modeling`, the runtime's
`workflow/modeling`, `authoring_driver`, `artifact_requests` and `math` owners, and
`pse-buildinfo/build.rs`. The current source route hydrates a complete source package through
generated checked batches and declarations before constructing a modeling revision. The
compiler already has demand-aware selection and specialization worth retaining. Persisting
the same whole-package hydration would leave US05 and much of EF01/EF02 intact.

The accepted direction changes blueprint §14.3–§14.4, §20.4 and the reuse portion of D14
through the coordinator's R0 decision route. It preserves complete physical typing, authored
mathematical meaning, contextual property validity and provider eligibility. Plan 27's
contextual accuracy contracts and their same-point evidence remain consumed scientific inputs.

## Target input and product contracts

The compiler receives a resolved demand over immutable version identities, the semantically
sufficient closure, and its interpretation context. It does not receive an entire problem
bundle merely to recover these inputs. Native database queries handle name and membership
resolution, graph selection, reverse dependency lookup and persisted product discovery. Rust
owns a shared kernel for physical inference, finite elaboration and scientific admission. All
production entry points use this kernel; database functions must not introduce a second copy
of those scientific rules. A genuinely global obligation may consume a whole selected scope,
with that requirement explicit rather than hidden in routine hydration.

Acquire A's protection for immutable source/interpretation selections before long-running
compilation or native preparation. Publish a durable description with its retained roots
under the same retention guard before releasing protection. Expired preparation cannot admit
a new product. This prevents an edited-away source being reclaimed between selected reads
and product publication; scientific work still runs outside the transaction. The early A2
slice supplies minimal protection/root admission before full A3 retention.

Keep declaration meaning at its existing semantic owner. [28a's schema and codecs](28a-canonical-substrate-and-revisions.md#schema-and-scientific-values)
mechanically lower it for storage. A selected kernel input can be a compact typed view of
those records. Arrow remains useful at numerical and export boundaries; constructing every
generated relation batch is not an admission prerequisite when the consumer needs only a
selected subset. Retain the generated shapes that still serve an actual consumer.

A durable compilation description records its demand, exact source versions, interpretation,
selected equations and variable/output coordinates, provider/capability selection, and the
portable recipe needed to construct numerical products. It also records the dependencies
that establish its eligibility. It contains no native pointers, evaluator closures, mutable
solver sessions or process-local Salsa handles. Reopening it performs compatibility and
eligibility checks against its recorded interpretation, rather than reinterpreting it using
the current defaults. Reconstructing a numerical product is distinct from redoing semantic
compilation. B2 includes working persisted-payload reconstruction for an ordinary solve,
using the existing numerical builder over recorded selected mathematical meaning. C2 consumes
this implemented slice; B3 broadens provenance and multi-workflow product sharing.

Dependencies include positive references, absent names, membership scopes, deletions and
provider/compiler/library/configuration interpretations actually consumed. An absent name
uses [28a's named guard](28a-canonical-substrate-and-revisions.md#revision-and-conflict-contract);
an open collection uses a scope guard. Adding an unrelated object should not invalidate a
product whose meaning does not depend on that scope. A broad dependency is correct when the
operation actually inspects the whole scope. Do not replace semantic dependencies with a
global head key or with an incomplete list of positive graph edges.

Old descriptions remain meaningful for their immutable revision. A newly selected revision
may reuse compatible descriptions, with its own resolved selection and admissibility evidence;
it must not relabel an old description as current without checking the changed dependencies.
Database functions and executable schema constraints have versioned interpretation identities.
Their changes participate in eligibility when their results were consumed.

Validation established for immutable values is retained with the actual predicates and
interpretation that established it. A stored `validated` boolean cannot supply this contract.
Across process restart, reuse depends on controlled immutable writes and a reproducible
interpretation. New imports, changed contexts, new numerical states and incompatible readers
receive the corresponding checks. Correctness tests continue to force validation explicitly.

Salsa can accelerate repeated kernel requests inside a process when it retains useful work.
It is neither the persisted dependency authority nor a mandatory cache alongside equivalent
database products. Native evaluator/sparse/factorization reuse remains governed by the actual
library signatures and mutability. Mutable numerical state belongs to an attempt; immutable
products can be shared where their libraries permit it. Limit retained products by live
consumer ownership and a bounded accelerator policy, rather than retaining all previous
source owners or all past request variants.

## Provenance and physical preparation

Separate complete executable/dirty-build attestation, attached to the run boundary, from the
relevant implementation identities that determine a compilation product. The relevant key
must change when a consumed kernel, provider, library or configuration changes. Moving a
test file or unrelated crate must not become a scientific dependency through a recursive
whole-tree hash. Establish the affected source boundary from the real implementation and its
dependencies, not a manually maintained list that silently omits a helper. Whole-build
observation remains complete; narrowing product keys is not permission to weaken it. The
2026-10-07 accepted production RC01 further separates artifact-specific linked identity and
actual deployment association from complete outer observation. [28h L0/L3](28h-native-setup-and-artifact-identity.md)
owns that target and its decision route before replacement; current linked-attestation receipts
remain historical evidence for the original contract.

This creates a new cross-process and potentially cross-build reuse capability. The existing
process-local cache does not establish that capability already. Version the identity frame
through R0 before relying on it, and retain exact compiler/library interpretation with the
portable description. A changed native ABI invalidates native reconstruction even if a
portable scientific description remains compatible.

EF08 needs a bounded library-contract decision, not an assumed database remedy. Inspect the
actual Symbolica/Numerica evaluator parameter signature used by `support_with_allowance` and
`compile_scope`. If it accepts a compact selected coordinate layout, implement one explicit
mapping shared by preparation and evaluation, preserving output order and all guard/provider
coordinates. If the library requires the complete signature, retain that necessary input and
remove only unnecessary earlier allocation or repeated setup. The outcome and evidence belong
to B4; it does not block A/C/D. SIMD, JIT or batch evaluation are not default deliverables of
this pivot. Adopt them only for a demonstrated supported operation after comparing complete
preparation and execution under equivalent scientific requirements.

## Remaining selected-preparation and deployment slices

F02 and EF05 remain at the [coordinator](28-surrealdb-unified-substrate.md#finding-dispositions).
The [capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
settles native grouping options; scientific closure and reuse eligibility retain this owner.
Current `selected_source.rs` limits namespace reads to namespace owners and builds a parsed-once
reference inventory. Runtime canonical selection groups name acquisition in batches of 64
and processes newly acquired rows/unresolved references. These source changes are partial
F02 correction; they do not establish physical hydration locality or a measured speedup.

| Existing package slice | Delivered operation and implementation boundary | Targeted acceptance and deletion |
|---|---|---|
| B1 grouped frontier, with A2 acquisition | Selected roots, interpretation and exact unresolved demands produce scientifically sufficient closure under unchanged lexical/import/provider rules. Carry namespace inventory and guard premises forward; resolve only the new frontier instead of rescanning accumulated declarations. Owners: `pse-modeling/src/selected_source.rs` and runtime `workflow/modeling/canonical.rs`. | Check nearer-name shadowing, deletion, imports, cycles, absent-name conflicts, unrelated namespaces and expiry. Include sparse `(scope,name)` pairs so independent IN sets cannot accidentally admit cross-pairs. Observe the actual query path to establish removal of repeated crossings/parsing, without a new planner or a fixed RPC quota. Remove remaining superseded singleton reads/rescans once consumers pass. |
| B1 physical hydration, with A2/B2 | Group selected manifests/extent metadata first; fetch bounded payload groups only after extent admission. Produce the same exact Arrow physical input and interpretation consumed by shared kernels and persisted descriptions. Protect selection on misses and hits until product admission. | Scale legitimate declaration count and include mixed-size blocks; show bounded group admission, refusal before oversized decoding, exact bit/shape identity and no unrelated full-bundle hydration. Inspect current hydration before changing it: a fresh targeted receipt must settle the remaining premise. Delete duplicate probes/conversions, preserving required scientific validation. |
| B3 deployed replay, with E2 and 28h L3 producer tooling | Fresh runtime, worker and installed Python extension captures associate actual artifact bytes with their relevant role/input closure and the deployment's separate complete outer observation. Relevant dependencies alone key scientific products; consumers reopen only under complete eligible context. | Qualify actual installed extension/import association, then solve/reopen/reconstruct. Changed relevant provider/config/input refuses reuse; irrelevant source changes outer observation without re-keying the product or recompiling an unaffected artifact. Fixture captures, matching outer hashes and git HEAD alone cannot establish installed eligibility. |

B3 also settles the audit's honest cross-target receipt question. A receipt for a runtime
library or worker must not silently establish installed Python target association merely
because outer source hashes agree. Exercise a genuine qualified receipt presented for the
other executable role and the supported Python receipt with its actual imported bytes;
require explicit role/target association under the existing trusted-operator mint policy.
This is a bounded target-policy control, not a presumed malicious-receipt defect. If the
existing supported mint already enforces the distinction, retain that route and its evidence.

Use direct bound record IDs, explicit projections and exact demand-pair bindings. Native
recursive traversal can supply structural reachability; deduplicated collection and path
enumeration have different meanings. A depth cap cannot certify complete scientific closure.
Check representative indexes/plans with version-scoped EXPLAIN diagnostics when the actual
query shape is uncertain; its output is not a stable golden, cache identity or latency claim.
Do not replace scoped selection with full-package hydration or parallelize redundant probes.
Batch size remains an implementation resource choice constrained by actual extents, not a
new semantic contract. Preserve force-validation and fresh storage protection on cache hits.

Existing EF02/EF08 focused resolution is preserved. Fresh A/B/A, value/structural edit and
selected acquisition controls must compose with C/D consumers before E3. E4 owns comparable
complete-operation timings. The current next step supersedes historical continuation text
below; prior receipts retain their original source and workload limits.

## Ordinary deployment-local replay

The maintainer accepted the enhancement review's [RC01](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#rc01)
on **2026-10-07**. [The coordinator](28-surrealdb-unified-substrate.md#rule-changes-confirmed-by-the-maintainer)
owns that decision; [28h L5](28h-native-setup-and-artifact-identity.md#deployment-local-replay-and-build-output-derivation)
supplies the effective-deployment observations and decision/design route. This section owns
the scientific receiving/reconstruction contract, not a second deployment observer.

At the reviewed `dacc9c3` foundation, default worker/Python admission has no qualified producer;
`CanonicalBodyRetention::get` consequently misses before durable lookup. Memory reuse and
retained outcomes work, but neither proves ordinary prepared-body replay after restart.
B5 adds that capability. Earlier B3 strict-capture requirements describe the stronger guarantee,
not a prerequisite for this new default route or for ordinary execution.

Use one optional opaque replay admission expressing either independently observed deployment-local
compatibility or explicitly supplied strict producer qualification. `CanonicalDeployment` and
the actual worker/Python composition roots consume it; callers cannot deserialize matching JSON
or supply a hash to mint eligibility. Persist a versioned guarantee/identity interpretation through
the existing owning declarations and portable product namespace. A local key scopes the actual
receiving artifact/role, reconstruction interpretation and complete consumed runtime/native context
under L5; existing scientific keys still own selected inputs, providers, profiles and positive/absent
dependencies. Identical outer checkout hashes are neither needed nor sufficient.

Publication and receiving share `CanonicalBodyRetention`, `portable::reuse_body`, the complete
premise checker and strict scientific reconstruction. Preserve bounded hydration, current protected
reads and compiler-issued publication authority. Do not add another decoder, cache or scientific
validator; do not reinterpret historical unqualified records or keys as newly eligible. Complete
context absence or changed premises produces a cache miss/refusal and fresh preparation, while
historical outcomes remain available. Malformed/corrupt products retain their existing observable
refusal semantics. No change forces rebuilding code or requalifying prior results.

| Package | Prerequisite and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| B5 — Ordinary restart reconstruction | Accepted enhancement RC01 and working L5 observation/interpretation slice after its decision route. Publish/reconstruct compatible local products through the existing protected checker. | Migrate default worker and Python import/runtime admission, portable publication and retention together. Replace strict-only replay gating; retain explicit strict admission under its own guarantee. Prove actual restart reconstruction rather than fresh admission, independent artifact/role association, changed native/configuration and missing-context refusal, corruption handling, protection expiry and original dependency invalidation. | Implemented; persisted-hit, context/cache refusal and installed Python controls pass; assembled E3/E4 pending. |

B5's receiving contract can be designed while C5/N5/N6 proceed. Its implementation cannot claim
safe local replay until L5 establishes the supported effective context and stable observation-to-use
lifetime. Unknown dynamic/runtime inputs refuse reuse; ordinary execution need not wait for B5.
Cross-role/cross-build eligibility remains a separately requested stronger scope.

## Shared numerical preparation extension

[28f](28f-shared-numerical-preparation.md) develops PE01/PE04 and comparable numerical variants.
N1 supplies ordered policy/coordinate projections to current assessment consumers; N2 extends
this owner's selected-input admission through the existing bounded math service and fresh
protection. Compiler-issued witnesses, portable descriptions and complete scientific eligibility
remain here. Immutable cached preparation cannot retain an expired storage read or attempt fence.

Ordinary solve/initialization, durable ready occurrences, continuation, fitting, dynamics,
shooting and analyses migrate wherever N0 confirms unchanged preparation is rebuilt. Values,
current source attribution, seeds and original-space assessment remain fresh. N3's library and
native session work consumes this same split; no second compiler/cache or persisted native
state is introduced. [28g T3/T4](28g-bulk-data-operations.md) groups physical source transport
and retains unchanged authored bytes without duplicating scientific admission. [28h](28h-native-setup-and-artifact-identity.md)
supplies actual role association; B3 integrates it rather than maintaining another capture rule.
N4/T5/L4 consumer closure precedes E3. Earlier B package Outcomes remain at their named scope.

## Parallel selected-preparation composition

B6 consumes [A4](28a-canonical-substrate-and-revisions.md#parallel-canonical-protection-and-contention)
and [N8](28f-shared-numerical-preparation.md#parallel-admission-and-native-lifetimes) to correct
Parallel F01/F02 preparation composition. Its baseline is `84a1caf17656f00f38b22e703c7cbc2b63a44d2d`: `checked_selection`
protects each call before dependency recheck/selection and later selected-admission flights.
`CanonicalBodyRetention` may publish durable reachability even on a memory hit. Those are
necessary effects whose equivalent operation/lifetime scope should be assessed, not removed
because numerical preparation is immutable.

Where A4 selects it, share an immutable protection owner only for compatible exact
problem/revision/interpretation and its actual consumer lifetime. Preserve separate complete
positive/absent/membership dependency receipts and per-case starts/results. Partial or failed
selection never becomes a completed shared receipt. Releasing/cancelling one caller cannot
release protection still consumed by another, extend an expired pin or qualify publication
from stale preparation. Keep the owner alive through issued native work/drain where required,
and admit product roots before its final release.

Use existing selected/artifact flights and publication acknowledgment semantics when reducing
equivalent work. Product identity currently consumes the selected protection/dependencies;
changing that identity is not an incidental cache-key edit and returns to A4/R0 if needed.
A compatible mathematical artifact does not by itself establish canonical reachability for a
new revision. B6 does not introduce a second publication checker or merge scientific outcomes.

Replace full worker-capacity charges on affected nested-provider/rebind preparation with
N8's admitted actual/conservative extent and bounded waiting behavior. Keep fresh private
compiler workspaces and private mutable provider/evaluator workers. Pure unchanged-value
checks stay outside unnecessary native jobs. A genuinely required whole-scope check remains
explicit. Migrate ordinary preparation, studies, initialization and demanded analysis consumers,
not only the conformance fixtures.

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| B6 — Parallel selected preparation | A4's working protection/publication slice where changed; N8 temporary admission. Compose exact selection/reachability effects at their valid reuse lifetime and move affected preparation/rebind callers to demand-based admission. | Remove displaced per-call equivalent protection/publication work only when replacement semantics are demonstrated; preserve receipts/eligibility. Test sixteen small consumers, mixed compatible/incompatible demands, changed revision/absence/provider, cancelled shared waiter, expiry, retained pressure and original scientific results. | A4 retains protection/identity and paces staging; N8 callers migrated with targeted demand controls; N4 and E3 acceptance pending. |

## Complete selected-preparation adoption

B7 moves `checked_selection`/`canonical_workspace`/`prepare_modeling_revision` composition
to explicit current selection effects plus J's reusable basis. The current preparation
includes values and lineage: neither the entire `PreparedModeling` nor its runtime wrapper
may be shared under an incomplete structural key. J0 defines the substitutable subset and
fresh attribution before B7 integrates it. Complete positive, absent, membership, physical,
provider and demanded-input dependencies remain authoritative.

Canonical discovery and reconstruction run outside tracked queries. Qualified admitted
products populate pure retention; exact receiving and persisted-payload admission remain
explicit. Every warm or cold selection discovers all required descriptions and publishes or
settles them before new protection is released. Exact existing acknowledgment reuse does not
require a new row/root for each selection. Root validity and current revision/instance/spans
are separate from immutable mathematical meaning.

| Package | Prerequisite and target behavior | Completion and status |
|---|---|---|
| B7 — Selected basis and receiving adoption | J0/J1/J2 and A5's working canonical slice; applicable I0/I1/I2 receiving profile/control. Integrate current source eligibility, description acquisition, pure basis lookup, fresh binding and explicit effects at actual Rust/worker/Python entries. | Implemented; Tested complete-basis/current-attribution, A/B/A, changed dependencies/provider/root, acquired absence and pin/ack controls. Displaced callbacks and canonical compiler workspace are removed. 28e owns the rebuilt affected journeys and qualification boundary. |

Pure same-process work need not wait for an unsupported-root replay guarantee. B7 can design
against the settled J contract while I0 investigates receiving protection; actual replay
integration waits for its supported premise argument. B7 supplies C8/N11 rather than
reimplementing their workflow policies or numerical state ownership.
Existing source selection/reconstruction primitives and an early J0-defined B7 interface
slice supply J1/I1 before full B7 migration. This breaks the apparent whole-package cycle:
J1/I1 need those specific contracts, while B7's completed consumer adoption needs working J2
and the applicable receiving implementation.

<a id="b8-retained-supplier-topology"></a>

## Graph/hash extension — B8 retained supplier topology

This **Implemented** extension supplies the compiler portion of
[Graph/hash F03](../design_review/reviews/design_review_graph-compilation-kernels-and-hashing_2026-10-09.md#f03).
The [coordinator](28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
owns the finding; [28f/N13](28f-shared-numerical-preparation.md#n13--demand-specific-supplier-registration)
owns runtime registration integration. B7 is a working foundation, not a missing prerequisite
to reimplement. Earlier baseline descriptions retain their original scope.

### Foundation assessment and target

At the investigation baseline, `AdmittedModeling::implicit_order_for` constructed nodes and
supplier edges from every implicit body's value-level output demands on each use. Selected
requests derived observation roots, traversed reversed dependencies, removed unrequested nodes
and only then topologically sorted. Immutable admitted bodies and existing shared allocation owners make
this topology suitable for preparation once. Fresh requested scope and numerical demand
remain separate decisions. AP-07 supports retaining discovery; AP-04/G6 require preserving
the distinctions that govern selection and execution.

Retain immutable supplier adjacency and stable identity-to-node correspondence with the
admitted product; derive observation roots for each selected request. Use the existing petgraph traversal
and topological algorithms first; a compact representation is eligible if it preserves the
consumed relation and reduces total machinery. The completed
[28k/GH1 inquiry](28k-graph-kernels-and-hashing-investigations.md#gh1) supports retaining
petgraph for this correction: case preparation can rediscover the immutable topology at Value
and again after binding at a higher derivative order, and observations also consume it.
No CSR or cross-call flow-cache benefit was established. Supplier edges express
dependency reachability, not physical connection occurrences; any deduplication must preserve
this particular consumer's meaning rather than establish a universal graph policy.

Construction supplies dependency discovery, not global acyclicity. Each request consumes
its selected rows or full scope and produces suppliers before consumers, or the existing
typed refusal. Preserve selection closure before cycle rejection: an unrequested cycle cannot
reject a valid selected closure, while a full-scope request must still reject it. Do not
precompute one globally accepted order and impose it on all requests. Likewise, preserve
missing observation/body and provider-demand failures at the scope that currently requires
them; eager preparation must not turn an unused row's failure into unconditional refusal.
Keep observation-root discovery selection-local if preparing all roots would broaden failure
scope. Adjacency can be shared without forcing that additional eager check.
In particular, current implicit-body provider-demand discovery is inventory-wide and
precedes selection filtering; preserve that obligation and its failures. Observation-body
demand/root lookup is selection-local. Unrequested cycle rejection and unrequested
observation failures must not be confused with those inventory-wide discovery obligations.

The topology belongs to the actual admitted primary/original view and its immutable bodies;
changed bodies, supplied systems or structural bindings produce the appropriate new product.
Selected closure/order may remain transient. No cross-revision topology cache, graph hash,
persisted runtime, public wire shape or second dependency authority is introduced.
Construction and retained/transient containers use current source bounds and allocation
owners, including escaping child aliases. Update retained-byte accounting before adoption;
sharing an Arc does not exempt adjacency/index storage from its charge.

### Package and scoped acceptance

| Package | Prerequisites and delivered capability | Consumer boundary | Progress |
|---|---|---|---|
| B8 — Retained supplier topology | Working B7/J2 and compiler admitted-body/view owners. Build reusable dependency discovery and select closure/order under existing semantics. | All compiler `implicit_order`/`implicit_order_for` views, original views and N13 registration consumers; remove per-use graph/body-demand reconstruction after migration. | Implemented; targeted compiler/native-runtime controls passed; affected journey status is owned by 28e. |

Use independent small graphs with known predecessor relations/order constraints, including
isolates, sibling/nested suppliers, reordered identities, empty/full selections, changed
edges, an unrequested cycle and a selected cycle. Check original versus primary views and
missing/unusable unselected observation bodies without broadening failure scope. Exercise
inventory-wide implicit-demand failures separately from selection-local observation failures.
Compare required supplier sets and ordering constraints, not arbitrary ordering among independent
nodes. Verify retention through eviction/clear and escaping child views. A focused construction
control should distinguish one prepared topology from repeated selection traversal.

Compile `pse-compiler` and its affected runtime consumer; use narrow `just unit-package`
controls with force-validation. B8's working selection API enables N13; agreeing its shape
alone does not establish runtime integration or close F03. Delete the displaced construction
path when callers migrate, retaining independent expected-graph and scientific checks.
[28e](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance)
owns assembled evidence; no timing gain or broader scientific qualification is asserted.

### Implementation checkpoint — 2026-10-09

**Implemented:** each admitted primary/original product owns an
`Arc<once_cell::sync::OnceCell<SupplierTopology>>`. Successful initialization occurs at the
existing `implicit_order_for` boundary; a failed or panicking fill leaves the memo empty for
retry. Inventory-wide implicit-body Value-demand discovery still precedes selected observation
lookup. Observation roots and missing-body failures remain selection-local. The derived memo
is excluded from mathematical equality and shared by owner attachments.

Selected requests traverse reversed dependencies and topologically sort a borrowed filtered
view of the retained graph; full requests sort the complete graph. Unrelated cycles remain
excluded from a valid selected closure. No graph clone/rebuild or `retain_nodes` path remains.
Node-filtered traversal retains the complete node index space, so this is not a claim of cost
proportional only to selected nodes.

Checked accounting reserves graph/node-index storage and one peak synchronous body-demand
allowance, accumulated request roots and whole-index traversal scratch before the product
escapes. It counts full formal-slot/compaction work, not only outputs/provider references.
Sequential body scratch is not summed as if simultaneously live. Owner attachment retains one
shared memo charge; primary and original views retain distinct topology identities.

<a id="b8-scoped-functional-evidence"></a>

### Scoped functional evidence

**Tested:** `just check-package pse-compiler -p pse-runtime` passed with zero failures and
warnings against a zero baseline. The targeted force-validation command below passed **6/6**
compiler tests on 2026-10-09:

```bash
just unit-package pse-compiler 'test(owned_frontier_inventory) | test(selected_shared_observation_body_demands_only_its_implicit_provider_output) | test(supplied_promoted_implicit_descriptors_precede_nested_consumer_planning) | test(supplier_topology_)'
```

Those controls exercise retained portable inventory/original views, selected output demands,
known ancestor/isolated/parallel-edge topology, selected versus unrelated cycles, empty scope,
initialization error/panic retry, synchronized first fill, owner sharing, and request-local
missing-body failure/retry. Owner controls check unchanged memo reservation, shared memo
identity and absence of a second charge independently of vector capacity changes during owner
attachment; lazy-fill accounting stability is checked separately.

**Tested:** the final native runtime selection passed 24/24 controls against a zero-failure
baseline, including selected nested observations, sibling-hint ordering, Value-demand
propagation and composed child second derivatives. The current registration consumers use
B8 ordering without sharing mutable evaluator state or replacing their derivative demands.
[28e's graph/hash acceptance](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance)
owns the exact commands/native/force-validation conditions and remaining affected journeys;
[28k's followup outcome](28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09)
records integrated implementation and inquiry conclusions. Finding dispositions remain at the
coordinator. These scoped results establish neither a latency improvement nor broader
E3/E4/E5 or scientific qualification. Portable reconstruction remains the production restart
path; the real Recipe persistence inquiry does not select an RC03 cutover.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| B1 — Selected kernel boundary | R0 and implemented A2 selection/codec slice. Resolve selected physical/modeling inputs and share admission/inference/specialization kernels. Preserve required context and global obligations. | Migrate authored `.pse`, Rust and Python modeling entry points and workflow composition. Remove their complete-bundle hydration requirement and `BundleOwner::_parent` retention when the last caller moves. | Implemented; scoped controls recorded; assembled E3/E4 pending |
| B2 — Durable descriptions and dependencies | B1 plus A2 immutable storage/guards/protection. Persist portable products and complete dependencies; implement ordinary-solve reconstruction, discovery, eligibility and invalidation. | C2 consumes working reconstruction; move Salsa to optional acceleration and delete overlapping cache/validation mechanisms with no remaining consumer. | Implemented; scoped controls recorded; assembled E3/E4 pending |
| B3 — Provenance and numerical preparation | B2 and R0 identity decision. Separate outer attestation from product keys; reconstruct and share compatible immutable numerical products without retaining whole revisions. | Migrate artifact requests and value-only, dynamics, fitting and study preparation. Delete global build keys from scientific reuse and unconditional repeated preparation. | Implemented; scoped controls recorded; assembled E3/E4 pending |
| B4 — Formal coordinate preparation | B1 and exact library signature evidence. Decide compact mapping versus required complete signature, implement justified improvement and record any remaining limitation. | Migrate every consumer of the changed evaluator layout together. Remove displaced dense maps/tests; preserve actual guard/provider work. | Implemented; scoped controls recorded; assembled E3/E4 pending |

These packages cross document boundaries. B1 can begin after A2's selected version and codec
slice works; it does not need A3's complete revision authoring UI. B2 can initially publish a
description for that selected immutable input without C's full run lifecycle. B3's attempt
consumers integrate with C2/C3 rather than inventing a temporary run service. Root coordinates
shared runtime/modeling declarations, registry generation and manifest edits.

## Verification

**Proposed acceptance:** compile touched packages with `just check-package` or `just check`
for cross-crate work, and use recipe-owned targeted units with force-validation. Each new
mechanism's controls accompany its implementation. Full qualification is owned by [28e](28e-rebuild-retirement-and-qualification.md).

Expose mistakes using the review's S01/S02/S05/S08 journeys:

- A value-only edit changes the admitted input/start while compatible structural and sparse
  products remain reusable; a structural edit changes the required coordinates and products.
- Define a previously absent name, delete a referenced object, change a relevant membership
  scope, or change provider interpretation. Each invalidates exactly the eligibility it
  affects; an unrelated edit does not trigger complete-package hydration.
- Equivalent scientific requests after restart reconstruct the same coordinate meaning and
  independently specified small physical outputs without repeating semantic compilation;
  a targeted reuse control distinguishes reconstruction from full recompilation.
  A changed consumed implementation invalidates
  reuse; an unrelated test edit changes outer attestation without changing the product key.
- Changed validation context is rejected or rechecked even for previously admitted values.
  Explicit force-validation exercises the checks retained for immutable reuse.
- A/B/A request sequences and many unreferenced revisions do not retain complete ancestral
  bundles. Bounded cache eviction may recompute; it cannot change eligibility or semantics.
- B4 checks reordered, missing and guard-only coordinates against explicit expected layouts
  and mathematical values, rather than an oracle generated by the same compacting routine.

Do not claim a compile-time or solve-time gain from these structural controls. E4 owns any
quantitative comparison; B4 may conclude that some dense library input is necessary.

## Checkpoint and next step

Current execution handoff, 2026-10-10: B1–B8 mechanisms are implemented;
scoped evidence retains its original conditions and does not close Plan 28.

Selected sufficient preparation, qualified portable reconstruction, exact immutable basis reuse, indexed description settlement and retained supplier topology are implemented. Complete authored checking remains the selected contract; changed demands, providers, layouts and current attribution retain their owners. Strict current-artifact qualification remains optional.

The [coordinator](28-surrealdb-unified-substrate.md#current-checkpoint) owns existing finding
dispositions and [28e](28e-rebuild-retirement-and-qualification.md#checkpoint-and-next-step)
owns the current campaign checkpoint, affected prerequisites and conditional continuation.
The subsequent SurrealDB review has its separate corrective owner in
[Plan 35](35-surrealdb-lifecycle-and-integration-remediation.md#existing-plan-coordination).
Follow those owners before selecting another campaign action; the older unconditional
continuation instruction is superseded. Do not reopen implemented mechanisms from historical prose.

## Outcome (recorded after implementation)

### What was built

**Implemented:** selected kernel inputs, actual persisted semantic dependencies, portable
strict reconstruction, producer capture/controlled qualification, compact preparation and
consumer migration. Global `SOURCE_IDENTITY`/`BUILD_IDENTITY` keys were removed from compiler
and mathematical reuse; complete outer attestation is supplied only by composition roots.
Bundle ancestry ownership and implicit routine whole-source Arrow exports were deleted.
Worker/study payloads now identify actual canonical modeling revisions; required physical
context remains explicit. Modeling-knowledge SQL convenience was removed in Rust/Python,
leaving typed projections and Arrow. Full Runtime/TableReader result-query migration is D.
Unchanged generator outputs preserve their source timestamps.

**Tested**, baseline zero, with explicit force-validation and linked native environment:
compiler scientific/replay controls passed 33/33; final authority-binding subset passed 2/2;
producer controls passed 17/17 including actual Cargo consumed-helper rekey, uncompiled
inline/file test locality and source-bound Cargo configuration executors. Configuration
qualification binds raw configuration, resolved executable bytes, declared inputs and
before/after capture; unknown, stale or incomplete closure refuses reuse. Canonical
portable controls passed 12/12 using an actual
eligible finite tool receipt, publication, client/service drop, reconnect and strict
ordinary scalar solve `9 -> 2`, plus refusal of corrupt qualified payloads and typed
callback deadline/cancellation controls. Neutral authority passed its pure unit and
`just scientific-replay-miri` (1 selected pure control, no native FFI). Native compact
implicit controls passed 26/26, including sparse high-ordinal formals and different
hint/nominal signatures. Selected-source controls passed 4/4; source/ownership regression
selection passed 31/31, including repeated revision edits and escaped shared payload leases.
The large portable control persists a compiler-issued 4,773,984-byte recipe, reconnects
and strictly reconstructs it with independently expected residual 7 and derivative 1.
Protected candidate hydration refuses an inadequate pool before inspecting corrupt blocks.
The compiler's eight portable controls include logical byte preflight and strict
reconstruction above the former single-message bound.
EF02's final force-validated local controls passed through `just unit-package`: 22/22
selected `pse-relations` columnar/prepared controls, 26/26 selected `pse-engine` admission,
input, preparation, role and round controls, and 6/6 selected `pse-runtime` document
controls. Actual evaluation counters distinguish unchanged native-owner reuse from
changed UDF meaning under identical schema. Cloning typed rows clears their certificate;
sharing the immutable row owner retains it. Raw admission and builder completion retain
their checks. Changed-owner projection reserves scratch, cancellation precedes cached
reuse, and invalid replacement preserves the prior round value.

Final assembled workflows remain pending at the maintainer-requested checkpoint. A owns
the common closeout checks and the interrupted reference-only run's evidence boundary.
`just codegen-unit-test` passed all 16 generator and global physical inventory controls,
including unchanged-output timestamp preservation and independent physical fixture values.
The finite producer fixture establishes the qualification/replay mechanism under its named
scope; it does not qualify the real compiler/math/native deployment. No production producer
eligibility, full-series qualification or quantitative performance improvement is claimed.

### A mistake made and corrected

The first replay representation contained hashes without sufficient admitted mathematical
meaning. Selected syntax and concrete owning-kernel receipts now supply strict reconstruction.
Review exposed safe receipt/producer impersonation and missed dense implicit consumers;
opaque exact-record authority and audited mint/writer roles now fence replay, and consumers
gather the actual compact signature. Actual tool integration also found mismatched bare/prefixed
outer hash encoding; both producer tool and runtime now use the owning `ContentHash` wire type.
Full generation exposed missing global physical entity-kind projection after routine modeling
columns were removed; the physical boundary now requests only the declaration kinds it consumes.
Assembled workflows exposed tests that still consumed implicit source exports or edited an
old head repeatedly. They now request explicit exports and edit the current revision.
Scientific import, fixture and typing refusals run on their actual selected demands;
global data declaration prerequisites remain checked by the shared admission kernel.
Full reference cases exposed a portable recipe limit tied to one RPC, and study admission
obscured that refusal behind an unclassified diagnostic. Logical recipe limits are now
independent of block transport, and finite byte refusals retain their resource facts.
Large-product controls exercise actual compiler-issued reconstruction and protected storage
rather than bypassing persistence or reducing the scientific workload.
The final obligation check also found discarded native validation owners at authored
ingress and SQL-text-only projection reuse. Actual local owner retention and receiving
context readmission replace those assumptions. A negative fixture that privately mutated
an admitted immutable row object now exercises a genuine changed row owner; the original
invalid-value refusal remains required.

### Deviations from the plan, deliberate

Conservative raw consumed-source fallback resolves macro/location representation uncertainty;
it may rekey on inline text that a normalized representation could otherwise exclude.
Unreviewed executable I/O/native closure remains a refusal. This accepts extra invalidation,
not unsafe reuse. Minimal adjacent worker/study/Python and generator changes were authorized
for runnable A/B consumers; full C/D/E migrations and E4 measurements remain separately owned.
No unsupported SIMD/JIT/batch evaluator path or second persistent authority was introduced.
