# pse-arrow binding

The repository layer of the design standard. It defines no principles; it maps the
[core principles](../core/design-principles.md) and the
[process-simulator profile](../profiles/process-simulator/principles.md) onto this
repository's authorities, commands, routes and local policies. Where this page and a cited
authority disagree, the cited authority wins and this page is corrected.

## Standard applied

| Layer | Document | Version |
|---|---|---|
| Core | [design principles](../core/design-principles.md), [review template](../core/design-review-template.md) | 2.0 |
| Profile | [process-simulator principles](../profiles/process-simulator/principles.md), [review additions](../profiles/process-simulator/review.md) | 1.0 |
| Binding | this page | — |

The same declaration is in [`standard.toml`](../standard.toml) for agents and skills.

## Reviews in this repository

- **Location:** `docs/design_review/reviews/design_review_{slug}_{YYYY-MM-DD}.md`; supporting
  evidence files under `docs/design_review/evidence/`.
- **Skills:** `design-review` (core method) with `design-review-process-simulator` (profile
  lenses). The `design-reviewer` agent applies both with this binding; the
  `library-leverage-reviewer` agent can assist with slot 8.
- **Default purpose:** design reviews default to **target** purpose (template, *Purpose*): the
  aim is the best-in-class simulator design, and blueprint, ADR, plan or policy text that blocks
  it is recorded in slot 11 as a required change with its route. Change reviews within an
  approved plan default to **conformance**.
- **Reviews are evidence, not authority.** Their findings take effect through a plan, an ADR or
  a `design:` PR.

## Where profile roles are decided

| Role (principle) | Authority in this repository |
|---|---|
| Model authority and identity (DP-01, DP-04, DP-05) | blueprint §2 (D1–D14) and §5, as amended by Plan 14 |
| Ownership of math, derivatives, sparse algebra, properties, structure and solvers (DP-13, DP-17, PS-07, PS-09) | [Plan 14](../../../plans/14-library-owned-process-simulator.md) D02 (crate ownership) and D03 (library and execution decisions); blueprint §1.3 and §3.3 |
| Supported math, provider and solver scope (DP-15, PS-02, PS-10) | [Plan 14 foundation contract](../../../plans/14-math-foundation-contract.md) and ADR-0082–0084 |
| Library capability evidence (DP-15) | `docs/capability-maps/` (`just lib-outline <file>` first) and the library skills: `symbolica-faer-oximo`, `native-solver-libraries`, `salsa`, `rust-graphs`, `datafusion`, `deltalake`, `library-research` |
| Error taxonomy (DP-21) | blueprint §23.2; every `pub enum *Error` derives `thiserror::Error` and implements `miette::Diagnostic` (AGENTS.md *Invariants*) |
| Dependency admission (DP-13) | [dependency policy](../../../dev/dependency-policy.md) and ADR-0066: no library or licence is refused in phases 0–1, so licence is never a reason to reject a §F candidate |
| Reference validation (PS-13) | IDAES parity against `idaes-pse==2.12.0` as a reference oracle, clean-room: read for behaviour, never copied ([relationship to IDAES](../../../relationship-to-idaes.md)) |

## Local policies that tighten the standard

Each is stated once in its authority; this list only points to it.

| Policy | Tightens | Authority |
|---|---|---|
| The baseline is zero: no lint finding, failing test or warning is tolerated at plan close | DP-23 | AGENTS.md prime directive 1 |
| Evidence labels (principles §D) are mandatory in ADR `evidence:` fields, PR descriptions and a plan's Verification and Outcome at close | DP-22 | AGENTS.md prime directive 4 |
| Never edit generated paths; fix the generator | DP-01 | AGENTS.md prime directive 2 |
| One resolved version per pinned dependency family | DP-15 | AGENTS.md *Invariants*; `just family-check` |
| `force_validate` in every test run; `panic = "unwind"`; never `target-cpu=native` | DP-03, DP-19, DP-11 | AGENTS.md *Invariants* |
| Delete replaced code, callers, tests and fixtures in the same change; no shims | DP-16 | AGENTS.md *Execution rhythm* |
| No library capability is withheld for lack of a consumer | DP-13, DP-16 | blueprint §3.3.1; ADR-0065 |
| Design alignment is established by agent judgment and review; probes, tests and written records are used where uncertainty warrants, not as proof of diligence, and there is no dedicated alignment tooling | DP-23 | this binding (maintainer decision, 2026-09-24) |

## Commands for evidence

| Evidence | Command |
|---|---|
| Targeted behaviour (*Tested*) | `just unit-package <pkg> <filter>` |
| Compilation of touched code | `just check-package <pkg>`, `just check` |
| Library fit where doubt remains (DP-15) | capability maps (`just lib-outline`), the library skills and their probes |
| Dependency families (DP-15) | `just family-check` |
| Reference validation (PS-13) | `just parity`, `just parity-container` |
| Plan-close qualification | the AGENTS.md *Verifying work* table |

## Routes for required changes

| Change | Route |
|---|---|
| Alters D1–D14, adds or removes a crate, governance, SHOULD deviation | ADR plus design review (AGENTS.md *When an ADR is required*) |
| Blueprint text | `design:` PR with a revision row (`PSE_DESIGN_EDIT=1`) |
| Deferred decision with a trigger | register row in `docs/adr/register.md` |
| Sequencing or scope | the active plan in `docs/plans/` |

## Known conflicts

| # | Standard says | Repository text says | Until resolved |
|---|---|---|---|
| K1 | Library-first math, derivatives, properties and solvers (DP-13, PS-07, PS-09) | Blueprint D6, D9, D10–D12, D14 and §7 predate Plan 14 (custom math IR, Pyomo backend, relational math transport) | Follow Plan 14 D02–D04 and ADR-0082–0084; the amendments are listed in Plan 14 D04 |
| K2 | Prefer runtime and library mechanisms over generated projections (DP-16) | Blueprint §4.2 and D9 prescribe schema-registry generation of adapters | Existing surviving generated contracts stand (Plan 14 D02). A new generator states why no runtime mechanism serves |
| K3 | Principle IDs `DP-nn`, `PS-nn`; evidence vocabulary in principles §D; exceptions in §H | Accepted ADRs and earlier reviews cite `DM-nn`, "charter §D" and "charter §H" | Read through principles §I and the RCA lineage below; accepted records are not edited |

## Where defect shapes tend to land in this stack

Illustrative places the shapes in the `design-review` skill's reference have landed before.
Their absence proves nothing.

| Pattern | Shape | Principles |
|---|---|---|
| `Schema` / `Field` built in more than one module for the same logical table | Second authority | DP-01 |
| `DataType` matched exhaustively in several places, each with its own coercions | Second authority, silent degradation | DP-01, DP-15 |
| Sentinel floats (`NaN`, `-1`, `f64::MAX`) where a typed status or null is the meaning | Silent degradation | DP-02 |
| `unwrap` / `expect` / `unwrap_or_default` at a deserialization or FFI boundary | Unguarded boundary | DP-03, DP-15 |
| UDF `signature()` / `return_type()` wider than the kernel handles; `supports_filters_pushdown` returning `Exact` for an approximate filter | Unbacked capability | DP-15, DP-08 |
| Plan or `ExecutionPlan` equality as a cache key without configuration or provider version | Incomplete reuse key | DP-09 |
| Schema metadata (units, domains, provenance) dropped by `project`, `cast` or conversion | Silent degradation | DP-15, DP-21 |
| Interior mutability inside a path documented as pure planning or inspection | Hidden effect | DP-18 |
| A tracked function reading an untracked field or an input through a non-tracked path (Salsa) | Handle mistaken for version | DP-09 |
| Interned or tracked IDs held across revisions; untracked struct fields changed between revisions (Salsa) | Unsound equality, incomplete reuse key | DP-04, DP-09 |
| `NodeIndex` persisted, or results indexed by `node_indices()` after removals (petgraph) | Index-space bridge | DP-04 |
| `UnGraph` / `into_undirected` on directed relationships; `GraphMap` where parallel edges carry meaning | Silent graph reinterpretation | DP-07 |
| Two majors of a graph library in `Cargo.lock` through a backend adapter | Version bridge | DP-15 |
| A whole-graph algorithm behind a DataFusion operator that claims a partitioning or exact pushdown | Global computation per partition | DP-08 |
| Evaluator, property or solver workspace shared across attempts or held inside a tracked query | Owned-workspace violation | DP-19, DP-18 |
| Solver return code mapped to success without the post-solve check | Status read as success | PS-10 |

## Lineage: Rust computation architecture guidelines (RCA)

The superseded RCA document's general requirements now live in the core; its library-specific
material belongs to the library skills.

| RCA | Now |
|---|---|
| §1 Backend selection and reuse boundary | DP-13 (placement), DP-09 (one reuse mechanism); Salsa specifics in the `salsa` skill |
| §2 Semantics across representations; definition/specialization/instance/artifact | DP-01, DP-04, DP-06 |
| §3 Dependency completeness, input ownership, derived-entity ownership | DP-09, DP-18, DP-04; Salsa specifics in the `salsa` skill |
| §4 Query boundaries, sound equality, reproducibility | DP-09, DP-10, DP-11 |
| §5 Relational fusion and graph projection; §5.1 specs; §5.3 heuristics | DP-13, DP-10, DP-07 |
| §5.2 Graph backend selection, version compatibility, index mapping, algorithm identity | DP-15, DP-04; catalog and probes in the `rust-graphs` skill |
| §5.4 Storage versions versus temporal semantics | DP-04, DP-11; Delta specifics in the `deltalake` skill |
| §6 Recursion | DP-12 |
| §7 Compilation, execution and publication | DP-19, DP-09, DP-18 |
| §8 Persistence, memory and invalidation scope | DP-19, DP-20 |
| §9 Implementation acceptance contract | Template slot 4 (with the profile's numerical columns) |
