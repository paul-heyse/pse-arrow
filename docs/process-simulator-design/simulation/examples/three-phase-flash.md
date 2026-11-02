---
title: "1. Isothermal three-phase flash"
notion_source: "https://app.notion.com/p/40a141b58369814a9cd7cb6507f83e99"
notion_last_edited: "2026-11-01T20:36:21.893Z"
---

# 1. Isothermal three-phase flash

Source: [Notion](https://app.notion.com/p/40a141b58369814a9cd7cb6507f83e99). Imported 2026-11-02.

**Version 3** · 2026-11-01

A hydrocarbon/water feed at fixed temperature and pressure is allowed to split into vapor, an organic-rich liquid, and an aqueous-rich liquid. This illustrates process-state scope, phase-selection semantics, and qualified equilibrium evidence.

**Profile:** Solve. **Scope:** process thermodynamics. This is a conceptual contract, not a validated three-phase benchmark or endorsement of an uncalibrated property package for this mixture.

## Physical model and case

| Part | Specification |
| --- | --- |
| Chemical system | Methane, propane, n-butane, water; permitted phases V, L1, L2 with explicit phase/component validity |
| Thermodynamic definition | A selected, versioned property package capable of the intended mixture and phases; Peng–Robinson with pinned interaction data is an illustrative candidate whose adequacy requires validation |
| State basis | Molar feed and phase flow rates, phase mole fractions, absolute temperature and pressure, consistent Gibbs-energy/reference conventions |
| Feed | 100 mol/s; overall mole fractions (0.40, 0.25, 0.15, 0.20); 300 K; 20 bar |
| Unknowns | Phase molar flows and compositions; phase presence/absence handled explicitly |
| Conservation | One component balance per independent component and composition normalization on present phases |
| Parameter source | Pinned interaction/property data with provenance and applicability |
| Requested equilibrium | Stable equilibrium under the selected physical package, with the attained evidence level reported |

For a phase with zero amount, some composition-dependent outputs may be undefined or non-unique. The interface declares that status rather than returning arbitrary values as physically identified phase compositions.

## Alternative mathematical formulations

One formulation minimizes total Gibbs energy on a fixed amount basis, or the equivalent Gibbs-flow functional on the stated steady feed basis, subject to conservation and phase admissibility.

Another uses chemical-potential/fugacity conditions and phase-appearance complementarity. These formulations can represent the same stable thermodynamic equilibrium **only with the necessary stability and global-selection conditions**. Stationarity and complementarity alone may admit metastable or otherwise unwanted solutions.

A realization alternative therefore records:

- Shared material/state/reference conventions.
- Admissible phase and composition domain.
- Required equilibrium and phase-stability semantics.
- Solution multiplicity and selection.
- Numerical evidence available for those requirements.

## Query and outputs

The Query requests phase flows and compositions, phase presence, balance residuals, and the available stability evidence. A derivative such as vapor-flow sensitivity to temperature is requested only with its valid branch/phase regime stated. At a phase transition it may be non-smooth, one-sided, or unavailable under the selected derivative contract.

The Query may request one admissible stable solution or a supported characterization of alternatives. A numerical initial guess may help locate a solution but does not define stability or branch semantics.

## Execution and evidence

Execution may solve a constrained Gibbs problem, a complementarity formulation, or another declared equilibrium algorithm. Tangent-plane methods can detect instability and provide stability evidence under the implemented search conditions. A local search that finds no negative direction is not automatically a global certificate.

Report separately:

- Material and normalization residuals.
- Numerical convergence and conditioning.
- Phase-stability tests performed and their scope.
- Globality certificate, if actually available.
- Remaining ambiguity or unsupported requirements.

If globally certified equilibrium is mandatory and available methods cannot certify it, the task reports that limitation. It does not relabel a locally stationary point as a proven global solution.

This corrects the earlier assertion that the two mathematical faces automatically give the same answer and that a generic stability test necessarily confirms a global minimum.
