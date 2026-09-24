---
title: Rust computation W07–W11 execution packet
status: done
date: 2026-09-23
adrs: [ADR-0076, ADR-0077, ADR-0078, ADR-0079, ADR-0080]
phase: 1
evidence: Tested — 32 isolated computation units, one linked callback unit and seven phase-guard units; product acceptance remains open
---

# Rust computation W07–W11 execution

**Historical scoped packet.** Its results and remaining-work statements describe
that execution boundary. Use the [W19 repair checkpoint](13-w19-repair-checkpoint.md)
for the current implementation, source seal and outstanding qualification.


Implements [Plan 13](13-rust-computation-architecture.md) after the completed
[foundation checkpoint](13-w00-w06-execution.md). The maintainer selected derived
compiler rule schedules instead of retaining manually numbered reference strata.
The shared starting tree is captured in `build/plan13/w07-w11/starting-source.json`.
No prior acceptance receipt is withdrawn or relabelled.

## Ownership and dependency order

W07 → W08 → W09 → W10 → W11. Pure shared graph algorithms may be implemented before
their compiler consumers; that does not close the consuming package.

| Boundary | Owner | Contract |
|---|---|---|
| Typed documents, inventory, scoped resolution | authoring/model/compiler | Registry values, exact membership, spans and bounded inputs |
| Specialization and occurrence binding | compiler/templates | Shared structural body; distinct actual bindings and values |
| Checked relational program | rules/runtime | Complete read/effect graph, derived schedule, limits and atomic completion |
| Rule schedule and structural analyses | structural | Pure complete projections and deterministic library results |
| Immutable semantic body and numerical topology | MathIR/compiler | Typed DAG/formal slots; instruction ABI, derivatives and source maps |
| Evaluators and attempt workspace | numerics/backends/runtime | Shared immutable program; live per-attempt state, resources and cancellation |

## W07 — typed authoring, resolution and topology

1. Preserve parsers, strict document decoding, source spans and identity policies.
   Generate document values against pse-model; move generated Arrow construction
   and native authoring adapters to runtime. Move callers, then delete old APIs.
2. Admit needed region inventories once. Index exact qualified names, packages,
   definitions, symbols, configuration, providers, domains, instances and support.
   Replace the partial configuration Expr interpreter with typed domain lookups;
   genuinely relational combinations retain bounded native joins.
3. Extend Salsa with resolved interfaces/references, configured definitions,
   specialization environments, instance layouts and separately requested sources.
   Failed lookups read scope membership; updates preserve handles and tombstones.
   Owned Arc results use returns(clone), semantic equality and compare-before-set.
4. Split structural specialization from occurrence bindings and runtime values.
   Keys include every consumed type/quantity/shape/domain/provider/compile-time
   value. Reject unsupported recursive expansion with a witness; never truncate.
5. Use W06 adjacency/order projections. Derive containment parent/depth/DFS intervals
   for ancestor membership. Preserve crossing edges and proved region independence.
   Move scope-rule closure consumers to requested traversal. Retain bounded all-pairs
   emission only for supported publication/inspection products.

Exit: repeated definitions/diamonds/renames/missing references/cycles and unrelated
edits are isolated units; bulk inventory query counts do not scale per instance.
L05 and W07-owned L06 deletions require caller inventories.

## W08 — checked dependencies and derived rule schedules

1. Remove stratum from RuleDecl and reference rule/dependency declarations; remove
   numeric constructor arguments. Declare compiled.rule_strata with program/rule
   identity and component/stratum ordinals; regenerate all consumers.
2. Split native preparation into binding, complete footprint/effect extraction,
   graph/schedule derivation, recursive-policy validation and reusable execution.
   Traverse subqueries using DataFusion visitors. Account for anti-joins, negation,
   aggregation, windows, limits, outer joins, support and conflict/undecided reads.
   Unknown extension effects fail unless explicitly declared. Check all read ports.
3. Derive SCCs from every selected producer and all writers of demanded heads.
   Reject settlement-sensitive SCC edges with a witness. Condense positive SCCs;
   positive edges have zero stratum increment, settled edges a strict increment.
   Semantic IDs break order ties. Generated registry assembly stays runtime-free.
4. Retain native rounds, epochs, semi-naive deltas, representatives, support and
   conflict handling. One complete program/region W05 request settles its strata
   internally and returns one completion. Failed/partial/stale work is not cached.
   Changes to support/representatives count even when head cardinality is unchanged.
   Enforce round/resource/cancellation bounds and retire final-reader temporaries.

Exit: complete read/effect checking and derived schedules replace manual strata;
units cover positive and prohibited cycles, duplicates/empty/negative inputs,
representative/support changes and exhaustion. L07 closes only after all callers move.

## W09 — shared semantic MathIR

1. Build mutable graphs locally and publish owned immutable semantic stage results.
   Carry roots/equations/kernel bindings/implicit systems/quantity and binder facts.
   Separate topology from source annotations, correspondence and runtime bindings.
2. Compile structurally equal specializations once with explicit internal formal slots.
   Resolve slots before persisted emission. Share bodies while retaining actual
   occurrence outputs and independent runtime state.
3. Convert P7–P10 to typed realization, expansion, binding and canonicalization.
   Decode imported graphs once. Infer quantity/binders once per relevant result;
   preserve ordered/repeated operands, guards, complete physical keys and value
   dependencies of folding. Use existing graph visitors, not another graph copy.
4. Generate one publication/inspection projection. Preserve canonical wire bytes
   and IDs. Delete whole-graph reloads and duplicate family dispatch. Backend
   interning remains derived and must retain its semantic-node correspondence.

Exit: no mandatory full-graph encode/decode between semantic passes; publication,
quantity, guard, metadata and shared-body/separate-binding controls pass. Track the
W11-owned remainder of L08 separately.

## W10 — exact incidence, matching, DM and blocks

1. Admit complete case-bound active equalities and free variables, with isolates.
   Preserve separate DOF/fixed/unused participation meaning. Record value assumptions
   behind zero-elision. Canonicalize typed ID maps and deduplicate presence edges.
2. Use rust-igraph maximum_bipartite_matching and is_matching. Check cardinality,
   partners, partitions and unmatched states. Budget conversion and internal
   adjacency, including validator workspace. Version ordering/algorithm identity.
3. Orient unmatched edges variable→equation and matched edges equation→variable.
   Forward reachability from unmatched variables gives underdetermined membership;
   reverse reachability from unmatched equations gives overdetermined membership.
   Preserve unmatched subcategories and remaining square membership. Verify the
   alternating-reachability vertex cover covers all edges and has matching size.
4. Build matched-pair dependency edges prerequisite→dependent. Use iterative
   kosaraju_scc, condensation and checked lexicographical topological sorting.
   Square blocks require perfect matching; rectangular regions remain explicit.
   Whole-problem block solving refuses unmet perfect-matching preconditions.

Exit: exhaustive graphs with 0–4 nodes on each side, brute-force matching oracle,
DM coverage/disjointness, block order, isolates, rectangles, duplicate incidence,
multiple maxima and incomplete/global/cross-region controls. Larger independent
SciPy/Pyomo reference comparison remains W19. L09 follows actual implementation.

## W11 — direct numerical lowering

1. Put pure NumericalTopology in MathIR: typed instructions, slot ABI, roots/masks,
   derivative coordinates, guards, source maps and exact capability identities.
   Compiler Salsa queries never capture pools, sessions or mutable solve state.
2. Lower MathIR directly into the existing guarded scalar instruction model.
   Refactor sparse differentiation around typed operands/operator contracts.
   Preserve quantity/scalarization and zero-exponent/parameter-only/guard behavior.
   Keep PhysicalExpr for supported genuine batch evaluation; no scalar fallback
   through one-row Arrow or an independently authoritative Expr arena.
3. Keep branch evaluation demand-driven. Invalidate parameter/variable-dependent
   slots transitively on refresh. Values consumed during compilation invalidate
   topology. Preserve finiteness, codes, masks, capability and cancellation checks.
4. Move native solver preparation to a shared prepared program and typed bindings.
   Each attempt owns workspace, initial values/bounds/scaling/warm starts, live
   cancellation/resources/permits and results. Migrate callers before deletion.
   NL/Pyomo stubs remain explicitly unsupported; no new solver/Hessian claim.

Exit: scalar/batch/oracle/derivative/guard/refresh/slot/capability units pass, independent
workspaces do not alias, callback planning/encoding counters stay zero. Close L10 and
the completed W11 portion of L08. Full solver journeys are authored for W19.

## Verification

Baseline: zero failures. Each receipt names command, explicit force-validation mode,
source identity, exact selected tests, result count and conditions. Add a scoped
unit-rust-computation recipe for these units. Run affected compilation, scoped Clippy,
family/dependency ceilings, formatting, pure generation and ADR/document checks.

**Tested:** repository test `incidence::computation_unit::exhaustive_four_by_four_matching_dm_certificates_and_block_order`
now replaces the temporary planning probe. It enumerates all 74,963 labelled bipartite
graphs with 0–4 nodes on either side and checks independent matching cardinality,
reciprocal partners, DM coverage/disjointness, vertex-cover certificates and block order.

The final pinned-toolchain commands and source/log SHA256s are captured locally in
`build/plan13/w07-w11/final-verification.json` and `final-source.json`. These are scoped
development receipts, not an implementation barrier seal. Failed exploratory receipts
remain beside the successful final logs rather than being overwritten.

| Command | Evidence and conditions | Result, baseline zero |
|---|---|---|
| `just unit-rust-computation` | Tested; nextest default profile, isolated in-memory library units, `pse-relations/force-validate` | 32 passed, 0 failed; 177 deliberately unselected |
| `just unit-native-callbacks-solver` | Tested; isolated unwind barrier, `pse-backend-native/ipopt,pse-relations/force-validate`, pinned linked interface, no solver invocation | 1 passed, 0 failed; 1 deliberately unselected |
| `.venv/bin/python -m unittest scripts.tests.test_implementation_phase` | Tested; phase-guard/manifest tooling only | 7 passed, 0 failed |
| `just check` | Interface-checked; locked workspace/all targets, dev profile, compile only | exit 0 |
| `just clippy-rust-foundations`; `just lint-native-data-solver` | Interface-checked; affected owners/all targets, explicit force-validation and `-D warnings`; solver feature in second command | exit 0 |
| `just family-check` | Interface-checked; locked resolved family versions and normal dependency ceilings | exit 0 |
| `just codegen-rust-contracts-check`; `just codegen-python-check`; `just codegen-docs-check` | Tested; pure generators, no compiler workflow; temporary Git-index condition below | exit 0 |
| `just fmt-rust-check`; `just lint-toml`; `just adr-lint`; `just docs` | Interface-checked; pinned formatters, decision metadata/index/register and documentation build | exit 0 |
| `just py-sync`; `just doctor` | Interface-checked; refresh/check editable native extension and environment, no product tests | exit 0 |

Strict regeneration uses a temporary copy of the Git index with intent-to-add for the
465 new generated paths. This permits the existing strict tracked-tree check while
leaving the real staging area unchanged. Rust, Python and Markdown bytes are compared
against fresh registry generation. Newly generated files must be staged with the work.

`13-acceptance-cases.toml` adds 26 implemented W07–W11 case groups and preserves every
inherited declaration. Deterministic controls include 1/10/1,000 repeats, scoped Salsa
execution events, exact canonical bytes, explicit graph-boundary counts and native
planning/batch positive controls around scalar evaluation. No timing claim follows
from these counters. W19/W20 product and performance acceptance remain **not_run**.

## Completion boundary

No pin upgrade, new graph runtime, Salsa persistence, faer integration, solver algorithm
or Hessian support. Existing exact Delta providers/witnesses and publication completeness
remain; W12–W15 own their later changes. W14 owns aggregate resource qualification.
W07–W11 completion does not close shared L04/L14 reconciliation, W18, W19 or W20.

## Outcome

**Implemented and Tested:** W07–W11 and L05–L10 are complete at their isolated
implementation boundary. All named final verification commands exit 0. The parent
plan retains W12–W20 and the shared deletion/reconciliation owners.

### Implemented library contracts

**Interface-checked and Tested:** pins are unchanged. These are the used APIs and their
local qualification, not claims about newer library releases.

| Library | Built-in responsibility | Application boundary and evidence |
|---|---|---|
| Salsa 0.28.4 | Input setters, tracked queries, `returns(clone)`, equality backdating, cancellation and execution events | Stable compiler handles and complete specialization requests; 1/10/1,000 repeated occurrences share a body while preserving actual bindings; missing-name membership and consumed-value controls |
| petgraph 0.8.3 | Iterative `kosaraju_scc`, condensation, DFS/BFS and reversed reachability | Whole admitted projections retain isolates, typed endpoints and edge sources; forest intervals, derived rule schedules, alternating DM and matched-pair block graphs |
| rustworkx-core 0.18.1 | Checked lexicographical topological ordering | Semantic-ID tie order for complete quotient graphs; cyclic inputs refuse with witnesses |
| rust-igraph 0.7.0 | `maximum_bipartite_matching` and `is_matching` | Exact dense bipartitions, reciprocal/edge/cardinality validation, reserved conversion/validator storage; exhaustive independent oracle over 74,963 graphs |
| DataFusion 55.1.0 | Complete plan/subquery visitors, provider replacement transforms, native joins/aggregates/unions and physical batch expressions | Bind rule SQL once; derive all read/effect edges; rebind current providers under exact schemas; retain genuine set-oriented inference and batch numerics |
| Arrow 59.3.0 | Declared batch construction, typed arrays and boundary projections | Checked inventories retain actual row tokens; mathematical graphs cross an Arrow boundary only at import or requested projection; exact canonical preimages compared |
| Delta/kernel pinned workspace revisions | Existing exact immutable providers and version witnesses | No Delta contract change in this scope. Selected durability/CDF work remains W12/W13; W07–W11 does not introduce another persistence authority |

The Salsa/rust-graphs/DataFusion/Delta skill routes supplied the capability contracts.
Context7 discovery for DataFusion subquery traversal and volatility was checked against
local pinned source; current discovery did not replace exact-release compilation.

### Deletion and caller receipts

**Implemented:** P0 DTO/parsing/semantic resolution lives in `pse-authoring`; generated
Arrow adapters and native entry points live in `pse-runtime::authoring_driver`.
All repository callers, including xtask and authoring tests, use that boundary.
P3's partial `Expr` interpreter is deleted. Typed inventory indexes retain duplicate
occurrences and absent keys; prospective child queues use parent/name indexes.
P4/P5 owner/source indexes and P9 selected-descendant traversal use complete inventories.

P5 port walking admits six complete typed inventories and executes one enabled-root
projection before a pure named-child traversal. `inferred.port_walks` preserves ordered,
possibly repeated domain axes, actual source-row tokens and complete read scopes.
There is no DataFusion execution inside the path step/child loops. The P5 port/member
joins and P6 ordered coordinate products remain native relational work: they compute
required set products rather than graph reachability. These deliberate retained
operations do not claim a depth-independent count for every relational product.

Six recursive containment/scope rules and their SQL files are removed. Rule membership
uses complete DFS intervals, with strict descendant semantics and direct-owner ports.
Bounded all-pairs reachability/tree/scope outputs remain supported inspection products;
no rule depends on their closure. Parent walks used to enumerate actual source witnesses
remain bounded by the admitted forest.

Manual `RuleDecl`/`RuleSpec`/reference strata are deleted, including benchmark and test
constructors. Checked native plans now derive SCCs and settlement boundaries and emit
`compiled.rule_strata`. Provider transforms reuse bound SQL while checking exact schemas.
Support and representative changes remain convergence inputs; exhaustion returns no
complete program. Reference generated contracts were regenerated from the registry.

The four public mathematical endpoints select one typed region ending at P7, P8, P9
or P10. Internal producer declarations remain for provenance and stage contracts.
P8–P10 graph reloads and public per-pass Arrow algorithm wrappers are removed. P7
retains source import correspondence; P8 operates on existing typed node ordinals.
Stage outputs preserve exact predecessor sources under explicit roles. The final
endpoint projects inferred graphs once and P10 additionally emits canonical compiled
math. Backend slot interning remains derived from typed semantic instructions.

The structural stub is replaced by exact complete-case incidence, library matching,
alternating DM/cover certificates and matched-pair block order. The independent tiny
oracle covers all bipartite graphs with 0–4 vertices per side. Structural rank is not
numerical rank, and rectangular systems do not pass whole-problem square-block admission.

The scalar MathIR-to-DataFusion `Expr` lowering arena and one-row callback fallback are
removed. `NumericalTopology` owns the instruction/derivative ABI; `PreparedProblem`
shares immutable topology and actual column bindings. Attempts retain separate values,
bounds, scaling, cancellation and workspace. The remaining `Expr` arena belongs to
explicit genuine batch evaluation. It is neither semantic identity nor solver callback
state. Linked callback units contain panics without calling Ipopt.

### Mistakes corrected

The initial deletion pass missed the P5 port frontier; its replacement and ADR-0080 are
part of this package. Converting P8 to typed graphs also exposed obsolete row-to-node
mapping lookups; roots now use validated typed ordinals. Force-validation exposed a
previously unbound SQL-check fixture; the fixture now binds the engine validation
context. Workspace compilation found two benchmark rule constructors still passing a
manual stratum; both callers now use the derived-schedule API.

### Deliberate remaining boundaries

Per-invocation specializations are shared now. Cross-invocation public compiler
orchestration and selected case consumers are W15; persistent artifacts are W12 and
aggregate resource qualification is W14. W17 reconciles complete executable acceptance
coverage after those consumers exist. W19 must compare cold/reused compiler outputs,
cross-stratum edits, larger independent structural references, publication/backend
round trips and independent solves. Existing solver fixtures use typed prepared
programs and are compiled, but are not executed here. W20 alone measures timing/RSS.
Formal acceptance of proposed ADR-0077/0078/0079/0080 remains a decision/design PR duty.


The retained upstream/tooling notices are unchanged: Cargo reports future
incompatibility in `proc-macro-error2 2.0.1`, and mdBook reports its large search index.
Neither is a workspace Rust diagnostic. No dependency upgrade was introduced solely
to suppress those notices.
