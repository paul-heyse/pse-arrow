---
name: design-review-process-simulator
description: Process-simulator profile for the design-review skill — applies PS-01–PS-13 and gates PS-G1–PS-G3 (physical typing and conventions, property validity envelopes, conservation and closure, well-posedness and structural analysis, formulation domains and smoothness, derivatives and scaling, initialization and recycle convergence, solver class selection, truthful solve outcomes, one model across analysis modes, honestly scoped results, shared model checks and reference validation). Use together with design-review whenever the subject is flowsheet simulation, unit or property models, equation formulation, numerical solving, dynamics, optimization or parameter estimation.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit, Agent
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Design review — process-simulator profile

This skill **layers onto `design-review`**; it never replaces it. Load the core skill first.
The core fixes the review's shape, gates including architectural fitness, finding standard
and decision rules. This profile adds principles, gates, slot content and lenses for process
simulation software.

## The standard

Find the profile through the repository's `standard.toml` (`[[profiles]]` entry
`process-simulator`). It points to two documents:

| Document | Role |
|---|---|
| Profile principles | PS-01–PS-13 with the core principles each refines, gates PS-G1–PS-G3, simulator false positives, and the functional target the design must serve |
| Profile review additions | What the profile adds to each core slot: physical-semantics table, well-posedness statement, numerical stage columns, simulator journeys, conformance status, and calibrated finding shapes |

Profile principles tighten core principles; a profile MUST is never waived by an exception
record. The repository binding says which libraries and components fill each role (for
example, which solver owns nonlinear roots). This profile names none.

## What the profile adds to the review

1. **Profile gates PS-G1–PS-G3 in slot 6**, settled independently like the core gates.
2. **The physical-semantics table and well-posedness statement** (slot 3), reconstructed
   from the subject. Cells you had to invent are unresolved decisions.
3. **Numerical stage columns in slot 5** for every stage that formulates, evaluates or solves.
4. **Simulator journeys in slot 4**, chosen by relevance. The workloads in the profile's
   functional target — edit and re-solve, studies, recycles, dynamics, extension — are in scope
   for a whole-simulator target review even when current plans exclude them. A bounded
   subsystem review states which workloads and neighboring contracts it examines.
5. **Conformance-suite status in slot 10** for each unit or property model touched.

## Architecture remains the organizing question

Use [design principles](../../../docs/design_review/design_principles/core/design-principles.md)
together with [Heuristics for Efficient Architecture](../../../docs/design_review/design_principles/core/efficient-architecture-heuristics.md)
for consequential architectural and implementation choices. Consider relevant execution patterns
before committing to physical organization, interfaces, preparation, assurance and lifecycles;
address material mismatches while the design remains easy to change. Apply only relevant patterns
qualitatively, without an exhaustive checklist, cost models or new proof machinery.
Use the companion within AP-07/G9, preserving independent scientific judgments and required
checks of each new numerical state.

Start with modeled phenomena, owned domain operations, responsibilities, consumed contracts,
composition and change scenarios under the core foundations. AP-04 requires the model to govern
formulation, evaluation and outcome interpretation, including their contextual bindings and
invariants; matching result schemas alone cannot satisfy it. Assess adding a model, replacing an
implementation and testing admission or policy in isolation where they distinguish alternatives. Assess ownership of library state,
upgrade cost and duplicated workflow decisions. Do not demand a new trait, registry or crate
merely because a numerical capability has its own name. Scientific correctness and G9 remain
separate judgments; neither certifies the other. Qualitatively assess AP-07 for the complete preparation,
evaluation, solve and publication/recovery route at relevant size, sparsity/stiffness, case count
and concurrency. Necessary coupled/global work remains valid. Compare repeated assembly, scalar
crossings, live workspaces/trajectories, invalidation and nested pools with simpler native
composition. Reuse immutable validity while its premises hold, retaining independent PS-10
checks of each new numerical state. Budgets, cancellation/drain and honest partiality preserve
scientific meaning. Explain material tradeoffs in plain language under the core's qualitative
assessment scope; it requires no numerical estimates, cost models, runtime cost accounting or
cost proof artifacts. Such mechanisms need a separate concrete functional or operational
requirement. Quantitative performance/capacity claims still need measurements, and existing
scientific correctness obligations remain.

## Lenses that settle numerical claims

Use these lenses when a concrete numerical question remains unresolved. Select the relevant
paths and follow them far enough to settle it; this is not a required tracing checklist:

- **Solve outcomes (PS-10).** Trace every solver return to its typed outcome and on to
  publication. Look specifically for acceptable-level stops, iteration or time limits,
  restoration exits and evaluation errors. Check that a post-solve verification (residuals,
  bounds, domains, closure) runs independently of the solver's status.
- **Derivatives (PS-07).** Follow the derivative chain through equations, property providers
  and custom kernels. One link without the claimed order makes "exact" false. Where correctness
  is in doubt, a finite-difference comparison at a nontrivial point settles it.
- **Domains and smoothness (PS-06).** Find each guarded function in the authored model and
  confirm the guard survives simplification into the evaluated form.
- **Well-posedness (PS-04, PS-05).** Establish whether degree-of-freedom and structural analysis
  run before the solver, and whether their diagnostics name model elements. Check that stream
  topology is never traversed as if it were a solve order.
- **Initialization (PS-08).** Follow the failure path of initialization: every temporary fix,
  relaxation or deactivation must be restored.
- **Units and conventions (PS-01).** Check basis, reference state and gauge/absolute pressure
  at each interface between models and providers, not only dimensions.
- **Reuse across cases (PS-11).** For a value-only change or a sweep point, identify what is
  rebuilt. Confirm that warm starts are recorded as inputs of the result they changed.
- **Solver machinery (PS-09).** Assess the complete consumed contract: established solvers own
  fitting numerical iteration, globalization, factorization and native model management.
  Scientific families, accuracy and start/branch permissions and bounded domain composition
  remain with their semantic owner where no fitting library supplies the complete contract.
  Own generic machinery where a qualified solver fits needs a stated reason or is a G8 finding.

The profile review additions give the evidence bar for each finding shape. Use the core
REFERENCE for authority, reuse and graph shapes.

## Failure modes specific to this profile

- A solve reported as verified from the solver's status alone.
- "Exact derivatives" accepted without tracing the provider chain.
- A match with a reference simulator accepted without the same property parameters, reference
  states and tolerances.
- A structurally singular or over-specified case discovered only as numerical failure, with no
  finding against the missing pre-solve analysis.
- Solver internals recommended for re-implementation where a qualified solver owns them.
