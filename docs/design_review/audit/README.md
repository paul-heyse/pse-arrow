# Design principle audit

**First audit, 2026-10-21.**
- **Audited source:** commit `a791c533eca0f65b195895f071e60cd1e38e121c` (`main`, "plan 48
  complete"), clean tree, no uncommitted changes included.
- **Publication:** a separate, later commit publishes this account. Reviews cite that
  publication revision and the finding anchors below.
- **Standard applied:** core design principles 3.4 with Heuristics for Efficient
  Architecture 1.0, process-simulator profile 1.5, and the pse-arrow binding, as selected
  by [`standard.toml`](../design_principles/standard.toml).
- **Prior baseline:** none. This is the first audited account, so nothing was inherited
  from an earlier audit.

The [audit skill](../../../.codex/skills/design-principle-audit/SKILL.md) maintains one
living account of current design-principle misalignment. The
[repository binding](../design_principles/binding/pse-arrow.md#design-principle-audit)
places it before a design-review engagement. Ordinary implementation reads its plan and
architectural owners; this collection is not default production-editing context. The
account diagnoses; it does not choose remedies, set priorities, authorize remediation or
certify that unlisted areas are aligned.

## Inputs and independence

The maintainer required this first audit not to rely on any prior design review. The
inputs were:
- the standard;
- the architecture sections under `docs/authoritative_design/` and the retained ADRs,
  read as declared design intent;
- the source and manifests at the baseline.

Design reviews (`docs/design_review/reviews/**`), review evidence
(`docs/design_review/evidence/**`) and implementation plans (`docs/plans/**`) were
excluded as inputs, as were review links inside ADRs and sections. Finding IDs from
earlier reviews that appear in code comments were not treated as evidence. Library
questions used Context7, upstream source and the native-solver-libraries skill, not the
retired capability maps.

## Method and coverage

The account was built by a coordinator plus independent read-only contributors:
- one orientation mapper (crate graph, owners, generators, Python boundary, four
  end-to-end routes);
- seven slice reviewers;
- one fresh challenge reviewer, who reassessed the reconciled draft.

Contributors read source and declared design and reasoned through representative
operations and the binding scenarios PSE-S01–S07. The coordinator grouped their candidates
by structural cause and spot-checked the most load-bearing citations of each finding.
Claims the coordinator confirmed are marked "coordinator-confirmed".

The challenge reviewer then re-read the decisive sources of every finding. Its
corrections were all reconciled into this text:
- citation fixes and recounts;
- one overturned consequence (partial study points, DA-007) and one false claim (a
  conformance store requirement, DA-019);
- weakened claims (DA-015 reuse, DA-020 scope, DA-030 LRU);
- regrouping:
  - store vocabularies moved into DA-007;
  - annotation lowering (DA-006) separated from view choice (DA-031);
  - identity-kind loss merged into DA-005;
  - relation names as strings added as DA-032.

No challenged point was kept against the challenge without a stated reason.

Nothing was built, run or measured during the audit. Evidence is source reasoning at
*Interface-checked* or *Implemented* strength (design principles §D). Every claim of cost
or magnitude is an unmeasured hypothesis unless stated otherwise, and findings name the
probe that would settle it.

| Slice | Inspected | Deliberately not examined |
|---|---|---|
| Semantic model and authority | Registry catalogs, `pse-ids`, `pse-quantity`, `pse-relations` physical obligations, specialization identity, compiler lowering, artifacts, Python contracts and codec | DataFusion invariant execution, SurrealDB codecs, Symbolica symbol handling, YAML admission, reference package content |
| Mathematics and compilation | `pse-math` admission, guards, jets, assembly; `pse-compiler` workspace, projection, grouping, solve views; `pse-modeling` specialization, grouping, evaluation; kernels; structural; rules | Selection-proof machinery, `derived`, continuation, engineering accuracy, curvature-rule correctness, compiled-module cache, quantity inference |
| Native execution | Execution seam and adapters, routing, quality, presolve finish, Ipopt/Uno/PETSc/KINSOL/recycle/dynamics selection; runtime math initialization, solves, paths, strategy, jobs, native child (production-caller searches include `xtask/` binaries) | IDAS/Diffsol internals, SCIP factorable internals, HiGHS diagnostics, conic lowering, KKT mathematics, IBEX internals, surrogates |
| Workflows, studies, dynamics, fitting | Steady, local and durable study, simulation, fit, shooting and horizon start paths; strategies; dispatch; annotations; covariance and propagation | Most result/persistence modules, source persistence internals, horizon and shooting loop internals, `fitting/profile`, numerical correctness |
| Persistence, results, diagnostics | Substrate catalog, generated SurrealQL and templates, canonical store owners, durable lifecycle, result projection/snapshot/selection, diagnostics, capture placement, vendored SDK diff | Study readiness scheduling, analyses and artifacts internals, replay, physical spool, cache-key completeness in depth, `pse-columnar` canon internals |
| Python boundary | `pse-py` bindings and documents, Python facade modules, codec, conformance and diagnostics tooling, generated contracts (sampled), parity isolation | `TableStream` internals, most of `pse-py` diagnostics, parity science, Python tests |
| Cross-route and workspace | Four end-to-end routes, manifests and dependency direction, reuse layers, change scenarios, test architecture, declared-map drift | SurrealDB protocol ordering, adapter internals, Salsa structural-literal folding, parity, performance |

**Overall limits.**
- Scientific and numerical correctness of the reference packages was not assessed.
- Thread-pool and CPU-admission interplay between DataFusion and native work was not
  examined.
- No finding below is a statement that unlisted code is aligned.

## Finding index

Each finding has one primary home. Identities are stable; retired IDs are never reused.

| ID | Finding | Home |
|---|---|---|
| [DA-001](semantic-model-and-authority.md#da-001) | Specialized model carried in the authoring syntax tree; resolved identity in synthesized names | Semantic model and authority |
| [DA-002](semantic-model-and-authority.md#da-002) | Derived identities minted from call-site strings and re-derived by consumers | Semantic model and authority |
| [DA-003](semantic-model-and-authority.md#da-003) | Modeling expressions stored as plain text; grammar and role restated per consumer | Semantic model and authority |
| [DA-004](semantic-model-and-authority.md#da-004) | Rust–Python decision vocabularies hand-spelled per layer | Semantic model and authority |
| [DA-005](semantic-model-and-authority.md#da-005) | Physical meaning and identity kind lost on Python side routes and result reads | Semantic model and authority |
| [DA-006](semantic-model-and-authority.md#da-006) | Numerical annotations lowered with different rules per analysis mode | Semantic model and authority |
| [DA-007](structure-and-invariants.md#da-007) | Canonical lifecycle state and store vocabularies as untyped text/flag bags | Structure and invariants |
| [DA-008](structure-and-invariants.md#da-008) | Persistence refusals lose typed cause; retry depends on prose | Structure and invariants |
| [DA-009](structure-and-invariants.md#da-009) | Operation kind as parallel closed sum types; pairing not structural | Structure and invariants |
| [DA-010](structure-and-invariants.md#da-010) | Propagated covariance admitted as certified on its face | Structure and invariants |
| [DA-011](structure-and-invariants.md#da-011) | Document versions and type pairing declared by hand | Structure and invariants |
| [DA-012](boundaries-and-composition.md#da-012) | Run lifecycle copied per operation kind; horizon cancellation diverged | Boundaries and composition |
| [DA-013](boundaries-and-composition.md#da-013) | Predicted-start policy implemented twice | Boundaries and composition |
| [DA-014](boundaries-and-composition.md#da-014) | Study value binding only for steady solves | Boundaries and composition |
| [DA-015](boundaries-and-composition.md#da-015) | Host boundaries re-compose runtime operations and bypass its reuse | Boundaries and composition |
| [DA-016](boundaries-and-composition.md#da-016) | Solver identity and composition knowledge outside the adapter seam | Boundaries and composition |
| [DA-017](boundaries-and-composition.md#da-017) | Capability inventory and declared matrix do not match what runs | Boundaries and composition |
| [DA-018](boundaries-and-composition.md#da-018) | Uno integration carries project numerics in a patched library | Boundaries and composition |
| [DA-019](boundaries-and-composition.md#da-019) | Python tooling restates Rust-owned contracts; a second canonical writer | Boundaries and composition |
| [DA-020](boundaries-and-composition.md#da-020) | Performance toolkit in the production runtime and foundation crate | Boundaries and composition |
| [DA-021](local-reasoning-and-assurance.md#da-021) | Domain values carry the composition root; policy tested through the full runtime | Local reasoning and assurance |
| [DA-022](local-reasoning-and-assurance.md#da-022) | Memory admission rests on hand-maintained size models | Local reasoning and assurance |
| [DA-023](local-reasoning-and-assurance.md#da-023) | Architecture contract interleaves status and lags code | Local reasoning and assurance |
| [DA-024](local-reasoning-and-assurance.md#da-024) | Bespoke machinery without a production consumer | Local reasoning and assurance |
| [DA-025](execution-fit.md#da-025) | Persistence machinery sized beyond actual participants | Execution fit |
| [DA-026](execution-fit.md#da-026) | Bespoke result index; per-row transactional locators; unpruned DataFusion scan | Execution fit |
| [DA-027](execution-fit.md#da-027) | Shared bodies grouped, re-expanded and re-identified | Execution fit |
| [DA-028](execution-fit.md#da-028) | Single-output bodies recompute shared members per output | Execution fit |
| [DA-029](execution-fit.md#da-029) | Superlinear whole-model scans in preparation | Execution fit |
| [DA-030](execution-fit.md#da-030) | Coarse compiler reuse granularity and parallel hand-keyed caches | Execution fit |
| [DA-031](semantic-model-and-authority.md#da-031) | Steady case resolution restates compiler eligibility and admission rules | Semantic model and authority |
| [DA-032](semantic-model-and-authority.md#da-032) | Relation policies name relations by literal strings | Semantic model and authority |

Considered and not raised as findings:
- The strategy driver's per-variant switches over recovery mechanisms. They are
  compiler-checked, the set is small, and the driver is the legitimate composition root.
- HiGHS basis export decided by reading a string-keyed value. The key comes from HiGHS's
  own string info API, and the decision stays inside the adapter
  (`crates/pse-backend-native/src/highs/diagnostics.rs:513`, `:589`).
- The placement of egobox, argmin, statrs, nalgebra and pounce-* libraries. Each is
  justified by its single consumer or integration owner (ADR-0175, ADR-0209).

## Strengths to preserve

These bound the findings above, and a later remedy should not discard them.

- **Physical typing.** One owner holds kind, basis, reference state, point/difference
  scale, shape and subject, with a single unit algebra (`crates/pse-quantity/src/quantity_type.rs:9-45`;
  `crates/pse-relations/src/validate/obligations/physical.rs:27-60`).
- **Domain guards survive rewriting.** They are created at operator admission and
  materialized before Symbolica sees the expression (`crates/pse-math/src/typed.rs:811-905`;
  `guarded.rs:151-278`).
- **Derivatives are library-owned with no silent fallback.** The available order is the
  minimum over provider order, smoothness and branch continuity. Unmet demand refuses, and
  the derivative-source vocabulary has no finite-difference value (`crates/pse-math/src/execution.rs:4237-4313`;
  `crates/pse-kernels/src/lib.rs:221-269`).
- **Conservation by construction.** Shared accumulators carry modes and roles, and closure
  is assessed from original terms (`crates/pse-modeling/src/specialize.rs:3015-3083`).
  Science is package data, and adding a unit model on existing physics is a package-only
  change (PSE-S01).
- **Solver outcomes are truthful.**
  - Adapters map native codes to typed outcomes without parsing messages. Uno's integer
    literals have not been verified against its C API (DA-018).
  - Qualification reads typed evidence and names no backend (`crates/pse-backend-native/src/quality.rs:581-676`).
  - There is an independent original-space re-check after presolve and transport.
  - Iteration- and time-limit results become seed-only unless policy explicitly allows
    more.
- **Routing never substitutes an explicit selection.** The adapter table is an exhaustive
  match, and published capability rows are generated from the records. The records
  themselves can overstate admitted capability (DA-017), so a remedy should not treat them
  as authoritative unexamined.
- **Initialization never mutates the specification.** Overlays never edit it, and only
  complete stages commit (`crates/pse-runtime/src/math/initialization.rs:349-369`;
  `workflow/staged.rs:100-127`).
- **Cancellation drain.** Compute is released only after the native thread joins
  (`crates/pse-runtime/src/math/jobs.rs:1428-1490`), under one process-wide budget.
- **The fenced durable lifecycle.**
  - Claim comes before native effects.
  - Close freezes membership.
  - Seal compares manifest and authority atomically.
  - Cancellation wins.
  - An expired worker cannot seal success.
- **Value changes do not rebuild structure.** `PreparedCase::rebind` shares plan,
  structure and artifacts, and steady studies rebind on a lane-held view with counted
  preparation.
- **Pure, tested policy cores** (`crates/pse-model/src/study_policy.rs`;
  `crates/pse-runtime/src/math/strategy.rs`).
- **The Python boundary performs no science.** Tables cross as Arrow streams, decoding is
  strict, unknown versions are refused, and parity isolation is enforced by import-linter
  contracts.

## Reading and organizing the account

The topic pages are organizational aids, not exclusive classifications or extra design
criteria. A finding's principle IDs carry the criteria. Its footprint can be narrow or
broad. The [writing guidance](../../../.codex/skills/design-principle-audit/references/audit-writing.md)
explains finding content.

| Topic | Page | Main concerns |
|---|---|---|
| Semantic model and authority | [semantic-model-and-authority.md](semantic-model-and-authority.md) | Model adequacy, authoritative meanings, independent consumer interpretation |
| Structure and invariants | [structure-and-invariants.md](structure-and-invariants.md) | Distinctions, relationships, state, lifecycle, configuration and enforcement |
| Boundaries and composition | [boundaries-and-composition.md](boundaries-and-composition.md) | Responsibilities, contracts, dependencies, orchestration and library delegation |
| Local reasoning and assurance | [local-reasoning-and-assurance.md](local-reasoning-and-assurance.md) | Required context, effects, isolation, diagnostics and independent verification |
| Execution fit | [execution-fit.md](execution-fit.md) | Necessary and repeated work, crossings, expansion, reuse, resource and recovery scope |

## Ownership and history

This collection owns observations at its stated baseline. Reviews own proposed
corrections and alternatives. Implementation plans own scheduled disposition and closure
evidence. Architectural owners and ADRs own accepted design. Link to the relevant owner
without reproducing its progress. Existing source-review finding IDs and evidence keep
their original meanings and dates.

Refresh in place before a new review engagement, using these observations and intervening
changes to focus current source inspection. Reconsider unchanged code when its
dependencies or expectations change. Preserve finding identity and material lineage, and
remove resolved material once its correction is established. Git preserves earlier
accounts, and the removal commit identifies closure evidence. No separate audit archive or
execution register is needed.
