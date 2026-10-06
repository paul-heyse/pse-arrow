---
title: Selected compilation and durable reuse
status: draft
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_execution-efficiency-and-surrealdb_2026-10-05.md]
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
attestation remains complete; narrowing product keys is not permission to weaken it.

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

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| B1 — Selected kernel boundary | R0 and implemented A2 selection/codec slice. Resolve selected physical/modeling inputs and share admission/inference/specialization kernels. Preserve required context and global obligations. | Migrate authored `.pse`, Rust and Python modeling entry points and workflow composition. Remove their complete-bundle hydration requirement and `BundleOwner::_parent` retention when the last caller moves. | Scheduled |
| B2 — Durable descriptions and dependencies | B1 plus A2 immutable storage/guards/protection. Persist portable products and complete dependencies; implement ordinary-solve reconstruction, discovery, eligibility and invalidation. | C2 consumes working reconstruction; move Salsa to optional acceleration and delete overlapping cache/validation mechanisms with no remaining consumer. | Scheduled |
| B3 — Provenance and numerical preparation | B2 and R0 identity decision. Separate outer attestation from product keys; reconstruct and share compatible immutable numerical products without retaining whole revisions. | Migrate artifact requests and value-only, dynamics, fitting and study preparation. Delete global build keys from scientific reuse and unconditional repeated preparation. | Scheduled |
| B4 — Formal coordinate preparation | B1 and exact library signature evidence. Decide compact mapping versus required complete signature, implement justified improvement and record any remaining limitation. | Migrate every consumer of the changed evaluator layout together. Remove displaced dense maps/tests; preserve actual guard/provider work. | Scheduled |

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

No B package is implemented by plan authoring. The next executable work is R0, then A2's
selection/codec slice and B1. B4's signature investigation is independent of server deployment.
The coordinator owns US05/EF01/EF02/EF05/EF08 dispositions; this document owns B progress and
eventual local evidence.

## Outcome (recorded after implementation)

### What was built

Pending implementation and named evidence.

### A mistake made and corrected

Pending execution.

### Deviations from the plan, deliberate

Pending execution; consequential decision changes follow the decision route.
