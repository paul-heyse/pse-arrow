---
title: "25f: Studies, diagnostics and continuation"
status: done
date: 2026-09-30
adrs: [ADR-0148]
review_sources: [docs/design_review/reviews/design_review_studies-diagnostics-and-admitted-bindings_2026-10-01.md, docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
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
| <a id="f1"></a>F1 Detailed failures and projections | Existing diagnostic/operation owners | Establish typed failure envelopes, closed rules and exhaustive projections | complete |
| <a id="f2"></a>F2 Contextual typed bindings | A1; I1; H1 | Resolve and physically admit overlays once, with deterministic composition | complete |
| <a id="f3"></a>F3 Pure occurrence policy | F1/F2; E2/E3/E4 | One study definition and pure dependency/start/conclusion operation | complete |
| <a id="f4"></a>F4 Executor and durable cutover | F3; G3 | Both executors apply the same policy; retain every terminal typed failure and occurrence | complete |
| <a id="f5"></a>F5 General run points and provenance | F4; E5 | Expose admitted solve/simulation/fit/horizon points and consistent route/result projections | complete |

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

## Consumed 25c prerequisite slice

**Implemented/Tested, 2026-10-01; scoped focused verification recorded in [25c Verification](25c-process-composition-and-conservation.md#verification):** Conditional-unit admission uses a typed WorkflowError envelope with a closed modeling.conditional_unit.admission rule family, source identities and retained mathematical cause. The affected workflow diagnostic traversal migrates with it. At that checkpoint, general failure envelopes, projections, studies, bindings and retry policy remained open; they are completed below. The maintainer authorized that required slice and its complete affected consumer migration; [25c](25c-process-composition-and-conservation.md) owns the slice evidence.

## Consumed 25e prerequisite slice

**Implemented/Tested, 2026-10-01:** Required E-consumer projections now use typed candidate qualifiers/refusals and retained admission facts. Structural admission errors preserve their original native cause plus authored paths. Existing study/durable consumers retain authored routes and shared scientific permission; the wider F1 failure envelope and F3–F5 occurrence-policy redesign remained open at that checkpoint and are completed below.
[25e Verification](25e-declared-analyses-and-qualification.md#verification) owns commands,
conditions, composite results and limits; that earlier slice alone did not close this plan.

## Verification

**Tested, 2026-10-01:** Focused controls ran locally on Linux, using the pinned nightly,
locked dependencies and explicit `pse-relations/force-validate` through the recipes. Failure
baseline: zero. Commands below ran inside `direnv exec .`; native recipes supplied the linked
solvers, licensed environment and default 120 GiB memory cap. Nextest used its default profile
and concurrency; excluded tests were not exercised. These controls establish the named
mechanisms, not integrated durable restart or whole-product qualification.

| Command (inside `direnv exec .`) | Result against zero failures | Established scope |
|---|---|---|
| `just unit-package pse-model 'test(diagnostic::tests)'` | 5/5 passed | Authoritative codes, closed envelope/location/operand contracts, recursive causes, finite physical observations and tagged IEEE evidence |
| `just unit-package pse-operations 'test(study_policy_unit) \| test(study_codec_unit)'` | 18/18 passed | Shared dependency/start/conclusion matrices, scientific permission independent of operational state, retry/effect rules, versioned histories, pre-context failures and same-attempt enrichment |
| `just unit-native-package pse-runtime native-solvers 'test(occurrence_execution_tests) \| test(binding_admission_unit) \| test(diagnostic_rows) \| test(workflow::diagnostics) \| test(study_operation_unit) \| test(study_request_codec_unit)'` | 35/35 composite passed after repairing compiler path-demand admission, signed-zero identity and fixture assumptions; final selection 35/35 | Physical contracts, affine/pressure conversion, duplicate aliases, rename/revision identity, distinct occurrences, dependency refusal, cancellation, parameter composition, actual native batching and task-local preparation counts; shared diagnostic/result projection and descriptor codecs |
| `just unit-native-package pse-runtime native-solvers 'test(study_operation_admission_tests)'` | 4/4 composite passed after correcting physical/controller fixture requirements | Existing simulation, fit and horizon owners; canonical horizon input/prior/trajectory values; replay correspondence and physical mismatch refusal |
| `just unit-native-package pse-runtime native-solvers 'test(initialization_restores_original_specification) \| test(kernel_initialization_retries_typed_preparation_rejections) \| test(kernel_initialization_shrinks_failed_native_nonlinear_steps) \| test(kernel_initialization_retries_callback_trials_without_event_history) \| test(derived_big_m_follows_value_only_bindings) \| test(pounce_convex_batched_study)'` | 6/6 passed | Existing initialization stop/retry/restoration policy, value-dependent Big-M and native Pounce batch agreement with analytic/HiGHS controls |
| `just unit-package pse-operations 'test(migration_frozen_source_identity_and_history_ownership) \| test(migration_plan25e_frozen_source_identity_and_follow_on_target) \| test(migration_source_and_history_prefix_are_admitted_together) \| test(migration_follow_on_preserves_enum_order_and_complete_layout_signatures)'` | 4/4 passed | Frozen exact 25e source/history admission and appended V5 target/layout preservation declarations; no live database execution |
| `just unit-package pse-runtime 'test(study_operation_unit) \| test(study_job_v5_codec_unit) \| test(termination_detail_v2_codec_unit)'` | 7/7 passed | Operation descriptors, JobPayloadV5 and TerminationDetailV2 strict version/shape round trips |
| `just unit-package pse-catalog 'test(member_ticket_unit)'` | 1/1 passed | Exact publication-ticket inventory/version/attempt admission before I/O |
| `just unit-package pse-codegen 'test(codegen::documents::tests::)'` | 6/6 passed | Strict owner-schema equality including recursive roots, heterogeneous tuple positions/arity and constrained fixed arrays |
| `bash scripts/memory-cap.sh just unit-package pse-backend-native 'test(dynamics_profile_duration_schema_matches_closed_serde_representation)'` | 1/1 passed | Closed duration schema matches existing serde representation |
| `just unit-package pse-schema 'test(modeling_identities_declared)'` | 1/1 passed | Surviving declared modeling identities after deleting the legacy study relation |
| `just py-unit-native python/pse/tests/test_studies.py -q` | 6/6 composite passed after fixture corrections and rebuilding the extension property; 3 integration/sweep tests deselected | Repeated occurrences and owned results, typed dependency refusal, normalized binding identity, recursive physical mismatch envelope/closed byte-array shapes, controller tuple positions and rejection of caller-supplied seed capability |
| `just py-unit-native python/pse/tests/test_native_boundary_contracts.py::test_generated_settings_admit_native_defaults_and_expose_named_diagnostics python/pse/tests/test_native_workflow.py::test_compiler_failure_retains_typed_authored_source_span -q` | 3/3 passed | Generated native settings/named diagnostics and two retained typed authored-source-span cases |

**Interface-checked, 2026-10-01:** `just check-package pse-runtime` and
`just check-package pse-py` passed all-target compilation with explicit force-validation.
Native focused selections also compiled their affected native consumers. Source compilation
and stale consumers were repaired before the final passes; this is composite evidence.
Current project compile errors/warnings: zero. The dependency future-incompatibility notice
for `proc-macro-error2 v2.0.1` remains outside that source-warning claim.

**Implemented:** `direnv exec . just codegen` passed all six schema targets, physical fixture
outputs, native bindings and hakari. `direnv exec . just py-sync-native` rebuilt the editable
native boundary and generated API stubs from the compiled PyO3 metadata. The native diagnostic
envelope property was checked against that rebuilt extension.

Forcing a `pse-py` Rust unit harness with `unit-native-package` failed at linking against
CPython: that crate declares `test=false` and builds an extension module. No Rust inspection
unit executed in that mode. The supported Python adapter controls above supply boundary
execution evidence instead. One Python selector invocation collected no tests because the
recipe split a spaced selector; explicit node IDs corrected the invocation.

The independent [contract review](../design_review/reviews/design_review_studies-diagnostics-and-admitted-bindings_2026-10-01.md)
returned **Accept** at its bounded **Implemented/Tested** scope and records the corrections.
Its findings concern signed zero, result-role admission, shared refusals, recursive source
attribution, physical horizon replay, repeat receipt attachment and early/stale terminal-history
preservation; all corrections are implemented with focused controls.
ADR-0148 remains proposed pending its decision-PR route; that status is separate from functional
completion.

Integrated PostgreSQL migration/preservation, worker restart/fencing/receipt recovery and mixed
operation journeys, the 1000-point sweep, broader native/Python/parity campaigns, formatting,
hygiene, governance, docs publishing and performance measurement are **not_run** here. They
remain assigned to [25k](25k-integrated-qualification-and-closure.md). No live development
store was migrated, no throughput/scaling claim is made and no broad qualification is implied.

## Outcome (recorded after implementation)

### What was built

**Implemented/Tested:** F1–F5 are functionally complete at the focused scope above. Source-owned
failure projections preserve detailed codes, closed rules/stages, physical and explicitly tagged
nonfinite observations, revision-bound locations and recursive causes through Rust, generated
Python and durable records. Coarse class, disposition, severity, scientific usability and retry
remain separate purpose-specific decisions.

Typed assignment admission resolves compiler-demanded paths/member references against an
immutable revision, checks complete physical meaning and composes canonical member values once.
The `StudyBindingV1` frame records normalized content independently of occurrence identity;
explicit-ID compatible renames preserve content identity while replay still requires the exact
revision. Canonical signed-zero bits remain distinct. Horizon input/prior/trajectory values use
that same physical admission; reconstruction checks their operation-owned destination/member
correspondence without converting values again.

One admitted definition and one pure policy govern in-process and durable solve, simulation,
fit and horizon points through existing operation owners. Equal bindings retain distinct runs,
explicit continuation/default/refusal choices and every requested occurrence. E remains the
aggregate scientific authority. Both adapters project the same typed outcomes; failures and
cancellations need no fabricated scientific result. Terminal attempt history also retains
pre-context decoding and stale-attempt failures without duplicating a recorded attempt.

The durable adapter rereads policy under existing locks, live leases and revision fencing.
Pre-effect publication tickets commit before native writes; unresolved/partial effects block
replay and retain actual members. Receipt attachment is idempotent only for the exact descriptor;
conflicts refuse, and safe attempt supersession retains previous effect/history knowledge.
Required G3 evolution freezes the exact 25e source and appends V5 while preserving historical
identities, payloads and links. Missing old policy/scientific facts remain explicitly unavailable.
Generated boundaries carry the new versions and closed transport shapes.

Legacy case-array study APIs, executor-local policies, path-keyed numeric overlays,
content-based point uniqueness, duplicate study outcome relations and their obsolete callers,
tests and fixtures were removed. No compatibility executor remains. Architectural owners
blueprint §5.3, §19.3, §20.6, §21.5 and §23.2 and revision 93 describe the enduring contracts.
The [series coordinator](25-design-remediation.md) owns adopted finding dispositions. This
completed record remains available while the active series consumes its evidence.

### A mistake made and corrected

The first overlay implementation treated the optional compiler path map as a complete target
inventory, so otherwise valid path bindings failed admission. Explicit submitted path demands
now go through the existing compiler once, and workers receive admitted member coordinates.
Ordinary unit conversion also erased canonical signed-zero identity; the canonical-input path
now preserves A's canonical bits without changing represented-unit conversion semantics.

Independent review then exposed repeated receipt attachment being treated as a conflict after
partial recovery. Exact descriptors are now admitted idempotently, conflicting descriptors still
refuse, and a newer safe attempt may replace the point inventory without losing old attempt
history. Targeted controls verify these distinctions. Python fixture/header/result-row and
native-extension rebuild assumptions were corrected before the final six-test selection passed.

### Deviations from the plan, deliberate

Only prerequisites required for this cutover were pulled forward from G3, H, I and J: preserving
V5 evolution, source-owned admission facts, binding/resource identity and generated boundaries.
Their wider plans remain open. No separate workflow engine, automatic uncertain-effect retry
or scientific permission inferred from catalog/member counts was added.

Functional closure uses targeted checks as authorized by the maintainer. Authored cross-owner
PostgreSQL/restart/publication and mixed-operation journeys execute in 25k, together with the
series' aggregate static and performance qualification. The unsupported Rust extension harness
is reported separately from the successful supported Python execution. ADR acceptance remains
on its decision route; functional closure does not change its status.
