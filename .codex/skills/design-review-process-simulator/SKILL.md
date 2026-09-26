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

Start with responsibilities, consumed contracts, composition and change scenarios under the
core foundations. Trace adding a model, replacing an implementation and testing admission or
policy in isolation where they distinguish alternatives. Assess ownership of library state,
upgrade cost and duplicated workflow decisions. Do not demand a new trait, registry or crate
merely because a numerical capability has its own name. Scientific correctness and G9 remain
separate judgments; neither certifies the other.

## Lenses that settle numerical claims

Numerical claims are unusually easy to assert in prose. Trace them through the design or
code rather than accepting the description, using judgment about where doubt is material:

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
- **Solver machinery (PS-09).** Search for own Newton steps, line searches, tear convergence or
  factorization. Where a qualified solver clearly fits, each needs a stated reason or is a G8 finding.

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
