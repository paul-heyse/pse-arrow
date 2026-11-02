---
title: "Placement guide"
notion_source: "https://app.notion.com/p/40a141b5836981bbb169cce81d8ecf29"
notion_last_edited: "2026-11-01T20:36:14.450Z"
---

# Placement guide

Source: [Notion](https://app.notion.com/p/40a141b5836981bbb169cce81d8ecf29). Imported 2026-11-02.

**Version 3** · 2026-11-01

Use the meaning and source of a statement to place it. A fact can be reused in more than one task, and its active enforcement can differ from where its source information is stored.

## Placement questions

1. **Is it a reusable definition?** Put the definition or versioned reference in Vocabulary.
2. **Is it a physical assumption or structural relationship of this selected model?** Put it in Model or in the selected realization's contract.
3. **Is it case knowledge or an externally supplied condition?** Record it in Specification, with units, source, uncertainty, and applicability.
4. **Does it state what this computation must find, enforce, assess, or report?** Put that active intent in Query.
5. **Is it a numerical representation or method for the same semantic problem?** Put it in Execution, with any user requirement in MethodRequest.
6. **Does it transfer or orchestrate results across problems?** Put it in Study.
7. **Is it evidence produced by computation?** Put it in Solution with its qualification and provenance.

The test is not whether two numerical runs happen to differ. Ask whether changing the item changes the physical assumptions, the requested answer, or only how that answer is approximated. Several roots can satisfy one unchanged problem; the requested branch must be stated separately from the initialization guess.

## Common placements

| Item | Where it belongs | Why |
| --- | --- | --- |
| Mole fraction in \[0, 1\] | Model: physical domain of the quantity | Defines admissible physical states |
| Property correlation valid from 200 to 600 K | Realization validity contract | A statement about the correlation's supported use, not a universal physical law |
| Vessel rated to 450 K | Source/rating attached to the relevant equipment definition or case, referenced by active Query requirements | Engineering source and active enforcement are distinct |
| Operate below 430 K in this case | Specification records the requirement; Query declares enforced, assessed, or advisory treatment | Meaningful for ordinary simulation as well as optimization |
| Reflux ratio search interval \[1, 5\] | Query: admissible decision domain | Defines this design search |
| Numerical variable clipping or regularization | Execution, recorded explicitly; a physical modification requires a new semantic formulation | Numerical safeguards must not silently redefine the process |
| Kinetic constant value | Specification binding | Case parameter value |
| Kinetic constant to be estimated | Query Estimand, with data and statistical model in Specification | Active estimation intent |
| Danckwerts inlet relation | Model boundary physics | Defines the physical boundary relationship |
| Inlet temperature or wall-temperature history | Specification binding | Supplied boundary data |
| Initial holdup | Specification initial condition | Physical initial data, distinct from a numerical guess |
| A consistent-initialization solve | Execution plan | Procedure to satisfy the declared initial constraints |
| Peng–Robinson definition | Vocabulary constitutive definition | Reusable physical model |
| Selecting Peng–Robinson versus an alternative model | Specification realization selection | Can change the converged physical prediction |
| CSTR versus a distributed reactor | Selected model/realization | Different physical assumptions with only some shared observables |
| Equivalent residual or complementarity formulation | Relation realization or recorded lowering with stated equivalence conditions | Equivalence must be established for the intended solution semantics |
| Mesh, collocation order, integration step | Execution | Numerical approximation of the selected model |
| Required mesh for a comparison study | MethodRequest, recorded in the submission and provenance | A comparison requirement without changing the idealized model |
| Number of trays, fixed | Template configuration instantiated in Model | Discrete equipment structure |
| Number of trays, optimized | Structural Decision over supported alternatives | Changes model structure; may use enumeration or a declared superstructure |
| Purity target by adjusting reflux | Query design specification, including the binding or degrees of freedom released | Active operating/design condition |
| PID controller structure | Model component with signal interfaces | Physical/control-system structure |
| Setpoint history | Specification binding | Supplied operating data |
| Gain tuning | Query Decisions and criterion | Task intent |
| Hourly actuator updates | Admissible trajectory contract and controller/actuator semantics | A physical/control restriction |
| Piecewise-constant approximation of an otherwise continuous control | Execution transcription | Numerical approximation only |
| Feed trajectory stored in a dataset | Specification value source, usable in Solve | Data source does not imply estimation |
| Sensor comparison without fitting | Observation model plus Query assessment request | Validation evidence |
| Scenario probabilities | Specification scenario structure | Knowledge or stated modeling assumption |
| CVaR confidence level | Query criterion aggregation | Risk preference |
| Prior or posterior retained on a parameter | Specification knowledge with explicit active use | May be bound nominally, propagated, or used in later inference |
| Report every 10 seconds | Query output request | Reporting resolution differs from solver time steps |
| “Purity numerical error below 0.0001” | Query evidence/accuracy request | Requires an error estimate or certificate of the specified kind |
| “True plant purity within 0.0001” | Requires a model/measurement uncertainty statement beyond numerical convergence | Cannot be guaranteed by solver tolerance alone |
| Start from a high-temperature guess | Guess in Specification | Numerical aid; excludes semantic branch selection |
| Return the stable high-temperature branch | Query solution-selection requirement | Determines which answers satisfy the question |
| Species labels, experiment IDs, sample timestamps | Typed dataset/index metadata | Identifier columns are not physical quantities and need no invented unit |
| Documentation and authorship | Non-semantic Annotation | Excluded from semantic hash |
| A tagged bound, assertion, or required validity check | The appropriate semantic contract | Not excluded from hashing merely because represented as metadata |

## Deferred extension examples

DFT functionals, molecular ensembles, full CFD turbulence closures, and molecular structures may use the same placement principles. They remain extension examples. The flowsheet frontend needs their external contracts only when a concrete process use case actually integrates them.

## Difficult cases

When an item appears to belong in two places, separate its **source** from its **active use**. A manufacturer's rating is reusable knowledge; enforcing it in a solve is task intent. A measured parameter distribution is knowledge; treating it as a fixed nominal input or propagating it is a task choice. A model's physical admissibility and an optimizer's search bounds should never overwrite one another silently.
