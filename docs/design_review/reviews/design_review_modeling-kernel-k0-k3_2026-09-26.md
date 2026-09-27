---
title: Modeling kernel K0–K3 target review
date: 2026-09-26
tier: design
purpose: target
standard: core-3.0/process-simulator-1.1
reviewer: Codex (author review, not independent)
---

# Modeling kernel K0–K3 target review

## 1. Scope, drivers and coverage

**Proposed; Accept at document level** for the amended K0–K3 target. This review assesses
architecture and specified semantics, not implemented behavior or full IDAES replication.
Core 3.0, process-simulator 1.1 and the pse-arrow binding apply. Plan 21 owns dispositions.

Inspected Plan 21 and companions, current architecture owners, authoring/parser and package
resolver, generated semantic values and registry generator, quantity inference, compiler
workspace, typed mathematics, and runtime composition/balance paths. Library skills and
pinned source establish interface fit; IDAES 2.13 characterization is source evidence only.
Native convergence, distributed models and seed science remain K4–K9, not accepted behavior here.

## 2. Decomposition, ownership and dependencies

| Responsibility | Contract and dependency | Effect owner / local setup |
|---|---|---|
| Authoring | Source to registry-shaped values; expression syntax and IDs | Pure bounded parsing |
| Model | Generated values and semantic framing | No native/Arrow dependency |
| Modeling | Checking, finite specialization, demand, dispatch, accounting | Immutable inputs; synthetic packages |
| Compiler | One Salsa workspace calling pure kernel and typed math | Existing input/lifetime owner |
| Math | Physical admission, Symbolica functions and partials | Existing library context |
| Runtime | Admission, resources, jobs and publication | Existing composition root |

The linear pipeline in the initial sketch is not a Cargo dependency chain. In particular,
modeling must not depend on compiler or native mathematics. The authoring package graph's
current `pse-structural` dependency unnecessarily imports physical machinery; K1 removes it.

## 3. Contracts, authority and constraints

Registry declarations own durable IR shapes. Source syntax and specialized outputs are
derived representations. Explicit semantic IDs are distinct from paths, graph/Salsa indices
and backend symbols. Defaults resolve before demand. Missing data is never an implicit zero.
Structural facts and runtime values have distinct dependencies and failure behavior.

| Physical element | Dimensions / units | Basis / reference | Envelope / authority |
|---|---|---|---|
| Function argument/result | Complete quantity scheme | Preserved, including affine differences | Typed before symbolic rewriting; `pse-quantity` |
| Contribution/port | Exact indexed quantity contract | No implicit coercion | Admission rejects mismatch |
| Partial derivative | Output difference / argument difference | Explicit argument held-variable convention | Domain obligations survive simplification |

**Well-posedness:** variable roles are explicit; demanded variable/equation bundles retain
ownership. Counts alone do not establish rank. Existing structural matching remains the
pre-solver owner. Equation cycles are not confused with recursive declaration expansion.

## 4. Change scenarios and composition

| Scenario | Expected boundary | Acceptance |
|---|---|---|
| <a id="s01"></a>S01 Add a kind and polymorphic correlation | Package data, generic schemas and existing inference | Synthetic kind and function with wrong-basis control |
| <a id="s02"></a>S02 Override a default and dispatch by member | Effective interface then demand, shared bodies | Only effective dependencies; two implementations/two bodies |
| <a id="s03"></a>S03 Compose indexed conservation | Generic contributions, explicit transfers | Pairing and independent closure controls |
| <a id="s04"></a>S04 Edit a runtime value or structural fact | Value admission versus structural queries | Incremental equals clean and query-event controls |
| <a id="s05"></a>S05 Test language/semantics in isolation | No solver/store startup | Pure crate tests |

## 5. Mechanisms and execution

Binding, guard resolution, effective defaults, eager structure, demand and dispatch are
bounded deterministic transformations. Ordered semantic IDs govern reproducible projections.
Graph SCCs diagnose recursive expansion; Salsa fixed-point recovery is not model semantics.
Typed math retains its existing scaling, domain and native outcome contracts. K3 finite
expression evaluation uses library derivatives; no new solve algorithm or success mapping is
introduced. Closure tolerances retain authored physical units and raw contribution lineage.

## 6. Architectural assessment and gates

| Foundation | Verdict at target level | Argument |
|---|---|---|
| AP-01 | satisfied | Science, semantic mechanisms, symbolic library and effects have separate owners |
| AP-02 | satisfied | Generated physical contracts isolate backend types |
| AP-03 | satisfied | Interfaces, slots and presets compose S01–S03 without implementation inheritance |
| AP-04 | satisfied | Registry authority; one workspace; one disposition owner |
| AP-05 | satisfied | Structural inputs, declaration graphs and outcomes explicit |
| AP-06 | satisfied | Pure frontend/kernel and isolated S05 fixtures |

| Gate | Target verdict and evidence |
|---|---|
| G1 Authority | pass: registry/source distinction and ADR-0097–0101 route |
| G2 Semantic fidelity | pass: complete physical schemes and explicit absence |
| G3 Validity | pass: validation before rewriting, explicit unsupported capabilities |
| G4 Hidden behavior | pass: no runtime values in structural predicates; explicit scopes |
| G5 Consistency/recovery | pass: immutable inputs, cancellation cannot publish partial specialization |
| G6 Transformation/reuse | pass: structure/value/lineage separation, equality-based reuse |
| G7 Capability claims | pass: finite K3 scope; K4–K9 excluded explicitly |
| G8 Library use | pass: pinned Winnow, petgraph, Salsa, Symbolica ownership |
| G9 Architecture | pass: six foundation arguments above |
| PS-G1 | pass: units, basis, references, affine differences and indexed contribution contracts |
| PS-G2 | pass: declared roles, structural matching and distinct graph meanings |
| PS-G3 | pass for touched target: library derivatives, retained obligations and independent closure; no new solver claims |

All passes above concern the amended specification. Runtime behavior remains untested by this review.

## 7. Findings

| ID | Cause and consequence | Principles / scenario | Correction and verification |
|---|---|---|---|
| <a id="f01"></a>F01 | Original K1/K3 deletion lists precede replacements; current consumers would break | AP-03, G2 / S03 | Stage retirement by last consumer; targeted replacement tests |
| <a id="f02"></a>F02 | Defaults after demand select the wrong dependency body; equal variable/equation counts overclaim DoF | PS-04, G6 / S02 | Resolve effective members first; retain structural matching; default/cycle controls |
| <a id="f03"></a>F03 | Closed science-specific axis/subject enums require Rust for a new kind | AP-03, AP-04 / S01 | Generic kind references with full physical controls |
| <a id="f04"></a>F04 | Unqualified value-reuse claim ignores numeric structural parameters | G6, PS-11 / S04 | Explicit structural/value inputs and event-backed incremental controls |
| <a id="f05"></a>F05 | Frontend graph import pulls physical kernel dependencies into pure tests | AP-01, AP-06 / S05 | Direct petgraph package graph; preserve resolver contract |

The amended target and execution packet incorporate these corrections; implementation
dispositions remain scheduled until their controls execute.

## 8. Library fit and ownership cost

Winnow extends the current parser; petgraph supplies SCC/order without persistent node handles;
Salsa supplies dependency tracking/backdating in the existing owner. Symbolica function maps,
derivatives and existing evaluator preparation avoid a custom AD engine. Generated tagged
alternatives avoid handwritten wire schemas. Current online examples are secondary to exact
pinned source. No new third-party dependency or general rule engine is required.

## 9. Alternatives and tradeoffs

Keeping science-specific runtime builders is simpler for one model but amplifies every future
port. Three science-layer crates repeat that boundary. One pure kernel plus data is selected.
Wrapping IDAES/Pyomo violates the clean-room production target. Separate Salsa ownership and
bespoke symbolic math add lifecycle or semantic duplication without a supported consumer need.
Finite std collections are the simplest viable specialization implementation; data-layer
admission retains Arrow/DataFusion roles without importing query infrastructure into the kernel.

## 10. Verification

**Interface-checked:** existing parser, schema generator, quantity inference, compiler and
library API paths inspected. **Proposed:** the execution packet's synthetic positive/negative
tests, finite math controls and incremental event assertions. No new test execution supports
this document. IDAES source characterization is not executed oracle agreement.

## 11. Authority changes, exceptions and disposition

ADR-0097–0101 record the target decisions. Accepted ADR status is not fabricated by this review.
Current architecture sections remain implementation descriptions; future contracts retain
their target label. Plan 21 owns F01–F05 and links this packet's eventual evidence. No SHOULD
exception or silent MUST waiver is needed for the stated amended target.

## 12. Decision

Behavioral specification: adequate for the stated finite kernel boundary. Architectural
fitness: adequate by AP-01–AP-06. Overall: **Accept at Proposed/document evidence level**.
This is an author review, not the independent review previously reserved to the maintainer,
and does not establish implementation acceptance or scientific qualification.
