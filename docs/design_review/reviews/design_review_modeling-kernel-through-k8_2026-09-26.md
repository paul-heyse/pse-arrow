# Modeling kernel through K8 — target review

## 1. Scope and drivers

Core 3.0 and process-simulator 1.1, design tier, target purpose. This review addresses
remaining K0–K8 work, including scientific package composition, steady/dynamic analyses,
existing fitting consumers and legacy retirement. K9 campaigns and measurements are
excluded. Drivers are extension locality, one science authority, meaningful physical
contracts, replaceable numerical implementations and locally testable mechanisms.

**Proposed: Accept-scoped for the corrective design below.** Existing source is
Interface-checked; this review does not certify the implementation or seed science.

## 2. Responsibilities

Syntax stays in authoring; pure admission/specialization in modeling; incremental
dependencies in the single compiler workspace; Symbolica/faer integration in math;
iteration in native backends; effects and attempts in runtime. Packages own scientific
definitions and numerical knowledge. Python and Arrow transports preserve these contracts.
Neither a new science crate nor an independent package/cache/solve framework is needed.

## 3. Contracts and authority

The registry owns durable shapes. Explicit package manifests own dependency closure;
source declarations own meaning and identity; checked products are immutable derivatives
bound to their physical context. Cases, policies, attempts and results have separate
lifecycles. Units, basis, datum, index shape, pressure convention and validity remain
complete physical contracts. Existing quantity references own their representation.
Structural admission precedes native execution and names source variables/equations;
stream topology does not become solve order.

## 4. Representative changes

| Scenario | Expected boundary and acceptance |
|---|---|
| S01 Add a correlation/package | New declarations and sourced data only; dependencies and conformance follow existing mechanisms |
| S04 Replace FeOS | Package potential and explicit realization replace the provider; unit consumers keep their scientific interfaces |
| S05 Switch mode | Same definitions, explicit analysis/fixture selection, complete physical trajectory outcomes |
| S08 Edit/reprepare | Admission and unaffected bodies reused; changed dependencies equal clean recomputation |
| S11 Test locally | Pure fixtures use compiler/math without runtime/native startup |
| PSE-S04 Change transport | Structured failure and lineage survive without consumer prose parsing |

## 5. Execution and scientific constraints

Typed transformations preserve original obligations and contributions. Scheme data selects
library numerics; implicit blocks use KINSOL and faer with original residual/bound checks.
Branch-scoped derivatives require explicit branch admissibility; unproved crossings refuse.
Caloric defaults consume authored primitive functions whose derivatives are checked, rather
than presuming general integration. Initialization uses immutable overlays. Native statuses,
physical qualification and scientific reference agreement are independent claims.

## 6. Foundations and gates

AP-01–AP-06 are satisfied by the **proposed** ownership, immutable admission, composed
workflows, package authority, enforced capabilities and pure local-test route above.
Current G9 remains unresolved for the full seed until the named changes are implemented.
Current G1–G3/G6/G7/G9 have the issues below; G4/G5/G8 and PS-G1–PS-G3 require targeted
evidence on the changed paths. Prior synthetic checks do not settle seed gates.
No gate is waived and no successful scientific output compensates for a boundary defect.

## 7. Findings

| ID | Interface-checked cause and consequence | Correction and principle |
|---|---|---|
| <a id="f01"></a>F01 | Python admits one bundle; global name lookup can resolve other loaded packages without a declared import. Ordinary package composition depends on callers. | Explicit closure admission and visibility; AP-03/AP-04, DP-09 |
| <a id="f02"></a>F02 | Checked maps are public and specialization takes an independent physical context; consumers can bypass admission meaning. | Immutable checked products/context binding; AP-05, DP-03 |
| <a id="f03"></a>F03 | Every runtime preparation republishes and rechecks the entire source inventory before tracked reuse. | Revision-owned admission and compiler-tracked dependencies; AP-06, DP-09/DP-10 |
| <a id="f04"></a>F04 | Shared conformance selects default bindings and one solver policy; initialization and dynamic seed fixtures require custom orchestration. | Typed fixture execution through existing operations; AP-03/AP-06, PS-13 |
| <a id="f05"></a>F05 | Study errors and some qualification errors become strings; public conformance starts a runtime even for pure fixtures. | Preserve typed failures and pure dispatch; DP-18/DP-21 |
| <a id="f06"></a>F06 | Closed scheme/realization enums do not implement the promised data/capability extension contract. | Checked scheme data and registered accelerator references; AP-02/AP-03 |
| <a id="f07"></a>F07 | Current regime selection refuses derivatives; current continuous integrals do not establish arbitrary caloric antiderivatives. | Branch-scoped derivative admission and explicit caloric primitives; DP-08/DP-15, PS-06/PS-07 |
| <a id="f08"></a>F08 | Seed omits live hydrocarbon vessel and existing reference knowledge; transient fitting still consumes the legacy model. | Add retirement knowledge and migrate consumers before deletion; DP-16, PS-11 |
| <a id="f09"></a>F09 | Existing BT demonstrations use different coefficients/datums from the IDAES reference inputs. | Separate production/source/demo/oracle provenance; DP-01/DP-11, PS-01 |

## 8. Library fit

Retain existing Winnow, petgraph, Salsa, Symbolica, faer and native solver ownership.
FeOS/teqp become independent reference execution only. No generic differentiation,
nonlinear iteration or linear algebra is reimplemented. Caloric identities are scientific
package data; checking their derivatives uses the existing symbolic capability.

## 9. Alternatives

Keeping public unchecked maps or caller-assembled package universes reduces immediate edits
but transfers invariants to every consumer. Per-model test code violates extension locality.
A universal new workflow engine duplicates existing execution. Globally smooth regime
switching would substantially enlarge scope and is not the maintainer-selected contract.
The chosen corrections reuse established owners and break immature internal interfaces.

## 10. Verification

The K8 execution packet names targeted synthetic, seed, replacement and final assessment
checks. No checks were executed for this design review. References read through the pinned
IDAES skill are static behavior/oracle records, not observations of executed parity.

## 11. Authority and dispositions

Plan 21 owns F01–F09 dispositions. Existing execution packets own reopened K1–K7 work;
the K8 packet sequences dependencies and owns seed/retirement progress. Proposed
ADR-0098–0101 are amended before implementation. Authoritative architecture changes use
the existing decision/design route. Formal ADR acceptance remains separate from this review.

## 12. Decision

Architectural fitness of the corrective target: **Proposed, adequate**. Behavioral and
scientific implementation adequacy: **unresolved pending targeted verification**.
Overall **Accept-scoped at design level** for E0–E10 excluding K9, with no waiver of a
MUST and no claim of campaign parity, global regime smoothness or measured performance.
