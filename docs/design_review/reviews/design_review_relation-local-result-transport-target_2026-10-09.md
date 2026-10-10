# Relation-local result transport target

**Date:** 2026-10-09  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** The relation inventory, projection, failure, cursor and publication contracts
proposed by [ADR-0169](../../adr/0169-relation-local-result-transport.md), from immutable
completion to Rust convenience access, Python streams and durable result blocks.  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and
the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** Product source at `4c24721e691187e1a5b28398b29722fbde671da8`, plus the proposed
ADR-0169 inspected on 2026-10-09. The inspected product paths have no working-tree changes.
Concurrent Plan 32 and other document changes are outside this assessment.  
**Reviewer:** Delegated design reviewer, independently assessing the coordinator-authored ADR
after an earlier focused advisory contribution. The reviewer did not implement the subject.  
**Disposition owner:** [Plan 33 EFF00/EFF07](../../plans/33-efficiency-principles-remediation.md#eff07).

**Architectural fitness: Accept at Proposed target-design strength. Behavioral/semantic
adequacy: Accept for the specified transport contracts. Overall decision: Accept.**

The target separates a scientific result from its requested representation and delivery.
Completion determines what happened and what relations exist; one relation projection owns
their rows; each request owns its traversal and mutable delivery state. This is the appropriate
boundary for selective access and bounded durable export. Intrinsic relation defects remain
sticky across clones, while resource and delivery failures cannot make the scientific result
or an unrelated relation fail. Exact completion and activation remain explicit.

This accepts the proposed design, not the existing complete-map implementation. No product
tests, probes or measurements were run. The review establishes neither an implemented remedy,
a measured improvement nor wider scientific qualification.

## 1. Scope, drivers and coverage

The functional target is useful selective inspection and complete durable publication of
already completed square simulation, optimization, dynamics and fitting/shooting results,
including truthful failed or interrupted observations. Longer trajectories and concurrent
readers must not require an unrelated complete Arrow map merely to inspect a header or name
inventory. Native completion retention remains necessary and is outside the proposed memory
reduction. No new capacity SLA or numerical method is introduced.

Inspected sources include runtime `workflow/results.rs`, `run.rs`, `simulation_results.rs`,
`modeling/trajectory.rs`, `modeling/results.rs`, `modeling/analysis_tables.rs`, `fitting/results.rs`,
`durable.rs` and `result_projection.rs`; Python `pse-py/src/workflow.rs`; blueprint §19.2 and
§21.1; and the source review's [F09](design_review_efficiency-principles-codebase_2026-10-09.md#f09).
The boundary excludes fresh scientific preparation, native solving, analysis retirement,
database lifecycle changes and whole-simulator adequacy.

## 2. Responsibilities and composition

| Owner | Decision and state | Consumed/exposed contract |
|---|---|---|
| Scientific completion/request owner | Immutable outcome, assessment, headers and available relations | Inventory and original scientific evidence; exporters cannot reassess or rerun science |
| Relation projection owner | Meaning, schema, row projection and intrinsic defect classification | Checked rows/chunks and exact coordinate correspondence for supported traversals |
| Request cursor/convenience owner | Mutable position, assembly, cancellation and delivery | Selected relation with explicit terminal success/error; complete convenience aggregates the same projection |
| Shared immutable result owner | Relation-local intrinsic failure and optional successful convenience retention | Clone sharing without sharing cursor position or retaining every streamed chunk |
| Durable publication owner | Attempt fence, global ordinals/ranges, exact membership and activation | Complete required manifest; bounded append is staging, not publication |
| Python adapter | Inventory names and one-consumption selected stream | The Rust transport meaning and allocation owners, without a second row authority |

These responsibilities fit existing owners. No new registry or generic execution framework
is needed. Changing delivery should not require knowledge of numerical qualification policy;
adding a relation should add its projection and inventory entry rather than branches in every
consumer. Tests of projection and error policy can use immutable fixtures without starting
solvers. Durable activation still requires its actual store boundary.

## 3. Contracts, semantic authority and physical meaning

ADR-0169 deliberately changes wrapper-dependent retry behavior. Intrinsic content, schema and
encoding-contract defects are typed relation-local failures. A caller's whole-table assembly
refusal, chunk allocation refusal, cancellation or destination/transport error is request-local.
Classification must follow the failure's cause, not whether an error happened inside a method
named `encode`. A later bounded request can therefore succeed after an earlier oversized
convenience request refused. No completed request can change the scientific assessment.

Inventory membership does not promise nonempty data or successful encoding. Advertised empty
relations retain their schema and manifest membership. Complete convenience must obtain every
advertised relation; selective access need only obtain its demanded relation. A cursor's prefix
is provisional until its explicit successful end, and a failed cursor cannot report completion.

| Physical element | Dimension/unit, basis and convention | Validity and authority |
|---|---|---|
| Values and trajectory samples | Preserve existing quantity/unit IDs, physical values, sample/time coordinates and source conventions | Immutable original result and registry schema; no transport conversion selects new physics |
| Sensitivities and fitting quantities | Preserve output/parameter identities, units and retained qualification conditions | Existing completion/evidence owner; encoding cannot invent omitted derivatives or permissions |
| Headers, checks and diagnostic observations | Preserve actual terminal class, candidate assessment and partiality | Completion owner under blueprint §19.2 and PS-10/PS-12 |

Well-posedness and numerical formulation are upstream and unchanged: this design neither declares
variable roles nor runs structural analysis or a solver. Numerical stage columns are therefore
not applicable. Physical fidelity is a preservation obligation of projection, not a new claim
that all upstream models or solver outputs are qualified.

## 4. Representative scenarios

The parent journey is [S05](design_review_efficiency-principles-codebase_2026-10-09.md#s05).
The following bounded cases distinguish this target from the current alternatives.

| ID | Stimulus and kind | Expected boundary and consequence |
|---|---|---|
| <a id="s01"></a>S01 | Select header/names from a substantial trajectory; representation demand | Inventory does no payload encoding; the selected header does not expand samples/sensitivities |
| <a id="s02"></a>S02 | One relation has an intrinsic defect; failure policy | Clones retain that failure only for that relation; another relation works; complete access refuses |
| <a id="s03"></a>S03 | Whole-relation assembly exceeds available memory; resource failure | That request refuses; a later bounded cursor can complete without a sticky resource failure |
| <a id="s04"></a>S04 | Add an output relation or replace a builder; instance/mechanism extension | Inventory/projection integration stays local; Python and durable consumers reuse their existing contracts |
| <a id="s05"></a>S05 | Export sample-major native completion into canonical output-major storage; representation composition | One projection preserves exact coordinates and global ranges/ordinals across chunks without a whole-table reorder |
| <a id="s06"></a>S06 | Concurrent clone cursors, cancellation, late intrinsic failure and escaped buffers; lifecycle | Cursors progress independently, failures retain their declared scope, prefixes remain incomplete and delivered arrays retain charges |
| <a id="s07"></a>S07 | Durable interruption or advertised empty relation; publication | Required membership remains exact; an interrupted prefix never activates; an empty relation remains represented |

Changing the physics or replacing a solver is outside this bounded transport decision. The
relevant simulator journeys are boundary roundtrip and growth/interruption after completion.

## 5. Execution, ordering and recovery

The current `RunResult::table` calls `tables`, which initializes a complete-map `OnceLock`.
Trajectory `tables` separately caches a complete map on success; Python name discovery calls
complete tables; durable `store_tables` obtains that map before its first block. The target
removes those dependencies rather than hiding them behind a stream interface.

Canonical trajectory transport currently globally groups `(symbol_id, sample)` using
`grouped_trajectory`, then establishes local base/ordinal state in `store_trajectory`.
Independent calls on newly generated chunks would reset global coordinates and only sort
within chunks. ADR-0169 explicitly prohibits this and owns a cursor for the complete relation.
Native output-major traversal can preserve canonical indexed storage while public traversal
remains sample-major. Both consume the same physical row projection and carry exact original
coordinates. Traversal layout is not another scientific authority.

Streaming must avoid both a complete-map prerequisite and a cache of all emitted chunks.
Bounded backpressure limits producer/consumer outstanding work; user-retained buffers remain
charged independently. A successful optional convenience cache may retain its chosen relation,
but a bounded request must not depend on creating that cache. Concurrent requests still share
the existing resource envelope; bounded chunks do not imply unbounded reader admission.

Durable append, descriptor reconciliation and seal retain their existing separation. Stable
global ordinals/ranges and exact empty membership govern activation. Cancellation or failure
can leave staged data, but cannot expose a prefix as a completed result. Existing attempt and
manifest recovery remains the owner; the ADR adds no independent publication protocol.

## 6. Foundations and gates

The judgments below apply to the proposed target, independently of pending implementation.

| Foundation | Judgment and evidence |
|---|---|
| AP-01 | Satisfied: scientific completion, projection and mutable request/delivery have distinct reasons to change |
| AP-02 | Satisfied: inventory, typed failure scope, terminal completeness and coordinate meaning are explicit |
| AP-03 | Satisfied: convenience, Python and durable publication compose one relation projection |
| AP-04 | Satisfied: outcome, relation membership, intrinsic defect, request failure and completion are consequential modeled distinctions governing consumers |
| AP-05 | Satisfied: immutable shared state and request-owned cursors expose the relevant structure and ownership |
| AP-06 | Satisfied: row/error policy can be assessed with immutable fixtures; only publication needs the store |
| AP-07 | Satisfied: small demand avoids unrelated encoding, and large relations have bounded traversal without retaining all chunks or globally reordering a complete Arrow table |

| Gate | Target judgment and reason |
|---|---|
| G1 | Pass: completion and one projection retain their separate authorities; consumers do not invent row meaning |
| G2 | Pass: exact identities, schemas, physical coordinates and partial-result interpretation are preserved explicitly |
| G3 | Pass: checked chunks and typed intrinsic/request failures retain defined rejection |
| G4 | Pass: discovery performs no encoding or scientific execution; export does not change assessment |
| G5 | Pass: request-owned mutation, bounded backpressure, explicit stream completion and complete-manifest activation address interruption |
| G6 | Pass: clone sharing is over immutable premises and errors of the same relation; traversal order preserves exact coordinate correspondence |
| G7 | Pass: ADR claims Proposed design, with no executed performance or scientific qualification claim |
| G8 | Pass: existing checked builders, allocation ownership, Arrow streams and block/seal operations remain the generic mechanisms |
| G9 | Pass: all seven foundations are satisfied for S01–S07 |
| PS-G1 | Pass for transport fidelity: units, quantities and source conventions are preserved; upstream physical consistency is not assessed |
| PS-G2 | Not applicable: no formulation, variable-role, structural-analysis or solver-entry change |
| PS-G3 | Pass for outcome transport: completion is immutable and prefixes cannot become scientific or durable success; upstream numerical integrity is not qualified |

Material refinements are DP-09, DP-10, DP-19–DP-24 and PS-10/PS-12. H5/H6/H8 support demand
locality; H15–H17 support traversals and bounded state; H23/H24 support coherent publication
and request-sized recovery. These are qualitative judgments, not measured speed or capacity.

## 7. Findings

No blocking target-design findings or SHOULD deviations were identified. The current F09
diagnosis remains valid and belongs to [Plan 33](../../plans/33-efficiency-principles-remediation.md#eff07).
Acceptance of ADR-0169 does not resolve F09. The implementation hazards in §5 and controls
in §10 are obligations of the accepted contracts, not additional design findings or a second
status ledger.

## 8. Library fit and ownership cost

The existing Arrow/checked-batch and allocation interfaces provide representation and ownership;
existing result blocks and manifest sealing provide bounded staging and visibility. The
scientific relation projection and error scope remain application responsibilities. This
review proposes no new library API or dependency and makes no unverified cross-version claim.
The relevant capability skills were consulted as routes; no external API uncertainty warranted
a probe. Migration cost is in splitting existing eager row producers and moving their consumers,
not building a new transport framework.

## 9. Alternatives and tradeoffs

Keeping the complete-map cache retains F09. Encoding each complete relation independently
improves selection but still obstructs bounded publication of a large demanded relation.
Caching every emitted chunk recreates that retained-state cost. Separate public and durable
scientific encoders weaken change locality and invite divergence. The selected one-projection,
request-owned-cursor design is the simplest viable alternative that serves all the stated
consumers. Generic Arrow mechanisms remain underneath it; they cannot independently supply
completion inventory or scientific row meaning.

The target may repeat bounded projection across independent streams instead of retaining a
complete Arrow map. That tradeoff is appropriate when small demand or resource pressure matters;
optional successful convenience retention remains available. A concrete relation requiring
unbounded global work or a new scientific interpretation would reopen its projection design,
as the ADR's revisit trigger states. No universal speedup follows.

## 10. Verification and evidence limits

**Proposed:** ADR-0169 supplies a coherent target and revealing controls. Source inspection
establishes the current complete-map and trajectory-reordering paths; it does not establish
that the replacement works. All product compile/test/probe/measurement commands are **not_run**.

EFF07 should exercise S01–S07 using fresh immutable completions: intrinsic failure shared
across clones but isolated from other relations; whole-table refusal followed by bounded
success; exact inventory including empty schemas; two independent cursors; interrupted and
late-failing prefixes; global row coverage and block ordinals across traversal/chunk boundaries;
no activation on interruption; and escaped-buffer validity after parent/stream drop. Use a
budget sufficient for native completion plus bounded transport and insufficient for the
complete Arrow map. This distinguishes the remedy from chunking an already materialized map.

Fresh representation roundtrips establish coordinate/bit/metadata fidelity, not independent
physical truth. No unit or property model is changed here; their conformance suites were not
run. Analytical expectations and applicable pinned external references remain the scientific
oracles when EFF07 changes their transport, with original tolerances. Invalid internal historical
values must not be promoted to scientific references. Quantitative benefit requires matched
complete-operation measurements; no minimum speedup gate is introduced.

## 11. Rule impacts and disposition

| ID | Current rule and owner | Required change and dependency | If retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Blueprint §19.2 describes complete-map trajectory transport, retry-on-failure there and sticky outer RunResult encoding; ordinary table wording says encoded once | Adopt ADR-0169's relation inventory/projection, clone-shared intrinsic failures and request-local assembly/delivery; EFF07 migrates all affected consumers | Selective and bounded requests retain F09 or wrapper-dependent failure semantics |
| <a id="rc02"></a>RC02 | Blueprint §21.1's Python streams consume current table access; current name discovery performs complete encoding | Make names inventory-only and selected TableStream access consume the same projection and error contract | Python discovery/access retains unrelated materialization despite the Rust target |

These refine the source review's confirmed RC02. The decision/design route must amend the
owning sections with a blueprint revision row before dependent implementation. No accepted ADR
is rewritten by this review. No additional rule changes or exceptions are needed. Plan 33 owns
current adoption, F09 disposition and EFF07 verification; this dated review does not copy status.

## 12. Decision

**Accept ADR-0169's proposed target for relation-local result transport.** Behavioral/semantic
adequacy and architectural fitness are separately acceptable for this document scope, with
all applicable gates passed and no SHOULD exception. The decisive evidence is the explicit
failure/completeness model, common projection with request-owned cursors, bounded retention,
and global coordinate contract resolving the inspected current mechanisms.

The next action is contract adoption at blueprint §19.2/§21.1, then EFF07 implementation and
its targeted controls. A replacement that prebuilds full maps, caches all chunks, shares mutable
cursors, makes allocation/delivery errors sticky, resets ordinals per chunk or activates a
prefix would fail this accepted design. No implementation, measured benefit, whole-simulator
adequacy or analysis-retirement protocol is accepted by this review.
