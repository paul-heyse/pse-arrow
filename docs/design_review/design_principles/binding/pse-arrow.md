# pse-arrow binding

The repository layer of the design standard. It defines no principles; it maps the
[core principles](../core/design-principles.md) and the
[process-simulator profile](../profiles/process-simulator/principles.md) onto this
repository's authorities, commands, routes and local policies. Where this page and a cited
authority disagree, the cited authority wins and this page is corrected.

## Standard applied

[`standard.toml`](../standard.toml) owns the selected versions and paths. Read the core
principles/template and applicable profile from that declaration; this binding does not copy
the version values. The core's version-transition table preserves historical DP/G references.

## Reviews in this repository

- **Location:** `docs/design_review/reviews/design_review_{slug}_{YYYY-MM-DD}.md`; supporting
  evidence files under `docs/design_review/evidence/`.
- **Skills:** `design-review` (core method) with `design-review-process-simulator` (profile
  lenses). The `design-reviewer` agent applies both with this binding; the
  `library-leverage-reviewer` agent can assist with slot 8.
- **Default purpose:** design reviews default to **target** purpose (template, *Tier and purpose*): the
  aim is the best-in-class simulator design, and blueprint, ADR, plan or policy text that blocks
  it is recorded in slot 11 as a required change with its route. Change reviews within an
  approved plan default to **conformance**.
- **Reviews are evidence, not authority.** Their findings take effect through a plan, an ADR or
  a `design:` PR.
- **Lifecycle:** a review stays in the tree while an open finding or a pending decision depends
  on it. Once findings are dispositioned and enduring rationale has its owner, it retires to Git
  history; retained ADRs cite it as `git:<commit>:<path>` (ADR-0096).

## Architecture scenarios

These are reusable scenario seeds, not new product requirements or a demand to implement every
analysis mode. Select and refine those that distinguish alternatives in the requested scope.
A review may link a row and record only its case-specific conditions and acceptance.

| ID | Stimulus and conditions | Expected response / architectural observation |
|---|---|---|
| <a id="pse-s01"></a>PSE-S01 | Add a unit/property model using existing physical concepts | Declarations and specialized behavior extend existing owners; admission, diagnostics and workflow policy are not re-authored |
| <a id="pse-s02"></a>PSE-S02 | Replace/add an implementation of an existing consumed solver/provider capability | Integration and explicit selection absorb the change; semantic consumers retain their contract; differences in capability remain visible |
| <a id="pse-s03"></a>PSE-S03 | Compose a new study or analysis workflow | Reuse model preparation and execution primitives; identify genuinely new policy and avoid copied end-to-end flows |
| <a id="pse-s04"></a>PSE-S04 | Change a result storage/transport representation without changing scientific meaning | Persistence/conversion owners absorb the change; physical meaning and computation remain under their existing owners |
| <a id="pse-s05"></a>PSE-S05 | Exercise admission or execution policy independently | Tests supply the actual semantic inputs/dependencies without starting unrelated solvers, stores or workflows |
| <a id="pse-s06"></a>PSE-S06 | Upgrade an integration library within the intended capability contract | Identify integration owners absorbing API/lifecycle changes and any justified consumer contract migration |

Inspect dependency edges from manifests and executing paths. Existing dependency/import checks
can support named boundary claims; an acceptance manifest is not an architectural dependency
map. No count of crates, traits, schemas or library features establishes quality.

## Follow-up ownership

The review records its version, inspected scope, findings and evidence strength. After a
finding is adopted for work, the owning plan's coverage/disposition table holds its current
state and links its scenario, decision where needed, packet and evidence. Until then the
review names a proposed work owner without claiming that work is scheduled. Use the template's
slot 11 fields; avoid an additional independently edited backlog.

The chain is foundation → scenario → finding → decision where required → packet → evidence.
Plan indexes link to the status owner rather than copying packet completion. A packet's
execution record owns its observations; a plan disposition links them. Accepted ADR evidence
records support at decision time, not live implementation qualification. Deferred architectural
decisions still use the existing ADR register and observable triggers. Workflow bookkeeping
and semantic judgments remain separate; there is no generated approval or architecture score.

## Where profile roles are decided

| Role (principle) | Authority in this repository |
|---|---|
| Architecture review and tracking (AP-01–AP-06, G9) | blueprint §24.4; ADR-0094; the current core and template |
| Model authority and identity (DP-01, DP-04, DP-05) | blueprint §2 (D1–D14), §5 and §6; ADR-0088/0089 |
| Ownership of math, derivatives, sparse algebra, properties, structure and solvers (DP-13, DP-17, PS-07, PS-09) | blueprint §1.3, §3.2, §3.3, §7, §14 and §18; ADR-0082–0084 |
| Supported math, provider and solver scope (DP-15, PS-02, PS-10) | blueprint §9, §18 and §25; ADR-0082–0084 and ADR-0093 |
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
| No library capability is withheld for lack of a consumer; integration cost remains assessable under the core | DP-13, DP-16 | blueprint §3.3.1 |
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
| K2 | Prefer runtime and library mechanisms over generated projections (DP-16) | Blueprint §4.2 and ADR-0031/0051 keep registry-generated Rust, Python and documentation contracts | Existing generated contracts stand. A new generator states why no runtime mechanism serves |
| K3 | Foundation IDs `AP-nn`, refinement IDs `DP-nn`, profile IDs `PS-nn`; evidence vocabulary in principles §D; exceptions in §H | Retained accepted ADRs cite `DM-nn`, "charter §D" and "charter §H" | Read them through principles §I; accepted records are not edited |

K1 (library-first mathematics versus pre-Plan 14 blueprint text) is resolved: the
architecture sections now describe library-owned mathematics, providers and native solvers.

## Where defect shapes tend to land in this stack

Illustrative places the shapes in the `design-review` skill's reference have landed before.
Their absence proves nothing.

| Pattern | Shape | Principles |
|---|---|---|
| An ordinary workflow extension edits unrelated policy, provider and presentation owners | Change amplification; trace the responsibility crossings | AP-01, AP-03 |
| Policy tests construct a full runtime that initializes native/storage state | Unnecessary test dependencies, if the policy needs only explicit inputs | AP-02, AP-06 |
| Consumer branching on incidental backend objects | Leaked implementation knowledge | AP-02 |
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
