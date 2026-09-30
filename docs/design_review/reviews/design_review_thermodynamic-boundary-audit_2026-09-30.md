# Plan 23 thermodynamic knowledge-boundary audit

## 1. Scope, drivers and coverage

Author review by Codex, 2026-09-30. Design tier, conformance purpose; Core 3.3,
process-simulator profile 1.3 and the pse-arrow binding. The boundary is Plan 23's
changed production Rust, registry, package declarations and their Python consumers.
Concurrent agent policy changes and Plan 24 are excluded. The disposition owner is
[Plan 23](../../plans/23-thermodynamic-domain-and-campaign.md#current-execution).

The drivers are package-only scientific extension, shared admission for inline and
binary banks, immutable revision reuse, owned physical conventions and truthful native
results. Architectural fitness is **satisfied at inspected implementation scope**.
Behavioral qualification passes the named local production suites and whole-seed run
recorded below. The overall decision is **Accept-scoped** to the implemented contracts
and exercised production behavior. Uncompleted measurements establish no performance
claim and do not leave the completed functional plan open.

## 2. Decomposition, ownership and dependencies

Every changed production crate belongs to an existing responsibility; no crate is added.

| Changed owner | Generic gap or removal | Consumed contract and local evidence |
|---|---|---|
| `pse-authoring` | Unit/type syntax, entity and attribute provenance, analysis time composition, native primitive fixture options | Generated declarations; parser/render roundtrips and primitive-option refusals use synthetic sources |
| `pse-modeling` | Abstract/refined records, keyed documents, uncertainty, measured attribute selection, read-only knowledge views, temporal specialization | Arrow-free declarations and row sets; keyed inline/document equivalence, cross-reference, provenance, measurement and temporal kernel units |
| `pse-compiler` | Tracked binary inputs, admission reuse and conservative owned allocation accounting | Immutable complete source and physical inputs; incremental/clean equivalence, default-expansion refusal and cancellation/retry units |
| `pse-backend-native` | Library central differences with explicit bounds and exact declared steps; primitive fixture options use the existing native option contract | Value-only derivative probes and synthetic bound/cancellation units; unchanged valve assertions and heater certification fixture |
| `pse-catalog` | Compact masked null-container payloads before restoring required durable list elements | Existing Arrow cast/take kernels; sliced nullable fixed-size struct-list roundtrip regression retains the declared schema |
| `pse-ids` | Source revision preimage includes the declared semantic inputs | Framed semantic identities; golden vectors distinguish binary/value/time changes |
| `pse-quantity` | Remove the obsolete constant-registry identities | Physical type/unit algebra remains generic; scientific constants move to package records |
| `pse-model`, `pse-relations` | Generated declarations, knowledge projections and fitted parameter cells; delete dataset/observation and constant relations | Registry generators remain the schema authority; force-validation and codegen checks qualify the projections |
| `pse-schema` | Generic entity, measurement, temporal and result contracts; delete replaced scientific registry facts | One declaration per schema; synthetic generated consumer tests and roundtrips |
| `pse-runtime` | Decode Arrow types once, retain binary documents, project admitted knowledge, compose time, select typed fit observations and explicitly export qualified fits | Pure package admission owns meaning; runtime owns effects, leases and native execution. Data-document, time, fitting and bounded projection tests |
| `pse-py` | One exact text/bytes conversion, typed knowledge and fit export, existing resource report projection | Rust owns contracts; Python native units exercise binary inputs, read-only knowledge and fitted export |
| `pse-rules` | Remove tests of deleted measurement/constant relations | Generated relation declarations remain invariant authority |
| `pse-kernels` | Delete scientific JSON parameter data | Published and oracle parameters are package documents; no new scientific execution code |

Other changes in `pse-math`, `pse-engine` and `pse-operations` are formatting,
documentation-link repairs and test migrations to generated contracts. They add no
scientific authority or production mechanism.

`xtask` uses FeOS only to generate independent oracle observations. It reads published
parameters; it is outside the production property evaluator. Test and benchmark changes
supply declarations and observe existing operations. They do not become model authorities.

## 3. Contracts, authority and constraints

Species identity, formula, constants, correlation forms, critical points, pair selection,
reference states, caloric dispatch, NRTL and SRK are authored package knowledge.
The kernel owns general typing, refinement, identity, completeness, provenance and taint.
Parquet decoding supplies typed columns; declaration storage units and identifier schemes
govern interpretation. Inspection retains its immutable source revision and cannot clear
test-only taint. Fitting selects admitted measured attributes, checks complete physical
type equality and canonical standard deviations, and refuses unsupported uncertainties.
Export requires the existing joined feasibility, stationarity, rank and check evidence;
the receiving package explicitly admits the result with fit/run/source lineage.

| Physical element | Unit/basis/convention | Validity and authority |
|---|---|---|
| Caloric coefficients and increments | Dimensioned coefficients; canonical absolute/difference types and datum | Authored forms, parameter sets, selected reference conditions and envelopes |
| Cubic and PC-SAFT parameters | Critical pressure/temperature, dimensionless acentric factor; PC-SAFT length and energy conventions | Published/source-attributed bank rows; package-selected missing-pair policy |
| NRTL excess Gibbs energy | Extensive energy and mole amounts; dimensionless activity coefficients | Authored homogeneous potential and independent closed-form/oracle checks |
| Fit observations | Complete declared quantity; sigma in its canonical difference unit | Measured facet, uncertainty type and fit observation selection |
| Dynamic coordinate | Physical time; each spatial coordinate remains independently owned | Analysis horizon, shared clock, endpoint initialization and package definitions |

Variable roles and structural equations are admitted before solver construction.
Missing/competing initial conditions and ill-posed structures are refused with named
model members. A numerical conditional dependent on integrated time remains executable;
structural topology guards retain their admitted endpoint semantics.

## 4. Change scenarios and composition

| Scenario | Change boundary and observation | Evidence scope |
|---|---|---|
| DM1/DM2: new bank and a second bank for one species | Data documents and authored selection; no per-bank Rust loader or model match | Published 20-species bank; FeOS checks of seven states including two new species; NRTL/two-bank selected conformance |
| DM3/DM4: wrong coefficients or oracle reads | Common admission names the attribute, expected/actual quantity and source; Reader applies taint to consumed rows | Targeted quantity refusal, keyed conflict, identifier and shared-bank production/fixture units |
| CT-S05: steady/integrated/simultaneous | One child definition; analysis owns time expansion and initialization | Selected CSTR checks and pure/native clock/initial-condition units |
| DM6: publish fitted parameters | Generic explicit export and explicit receiving-bank admission | 34 native fitting units and export refusal; expanded vessel export/readmission acceptance passes |
| CT-S08/M1: value and structural edits | Tracked specialization, case views and evaluators; immutable overlays | Preparation tests and 1,000-point native Python acceptance pass; scaling timings remain unmeasured |
| PSE-S02: derivative-check library substitution | Native adapter owns the library and bounded comparison; model derivative programs unchanged | Six native derivative tests and original valve conformance pass |

## 5. Mechanisms and execution

| Stage | Ownership and numerical contract | Effects, limits and evidence |
|---|---|---|
| Bank admission | Exact units/types and identities from declarations; no solver | Decode/row/default/index allocations precharged; finite rows/cells/bytes, cancellation; 100,000-row and retry units pass |
| Specialization and lowering | Authored laws/guards and Symbolica mathematics; structural/value separation | Salsa owns reuse; retained schemas include envelopes, keys and derived/unique attributes. Library heaps outside callbacks are explicitly excluded from retention observations |
| Derivative diagnostics | Analytic model programs compared with `finitediff` central differences inside bounds and inward one-sided differences at active bounds | Explicit normalized step/tolerance and cell allowance, no analytic derivatives in value callback; incomplete/cancelled/rejected samples cannot pass |
| Native solves | Class-specific accepted adapters, existing scaling and validation | Effective native options retained; original obligations checked independently of solver status |
| Integrated checks | Shared physical clock supplied at each sample; original physical tolerances | Corrected controlled trajectory passes its unchanged oracle |
| Fit publication/inspection | Joined qualified fit evidence; canonical typed immutable projection | Bounded memory leases; no implicit bank publication or writable inspection |

The SCIP heater probe independently passes global certification with presolve disabled;
default presolve contradicts a feasible validated local candidate. The fixture states the
native option; orchestration contains no hidden PC-SAFT/SCIP special case. The original
BT_PR liquid tangent-plane limitation R-52 remains separately scoped.

## 6. Architectural assessment and gates

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | satisfied | Knowledge, pure admission, tracked preparation, effects and projection have distinct owners |
| AP-02 | satisfied | Native options enter the existing adapter validation; byte documents and typed fits share one consumed contract |
| AP-03 | satisfied | New banks/methods and time analyses compose declared definitions rather than add parallel scientific paths |
| AP-04 | satisfied | Consequential scientific distinctions are explicit, and applicability, selection, evaluation and outcomes consume their owned definitions |
| AP-05 | satisfied | Revision, keyed identity, source, role, fit lineage, temporal policy and unsupported outcomes remain explicit |
| AP-06 | satisfied | Synthetic kernel admission tests require neither Arrow nor solvers; format and native tests remain at their boundaries |

G1–G9 and PS-G1–PS-G3 are satisfied within the inspected architecture and named local
production test scope. The full seed and expanded native acceptance exercise physical
contracts, refusals, solver obligations and fitting lineage. The 1,000-point public study
exercises preparation reuse with one structural view. These results establish no
preparation-scaling or paired smooth/nested performance result; those measurements are
incomplete at closure.

## 7. Findings

No new architectural violation was found in the inspected boundary. Two improvements
from the implementation strengthen alignment: numerical checks use library differences
while retaining authored policy; native workarounds are explicit execution inputs, with
adapter validation and result evidence. Neither changes scientific assertions.

The 1,000-point native Python acceptance retains one structural view with 999
rebuilt/shared preparations and distinct endpoint results. No structural-rebuild-per-point
finding arises from that executed scenario. Preparation-scaling and paired nested
performance remain unmeasured; they establish no further conclusion.

## 8. Library fit and ownership cost

Arrow/Parquet own binary decoding, DataFusion owns read-only relational inspection,
Salsa owns immutable query reuse, petgraph owns taint closure traversal, Symbolica owns
authored mathematics/derivatives, and accepted native adapters own solving/integration.
`finitediff` 0.2.0 supplies fallible value-only differences; the adapter's one-dimensional
chart preserves the exact declared step and bounds. Library documentation/source and
existing capability skills support these consumed contracts. Upgrades remain localized
to their integration owners and tests. No library or license was rejected.

## 9. Alternatives and tradeoffs

Per-bank loaders, a second writable catalog and separate dynamic scientific definitions
would re-express knowledge in Rust. They are rejected for these scenarios. The simplest
viable implementation is the existing generic declared-record/row admission with format
adapters and generated projections. A finite-difference chart adds bounded orchestration
but preserves the library's difference implementation and avoids hand-coded numerical
rules. Native options keep backend specifics in declared execution inputs and adapters;
an additional solver-policy engine would add no useful responsibility.

## 10. Verification

**Tested, zero failures:** `just seed-conformance`, local Linux, 128 GiB pool,
one native thread, 600 seconds per solve and 120G OS cap: 115/115 seed fixtures,
4,046 checks; 6/6 domain fixtures, 53 checks. `just test --profile ci` passes
1,991/1,991 default tests and `just native-test --profile ci` passes 2,306/2,306
native tests, with explicit force-validation and at most 16 test processes.
`just native-test acceptance:: --profile ci --test-threads=2` passes 17/17 expanded
acceptance tests, including vessel export/readmission and authored fixture policy.
`just publication-test --profile ci` passes 9/9 tests. These are local test modes.

**Tested, zero failures:** `just native-python build/plan23-q-bounded` passes 173/173
linked-native assertions, including the public 1,000-point study. The checkout cleanup
plugin is disabled for this run because concurrent changes were attributed to tests;
all selections and assertions remain active. The 100,000-row admission, budget/refusal
retry and sliced required-list storage round-trip correctness tests also pass.

**Measured:** `just bench-production` completes the native cache and consolidation
workloads. `just thermodynamic-campaign flash build/plan23-flash-measure-complete`
completes the smooth 200-feed sample with 199 accepted outcomes in 511.457 seconds,
using default independent starts, a 48 GiB pool, one solver thread, 600 seconds per solve
and 120G OS cap. All 200 outcomes are retained, including the infeasible feed 17.
The nested report and preparation-scaling/admission timing campaign are incomplete;
no paired or scaling performance claim is made. Plan 23's Outcome owns detailed
conditions and supported limits.

## 11. Authority changes and disposition

Plan 23 is complete and its Outcome owns the production evidence and finding
dispositions. ADR-0130–0134 remain proposed under their separate decision-PR route;
this scoped review records implementation assessment without changing ADR status. The enduring bank, inspection,
time, fitting and fixture-option contracts belong in the existing architecture sections
with a blueprint revision. No policy/tracing requirements are changed by this audit.

## 12. Decision

Architecture: **satisfied within inspected scope**. Behavioral qualification: **Tested**
for the named local production suites, whole seed and expanded acceptance.
Overall: **Accept-scoped**, 2026-09-30. The scope includes typed bank admission,
inspection, analysis-owned time, measured-attribute fitting and explicit native policy;
it retains the R-52 SCIP contradiction and makes no claim for uncompleted performance
measurements. Plan 23 is done at the maintainer's direction.
