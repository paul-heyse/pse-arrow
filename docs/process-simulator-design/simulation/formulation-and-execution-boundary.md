---
title: "Formulation and execution boundary"
notion_source: "https://app.notion.com/p/40a141b5836981ac9cf0d0d0a7295de5"
notion_last_edited: "2026-11-01T20:36:11.319Z"
---

# Formulation and execution boundary

Source: [Notion](https://app.notion.com/p/40a141b5836981ac9cf0d0d0a7295de5). Imported 2026-11-02.

**Version 3** · 2026-11-01

The submission boundary separates engineering meaning from execution choices. It receives an immutable description of the selected physical model, the operating case, and the question. It returns model-level answers together with evidence about feasibility, convergence, stability, optimality, uncertainty, and numerical error.

The boundary is a contract for validated structure and declared capabilities. It is not a claim that every general nonlinear or externally evaluated problem can be proved well posed before computation.

## What crosses

| Formulation responsibility | Execution responsibility |
| --- | --- |
| Physical geometry, domains, inventories, interfaces, selected constitutive assumptions | Numerical mesh, grid, finite elements, collocation points |
| Time horizon, initial/boundary conditions, physical events and resets | Step-size selection, event localization algorithm, consistent-initialization procedure |
| Relations and selected physical realizations | Recorded reformulations, scaling, elimination, transcription, decomposition |
| Required operating conditions, active roles, criteria, branch/stability expectations | Solver and initialization strategy, tearing, ordering, continuation |
| Measures and dependencies of uncertain inputs | Sampling design, quadrature, estimator implementation |
| Requested numerical evidence and output accuracy | Tolerances, refinement, error estimation and evidence of what was achieved |
| Non-semantic starting guesses, separately marked | Warm-start use, homotopy path, multi-start |
| Required execution conditions for comparison or reproducibility | A plan satisfying those conditions, or a precise unsupported result |

Sequential-modular and equation-oriented approaches can be execution strategies for the same process formulation **when the selected unit and property models support the needed evaluation interfaces**. An opaque unit that only accepts one causal input/output arrangement does not automatically supply arbitrary residuals, derivatives, or inverted evaluations.

## Description and evaluation contract

The description contains or pins index sets, quantities, symbolic relations, model/template/package versions, case data, and Query. External realizations are identified by immutable artifact/version references and the configuration needed to evaluate them.

An evaluation interface declares what is available rather than promising every capability:

- Residuals, explicit outputs, or both; supported input/output arrangements.
- Validity ranges and material/state conventions.
- Structural incidence or sparsity, including whether it is exact, conservative, or unknown.
- Derivative order and method, smoothness assumptions, determinism, and batch support.
- Nested-solve behavior, solution selection, failure modes, and derivative validity.
- Resource or runtime dependencies required for reproducibility.

An opaque function version by itself is insufficient if its databases, weights, runtime settings, or hidden state can change. Those dependencies are pinned or explicitly marked unresolved.

## Validation and evidence

| Check class | Examples | Permitted conclusion |
| --- | --- | --- |
| Structural/type validation | Resolved references; compatible units/kinds; non-overlapping bindings; material scope; required fields; legal connection contract | Pass or definite violation for the representation checked |
| Structural analysis under assumptions | Incidence matching, degrees of freedom, candidate DAE structure, declared derivative reachability | Pass/fail with assumptions and unresolved conditions |
| Numerical diagnostic | Consistent initialization, residual/Jacobian rank at a point, conditioning, phase stability search, practical identifiability | Evidence local to the points and methods tested |
| Certification where supported | Certified infeasibility, global bound, rigorous error estimate, proven property of a restricted model class | Only the specifically established certificate |
| Unsupported or unknown | Opaque dependence, unavailable analysis, unproved globality | Explicit limitation; no invented success or universal rejection |

A zero structural degree-of-freedom count does not prove existence or uniqueness. Boundary and initial conditions require more than counting entries. Structural DAE index and identifiability analyses are used where supported and qualified where inconclusive.

Required semantic contracts that cannot be satisfied are rejected. Optional analyses may produce an explicit unavailable result. A user can request strict evidence requirements; the system must then report inability to meet them rather than silently downgrade.

## Problem and plan lifecycle

```txt
validate(problem) -> qualified diagnostics
propose(problem, execution policy) -> candidate execution plans
run(plan, immutable submission) -> Run
result(run) -> Solution with model-level outputs and evidence
```

Plan proposal remains separate from execution so plans can be inspected and reused. A compiled structural plan may be reusable across changed bindings when its dependency and invalidation rules permit; a full semantic-problem hash is not the only useful cache key.

The semantic hash identifies the question, excluding clearly non-semantic guesses and documentary annotations. The submission hash includes guesses and MethodRequest. Required numerical settings remain part of the submission and provenance even when they do not alter the idealized physical question. Each actual Run has a separate identity; matching problem and plan hashes do not imply identical timing, numerical path, or all floating-point results.

## Reconstruction and diagnostics

A lowering records how its outputs derive from model quantities and relations. Values and diagnostics are reconstructed to engineering terms where possible. This map is not necessarily an exact inverse: elimination, aggregation, relaxation, discretization, and surrogate substitution may lose information.

Derivatives, duals, and error estimates are returned only where their meaning survives the transformations and required assumptions. A generated object may map to multiple origins and a transformation. Unsupported reconstructions are reported, with lower-level artifacts retained for inspection.

The primary interface reports statements such as “tray 14 energy balance residual: 3.2 kW.” Numerical mesh and solver details remain available in provenance and diagnostics where they help investigation; they do not become the engineering model's canonical identities.

## Accuracy means several different things

Numerical convergence, estimated numerical error, certified error bounds, uncertain-input propagation, and physical model discrepancy are distinct. An output request such as “purity within 0.001” must specify which error notion is intended. Solver residual tolerances alone generally do not certify that output bound.

The selected plan reports achieved evidence and unmet requirements. Model alternatives or uncertain parameters are not silently folded into a numerical tolerance guarantee.
