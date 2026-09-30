---
title: "25f: Studies, diagnostics and continuation"
status: draft
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s10]
---

# 25f: Studies, diagnostics and continuation

## Context and target

F05–F08/F17 and FU09 identify competing study policies, loss of typed failure detail, untyped
bindings and confusion between reusable content and experiment identity. R6/R7 retain the
distinctions between scientific usability, seed eligibility, operational completion and retry.
This plan also supplies the semantic vocabulary consumed by J3/F24 and consumes E5's route facts.

One study definition is executed in process or durably with the same scientific decisions.
Its pure point policy consumes typed facts; executors own effects, locks, fencing and retries.
Detailed diagnostic identity survives all boundaries even when no scientific result exists.

## Decisions and interfaces

**Failures.** Typed cause/code is authoritative. FailureClass is a coarse projection; boundary
disposition, severity and retry permission are purpose-specific projections. Preserve structured
locations, observations and causal history. Split request admission, internal invariant and
typed-source errors rather than routing them through Contract prose. Human messages explain
the structured facts and never replace them. Numerical/qualification rule names become admitted
closed vocabulary, including authored diagnose requests.

**Retry.** Operation and effect state matter. Retry only a declared transient failure when the
effect is known absent or a proven idempotent/reconciliation contract permits it. Unknown
publication effect first reconciles; deterministic encoding/invariant defects do not retry.
Initialization consumes its declared scientific attempt policy and exhaustive stop projections.
No default wildcard treats a new stop as trial rejection.

**Bindings.** Resolve each supplied target path/member reference against the selected immutable
revision, check target role and complete physical contract with A, compose once, then persist/hash
member identity and canonical typed value. Reject conflicting duplicate assignments; never use
last-write-wins. Paths and supplied units may remain attribution. Explicit-ID renames survive
only when target meaning stays compatible; named-policy renames are new entities.

**Study occurrences.** A point is an occurrence with its own admitted run operation, dependencies,
attempts and results. Equal bindings may share preparation, never occurrence identity.
Occurrence-level idempotency preserves one submitted point across retries. Ordering, usable-result
dependency and seed continuation are distinct edge meanings.

**Dependency defaults.** Independent points need no predecessor. A continuation defaults to
requiring a usable compatible predecessor seed; missing/incompatible seed refuses the dependent.
AllowSeedOnly and FreshOnUnavailable are explicit recorded choices. An explicitly supplied seed
that fails compatibility refuses rather than falling back. Fresh fallback applies only to declared
absence/incompatibility, not arbitrary internal errors. Ordering-only edges wait for terminal
completion, including a failed terminal predecessor, without claiming scientific success.
Study cancellation still prevents new dispatch. A failed usable-result dependency yields an
attributable dependency refusal; cancellation is represented separately.
Seed need is also explicit: a constant/seed-free operation reports NotNeeded rather than an
unavailable seed. Preserve its declared ordering or predecessor-success condition, but do not
refuse it for lacking a seed it cannot consume.

**Conclusion.** Preserve per-point scientific and operational facts. All required points usable
permits complete study success; failed/refused points with useful results elsewhere yield partial
scientific availability, not unconditional success. Cancellation remains explicit while available
members remain inspectable. E owns each run's usability, including multi-result requirements; this
plan does not promote a partially available run by counting tables.

**Generalization.** Points select supported case solves, simulations, fits and horizons through
the existing operation owners. A typed admitted run descriptor and its immutable inputs are the
shared contract. Do not persist arbitrary closures or invent a second fitting/simulation engine.
Unsupported operation/dependency combinations refuse before scheduling.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="f1"></a>F1 Detailed failures and projections | Existing diagnostic/operation owners | Establish typed failure envelopes, closed rules and exhaustive projections | planned |
| <a id="f2"></a>F2 Contextual typed bindings | A1; I1; H1 | Resolve and physically admit overlays once, with deterministic composition | planned |
| <a id="f3"></a>F3 Pure occurrence policy | F1/F2; E2/E3/E4 | One study definition and pure dependency/start/conclusion operation | planned |
| <a id="f4"></a>F4 Executor and durable cutover | F3; G3 | Both executors apply the same policy; retain every terminal typed failure and occurrence | planned |
| <a id="f5"></a>F5 General run points and provenance | F4; E5 | Expose admitted solve/simulation/fit/horizon points and consistent route/result projections | planned |

F1 is an early contract, independent of the eventual study engine and store migration. Its
persistent vocabulary consumers wait for G3. F2 likewise admits bindings before durable storage.

### F1 — Preserve detail at each boundary

**Implementation vision.** The operation emits a diagnostic envelope containing detailed code, typed rule and stage,
coarse class, boundary disposition, severity, affected semantic identities, revision-bound locations,
typed observations and cause links. An attempt adds occurrence/attempt identity and causal
predecessor without rewriting the original failure. Observation values distinguish missing
evidence, finite physical values and explicitly tagged nonfinite numerical observations; an
infinity observed during failure is not admitted as an ordinary physical value. The envelope
exists before a model/result table does. Decode/admission validates vocabulary and observation
shape. A unit mismatch consequently retains both operand contracts and locations through Rust,
Python and storage, while retry consumes its separate operation/effect policy.

Migrate error families at their source rather than classifying flattened text in a central
downcast chain. Preserve source errors where useful; project stable code, observations and
locations through the owned diagnostic interface. Declare member descriptions and typed
qualification/numerical rules. F17's result predicate delegates to E's single final decision;
retry and severity remain different exhaustive operations.

Focused controls distinguish syntax, unknown ID, units, numerical evaluation, resource limit,
cancellation, panic and internal invariants. Test persistence failures with known-absent versus
unresolved effects, and reject misspelled authored rules at admission. Delete consumed string
comparisons, blanket class-to-code reconstruction, wildcard partitions and stringify-only source
conversions. J3 owns the common strum-based mechanics; semantic projections remain owned here.

### F2 — Admit overlays

**Implementation vision.** Admission takes the immutable selected revision, authored case defaults and submitted target/
quantity assignments. It resolves references, rejects unknown/derived/unwritable targets,
normalizes aliases to member identities, checks duplicate submitted assignments and role/physical
compatibility, then invokes A1 conversion. The authorized overlay replaces a case default once;
that is different from contradictory duplicate assignments within the overlay. Output entries
contain member identity, expected target/context identity and canonical value, with original
path/unit retained as attribution. I frames this admitted map, not JSON spelling. Both workers
receive it without resolving paths or converting units again; parameter bindings follow the
same rule.

Use A's checked conversion and complete target contract before deriving the binding identity.
The compiler/runtime binding consumed by each executor is the same admitted representation,
not a second conversion performed by a worker. Parameter bindings also carry physical meaning;
canonical scalar storage internally does not make unitless public input acceptable.

Focused controls include 80 °C → 353.15 K, equivalent pressure units, wrong basis/datum/subject,
duplicate assignment, incompatible revision and compatible explicit-ID rename. Delete persisted
path-keyed bare-number overlays and executor-local composition. J generates the new boundary;
G migrates durable definitions explicitly.

### F3 — Pure study decisions

**Implementation vision.** The immutable definition contains format version, pinned source/context references and ordered
point occurrences. Each point has an occurrence key, operation-owned admitted descriptor,
canonical binding reference, ordering/usable-result dependencies, at most one selected seed
source/result role, seed-only/fallback choices and applicable attempt policy. Admission verifies
unique keys, dependency references/acyclicity and compatible output/seed roles. Scientific facts
(E's decision, result availability and seed capability) remain separate from attempt facts
(lifecycle, cancellation, attempts and effect state). The transition returns Wait, Start, Refuse,
Cancel or Reconcile actions, chosen start provenance and conclusion. A seed-only predecessor
can satisfy an explicitly permissive seed edge but never a usable-result edge.

The pure transition takes point states, scientific decisions, compatible seed facts, edge intent
and cancellation; it returns actions and conclusions. It performs no database, artifact or native
solver work. Both executor adapters supply the same facts and apply the same decisions.
A fit/horizon may expose multiple results; its owner supplies explicit aggregate usability and
selected seed compatibility, never an arbitrary first result.

Focused fake-result matrices cover usable, seed-only, constant evaluation, partial multi-result,
missing/incompatible seed, predecessor refusal, cancellation and explicit fallback. Repeated
bindings from different starts and independent repetitions remain distinct. Delete duplicate
in-process/durable policy definitions after both consumers switch.

### F4 — Durable failures and occurrence identity

**Implementation vision.** An action carries the occurrence and expected state revision from which it was derived. Under
its existing transaction/locks the durable adapter rereads state, validates or recomputes the
pure decision, and applies lifecycle/scheduling effects atomically. It resolves the chosen seed
artifact without reinterpreting policy. Attempt identity is distinct from occurrence identity,
so retry does not duplicate the requested experiment. Terminal records retain the scientific
decision when available, member references, diagnostics, actual start and effect state. Unknown
publication effect schedules reconciliation before retry. The common outcome projection joins
these facts while preserving their distinct meanings.

Remove binding-hash uniqueness while preserving point occurrence identity and retry idempotency.
Apply pure actions under existing locks/leases/fencing; a prepared decision is not authority to
ignore a changed durable state. Persist a typed terminal-attempt envelope even for pre-admission
failure, with causal attempt history. Scientific members attach only if available. Never fabricate
a model/result table to make diagnostic persistence possible.

Use one outcome relation/projection for both executors, with operational state explicitly separate.
Focused policy/codec controls cover retries, repeated occurrence submission, typed refusals and
round-trip detail. Real PostgreSQL, restart and publication journeys execute in K3. Delete
prose-only semantic failure fields, duplicate outcome relations and content-based point uniqueness.

### F5 — General operations and explanations

**Implementation vision.** Each operation owner supplies a descriptor containing kind, authored selection, immutable inputs,
admitted settings, expected output roles and seed input/output capabilities. A fit can expose
parameter estimates and profiles; a simulation can expose a trajectory. Neither becomes an
arbitrary first table. Study admission rejects feeding a trajectory output into an incompatible
solve-seed slot before scheduling. Dispatch invokes the existing operation executor with the
descriptor/binding; E returns aggregate scientific usability and route facts, and F attaches
occurrence, attempts and selected start history. This supports mixed admitted run kinds without
a second study-specific execution pipeline.

Migrate points from case-only payloads to operation-owned admitted descriptors, including the
inputs needed to reconstruct the request under its pinned source/context. Reuse existing run
admission and resource supervision. Link E5 route intent, automatic/explicit selection, derived
classes and attributable refusal to each attempt and result. Preserve selected starts and actual
seed/fallback reason in lineage.

Focused controls admit each supported run kind and refuse incompatible seed/result dependencies.
Verify partial multi-result projections and identical policy between executor adapters. Full
mixed-kind durable journeys execute once in 25k. Remove caller copies of defaults, usable
predicates and route interpretation; do not create study-specific execution engines.

## Authority and handoff

Update blueprint §19.3 and §23.2, with the diagnostic/boundary decision route where contracts
change. F4's schema transition requires G3 and the ADR-0114 evolution decision. Python-boundary
changes follow J's route. Existing native stop/candidate facts are not collapsed into job status.

The public output is a shared study definition, admitted binding, diagnostic envelope and
purpose-specific projections. E owns scientific truth, G durable interpretation, I hash/resource
identity and J generation. This document does not duplicate their authorities.

## Execution and evidence

All changes and expected benefits here are **Proposed**. Packet status is planning state;
no implementation or new product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Not implemented; record actual behavior and evidence labels at closure.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
