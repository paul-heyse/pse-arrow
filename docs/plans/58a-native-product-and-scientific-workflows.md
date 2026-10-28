---
title: Native product and scientific workflows
status: draft
date: 2026-10-28
adrs: []
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [current-work, architecture, libraries, validation]
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md]
---

# Native product and scientific workflows

This supporting plan develops the scientific application target of [Plan58](58-rust-owned-system.md). It is a new decomposition of the
Rust-owned target, superseding Plan57's delivery scope rather than copying its packages. All packages are planned; authoring this
document authorizes no implementation. The source [functional
review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md) supplies O01, O10 and O12, with a bounded O13
decision. Its observations are planning inputs, not evidence that a native application journey has already passed.

The [hard-cutover review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I01),
published at `c3b7f136d`, additionally supplies I01/I02/I07/I09/I10/I13/I14. The coordinator owns
their dispositions. This amendment retains scientific outcomes while removing unnecessary Python delivery
requirements; no package inherits unfinished Plan57 work.

## Target and foundation assessment

The product root is a callable Rust library: admitted authored definitions and explicit physical context enter existing
authoring/compiler/runtime owners; typed prepared operations execute through the original native supervisor and return immutable results
or an owned operation handle. Ordinary in-process calls do not serialize through JSON, an MCP catalogue, Python objects or a universal
dispatcher. Small executable commands may expose useful operations when inexpensive, but a product CLI is neither the architecture root
nor a completion requirement. A future frontend consumes the same capabilities without implementing scientific admission, workflow
policy or result interpretation.

Normal linked Rust APIs are the immediate public capability. Do not first construct public pyclasses,
frozen Python document wrappers, a replacement codec or a foreign ABI. Explicit build/development generation
may publish declarations needed by retained consumers; ordinary execution, reads and help do not regenerate
code. Validate external files/requests for finiteness, versions, unknown fields and physical context at their
actual ingress, then carry established premises through Rust construction and borrowing.

Current foundations include `pse-runtime/src/workflow/mod.rs`, `operation_preparation.rs`, `sampling.rs`, `strategies.rs`,
`result_snapshot.rs` and `pse-compiler/src/workspace.rs`. `RuntimeOperationRequest`, `ModelingPackage::prepare_operation`,
`PreparedOperation::start` and `RunHandle` already connect shared native capabilities. Existing `pse-project` and `pse-worker` are
binaries under `crates/pse-runtime/src/bin/`; their existence does not establish native caller coverage of every supported workflow.
Python wrappers in `crates/pse-py/src/workflow/` identify retained user outcomes to migrate, rather than an object graph to recreate.

The focused assessment preserves blueprint §14.1/§14.4 preparation and §18 scientific execution: Salsa tracks semantic dependencies;
native compilation is an explicit effect outside queries; immutable plans/programs are reusable while mutable evaluators, numerical
factors and solver state belong to an attempt. Prepared operations and durable study descriptors have deliberately different
eligibility: host-only shooting must not acquire a persisted projection simply to make APIs uniform. Native callers need clear
construction and composition, not another scientific engine.

Keep maintained Symbolica 3.0.1 and faer 0.24.4 realizations and their vendor deltas. Symbolica multi-output evaluation already shares
expression work; faer separates sparse symbolic structure, numeric factors and explicit scratch. Reuse structure only under compatible
sparsity and numeric factors only under unchanged coefficients. Borrow native views where lifetimes fit; avoid frontend-shaped arrays
between native stages. No new JIT/SIMD/CUDA, exact-root, interval, partial spectrum or matrix-free capability is required by migration.
Sources are the review's O10 and mechanism table, `pse-math/src/{library,implicit,native_compiled}.rs`, and the existing vendor notes.

Core 3.4, heuristics 1.0, test assessment 1.2 and process-simulator 1.5 govern this target. The coordinator's
[T01](58-rust-owned-system.md#t01) and [T02](58-rust-owned-system.md#t02) own early shared contracts and the ADR/formal-review route
before contract-changing code, including changes to D12/blueprint §21. This document proposes no independent rule acceptance. Optional
PyO3 implementation is deferred; its absence cannot block native scientific completion.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [A01](#a01) Native application contract | planned | Native workflow owner | Accepted shared contract and complete retained scientific caller scope |
| [A02](#a02) Incremental workspace and source workflows | planned | Authoring/compiler owner | Native revision/edit/reuse behavior and source/history consumers |
| [A03](#a03) Prepared local execution and results | planned | Runtime/math owner | Typed author→prepare→execute→inspect composition independent of Python |
| [A04](#a04) Complete scientific mode callers | planned | Scientific workflow owners | Every supported mode constructible and failures honestly scoped |
| [A05](#a05) Native studies and replay | planned | Study/runtime owner | Generated studies, durable handoff and recorded-point replay |
| [A06](#a06) Adaptive study decision and retained strategies | planned | Study strategy owner | Bounded comparison decided; retained native strategies migrated |
| [A07](#a07) Consumer cutover and assurance handoff | planned | Native workflow integrator | Replaced consumers deleted and required H journeys ready |

<a id="a01"></a>
## A01 — Establish the native application contract

Start after T01's shared-contract decision and T02's consumer scope are available. Inspect native producers, Python scientific entry
points and representative users to settle the smallest complete Rust caller surface for admission, revision/edit, preparation, local
run, study, diagnostics and selective reads. Record useful operations beside their existing owners, with actual input/context, output
ownership, effects, unsupported cases and cancellation policy; no catalogue framework or operation registry is an implementation
deliverable. Expose missing construction operations at those owners rather than teaching callers to reconstruct private layouts. Keep
pure/local work store-free where supported. The decision exits with agreed callable boundaries and consumer responsibilities, including
the host-only/durable distinction. Required ADR/formal review precedes incompatible boundary changes. Targeted acceptance exercises
native request construction and refusal without Python or unrelated storage/solver setup; remove superseded private construction paths
with their migrated callers.

<a id="a02"></a>
## A02 — Make the incremental workspace independently useful

Consume A01's source/revision contract and extend existing `CompilerWorkspace` inputs and tracked derivations, not a second workspace
cache. Native users admit a package, edit values or structure, prepare demanded products and inspect dependencies/diagnostics tied to
authored identities. Value edits preserve unaffected structural products; membership, deletion, absent lookup, provider,
physical-context and interpretation changes invalidate everything they can affect. A new unit using existing physics extends
declarations; adding a physical concept changes its semantic owner. Use existing graph projections for explanations; SCCs and
deterministic traversal must not confuse physical recycles with acyclic dependency order or persist local graph indices as identities.
A02 owns migration of native source/project/history callers through the existing runtime/project and
operations revision/publication owners. D supplies connection/authentication and canonical transition policy
where persistence is selected; A owns the scientific source/submission callers and their preparation.
Salsa persistence is deferred pending a concrete consumer: if selected later, compatibility/input namespaces and safe
recomputation are mandatory. Targeted compiler tests distinguish value-only, structural and deletion edits, concurrent read/write drain
and unavailable-cache recomputation. Delete Python-only workspace caching after caller migration.

<a id="a03"></a>
## A03 — Compose preparation, local execution and inspection

Consume A01 and A02's usable preparation slice, plus [C02](58c-runtime-lifecycles.md#c02)'s implemented start/stop/join contract. Use
`PreparationServices`, `ExecutionServices`, complete source/context identity, admitted bindings and `PreparedOperation` to execute a
local scientific operation without Python or a database when the operation is local. Share immutable evaluator/structural products
across compatible requests; create fresh controls, mutable evaluators, factors and provider workers for each attempt. Preserve current
physical/domain/derivative admission and independent post-solve checks; library success is not a verified scientific result. Expose
original typed diagnostics and partial/refused outcomes. Consume the results owner's protected snapshots/selective readers and explicit
retention rather than materialize all results into a facade. Returned handles preserve output lifetime independently of preparation
lookup eviction. Targeted runtime tests cover a local successful case, invalid physical context, unsupported route, canceled observer
and retained result after producer closure. Remove native-to-Python-to-native assembly and copied wrapper policy as each actual consumer
moves.

<a id="a04"></a>
## A04 — Preserve all supported scientific modes through native callers

After A03, migrate square/root, optimization, initialization/recycles, dynamics/events, fitting, sensitivity, generated-design/study and
diagnostic callers, retaining existing class-specific scope. Include current horizon/shooting and strategy operations where supported;
do not widen their durable or solver eligibility by implication. Inputs retain explicit starts, bounds, scaling, physical envelopes,
derivative order and source provenance. Numerical iteration/globalization remains with qualified existing libraries. Shared model
definitions feed modes rather than parallel formulations. For each moved mode, pair a native successful call with a revealing
failure/partial case and retain the independent scientific expectation or oracle selected by its existing assessment owner. Targeted
tests run at the scientific owner; full multi-mode journeys wait for H. Remove wrapper-only tests and fixtures only after the retained
scientific claim has a native test or explicit H owner. No permission to lose peripheral interfaces authorizes loss of one of these
scientific modes.

Coordinate F04's independent-reference cutover: the existing parity candidate harness imports `pse.Runtime`
and generated Python types. Native candidate operations must exchange explicit cases/results with isolated
IDAES/Pyomo, SciPy or teqp references instead. A public CLI or a required Python extension is unnecessary.
Retain original tolerances and physical interpretation; a deliberately wrong candidate must fail comparison.

<a id="a05"></a>
## A05 — Compose native designs, studies and reproducible results

Consume A03/A04, [D01](58d-canonical-control-and-connectivity.md#d01)'s connection/authentication,
[D03](58d-canonical-control-and-connectivity.md#d03)'s canonical execution policy and
[C03](58c-runtime-lifecycles.md#c03)'s worker admission contracts. A05 owns native recording/submission
callers and scientific preparation; D owns persisted submission, readiness, claim and settlement meaning.
Native generated designs produce owned realized overlays; local studies remain local until explicit recording.
Durable submission carries exact original source/operation identity and uses the same canonical definitions,
dependencies, attempts, assignments and result references later queried for analysis. Workers perform effects
without maintaining a second durable interpretation of readiness. D02 supplies original command reconciliation. Keep
recorded-realized-point replay, warm-start dependencies, actual failures, original result provenance and source/history retention.
Cancellation and restart must not reinterpret an imputed value as an observation or regenerate realized points from a seed. Reuse shared
prepared structure across cases while respecting nested native admission and attempt isolation. Preparation and
required resource admission precede execution claim; D must recheck prepared seed evidence, dependency revisions
and cancellation before granting that claim. Expensive preparation/native science stays outside short database
transactions. Install dependency edges before activation; this scope does not introduce live graph mutation.
Notifications only wake workers; canonical state resolves missed events. Retain useful calculation/progress
observations at scientific granularity, not every runtime event or live object graph. Targeted tests cover deterministic point
identity under reordered completion, generation failure, recorded replay after producer closure and interrupted
publication/reconciliation, cancellation or dependency revision changes after preparation, and worker replacement followed
by analysis of the original lineage. Delete Python study assembly and redundant conversion with migrated consumers.

<a id="a06"></a>
## A06 — Decide adaptive composition without inventing another optimizer

Preserve current native adaptive strategies and their thread-affine `EgorSolver`/`EgorState` owner. Before any new ask-and-tell surface,
compare that realization with pinned EGObox 0.38.1 `EgorServiceFactory`/`EgorServiceApi::suggest` against actual
suggest→evaluate→observe→resume needs. Research exits with one bounded choice: reuse the actor; adopt service ask-and-tell for a named
consumer; or defer the extension with that consumer/trigger recorded. Check exact release APIs, state serialization/reconstruction,
failure/imputation semantics, batching and thread/lifetime requirements. A source/API check suffices for rejection; unresolved execution
premises require a small targeted experiment during authorized implementation, not an open-ended benchmark campaign. Do not mix newer
overview APIs into current ego/moe/gp/doe 0.38.1/0.35.5/0.36.4/0.35.1, ndarray 0.16.1 or rand_xoshiro 0.6.0. Stable study/point identity
assigns random streams. If adopted, the same package migrates the concrete consumer, tests truthful observations and resume/cancel
behavior, and deletes its replaced optimization loop. Deferral does not remove A05.

<a id="a07"></a>
## A07 — Complete native consumer cutover and hand off qualification

Finish with every retained scientific workflow callable through Rust APIs while Python is absent. Migrate actual documentation/examples,
worker roots and non-Python consumers alongside each owner; retain only optional binding obligations explicitly selected elsewhere.
Coordinate deletion of the retiring Python facade/MCP consumers with their owning plan; never remove a shared guarantee merely because
its old wrapper disappeared. A future frontend adaptation must consume these operations without copying their scientific policy. Product
command additions remain discretionary.

Use integrated cutover sets: owner, real consumers, configuration, generator declarations, targeted tests and
obsolete implementation deletion belong together. Temporary development incompleteness earns no acceptance;
do not add compatibility DTOs, a FastMCP proxy or a Python scientific fallback to keep intermediate commits usable.
Retire Python projections, stubs, manifests and self-check expectations with their last retained consumer in F/G.
Keep genuine external scientific contracts, not mechanism-specific drift guards.

## Verification and current checkpoint

**Proposed:** use `just check-package pse-compiler`/`pse-runtime` and narrowly selected `just unit-package <pkg> <filter>` for changed
behavior, with explicit force-validation on actual Arrow correctness closures. Package descriptions specify discriminating cases, not
executed tests. [H](58h-qualification-and-publication.md) owns assembled native local/durable scientific journeys, independent
oracle/reference selection, scope-end integration/solver/component checks, `just hygiene` and manual governance/docs/native/powerset
checks. Measurements need a separate decision purpose; no speedup is claimed here. All implementation and qualification are **not_run**;
no production code, tests or builds changed. Completion requires the coordinator's combined acceptance and truthful zero-baseline
reporting, not this document's package count or a local targeted pass.
