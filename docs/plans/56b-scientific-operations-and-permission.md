---
title: "Plan 56b: Scientific operations and permission"
status: in-progress
date: 2026-10-25
adrs: [ADR-0227]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, numerical-execution, validation]
---

# Plan 56b: Scientific operations and permission

## Purpose and assessed foundations

**Proposed:** completion-owned scientific permission governs every aggregate, derived product
and public/recovered consumer; each analysis kind owns its lifecycle facts. This companion
addresses DA-009/017/021/038/039. [Plan 56](56-design-principle-alignment.md) owns scope,
disposition and decisions. This is plan-authoring design advice, not a new broad audit or a
scientific qualification.

Reuse the pure `workflow/numerics.rs` decisions, shared `RunSupervisor`, typed `RunOutcome`,
immutable preparations, original assessment and class-specific native adapters. Keep runtime's
scientific-workflow responsibility under ADR-0175. The present weaknesses are consumers deciding
again and broad internal reach, not proof that all science needs a new crate or policy engine.

## Target: owned facts with distinct scopes

| Fact | Owning meaning |
|---|---|
| Candidate permission | `numerics::{native_use,original_completion,complete}` decides result/seed use from actual evidence and policy |
| Original aggregate permission | Completion owns requested coverage, successful original report and complete nonempty required assessments |
| Product-local mathematical validity | Fitting/local analysis owns rank, approximation, stationarity/statistical prerequisites and applicable PSD evidence |
| Product permission | Governing original permission and the required local validity, retaining reasons and qualifiers |
| Estimate qualification | Existing `estimate_qualified` keeps its separate stationary/identifiable point-estimate meaning |
| Publication/recovery | Durable owners state committed/retained/unresolved effects; they do not grant scientific permission |

Select the conservative existing invocation/`RunResult::usable` rule. A failed original fit may
retain computed covariance/interval bytes for diagnosis, but cannot expose them as qualified fit
inputs or publish a contradictory usable-product claim. Do not relabel the bytes as external
covariance, equate covariance validity with estimate qualification, or invent conditional/partial
scientific reuse. A horizon publication failure retains authentic prefix evidence but does not
promote the whole requested operation through a publication-failure exception.

Completion records carry the governing scope, decisions, reasons and qualifiers, not a new bool
for each consumer to interpret. Version `Completion` and its persisted projections together.
Local results, studies, invocation, Python, result inspection and canonical hydration copy those
facts. Current DA-010 content reconciliation, external provenance, PSD, coordinate compatibility,
allocation and cancellation protections remain necessary independent checks.

### Operation ownership and local reasoning

Keep a closed analysis enum and typed request/report pairing. Each kind supplies request facts
(sources, identity, context/provenance, numerics and tagged replay constructor) and completed
facts/products (permission, native report/trace, start availability, retained extent and projection
inputs). One central exhaustive dispatch invokes the per-kind implementations. Lifecycle readers
do not maintain a new match per question. A plugin hierarchy or universal analysis language is
unselected; ordinary structs, enum dispatch and domain functions are sufficient.

Persist one tagged replay constructor instead of parallel optional constructor fields. A modeling
sequence may own a vector in its variant. Put horizon/modeling-specific payloads with their
outcome variants; eliminate omission wildcards. Preserve host-only shooting versus the persisted
study subset and compact/full horizon distinctions.

Narrow internal workflow visibility by responsibility. Immutable source, physical and prepared
science values cease carrying the whole effectful runtime merely for convenience. Execution
facades supply services when an effect is needed, with explicit lifetime ownership. Remove the
modeling-canonical/fitting declaration cycle by placing common declaration projections with
modeling. This is an incremental responsibility split, not a wholesale runtime rewrite.

### Capability and failure contracts

Publish default-settings capability plus a configured-capability query from the same resolved
adapter admission. Attribute dynamics from the actual method/backend; remove the non-IDAS
wildcard. Recovery-profile availability comes from the adapter seam rather than `Backend::Pounce`
checks. KINSOL retains its accepted fixed-point role. Direct Python settings should remove
always-refused PETSc composed options rather than adding a new workflow just to keep their names.
Settings-sensitive Clarabel/SCIP thread refinements remain explicit.

Construct failures by meaning: request/scientific refusal, unsupported, resource/effort,
cancelled, integrity/infrastructure and internal invariant. Add a math internal representation;
do not mechanically rename all `contract()` calls. Lazy admission may persist deterministic
scientific refusals, but transient resource/time/internal failures must not become permanent
scientific rejection. Effect knowledge controls whether retry is safe; class alone cannot prove
no effect. Diagnostics retain source identities and native class/stage/rule; F consumes them for
redacted remedies and appropriate telemetry severity.

## Packages

### B01 — Governing aggregate and product permission

Extend the existing completion owner and serialize its facts/reasons through C03/C05. Migrate
`RunResult::{usable,invocation_evidence}`, `fitting/results.rs::uncertainty` including own-prediction,
local `parameter_covariance`, persisted `fit_covariance`/reconciliation, study facts, Python
`NativeRunHandle::usable`, horizon completion and result inspection. Both qualified-fit
constructors require product permission; retained diagnostic payload does not itself supply it.
Remove duplicated aggregate conjunctions and `Unknown` used where the producer's actual permission
is available. Missing historical/current permission refuses explicitly; no success is inferred.

### B02 — Route evidence and composite decisions

Make residual LM and NLP routes supply a small common native evidence view to the shared
completion policy; do not fabricate a KKT report for LM. The same incumbent policy applies to the
same evidence, without claiming limited feasible incumbents are stationary or statistically valid.
Extract profile-pin acceptance and horizon move authorization into pure functions. Pins may
strengthen stationarity, but declare their original-check obligations and preserve applicability
qualifiers through interval ends. Keep numerical iteration library-owned.

### B03 — Kind-owned request/completion projections and tagged replay

Migrate `durable.rs::{identities,request_inputs,request_context,request_provenance}`, replay,
attempt documents, start availability, diagnostics, study preparation/binding and Python accessors.
Share declared-procedure dispatch and predicted-start composition where the operation is identical.
Delete superseded projections, optional-constructor conventions and `_ => None` omissions together
with their callers/fixtures. A newly supported kind supplies its owned facts at the dispatch seam.

### B04 — Narrow scientific value/service boundaries

Use B03's consumed interfaces to separate source/prepared state from execution services. Move
fitting declaration projection out of the reverse modeling dependency and narrow visibility of
kind, durability, result and formulation internals. Migrate one coherent owner/consumer slice at a
time; keep existing native, store and cancellation lifetimes explicit. The required value slices
are `ModelingPackage`, `FitProblem` and prepared source/model values: retain immutable model,
physical and source context while passing explicit execution/persistence services at effectful
entry points instead of reaching through `prepared.source.runtime`. Separate project merge/
preparation from modeling semantic internals; kind, durability, project, result and formulation
owners expose narrow projections and restrict visibility to their actual consumers. Formulation
consumes A02 structural facts and explicit observed values; it does not classify targets inside
provider observation. Preserve the public workflow facade where useful without retaining its
whole composition root in scientific values. Completion/composite/formulation decisions must be
exercisable without starting a store or solver. No replacement composition root or crate is added.

### B05 — Resolved capability publication and attribution

Migrate static/configured inventory, backend attribution and recovery-profile consumers. Remove
unreachable direct boundary choices and regenerate their contracts. Keep exhaustive adapter
selection and refusal before effects. No solver replacement or altered numerical algorithm is
part of this package.

### B06 — Typed failures through admission and recovery

Migrate constructor families by owning operation, prioritizing lazy admission, result-inspection
integrity, poisoned providers and missing-input attribution; then cover remaining math/workflow
families within the affected supported operations. Couple native cause construction with C02's
effect knowledge and F03's presentation. Equivalent conditions receive one class across paths;
ordinary observation events do not masquerade as registered diagnostic codes.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | implemented; review corrections and acceptance pending | 56b | Completion/product facts govern every local, persisted and public consumer; diagnostic bytes retained |
| B02 | implemented; profile endpoint acceptance pending | 56b | Shared evidence policy and explicit pure composite decisions; qualifiers survive |
| B03 | implemented projections/tagged replay; focused acceptance pending | 56b | Each kind supplies lifecycle facts once; tagged replay and exhaustive consumers replace old projections |
| B04 | implemented; binding/issuer guard qualification pending | 56b | Selected owner slices have narrow inputs/visibility and no reverse declaration cycle |
| B05 | focused native controls tested; public qualification pending | 56b | Default/configured capability agrees with admission and exact backend attribution |
| B06 | implemented; classification repair and composed qualification pending | 56b | Meaning-based failures drive scientific persistence, safe recovery and host diagnostics |

B01's pure decision can precede the canonical cutover; persisted completion requires C03/C05.
B03 provides early constructor/fact declarations to C before all lifecycle migrations finish.
B02 and B05 can proceed independently after their actual evidence contracts are ready. B04 uses
working projections, not merely an agreed interface. No whole-document barrier is implied.

**56a prerequisite — 2026-10-25:** the pure completion-owned governing conjunction now supplies
`RunResult::usable`; protected canonical result inspection reads the producer's existing Assessment
from the same admitted selection. Missing permission remains explicit; selected-product usability
keeps its separate meaning. This does not version Completion or complete the local/persisted
covariance and downstream permission migration. [56a](56a-model-meaning-and-physical-interpretation.md#verification-and-outcome--2026-10-25)
owns this slice's acceptance; DA-038 remains open in the coordinator.

## Revealing acceptance

**Planned:** extend `original_fit_failure_withholds_completed_statistical_products_in_invocation`
through result rows, local/persisted covariance and downstream propagation. Assert retained bytes,
agreed refusal and unchanged independent local validity/estimate meanings. Add successful and
external-covariance controls, LM/NLP policy/termination equivalence, missing and partial coverage,
callback errors, applicability-qualified profile pins and horizon publication failure.

Exercise every existing kind through the shared projections; malformed replay combinations refuse,
and host-only shooting never silently becomes a study operation. Compare local and recovered
completion facts. Construct the model/fit/prepared scientific values and exercise their pure
projections/decisions without runtime infrastructure; exercise their effectful execution through
the explicit service boundary as a separate control. Check project/result/formulation dependency
direction and restricted visibility, not a target count of files or `impl` blocks. Pure completion
tests alone cannot close DA-021. Capability controls include
KINSOL fixed-point, Clarabel/SCIP settings-sensitive threading and exact dynamics method identity.
Inject resource/time refusal during lazy admission, internal/integrity failures in inspection,
genuine missing initial conditions and equivalent poisoned-provider errors. Assert typed class,
affected sources, effect/recovery state and redacted remedy, not exception prose.

Targeted tests accompany each integrated change with explicit force-validation for Arrow closures.
G owns composed scientific/native/Python/operator qualification. No new numerical, runtime or speed
evidence is claimed by authoring this document.

## Execution checkpoint — 2026-10-25

The maintainer authorized all B01–B06 with only the working C/F prerequisites needed for complete
durable/public adoption. Qualify the new interpretation in a fresh owned deployment and preserve
the existing selected store. Scientific values use explicit narrow services at effectful Rust
entry points; existing Python/MCP facades retain convenience. Completion v2, stored completion v3
and replay v5 publish together; standalone fit-profile v2 identifies required chain/point qualifiers. Receipt framing and file placement stay unchanged. RC01–RC06
remain pending. ADR-0227 records the selected contract before publication. 56a evidence retains
its original scope; no B package is complete from that prerequisite alone.


The source cutover now records original scope and product permission in Completion, shares request/
completed lifecycle projections, and uses a single tagged replay constructor. Native adapters own
configured capability and replacement starts. Typed admission effects preserve acknowledged source
revisions and unresolved publication; host remedies consume those facts. These source changes are
under integration, with no B package completion claim.

B04 removes broad runtime retention from the selected modeling, fitting and prepared strategy values.
Explicit preparation, source, publication and execution capabilities preserve existing allocation,
validation, cancellation and selected durability behavior. Canonical source revisions retain their
original issuer as immutable provenance: a supplied target service cannot stand in for that fact.
Python effectful facades carry the explicit services supplied during preparation. Remaining work is
consumer/fixture migration, full generation, focused scientific execution and the selected composed
qualification against the fresh owned deployment. Earlier source-drift refusals executed no tests;
they provide no scientific acceptance evidence.


Independent implementation review of `8943cf557` identified two corrections before acceptance:
profile products must require actual complete endpoints rather than an outer successful diagnostic
collection, and explicit receiver services must authenticate retained source bindings and durable
issuer before effects. Both are being integrated with focused controls; no completion is inferred
from the original compile. The first focused runtime selection executed 29 tests (28 passed,
one internal-cause classification mismatch); the typed wrapper projection is being corrected.
The five focused native backend controls passed, including KINSOL execution. Final generation,
correctness extension, fresh owned canonical/Python journeys and scope-end checks follow the
corrected source baseline. Earlier qualification keeps its exact original scope.
