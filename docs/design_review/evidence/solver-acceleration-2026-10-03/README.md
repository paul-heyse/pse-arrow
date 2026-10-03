# Evidence: solver acceleration and globalization (2026-10-03)

Supporting evidence for the design review
`docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md`
(principal document; it owns scope, judgment, findings and evidence limits). Files here carry
bounded evidence only and no competing verdicts.

**Baseline.** `f0b902589723a86bc6755ce1c76b45a2ef90224b` plus the uncommitted Plan 25k working
tree at review start (`git diff` SHA-256 `1cee9b65…a232`, 49 files). A concurrent session was
active (Plan 25k edits; draft ADR-0153); source citations refer to the tree as read.

**Standard.** Core 3.3, process-simulator profile 1.3, `pse-arrow` binding
(`docs/design_review/design_principles/standard.toml`).

| File | Role |
|---|---|
| [colleague-input.md](colleague-input.md) | External review that prompted this work: functional targets A–H, mechanism proposals and bibliography. Input leads, not authority. |
| [mechanism-matrix.md](mechanism-matrix.md) | Coordinator synthesis: current-state observations C1–C14, mechanism × pipeline matrix M1–M18, conflicting rules and their intent, candidate contracts H1–H10 (hypotheses given to the independent reviewers). |
| [pipeline-map.md](pipeline-map.md) | Code map: the execution spine, seams for starts, retained state, problem modification, strategy, failure handling, budgets and accuracy; composing workflows; testability. |
| [reuse-structure-map.md](reuse-structure-map.md) | Code map: cross-solve reuse and prediction inventory; structural decomposition and its consumers. |
| [controls-facts-metrics.md](controls-facts-metrics.md) | Code map: solver controls per adapter, selection facts, in-solve observations and metrics, the options surface, accuracy tiers. |
| [pinned-library-capabilities.md](pinned-library-capabilities.md) | Library research: what the pinned solvers provide for acceleration and globalization and what pse uses (used / raw-reachable / not exposed / not provided). |
| [candidate-libraries.md](candidate-libraries.md) | Library research: unpinned candidates (PETSc, Uno, Ceres, russell_nonlin, HOMPACK, egobox and others) against the trial-failure, bounds, cancellation and typed-status requirements. Upstream sources it cites were read in a scratch copy that is not retained. |
| [mechanisms-local-and-predictive.md](mechanisms-local-and-predictive.md) | Literature cards M1–M8: sensitivity and path prediction, advanced step, root predictors, inexact Newton, Jacobian reuse, Anderson, trust-region/LM, Ipopt globalization. |
| [mechanisms-globalization-and-decomposition.md](mechanisms-globalization-and-decomposition.md) | Literature cards M9–M17: continuation, constructed homotopies, pseudo-transient continuation, DAE initialization, structural solving, reduced space, nonlinear preconditioning, multifidelity, multi-scenario decomposition. |

| [assessment-composition.md](assessment-composition.md) | DR-1 independent supporting assessment: pipeline composition and domain model (findings DC-F01–F11, contracts DC-C1–C10, replacement rules R1–R10, options catalogue). Bounded judgments; the principal review reconciles them. |
| [assessment-mechanisms.md](assessment-mechanisms.md) | DR-2 independent supporting assessment: mechanism placement, library decisions, numerical contracts per family, escalation-ladder content (findings DM-F01–F15). Bounded judgments; the principal review reconciles them. |

Code maps and library research cite source as read at the baseline; literature cards label each
claim by how it was sourced (read, abstract, known, interpretation). No benchmark or execution
evidence is included by design.
