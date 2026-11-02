---
title: "Simulation concepts — Specification"
---

# Simulation concepts — Specification

<a id="source-40a141b5836981bb821ee52575e7f346"></a>

## Binding

Source: [Notion](https://app.notion.com/p/40a141b5836981bb821ee52575e7f346).

**Refers to:** Slice, value source

**Rules:** Resolve overlays to one effective assignment per selected element, rejecting unexplained conflicts. Fixed values, initial conditions and numerical starts remain distinct. A variable released for design or fitting loses its incompatible fixed binding in that task's effective case; DAE consistency and rank are checked separately.

**Multiplicity:** 0..n

**Owns:** [Value source](specification.md#source-40a141b58369812a878bf0170c34af4d)

**Definition:** A hard case assignment to a selected quantity occurrence or domain slice: fixed value, initial or boundary condition, input trajectory or spatial profile, together with its source and precedence.

**Layer:** Specification

**Owned by:** [Scenario](specification.md#source-40a141b583698126a47fc30d7116321a)

**Concept:** Binding

<a id="source-40a141b5836981d08b68ff59b0ba8c17"></a>

## Dataset

Source: [Notion](https://app.notion.com/p/40a141b5836981d08b68ff59b0ba8c17).

**Refers to:** Nothing

**Rules:** Physical measurement columns carry quantity kinds, units and relevant reference conventions; label and metadata columns carry their appropriate nonphysical types. Observation alignment, missingness and uncertainty are explicit.

**Multiplicity:** 0..n

**Definition:** Versioned measured or reference data with a typed index and typed columns, including physical measurements, categories, timestamps, provenance and quality fields.

**Layer:** Specification

**Owned by:** [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d)

**Concept:** Dataset

<a id="source-40a141b58369818bb73fc96cef0fb17a"></a>

## Distribution

Source: [Notion](https://app.notion.com/p/40a141b58369818bb73fc96cef0fb17a).

**Refers to:** Slices

**Rules:** Scope, dependence, support and source are explicit. Validity or normalization is checked when available and otherwise reported as assumed, unsupported or unresolved; arbitrary properness is not universally decidable. A realized scenario value does not erase its source distribution.

**Multiplicity:** 0..n

**Definition:** A probabilistic law on selected quantities, possibly joint, empirical, conditional or a random process. A prior, input distribution and posterior are different uses of that law; a nonprobabilistic robust uncertainty set is represented separately by an admissible-set contract.

**Layer:** Specification

**Owned by:** [Scenario](specification.md#source-40a141b583698126a47fc30d7116321a)

**Concept:** Distribution

<a id="source-40a141b5836981b1b8cad15f29cb98ea"></a>

## Guess

Source: [Notion](https://app.notion.com/p/40a141b5836981b1b8cad15f29cb98ea).

**Refers to:** Slice, value source

**Rules:** A start may change convergence or which local solution is found and must be retained in submission/attempt provenance. If a particular branch is required, state that requirement and its continuation/selection policy explicitly; excluding guesses from a semantic projection does not make branch choice irrelevant.

**Multiplicity:** 0..n

**Owns:** [Value source](specification.md#source-40a141b58369812a878bf0170c34af4d)

**Definition:** A numerical starting value or seed for a solve, often obtained from a qualified prior result. It is separate from a physical initial condition, hard binding or semantic branch-selection requirement.

**Layer:** Specification

**Owned by:** [Scenario](specification.md#source-40a141b583698126a47fc30d7116321a)

**Concept:** Guess

<a id="source-40a141b5836981c7a84ee7dbaf118f76"></a>

## Noise model

Source: [Notion](https://app.notion.com/p/40a141b5836981c7a84ee7dbaf118f76).

**Refers to:** Dataset columns, slices

**Rules:** Positive-definite covariance is required only by methods that invert it as such. A singular/degenerate model needs an explicit supported subspace or deterministic-constraint treatment. Noise assumptions, fitted variance and regularization are retained in estimate and interval validity.

**Multiplicity:** 1

**Definition:** The error model of an observation: distribution or supported loss, covariance/dependence, heteroscedasticity and any outlier treatment; selected noise parameters may be estimated jointly.

**Layer:** Specification

**Owned by:** [Observation model](specification.md#source-40a141b583698134b3becba0c6ce7293)

**Concept:** Noise model

<a id="source-40a141b583698134b3becba0c6ce7293"></a>

## Observation model

Source: [Notion](https://app.notion.com/p/40a141b583698134b3becba0c6ce7293).

**Refers to:** Dataset, slices, scenarios

**Rules:** Consumed observations resolve to the selected data revision and model meaning. Missing, censored, correlated or out-of-domain observations require declared treatment; observations do not automatically become hard process specifications.

**Multiplicity:** 0..n

**Owns:** [Noise model](specification.md#source-40a141b5836981c7a84ee7dbaf118f76)

**Definition:** An explicit mapping from model quantities or derived observations to measured data, with index/time/location alignment, sensor transformations and the declared noise model.

**Layer:** Specification

**Owned by:** [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d)

**Concept:** Observation model

<a id="source-40a141b58369817d8931d3c11b397675"></a>

## Realization selection

Source: [Notion](https://app.notion.com/p/40a141b58369817d8931d3c11b397675).

**Refers to:** Slot, alternative, scenario

**Rules:** Every demanded slot has a resolved choice or a supported structural decision. Scientific fidelity and thermodynamic/property conventions are semantic selections; solver choice, mesh/order and tolerances are separate numerical realization inputs.

**Multiplicity:** 0..n

**Definition:** The explicit selection of a scientific-model, formulation or structural alternative for a typed slot, scoped to a model revision or operating case as declared.

**Layer:** Specification

**Owned by:** [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d)

**Concept:** Realization selection

<a id="source-40a141b583698126a47fc30d7116321a"></a>

## Scenario

Source: [Notion](https://app.notion.com/p/40a141b583698126a47fc30d7116321a).

**Refers to:** Nothing

**Rules:** Effective hard inputs are complete for the selected task/profile. Shared quantities and cross-scenario conditions are linked explicitly; scenario creation does not duplicate or mutate the authoritative model.

**Multiplicity:** 1..n

**Owns:** [Binding](specification.md#source-40a141b5836981bb821ee52575e7f346); [Guess](specification.md#source-40a141b5836981b1b8cad15f29cb98ea); [Distribution](specification.md#source-40a141b58369818bb73fc96cef0fb17a)

**Definition:** A named operating case or experiment under a selected model revision, with effective bindings, schedules and uncertainty context. Shared and case-local quantities are declared independently.

**Layer:** Specification

**Owned by:** [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d)

**Concept:** Scenario

<a id="source-40a141b5836981e3adccdca7af9b53f4"></a>

## Scenario structure

Source: [Notion](https://app.notion.com/p/40a141b5836981e3adccdca7af9b53f4).

**Refers to:** Scenarios, slices

**Rules:** Probabilities are normalized when the selected probability model requires it; arbitrary experiment/importance weights are not probabilities. A shared physical quantity uses one linked value in the declared scope, while stage-dependent decisions remain explicitly distinct.

**Multiplicity:** 0..1 (absent means one scenario)

**Definition:** The scenario/experiment index, its probability or importance semantics, and explicit shared-quantity, recourse or nonanticipativity links between cases.

**Layer:** Specification

**Owned by:** [Specification](specification.md#source-40a141b583698104b5e7f14100c5b90d)

**Concept:** Scenario structure

<a id="source-40a141b583698104b5e7f14100c5b90d"></a>

## Specification

Source: [Notion](https://app.notion.com/p/40a141b583698104b5e7f14100c5b90d).

**Refers to:** Nothing

**Rules:** Owns at least one selected scenario and records the source and precedence of effective bindings. Knowledge may include sourced admissible ranges; bounds are not assigned to a layer solely because they are bounds. Query roles/requirements and numerical method controls remain separately identified.

**Multiplicity:** 1

**Owns:** [Scenario](specification.md#source-40a141b583698126a47fc30d7116321a); [Scenario structure](specification.md#source-40a141b5836981e3adccdca7af9b53f4); [Dataset](specification.md#source-40a141b5836981d08b68ff59b0ba8c17); [Observation model](specification.md#source-40a141b583698134b3becba0c6ce7293); [Realization selection](specification.md#source-40a141b58369817d8931d3c11b397675)

**Definition:** The selected operating-case knowledge: values, observed data, schedules, uncertainty/prior information and explicit scientific-model or parameterization choices. Physical domains, operating/design constraints and numerical safeguards retain distinct purposes and provenance.

**Layer:** Specification

**Owned by:** [SimulationProblem](problem-root.md#source-40a141b58369813f9bc6dcfae91d96c7)

**Concept:** Specification

<a id="source-40a141b58369812a878bf0170c34af4d"></a>

## Value source

Source: [Notion](https://app.notion.com/p/40a141b58369812a878bf0170c34af4d).

**Refers to:** Dataset, databank entry, Solution

**Rules:** Resolve and freeze the actual consumed revision, selection and conventions for each admitted task. Exact immutable references may be retained instead of copying all data into the problem. Live sources require a defined snapshot/observation boundary; a source label alone is insufficient.

**Multiplicity:** 1

**Definition:** The provenance and immutable content or snapshot reference supplying a literal, dataset value, databank parameterization or qualified upstream output.

**Layer:** Specification

**Owned by:** [Binding](specification.md#source-40a141b5836981bb821ee52575e7f346); [Guess](specification.md#source-40a141b5836981b1b8cad15f29cb98ea)

**Concept:** Value source
