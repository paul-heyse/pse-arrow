---
title: "Worked examples"
notion_source: "https://app.notion.com/p/40a141b5836981c6b998fa0e81b2d8b5"
notion_last_edited: "2026-11-01T20:36:21.085Z"
---

# Worked examples

Source: [Notion](https://app.notion.com/p/40a141b5836981c6b998fa0e81b2d8b5). Imported 2026-11-02.

**Version 3** · 2026-11-01

These conceptual examples exercise the formulation contract. They are design illustrations rather than claims that the corresponding implementations, numerical results, or global guarantees already exist in pse-arrow.

The immediate frontend is flowsheet process modeling. Examples 1–3 exercise thermodynamic states, dynamic process models, estimation, and optimization. Example 6 illustrates a later control workflow over the same process model. Examples 4–5 preserve useful integration thinking while explicitly deferring full CFD and molecular/quantum simulation engines.

| Example | Profile | Purpose and scope |
| --- | --- | --- |
| 1. Isothermal three-phase flash | Solve | Property/state scope, physical phase equilibrium, equivalent-formulation conditions, and qualified stability evidence |
| 2. Kinetic parameters from four batch experiments | Infer | Dynamic model reuse, shared parameters, observation alignment, and qualified estimation uncertainty |
| 3. Best temperature profile for the same reactor | Extremize | Reuse of process physics with new case/task intent; variable horizon and explicit actuator schedule |
| 4. CFD mixing vessel inside a steady flowsheet | Solve | Deferred integration illustration: shared external observables, capability-limited substitutions, conservative interface mapping |
| 5. DFT to microkinetics to a packed-bed reactor | Study | Deferred molecular integration: external scientific results, material/site alignment, uncertainty assumptions, distinct minima and saddle searches |
| 6. Nonlinear MPC with moving-horizon estimation | Study of Infer and Extremize | Later process/control workflow: aligned windows, actual applied moves, uncertainty transfer, and failure handling |

## What these examples establish

- The physical model can often be reused while Specification and Query change.
- A shared interface permits composition, not a guarantee that physical alternatives predict identical answers.
- Equivalent mathematical formulations require conditions and consistent solution-selection semantics.
- Structural validity, numerical convergence, and global/physical claims must be distinguished.
- Data edges preserve units, indices, conventions, and the declared meaning of uncertainty.
- Model capabilities constrain regime and observable requests.
- Numerical guesses never define a hidden branch-selection policy.

The companion flowsheet domain design supplies the equipment, material, thermodynamic-state, and connection concepts needed to author these tasks consistently. Broad general examples should not delay the smaller end-to-end process cases needed to validate that frontend.

## Examples

[1. Isothermal three-phase flash](three-phase-flash.md)

[2. Kinetic parameters from four batch experiments](batch-parameter-inference.md)

[3. Best temperature profile for the same reactor](reactor-temperature-optimization.md)

[4. CFD mixing vessel inside a steady flowsheet](cfd-flowsheet-coupling.md)

[5. DFT to microkinetics to a packed-bed reactor](dft-to-reactor.md)

[6. Nonlinear MPC with moving-horizon estimation](mpc-and-state-estimation.md)
