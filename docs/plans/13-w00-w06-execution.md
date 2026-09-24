---
title: Rust computation W00–W06 execution packet
status: done
date: 2026-09-23
adrs: [ADR-0076]
phase: 1
evidence: Tested — 82 isolated foundation units and 8 Rust governance units with force-validation; 7 Python guard units; product acceptance remains open
---

# Rust computation W00–W06 execution packet

**Historical scoped packet.** Its results and remaining-work statements describe
that execution boundary. Use the [W19 repair checkpoint](13-w19-repair-checkpoint.md)
for the current implementation, source seal and outstanding qualification.


The maintainer selected dedicated `pse-codegen` and the foundation-only W06 endpoint.
Parent scope, verification and inherited acceptance: [Plan 13](13-rust-computation-architecture.md).
Progress authority: [inventory](13-execution-inventory.md); executable cases: `13-acceptance-cases.toml`.

## Dependency order

W00 → W01 → W02 → W03 → W04 → W05; W06 depends on W03.

## W00 — authority and assurance

ADR-0076 reconciles backend placement, three crate additions, Salsa, identity and graph contracts.
Active plan is declared once in workspace metadata. Phase guards read that declaration and prohibit
historical-plan bypass. Preserve all Plan 10/11 identities and failed/unrun/unqualified evidence.
Full implementation/deletion seal belongs to W18, never this checkpoint.

## W01 — exact library profile

Qualify Salsa 0.28.4 (macros, inventory, salsa_unstable; no defaults), petgraph 0.8.3,
rustworkx-core 0.18.1 and rust-igraph 0.7.0 together. Preserve Arrow 59.3.0, DataFusion 55.1.0,
object_store 0.13.2, Tokio 1.53.1 and existing Delta/kernel revisions. Record actual feature
unification, including rustworkx's mandatory Rayon and petgraph defaults. Compile the used APIs;
licence eligibility is not combined qualification. Enforce resolved normal dependency ceilings.

## W02 — representation and ownership

- `pse-ids`: identity/framing/scalar semantics only; `pse-diagnostics`: typed vocabulary only.
- `pse-columnar`: native canonicalization, buffers, reservations, cancellation and engine errors.
- `pse-model`: registry-generated plain rows/enums/shared payloads, no native library closure.
- `pse-codegen`: generator API and syn/quote/prettyplease; no generated-output bootstrap dependency.
- Move handwritten operator/reference contracts to MathIR; registry projects those declarations.
- Templates consume model values and MathIR contracts. Core work checks use synchronous callbacks.
- Share RelationRow/FieldCheckedBatch mechanics; preserve full schema/value checks and owner identity.
- Split plain values from Arrow codecs; serde follows actual DTO consumers; deduplicate only equal meanings.
- Pure generation through xtask without default features; missing generated roots fail checks.

## W03 — admission and equality

Admitted immutable change sets distinguish definition, specialization and instance identity.
Set/bag/sequence equality is explicit. Preserve signed zero, operand roles/order/repetition, quantity,
policy/provider choices, absence membership and many-to-many source mappings. Source provenance does
not invalidate every numerical body. Stable implementation identity includes actual dirty source,
locks, build configuration and registry/operator/kernel ABI; no pointers, UUIDs or Salsa IDs.
Named boundaries own local/relational/graph/backend/publication validation. Checked Arrow take/filter,
strict cast and shared slice/project owners preserve only proved certificates. Record RCA §9 S1–S9 contracts.

## W04 — Salsa coordinator

Stable entity-to-input map; compare before setters; track fields and membership separately.
Default to plain owned comparable results; canonical tracked producers only where useful.
Runtime prevalidates updates, closes admission, cancels every active handle, drains and applies all
setters exclusively. Publish one application revision. On failure rebuild the last admitted state.
Fresh per-job clones and per-handle cancellation; no idle clones or Salsa borrows across await.
Bound input/payload/job/query/graph/generation lifetimes; LRU is not a metadata bound. Cancellation,
resource refusal and IO failure are not memoized semantic failures. Fresh database is the unit oracle.

## W05 — relational phase handoff

Move current native compiler implementation mechanically to `pse-runtime::compiler_driver`; update
callers directly, no compatibility facade. Pure requests plus revision-qualified submission envelopes
execute outside Salsa. Stream and validate all phase outputs, then install the entire phase once.
Reject stale completions; bound in-flight deduplication. Preserve sessions/functions/native caches,
exact immutable Delta views and selected MemTable immutability. Blocking jobs retain charge until exit.
Retained wrappers: W07 traversal, W08 rules, W09 MathIR, W10 structural, W11 numerics; W15 reconciliation.

## W06 — graph projections

Canonical immutable petgraph Graph with checked u32 bounds, isolates, labelled parallel edges and IDs.
Typed package/template/containment/port/expression/rule/kernel/unit/incidence contracts; complete scope
required for whole-problem analysis. Expression traversal uses existing ordered MathIR adjacency.
Use iterative kosaraju_scc, Dfs/DfsPostOrder/Bfs/Reversed, rustworkx lexicographical order and cycle
witnesses. Condensation uses a unit-weight quotient and retains typed original edge provenance.
Recompute changed regions; no dynamic graph engine or unnamed all-pairs closure. Qualify rust-igraph
conversion now; production maximum matching, DM and block analysis belong to W10.

## Verification

Baseline zero. Select isolated units with explicit `pse-relations/force-validate`; static checks,
locked compilation and pure regeneration are allowed. No compiler/storage/solver/Python component
journeys or performance campaign until W18. Case receipts distinguish Implemented, Tested and Measured.

Executed on 2026-09-23 with the pinned Rust 1.98.1 toolchain. The final source/log digests
and command results are in local `build/plan13/final-verification.json`; the source snapshot
is `build/plan13/final-source.json`. These are development receipts, **not** a W18 seal.

| Command | Mode and evidence | Result, baseline zero |
|---|---|---|
| `just unit-rust-foundations` | Tested; nextest default profile, isolated in-memory units, explicit `pse-relations/force-validate` | 82 passed, 0 failed; 226 deliberately unselected |
| `just unit-rust-foundations-governance` | Tested; 7 Python unittest controls plus 8 Rust static/generation units, explicit force-validation | 15 passed, 0 failed |
| `just check` | Interface-checked; locked workspace, all targets, dev profile; compile only | exit 0 |
| `just clippy-rust-foundations` | Interface-checked; all targets of the foundation and moved native owners, force-validation, `-D warnings` | exit 0 |
| `just family-check` | Interface-checked; actual resolved normal dependency closures and family versions | exit 0 |
| `just codegen-rust-contracts-check` | Tested; pure generation, no xtask default features; isolated Git-index profile below | exact regeneration, exit 0 |
| `just fmt-rust-check`; `just lint-toml` | Interface-checked; pinned formatting tools, complete configured scope | exit 0 |
| `.venv/bin/ruff check scripts/implementation_phase.py scripts/tests/test_implementation_phase.py` | Interface-checked; scoped Python tooling lint | exit 0 |
| `just adr-lint`; `just docs` | Interface-checked; decision metadata/index/register and documentation build | exit 0 |

The ordinary strict codegen recipe rejects untracked generated files. Its byte comparison
passed; the new model/runtime roots and native `facts.rs` were then marked intent-to-add
in a **temporary copy** of the Git index for the successful strict check. The real index
was unchanged. Those files must be included when the change is staged. The Rust governance
regeneration unit independently compared every generated language/root with fresh output.

Cargo still reports the existing upstream future-incompatibility notice for
`proc-macro-error2 2.0.1`; no unrelated library upgrade was made to suppress it. mdBook
reports its existing large search-index notice. Startup doctor reports the Python environment
and editable extension stale against the changed lockfile. No Python/native product,
solver, integration or performance qualification is claimed; those remain behind W18.

## Outcome

### What was built

**Implemented and Tested:** W00–W06 foundations and the L02 vocabulary/native dependency
extraction. Dedicated `pse-model`, `pse-columnar` and `pse-codegen` crates remove native
engines and generator machinery from the semantic core. The native compiler and callers
now use `pse-runtime::compiler_driver`; `pse-compiler` owns typed admission, stable entity
inputs and Salsa queries. Shared generated builders/codecs preserve registry contracts;
equivalent expression payload declarations share values and serde follows actual consumers.

The coordinator performs cancel/drain/install with last-complete-state recovery and
generation ownership. The relational driver installs complete bounded phase inventories
once, rejects stale completions and preserves actual session/provider owners. Typed graph
adapters cover all nine declared families using petgraph, rustworkx and rust-igraph.
[Stage contracts](13-stage-contracts.md) record equality, completeness, cycles, effects,
cost and ownership; the [library profile](13-foundation-library-profile.md) records exact
features, used APIs and the resolved dependency moves.

The manifest retains 179 carried Plan 10 cases, 18 carried deletion mappings, all 75
namespaced Plan 11 case groups and all 16 unqualified acceptance entries. Nine new
foundation case groups carry scoped development evidence. No historical failed, unrun
or unqualified acceptance was converted into success.

### A mistake made and corrected

The initial graph wrapper assumed rustworkx topological ordering rejected a cycle.
The isolated tests exposed its partial-result behavior. The adapter now checks full
vertex coverage and returns an actual cycle witness, including disconnected self-loops.
The crate moves also exposed stale benchmark/test paths and an invalid mutable-MemTable
fixture; direct callers were moved, and the fixture now separately proves mutable-source
refusal and changed-witness invalidation.

### Deviations from the plan, deliberate

The maintainer selected dedicated `pse-codegen` and the foundation-only W06 endpoint.
Still-used native pass wrappers moved mechanically; their semantic replacements remain
W07–W11 and final consumer/deletion reconciliation remains W15. No compatibility facade
was retained. The full generated contract mirror remains because it still performs
executable schema/metadata checks; it was not replaced by a fingerprint-only shortcut.

The unexpanded specialization key does not certify a bound instance body; W07/W09 add
the full binding environment. Salsa cardinality/lifetime controls and its memory report
are implemented; complete aggregate accounting and allocator/RSS qualification remain W14.
No mutable/index-hole graph adapter or dynamic graph engine was introduced. Production
matching/DM/block analysis remains W10. ADR-0076 remains proposed for the formal decision
PR. W18, W19 and W20 are open and no implementation barrier was sealed.
