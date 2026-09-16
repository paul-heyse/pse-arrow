# Design review: Wave 2 semantic compilation contracts

## 1. Decision and scope

**Proposal:** ADR-0060–ADR-0064 and blueprint revision 32, principally §6.15,
with plan 04 W2-01 as the delivery boundary.

**Status:** Proposed design; Interface-checked integration routes. This is not an
implementation acceptance review and establishes no executed Wave 2 behavior.

**Reviewer / author:** Codex independent document reviewer for the maintainer's
authorized Wave 2 implementation.

**Observable outcome:** finite, explainable P3–P10 compilation from ordinary
package declarations, strict generated manifest fields, and immutable Python
inspection without treating hashes as validation evidence.

**Baseline:** the existing rule compiler admits typed one-shot results, with
bounded UNION ALL recursion. It does not yet provide the proposed stratum executor.
P3 retains typed source references in normalized mathematical families. The existing
compiled mathematical reader and writer do not admit all those unresolved
alternatives. P10 has an actual physical inference implementation, but its existing
registered production route uses the separately declared predecessor fixture.

**Supported scope and non-goals:** review the contracts needed by the finite,
steady, single-phase Wave 2 library and the two declared boundary changes. P11–P16,
scalar problem closure, solver execution, derivative capability, optimal tear
selection, finer memoization and unconsumed Appendix B contracts are outside this
review. Their absence is not a defect in this bounded proposal.

**Method and coverage:** read the complete charter, directive and review template;
read ADR-0060–0064 and blueprint §6.15; traced their relevant existing consumers in
`pse-schema` rule/column/invariant declarations, `pse-rules` head admission,
execution, recursion and derivations, `pse-mathir` graph/reference/payload/load/emit
interfaces, and the compiler's P3/P10 mathematical adapters. Incorporated the catalog
agent's bounded manifest/Python interface inspection, without converting that
source inspection into runtime evidence. Attacked same-key competing values,
conflict-dependent assertions, scalar-to-species demand, kernel output selection,
guarded exclusions, unresolved mathematical payloads and retained Arrow ownership.
No Cargo, native build, Python suite, benchmark or terminal gate was run by this
reviewer. Concurrent implementation is not covered by this document-stage verdict.

**Review checkpoint:** the first revision-32 draft required revision on W2C-01–06.
The reviewer then read the corrected §6.15.2–§6.15.4 and ADR-0062/0063. All six
counterexamples now have a coherent declared route. The final decision is **Accept
the bounded document design**, at Proposed/Interface-checked evidence only. The
historical counterexamples remain in §7 as implementation regression obligations.

## 2. Authority and lifecycle map

| Concept | Type and identity | Authority / owner | Revision boundary | Update path | Derived representations |
|---|---|---|---|---|---|
| Physical records and stock templates | Declared semantic IDs, complete quantities and domains | Shipped package declarations | Admitted package revision | New validated package | Generated leaf fixtures; normalized facts |
| Manifest fields | Recursive ManifestSpec with checked native bindings | Registry | Explicit wire/schema versions | Generator, retaining existing codecs | Rust/Python wire types and documentation |
| Configured prospective instance | Actual root ID or framed child/declaration/index identity | Authored facts plus P3 derivation | Complete P3 inputs | Explicit binding change | Normalized values and finite candidate scopes |
| Rule assertion and decided fact | Versioned rule, complete head key and actual typed value | Declared RulePlan; stratum result | Complete bound stratum inputs | Recompute through the declared executor | Heads, outcomes and distinct support edges |
| Method requirement and selection | Scope/property/complete index; exact method version | P6 under declared selection policy | Complete candidate inventory, including absence | Recompute after any candidate or dependency change | Candidate, resolution and requester relations |
| Realized mathematics and participation | Declaration/instance/domain identities; artifact-local node IDs | P7–P9 derived bundles | Exact predecessor ports | Ordered substitution and law/method realization | Indexed equations and P10 canonical graph |
| Inspection handle | Exact admitted manifest/ref and output port | Catalog | Immutable pinned snapshot | Open a new handle for another observation | One-consumption Arrow stream; retained arrays |

Specialized DAG rebuilding, quantity inference, graph heuristics, resource accounting
and executor scheduling remain ordinary native code behind declared contracts.
DataFusion supplies relational operations, not domain policy. Paths and qualified
names remain labels; node ordinals remain local to their mathematical artifact.
Sharing a node cannot erase occurrence provenance or define instance identity.

## 3. Semantic contracts and invariants

| Contract | Representation | Enforcement boundary | Failure behavior | Evidence |
|---|---|---|---|---|
| Configuration retains declared meaning | ConfigValue and exact source keys | P3 admission | Unknown key, duplicate assignment or incompatible value rejected | Proposed, §6.15.1 |
| Child scope universe is finite before feature resolution | Prospective instance/domain bindings | P3 expansion, then P4/P5 correspondence | Recursive/unresolved choice fails; no successful truncation | Proposed, §6.15.1 |
| Four-valued facts retain competing actual values and causes | Head-derived typed assertion relations and located support | Rule admission and complete stratum resolution | Reject aborts the stratum; Undecided heads have no same-stratum consumer | Proposed, corrected §6.15.2; W2C-01/02 |
| Demand supports finite index transformations | Source-axis, fixed-member and bound-domain mappings; all requesters | P6 before expansion | Invalid mapping, unknown scope or unbounded demand fails | Proposed, corrected §6.15.3; W2C-03 |
| Selection absence and ambiguity carry no fabricated winner | Nullable winners, typed output alternatives and complete candidate inventory | P6, then P9 correspondence | Unsupported/ambiguous requirement prevents graph publication | Proposed, corrected §6.15.3; W2C-05 |
| Every applicable law term participates once | Complete realized candidate partition; earlier false-guard witnesses | P7/P8, then P10 physical checks | Missing, duplicate or incompatible participation rejected | Proposed, corrected §6.15.4; W2C-06 |
| Pending representation requests survive until their owner resolves them | Explicit inferred mathematical family, actual references and resolved gathers | P7–P9 emission, P10 input/output admission | Unsupported or unresolved payload refused | Proposed, corrected §6.15.4; W2C-04 |
| Boundary decoding does not certify semantics | Generated strict wire fields plus catalog admission | Open/import and stage replay | Wrong versions, fields, parents, ports or actual contents rejected | Proposed; existing interfaces inspected |

Absence has explicit meanings: no assignment, a default's actual source, unknown or
conflicting rule truth, scalar empty index, inapplicable tagged fields, unavailable
winner, cancelled attempt and empty successful output must remain distinguishable.
ConfigValue contains admitted values; unknown facts require their explicit outcome
records rather than a fabricated Boolean payload.

Equivalence is exact admitted key/value and ordered-IR equality under the declared
physical contract. Float payload comparison must preserve required signed-zero
distinctions; `Cell`'s derived `PartialEq` alone is not that comparison. Hashes locate
candidates or encoded objects. They establish neither convergence nor absence,
physical compatibility, valid replay, or scientific correctness. Numerical method
checks require independently justified reference values and tolerances.

## 4. Derivation and execution design

| Stage | Declared inputs | Output | Preconditions | Effects and ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| Package admission and P3 | Exact documents, types, domains, defaults and bindings | Normalized values, syntax, finite scopes | Strict source admission and finite expansion | Owned temporary work; no inspection-driven changes | Exact source row/key/span and complete input environment |
| P4/P5 | P3 scopes plus versioned rules and material/connection facts | Decided features, membership, ports and topology | Legal strata; finite scope; actual member compatibility | Shared runtime and resource budget | Positive support and complete negative bindings |
| P6 | Seeds, guards, scopes, all candidates and policy | Finite requirements and explicit selections | Complete dependency mappings and supported provisions | Private fixed-point workspace | Every requester and rejected/tied candidate remains inspectable |
| P7/P8/P9 | Exact earlier ports and bound templates/laws/methods | Complete indexed math and participation bundles | No new late demand; resolved instance references | Ordered native rebuilding plus relational matching/grouping | Per-occurrence lineage; complete stage outputs, including empties |
| P10 | Actual instantiated rows, roots, symbols, domains and physical/kernel context | CanonicalMathGraph | Structural and physical admission | Existing inference under strict ordering | Actual operation/conversion evidence and remapped roots |
| Open and export | Exact ref/manifest/receipt and requested output port | Immutable handle and stream | Catalog admission and producer correspondence | Validation-only replay; owned arrays; no publication | Handle pins original observation; later ref movement is separate |

Containment, physical connectivity, rule dependency, stage scheduling and support
are distinct relationships. Physical recycles are valid inputs to tear selection;
containment cycles are invalid. The tear heuristic must demonstrate that removing
its actual chosen edges makes the actual graph acyclic, without an optimality claim.

Publication inherits blueprint §14.3 and §22.2: complete output and terminal
evidence precede success; failure/cancellation cannot look like an empty successful
stage. Whole-stage reuse checks complete positive and negative inputs and re-admits
actual outputs. No row-level reuse guarantee is introduced.

The minimal reusable P7 mathematical interfaces are Interface-checked as follows:

| Existing entrypoint | Reusable responsibility | Boundary or hazard |
|---|---|---|
| `pse_mathir::ExprGraph::insert`, `graph.rs:81` | Ordered structural admission and exact framed-value sharing | Does not bind template references or establish physical types. Scope is excluded from sharing and merges to null; preserve occurrence lineage separately. |
| `topo::postorder_with_bindings`, `topo.rs:20`; `walk::walk` | Traverse ordinary, payload-held and kernel-input dependencies | Storage order never permits eager evaluation of a guarded branch. |
| `Payload::map_node_references`, `payload.rs:275`; `EquationRecord::map_node_references`, `equation.rs:79` | Rebase every held node ordinal while preserving order | Does not bind semantic IDs, DomainRef, ValueRef, predicate source IDs or lexical binder identities; P7 supplies that explicit environment. |
| `relations::load_untyped`, `relations/load.rs:35` | Admit actual rows, reject duplicates/orphans/gaps/cycles, return complete `node_mapping` | Supplied quantities, selections and hashes remain claims; this is not physical inference. |
| `relations::emit_untyped_with_bindings`; `MathRelationSource` / `MathRelationSink` | Arrow-free complete ordered graph transport | The existing compiler `RelationSource::from_batches` needs normalized/instantiated family support; changing a namespace is not admission. |
| `index::bind`, `index::resolve_binder`; `DomainRef::require_actual`, `ValueRef::require_symbol`, `GuardRef::require_math` | Check actual lexical/reference boundaries | Repeated binders and unresolved alternatives fail. Bind per instance occurrence before compiled admission. |
| `canonicalize(CanonicalizeInput, Policy::Strict)` | Existing P10 physical inference, guarded folding and canonical numbering | Supply actual symbol/domain/kernel/quantity facts, ordered roots and complete bindings; `insert_typed` and `set_quantity_type` cannot replace inference. |

There is no existing public generic instance-substitution API in the inspected
baseline. The small extension is a checked instance/binder environment and ordered
rebuild through these interfaces, with an explicit instantiated relation adapter.

## 5. Representative journeys

**Ordinary extension:** a second lumped unit uses the same typed template constructs,
bound domains and law declarations. P3 creates its finite context; P4/P5 resolve
guards and ports; P6 closes demand; P7–P9 realize declarations; P10 re-establishes
physical meaning. The extension adds package semantics and conformance cases, not a
unit-name branch. A genuinely new operation still needs its registered algorithm.

**Meaningful change:** add an equally ranked method candidate, change a species
domain, or alter a default. The complete candidate/domain/policy input changes even
if an old selected output happens to be equal. Clean recomputation must match actual
values, support and current lineage. A forced-equal lookup key is a negative control.

**Boundary:** an indexed template expression contains a unit-bearing smoothing
tolerance and a gathered species value. The normalized reader preserves both requests;
instance binding resolves lexical/domain references and the gather; a declared
pre-P10 family retains the still-pending physical tolerance. P10 verifies the complete
quantity and converts it once. The initial compiled-only adapter cannot substitute
for that missing boundary contract.

**Failure:** two actual values compete for one head key. The complete stratum must
refuse the conflict or produce explicit undecided evidence before any dependent
decided output is published. Cancelling later realization retains the previous
visible snapshot and records a failed/cancelled attempt. Closing a Python handle
does not invalidate already exported arrays or release their reservations early.

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Pass, Proposed | Package and ManifestSpec authority are explicit; one registry-versioned precedence policy; assertion schemas project the actual head | Generate projections and validate the exact declaration relationships |
| G2 — Semantic fidelity | Pass, Proposed | Actual competing payloads, tagged finite axis mappings and tagged provider outputs preserve required distinctions | Exercise W2C-01/03/05 counterexamples and exact float semantics |
| G3 — Validity | Pass, Proposed | Reject aborts whole strata; Undecided same-stratum consumers fail admission; inferred math retains only declared pending alternatives | Enforce preconditions and postconditions on every actual route |
| G4 — Hidden behavior | Pass, Proposed | Selection is declared; inspection and validation do not publish or create demand | Keep validation-only producer replay explicit |
| G5 — Consistency and recovery | Pass, Proposed | Existing complete-bundle/terminal-record protocol and retained Arrow owners are preserved | Exercise each new stage and Python lifetime boundary before runtime acceptance |
| G6 — Transformation and reuse | Pass, Proposed | Bounded conflict scheduling avoids history-dependent retraction; typed intermediate facts and complete positive/negative inputs govern reuse | Full value/support comparisons, actual producer replay and meaningful-change fixtures |
| G7 — Truthful capability claims | Pass, Proposed | The instantiated-family and provider-output routes are explicit; unsupported rule shapes and later numerical capabilities are refused | Implement and qualify each claimed route before changing its evidence label |

These verdicts assess the written design. Passing a Proposed gate would authorize
implementation of that contract; it would not certify the resulting runtime.

## 7. Principle findings

| Finding / judgment | Principles | Concrete evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| **W2C-01 — Distinct competing assertions; satisfied in corrected proposal** | DM-06, DM-08, DM-09, DM-46 | Initial §6.15.2 identified rule/head/key without the competing value. Corrected §6.15.2 projects typed assertion relations from non-provenance head columns and names each assertion in support. | K→V1 and K→V2 now have independently recoverable values and causes. | Implement exact typed candidate admission and value comparison; exclude provenance from fact equality. | One rule and two rules each emit conflicting nested/float values under one key; all values and distinct causes remain recoverable. |
| **W2C-02 — Bounded conflict semantics; satisfied in corrected proposal** | DM-07, DM-22, DM-24, DM-31 | Initial monotone blocked conflicts could retain Y's conflict after its only X-dependent assertion lost support. Corrected §6.15.2 and ADR-0062 abort Reject strata and forbid same-stratum consumers of Undecided heads, including self-recursion and mixed head policies. | No successful output depends on a history-sensitive retraction algorithm. | Validate this supported subset before execution; assign actual stock consumers to legal strata. | Independent set oracle; shuffled rules/partitions; illegal Undecided dependency rejected; Reject conflict publishes no stratum output. |
| **W2C-03 — Finite dependency axis expansion; satisfied in corrected proposal** | DM-09, DM-19, DM-22 | Initial integer-only mapping could not express scalar mixture enthalpy demanding species enthalpies. Corrected §6.15.3 defines source-axis, fixed-member and bound-domain alternatives. | The stock dependency has an explicit finite, ordered expansion route. | Validate each target axis, member and state-bound domain; reject ambiguity or unbounded expansion. | Scalar-to-species, phase-to-phase/species and invalid-axis/domain examples produce exact expected finite requirement/support sets. |
| **W2C-04 — Intermediate mathematical ownership; satisfied in corrected proposal** | DM-21, DM-22, DM-24, DM-42 | Existing compiled-only adapters cannot retain every normalized request. Corrected §6.15.4 and ADR-0063 declare complete `inferred.math_*` bindings with actual references/resolved gathers and pending conversion/smoothing payloads. | P7–P9 can serialize valid intermediate meaning before P10 establishes physical validity. | Extend the existing reader/sink family and reuse the existing graph; remap every root, equation and kernel input together. | Source-to-P10 nested/guarded expressions with noncanonical units; serialized intermediate reopen; all roots and payload references remap together. |
| **W2C-05 — Method policy/output addressing; satisfied in corrected proposal** | DM-02, DM-06, DM-19, DM-43 | Corrected §6.15.3 uses one registry-versioned precedence policy and a template-symbol/kernel-output discriminator. | Selection has no ambient package policy and kernel output correspondence is explicit. | Validate output ordinal/declaration against the exact realization and signature; reject unsupported alternate policy selection. | A multi-output kernel maps each advertised property to its exact output contract; wrong mapping fails; candidate-order changes do not select a winner. |
| **W2C-06 — False-guard exclusion correspondence; satisfied in corrected proposal** | DM-08, DM-09, DM-46 | Corrected §6.15.4 retains P4 predicate outcomes with exact declaration/instance support for false guards; those declarations do not become realized contributions. P8 partitions realized contributions and no longer includes `guard_false` as a participation reason. | Earlier exclusions and the later realized partition have distinct, explainable meanings. | Preserve both witness routes and validate their source correspondence through reopen. | False guard remains explainable after reload; changing it to true adds exactly the expected candidate and participation, with current support. |
| **Single physical/manifest authority; satisfied in proposal** | DM-02, DM-41, DM-51, DM-52 | ADR-0060 derives recursive wire fields with checked native bindings; ADR-0064 derives leaf fixtures from package declarations. | Removes independently maintained field/physical rosters while preserving explicit semantic admission. | Keep generator ownership and schema version decisions explicit. | Full decoded fixture equality plus independent scientific checks; strict nested wire parity and malformed-input tests. |
| **Non-mutating inspection and owner lifetime; satisfied in proposal** | DM-20, DM-28, DM-29, DM-35, DM-37 | ADR-0061 pins exact refs/ports, uses existing-directory opening and retains actual buffer leases; catalog interface inspection supports these routes. | Inspection can be immutable without pretending admission is lazy or free. | Preserve no-publication replay and exact error/cancellation states. | Ref movement, partial drain, cancel/close and retained-array drop-order tests under one finite runtime. |

**Applicability:** all authority, semantic typing, transformation, dependency,
boundary, provenance and evolution groups apply to this change. Execution/recovery
principles apply to the new stages and inspection lifecycle, through the existing
publication contract. Performance principles apply to accounting and honest claims;
no speedup or memory measurement is asserted. Solver/backend numerical guarantees
outside the declared graph/binding scope are not rated. No optional maturity score
is used to offset unresolved mandatory gates.

## 8. Alternatives and architectural leverage

| Alternative | Duplication / extension locality | Correctness and operational risk | Cost | Performance evidence | Decision |
|---|---|---|---|---|---|
| Retain Wave 1 fixture and per-unit construction | Repeats template and provider meaning in native code | Cannot deliver generic Wave 2 compilation | Small immediate change; high semantic coordination | None for Wave 2 | Rejected |
| Complete existing registry/rule/MathIR contracts | One declaration plus specialized algorithm and conformance tests | Requires exact support, mapping and boundary contracts | Bounded additions with existing generators/adapters | Proposed; end-to-end costs still open | Preferred after findings close |
| Simpler viable execution: uncached complete stages, Reject conflicts, terminal Undecided strata, typed candidate rows | Same domain declarations; less truth-maintenance/reuse machinery | Explicit unsupported plan shapes; no hidden retraction algorithm | Lower implementation and failure surface | Must measure before adding finer machinery | Suitable first complete implementation within scope |

Finite worktables are justified by recursive demand and topology. Typed candidates
are justified by conflicts, not speculative generic execution. A second MathIR AST,
row-level memo engine, dynamic rule platform, proof-path enumeration or per-unit
builder is not justified. Native DAG rebuilding and graph heuristics remain code;
their domain choices, identities, inputs and errors remain declared.

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Named check | Required conditions | Current result |
|---|---|---|---|---|
| Exact finite rule semantics | Proposed | `stratified_fixed_point`, `four_valued_rule_outcomes` | Independent oracle; conflicting actual values; duplicate supports; null/false; cycles and bound failure | Not run; corrected contracts reviewed |
| Complete demand and realization | Proposed | `demand_seed_closure`, `realization_completeness`, `template_instantiation` | Finite scalar/indexed dependencies; missing/tied providers; second generic unit; changed guards/domains | Not run |
| Physical and conservation meaning | Proposed | `law_participation_completeness`, `method_realization_units`, `authored_to_typed_graph` | Actual exhaustive partitions, ordered descriptors, natural-unit conversions, complete production P3–P10 inputs | Not run |
| Reuse and lifecycle correctness | Proposed | `incremental_vs_clean_wave2`, `wave2_publication_lifecycle` | Actual source/candidate/policy change; forced-equal lookup keys; failure/cancel at each new stage | Not run |
| Generated boundary parity | Proposed / Interface-checked route | `manifest_generated_contract`, `python_snapshot_streams` | Strict nested fields; exact ref/port pinning; unknown metadata; retained arrays; cancellation | No execution claim in this review |
| Scientific package authority | Proposed | `standard_fixture_matches_yaml`, `reference_package_admission` | Full decoded records; independent formulas/sources and declared tolerances | Generation equality alone cannot close scientific validity |
| Costs and resource limits | Proposed | `wave2_resource_envelope`, whole-workflow benchmarks | 32 GiB normal ceiling, separate tiny refusal cases; accounted and process peaks; cold/warm runs | No measurements claimed |

Runtime acceptance remains plan 04 W2-12: focused `just test-package <owner>
-p pse-relations` with explicit force-validate through the recipe, regeneration and
governance, `just ci-pr`, `just features-powerset`, `just test-release`, host Python
and pinned parity, plus real golden write/check and benchmark smoke. Each receipt
must name its command, mode, result count and zero failure/warning/skip baseline.
No stale Wave 1 receipt qualifies these new contracts. Local scratch logs alone are
not durable release evidence. Cost accounting includes candidate/support growth,
normalization, producer replay on open, transfer and retained-output memory.

## 10. Exceptions and unresolved decisions

No SHOULD deviation is requested. The proposed restriction on Undecided heads is a
declared supported rule subset, provided stock Wave 2 rules fit it and unsupported
plans fail at registry admission. It is not permission to silently weaken requested
semantics. W2C-01–06 are closed as document questions and remain implementation
regression obligations. Broader rule shapes may trigger a future decision when a
real consumer requires them. No runtime gate or scientific acceptance is waived.

Formal ADR acceptance and its decision-PR workflow remain pending. Proposed ADRs
and a design review do not by themselves mark plan 04 implemented or tested.

## 11. Decision and implementation changes

**Decision: Accept the bounded document design.** The corrected revision-32
contracts and ADR-0060–0064 provide coherent implementation routes for the reviewed
scope. Evidence remains Proposed/Interface-checked. No additional blocking design
finding was identified in the bounded correction review; implementation, generated
schema/pass closure, formal ADR acceptance and all new executable gates remain open.

| Priority | Change | Principles | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| P1 | Implement exact assertion/support and bounded conflict semantics | DM-07, DM-08, DM-22, DM-46 | Reviewed §6.15.2/ADR-0062 amendment; actual executor behavior still unverified | Independent conflict/support/stratum admission fixtures |
| P1 | Implement finite dependency mappings and provider output/policy addressing | DM-09, DM-19, DM-43 | Scalar-to-species and kernel-output journeys are representable; qualify actual rows | Exact requirement/candidate/provision tests |
| P1 | Implement intermediate mathematical family and exclusion correspondence | DM-22, DM-24, DM-42, DM-46 | Declared complete P7–P10 route, including pending physical requests | Intermediate reopen and guarded source-to-P10 fixtures |
| P2 | Implement the accepted contracts through existing generators/adapters | DM-02, DM-52, DM-56 | Current source review plus the named executable gates | Regeneration and semantic boundary tests |
| P2 | Qualify actual lifecycle, cost and scientific claims before closure | DM-39, DM-53, DM-54, DM-59 | Durable command/mode/count receipts against baseline zero | Terminal gate ledger; independent scientific evidence |

The current verdict concerns the reviewed document snapshot. Any later correction
needs a bounded rereview; any runtime claim needs its own executable evidence.
