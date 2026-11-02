---
title: "4. CFD mixing vessel inside a steady flowsheet"
notion_source: "https://app.notion.com/p/40a141b5836981608647f7043cdfad49"
notion_last_edited: "2026-11-01T20:36:24.415Z"
---

# 4. CFD mixing vessel inside a steady flowsheet

Source: [Notion](https://app.notion.com/p/40a141b5836981608647f7043cdfad49). Imported 2026-11-02.

**Version 3** · 2026-11-01

> Deferred integration illustration. The flowsheet frontend must be able to accommodate an external specialized unit through a declared interface; building a full CFD domain or solver is not a prerequisite for the process-modeling implementation.

Two feeds enter a mixing vessel, and its outlet feeds a separator. The vessel can be represented by a lumped model or an external distributed model. Those are different physical assumptions sharing some external observables, not guaranteed equivalent versions of one answer.

**Profile:** Solve for the selected supported regime.

## Shared interface and alternative-specific detail

| Contract | Shared or alternative-specific? |
| --- | --- |
| Inlet component flow, enthalpy flow, pressure convention, material identity/basis | Shared public boundary contract |
| Outlet component flow, enthalpy flow, pressure convention | Shared where both alternatives support them |
| External heat/work transfer and inventory behavior | Shared only if explicitly included by both alternatives |
| Internal concentration, velocity, pressure, or turbulence fields | Distributed-model observables |
| Spatial coefficient of variation over an outlet | Distributed observable requiring field and weighting definitions |
| Perfect-mixing assumption | Lumped model assumption, not an observed CFD mixing-quality result |

A CSTR replacement can support a screening calculation using the shared outlet contract. It cannot provide a resolved concentration field. A mixing coefficient of variation of zero in a perfectly mixed model is imposed by that model's assumption; it is not evidence that the actual vessel mixes perfectly.

A Query that requires internal fields is rejected or explicitly changed when selecting the lumped alternative. No implicit substitution of unavailable outputs occurs.

## Distributed-model example

The distributed alternative may expose a region Ω with vessel/impeller geometry, inlet/outlet/wall regions, species transport, momentum/continuity equations, and a selected constitutive closure. Geometry and physical wall conditions are semantic; numerical mesh and discretization remain execution details.

For illustration, impeller speed is 120 rpm, vessel diameter 1.2 m, and height 1.5 m. These values are not a validated design. The chosen closure and boundary conditions require their own applicability evidence.

A steady RANS closure and an LES model are not interchangeable settings of one identical steady problem. LES generally supplies time-resolved behavior and statistical observables under its stated regime. Any comparison defines the averaging interval, outputs, and physical assumptions explicitly.

## Conservative interface mapping

A distributed-to-lumped outlet mapping integrates component flow and energy flow consistently. A flow-weighted concentration average alone does not establish all necessary conservation or state compatibility.

The mapping declares:

- Species/component identity and phase compatibility.
- Mass or molar basis and reference conventions.
- Component-flow and enthalpy-flow integrals.
- Pressure convention and any momentum/velocity assumptions.
- Geometry, normal orientation, and sign conventions.
- Supported flow direction, including whether reversal is allowed.
- The property/state reconstruction used by the receiving separator.

A uniform inlet profile is a physical boundary-profile assumption. It conserves the requested integrated flows only when defined with compatible area, velocity/density, composition, and energy data.

## Query and execution

A common screening query requests separator product composition, conservation diagnostics, and selected outlet values. A distributed-only query additionally requests field coverage and a defined mixing statistic.

Execution may use a monolithic or partitioned coupling strategy if the evaluators support it. Partitioned convergence must be assessed against the coupled interface equations; merely solving each component once does not solve a recycle/coupled problem.

Requested numerical error in product purity is distinct from model discrepancy between lumped and distributed alternatives. Mesh refinement may estimate error for the selected distributed model. It does not prove that either physical model represents the real vessel within that tolerance.

This example preserves the extension path while correcting the earlier claim that switching to a CSTR required no change even when the Query requested CFD-only fields.
