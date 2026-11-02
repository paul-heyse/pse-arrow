---
title: "Overview"
notion_source: "https://app.notion.com/p/40a141b5836981a2b5dcc14562bb8bd6"
notion_last_edited: "2026-11-01T20:41:16.324Z"
---

# Overview

Source: [Notion](https://app.notion.com/p/40a141b5836981a2b5dcc14562bb8bd6). Imported 2026-11-02.

**Version 3** · 2026-11-01

> The general formulation contract separates a physical model, the case in which it is evaluated, the question being asked, and the numerical method used to answer it. The immediate implementation domain is flowsheet process modeling. Molecular simulation, quantum chemistry, and full CFD remain extension and integration vocabulary rather than commitments for the flowsheet frontend.

The object submitted for one computational task remains:

```txt
SimulationProblem = Model + Specification + Query
                    + referenced Vocabulary
                    + optional MethodRequest
```

A Study composes these atomic problems. A Solution reports the answer, the evidence supporting it, and how it was obtained. This envelope can accommodate several domains; it does not replace the engineering concepts that each domain needs.

## What each part means

| Part | Responsibility |
| --- | --- |
| Vocabulary | Reusable definitions and their versions: physical kinds, units, chemical identities, constitutive models, interface types, component templates |
| Model | Instantiated structure, physical assumptions, quantities, relations, interfaces, and declared capabilities |
| Specification | Case knowledge: parameters, feeds, initial and boundary data, observations, uncertainty information, selected physical realizations, and the source of operating requirements |
| Query | Active roles, task profile, enforcement or assessment of operating requirements, objectives, outputs, and requested numerical evidence |
| MethodRequest | Numerical preferences or required execution conditions; these never silently change physical assumptions |
| Study | Replication, experiments, sweeps, continuation, estimation/control workflows, and explicit transfer of results |
| Solution | Model-level values and diagnostics, qualification of what was established, and reproducible provenance |

The same suitable model can support simulation, design, optimization, estimation, and uncertainty propagation. Its Specification and Query will often both change. Reuse is conditional on the model's capabilities: transient storage behavior requires explicit inventories and accumulation laws, while memoryless units may remain algebraic within a dynamic flowsheet.

## Scope and the process frontend

The near-term domain is interconnected process equipment and its internal physics: material, energy, and work transfer; reaction and phase behavior; lumped and distributed units; steady and dynamic operation; and the studies performed on those systems.

A flowsheet is more than a graph of generic mathematical components. Authoring also needs explicit process contracts for material definitions, thermodynamic states and property packages, reaction packages, balance regions, unit operations, and connections. The companion [Flowsheet Process Domain Model](../process/index.md) supplies those engineering semantics and lowers them into this general formulation contract.

Spatial domains, particle coordinates, stochastic relations, and nested external models remain representable. Their presence in the catalog does not require building a CFD, molecular-dynamics, or electronic-structure engine before implementing IDAES-style process examples.

## Four distinctions that prevent accidental changes of meaning

| Choice | Example | Where it belongs |
| --- | --- | --- |
| Physical model | Perfect mixing versus axial dispersion; equilibrium versus rate-based transfer; one thermodynamic package versus another | Model definition and selected realization in Specification |
| Mathematical formulation | Equivalent residual, reduced, complementarity, or minimization representation, under stated assumptions | Relation realization or a recorded lowering with an equivalence contract |
| Numerical approximation | Mesh, time step, collocation order, quadrature, local solver tolerances | Execution, optionally constrained by MethodRequest |
| Task and evidence | Find a feasible state, estimate parameters, minimize cost, request a stability check or an error estimate | Query and Solution |

Two physical models may share useful external interfaces while predicting different answers. Matching interface names does not prove physical equivalence. A CSTR and a distributed reactor may share outlet flow and enthalpy observables; a CSTR does not inherently provide the distributed reactor's internal concentration field.

## Principles retained and clarified

1. **Physical equations remain acausal where their semantics permit it.** Roles are assigned for the active task rather than baked into reusable balance equations. External directional models declare the evaluation capabilities they actually provide.
2. **Five supported task profiles organize the current contract.** Solve, Extremize, Expect, Infer, and Analyze are useful public profiles, with Infer first class. They are not a theorem that all future workflows must fit a permanently closed matrix.
3. **Physical assumptions and numerical approximations are separately recorded.** An execution plan may not silently replace the selected physics, feasibility requirements, or requested solution branch.
4. **Semantic identity is stable; human paths are navigational.** Stable identifiers and origin mappings support rename, reparenting, instantiation, and compilation without losing traceability.
5. **Alternatives advertise compatible public contracts.** Units, material scope, state basis, validity, observables, derivatives, and solution-selection semantics are checked before substitution.
6. **Problems are immutable snapshots.** Referenced definitions, data, and external realizations are pinned. Semantic and submission hashes identify different aspects of the request; only documentary annotations are non-semantic, and every actual run has its own identity.
7. **Validation is evidence-qualified.** Structural checks precede numerics where possible. Existence, uniqueness, stability, conditioning, identifiability, and accuracy are claimed only to the extent supported by analysis or computation.
8. **Diagnostics return to engineering meaning.** Results and failures refer to quantities, relations, components, and transformations; unavailable or ambiguous reconstructions are stated explicitly.

## How to read the rest

Read [Layers](layers.md) for ownership, [Answer-kind contracts](answer-kind-contracts.md) for the supported task profiles, and [Formulation and execution boundary](formulation-and-execution-boundary.md) for the compilation contract. The [Worked examples](examples/index.md) distinguish process-modeling examples from deferred integration illustrations.

Consequential revisions and their status belong in the project's Decisions log. The existing choices to keep the mesh in execution, Study outside the atomic problem, Infer first class, symbolic knowledge preferred, and execution capabilities modular remain intact.
