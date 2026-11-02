---
title: "5. DFT to microkinetics to a packed-bed reactor"
notion_source: "https://app.notion.com/p/40a141b583698104990aef1a40c1bafb"
notion_last_edited: "2026-11-01T20:36:25.263Z"
---

# 5. DFT to microkinetics to a packed-bed reactor

Source: [Notion](https://app.notion.com/p/40a141b583698104990aef1a40c1bafb). Imported 2026-11-02.

**Version 3** · 2026-11-01

> Deferred molecular/quantum integration illustration. The present flowsheet domain consumes versioned reaction/property information through declared contracts. It does not need to implement electronic structure, molecular simulation, or a universal quantum problem ontology before process models work.

Adsorption and transition-state calculations inform rate constants; a microkinetic model then contributes to a packed-bed process model. The Study illustrates scientific-data provenance, index/basis alignment, and uncertainty transfer across independently specialized tools.

## Study structure

| Stage | Intended computation | Contract passed onward |
| --- | --- | --- |
| Molecular/surface calculations | Adsorbate minima and explicitly specified transition-state searches | Energies, geometries, charge/spin conventions, model/settings, convergence and stationarity evidence |
| Frequency/stability analysis | Vibrational or Hessian information with the appropriate mass/coordinate convention | Modes, imaginary-mode count, treatment of constraints and near-zero modes |
| Rate construction | Declared transition-state/statistical-mechanical model | Rate expressions/parameters with units, standard states, thermochemical conventions, validity, and uncertainty |
| Microkinetics | Reaction/site balances and rate relationships | Rates on a declared site/area/volume basis, supported conditions, and derivatives if available |
| Packed-bed process model | Axial balances with feeds, geometry, transport/thermal assumptions, and reaction source terms | Conversion, profiles, conservation and numerical evidence |
| Uncertainty study | Propagation under an explicitly assumed or inferred joint uncertainty model | Statistics with estimator uncertainty and limitations |

A sweep can index CO\*, O\*, O2\*, CO2\*, and selected transition-state candidates on a specified Pt surface. Labels are aligned explicitly; matching text alone does not establish physical identity or the same reference state.

## Electronic-structure request and evidence

A chosen level of theory, charge/spin state, surface geometry, boundary conditions, and constrained atoms are part of the scientific model. Settings such as a 450 eV plane-wave cutoff or a 4×4×1 k-point grid can be required MethodRequest entries for a comparison. These illustrative settings are not asserted to be sufficient for convergence.

An adsorbate geometry minimum and a first-order transition state are different mathematical targets:

- A geometry minimum seeks a stationary point with the appropriate positive-curvature behavior after handling physical symmetries and constraints.
- A transition-state search seeks a saddle with the declared unstable mode structure; ordinary unconstrained energy minimization does not produce that target by adding an “index 1” label.
- An explicit stationary/saddle problem and a corresponding analysis profile, or a specialized external evaluator, supplies the needed contract.

Electronic self-consistency and a geometry optimizer's local convergence do not establish a global electronic or nuclear ground state. Returned evidence distinguishes the requested physical state from what the numerical method has established.

## Rate and basis alignment

Energy differences become rate parameters only through declared scientific assumptions. Zero-point/thermal corrections, standard states, degeneracies, pressure/concentration conventions, and treatment of constrained/imaginary modes are explicit.

A value in eV per molecular event must be converted consistently before use in a molar process rate expression. Surface-site rates need site density and area or volume conversions before entering a reactor balance. Reversibility and thermodynamic-cycle constraints are preserved where the reaction model requires them.

## Uncertainty transfer

An assumed 0.1 eV error scale may be used for an illustrative sensitivity scenario, but it is an explicit modeling assumption, not a measured universal DFT uncertainty. Shared energy references create correlated barriers. Independent marginal errors must not be introduced without justification.

Data edges preserve the joint covariance or sampling representation, align species/reaction/site labels, and state whether the downstream value is a nominal binding, prior, propagated uncertainty, or a surrogate-training datum.

## Process computation

The packed-bed model may use a one-dimensional axial domain, gas and surface state variables, site balances, heat/transport closures, and a declared rate source. Whether it is steady or dynamic depends on its own capabilities and inventories.

Expect can then request conversion statistics, quantiles, or sensitivity indices under the chosen uncertainty model. Sampling error, numerical solution error, and the assumed molecular/model discrepancy are reported separately.

This example retains the original multiscale idea while correcting the claim that transition-state search is ordinary minimization and avoiding unsupported global ground-state or uncertainty guarantees.
