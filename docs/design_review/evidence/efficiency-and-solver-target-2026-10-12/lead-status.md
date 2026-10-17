> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Stage 0 (coordinator): status of every prior lead at the baseline, recorded before synthesis. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# Lead status at the review baseline

**Baseline:** `6c368c88a`, 2026-10-12. This document records **status only**. The review maps these leads to its causes after synthesis.

Status sources:
- the owning plans' disposition and packet tables;
- Plan 36's stop checkpoint;
- the Plan 37 Outcome;
- for the solver path, the 2026-10-12 review plus a source diff since its baseline `ff5a4aa2f`.

| Lead family | Source | Status at baseline | Owner |
|---|---|---|---|
| Efficiency F01–F28, A01–A05 | 2026-10-11 review and audit | **Scheduled.** Many slices are implemented, with scoped controls (Plan 36 checkpoints). P-G1 assembled acceptance is open and stopped. F20 (sparse rank) and F28 (host policy) are **Open**. No finding is resolved except F29 | Plan 36 coordinator |
| Efficiency F29 | Same | **Resolved** (P-D5) | Plan 36 |
| Plan 36 packets (39) | 36a–36e | P-D5 is complete. The other 36 product packets are "In progress", with implemented slices per the companion checkpoints. P-G0 and P-G1 are in progress. R36-01 (SPQR sparse rank), R36-02 (`hashing_may_use_pool`) and R36-03 (optional publication) are unselected | Plan 36 companions |
| Plan 36 RC01–RC20 | Plan 36 rule decisions | **Accepted 2026-10-11.** Proposed ADR-0173–0180 record them; their status is still proposed | Plan 36 / P-G0 |
| Plan 28 US01–US05 | Unified-substrate review, 2026-10-05 | Scheduled; assembled E2/E3 pending | Plan 28 A–E |
| Plan 28 EF01–EF08 | Execution-efficiency review, 2026-10-05 | EF02 and EF08 resolved; the rest Scheduled | Plan 28 |
| Plan 28 F01–F05 | Completion audit | F01 and F03 resolved; F02, F04 and F05 Scheduled | Plan 28 |
| Plan 28 PE01–PE06 | Production-execution efficiency review, 2026-10-07 | Implemented with scoped companion evidence; composed E3/E4 acceptance remains | 28f, 28g, 28h |
| Plan 28 Enhancement F01–F05 and others | Remaining-design enhancement review, 2026-10-07 | Implemented; E3/E4 pending | Plan 28 |
| Plan 28 Parallel F01–F05 | Parallel-execution review, 2026-10-08 | Implemented; scoped managed controls pass; E3/E4 pending | Plan 28 / 28e |
| Plan 28 PA01–PA04 | Preparation assurance and reuse review, 2026-10-08 | Implemented; assembled E3 pending | 28i, 28j |
| Plan 28 Graph/hash F01–F03, U01 | Graph and hashing review, 2026-10-09 | Resolved | 28k / 28e |
| Plan 28 root E3/E4/E5 campaign | Plan 28e | Open and stopped | 28e |
| Plan 32 | Workspace content lifecycle | In progress | Plan 32 |
| Plan 37 F01–F07 | SurrealDB lifecycle review, 2026-10-12 (retired; `git show e145eb7d2:...`) | **Resolved.** Plan 37 is done at `6c368c88a`; bounded Q1 acceptance only | Plan 37 (closed) |
| Solver F01–F13 | Solver-methods review, 2026-10-12 | **Open, unscheduled.** Selection, composition, walker, bridges, recycle defaults, HiGHS basis and observation code are unchanged since `ff5a4aa2f`. The diagnosis describes current code | None yet (proposed new plan) |
| Solver RC01–RC13 | Same | **Unconfirmed** by the operator | — |
| Acceleration review 2026-10-03, F07/F11/F17 open, others partial | Via solver review A1 §5 | Subsumed by solver F01–F13, as that review proposes | — |
