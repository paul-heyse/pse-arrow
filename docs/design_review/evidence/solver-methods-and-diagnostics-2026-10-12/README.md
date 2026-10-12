# Solver methods, selection and diagnostics review: supporting evidence

The [principal review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md) owns:

- the combined architectural judgment;
- the finding identifiers (`F01`…`F13`) and rule impacts (`RC01`…`RC13`);
- the decision and the evidence limits.

The documents here are bounded supporting assessments and evidence. They are not implementation plans, competing verdicts or finding-status ledgers. Recommendations remain *Proposed* until dispositioned through plan creation.

**Baseline.** `main` at `ff5a4aa2f7da60dff28a72b5b3bd563d75b43ab1` plus the uncommitted Plan 36 working tree (diff fingerprint `45ceea51b0698679`), read as it stood on 2026-10-12. The review changed no production file.

**Standard.** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md), selected by [`standard.toml`](../../design_principles/standard.toml).

**Maintainer requirements (settled 2026-10-12):**

- three-layer method selection: a priori rules from mathematical facts; in-solve observation-driven switching; an advisory recorded-history layer that only reorders or tunes admitted options;
- diagnostics and measurement as product capability;
- method adoption never gated on benchmarks;
- new libraries must be free to obtain, with licence type not an exclusion;
- parallelism that comes from code architecture is out of scope.

## How the review was conducted

1. **Wave 1: read-only inquiries.** No builds, tests or native runs; nine in total.
   - Three `code-mapper` maps of the current tree:
     - [selection and escalation](selection-and-escalation-map.md) (A1), including the status of the 2026-10-03 acceleration review's findings and mechanisms;
     - [observation, persistence, identity and replay](observation-and-persistence-map.md) (A2);
     - [structure, decomposition and evaluation capability](structure-and-decomposition-map.md) (A3).
   - Four `library-research` inventories at the pinned versions. Each covers methods, method-choice controls, reuse hooks, the complete statistics API, bound versus unbound, newer releases, and interrupt or observation hooks:
     - [NLP](nlp-solver-capabilities.md) (B1: Ipopt, POUNCE, Uno);
     - [nonlinear equations, least squares and all sparse factorization statistics](nonlinear-and-linear-solver-capabilities.md) (B2: KINSOL, PETSc SNES/TAO, diffsol-nl, KLU, faer, FERAL, MUMPS, SPRAL, Pardiso, HSL, QDLDL);
     - [dynamics](dynamics-solver-capabilities.md) (B3: IDAS, Diffsol, CVODES/ARKODE, PETSc TS);
     - [LP, conic, global and auxiliary](lp-conic-global-solver-capabilities.md) (B4: HiGHS, Clarabel, SCIP, IBEX, egobox).
   - [Free candidate libraries](candidate-libraries.md) (B5a).
   - [IDAES 2.13 diagnostics and initialization behaviour, simulator reference practice and the algorithm-selection literature](reference-practice-and-selection-literature.md) (B5b). IDAES was read for behaviour only, under the clean-room rule.
2. **Wave 2: the principal reviewer's analyses.**
   - [Method landscape and selection](method-landscape-and-selection.md): the mathematics of which methods suit which problem shapes, the feature model with lifetimes, and cause-typed escalation.
   - [Observation, diagnostics and measurement](diagnostics-and-measurement.md): the three-consumer observation model, detail tiers, failure diagnosis, the history store, and the captured-versus-available matrix for every bound solver and linear solver.
3. **Wave 3: independent judgments.**
   - An [independent design assessment](independent-design-assessment.md) by a fresh `design-reviewer`. It received the standard, target, requirements, raw wave-1 returns and neutral questions, but not the principal's conclusions.
   - An adversarial [numerical-methods challenge](numerical-methods-challenge.md) of the principal's draft mathematics. Its corrections are folded into the method landscape.
4. **Wave 4: reconciliation.** The principal review §13 records where the judgments agreed, where they differed and how each difference was settled. Decisive claims were re-read in source by the principal reviewer, and often by the independent reviewer too.

No probes were executed. Where a probe would settle magnitude or behaviour in the built environment, the worker documents name it: question, command, expected observation.

Worker returns are published unchanged, except that repository-absolute paths were normalized to repository-relative ones. Identifiers inside them (A1 leads, IR-nn, B-sections) are local to each document.
