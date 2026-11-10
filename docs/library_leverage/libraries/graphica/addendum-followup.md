# Graphica addendum: correspondence and bounded topology exploration

Inspected 2026-11-09 at `924b0c3b28e749b9cd94ad06321b9547c6ff992c`, with parallel
Plan72 work present. The [capability addendum](../../../libraries/scientific_calcs/graphica.md#graphica-rust--capability-addendum)
was read completely from its supplied line1290 through the end. **Interface-checked:** selected
locked Graphica 3.0.1 source and current flowsheet/incidence consumers. **Proposed:** opportunity
priorities below. No execution test, benchmark or new reference census. This refresh adds depth
to the [earlier discovery](briefing.md) without selecting adoption.

## Structural correspondence makes reuse a more concrete inquiry

The useful new detail is how to obtain an actual correspondence between two equivalent graphs:
if `c_A` and `c_B` map their original vertices into equal canonical graphs, then
`inverse(c_B) ∘ c_A` maps vertices of A into B. This can avoid a second graph-matching operation
when transporting bindings between repeated equipment/network templates. The mapping need not
be unique in a symmetric graph; a valid mapping is not a distinguished physical identity.

The follow-up must confront a concrete gap: `CanonicalForm` returns no corresponding original
edge permutation. [Flowsheet connections](../../../../crates/pse-structural/src/flowsheet.rs)
retain independent occurrence IDs, physical ports, bindings and decision groups. Parallel edges
with equal labels can be structurally interchangeable yet carry different external records.
Adding occurrence IDs to the equality labels recovers distinction at the cost of the cross-model
equivalence the template inquiry was trying to exploit.

A worthwhile deeper review should compare an explicit incidence/port encoding with external
edge reconciliation on one repeated equipment template. Determine whether vertex maps plus that
encoding can transport parameter axes, port roles and result/derivative coordinates without
reconstructing the entire mathematical preparation. Mathematical template reuse can parameterize
varying values; deduplicating scientific trials instead requires those values and all other
relevant premises to agree. Current semantic hashes are deliberate identity products, not failed
attempts at unlabeled canonicalization.

`HiddenData` sharpens this boundary. Equal canonical structures may retain different hidden
metadata at symmetric positions. Structural equality therefore does not give canonical bytes
for every attached provenance field. Keep the reusable structural key separate from source
record transport and define reconciliation explicitly. Hidden metadata also remains displayed.

## Closed and inventory-constrained topology generation

Empty external-edge input is supported: the generator seeds a default internal vertex. This
extends the research beyond terminal-attached structures to a possible closed circulation,
recycle or network-design workflow. Combine typed signatures, exact structural constraints,
canonical deduplication, physical/model admission and shared numerical evaluation. This could
enable a useful new engineering capability even though there is no existing local enumerator
to replace.

Signatures are repeatable permitted attachment patterns, not a required equipment inventory,
and internal payloads use `N::default()`. A review should establish whether exact equipment counts
and named port roles can be represented with safe partial pruning and completed-candidate checks,
without building a second enumerator in the application. A missing motif in a partial graph is
not generally a safe reason to reject every possible completion. Default zero-flow bridge rules
can exclude valid closed-network candidates and are not physical flow analysis.

Finite structural size is a decisive feasibility question. `max_loops` passes the supplier's
termination-setting check but degree-two cycles of arbitrary length all have one independent
cycle. Callback presence likewise proves no finite family. Use a justified inventory or vertex
bound for the selected topology family, then assess actual search/retention behavior. This is
a mathematical distinction, not a measured claim that a particular invocation hangs.

`progress_fn` observes each new canonical graph; it does not receive the result map or symmetry
size. Returning false retains the triggering graph in `Err(partial_results)`. `abort_check` has
different polling points. Neither callback creates a streaming ownership interface or eliminates
the retained result/search state. An interactive explorer may use incomplete populations explicitly;
a complete design search cannot silently reinterpret them as exhaustive.

## Corrected symmetry and analysis boundaries

The previous technical account incorrectly said the fixed-vertex symmetry factor doubles directed
self-loops. The locked implementation checks `!e.directed`. For m identical self-loops, directed
loops contribute `m!`, while undirected loops contribute `2^m * m!`. The earlier statement was
an inspection error, now corrected in [canonical-generation](canonical-generation.md).

Exposed orbit generators are vertex permutations. The total group size also includes edge
permutations and undirected loop reversal factors, for which equivalent transformation generators
are not exposed. Determine whether the engineering consumer needs only a count, an actual vertex
mapping or full scientific equivalence actions. Neither group size nor orbit membership alone is
a probability weight or permission to omit a differently bound scientific trial.

The added source details also narrow several generic-looking APIs:

- If any valence-one vertex exists, `has_zero_flow_bridges` asserts that vertex **index zero** has that status. Merely choosing a different leaf as DFS root does not meet this condition.
- Bridge counts omit edges adjacent to valence-one vertices; connectivity is weak, not directed reachability or SCC structure.
- `set_directed(false)` does not normalize stored endpoints. `canonize_edges` sorts records; full canonicalization rebuilds normalized undirected endpoints. Fixed-vertex symmetry counts after direction mutation need the appropriate normalization.
- Canonicalization supports disconnected/empty graphs; generation produces connected candidates. Result-map iteration order is not a stable enumeration order.

These are constraints on a topology capability, not reasons to replace existing petgraph,
rustworkx or POUNCE responsibilities. Edge convenience methods, Empty, Display and cloneable
settings are useful utilities but do not presently justify independent deep-review engagements.

## Follow-up priority

First investigate mapped structural template reuse, with edge/port correspondence as the key
question. Separately consider bounded open/closed topology synthesis as a functional extension,
with a chosen engineering network family and explicit equipment inventory. The addendum improves
both inquiries materially; it does not establish a universally substitutable canonical identity
or a complete flowsheet-design engine.

All addendum sections1–9, smaller distinctions and compositions were considered. Source:
[locked-version implementation and tests](https://docs.rs/crate/graphica/3.0.1/source/src/lib.rs),
especially `canonize`, `generate_impl`, `get_edge_automorphism_group_size`, `set_directed`,
`canonize_edges` and spanning-tree methods; local interpretation additionally consumed
[incidence](../../../../crates/pse-structural/src/incidence.rs).
