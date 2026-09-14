---
title: Reconcile blueprint revision 5 with library-backed contracts
status: done
date: 2026-09-13
adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0043, ADR-0044, ADR-0045, ADR-0046, ADR-0047, ADR-0048]
phase: 0
---

# Blueprint revision 5 contract reconciliation

## Context

The [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md)
identified thirteen contract findings and nine library opportunities. The user
authorized updating the blueprint, decision records and related documentation.
This plan delivers that design amendment and an implementation handoff. Its
completion does not mean the proposed compiler or solver contracts are built.

The reviewed baseline is revision 4 at commit
`fb0717d9151ada4884bd7ccb5efc27422bb92f7a`; the historical review and its
[reproducible characterization receipts](../design_review/evidence/blueprint-rev4-2026-09-13/README.md)
remain evidence for that baseline. The current architecture is
[blueprint revision 5](../authoritative_design/blueprint.md), with the bounded
[follow-up review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md).

## Decisions

All new records have **Proposed** evidence and `status: proposed`. The accepted
records' arguments are preserved; supersession changes only lifecycle fields and
status history. No dependency pin or family is moved.

| New record | Replaces | Decision / review coverage |
|---|---|---|
| [ADR-0039](../adr/0039-active-semantic-admission-and-quantity-algebra.md) | ADR-0008 | Active recursive admission, lossless head conversion and complete quantity algebra; R4-01/05/08, L1/L6 |
| [ADR-0040](../adr/0040-demand-seeds-and-stage-bundles.md) | ADR-0011 | Demand seeds before realization and named immutable stage outputs; R4-06 |
| [ADR-0041](../adr/0041-complete-stage-cache-inputs.md) | ADR-0017 | Complete stage keys, candidate absence and separate bound-value refresh; R4-07 |
| [ADR-0042](../adr/0042-defer-fine-grained-memoization.md) | ADR-0020 | Whole-stage reuse first; R-22 before finer reuse and R-01 before salsa |
| [ADR-0043](../adr/0043-complete-kernel-outcomes-and-bindings.md) | ADR-0012 | Units, effects, missing input, mandatory failure outcomes and actual derivative/backend bindings; R4-02/10, L2/L6 |
| [ADR-0044](../adr/0044-noncanonical-plan-evidence.md) | ADR-0019 | Noncanonical plan evidence outside semantic identity and reuse; R4-12 |
| [ADR-0045](../adr/0045-canonical-content-and-encoded-integrity.md) | ADR-0023 | Canonical v2, recursive null normalization, membership, physical checksums and publication; R4-03/04 |
| [ADR-0046](../adr/0046-shared-accounted-runtime-memory.md) | ADR-0029 | Shared runtime and fallible reservations for accounted platform buffers; R4-13, L5 |
| [ADR-0047](../adr/0047-guarded-numerics-and-safe-arrow-borrows.md) | ADR-0014 | Ordered numerical semantics, true branch execution and safe buffer ownership; R4-02/09, L2/L7 |
| [ADR-0048](../adr/0048-standard-relational-and-import-operators.md) | Supplemental | Shared predicates, aggregate/unnest and explicit-schema readers; R4-11, L3/L4/L7/L8/L9 |

The implementation starts with the simpler compiler from the review: complete
stage keys and a bounded canonicalizer. Changed semantic lineage stays in the
stage snapshot; equal relation storage does not authorize skipping a dependent
stage. No per-instance minimal invalidation or automatic whole-stage backdating
is promised. This is the smallest trustworthy initial granularity, not a measured
claim that finer reuse would be slower.

## Plan

### Documentation deliverables

- [x] Amend the blueprint in place, retain stable section numbers, and add the
  revision row and governed-section decision markers.
- [x] Allocate ten proposed ADRs and record nine symmetric supersessions without
  rewriting accepted arguments.
- [x] Reconcile all four capability maps' current recommendations, gates and open
  items while retaining original library evidence conditions.
- [x] Update the deferred register: R-01/R-11 and new R-22–R-25.
- [x] Provide the bounded follow-up design review and the implementation sequence
  below; update navigation and stale scaffold citations.
- [x] Complete the documentation/governance verification record below.

### Implementation handoff — Proposed, not executed by this plan

Each packet implements the cited blueprint contract. Test identifiers below are
proposed fixture names, not claims that those tests already exist. The owning
implementation change should place them in the indicated existing test workspace
and run its normal repository recipes. No newly added framework is needed.

| Order / packet | Depends on | Responsibility and consumer | Required acceptance before the next dependent packet |
|---|---|---|---|
| A — Registry and operation contracts | Normal maintainer decision process | `pse-schema`, `pse-relations`, `pse-quantity`, `pse-mathir`: admission, snapshot classes, ports, quantity compositions, WeightedMean, numerical/kernel outcomes; blueprint §4, §6–§8 | Generated contracts and supported/rejected physical-type fixtures agree; generator output remains the sole mechanical declaration |
| B — Bounded identity and local publication | A; reservation interface from E | `pse-ids`, `pse-catalog`: v2 canonical bytes, complete membership, physical encoding checksums and conditional refs; blueprint §5/§20 | Canonical metamorphic fixtures and corrupt/interrupted publication fail safely; no v1 hash relabeling |
| C — Demand, stage graph and rule execution | A–B | `pse-authoring`, `pse-rules`, `pse-compiler`: P3 demand seeds, finite P6 closure, explicit stage ports, shared predicate evaluation and aggregate/unnest; blueprint §9.6/§14 | Ordinary semantic admission, complete pushdown oracle, expansion reference and missing/late demand rejection |
| D — Complete stage reuse and plan evidence | C | `pse-compiler`, `pse-rules`, `pse-catalog`: all input/absence/policy/binary dependencies, immutable outputs, fresh lineage and noncanonical diagnostics; blueprint §14.3–§14.4/§20.2 | Every supported dependency mutation matches clean compilation; encoded plan variation cannot change semantic keys |
| E — Resource ownership and native preparation | A; allocation interface precedes B's writers | `pse-runtime`, `pse-numerics`, kernel/backend crates: shared RuntimeEnv, reservations, safe Arrow owners and actual branch regions; blueprint §14.3/§18 | Budget/cancellation release, borrowed-slice lifetime and guarded residual/derivative fixtures; integration with B/C before their acceptance |
| F — Backend and Python conformance | A–E | Native/NL/Pyomo bindings and `pse-py`: complete kernel contracts, numerical-policy refusal, affine qualification and coarse streams; blueprint §18/§21 | Each advertised route has a real conformance fixture; excluded-branch and invalid/missing outcomes remain distinct |
| G — Observation import and measured refinements | A–F as needed by the actual consumer | `pse-authoring` observation reader; existing Arrow CSV/JSON interfaces via deliberately enabled pinned features; later scan/transfer optimization | Explicit schema, fields, units and target tests first. R-22–R-25 require separate equivalence and end-to-end measurements before optional adoption |

Implement E's small reservation/ownership interface with A, then its native
execution body after D. This avoids treating memory accounting as a later retrofit
to B's allocations. Arrow CSV/JSON reader availability in the library does not
mean their features are enabled in the current direct Arrow dependency; G must
wire only the required existing-family features through workspace policy.

| Proposed fixture / owner test workspace | Required cases and observable result |
|---|---|
| `semantic_admission_paths` — `tests/conformance`, `tests/engine` | Wrong storage, unknown/malformed/versioned/nested extension metadata, ordinal targets/ranges; ordinary query/provider/import/output paths reject before assuming validity. Direct factory invocation is only a positive control. |
| `head_conversion_exactness` — `tests/engine` | Fractional integer cast, large integer Float64 loss, same-storage wrong units/basis/datum, nested/nullability mismatch; only named admissible conversions reach the head. |
| `quantity_composition` — `tests/conformance` | Additive material/energy sums, scaled temperature differences, gauge/absolute pressure, WeightedMean, molar/mass basis, enthalpy transport and standard gas-law/Arrhenius compositions; no dimensional-only inference. |
| `canonical_null_equivalence` — `tests/conformance` | Hidden primitive and nested payload/validity, sliced arrays, dictionary changes, batch/row order, all-valid bitmap, NaN/signed-zero rules and empty relations; same declared content has the same v2 digest. |
| `encoding_roundtrip_identity`, `snapshot_membership` — `tests/conformance` | IPC stream/file and Parquet share logical identity with separately checked bytes; explicit empty members, changed contract/parent, excluded sidecars and duplicate/missing ports. |
| `publication_fault_matrix` — `tests/lifecycle` | Existing truncated/corrupt object, failed serialization finish, interruption before each object/manifest/ref, CAS conflict and untrusted restore; only a complete validated snapshot is visible. |
| `demand_seed_closure`, `stage_bundle_graph`, `realization_completeness` — `tests/engine` | Law-only, initializer-only and transitive method demand; candidate addition/removal, ambiguity and unsupported demand; unique bound ports, acyclic stages, no P6 dependency on realized P7/P15 IDs; late demands fail. |
| `incremental_equals_clean` — `tests/engine` | Species/domain add/remove, package units, child/template binding, candidate universe/absence, kernel contract/binary, policies, engine settings, fixed/parameter values/treatment and free guesses; complete outputs and lineage agree. |
| `plan_evidence_roundtrip` — `tests/engine` | Resolve only pinned admitted providers/bindings; preserve attributable decoded plan meaning. Fresh-map/process byte differences are allowed and cannot affect memo keys. |
| `pushdown_vs_unpruned` — `tests/engine` | Compare complete values and multiplicities with an Unsupported provider; aliases, nulls, IN/disjunction, dictionary keys, filters omitted by projection and limits. Deliberately over-pruning/empty providers fail the oracle. |
| `relational_expansion_reference` — `tests/engine` | Aggregate/unnest grouping, ordinal preservation, provenance, empty/null lists and duplicate membership; checked count/integer sums; no undeclared floating reduction. |
| `kernel_outcome_conformance` — `tests/conformance` | Scalar/batch/UDF/native/exported adapters: natural units, null rejection/propagation, invalid domain and unsupported derivatives; tolerant results include every requested coordinate and mandatory reason. |
| `guarded_branch_execution`, `numerical_policy_conformance`, `nonlinear_affine_lowering` — `tests/conformance` | Mixed/nested masks, excluded log/division/overflow, CSE on/off, derivative branches, cancellation, signed zero, explicit FMA/reassociation policy, constant-degree-zero and nonlinear children in Affine; actual backend routes preserve supported behavior or refuse. |
| `arrow_borrow_lifetime` — `tests/lifecycle` | Eligible nonzero-offset slices, owners outliving callbacks/stream drains, mutation isolation, cancellation and reserved-copy fallback; null numerical inputs are rejected. |
| `query_memory_budget`, `canonicalization_memory_budget`, `result_memory_budget` — `tests/lifecycle` | Single and concurrent snapshots, non-spillable consumers, all live copies and failed reservations; release on cancellation/failure, no partial publication. Report whole-process peaks beside pool accounting. |
| `explicit_schema_observations` — `tests/conformance` | CSV headers and JSON unknown fields, duplicate/missing required columns, declared null spellings, finite/exact values, units, target IDs and located errors; no schema inference or null-to-zero fallback. |

## Verification

**Proposed:** all runtime fixtures and measurement packets above. Implementation
acceptance requires the appropriate Rust tests with explicit
`--features pse-relations/force-validate` (owned by `just test`), then the named
slice/conformance/lifecycle gates. Python/native changes require `just py-sync`,
`just py-test`, `just quality`; solver-backed parity uses `just parity-container`
against IDAES 2.12.0. Baseline is zero failures for each command and mode.

**Tested — historical characterization only:**
`bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh` completed six
characterization groups with zero failed assertions, baseline zero, in the
revision-4 review's dev/force-validate environment (Rust 1.98.1, Arrow 59.3.0,
DataFusion 55.1.0, offline locked resolution). It is not rerun for prose changes
and does not test the proposed v2 canonicalizer or any production integration.

**Interface-checked:** pinned source and rustdoc routes are recorded in that
review. Additional Context7 queries during this amendment returned broad
analyzer/runtime/reader documentation; they are discovery evidence, not a
substitute for the pinned call-shape checks or executable fixtures.

| Current amendment check | Mode / baseline | Result |
|---|---|---|
| `just doctor` | Environment preflight; zero failed checks | Passed at session start |
| `just adr-index`, `just adr-lint` | Generated index, front matter, section links, supersession and accepted-record immutability; zero findings | Passed: 48 records, 25 register rows; index current |
| `just docs` | mdBook build; zero errors; links additionally checked by the focused audit | Passed |
| `just lint-typos`, `just lint-license`, `just lint-agents`, `just fmt-check` | Repository static checks; zero findings | Passed; REUSE covered 368/368 files |
| `just family-check`, `just codegen-check` | Current-checkout dependency families/evidence agreement and generated-tree hygiene; zero failures | Passed; 54 shared evidence-family packages agree; 4 generated scopes clean |
| `just test-package pse-tests-governance -p pse-relations` | Nextest default/test profile, locked, Arrow `force-validate` explicitly enabled; baseline zero | 19 passed, 0 failed, 0 skipped across 13 binaries; run `e60383cf-16e5-4c73-a3da-175ff0e30279` |
| `git diff --check` and focused document consistency audit | Whitespace, original heading sequence, P0–P16, decision markers, table shape, manifest JSON and preserved evidence inputs; zero findings | Passed: 181 original headings retained, one inserted subsection; 17 pass rows; 55 governed-section references; 9 immutable supersession pairs; 271 tables and 257 local links at initial audit |

**Validation corrections:** initial `just governance`/family results were not
credited because a cached `xtask` binary retained the root of a temporary setup
checkout. A fresh task-specific Cargo target verified this tree; refreshing only
the driver's source timestamp then made ordinary `just family-check` resolve this
checkout too, without changing driver contents. A package-only force-validation
attempt failed before tests because the governance crate does not contain the
relations feature. Selecting `pse-relations` alongside governance, as in the
successful command above, supplies the required feature. The current `xtask
governance` recipe omits that feature, so the test, codegen and family components
were verified explicitly; no driver/governance implementation was changed here.

The host shell currently emits `/etc/bash.bashrc: line 7: PS1: unbound variable`
during recipe startup. Record command exit status and gate findings separately;
a successful check is not a claim that startup is warning-free. API-reference doc
lint remains deferred under R-20; no source generator or numerical acceptance is
inferred from a book build.

## Open items

- Runtime implementation and scientific parity remain the Proposed handoff above.
- R-22: measured complete finer dependency bundles; R-01: salsa only after that.
- R-23: larger canonical relation envelope or a new fixed-boundary format.
- R-24: qualify Parquet pruning and bounded coalescing separately by their consumers.
- R-25: exact supported predicate preimages, then measured total benefit.
- Maintainer publication remains the existing decision-PR process. This local
  amendment does not create a PR, merge a branch or accept an ADR. Use each
  record's `adr: ADR-NNNN <title>` and `adr`/`needs-review` labels as required by
  the repository; a named `design:` follow-up may carry the integrated blueprint.
  Its description must state that `PSE_DESIGN_EDIT=1` was used for the user's
  expressly authorized blueprint amendment and cite the revision-5 review.

## Outcome (recorded after implementation)

### What was built

The design/documentation amendment described above. Blueprint revision 5 and
ADRs remain **Proposed**; library receipts retain their original evidence labels.
No process-model implementation, new library family or solver behavior is added. Scaffold changes are contract citations, diagnostic wording and the benchmark placeholder label; the parsed Cargo manifest is identical to the baseline.
Current static verification is recorded separately in Verification.

### A mistake made and corrected

An edit selector matched the P3 table's `Canonicalization` cell instead of the
acceptance-table cell and removed intervening sections in an intermediate local
draft. The task's own edits were reconstructed from the baseline, selectors were
bounded to their intended section, and the entire original heading sequence plus
all 17 pass rows were checked. No unrelated work was overwritten. The final audit
also corrected self-referential hash descriptions, incomplete port naming and
stale current/historical guidance.

### Deviations from the plan, deliberate

The canonicalizer is a bounded v2 contract instead of promising arbitrary-size
one-batch construction. Whole-stage keys and explicit lineage replace premature
fine reuse; plan bytes remain noncanonical evidence. Additional optimization is
deferred by measured triggers. These are explicit ADR-0041/0042/0044/0045
decisions, not undocumented SHOULD exceptions or implementation performance claims.
