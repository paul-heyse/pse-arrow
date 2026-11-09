---
title: Shared engineering and solver contracts
status: review
date: 2026-11-08
tier: design
purpose: target
focus: principle-focused
doc_role: review
doc_topics: [design, architecture, numerical-execution, performance, validation]
standard: core-3.4
profile: process-simulator-1.5
baseline: 757ce921cd9809d91778adec5746edf6a76941b5
audit_publication: cf6822d10
evidence: Interface-checked
decision: Revise
---

# Shared engineering and solver contracts

**Revise toward shared source constructs, consumer-scoped compiled contracts and one native
composition for common solve preparation and launch.** The broad engineering build has two
separate causes. A registry declaration helper can produce many independent wire types for
one concept, and ordinary runtime consumers compile engineering messages and validators they
never use. Address both. A single shared message would still compile everywhere under the
current module boundary. Splitting files would leave duplicated contract meaning and could
leave the same Cargo unit intact.

Keep one scientific model across operation modes. Use named source constructs for genuine
common meaning, typed variations for distinct capabilities, and generated projections for
independent readers. Native admission, numerical execution and physical settlement retain
their own contracts. Protobuf helps share a declaration and derive multiple representations
from it. The repository still decides which meanings to share and which outputs to compile.
[Protobuf imports](https://protobuf.dev/programming-guides/proto3/#importing-definitions)

## Scope and evidence

The review covers `pse-runtime` and `pse-backend-native` with immediate declaration, generation
and consumer edges where they explain a finding. The target serves ordinary result reading,
modeling inspection, explicit and source-expanded Study preparation, and native square-system,
optimization and dynamic work. Workloads include many cases over an unchanged structure,
concurrent prepared operations, interruption and independent local consumers. The test route
is evidence of compile reach. Numerical model conformance, the whole-workspace test system,
and database and transport replacement are outside this decision.

Codex coordinated separate engineering and native investigations against `757ce921c` and
relevant working-tree changes on November 8, including the engineering descriptors,
generated Rust and native provider declarations. Core 3.4, efficiency heuristics 1.0,
test-assessment 1.2 where test evidence is used, and process-simulator 1.5 apply. At the operator's direction
the existing audit was used without refresh. DA-053 remains applicable to mode-specific
composition. DA-059 motivates checking projection demand, while its historical relation
finding has a different footprint. DA-052 remains a bounded inquiry and supplies no general
case for merging retained records. The audit publication at `cf6822d10` retains its original
source baseline `19e7b49a6` and narrower follow-on source `03765c22e`.
[Selected standard](../design_principles/standard.toml) ·
[Audit baseline](../audit/README.md) ·
[Audit findings](../audit/boundaries-and-composition.md)

The Proto briefings at `f55951cfe` supply hypotheses, including CBO-195 on owner-derived
projections and CBO-046 on generated calls. Current source already has both source-aware and
compact runtime descriptors, typed in-process validator preparation, and a generated Buffa
crossing for the private root child. These supersede the briefings' earlier implementation
observations about those paths. Their broader storage, telemetry and client alternatives
enter this review only where they constrain a shared contract.
[Proto briefings](../../library_leverage/libraries/proto/briefing.md) ·
[Proto comparison](../../library_leverage/reviews/proto/README.md) ·
[Private child wire](../../../crates/pse-runtime/src/math/native_child/wire.rs)

Observed source and interfaces are **Interface-checked**. Target mechanisms and effects
are **Proposed**. The original Study Routes invocation is retained build and execution
evidence, with its original failure. The current review compiled no targets and ran no
product tests or performance campaign.

## Engineering meaning and compilation

### <a id="f01"></a>F01 — Common index binding is lowered by occurrence

`pse-schema::catalog::modeling::indices()` supplies one list shape containing the name
and static-domain expression. The list also carries the `DomainBeforeName` binder facet.
Its anonymous item has no shared projection identity. The same two field names, numbers
and types appear in 19 engineering descriptor messages, including equation, connection,
state-port and state-specification occurrences. The common rule is already centralized in
the registry helper, so this is proliferation in generated identity and consumers, with
little present source duplication inside that helper. An independent client still receives
19 types where the same binding item could be used.
[Modeling declarations](../../../crates/pse-schema/src/catalog/modeling.rs)

**Target.** Give the common authored index-binding item one named registry owner, following
the existing named `ModelingUnitFactor` pattern. Generate its wire identity, view and local
constraints once and reference it at each use. Keep the list facet and occurrence-specific
constraints with the declaring field, including name resolution in its own lexical context
and provenance for each occurrence. An extension that applies only to transports or equations
belongs on that use, rather than changing every index binding. Retire the anonymous
occurrence-specific wire item declarations and corresponding generated conversions as their
consumers move. Before accepting the named type, compare the 19 field uses for effective
options, tags, presence and interpretation. Equal direct field shapes identify candidates.
The binder rule and the registry owner establish why this candidate merits consolidation.

Other lookalikes require narrower reuse. Header and full-page cohort requests already share
`StudyCohortCursor`, while their operations admit up to 64 and 256 items respectively.
Keep typed requests and their limits. Share the cursor and any proven common page input rule
at their owner. Keep source selectors, admitted revisions, passive recorded observations and
live protected reads distinct. The recorded-result selection and context are already shared
source meanings. A matching identifier or payload field does not grant publication,
restoration or read permission. Keep enum vocabularies and role identities that refuse
different values distinct even when their encoding is identical.
[Cohort workbench](../../../crates/pse-runtime/src/workflow/study_workbench.rs) ·
[Recorded selection](../../../crates/pse-runtime/src/workflow/result_selection.rs)

The largest direct shape group contains 117 single-string `value` messages, including
`BasisKind` and `CandidateRefusal`. The `Shape::StringEnum` lowerer derives each
vocabulary from the source owner and emits that member set as a local validation rule.
The adapters parse against the original native type. These are distinct vocabularies with
a repeated representation, so count the group as a code-generation opportunity and do not
merge it into a generic string message. Prefer sharing the lowering and conversion policy,
which the generator already does. A scalar or numbered-enum representation is a candidate
only where explicit presence, unknown values, stable numbering, local checks and archived
interpretation can be represented faithfully. A change in packing or spelling alone does
not justify a migration across this population.
[Vocabulary lowering](../../../crates/pse-codegen/src/codegen/boundary_projection.rs) ·
[Vocabulary adapters](../../../crates/pse-codegen/src/codegen/boundary_projection/adapters.rs)

### <a id="f02"></a>F02 — Ordinary readers compile the engineering contract bundle

`pse-runtime` unconditionally includes the engineering messages, validators and descriptor
bytes. Its `engineering-service` feature gates the generated service bindings and some native
adapters. With the feature excluded, `pse-project` still builds the runtime library and reaches
the default native backend library. The working-tree output inspected contains 822,477 lines
of generated engineering messages and related implementations, 206,548 lines of validators,
and 72,551 lines of native adapters, with distinct inclusion conditions. These are source
volume observations. The two library units do different work and this evidence assigns no
fraction of the build duration to any of those files.
[Runtime module](../../../crates/pse-runtime/src/host_operations/engineering/generated.rs) ·
[Runtime manifest](../../../crates/pse-runtime/Cargo.toml) ·
[Backend manifest](../../../crates/pse-backend-native/Cargo.toml)

The existing recorded-result cut proves an ownership pattern at source level. Four passive
declarations live at the model owner, a finite adapter closure serves ordinary export, and
engineering imports their exact file and maps the four Rust types back to that owner. The
runtime's ordinary export already uses that smaller adapter. It still builds the broader
runtime unit. Ordinary result tests also refer to the full descriptor for some cross-world
assertions. Put those assertions at the service or independent interoperability boundary so
the ordinary test can use the finite descriptor and still exercise the real native reader.
Separate descriptor pools for the recorded and engineering worlds have explicit boundaries.
Preserve that distinction wherever the same type is used in readers of differently scoped
worlds. Plan 66b retains the prior generator and receiving evidence.
[Recorded-result generator](../../../xtask/src/codegen/recorded_result.rs) ·
[Plan 66b](../../plans/66b-contracts-generation-and-interpretation.md)

**Target.** Declare useful source roots at the owners and compute transitive closure over
named fields, alternatives and imports. Generate one definition and validator implementation
per emitted Rust type owner. Derive supplier selections, ownership mappings and provenance
from that same selection. Keep separate the complete source-aware descriptor publication,
compact runtime reflection pools, compiled Rust capabilities and registered procedures.
A complete independent descriptor artifact may describe the full interface without an
ordinary reader linking its generated implementations or registering its procedures.

Place the broad service output and service-only adapters behind the existing explicit
service boundary. Move the finite passive consumers that still need a broad message to their
actual contract slice, beginning with the existing result cut and the separate
`ModelingInspection` root. Keep adapters that need private runtime construction at the
native owner, generated from the same admitted graph and compiled only with the actual
crossing. A distinct compiled contract owner is warranted where independent clients or
ordinary tools otherwise rebuild with performing runtime code. Prefer an existing suitable
owner where its dependency direction fits. A pure owner cannot depend back on runtime,
and pulling a service transport into a passive owner just to obtain generated traits defeats
part of the cut. The report selects this dependency boundary. A new crate would require a
separate decision after testing the exact cross-owner composition below. Ordinary readers
that still reach the base backend for real runtime work can retain it. A narrower passive
reader should drop that edge when the whole consumer can move. Keep feature sets intentional
for prepared solver profiles and automatic class-based selection at execution.

### <a id="f03"></a>F03 — Study input variation crosses an independent authoring capability

Field reachability from the compact engineering descriptor includes 576 of 602 top-level
engineering messages across the 29 RPCs. `InspectSource` reaches 4 and `InspectModeling`
181. `PrepareStudy` reaches 380 and `RestoreStudy` 390. These counts follow message-field
edges from input and output roots and exclude other packages from the totals. They describe
schema reach, independently of how Rust code is partitioned. In particular, `StudyInputs::Lazy`
reaches `StudyVariants`, then `StudySourceChoice` and `SourceDelta.additions`. An explicit
input consumer of the same generated request inherits that authoring closure even when it
cannot perform an inline source edit.
[Study inputs](../../../crates/pse-runtime/src/workflow/study_variants.rs) ·
[Engineering generator](../../../xtask/src/codegen/engineering.rs)

**Target.** Separate the explicit-point preparation root from source-expanded preparation
at the independent interface. Share the immutable operation definition, provenance and
prepared execution meaning where they agree. The explicit root consumes selected sources and
finite points. The source-expansion root owns authored changes, validation against the
exact parent, candidate enumeration and resulting provenance. Both use the same native
Study preparation and execution ownership where their scientific inputs agree. A consumer
that can author an inline delta gets that contract through the source-expansion root. The
explicit contract should have no transitive authored-delta dependency through request,
response or restore evidence. Restoration continues to bind the original variant and
interpretation; prove that a passive receipt reference can express this before narrowing
any live restore root. Retire the broad explicit/lazy wire request after its clients move
to their actual capabilities. Retain a single native Study operation and each scientifically
distinct input path. Keep descriptors and canonical readers required to interpret retained
evidence under their original version.

## Native scientific variation and shared policy

### <a id="f04"></a>F04 — Worker wire declarations mirror native demand and session identity

`ProviderDemand` and the scientific worker's `Demand` separately declare complete jets,
selected columns, Jacobian action, transpose action and weighted second-order demand.
Handwritten maps translate both directions. `Prepare` separately carries the native request's
outputs, order and demand. The handler maps integer derivative order to a native enum that
already has its own projection. Worker `Handle` and `SessionHandle` both carry the same passive
attempt/session/generation reference. Unlike the index-binding case, a new derivative product
must update independently declared alternatives and both mappings. The prepare route also
rechecks the same immutable expected request at successive construction boundaries.
[Native provider contract](../../../crates/pse-kernels/src/provider.rs) ·
[Worker protocol](../../../crates/pse-kernels/src/worker_protocol.rs) ·
[Worker handler](../../../crates/pse-runtime/src/scientific_worker/handler.rs) ·
[Prepared session](../../../crates/pse-runtime/src/scientific_worker/session.rs)

**Target.** Co-declare the projection on the native request, demand alternatives and a
single passive session reference. Generate the checked integer-width and alternative mapping.
Admit immutable demand once before constructing a provider, then carry the checked request
to binding. Binding still checks fresh actual capabilities, provider generation, physical
context and session state. Invocation validates handles and demanded extents against that
bound attempt, and attempt-local linearization retains its owner. Transport-specific request
bounds and deadlines remain with the crossing. Delete the mirrored demand and handle
declarations and handwritten conversion arms. The existing worker's integer mapping
`0/1/2` differs from the native projected alternative tags `1/2/3`. A projected shape
change requires an explicit private protocol revision and coordinated managed images.
Old bytes must retain their old interpretation or be refused. Verify generated support for
the exact ordinal and tuple/record variant shape before selecting the final emitted layout.

### <a id="f05"></a>F05 — Modes repeat native preparation and admission composition

Shooting and modeling fitting separately assemble structural facts, routing requirements,
settings, budgets, route selection, witness custody and `math::solves::admit_profile`.
Shooting reserves construction space before obtaining a witness and reduces the charge
afterward. Fitting can borrow an existing immutable witness and separately decides how much
retained memory to charge. Those are valid ownership variants. A change to common admission
or witness retention policy currently crosses both mode composers. The backend already
centralizes eligibility in `routing::Requirements::decision` and publishes capability rows
from `execution::Capability`. `OracleContract` already shares structure, coordinate ordering
and derivative/smoothness facts between algebraic oracles without requiring common callbacks.
[Shooting](../../../crates/pse-runtime/src/workflow/shooting.rs) ·
[Modeling fit](../../../crates/pse-runtime/src/workflow/fitting/modeling.rs) ·
[Backend execution](../../../crates/pse-backend-native/src/execution.rs) ·
[Oracle contracts](../../../crates/pse-backend-native/src/lib.rs)

**Target.** One native preparation operation combines immutable formulation facts, explicit
fresh or retained witness ownership and the admitted resource context. It selects the
class-specific adapter through the existing router, reconciles the witness reservation and
returns the chosen route with the exact artifact and callback demands. Modes supply the
actual formulation and keep scientific decisions about windows, residual objectives,
initialization and continuation. The selected adapter supplies its requirements from one
typed capability composition used by routing and conditional artifact preparation. Delete
mode-local repetitions of the common routing/admission sequence and its charge policy.
Reuse preparation across case/value changes while its structural, formulation and provider
dependencies hold. Recheck the original scientific problem for each new numerical state.

Square roots retain square admissibility, sign constraints and the distinct need for
directional Jacobian action versus a Jacobian-based preconditioner. Constrained optimization
retains objective, bounds, constraint and exact or limited-memory curvature contracts.
Dynamics retains time/axis, initialization and event context, derivative requirements
and partial trajectory extent. Residual fitting still requires its own eligibility.
Carry these class facts as typed variants of admitted work. Keep direct native callbacks,
domain restrictions, scaling and derivative provenance, and library-owned iteration.
Retain solver-native termination, original-coordinate screening and publication permission
as distinct results. An available Hessian, sparse structure or final point cannot be inferred
from transport or generic solver success.
[KINSOL adapter](../../../crates/pse-backend-native/src/execution/kinsol.rs) ·
[Native solver library reference](../../../.codex/skills/native-solver-libraries/SKILL.md)

### <a id="f06"></a>F06 — Launch policy is repeated before a shared supervisor

Simulation and fitting independently compose deadline, cancellation, activity and durable
preflight before calling `RunSupervisor`. Prepared Study has its own standalone/claimed
launch selection. The supervisor already shares registration, admission and joined completion.
On admission refusal it cancels and awaits `handle.finish()`, discarding the finish result.
The initiating refusal can therefore suppress an independent cleanup error. This is a
source-traced failure path. The review observed no leaked worker or failed release.
[Run supervisor](../../../crates/pse-runtime/src/workflow/run.rs) ·
[Study operation](../../../crates/pse-runtime/src/workflow/study_operations.rs)

**Target.** The existing supervisor owns one typed launch context for original operation,
attempt, deadline, cancellation, explicit claim state and registered physical owner. Modes
provide their dispatch and native result construction. The supervisor joins actual completion
and preserves independent finish causes on refused admission. A passive wait, disconnected
client, process exit and settled I/O remain distinct events. Delete the repeated common
preflight and deadline branches after the active callers move. Reuse `RunRequest::facts`
and the existing `Completion`/scientific permission projection at readers that need those
facts. Preserve separate identities for exact source, mathematical preparation, ordered
request, operation and attempt. Solver reports and dynamic trajectories retain distinct
payloads inside their original completion. Share the facts about authority and extent across
diagnostics, result readers and independent clients.
[Run outcomes](../../../crates/pse-runtime/src/workflow/run_outcome.rs) ·
[Diagnostics and results briefing](../../library_leverage/libraries/proto/briefings/diagnostics-and-results.md)

The persistent provider and the bounded private KINSOL child both need physical child
custody. Their communication, wait and shutdown contracts differ. The provider already
retains admission when wait cannot establish exit and joins I/O independently. The private
child has a bounded inherited channel and test-only call sites in the inspected source.
Keep its protocol and direct callbacks. Share a physical custody primitive only if a
maintained placement uses both with equivalent exit/unknown-wait guarantees. A private
child's best-effort drop is insufficient as the only settlement evidence for any new live
placement. Canonical reconstruction retains its separate C04 decision.
[Provider custody](../../../crates/pse-runtime/src/scientific_worker/process.rs) ·
[Private root child](https://github.com/paul-heyse/pse-arrow/blob/9de6d7d94e22890bf2c8ba73c2d0b47e0d700562/crates/pse-runtime/src/math/native_child.rs)

## Pivotal mechanisms and alternatives

**Selected generation composition.** Keep the registry and native `ProjectionGraph` as
declaration owners for first-party science. Emit explicit imports and select source files with
Buffa 0.9.2 `generate_with_diagnostics`. Its complete corpus resolves imports and options
while `files_to_generate` selects output at file granularity. Preserve warnings about
unmatched overrides or unresolved external ownership. Place genuinely shared definitions in
an imported file and use exact `extern_paths` for shared types. Existing same-package
recorded-result imports establish one example. They do not establish an arbitrary
same-package cross-crate split. Derive supplier mappings from the same ownership selection
and require a Rust owner for every referenced type. The pinned maintained view profile
requires matching allocation-accounting capability through admitted imported/deferred
decoders. Shared corpus preparation is reusable only for the exact effective descriptor
world and matching representation controls. Keep reflection pools explicit and preserve
effective options and available source information in their respective artifacts.
[Pinned Buffa generation](../../../vendor/buffa-codegen/src/lib.rs) ·
[Maintained decoder profile](../../../vendor/buffa-codegen/pse-patch.md) ·
[Buffa reference](../../libraries/proto/Buffa.md)

**Selected service and validator composition.** Feed ConnectRPC codegen 0.9.0
`generate_services` and the maintained Protovalidate-Buffa 0.7.2 `prepare`/render
pipeline from the same admitted corpus. Both already run through typed composition.
Views are required on actual Connect message roots. Generate validators at the crate
owning the Rust type because the supplier emits `Validate` implementations for it.
Connect's `EncodableImpls::AllMessages` can supply view encoders in the defining crate
and skip externally owned types downstream. The default outputs mode visits output types
of services in selected files, deduplicates within one invocation, and skips absolute
external mappings. A `crate::` re-export of a foreign type would defeat that recognition.
Prefer default outputs mode with message and service outputs in the same owner. Use
all-messages mode only for a finite service-ready owner where a downstream service
actually needs to return shared binary views. Owned returns are available where a view
would add little value. Keep a passive owner free of Connect dependencies where possible.
Verify the precise mixed passive/service split before allowing separate crates. If it
cannot preserve a single Rust type and validation owner, keep service-used types and their
traits together and move truly passive types into the passive owner.
[Current supplier composition](../../../xtask/src/codegen/proto_projection/composition.rs) ·
[ConnectRPC codegen 0.9.0](https://github.com/connectrpc/connect-rust/blob/v0.9.0/connectrpc-codegen/src/codegen.rs) ·
[Maintained validator](../../../vendor/protoc-gen-protovalidate-buffa/README.md) ·
[Rust coherence](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules)

Generated registrations can expose each actual service slice through ConnectRPC 0.9.1
`Router`. Where separately assembled services need composition, `try_merge_in_place`
can refuse route collisions. Check registered procedures directly, keep effective route
limits and early aggregate admission, and authenticate the prepared endpoint. Descriptor
discovery is separate from authorization and does not require registering a procedure.
Generated structural validation retains local required alternatives and bounded fields.
Native owners still enforce exact context, durable effects, units/datum, original
permission and physical settlement. Context7 supplied capability leads from upstream
`main`. Release and maintained source govern the mechanisms selected here.
[Installed version's Router source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/router.rs) ·
[Local services briefing](../../library_leverage/libraries/proto/briefings/runtime-and-local-services.md)

| Alternative | Judgment |
|---|---|
| Keep the current single engineering Rust population and name shared fragments | Useful first reduction for F01. The ordinary reader still compiles unused contracts, and a narrow client still carries broad Study reach. |
| Split generated files by operation in the same ungated Rust unit | Useful for ownership and supplier selection. It cannot settle F02 while every root still compiles the same modules. A per-operation crate or feature matrix would add composition and invalidation cost for capabilities that vary together. |
| Consumer-scoped owner-derived hybrid with explicit Study roots | Selected. Independent consumers get interoperable descriptors and useful Rust subsets. Share compiled contracts across real dependents. Keep native admission and typed variation behind the service. The cross-owner trait and reflection controls below settle the exact compilation placement. |
| Native-only operations and explicit adapters | Selected for same-process work, numerical callbacks and internal policy. Independent clients and retained inspection still need generated boundaries. |
| Proto-first declarations or Rust-first `proto_rs` 0.11.26 | Proto-first fits independently governed schemas. For the inspected first-party contracts, a second executable scientific authority or registry mapping would remain. The published Rust-first annotations, schema export and fallible shadows are credible for an isolated owner, but a complete replacement would also have to cover the existing registry, validation, reflection and Connect composition. No such simplification was established. |
| Buffa `apply_companions` to combine generated output trees | Candidate only where a package-scope companion replaces handwritten scaffolding. The helper is single-application, requires matching package stitchers, and can leave an orphan file. The current typed supplier composition already avoids repeated parsing and provides its own admitted output paths. Companions alone do not improve compile reach. |

[Published Rust-first alternative](https://docs.rs/crate/proto_rs/0.11.26) ·
[Generation briefing](../../library_leverage/libraries/proto/briefings/contract-ownership-and-generation.md)

## Change and failure scenarios

| Scenario | Target behavior and verification |
|---|---|
| <a id="s01"></a>S01 · Add a rule shared by authored index bindings | Change the named registry item once. Generation emits a single definition, validators and conversion family. An independent descriptor reader and native adoption control distinguish a malformed binding from a well-formed binding that fails lexical resolution. An equation-only rule remains local to equations. Compare tags, presence, metadata and preflight across all migrated uses. |
| <a id="s02"></a>S02 · Select an ordinary result reader or explicit Study client | The ordinary target's planned units and actual module includes exclude broad engineering Rust, while a separate reader opens a complete approved descriptor world. The explicit request and response closure exclude authored inline edits. An independent client exercises both actual procedure slices. Test missing import, foreign pool and duplicate registration. A service-only change leaves the passive contract and its consumer's compiled input unchanged. |
| <a id="s03"></a>S03 · Extend or substitute a native capability | Add the capability and exact artifact/callback demand at its adapter. A square root with directional demand, a constrained fit with different curvature needs and a dynamic run with events select their own native contracts. An unsupported derivative is rejected before work. Source construction, fitting and shooting share the admission composer. Preserve original-coordinate acceptance, result axes and the difference between a candidate, a screened result and publication permission. |
| <a id="s04"></a>S04 · Interrupt preparation or refuse admission with an independent finish error | Under a finite resource reservation, a refused dispatch keeps its original cause and an independent native finish cause. Preserve any still-live owner until joined settlement. For a worker that prepared successfully but lost its acknowledgement, query or retire the original attempt rather than starting it again. A stale session reference cannot regain current authority. |

An expanded study can still legitimately need many distinct edited structures. Share
immutable preparation and witnesses only when exact structure, physical context, provider
generation and formulation still match. Bind each case's own values and preserve its
continuation and screening result. A wire type merge should neither widen the lifetime of
a protected read nor turn a borrowed witness into an owned copy. Case concurrency,
derivative callbacks and trajectory/event retention remain at their actual execution
owners. The review found no reason to move them across a new process or encoding boundary.

## Assessment and evidence limits

| Foundation | Observed verdict and reason |
|---|---|
| AP-01 Separation of concerns | Violated. F02 crosses independent compilation responsibilities. F05 and F06 leave common policy spread across mode composers. |
| AP-02 Stable contracts | Violated. The native provider extension in S03 requires independent protocol edits in F04. |
| AP-03 Composition | Violated. F03 joins independently consumed capabilities in one wire root. F05 joins common admission separately per mode. |
| AP-04 Domain model and authority | Violated at the worker boundary in F04. Native oracle and operation owners preserve meaningful differences that a universal DTO would flatten. |
| AP-05 Explicit structure | Violated. F01 presents a common binding as occurrence-specific contracts. F02 fails to express the ordinary consumer's actual generation reach. |
| AP-06 Local reasoning and testability | Violated. S01 and S02 expose incidental cross-owner generated work. S03 requires common admission changes in independent mode paths. |
| AP-07 Execution fits workload | Violated for the selected development workload. The ordinary read compiles unrelated generated input, and the distinct route test reaches a broad executable closure. Shared generated code and native workload cost require separate measurement. |

| Gate | Observed verdict in scope |
|---|---|
| G1 Authority | Fail. F04 independently declares and translates the same provider demand and session reference. |
| G2 Semantic fidelity | Unresolved for the affected portable population. The traced mappings distinguish key scientific meanings, but their full presence, variant and retained-version controls were not exercised. F01 alone establishes no semantic loss. |
| G3 Validity | Unresolved for assembled crossings. Selected native adoption, local validation and admission are visible. S01 and S03 still need discriminating boundary evidence. |
| G4 Hidden behavior | Pass for the inspected passive descriptor, explicit service start and direct callback boundaries. Source routes distinguish inspection from invocation. |
| G5 Consistency and recovery | Fail for original failure preservation in F06. The native refusal path can lose an independent settlement error. No false commit or actual orphan was observed. |
| G6 Transformation and reuse | Unresolved for the reviewed portable/retained representations and shared preparation lifetimes. Exact source and pool controls constrain the recommendation. |
| G7 Truthful capability claims | Pass for the inspected typed adapter selection and separation of observation from scientific permission. The broader live capability population remains outside that evidence. |
| G8 Library leverage | Pass for the inspected solver and generation paths. Existing libraries own numerical iteration, descriptor generation and local validation. No established library replacement for scientific or custody policy was found. |
| G9 Architectural fitness | Fail. The authority and composition failures and the build reach in F01–F06 require revision. |
| PS-G1 Physical consistency | Unresolved for portable round trips. The target carries original quantity identity, order, basis and validity through owner-derived projections and native admission. No unit or property model conformance is qualified here. |
| PS-G2 Well-posedness | Unresolved across the assembled modes. Variable and row roles, structural assessment and witnesses are inputs to admission. The shared composer must preserve pre-solve rejection and attribution. |
| PS-G3 Numerical integrity | Unresolved across the assembled modes. The target keeps formulation/domain checks, derivative and scaling provenance, class-specific status and independent post-solve screening. No cross-backend numerical equivalence is inferred. |

**Measured, historical.** `pse inspect tests` and `pse inspect invocation --detail outputs`
for `01a1bde7-2ebd-7547-ad0b-73507cd795b6` recovered the original default-profile,
force-validated `pse-runtime:engineering_service` selection with KINSOL, Diffsol and Ipopt.
The original retained report records 473.57 seconds in the build phase and 5.04 seconds
in the test phase. Cargo reported 7m 51s and named `pse-backend-native` and `pse-runtime`
as compiling. The selected test was
`routes::prepared_engineering_study_client_simulation_fit_and_horizon_keep_original_physics`.
It recorded one failure against the zero baseline. These measurements describe that invocation
and its existing artifacts. They establish neither per-crate timing nor repeated native
library compilation, and they do not qualify the simulation.

**Interface-checked, current.** `scripts/pse inspect test-selection --admitted` with
`pse-runtime:engineering_service` and the original feature set explained one selected
test root and 1,120 reached planned units without building or listing tests.
`scripts/pse inspect dependency-units pse-backend-native` explained both that service root
and the `pse-runtime --bin pse-project -F product-reader,force-validate --without-feature engineering-service`
root. Both expose the backend dependency through runtime. Source inspection establishes
the actual ungated generated module inclusion. The descriptor counts came from a passive
`buf convert` of `descriptor.bin` and a walk over input/output message references.
None of these observations establishes incremental invalidation behavior or a speedup.

**Proposed acceptance.** Resolve F02's placement uncertainty with one cross-owner compile
control covering a shared message, an imported validator, an actual Connect service and
a passive reader. Verify a valid and refused crossing and independent discovery with the
selected descriptor world. An ordinary target should continue to compile after a
service-only edit without reaching its generated inputs. Check planned units and actual
compiled modules, then compare edit/rebuild behavior under the same toolchain, feature
closure, profile and cache state only if claiming a numerical improvement. The structural
findings and selected authority design do not depend on a new timing campaign.

For the source-forward generation control, change a selected `ModelingInspection`
field before regenerating. The extractor must build from the source owners and produce
its new closure without compiling stale service adapters. Then compile the resulting
ordinary and service consumers. This distinguishes a smaller output from a genuinely
independent bootstrap. Use the finite recorded-result control as the comparison.

## Disposition and decision

**Behavioral adequacy:** revise the duplicate native authority and failure preservation.
**Architectural fitness:** revise the generated closure and common native composition.
**Overall:** Revise. The proposed target is grounded sufficiently to plan, with the
cross-owner compilation control and scientific boundary checks still required. The findings
below route to existing owners for disposition. This report does not schedule them.

| Finding | Primary disposition owner | Dependent or preserved scope |
|---|---|---|
| F01 · Index binding | [66b B01 and B05](../../plans/66b-contracts-generation-and-interpretation.md#b01--deliver-useful-generation-closures-and-reproducible-bootstrap) | Shared registry rule, one generated type, local occurrence context. |
| F02 · Compiled closures | [66b B01](../../plans/66b-contracts-generation-and-interpretation.md#b01--deliver-useful-generation-closures-and-reproducible-bootstrap) | 66e E02 ordinary result receiving and 66f F04/F05 independent consumers. |
| F03 · Study roots | [66a A01](../../plans/66a-scientific-operations-and-product-semantics.md#packages) | 66b B01/B03 generation, 66c C02 actual service consumers and 66f F05 client change. |
| F04 · Worker projections | [66b B03](../../plans/66b-contracts-generation-and-interpretation.md#b03--finish-finite-source-to-consumer-migrations) | 66c C03 session admission and 66d D04 derivative truth. |
| F05 · Preparation composition | [66d D01/D02](../../plans/66d-mathematical-representation-and-numerical-execution.md#packages) | 66c preparation consumes the selected route, witness and artifact demand. |
| F06 · Launch and settlement | [66c C03](../../plans/66c-preparation-execution-and-service-lifecycles.md#c03--close-performing-owners-and-worker-sessions) | C02/C05 actual consumers and 66e E06 diagnostics. |

### Rule impacts

- <a id="rc01"></a>**RC01 · Engineering wire surface.** F03 changes the effective local
  procedures and the source of their generated API under blueprint §4.7 and §21.8. Publish
  the selected explicit and source-expansion contract with intentional versioning and
  migrate the actual clients in one active path. If the combined endpoint remains the
  public contract, F01 and the passive part of F02 still apply, while explicit clients
  retain its broad schema dependency.
- <a id="rc02"></a>**RC02 · Compilation ownership.** F02 may require a new compiled owner
  only if no existing suitable crate can supply the required one-way dependency. Follow
  the crate decision route before any addition. If the boundary stays inside existing
  crates, scope service code to its feature and ordinary selection, and report the remaining
  invalidation where both consumers still share a compiled unit.
- <a id="rc03"></a>**RC03 · Worker derivative order.** F04 replaces the private wire order
  table with an owner-derived projection under the scientific worker boundary. Keep explicit
  tags and change interpretation only through the revision and coordinated-image decision.
  If those exact wire tags must stay, generate a single checked mapping from the native
  owner and retain the version-specific numbering at that boundary.

Give F02 and F03 first attention for the ordinary feedback loop, while F04 and F06 address
authority and failure preservation. Their corrections do not require waiting for a measured
compile benefit. Establish the canonical names and interpretations before generating
smaller consumer outputs. Prove exact shared-trait ownership before adding a compilation
owner. The target retires each replaced declaration, adapter and entry point with its
consumers, and records any required retained-evidence interpretation at its original
authority.
