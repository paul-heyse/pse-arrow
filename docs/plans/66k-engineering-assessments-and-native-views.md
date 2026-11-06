---
title: "Plan66k: Engineering assessments and native views"
status: planned
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/66-integrated-simulation-target.md
doc_topics: [current-work, design, validation]
---

# Engineering assessments and native views

## Complete remaining outcome

This companion of the sole go-forward [Plan66](66-integrated-simulation-target.md)
owns remaining engineering cost/environment/utility assessments and useful native
stream/result/flowsheet/trajectory/uncertainty views. These are complete scientific
consumer outcomes, not optional presentation polish. It consumes physical and analysis
products from 66g–66j, shared meanings from 66a, protected data from 66e, and exact
population/independent expectations from [66l](66l-required-population-and-assembled-acceptance.md).
No earlier plan remains an execution prerequisite or second status home.

**Interface-checked baseline:** main `66341514d` already has
[authored costing](../../packages/reference/process/models/costing.pse), a
[costed exchanger](../../packages/reference/campaign/models/exchanger-costing.pse),
[protected result inspection](../../crates/pse-runtime/src/workflow/result_inspection.rs)
and [scoped local analysis](../../crates/pse-runtime/src/workflow/local_analysis.rs).
Costing source distinguishes mathematical domain from empirical applicability, including
unknown/extrapolated support and currency/base period. Preserve these implemented
distinctions. Existing arithmetic and generic result projections do not establish all
released costing methods, dedicated environmental/utility workflows or native rendered
views. Existing inspection/adoption mechanisms need only their missing consumer changes
and receiving evidence.

The target specifically realizes review
[I09 engineering assessment and native views](../plan_improvement_reviews/plan64-65-integrated-target-and-library-leverage-2026-11-06.md#I09-G06),
with I01/I02/I07/I10 and scientific S01/S04/S07. It extends authored composition and
existing native products, rather than introducing a life-cycle database, generic
assessment engine or parallel result authority.

## Assessment meaning and finite population

| Outcome | Required scope, ordinary inputs and interpretation |
|---|---|
| Equipment/process costing | Selected SSLW equipment correlations, power QGESS/NETL and JSON coefficients, TSA DAC costing and HDA/methanol/SCPC consumers; actual capacity/design/material/geometry/pressure/temperature and applicability limits, currency/base year/index, installation/operating basis and contributor ownership |
| Integrated accounting | Capital versus operating/resource cost, utilization/physical operating period, aggregation/system boundary and allocation; shared contributors cannot be counted twice; design and operation results retain their original selected physical products |
| Environmental assessment | Selected native environmental consumer with declared inventory, factor/method source/version, physical boundary, allocation and characterization units/time basis; a cost contribution is not an environmental characterization factor and general LCA equivalence is not assumed |
| Utility/heat integration | Required released utility-minimization and selected plant consumers; heat levels, hot/cold stream identity, feasible duties, chosen minimum approach/shifted-temperature conventions and target interpretation; a duty sum alone is not a pinch/utility target |
| Native engineering views | Stream tables and flowsheet/result visualization, physical trajectories and local sensitivity/uncertainty views; source/stream/species/phase identity, units/datum, time/member/point context, availability/partiality and provenance; independent exported/read interpretation |
| Diagnostics and extension views | Selected diagnostic/scaling/initialization/custom authoring workflows and moving-bed diagnostics; structural zero-member analysis, numerical findings and successful point products remain separate; editable views use the admitted typed edit path |

The source/configuration/input provenance transferred to 66l is
`git:66341514d:docs/plans/64h-required-process-population.md`, including direct costing
callables, modern/compatibility consumers and notebook visualizer roles. Generated
PFD SVGs, stream CSVs and plot series are output expectations, not ordinary physical
inputs. Source wrapper/rendering duplicates do not add scientific models, but their
distinct requested table/plot/report outcome cannot disappear when wrappers retire.
Full GUI imitation is outside the prerequisite target; selected useful native
visualization/reporting remains required.

An assessment request identifies the exact physical result, chosen method and factor
source, accounting boundary, period/utilization/allocation and demanded product. Its
output retains physical contributors and original identity, with qualification or
withholding appropriate to empirical support. Arithmetic may be available while a
qualified purchase-price or environmental conclusion is unavailable. Missing required
coefficients/factors stay unavailable, not zero. Unknown support cannot become in-range
merely because a mathematical expression evaluates.

## Native composition, data and rendering

Extend existing authored cost/quantity/accounting contributions where sufficient.
Different assessment rules may require a bounded authored/native operation, but a
collection of sums does not justify a universal ontology. Preserve inventory/conservation,
economic allocation, environmental characterization and heat targeting as distinct
semantic responsibilities under [66a](66a-scientific-operations-and-product-semantics.md).
Keep ordinary factor/data admission at its scientific source owner; oracle/reference
tables remain validation evidence unless independently admitted as ordinary science.

[66e](66e-protected-data-and-result-delivery.md) owns exact selected protected reading,
canonical effects and original interpretation. DataFusion selected factual joins and
aggregations can serve reporting, while native assessment owns units, allocation,
qualification and unavailable products. Do not make SQL infer science or mirror it in
a second database. Reuse immutable selected physical results: changing only currency
period, utilization, allocation or factor revision should invalidate the relevant
assessment product without rerunning an unchanged physical solve.

Native numerical composition can reuse Symbolica 3.0.1 multi-output evaluator/instruction
preparation through [66d](66d-mathematical-representation-and-numerical-execution.md)
where actual repeated nonlinear assessment/response demands justify it. ndarray 0.16.1
views/Zip support selected batch assembly; units and source coordinates remain owned
outside array indices. A currency conversion or tabulation is not automatically a new
solver problem. Preserve existing faer/SVD-derived uncertainty qualification when a view
consumes it; no rendering or aggregation step can manufacture covariance or certainty.

Views consume selected native products without executing science again, issuing product
permission or silently interpolating unsupported regions. A trajectory needs physical
time and interpolation/domain policy; an uncertainty band declares its statistical
meaning; a flowsheet edge retains original port/stream identity. Unavailable siblings
remain visible without hiding available independent products. Editing routes through
the same typed source/admission operations and invalidation as other clients.

Choose a rendering library only after required table, plot, graph, interaction and
export requirements are fixed in K04. Compare an existing native client projection,
an established plotting/graph library and external exported viewing on complete
responsibility, including headless output and original metadata fidelity. This plan
selects no unresearched graphical stack. Context-specific exact-version primary-source
assessment should resolve the actual choice; an absent overview is not absence of a
useful library. Avoid building plotting/layout machinery merely to avoid a dependency.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| K01 | planned | 66l selected methods/callables/consumer and input roles; existing costing/inspection source | Remaining cost/environment/utility/view outcomes, factor provenance and scientific qualification distinctions are concretely selected without recreating delivered arithmetic |
| K02 | planned | Selected working 66g/66i physical consumer; K01 ordinary coefficients and 66a product slice | Missing equipment/process costing and integrated accounting deliver scoped original contributions, applicability and period/currency semantics through protected reading |
| K03 | planned | Selected K01 environmental/utility methods and factor closure; required physical products | Selected environmental and utility assessments deliver declared boundary/allocation/time/heat-level interpretation and independent expectations, starting with one contrasting complete case |
| K04 | planned | Existing protected inspection plus actual K01 table/plot/graph requirements; 66b/66f client slice | Native stream/flowsheet/result and trajectory/uncertainty views preserve source/physical identity, partiality, time and scope without rerun or permission escalation |
| K05 | planned | Selected K02–K04 native producers and 66e protected/export path | Assessment/view clients and independent readers consume the same original products; typed edits use admitted operations; replaced aggregation/presentation interpretation and fixtures are deleted |
| K06 | planned | Complete selected consumer journeys and 66l independent expectations | Costed physical and contrasting assessment consumers complete native assessment, exact read, useful native view and independent export acceptance with truthful support and provenance |

K01 is a bounded scientific-method and actual-view selection, not a reason to defer
mandatory outcomes indefinitely. Start K02 from the delivered costed exchanger to close
missing receiving/qualified-period behavior, then its required HDA/plant/TSA methods
as physical dependencies become ready. Existing exchanger arithmetic needs no rewrite.
K03 selects one environmental or utility case as the first contrasting complete
assessment, then closes the remaining selected environmental and mandatory released
utility methods; the first case is not its completion boundary. K04 can develop
ordinary stream/result views from accepted steady products while trajectory/uncertainty
views consume their respective ready 66h/66j slice; neither waits for all families.

## Consumer cutover and planned assurance

Each delivery includes operation admission, exact selected source/physical context,
native assessment or observation, scoped diagnostics, protected retained reading,
typed client/view and independent export. [66b](66b-contracts-generation-and-interpretation.md)
owns declarations/adoption; [66c](66c-preparation-execution-and-service-lifecycles.md)
owns performing/cancellation lifetime where work occurs;
[66f](66f-developer-operations-clients-and-evidence.md) owns the actual client journey.
New output names do not imply a new type/service for every concept. Delete replaced
ad-hoc sums, parallel permission interpretations and presentation adapters after all
selected callers migrate and revealing controls pass; keep scientific negative cases
at their surviving owner.

**Planned verification:** distinguish correct arithmetic from qualified empirical
conclusion. Challenge double-counted contributors, wrong currency/base period,
utilization/time units, allocation and missing or changed factor provenance; unknown/
extrapolated support stays truthful. Independently compute selected cost and utility/
environment expectations rather than copying the production aggregation. For heat
targets vary minimum approach, feasible heat levels and stream identity so an incorrect
duty-only implementation fails. Change an assessment-only input and establish proper
derived invalidation without unrelated physical execution.

Views must detect reordered species/streams, different datum, wrong point/time/member,
missing output, failed siblings and structural analysis without state. Check labels,
units, original provenance and uncertainty meaning in native rendered and independent
exported outcomes; compare scientific meaning rather than pixel-for-pixel upstream
styling. Observational reads never grant current execution/product permission. 66l
owns assembled finite coverage and scope-end qualification. No graphical stack,
numerical campaign or benefit is newly qualified by this authoring change.
