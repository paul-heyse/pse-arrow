---
title: "Plan73g: Finite topology exploration"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73g-finite-topology-exploration.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Finite topology exploration

## Outcome and baseline

**Proposed.** This companion of [Plan73](73-symbolica-ecosystem-improvements.md) delivers O09: a researcher generates a finite circulation family, inspects structurally distinct candidates and requests ordinary physical admission and scientific evaluation. Enumeration completeness, physical admission and scientific feasibility are distinct results. Own conversion, operation/result delivery and independent reading here, without waiting for unfinished Plan66 packages.

**Interface-checked.** Baseline `baadc7e32` includes typed flowsheet declarations, physical ports, parallel connection occurrences and grouped tears in `pse-structural/src/flowsheet.rs`. Existing authored units and recycle examples provide science, not topology enumeration. Retain suitable native graph and solver owners; no universal synthesizer, graph service or replacement tear algorithm is selected.

## Finite first family and scientific consumer

The open family contains exactly one mixer M, splitter S, heater H and cooler C, one fresh boundary and one purge boundary. Fresh enters M; purge leaves S. Other connections form one directed circulation visiting each internal instance once. Named fresh/recycle and purge/continuation roles are fixed. With M as anchor, permutations of S/H/C give six role-distinct candidates. H/C are distinct scientific roles; occurrence spelling is renameable. No extra equipment, self-loop, disconnected component or degree-two subdivision is admitted.

The closed control contains exactly H/C with two opposite directed material connections and no external legs. Ordinary science fixes circulation flow and a pressure datum with declared heat/cool targets; do not add an artificial feed for a generator convention. Use ordinary authored mixer/splitter and heater implementations (negative duty/declared cooling realization for C), constant-cp material, explicit pressure treatment, feed state and positive purge fraction. Selected targets must admit an independently solvable control; incompatible targets remain a scientific rejection rather than changing the structural population.

Each candidate passes ordinary physical typing, degrees-of-freedom/structural checks, initialization and original-coordinate conservation/duty assessment. Include a physical port-contract mismatch and a scientifically incompatible candidate. A generated graph never certifies feasibility.

## Native generation and scientific interpretation

Use Graphica3.0.1 `Graph::generate`, typed directed half-edge signatures, native filters and canonicalization. The [generation contract](../library_leverage/libraries/graphica/canonical-generation.md) and [mathematical correspondence account](../library_leverage/libraries/graphica/mathematical-template-reuse.md) establish the selected source conditions.

Signatures are reusable kinds, not exact counts; generated internal payloads are `Default`, not typed equipment instances, and signature order is not named-port order. Generate neutral signatures with fresh/purge edge kinds and explicit total vertex/equipment bounds. Decorate the two degree-two positions with H/C using a finite permutation operation, then canonicalize the role-labeled interpreted candidates. Validate exact inventory and single circulation on completed candidates. This avoids claiming the generator assigned named internal science.

Prune only monotonic impossibilities in partial `filter_fn` checks. Missing final motifs and exact final counts cannot be rejected before their completion is impossible. `max_loops` alone does not exclude arbitrarily many degree-two subdivisions; explicit vertex bounds and the inventory predicate establish finiteness. G02 must show the native generation/filter configuration actually enforces those bounds. If the selected supplier API cannot bound recursive exploration before materialization, own a checked supplier extension under B's control contract before widening the family; do not replace native enumeration with a second bespoke search.

Role-colored incidence vertices carry units, ports and connection occurrences where mapping needs them: Graphica supplies a vertex map but no canonical edge map. Preserve parallel edges and named roles; physical identity is not a dummy index. A owns mapped mathematical reuse, E native indexed mathematics. Neither graph equality nor symmetry grants provider/mathematical equivalence.

Configure closed-network bridge/root options deliberately. Their topology conventions are not conservation or physical flow. Generation emits connected candidates; canonicalization also accepts empty/disconnected inputs, which are contract controls but outside this family. Directed/undirected self-loop factors and exposed vertex generators do not imply complete edge actions or statistical weights. `HiddenData` excludes metadata from comparison, not display or canonical provenance ordering.

## Lifecycle, operations and publication

The request declares family/version, equipment templates, physical role interpretation, explicit bounds/predicates, selected science and demand. Candidate identity includes application interpretation and sorted canonical structure; do not use map iteration or a display string as identity. Preserve authored declarations/source and generation premises for each member.

Generation retains a result map; progress callbacks do not transfer ownership. Abort polling occurs at connected completed candidates, without guaranteed bounded latency through recursion/canonicalization. Report requested stop, actual settlement and incomplete enumeration honestly. The tiny selected family makes map retention appropriate. Do not promise streaming ownership or instantaneous cancellation.

Keep completed enumeration with scientific rejects distinct from interrupted enumeration and successful solves. Retain unsupported, physically refused and numerically failed members. Publish typed candidate sets through current operation/result custody and independent native readers/exports with role, source, units and partiality. Rendering consumes the original product without rerunning science. DOT/Mermaid helpers may project an actual result after escaping/metadata checks; do not migrate suitable existing graphs solely for formatting.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| G01 | open | Shared source/product contract; typed flowsheet admission | Selected families, exact inventory/roles/predicates, independently listed six open/one closed populations and scientific/identity decision routes established. |
| G02 | open | G01; source-qualified Graphica bounds/filter/canonicalization | Native generation and finite H/C decoration produce exact populations with stable scientific role correspondence and honest completion. |
| G03 | open | G02 and ordinary authoring/physical/structural/scientific owners | All selected candidates convert to ordinary model operations; supported science executes and independent assessment retains physical refusal, incompatibility and solve outcomes. |
| G04 | open | G02/G03 products and protected lifecycle | Candidate-set request/result, cancellation/settlement, publication and independent read/export complete; replaced provisional conversion/result paths removed. |
| G05 | open | G01–G04 delivery | Complete/incomplete open/closed journeys ready for H; detailed topology leads reconciled and admitted envelope published. |

G01 defines finite examples as actual implementation scope, not merely toy generator tests. G02 tests rename invariance, H/C distinctions, fresh/recycle ordering, parallels and exact population against independently authored lists. G03 supplies original-coordinate material/energy evidence and explicit degrees-of-freedom handling. G04 migrates consumers and generated boundaries with their declaration changes; no in-memory-only substitute qualifies production reading.

G05 closes inventory/filter/correspondence/automorphism/helper/combinatorial leads at their consuming package. Whole-flowsheet caches are not selected; A's independent-motif comparison owns that question. General superstructure optimization, disconnected generation and arbitrary inventory growth are different supported outcomes, reopened only for a named family with explicit scientific/search/lifecycle contracts. Current SCIP, petgraph/rustworkx/POUNCE responsibilities remain suitable where their consumed operations differ.

## Verification

**Proposed:** exact independently enumerated populations; open/closed bridge controls; role-sensitive rename/port/parallel cases; incomplete cancellation with settled map; physical-invalid and scientific-incompatible members; independent original-coordinate duty/conservation; protected reader/export. H selects relevant scope-end integration/static qualification. No tests, probes or performance measurements ran in authoring. Retain the zero target and name actual commands/selections when execution supplies evidence.
