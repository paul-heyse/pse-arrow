# Efficiency and solver target review: supporting evidence

The [principal review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md) owns the combined architectural judgment, the finding identifiers (`F01`…`F14`), the rule impacts (`RC01`…`RC11`), the decision and the evidence limits. The [solver target](solver-target.md) is a supporting document. It holds the detailed solver target that the principal summarizes in F09–F13 and RC10.

The other documents here are bounded supporting contributions, preserved as returned. They are not plans, competing verdicts or finding ledgers. Their local labels are not finding identifiers. Examples are O1, K1, A1, S1, L and P1, and X2's lettered facts. Recommendations remain *Proposed* until plan creation dispositions them.

**Baseline.** `main` at `6c368c88acda548d421b44e8fc4f457c95bcec7e`, clean tree, 2026-10-12, after Plan 37 closed. The review changed no production file.

**Standard.** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md), selected by [`standard.toml`](../../design_principles/standard.toml).

**Maintainer inputs, 2026-10-12:**
- **A holistic review.** It is not mechanism-by-mechanism. The detailed effort goes into what the already-used libraries can do.
- **Development loop.** In scope only where product-shaped.
- **Parallel execution.** In scope and to be designed.
- **No HSL.**
- **Solver requirements.** Those of the 2026-10-12 review stand.

## How the review was conducted

Fifteen read-only inquiries ran. Nothing was built, tested, probed or measured. Each brief is recorded in [inquiry briefs](inquiry-briefs.md).

1. **Stage 0: lead status.** The coordinator recorded the [status of every prior lead](lead-status.md). Leads were mapped to this review's causes only after synthesis.
2. **Stage 1: parallel inquiries.**
   - **Obligations ledgers** (`code-mapper`), each starting from the actual entry point:
     - [interactive journeys](w-a-interactive-obligations.md) (W-A);
     - [multi-case and long-running work](w-b-multicase-obligations.md) (W-B), including the product-concurrency inventory.
   - **[Machinery provenance and qualification archaeology](w-c-provenance-archaeology.md)** (W-C), from git history and plan texts.
   - **Library-responsibility surveys** (`library-research`). Each surveyed the full capability surface at the pin, neutrally, before current code. Each then mapped the ledgers onto its libraries in a Part B:
     - [R1 durable state and coordination](r1-durable-state-and-coordination.md) and its [mapping](r1-part-b-mapping.md);
     - [R2 preparation, reuse and evaluation](r2-preparation-reuse-evaluation.md) and its [mapping](r2-part-b-mapping.md);
     - [R3 observation, results and analytics](r3-observation-results-analytics.md) and its [mapping](r3-part-b-mapping.md);
     - [R4 numerical methods and native sessions](r4-numerical-methods-native.md) and its [mapping](r4-part-b-mapping.md);
     - [R5 capacity, parallelism and the Python boundary](r5-capacity-parallelism-boundary.md) and its [mapping](r5-part-b-mapping.md).
   - **[Clean-sheet allocations](c1-clean-sheet-allocations.md)** (C1, `design-reviewer`). It was designed from the target, the standard and the library surveys only, without reading code, plans, ADRs or prior reviews. It guards against anchoring on existing mechanisms.
3. **Landscape checkpoint.** The coordinator posted the opportunity themes and open premises to the maintainer before synthesis.
4. **Stage 2: principal synthesis.** The coordinator re-read the decisive source for:
   - the per-solve store path;
   - native session launch;
   - transient-fit evaluation;
   - protection acquisition.
5. **Stage 3: independent challenges** of the first draft, run in fresh context. Both were adopted in substance and reconciled in principal §11.
   - The [omission and preservation challenge](x1-omission-preservation-challenge.md) (X1) contributed:
     - the topology cause (F03);
     - the local journal tier;
     - occurrence versus content identity;
     - shared native resources;
     - the determinism owner.
   - The [numerical-methods challenge](x2-numerical-methods-challenge.md) (X2) contributed:
     - library facts A–I;
     - allocation reuse versus declared warm edges;
     - corrected per-block, KKT, recycle, fitting, history and stop-fidelity statements.

## Finding numbers cited by the challenges

X1 and X2 reviewed the first draft. That draft had no topology finding, so the numbers they cite shift by one from F03 onward:

| First draft | Final |
|---|---|
| F01, F02 | F01, F02 |
| — | F03 (topology; new, from X1 O1) |
| F03 | F04 |
| F04 | F05 |
| F05 | F06 |
| F06 | F07 |
| F07 | F08 |
| F08 | F09 |
| F09 | F10 |
| F10 | F11 |
| F11 | F12 |
| F12 | F13 |
| F13 | F14 |

## Corrections to earlier evidence

X2's facts A and B were read against the pinned Ipopt 3.14.20 and SUNDIALS 7.1.1 sources. They contradict four earlier claims:
- the 2026-10-12 solver evidence `nlp-solver-capabilities.md` (B1), that `ReOptimizeTNLP` keeps symbolic analysis;
- `nonlinear-and-linear-solver-capabilities.md` (B2), that KLU symbolic analysis is kept across KINSOL calls;
- `dynamics-solver-capabilities.md` (B3), that it is kept across `IDAReInit`;
- R4 here, in its Part B §1.1.

Those documents are not edited; the principal review and this README record the correction. W-B's description of FERAL's pool as global is corrected by R5 and X1: FERAL 0.18 builds one pool per solver.

## Evidence limits

- **Static reading only.** Code claims are *Interface-checked*. Library facts were checked against pinned sources or the library skills' existing executed probes.
- **Round trips.** Store round-trip counts come from code structure.
- **Unmeasured:** library behaviour under contention, durability on the RocksDB server and every performance effect.
- **Narrow coverage:**
  - Supervisor scripts, agent tooling and host admission were examined only where the product drives them.
  - Unit and property model internals were not examined.
- **Process note.** Two library inquiries, R2 and R3, sent the maintainer's email address in a crates.io User-Agent header while checking latest versions.
  - Both reported it.
  - Later briefs forbid it.
  - No other network request is known to have carried a personal identifier.
