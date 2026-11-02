---
title: "Simulation concepts — Query"
---

# Simulation concepts — Query

<a id="source-40a141b5836981698bc6e81d9492477b"></a>

## Accuracy demand

Source: [Notion](https://app.notion.com/p/40a141b5836981698bc6e81d9492477b).

**Refers to:** Output request

**Rules:** Output accuracy is additional to physical validity, conservation and required original-model acceptance. Numerical solver tolerances are not empirical model accuracy or automatically output-error bounds. Confidence levels apply to statistical claims, and unsupported proof demands are reported explicitly.

**Multiplicity:** 0..n

**Definition:** A typed requirement on an output or assessment, specifying quantity, absolute/relative tolerance and the kind of evidence required: residual/closure, discretization estimate, statistical uncertainty, certified bound or other declared quality.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Accuracy demand

<a id="source-40a141b583698174bd25f3faeb5ec128"></a>

## Admissible set

Source: [Notion](https://app.notion.com/p/40a141b583698174bd25f3faeb5ec128).

**Refers to:** Slices, slots

**Rules:** An unbounded domain is not automatically invalid. Finite bounds, coercivity or compactness are required only for an explicitly claimed existence result or a selected method needing them. Bound purpose/provenance is retained; a physical schedule restriction is semantic, while a numerical control mesh is a separate approximation.

**Multiplicity:** 1 per Decision or Estimand entry

**Definition:** The declared domain for a decision, estimand or uncertain input: sourced ranges, discrete membership, integrality, structural alternatives or restrictions on an allowed trajectory. Nonprobabilistic uncertainty sets carry explicit quantifiers and are distinct from probability laws.

**Layer:** Query

**Owned by:** [Role assignment](query.md#source-40a141b583698129ab10f88ff89b9bd7)

**Concept:** Admissible set

<a id="source-40a141b58369812e915ffff4a3b244c5"></a>

## Answer kind

Source: [Notion](https://app.notion.com/p/40a141b58369812e915ffff4a3b244c5).

**Refers to:** Criteria, datasets, observation models, distributions

**Rules:** The current families are an extensible starting catalog, not a closed universal taxonomy. Compatibility is checked for the selected task and requested products; profile presence does not establish implementation or scientific qualification.

**Multiplicity:** 1

**Definition:** A registered task-profile family such as Solve, Extremize, Infer, Expect or Analyze, with versioned parameters, eligibility and result/assessment contracts. Profiles may be composed explicitly.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Answer kind

<a id="source-40a141b583698159b2f0e9cb7375ecba"></a>

## Criterion

Source: [Notion](https://app.notion.com/p/40a141b583698159b2f0e9cb7375ecba).

**Refers to:** Query expressions, slices

**Rules:** Aggregation and quantifiers are explicit and dimensionally consistent. Scenario-conditioned physical constraints may remain pointwise; risk or robust requirements are not inferred from an Uncertain label. Multiple objectives state their scalarization, lexicographic policy or requested frontier coverage.

**Multiplicity:** 0..n

**Definition:** A scalar objective functional after explicitly declared aggregation over indices, time, scenarios or uncertainty, with sense, units, weight/priority and any risk measure or multiobjective interpretation.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Criterion

<a id="source-40a141b58369811fa53cff172942623d"></a>

## Output request

Source: [Notion](https://app.notion.com/p/40a141b58369811fa53cff172942623d).

**Refers to:** Slices, query expressions, relations, query constraints

**Rules:** Required products need an admitted capability path under the actual formulation, branch and context. Exact and approximate derivatives are distinguished. Output demand does not imply every requested product exists after a failed or partial run; coverage and availability remain explicit.

**Multiplicity:** 1..n

**Definition:** Selected scientific values and observations with scope, sampling/coverage, derivatives, duals or statistics, together with which products are required and which may be returned unavailable with a reason.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Output request

<a id="source-40a141b5836981c4a79edbc9b84b41d1"></a>

## Query

Source: [Notion](https://app.notion.com/p/40a141b5836981c4a79edbc9b84b41d1).

**Refers to:** Nothing

**Rules:** The Query does not silently replace physical laws. Simulation, optimization, fitting and uncertainty studies may reuse one model while also selecting different operating cases, observations or scientific assumptions. Its task contract is extensible rather than a closed five-way enum.

**Multiplicity:** 1

**Owns:** [Role assignment](query.md#source-40a141b583698129ab10f88ff89b9bd7); [Answer kind](query.md#source-40a141b58369812e915ffff4a3b244c5); [Query expression](query.md#source-40a141b5836981b28051e103b611ea4c); [Criterion](query.md#source-40a141b583698159b2f0e9cb7375ecba); [Query constraint](query.md#source-40a141b58369810e8cfceac78adf28da); [Output request](query.md#source-40a141b58369811fa53cff172942623d); [Accuracy demand](query.md#source-40a141b5836981698bc6e81d9492477b); [Regime](query.md#source-40a141b58369817bb1f2e47929237ada); [Assessment requirement](cross-cutting.md#source-40b141b58369819db846e3d9a5250500)

**Definition:** The task-specific question over a selected model and specification: quantity roles, feasibility/design requirements, objectives, uncertainty treatment, assessments and output demands. It references a registered task profile or an explicit composition of profiles.

**Layer:** Query

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** Query

<a id="source-40a141b58369810e8cfceac78adf28da"></a>

## Query constraint

Source: [Notion](https://app.notion.com/p/40a141b58369810e8cfceac78adf28da).

**Refers to:** Slices, query expressions, distributions

**Rules:** Solve may include ordinary feasibility constraints. A design-specification change declares all target equations and released scalar or block variables; count, matching and applicable rank checks determine closure, not a one-target/one-slice assumption. Soft penalties and probabilistic/robust quantifiers require an explicit supported task contract.

**Multiplicity:** 0..n

**Definition:** A task requirement distinct from a physical law: hard point/path/terminal feasibility, a quantified chance or robust constraint, an explicitly penalized soft condition, or a design target paired with released variables/specifications.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Query constraint

<a id="source-40a141b5836981b28051e103b611ea4c"></a>

## Query expression

Source: [Notion](https://app.notion.com/p/40a141b5836981b28051e103b611ea4c).

**Refers to:** Slices, other query expressions

**Rules:** Acyclic; may use expectation and probability operators.

**Multiplicity:** 0..n

**Definition:** A named expression defined for this question only: selectivity, cost, purity, conversion. Keeps the Model untouched.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Query expression

<a id="source-40a141b58369817bb1f2e47929237ada"></a>

## Regime

Source: [Notion](https://app.notion.com/p/40a141b58369817bb1f2e47929237ada).

**Refers to:** Continuous domain

**Rules:** Use an explicit selection or a documented profile default. A steady request suppresses selected temporal accumulation; it does not remove spatial domains. Initial, boundary, phase and periodic consistency conditions follow the selected formulation and may require diagnostic assessment.

**Multiplicity:** 0..1 per evolution coordinate

**Definition:** The requested temporal or evolutionary behavior: steady, transient or periodic under a declared period and forcing convention. It is independent of spatial resolution, unit-local holdup and the numerical integration/transcription route.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Regime

<a id="source-40a141b583698129ab10f88ff89b9bd7"></a>

## Role assignment

Source: [Notion](https://app.notion.com/p/40a141b583698129ab10f88ff89b9bd7).

**Refers to:** Slices

**Rules:** Conflicting effective numerical roles for the same occurrence are refused within one task scope. Roles may change across tasks; shared/case-local identities and linking constraints are explicit. Observation or uncertainty annotations do not silently fix a variable or overwrite its numerical role.

**Multiplicity:** 1 set, 0..n entries

**Owns:** [Admissible set](query.md#source-40a141b583698174bd25f3faeb5ec128)

**Definition:** The effective use of a quantity slice in a selected task and case: fixed input, unknown, decision or estimand, with uncertainty/measurement treatment recorded explicitly. Specified, Unknown, Decision, Uncertain and Estimand are useful presentation labels, not immutable physical natures.

**Layer:** Query

**Owned by:** [Query](query.md#source-40a141b5836981c4a79edbc9b84b41d1)

**Concept:** Role assignment
