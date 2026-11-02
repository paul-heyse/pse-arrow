---
title: "Simulation concepts — Solution"
---

# Simulation concepts — Solution

<a id="source-40a141b583698188916ac01f3414f387"></a>

## Accuracy report

Source: [Notion](https://app.notion.com/p/40a141b583698188916ac01f3414f387).

**Refers to:** Accuracy demands

**Rules:** Each requirement is met, not met, unsupported or not assessed, with proved/estimated/assumed status made explicit. Numerical convergence or an empirical mesh comparison is not promoted to a global physical-accuracy certificate.

**Multiplicity:** 0..1

**Definition:** Findings against each requested accuracy and mandatory acceptance requirement, naming the evidence kind, method, assumptions, scope and achieved quantities when available.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Accuracy report

<a id="source-40a141b5836981759d35df77a6e2f160"></a>

## Analysis result

Source: [Notion](https://app.notion.com/p/40a141b5836981759d35df77a6e2f160).

**Refers to:** Slices, relations

**Rules:** Separate structural diagnostics, numerical rank evidence and mathematical proof. Underdetermined or singular models may still be analyzed; a successful Solve is not universally required. Unsupported index, global-identifiability or stability claims remain unavailable.

**Multiplicity:** 0..n

**Definition:** Structural or local findings such as counts/matching, differential structure, sensitivities, linearization, eigenanalysis, identifiability or information measures, each tied to its inputs, method and assumptions.

**Layer:** Solution

**Owned by:** [Solution member](solution.md#source-40a141b583698178a443d0006d4e600a)

**Concept:** Analysis result

<a id="source-40a141b5836981e28eddeb1dac551b51"></a>

## Derivative set

Source: [Notion](https://app.notion.com/p/40a141b5836981e28eddeb1dac551b51).

**Refers to:** Output requests, slices

**Rules:** State exact, approximate, directional or unavailable status and applicable regularity/validity conditions. A derivative of a transformed or local branch is not silently advertised as a global physical sensitivity.

**Multiplicity:** 0..1

**Definition:** Available sensitivities, Jacobians, Hessians or linearized models with input/output semantic coordinates, units, derivative convention and the selected branch/active-set context.

**Layer:** Solution

**Owned by:** [Solution member](solution.md#source-40a141b583698178a443d0006d4e600a)

**Concept:** Derivative set

<a id="source-40a141b5836981619fc2f56e768b3161"></a>

## Diagnostic

Source: [Notion](https://app.notion.com/p/40a141b5836981619fc2f56e768b3161).

**Refers to:** Relations, slices, components

**Rules:** Separate a proved violation, a numerical/heuristic finding and an unavailable assessment. Translate to source meaning where supported; retain opaque/native details when no valid translation exists instead of inventing one.

**Multiplicity:** 0..n

**Definition:** A source-attributed observation about contracts, structure, numerical behavior or physical validity, retaining affected semantic identities and the original producer/cause.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Diagnostic

<a id="source-40a141b5836981938396d09024102dff"></a>

## Dual set

Source: [Notion](https://app.notion.com/p/40a141b5836981938396d09024102dff).

**Refers to:** Relations, query constraints

**Rules:** Key by semantic constraint identity and retain qualification, uniqueness/regularity limits and reconstruction evidence. An economic or stream-price interpretation is an additional contextual analysis, not a property of every connection multiplier.

**Multiplicity:** 0..1

**Definition:** Available multipliers associated with original physical relations, bounds or task constraints after a supported reconstruction, with objective, scaling, sign and unit conventions.

**Layer:** Solution

**Owned by:** [Solution member](solution.md#source-40a141b583698178a443d0006d4e600a)

**Concept:** Dual set

<a id="source-40a141b5836981948f10d1d7ca299ca2"></a>

## Estimate

Source: [Notion](https://app.notion.com/p/40a141b5836981948f10d1d7ca299ca2).

**Refers to:** Estimand slices, observation models

**Rules:** Covariance and confidence/credible regions are conditional products, not automatic outputs. Rank deficiency, active bounds, nonregularity, estimated noise or inadequate posterior evidence may require restricted-direction results or withholding intervals with a reason.

**Multiplicity:** 0..1

**Definition:** Available fitted values, posterior products or uncertainty summaries, with observation residuals, assumptions, regularization/prior influence and qualification.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Estimate

<a id="source-40a141b5836981c9bfd6c7611aae1941"></a>

## Provenance

Source: [Notion](https://app.notion.com/p/40a141b5836981c9bfd6c7611aae1941).

**Refers to:** Execution plan

**Rules:** Record enough information to audit what was actually consumed and produced, including incomplete coverage. Provenance identifies a computation and supports reproduction; it does not alone guarantee bitwise reproducibility or equivalent behavior on another implementation.

**Multiplicity:** 1

**Definition:** The exact consumed source/model/case/task revisions and identity projections, derived plan and artifact versions, selected scientific/formulation/numerical choices, actual starts, seeds, provider build/environment evidence and retained completion records.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Provenance

<a id="source-40a141b5836981119471decc17a0714d"></a>

## Representation basis

Source: [Notion](https://app.notion.com/p/40a141b5836981119471decc17a0714d).

**Refers to:** Mesh or grid artifact in provenance

**Rules:** Evaluation is available only where the representation and retained coverage support it. Do not interpolate across unresolved discontinuities, missing segments or invalid phase/branch regions; unsupported values remain unavailable.

**Multiplicity:** 0..1 per trajectory or field value; required when interpolation is offered

**Definition:** The declared representation of a reported trajectory or field, including sampling/interpolation rules, mesh or basis identity, valid subdomains and event-side conventions.

**Layer:** Solution

**Owned by:** [Value set](solution.md#source-40a141b583698176aea2d453f400ae87)

**Concept:** Representation basis

<a id="source-40a141b5836981029300ed3530505b8c"></a>

## Solution

Source: [Notion](https://app.notion.com/p/40a141b5836981029300ed3530505b8c).

**Refers to:** The problem, by semantic and submission hash

**Rules:** Scientific outputs use stable semantic identities and explicit reconstruction/representation metadata; generated artifacts remain distinguishable and inspectable through provenance. Native termination, member qualification, accuracy and final usability are separate. Partial results do not imply all output requests were fulfilled.

**Multiplicity:** Root, or 1 per run in a Study node

**Owns:** [Status](solution.md#source-40a141b583698165a9ccf9ce28c8145b); [Solution member](solution.md#source-40a141b583698178a443d0006d4e600a); [Estimate](solution.md#source-40a141b5836981948f10d1d7ca299ca2); [Statistic set](solution.md#source-40a141b58369813a8300e656a69146f1); [Accuracy report](solution.md#source-40a141b583698188916ac01f3414f387); [Diagnostic](solution.md#source-40a141b5836981619fc2f56e768b3161); [Provenance](solution.md#source-40a141b5836981c9bfd6c7611aae1941); [Validation finding](cross-cutting.md#source-40b141b5836981a0972ec6905ba54ace)

**Definition:** The retained outcome of a selected semantic task and execution attempt: zero or more candidate/qualified members plus coverage, assessments, diagnostics and provenance. Failure, refusal or analysis-only outcomes may have no numerical member.

**Layer:** Solution

**Concept:** Solution

<a id="source-40a141b583698178a443d0006d4e600a"></a>

## Solution member

Source: [Notion](https://app.notion.com/p/40a141b583698178a443d0006d4e600a).

**Refers to:** Nothing

**Rules:** Distinctness and ordering are stated for the selected outputs and tolerances. A finite collection is not automatically exhaustive; whole-run status cannot replace per-member qualification.

**Multiplicity:** 0..n

**Owns:** [Value set](solution.md#source-40a141b583698176aea2d453f400ae87); [Derivative set](solution.md#source-40a141b5836981e28eddeb1dac551b51); [Dual set](solution.md#source-40a141b5836981938396d09024102dff); [Analysis result](solution.md#source-40a141b5836981759d35df77a6e2f160)

**Definition:** One candidate or qualified result branch, such as a root, optimum or Pareto point, with its own values, coverage and validity/acceptance findings.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Solution member

<a id="source-40a141b58369813a8300e656a69146f1"></a>

## Statistic set

Source: [Notion](https://app.notion.com/p/40a141b58369813a8300e656a69146f1).

**Refers to:** Output requests

**Rules:** Errors, confidence/credible intervals and effective sample diagnostics are reported when justified and otherwise marked unavailable. Finite-sample diagnostics do not prove stationarity, mixing or an exact distribution.

**Multiplicity:** 0..1

**Definition:** Requested statistical summaries with sampling/estimation method, retained coverage and any uncertainty or convergence evidence.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Statistic set

<a id="source-40a141b583698165a9ccf9ce28c8145b"></a>

## Status

Source: [Notion](https://app.notion.com/p/40a141b583698165a9ccf9ce28c8145b).

**Refers to:** Relations, bindings

**Rules:** Local nonconvergence is not infeasibility; a candidate is not automatically a usable answer. Certified infeasibility/global bounds require their evidence. Conflict diagnostics may return a found or irreducible set without claiming minimum cardinality, and unavailable translations retain the original cause.

**Multiplicity:** 1

**Definition:** Structured outcome facts: admission/refusal, native termination, candidate availability, original-space qualification, physical closure, requested-product coverage and usability, with reasons and any certificate separately identified.

**Layer:** Solution

**Owned by:** [Solution](solution.md#source-40a141b5836981029300ed3530505b8c)

**Concept:** Status

<a id="source-40a141b583698176aea2d453f400ae87"></a>

## Value set

Source: [Notion](https://app.notion.com/p/40a141b583698176aea2d453f400ae87).

**Refers to:** Slices, scenarios

**Rules:** Record complete, partial or unavailable coverage for each requested output. A failed or stopped run need not cover every request, and absent outputs are never silently filled.

**Multiplicity:** 0..1 per solution member, with explicit coverage

**Owns:** [Representation basis](solution.md#source-40a141b5836981119471decc17a0714d)

**Definition:** The available scientific values for a solution member, indexed by semantic quantity, case and declared point/trajectory/field/sample coverage.

**Layer:** Solution

**Owned by:** [Solution member](solution.md#source-40a141b583698178a443d0006d4e600a)

**Concept:** Value set
