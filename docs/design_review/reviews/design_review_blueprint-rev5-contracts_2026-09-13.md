# Design review: blueprint revision 5 contract amendment

## 1. Decision and scope

**Decision: Accept for the bounded proposed-design scope below.** The revised
contracts address R4-01–R4-13 and assign a concrete consumer or deferred trigger
to L1–L9. Acceptance means the reviewed mechanisms, boundaries and refusal paths
are specified. It does not certify their implementation, scientific validity or
performance, and does not change any ADR to `accepted`.

**Proposal:** [blueprint revision 5](../../authoritative_design/blueprint.md),
ADRs 0039–0048 and their capability-map/register reconciliation.
**Standard:** [Data Model–Based Design Charter](../design_principles/DATA_MODEL_DESIGN_CHARTER.md),
gates G1–G7. **Author/reviewer:** Codex, including review of its own amendment;
this is not an independent second-person review. **Date:** 2026-09-13.
**Evidence:** **Proposed** architecture, selected **Interface-checked** library
mechanisms and the explicitly bounded historical **Tested** characterizations
in the [revision-4 review](design_review_blueprint-rev4-library-contracts_2026-09-13.md).

**Observable outcome:** implementers have one declared route for admitting
physical meaning, resolving demand, binding complete stage inputs, handling
guarded failures, and preserving content through alternate encodings. Existing
Arrow/DataFusion operators perform their standard jobs; domain rules remain in
the platform contracts that invoke them.

**Method and coverage.** Inspected the changed semantic contracts in blueprint
§§4–9, 14, 18, 20–21 and 24–26, their declarations in §6, affected decisions and
glossary, all new ADRs and their supersession boundaries, and the four capability
maps' current recommendations. Replayed the earlier review's counterexamples
against the proposed rules by inspection. Followed the concrete journeys in §5;
checked section/port ownership, membership framing, numerical/failure policy and
the implementation handoff. Static validation is recorded in plan 02.

This is a follow-up on the changed contracts. Unchanged thermodynamic
correlations, every unit template, the full IDAES coverage matrix, all specialized
diagnostics and later-phase solver/provider routes were not re-certified. No
product compiler, solver, canonicalizer or Python adapter was implemented or
executed by this amendment. The retained probe receipts were not regenerated.
Context7 returned broad analyzer/runtime/reader material during reconciliation;
the pinned source/probe evidence in the earlier review remains the basis for
precise library claims. Unbuilt behavior is outside this acceptance claim,
rather than silently being scored as tested.

The reviewed blueprint SHA-256 is
`63629cd7221d7083f95b7e39f916cc6a6d3395599a2faadf5674e98c175c1a5b`.
Dependency pins/features, both lockfiles, the toolchain, charter and original
probe source are unchanged; Cargo.toml has only a citation-comment amendment.

**Supported limits:** local single-writer publication; explicitly admitted Arrow
layouts; the row/byte/offset envelope in blueprint §5.3; complete stage input
keys; pure contracted kernels; and only numerical/backend routes whose actual
implementation and conformance binding is available. Larger canonical streams,
finer dependency tracking, Parquet optimization, coalescing and preimages retain
register triggers. The design supplies rejection paths for unsupported cases.

## 2. Authority and lifecycle map

| Concept | Identity and authority | Revision boundary / permitted update | Derived representations |
|---|---|---|---|
| Relation and physical quantity meaning | Registry declarations, semantic IDs and complete quantity-operation rules; blueprint §4/§6.2/§8 | Versioned contract/package amendment admitted through P2 | Arrow fields/views, recursive validators, analyzer checks and generated adapters |
| Model / case | Stable authored identity; separate snapshot classes and explicit model/overlay parents | Change set and atomic P2-valid commit; compiler cannot write authored/reference facts | Normalized seeds, indexed/scalar math and case-bound views |
| Stage inputs and results | Named input/output ports; one producer per port | Complete immutable output snapshot after postconditions; an earlier stage is never overwritten | Catalog handles, physical encodings, memo entries and attempt sidecars |
| Kernel and numerical policy | One KernelSpec, implementation/derivative bindings and selected versioned numerical policy | New contract or selected case policy changes dependency inputs | UDF, native, exported and Pyomo bindings that pass the declared conformance scope |
| Logical / encoded identity | `pse-ids`; canonical v2 relation/snapshot preimages versus finished-byte checksums | New logical meaning changes content identity; alternate valid encodings can share that identity | Physical manifest, checksum-named objects and conditional ref |
| Evidence and failure | Required typed outcomes, terminal attempt/run state and noncanonical diagnostic objects | Immutable outcome publication; mutable workspaces remain run-owned | EXPLAIN/protobuf, structured diagnostics and provenance queries |

**Opaque behavior:** specialized kernel bodies, numerical workspaces, solver
callbacks and storage I/O remain ordinary code behind the declared contracts.
Registration or a function signature cannot supply a missing derivative,
conditional implementation or physical conversion.

Renames preserve semantic IDs while changing label-bearing relation content.
Row/batch order, supported dictionary encodings and hidden null payload do not
change declared logical identity. Changed semantic parents remain visible even
when an unchanged relation shares storage; initial reuse does not discard lineage.

## 3. Semantic contracts and invariants

| Contract | Representation and boundary | Defined failure | Evidence / decisive fixture |
|---|---|---|---|
| Active semantic admission | Generated recursive validator at provider/bundle, pre-rewrite, analyzer, post-optimizer and output boundaries; blueprint §4.4 | `schema.*` or head mismatch before use | **Proposed**; E1 refutes passive registration; `semantic_admission_paths` |
| Complete quantity and head conversion | Quantity-operation selection/derivation, exact types by default and checked explicit conversions; §8.3/§14.2 | Unsupported/ambiguous composition or lossy/incompatible head rejected | **Proposed**; E3 plus `head_conversion_exactness`, `quantity_composition` |
| Decidable stage ownership and demand | P3 seeds, finite bound scopes, P6 closure and port-bound P7–P10 bundles; §9.6/§14.1 | Unknown/ambiguous demand, undeclared port or late realization demand fails | **Proposed**; `stage_bundle_graph`, `demand_seed_closure` |
| Guarded operations and outcomes | Actual CASE/branch regions, complete kernel outcome relation and selected numerical policy; §7.3/§18 | Excluded branches do not execute; selected invalid operations return typed failure; unsupported routes refuse | **Proposed**; E2/E6 plus actual backend conformance |
| Stable content with checked physical envelopes | Recursive canonical copy, explicit metadata/validity/framing/membership and separate checksums; §5.3/§20 | Unknown layout/version, size excess, incomplete member or corrupt encoding rejected | **Proposed**; E4/E5 are counterexamples, not a v2 implementation proof |
| Complete reuse and accounted resources | All named inputs/absence/policies/binaries; shared runtime with fallible reservations; §14.3 | Changed key cannot hit; failed reservation cannot publish a partial successful stage | **Proposed**; incremental/clean and concurrent resource fixtures |

Missing measurements remain nullable values. Invalid evaluation is a typed error,
or a mandatory per-coordinate outcome in the separate tolerant operation; a null
alone cannot stand for both. An absent optional input is part of the cache key.
An empty required output is explicit, while a failed output is never an empty
successful bundle. Unknown/conflicting rule facts remain outside decided heads.

Equivalence is role-specific: canonical content identity, physical checksum
integrity, full values/multiplicities for pushed scans, guarded numerical outcomes
under the selected absolute/relative/signed-zero policy, and attributable decoded
meaning for noncanonical plan evidence. These are not interchangeable tests.

## 4. Derivation and execution design

| Operation | Declared inputs / assumptions | Output, effects and invalidation |
|---|---|---|
| Admit / normalize | P2-valid authored/reference snapshot; registry and quantity contracts | Typed P3 expressions and finite demand seeds; no inspection-driven construction |
| Resolve / instantiate | Features, scopes/domains, candidate universe including absence, method preferences and bindings | Closed or explicitly unresolved demand, then complete named math bundles; real requirements cannot appear as an undeclared P6 feedback loop |
| Match / group / expand | Bound rule ports, key discipline, null/empty/order policies and contribution provenance | Shared DataFusion/Arrow predicate execution, aggregate/unnest descriptors and indexed equations until the expansion consumer |
| Bind / prepare / execute | Case values separately from guesses, numerical policy, actual kernel/backend contracts and shared budget | Owned prepared/branch workspaces, mandatory outcomes and policy-qualified derivatives; refresh bound values independently of topology reuse |
| Encode / publish / reuse | Admitted complete relations, reserved buffers, membership and semantic parents | Canonical logical hashes; separate finished physical bytes; verified manifest then conditional ref. Complete-key memo hits only; attempts/evidence remain sidecars |

The library opportunities are bounded: AnalyzerRule and recursive traversal for
admission; built-in CASE for relational guards; physical predicates and Arrow
filters for exact scans; standard aggregation/unnesting for P8 and memberships;
shared RuntimeEnv/reservation APIs for accountable allocations; field-aware UDF
arguments and scalar paths; safe ScalarBuffer owners; and explicit-schema readers
for observations. No new query engine or general inversion/ingestion framework
is justified. CSV/JSON feature wiring remains part of the future import consumer.

## 5. Representative journeys

**Ordinary extension.** A method requiring a second property declares the
requirement and implementation/quantity contract. P3/P6 bind the finite scope and
candidate set before P9 realization. Its adapters come from the admitted binding;
an unavailable derivative or undeclared late dependency is rejected. Extension
work is the new declaration, any new algorithm and its focused conformance fixture.

**Meaningful change.** Add a species, change a unit set, or remove a provider.
The domain/package/candidate input changes, so the complete P7/P6-dependent keys
change. Fresh output snapshots preserve the new lineage. A fixed-value change
refreshes `bound_values` and any value-dependent analysis/preparation even if the
instruction topology is the same. Clean compilation is the equivalence oracle.

**Alternate representation.** Construct equal nested nullable relations with
different hidden payload/child validity and serialize each as IPC file or
Parquet. Canonical v2 specifies normalization before logical hashing. Each finished
encoding has its own checksum; restore decodes and validates meaning instead of
substituting a byte checksum for logical identity. A Python consumer that loses
an extension cannot re-enter a checked platform path by silently relabeling storage.

**Failure.** A mixed batch selects different branches and includes a log-domain
error only in an excluded branch. CASE/native masks prevent its evaluation; a
selected invalid branch instead returns the declared error/outcome. Cancellation
or reservation failure releases owned workspaces and prevents successful stage
publication. A truncated existing object fails verification before a new ref can
expose it. These are proposed journeys with the named acceptance fixtures in plan 02.

## 6. Acceptance gates

Each Pass below is **Proposed document scope**, not runtime assurance.

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Pass | Registry meaning, named stage producers and explicit snapshot classes separate model, derived output and sidecars; §4/§5.3/§14.1 | Generate the projections; preserve accepted ADR arguments |
| G2 — Semantic fidelity | Pass | Complete quantity rules, exact head admission, explicit missing/failure states, recursive canonical representation and numerical policy; §7–§8/§14.2/§18 | Execute semantic/representation fixtures before claiming correctness |
| G3 — Validity | Pass | Every supported admission route invokes the validator; unsupported layouts, conversions, ports and bindings have rejection paths; §4.4/§5.3/§18.5 | Test actual entry points, including nested and optimized-away fields |
| G4 — Hidden behavior | Pass | Guarded execution is a real control-flow route; kernels are declared pure; candidate and policy inputs are explicit; §9.6/§14.3/§18.2 | Branch/effect and late-demand negatives |
| G5 — Consistency and recovery | Pass | Complete stage publication, distinct logical/encoded identity, verified existing objects, complete manifest then ref; §14.3/§20 | Local interruption, corruption, CAS and cancellation fixtures; wider deployment stays deferred |
| G6 — Transformation and reuse | Pass | Complete keys, explicit ordered numerical contracts, lossless/converting head policy and full pushdown oracle; §7.3/§14.2–§14.4/§24 | Incremental/clean, branch/numerical and complete-result differential checks |
| G7 — Truthful capability claims | Pass | False registration/conditional/protobuf/resource guarantees withdrawn; explicit implementation bindings and unsupported refusal; §18.5/§18.9/§26 | Keep entries Proposed until actual routes pass; no performance claim from API availability |

## 7. Principle findings

No residual MUST gap was identified in the revised, bounded contract decisions.
The table records the decisive corrections, not new assertions that the runtime
already enforces them. Verification names are Proposed fixtures in plan 02.

| Finding / disposition | Principle IDs | Concrete evidence | Consequence addressed | Correction in revision 5 | Verification |
|---|---|---|---|---|---|
| R4-01/05 — Addressed in design | DM-06, DM-07, DM-41–DM-43, DM-59 | §4.4/§14.2; E1 ordinary query bypass and E3 lossy casts | Invalid quantities can no longer be admitted by the specified route merely because storage is castable or registered | Active recursive admission and source-aware explicit conversion | `semantic_admission_paths`, `head_conversion_exactness` |
| R4-02/09 — Addressed in design | DM-24, DM-28, DM-40 | §7.3–§7.4/§18.2/§21.2; E2/E6 | Excluded-branch errors, reordered cancellation and nonlinear Affine bodies had inconsistent behavior | CASE/regions, guarded ordered default, selected numerical policy and qualified LinearExpression | Guarded/numerical/nonlinear-affine fixtures |
| R4-03/04 — Addressed in design | DM-12, DM-14, DM-15, DM-48 | §5.3/§20; E4 | Equal relations differed by hidden null bytes, while valid file bytes contradicted logical object names | Recursive v2, explicit membership/framing and independent encoded checksums | Canonical/encoding/membership/publication fixtures |
| R4-06/07 — Addressed in design | DM-18, DM-21–DM-23, DM-31–DM-33 | §6.11/§9.6/§14.1–§14.4 | Ambiguous producer order and omitted domain/binding/value inputs could create stale models | Early seeds, bound ports, complete output snapshots and conservative stage keys | Demand/stage graph and incremental/clean fixtures |
| R4-08/10 — Addressed in design | DM-01, DM-06, DM-08, DM-25, DM-43, DM-44, DM-47 | §6.2/§6.11/§6.13/§8.3/§18.5 | Adapters invented quantity, missing/failure or derivative semantics | Complete composition and kernel contracts, explicit WeightedMean and mandatory outcomes | Composed quantity and actual kernel/backend fixtures |
| R4-11/12 — Addressed in design | DM-15, DM-48, DM-53, DM-54, DM-59, DM-60 | §5.4/§14.2/§24.1; E5 | An empty pushed result passed the old oracle; noncanonical protobuf was presented as stable | Full values/multiplicities oracle; noncanonical diagnostic encodings outside reuse | Deliberate over-pruning negative; decoded-plan attribution/round trip |
| R4-13 — Addressed in design | DM-30, DM-35, DM-39, DM-59 | §5.3/§14.3/§18.2 | A query pool did not account for other live allocations or oversized relation construction | Shared budget, fallible platform reservations and explicit supported envelope | Separate query/canonicalization/result budgets, concurrent release and process peaks |
| Proportionality — Accepted simplification | DM-33, DM-56–DM-58 | §14.3/§24.3/§26; ADR-0041/0042 | Early fine invalidation and canonical-plan machinery added unproved semantic choices | Complete keys first; exact input equality for skipping, measured triggers for refinement | R-22/R-01 before adoption; full dependency fixtures precede timing |

**Applicability.** All twelve groups bear on these cross-boundary changes, but
this review reassesses only the principles listed below and their revised
mechanisms. Other principles and unchanged subsystems retain the earlier review's
coverage limits; they are not silently recertified or declared irrelevant.
“Satisfied” is a verdict on specified **Proposed** design, with the boundary and
rejection behavior identified above. No numerical maturity score is assigned.

| Applicable principles | Verdict | Mechanism / limiting scope |
|---|---|---|
| DM-01–DM-04 | Satisfied | Complete domain meaning; registry authority; separate physical layouts; contracted opaque bodies |
| DM-06–DM-10 | Satisfied | Typed operations/outcomes, active admission, bound domains/ports and queryable declarations |
| DM-12–DM-15 | Satisfied | Explicit model/case/output/attempt roles, coherent publication and role-specific identity |
| DM-18–DM-20 | Satisfied | Seeds before realized IDs, candidate closure and declared bindings; inspection does not construct physics |
| DM-21–DM-25 | Satisfied | Stage ownership, complete transformation contracts, traceable outputs and guarded lowering |
| DM-28–DM-30 | Satisfied | Pure kernel contract, owned workspaces, typed partial/terminal outcomes and publication boundary |
| DM-31–DM-33, DM-35 | Satisfied | Complete inputs and absence, conservative initial granularity, shared resource/ordering contracts |
| DM-36–DM-40 | Satisfied | Native layouts and safe owners for concrete consumers, coarse streams, explicit numerical policy; benefits remain hypotheses |
| DM-41–DM-44 | Satisfied | Mechanical adapters, explicit losses/conversions, actual binding requirements and unsupported refusal |
| DM-46–DM-48 | Satisfied | Fresh semantic lineage, mandatory typed failure evidence, logical reproducibility distinct from diagnostic bytes |
| DM-51–DM-54 | Satisfied | Explicit v2 refusal/migration path, generated contracts and adversarial/differential acceptance specifications |
| DM-56–DM-60 | Satisfied | Existing mechanisms with consumers, smaller initial reuse, falsifiable evidence and named change-level fixtures |

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication / locality | Risks and cost | Performance evidence | Decision |
|---|---|---|---|---|
| Revision 4 as written | Adapters/pass authors must invent missing validation, type, dependency and failure meaning | Unsupported library guarantees and premature fine reuse | No end-to-end measurement | Rejected by the earlier review |
| Revision 5 | Shared contracts invoke existing Arrow/DataFusion mechanisms; kernel bodies remain ordinary code | More explicit boundary work; bounded canonicalizer and coarse keys simplify correctness | Benefits and recomputation costs remain hypotheses | Selected for the stated architecture |
| Smaller Slice A runtime with whole-stage recompilation and only currently consumed backend bindings | Same semantic registry/IR; omit memo hits and defer every optional adapter/optimization until a consumer needs it | Extra recomputation, smaller implementation and failure surface; still requires active admission, quantity semantics and coherent publication | Measure before enabling memo/transfer refinements | Viable implementation sequence within revision 5, provided unsupported routes remain explicit |

Registry and operation generation are justified by repeated Rust/Python/storage
boundaries; typed aggregate/unnest operations have P8/membership consumers. A new
distributed service, universal inversion engine, extra dataframe system or
automatic dependency framework is not justified by these findings. Canonical
normalization, sparse numerical algorithms, masks and buffer ownership remain
ordinary typed code, not new DSLs.

## 9. Verification and measurement plan

The dependency-ordered packets and exact proposed fixtures are in
[plan 02](../../plans/02-blueprint-revision-5-contracts.md#verification).

| Claim / risk | Evidence | Check and conditions | State |
|---|---|---|---|
| Documentation consistency | **Tested**, static/governance scope only | Plan 02's exact recipe/results table; baseline zero; original headings, pass rows, governed markers, links and immutable ADR bodies | Results recorded in plan 02; no runtime inference |
| Library counterexamples | **Tested**, historical | Retained revision-4 runner/output: six groups, zero failed assertions, Rust 1.98.1, Arrow 59.3.0/DataFusion 55.1.0, dev, force validation, locked offline resolution | Not rerun for prose; does not test the new integration |
| Active admission, quantity, kernel and numerical preservation | **Proposed** | Real import/query/output/backend fixtures, including excluded branches, lossy casts and missing-versus-invalid outcomes | Required before behavioral acceptance |
| Canonicalization, identity and recovery | **Proposed** | Nested/null/metadata metamorphisms; IPC/Parquet differential; corrupt existing objects and interruption at every publication step | Required before storage acceptance |
| Complete reuse and exact scans | **Proposed** | Mutate each named semantic dependency versus clean compile; full pushed/unpruned values and multiplicities | Required before reuse/pushdown acceptance |
| Cost and resources | **Proposed** | Cold/warm construction through solve/restore; pool and process peaks, copies, spills, decoded bytes and transfer/solver time under recorded limits | No speedup or process-wide OOM immunity claimed |

No ordinary full Rust/Python/solver suite can establish unimplemented behavior.
Current runs are scoped static/governance checks; future implementation changes
must run their actual test/slice recipes. API-reference lint remains deferred
under R-20 and is not credited as interface verification.

## 10. Exceptions and unresolved decisions

No SHOULD deviation is needed for these revised contracts: whole-stage reuse is
the smallest currently trustworthy granularity, supported boundaries refuse
unavailable behavior, and optional features have explicit triggers. This review
does not close unimplemented behavioral acceptance by treating it as an exception.

| Decision outside the initial supported scope | Control / owner | Observable revisit trigger |
|---|---|---|
| Finer reuse, then salsa | R-22/R-01; paul-heyse | Complete finer input fixtures plus measured whole-stage preparation cost |
| Larger canonical relations / fixed-boundary streams | R-23; paul-heyse | Actual workload exceeds declared bounds and justifies a versioned format decision |
| Parquet pruning / bounded coalescing | R-24; paul-heyse | Each consumer independently shows full equivalence and lower total cost |
| Predicate preimages | R-25; paul-heyse | Exact supported predicate equivalence, including null/endpoint/floating cases, then measured benefit |
| Cloud/multi-writer stores and later providers | Existing register rows including R-10 | Concrete deployment/provider requirements and their capability/conformance checks |

## 11. Decision and implementation changes

**Accept the revised, bounded Proposed design contracts.** All seven gates pass
for that document scope. The review does not accept runtime behavior, every
unchanged blueprint detail, an ADR status transition or a performance claim.

| Priority | Change / next acceptance boundary | Principles | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Generate/admit complete quantity, kernel, snapshot and port contracts | DM-06–DM-08, DM-22, DM-43 | Plan 02 packets A–C; actual boundary negatives | E1/E3 counterexamples become integration fixtures |
| 1 | Implement guarded numerical outcomes and coherent identity/publication | DM-14–DM-15, DM-24, DM-30, DM-40 | Packets B/E/F; full lifecycle and backend routes | E2/E4/E6, corruption and excluded-branch regressions |
| 1 | Establish complete reuse and pushdown correctness | DM-31–DM-33, DM-53–DM-54 | Packets C/D; clean differential and full-result oracle | Domain/candidate/absence/fixed-value mutations and over-pruning negative |
| 2 | Adopt bounded standard operators and real observation imports | DM-18, DM-25, DM-41, DM-56 | Packets C/G; independent expansion/import fixtures | Null/empty/order/units/target cases |
| 3 | Refine only after measuring the complete workload | DM-39, DM-58–DM-60 | R-22–R-25 experiments under identical meaning and resource limits | Recorded conditions, equivalence and end-to-end cost |
