---
title: Selected compilation and durable reuse
status: in-progress
date: 2026-10-05
adrs: [ADR-0164]
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
| B1 — Selected kernel boundary | R0 and implemented A2 selection/codec slice. Resolve selected physical/modeling inputs and share admission/inference/specialization kernels. Preserve required context and global obligations. | Migrate authored `.pse`, Rust and Python modeling entry points and workflow composition. Remove their complete-bundle hydration requirement and `BundleOwner::_parent` retention when the last caller moves. | Implemented; final verification in progress |
| B2 — Durable descriptions and dependencies | B1 plus A2 immutable storage/guards/protection. Persist portable products and complete dependencies; implement ordinary-solve reconstruction, discovery, eligibility and invalidation. | C2 consumes working reconstruction; move Salsa to optional acceleration and delete overlapping cache/validation mechanisms with no remaining consumer. | Implemented; final verification in progress |
| B3 — Provenance and numerical preparation | B2 and R0 identity decision. Separate outer attestation from product keys; reconstruct and share compatible immutable numerical products without retaining whole revisions. | Migrate artifact requests and value-only, dynamics, fitting and study preparation. Delete global build keys from scientific reuse and unconditional repeated preparation. | Implemented; final verification in progress |
| B4 — Formal coordinate preparation | B1 and exact library signature evidence. Decide compact mapping versus required complete signature, implement justified improvement and record any remaining limitation. | Migrate every consumer of the changed evaluator layout together. Remove displaced dense maps/tests; preserve actual guard/provider work. | Implemented; final verification in progress |

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

B1–B4 functional mechanisms are implemented. Checked batches and immutable typed rows
retain the actual native validation owner. Same-owner readmission preserves existing
evidence; changed owners and values receive native checks. Engine receiving boundaries
retain the receiving owner, and failed round replacement leaves the prior epoch unchanged.
Canonical resolution supplies the sufficient
lexical/import/member/inverse-supplier closure with presence and absence premises. Routine
modeling handles retain canonical revision, physical/provider context and attempt-local
factories; they do not retain a complete declaration workspace. Inventories and source
Arrow exports are explicit. The global physical entity universe has its own native
projection; selected modeling and physical meaning are not conflated.

Portable products retain selected syntax and complete owning-kernel construction receipts.
Strict reconstruction uses the ordinary numerical builder without fresh inference or proof
fallback. Opaque authority authenticates exact whole records to the qualified compiler-issued
payload and request. Scientific writer and deployment/replay qualification roles cross
explicit audited trust assertions. Decode scratch is reserved before parsing. Compact
formal mappings include guards/providers/nested scopes; all consumers gather the worker's
actual input formals, while derivative axes keep their requested order.

Portable recipes have a finite 64 MiB logical limit, with independently bounded envelope
metadata. Encoding counts the actual extent before allocation. Canonical product bodies
and dependencies travel through immutable blocks beneath the 4 MiB protocol limit.
Store-issued candidate extents support reservation before hydration; strict reconstruction
also reserves its owning-kernel decode allowance. These format limits confer no entitlement
to process memory: a smaller admitted pool refuses before allocation.

Relevant producer tooling now captures actual Cargo production units and emitted dep-info.
A genuine finite Rust tool capture qualifies and drives a persisted native reconstruction
control. The inspected real `pse-runtime` dev deployment remains **ineligible**: 153 concrete
refusals cover unreviewed build-script/procedural-macro I/O, ambiguous artifact associations,
Cargo configuration, active environment and native closure. This is an explicit qualification
boundary, not permission to manufacture a production receipt. Supplied source-bound reviews
and complete actual native/configuration inputs are required before that deployment can reuse
scientific products across builds. Normal admission and canonical persistence remain available.

The maintainer requested closeout and a committed checkpoint on 2026-10-06, then cessation
of work. Functional scope and focused controls are complete; final assembled A/B acceptance
is pending. No compile-time or solve-time gain is inferred from structural controls.

The later 2026-10-06 authorization supersedes the closeout rerun order: complete all
28a–28e functional work with targeted tests before E3 assembled qualification. Transfer
the applicable assertions of the former 84-case Python and 106-control Rust selections
to the migrated target. Preserve the original 1,000-point study and scientific demands.
Actual supported runtime, Python-extension and worker producer qualification proceeds
as B3 work; unknown or stale executable inputs continue to refuse persisted reuse.

The complete outer source inventory now includes the worker's `xtask` composition
root alongside library and vendored sources. Its actual filesystem control catches
dirty worker edits while excluding build outputs. Selected producer dependencies
continue to own scientific reuse eligibility; the outer inventory does not replace
the guarded native/configuration closure or actual linked-artifact association.

The supported capture uses the dedicated `producer` profile, inheriting `dev`, with
compiler wrappers disabled. Existing `dev` artifacts may have been produced by a
cache daemon whose startup configuration is unknown; a current environment cannot
retroactively qualify those effects. Fresh selected artifacts in the separate profile
provide actual compiler-unit association under the reviewed conditions without cleaning
or moving the checkout's Cargo target directory. The exact libclang file survives nested
setup. The Python capture binds the actual Maturin configuration file and linked feature
context, preserving the selected abi3 contract; PyO3 interpreter discovery is inactive
in this supported configuration.
The capture's feature set must match each actual linked consumer. Reached effectful
executors are refreshed by Cargo within the selected profile before a new arbitrary-I/O
review can qualify them. Selected native namespaces retain candidate additions, topology
and file contents; immutable snapshots associate actual build outputs and child context.
The supported native context fixes CMake build metadata with `SOURCE_DATE_EPOCH=0` and
`TZ=UTC` and uses the pinned toolchain's explicit Rustfmt. Target-specific `-MD` flags
retain consumed native dependencies under reviewed tool and search-namespace guards;
`CC_ENABLE_DEBUG_OUTPUT=0` excludes recovered compiler-probe logging while ordinary
compiler errors remain visible. The capture tool is built separately and launched
directly through one native setup, preserving the ordinary caller context instead of
Cargo-run's tool-local loader paths. This deployment choice does not itself grant eligibility:
selected executable helpers and native inputs still require source-bound reviews before
ordinary persisted replay. Focused capture controls do not substitute for that production
qualification.

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
