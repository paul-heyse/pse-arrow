---
title: Efficiency principles remediation
status: done
date: 2026-10-09
adrs: [ADR-0169, ADR-0170, ADR-0171, ADR-0172]
review_sources: [../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md]
scenario_sources: [../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#4-representative-journeys-and-change-locality]
---

# Plan 33: Efficiency principles remediation

## Purpose, ownership and evidence boundary

This standalone plan adopts the eleven findings and six investigation avenues from the
[codebase efficiency-principles review](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md).
It owns their decisions, implementation packets, finding dispositions and affected
qualification. This work is independent of Plan 28. Shared source files and consumed
contracts do not transfer the new scope to that plan or make its E3/E4/E5 campaign a
completion prerequisite here.

The maintainer authorized creation of this plan on 2026-10-09, confirmed the review's
RC01–RC05 directions, and then replaced its internal-history preservation assumptions
with the design-phase directions below. The maintainer authorized execution on 2026-10-09. Implementation and affected acceptance
are complete; the Outcome records their scope and limits. The cancelled campaigns remain paused.
The source baseline is `main` at `4c24721e691187e1a5b28398b29722fbde671da8`, including the
uncommitted principal review. Preserve concurrent Plan 32 and other dirty work; recheck
the actual checkout before execution.

Core 3.4, Efficiency Heuristics 1.0 and Process Simulator 1.5 apply through
[the selected standard](../design_review/design_principles/standard.toml) and
[repository binding](../design_review/design_principles/binding/pse-arrow.md).
Architectural fitness, semantic correctness, scientific qualification and measured benefit
remain separate judgments. Static source evidence establishes repeated work; it does not
establish a latency or memory improvement. The review's projection probe establishes omitted
input sensitivity, not a real stale successful-receipt reuse or an escaping scientific error.

### Design-phase direction

Build the target directly. Delete replaced code, callers, tests and fixtures when their
replacement's targeted controls pass and callers have moved. Do not introduce compatibility
APIs, dual production implementations, legacy readers or migrations solely to preserve
immature internal history.

Generated internal runs, result/analysis state, compiler artifacts and test state may be
discarded and regenerated when contracts change. Preserve authored models, physical
definitions and external reference fixtures. Coordinate any actual reset with the owning
store, active users and reference holders; permission to rebuild derived state is not
permission to destroy concurrent work or unrelated shared storage. Do not add routine
whole-store resets to ordinary execution.

Retired analyses must ultimately disappear completely, including headers, graph payloads,
input lineage, retirement markers and cleanup records. Active protection, authentic drain
and interruption safety still apply. Temporary coordination exists only until safe deletion
finishes; retaining a permanent tombstone or audit payload is not the target.

Historical internal results have no authority after their assumptions or contracts become
invalid. Tests must use current contracts, applicable analytical expectations, conservation,
independent oracles and the pinned external IDAES/Pyomo references. Fresh current-run
roundtrip/restart fixtures remain useful. Earlier internal outputs may explain an investigation
but must not become golden scientific results, migration obligations or performance gates.
Neither mechanically preserving old values nor regenerating expected values from the same
questionable transformation establishes correctness.

The simulation ultimately follows physical first principles. Independent thermodynamic
evaluation methods are a useful future validation direction, but constructing that framework
is not a priority or a completion requirement of this plan. Its absence does not make old
internal results authoritative. External references retain their stated assumptions and
limitations; clean-room behavior comparison against pinned IDAES/Pyomo is distinct from
internal historical regression matching ([relationship to IDAES](../relationship-to-idaes.md)).

### Adoption criterion

Adopt a correct, applicable approach expected to improve execution fit unless it does not
work or significantly impairs performance. Small, inconclusive, noisy or slightly adverse
timings do not independently justify exclusion. Concurrent repository workloads can distort
timings. Apply this criterion to the complete affected operation, while retaining physical
checks, exact dependencies, ownership and recovery obligations. There is no minimum measured
speedup gate. Quantitative claims still require comparable measurements; a key hash and a
complete preparation are different operations.

## Confirmed rule changes and adoption routes

All decisions below were confirmed by the maintainer on 2026-10-09. Review RC identifiers
refer to the efficiency review, not the distinct Graph/hash RC identifiers in Plan 28.
Confirmation selects the target; required architecture/ADR adoption precedes dependent
implementation. Accepted ADRs are not rewritten in place.

| Rule impact | Confirmed direction | Route and dependency |
|---|---|---|
| [RC01](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc01) | First seek narrow feature activation without selecting unrelated relation targets. If that cannot satisfy the universal mandate, require force-validation for every actual Arrow-consuming test/check closure, including dev-dependencies. | EFF00/EFF02 settle the mechanism. If the mandate changes, adopt the governance amendment through ADR plus target review and update blueprint §24.1, agent instructions, binding, aliases/recipes and applicable validation guidance together. |
| [RC02](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc02) | Selective access has relation-local sticky encoding errors. Name discovery does not encode payloads. Explicit complete-table access and durable publication still require every required relation. | EFF00 records the changed result/Python contract through the required ADR/review route and blueprint §19.2/§21; EFF07 migrates all affected consumers. |
| [RC03](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc03), as amended | Fully delete retired analyses after safe drain, including lineage and permanent retirement markers. The review's retained-header/lineage remedy is superseded by this user direction. | EFF00 settles identity/admission/retirement semantics and the relevant metadata/publication decision route; amend blueprint §20.4 and affected analysis contracts before EFF10. |
| [RC04](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc04) | Remove duplicate rendering/hashing and target compact body-relative occurrence ownership. If identity/wire meaning changes, select a new interpretation and rebuild disposable internal state; retain no legacy decoder or migration for that state. | EFF00/EFF03 use the hashing/metadata ADR and review route if those contracts change, updating blueprint §5/§14 at their owners. Changes within existing meaning need no new decision. |
| [RC05](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#rc05), as amended | Expand conservative consumed-input coverage and advance the scope interpretation. Obtain fresh qualification; old internal receipts need no preservation machinery and cannot be relabelled as current passes. | EFF01 corrects the scope/reuse implementation. Existing requested-effects and complete-input contracts remain; use a design amendment only for an actual contract change. |
| Derived-state cutover | Preserve authored/external inputs and current live obligations; rebuild affected generated internal state directly. No legacy path solely for history. | EFF00 identifies the affected store interpretation and owners. Use blueprint §20.5's creation/opening boundary and amend any incompatible durable-evolution contract through its decision route. Execute only the necessary coordinated cutover. |
| Acceptance basis | Invalid internal historical values are not scientific or performance oracles. First-principles thermodynamic validation remains outside this scope. | EFF00/EFF11 reconcile affected fixtures and qualification guidance with blueprint §24 and PS-13. Preserve legitimate external-reference tolerances and ordinary engineering tolerances; do not widen them to make a changed result pass. |

EFF00 allocates ADR IDs only when the actual decision is ready; this plan reserves none.
The original Revise review is evidence for problems, not target acceptance for a changed
contract. Obtain the scoped target review where the decision rules require it. This is not
a new product-wide review or a review requirement for every indexing change.

## Foundations and target design

### Reuse the semantic and numerical authorities

Retain authored physical meaning, complete selected dependency premises, exact full-basis
equality behind fast prehashing, library-owned mathematics/solvers, attempt-private mutable
state, original-space assessment, exact manifests and allocation ownership. Positive,
absent and membership dependencies remain necessary. Solver status is not scientific
acceptance. Current values, effects, protections and actual provider demands remain current
even when immutable preparation is reused.

The review's source-backed assessment supplies most foundations. Focused authoring inspection
also confirmed the selected test-build root in `scripts/select.py`, input interpretation 2
in `scripts/validation_scope.py`, complete-map access in runtime/Python/durable consumers,
the distinct AST and occurrence-string roles, and content-derived analysis keys guarded by
retirement records. Those existing contracts cannot simply be removed without migrating
their consumers. New indexes and compact views stay with the existing meaning owner, rather
than introducing another dependency graph, scientific registry or orchestration framework.

The useful extension boundaries are concrete: a new output relation supplies its encoder
and inventory entry without changing scientific completion; a new block strategy consumes
the same original-coordinate view; a new property supplies actual operand facts without
changing numerical policy ownership. A changed lexical scope invalidates admission even
when a numeric body appears equal. These scenarios shape the seams without adding new
product features to this plan.

### Local execution, build closure and qualification

Native capability selection, canonical effects and build targets are distinct inputs.
The existing composition root must consume an explicit execution-effects requirement.
Native-local execution keeps native setup, admission, exact Nextest selection, terminal
reconciliation and descendant drain, without building a complete worker or requiring the
canonical observer. Canonical execution keeps its worker/service association. Unknown
requirements remain conservative; a local invocation cannot silently acquire canonical
effects. Do not infer effects from the word `unit`, source scanning or an exact-test registry.

Build the selected responsibility's actual dependency closure. Investigate feature activation
on the pinned Cargo configuration before changing the mandate; inspect cold build targets
without clearing the shared target directory. Actual Arrow-consuming tests retain explicit
validation and the single type universe. Alternating semantic, composite and native selections
must retain intended feature sharing.

Qualification applicability covers all consumed source, execution policy and effective native
configuration. A conservative scripts closure is preferable to an incomplete clever subset.
Include the omitted scripts, benchmarks and selectors identified by F02, and check current
configuration association rather than rehashing only paths recorded by an old capture.
An unchanged-input guard may reuse a fresh successful report only under the new complete
interpretation. This does not impose strict artifact qualification on ordinary design iteration.

### Compact preparation and scientific context

Occurrence preparation renders/hashes a body once and records deterministic body-relative
positions. The target retains immutable body syntax once, with occurrence information sufficient
for binders, synthetic empty spans, admissions and diagnostics. Do not delete the checked AST
that drives dependency discovery. If compacting changes key/wire semantics, switch directly to
the new interpretation and regenerate derived artifacts; diagnostics can obtain syntax from
the owning body rather than retaining every rendered subtree.

Selected closure uses existing declaration ordering/ranges and owner-built reverse dataset
relationships, then constructs only the selected product. Imports, inherited/refined members,
scope visibility, provenance, supplier identity, test taint and missing-name consequences
remain complete. This removes selection amplification without presuming a full incremental
typed checker.

Conditional plans share one immutable original variable/parameter universe. Blocks retain
compact solved-coordinate selections, fixed overlays and original-ID mappings; mutable
workspaces remain private. Prepare predecessor access once. Complete arithmetic, guard and
provider dependencies must reach execution even when absent from numerical incidence.
Validate the original binding and retain transactional rollback and original post-solve checks.

Fresh numerical difference inference receives the actual immutable physical prerequisites
for its operands and operation. Thread that context through numerical allowance and conditional
boundary preparation, including invalidation when facts change. An invariant identifier alone
is not proof; scalar substitution and arbitrary tolerance changes are not corrections. Audit
the affected `NoInvariantFacts` uses by operation, rather than mechanically replacing qualified
receipt replay. Trace the actual PC-SAFT failure before attributing its first caller.

Fitting prepares per-experiment observation lists, response ranges and binding indexes once
under its existing sparse mapping owner. Refills follow actual contributions while preserving
source order, duplicates, exclusions, sample/output identity, shared-parameter addends, derivative
upgrade permissions and independent final candidate/rank checks.

### Selective output and durable publication

Scientific completion remains immutable and authoritative. Enumerate available relation names
from the completion/request inventory, then encode only demanded relations. Cache successful
encoding or its sticky failure per relation across clones. A failure in an unrelated relation
does not poison a valid selective request; complete access still fails if any required relation
fails. Transient transport/cancellation failures must not become scientific completion or
unrelated encoding failures.

Use one relation-aware encoding path for explicit complete-table convenience, single-relation
Rust/Python access, trajectories, fitting/shooting and bounded durable export. A useful complete
API is not a second legacy encoder. Chunk large relations directly from immutable native
completion instead of building the complete Arrow map before the first durable block.
Preserve schemas, units, ordering, exact identities, partial-result interpretation and escaped
buffer charges. Activate durable results only after the complete required manifest is sealed.
This removes unnecessary Arrow materialization, not inherent native report retention.

### Live coordination and complete analysis deletion

Hot resource metadata contains live owners, pending handoffs and unresolved reclamation.
Completed records leave that path only after authentic terminal disposition and actual drain.
Keep exact outcome/reference information only while a current owner consumes it, outside
full-ledger hot scans; discard obsolete history rather than inventing a permanent archive.
Missing or unknown metadata is not evidence of success or drain. Native generation discovery
must not repeatedly scan all completed operation payloads. Preserve unit/PID generation checks,
borrowers, failed/unknown pins and no-recreation requirements of live operations.

Separate analysis semantic content from a fresh analysis occurrence. Existing content-derived
keys plus permanent retirement markers are insufficient for marker-free deletion: deleting
the marker alone lets a late `begin` recreate the same analysis. Fresh creation therefore needs
a current admission/lifetime authority; retries, append and activation operate only on an
already admitted live occurrence and may never upsert a missing one. A deliberately recreated
analysis gets a fresh occurrence, even with identical inputs and method.

Use bounded creation admission tied to the original operation and its original finite
validity window, with a fresh occurrence identity issued by the creation owner. Admission
issuance itself requires current authority; replay cannot mint another admission, renew its
window or turn a missing occurrence into fresh creation. Equal-content deduplication is not
a requirement; a content digest identifies meaning, not an indefinitely replayable occurrence.
Reuse existing operation/session/protection machinery where it supplies this contract.
EFF00 settles the concrete admission/settlement boundary; EFF10 implements and exercises it
before implementing collection. Do not introduce permanent per-analysis markers, historical
readers or a general-purpose new lifecycle service.

Retirement closes admission, fences new mutation/read acquisition, waits for relevant existing
protections and writers to drain, and removes all analysis-owned edges/nodes/inputs/roots and
header in bounded resumable work. Expiry closes admission but does not prove that an already
dispatched transaction aborted or committed before the deadline. Final cleanup also requires
settlement of those transactions under the existing guarded lifecycle owner. Keep temporary
progress only while deletion is incomplete; then remove guards, retirement and admission/cleanup
records too. Migrate run-retirement predicates that currently interpret analysis input edges
through marker presence; marker removal cannot leave false retention obligations.

Analysis handles do not acquire a new indefinite reader lease. Preserve each current atomic
page/operation's protection and completion; retirement may refuse the next page of an existing
handle. Previously delivered copied buffers retain their allocation owner. A partially read
traversal cannot claim complete analysis after such refusal. Coordinate source/run obligations
without deleting independently retained data. Restart resumes unfinished cleanup without
scientific recomputation. Cleanup after complete deletion may report absence without proving
whether the occurrence was deleted or never created. Stale creation/read/append/activation
requests must refuse without leaving durable per-analysis residue.

## Implementation packets and dependencies

The table is the sole packet-progress owner. Detail below supplies target behavior and
revealing acceptance, not a second status ledger. Packet status below records implementation and acceptance progress. A working prerequisite means implemented and exercised behavior, not merely
an agreed interface or another plan's historical pass.

| Packet | Required input | Delivery and targeted acceptance | Progress |
|---|---|---|---|
| <a id="eff00"></a>EFF00 — Adopt affected contracts | Confirmed decisions above; focused target assessment of changed contracts. | Record required ADR/review/design amendments, derived-state cutover boundaries and analysis admission/fencing design. Reconcile fixture provenance and external versus invalid internal comparisons. Publish shared decisions before dependent code. | Complete: changed targets reviewed, ADR-0169–0172 proposed and enduring architecture/governance owners updated; implementation and actual-backend acceptance recorded in the Outcome. |
| <a id="eff01"></a>EFF01 — Complete applicability (F02) | Existing complete-input intent; independent of other remedies. | Expand scope/effective configuration and bump interpretation. Detect every reviewed omission with positive controls; exercise actual unchanged-input refusal using a newly successful bounded report after a relevant change. Remove obsolete scope/reuse assumptions. | Complete: consumed-input/configuration sensitivity and actual successful-report reuse refusal pass under the new interpretation. |
| <a id="eff02"></a>EFF02 — Local execution and build closure (F01/F04) | Settled requested-effects input; EFF00 rule amendment only if needed. | Migrate runner/capture/recipes and actual callers; remove unconditional worker/observer and unrelated relation build roots. Run the named native-local status control without canonical service plus canonical/managed positive controls, Arrow-validation and feature-sharing controls. | Complete: explicit effects, actual Cargo closure validation, generated feature ownership, native-local and positive canonical route/placement/drain controls pass. |
| <a id="eff03"></a>EFF03 — Compact occurrences (F05) | EFF00 occurrence interpretation decision if needed. | Migrate checking, binding and typed lowering together. Remove duplicate renders/discarded hashes and redundant subtree retention. Check deterministic occurrence identity, binders, synthetic spans, physical admission and diagnostics using independently specified fresh bodies. | Complete: compact body-relative occurrences and direct wire cutover integrated; modeling/compiler and actual Local receiving controls pass. |
| <a id="eff04"></a>EFF04 — Direct selected closure (F06) | Existing checked-package semantics; coordinate occurrence consumer edits with EFF03. | Range/index occurrence and dataset ownership; build selected-only products. Cover imports/refinements, shadowing, additions/deletions, absent names, provenance and taint. Delete whole-map-per-declaration and clone-then-filter mechanisms. | Complete: selected-only products integrated; complete dependency/provenance and joined occurrence controls pass; clone-then-filter construction removed. |
| <a id="eff05"></a>EFF05 — Shared conditional universe (F07) | Existing original-coordinate contracts; EFF06 context slice where fresh boundary differences consume it. | Migrate compiler/runtime initialization, automatic block composition and PETSc block consumers. Replace per-block full coordinate copies and repeated edge scans. Cover independent scalar blocks, predecessor chains, guards/providers, missing inputs, cancellation, rollback and original assessment. | Complete: shared coordinates/bindings and prepared predecessor indexes integrated; conditional/native and bounded authored recycle controls pass. |
| <a id="eff06"></a>EFF06 — Actual numerical facts (F08) | Current physical prerequisite owner and actual operand contracts. | Trace the first failure; migrate numerical policy and conditional-boundary inference and their callers. Test present/absent/changed facts and exact allowances, then rerun a freshly prepared PC-SAFT fixture under its current lawful scientific specification. No scalar fallback. | Complete: actual operand facts integrated, original first caller traced and fresh PC-SAFT cold/warm/value/structural stages prepared without relaxed allowances. |
| <a id="eff07"></a>EFF07 — Selective/chunked export (F09) | EFF00 result/Python error contract; existing checked builders, block writers and sealing. | Migrate ordinary, trajectory, fitting/shooting, Python names/table and durable consumers. Test unrelated relation failure, complete access failure, exact chunk rows/manifests, interrupted publication and escaped arrays under bounded transport memory. Delete complete-map-first encoding routes. | Complete: relation-local encoding and bounded cursors integrated across Rust/Python/durable consumers; focused/native/shooting/Python controls pass; complete-map-first export removed. |
| <a id="eff08"></a>EFF08 — Prepared fitting relationships (F10) | Existing sparse experiment/response mapping; independent of EFF07 export. | Index observations, response entries and local bindings; migrate steady/transient refill and final gradient-to-response upgrade. Verify sparse addends/gradients analytically on fresh fixtures, duplicates, exclusions and shared parameters, with final rank checks intact. | Complete: prepared sparse relationships integrated; analytic gradients/Hessians, duplicate/exclusion/refill, rank, fixed-fit and transient demand controls pass. |
| <a id="eff09"></a>EFF09 — Live resource coordination (F03) | Existing terminal/drain/reference contracts; EFF01 input coverage before relying on reused qualification. | Move completed ownership out of hot scans; delete unconsumed history and obsolete full-ledger/native discovery paths. Hold live participants fixed while growing completed history; test references, unknown records, repeated cleanup, surviving descendants and exact generation identity. | Complete: live-only coordination and one-pass pins integrated; joined tooling and actual lifecycle recovery/ownership controls pass. |
| <a id="eff10"></a>EFF10 — Full analysis retirement (F11) | Adopted admission/fencing design from EFF00; existing exact source/result protections. | First implement and exercise creation admission/settlement; then migrate runtime, generated declarations, operations, retirement predicates and Python consumers to fresh occurrences and bounded full deletion. Test complete/partial staging, dispatched creation crossing expiry/cleanup, late mutations, page readers, restart and fresh recreation. Verify no residue remains after stale retries. | Complete: fresh occurrences, guarded settlement and full bounded deletion integrated; actual-backend races, initializer admission, Python lifecycle and all 28 actual maintenance controls pass. |
| <a id="eff11"></a>EFF11 — Affected integration and closure | All functional packets, selected investigation corrections, consumer migration and deletion complete. | Run affected composed journeys and scope-end checks; resolve failures against zero. Record scoped acceptance and any actual measurements, move enduring meaning to owners and retire resolved work under ADR-0096. | Complete: affected joined acceptance, corrected scope-end checks, enduring-owner updates and current-work removal recorded in the Outcome. |

EFF01 is the first executable correction and need not wait for broader target choices.
EFF00 resolves shared decisions alongside it. EFF02, EFF03/EFF04, EFF06, EFF07, EFF08 and
EFF09 can proceed independently once their stated contracts are available. EFF05 consumes
only the needed working numerical-context slice, not all numerical qualification. EFF10
does not wait for all result export or resource-history work. EFF11 is the functional join.

Logical independence is not edit isolation: EFF02/EFF09 share runner/resource scripts;
EFF03/EFF04 share checking products; EFF05/EFF06 share boundary preparation; EFF07/EFF10
share runtime/store consumers. Assign one writer per shared surface during concurrent work,
with the root responsible for integration. Regenerate changed declarations through
`just codegen`; never hand-edit generated queries, codecs or Python contracts.

## Bounded investigation packets

Each packet owns a decision and any selected correction through its consumer integration,
targeted acceptance and EFF11 handoff. The table owns their completed decision status;
the investigation decisions below record adoption, retained settings and triggers. An adopted alternative must name what it
replaces and remove that implementation after migration. A retained design needs a concrete
semantic/workload reason and observable reconsideration trigger; numerical noise alone is
not a reason. Unresolved questions constrain only their dependent change.

| Packet | Question, evidence and decision boundary | Integration and acceptance | Progress |
|---|---|---|---|
| <a id="ei01"></a>EI01 — Selected gather | Can selected forecasting/scratch follow touched source extent instead of the full source while remaining conservative for duplicate multiplicity and physical aliases? Inspect current checked-take ownership and representative nested/variable-width layouts; do not reopen the repaired historical quadratic bug as an unfixed diagnosis. | Adopt selected forecasting/scratch where sound, otherwise identify the exact alias/layout limitation. Integrate at checked take, with bounds/nulls, duplicates, empty selection, reservation refusal and escaped ownership controls. | Complete: selected forecasting/gather adopted for supported layouts; conservative library fallback retained where required; bounds/alias/ownership controls pass. |
| <a id="ei02"></a>EI02 — Selected scalar verification | Can exact entity/field/partition selection avoid rebuilding/sorting the complete scalar metadata index? The ordinary once-per-64-cells decode claim was retracted and must not become a test premise. | Integrate selected metadata verification at the existing protected reader. Check sparse/empty/duplicate requests, exact partition keys, digest/index integrity and expiry. Retain broader checks only where required for complete admission. | Complete: selected scalar verification adopted; protected reader and exact partition/digest controls pass. |
| <a id="ei03"></a>EI03 — Analysis scope and extent | Establish one actual required workload before changing the 4,096-node/8,192-edge limits. Determine the semantically sufficient input graph before root selection; bounded output does not imply local input. Compare existing visitors/native connected retrieval with full construction. | Select demand-first construction, a sound larger-scale route, or explicit supported limitation. Migrate the actual consumer and preserve its method's complete reachability, edge meaning, source/evidence correspondence and resource refusal. Coordinate identity/retirement with EFF10; invent no SLA or general graph-service requirement. | Complete: current complete graph method and supported limits retained; workload, early validation and exact-cap controls established. |
| <a id="ei04"></a>EI04 — Typed checker incrementality | Consume GH3/N0's observations and candidate as leads, not old timing thresholds. Settle typed positive/negative/membership, visibility, deletion, facts/provider/policy, document/provenance and failed-publication atomicity domains. Assess whether current owners/Salsa can remove actual upstream rechecking. | Adopt a complete incremental realization if it works; migrate publication/checker consumers and remove the replaced full route. Otherwise retain complete checking with the exact unresolved obligation and trigger. Differential checking may be a temporary investigation tool, never the sole oracle or a permanent second production checker. EFF03/EFF04 proceed independently. | Complete: complete authored checking retained for the concrete missing dependency/publication domains stated below; reconsideration trigger recorded. |
| <a id="ei05"></a>EI05 — Graph, hashing and pure persistence | Reassess specific reuse opportunities from current graph/hash foundations: exact content equality, operation-shaped graph layouts, portable reconstruction and bounded pure Recipe persistence. Determine complete keys, producer/interpretation, receiving trust, resource lifetime and current-effect boundaries. Compare complete preparation/restart work, not key hashing against preparation. | Select and integrate each applicable correction in this plan; changed durable hashing/persisted-runtime contracts use their required ADR/review route first. Preserve collision/equality, missing/membership invalidation, fresh provider demand/state and the selected receiver admission/reconstruction contract. Stronger `QualifiedProducer` assurance applies only to its selected profile, not every deployment-local reuse. No broad store/solver placement pivot follows from these findings alone. | Complete: bounded pure Recipe persistence and exact semantic identity retained; compact body cutover and actual Local receiving controls integrated. |
| <a id="ei06"></a>EI06 — Build tuning | After EFF02 removes unrelated work, examine pinned profiles/features/cache/native preparation for actual supported developer routes. Separate build, setup and test execution; use fresh comparable observations where useful without clearing shared targets or caches. | Integrate technically sound expected-beneficial changes and affected callers. Preserve floating-point guarantees, unwind, pinned toolchain and exact dependency families. Test the changed route and relevant alternation; retain a setting only for a concrete fit/correctness or significant-performance reason. | Complete: admitted-CPU jobs and selected-root build observation adopted; existing scientific/compiler settings retained with stated fit reasons. |

EI01/EI02 can start from existing data owners. EI04 can start domain design independently
of occurrence layout; coordinate only its consumed representation. EI03 consumes EFF10's
identity/lifetime decisions if its stored representation changes. EI05 does not reopen
implemented J3/B8/N12/N13 corrections or automatically select Graph/hash RC02/RC03.
EI06 follows local build isolation so tuning does not hide unrelated work.

If an investigation establishes a consequential new architecture beyond its bounded question,
record the concrete decision and its route here before implementing it; do not silently spread
it into Plan 28. Actual deferred decisions use the existing ADR register with an owner and
observable trigger. A simple retained-setting conclusion is not a speculative new backlog.

## Existing-plan and workspace boundaries

| Existing owner | Consumed contract or handoff |
|---|---|
| Plan 28 coordinator/28e | Preserve original scientific dispositions and assembled campaign obligations. Link this plan's independent ownership and later evidence; do not mirror its packets or make its completion await E3/E4/E5. |
| 28f numerical-fact boundary | F08 correction and future targeted status move to EFF06. Preserve the original four typed refusals and the originally unlocated first caller as historical observations, not a PC-SAFT pass or correctness baseline; EFF06 now records the traced attribution. Broader numerical consumer/scientific obligations stay with their existing owners. |
| 28f N0/28k GH3 | Their original complete-checker conclusion remains an observation of the earlier scope. Prospective checker-domain decision/integration moves to EI04; no competing proposal owner remains at N0. |
| 28b/28j/28f | Consume complete dependencies, exact preparation basis and fresh numerical registration; EFF03–EFF06/EFF08 own the newly adopted work. Shared files do not transfer all earlier B/J/N scope. |
| 28d/28g | Consume protected read, checked block, complete activation and buffer ownership. EFF07 owns selective encoding and EFF10 owns full analysis deletion. Retention/reopening assumptions are amended through this plan's contract route; earlier staging/paging outcomes keep their original conditions. |
| Plan 30c/30d | Consume terminal identity, actual drain, protection/reference and host-admission contracts. EFF02/EFF09 migrate the new local-effects/hot-history behavior; do not relabel Plan 30's historical acceptance as evidence for these changes. |
| Plan 32 | Workspace document/artifact lifecycle remains separate from persistent scientific analysis state. Coordinate shared resource-script edits and producer artifact meaning where they intersect; neither plan waits for completion of the other. |

The cancelled feature matrix remains stopped. Any future matrix invocation is selected
explicitly at the relevant scope end, not an automatic continuation of that run. Historical
interrupted reports establish no successful coverage. Useful review observations retain their
conditions while consumed; resolved reviews/plans retire to Git history rather than acquiring
new archive copies or permanent evidence-preservation machinery.

## Verification and completion

**Implemented verification approach:** during functional work, use `just check-package <pkg>` (or
`just check` for actual cross-crate scope) and the packet's targeted `just unit-package`
controls. Native controls use the selected native-capability recipe and checkout environment.
Tooling packets use focused `scripts.tests` controls through `scripts/pse-env -- .venv/bin/python`.
Keep actual Arrow consumers force-validated under the adopted RC01 mechanism. A new mechanism
gets revealing controls in the same change; remove tests belonging solely to deleted machinery.

EFF02's concrete native-local control is
`just unit-native-capability-package pse-backend-native ipopt solver status_scope_and_finite_infinity_are_not_conflated`.
Its success must not depend on canonical observer/store configuration. Positive canonical
controls separately establish required effects, placement, receiver association and drain.
Choose Nextest filters from current tests when execution starts; this plan does not create an
exact-test-name admission registry.

Revealing expectations are independently specified: exact selected declarations/occurrences,
lawful and unlawful physical operand facts, small analytic sparse-gradient examples, known
graph relationships, exact row coverage and explicit lifecycle outcomes. Current-run roundtrip
and restart controls test current identities and representation fidelity using freshly generated
state. They do not establish scientific truth by matching old internal bit patterns. Inspect
existing historical-value assertions for provenance; retain applicable external IDAES/Pyomo or
published analytical comparisons and remove invalid internal snapshots. Do not change external
pins, weaken physical/solver checks or enlarge tolerances merely to preserve a passing test.

Once functional packets, selected investigation implementations and consumer deletions are
complete, EFF11 selects affected Rust integration/component/solver and native Python journeys:
edit/value/structure preparation, lawful PC-SAFT preparation, independent/chained initialization
and recycle, mixed fitting with final rank, selective trajectory/result transport, protected
durable publication/reopening and analysis retirement/recreation. Include the affected external
reference comparisons with their original assumptions. Use a bounded current workload for each
contract; do not substitute a product-wide campaign for a missing targeted control.

Run `just hygiene` and the applicable manual `just governance`, `just docs`, native-data/contract
and feature-powerset checks at scope end under AGENTS.md. Select relevant performance campaigns
for concrete benefit or suspected significant-regression questions. A published speedup needs
matched complete operations and recorded source/profile/native configuration, workload, concurrency
and resource conditions. Slight/noisy timing changes do not block adoption. Do not freeze old
artifacts, run broad campaigns per packet or clear caches to manufacture comparable evidence.

Report commands, modes, scope and actual results against the zero-failure baseline. The source
review's unresolved whole-simulator PS-G1/PS-G3 judgments are not converted to passes by targeted
remediation. **Tested** and **Measured** claims name their actual controls and conditions;
**Implemented** requires migrated consumers and deleted replacements. The actual **Tested**
scope, commands, zero-failure baseline and exclusions are recorded in the Outcome below;
rendering this document supplies no additional product evidence.

Completion requires all eleven findings resolved with the correction's evidence, all six
investigation decisions and selected integrations complete (or explicit properly owned
decision deferral), affected assembled acceptance complete and enduring contract changes at
their proper owners. A packet can finish without qualifying unrelated scientific scope.
EFF11 records the required Outcome, including an actual mistake corrected and deliberate
deviations. Remove the plan from current-work publication and retire resolved material when
its enduring meaning has an owner and no live consumer remains, following ADR-0096.

## Finding dispositions

This is the sole disposition owner for the efficiency review's F01–F11. Scenario references
are the review's existing S01–S07, not a new registry. Scheduling is not resolution.

| Finding | Scenario | Disposition | Decision/work owner | Evidence or completion condition |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f01) | S01 | Resolved | EFF02 | Native-local status control and positive canonical runtime/Python route, placement and drain controls passed; see Outcome. |
| [F02](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f02) | S01 | Resolved | EFF01 | Joined applicability controls passed actual newly stale successful-report refusal and consumed input/effective configuration sensitivity; see Outcome. |
| [F03](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f03) | S07 | Resolved | EFF09 | Live-only hot coordination, one-pass pins and 213 joined tooling controls passed; actual lifecycle ownership/recovery acceptance passed; see Outcome. |
| [F04](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f04) | S01 | Resolved | EFF00/EFF02 | Actual Cargo target/host/dev closure controls and native-local/canonical alternation passed; explicit validation retained for Arrow consumers; see Outcome. |
| [F05](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f05) | S02 | Resolved | EFF00/EFF03 | 24 modeling and 11 compiler controls plus actual Local reconnect/cold-miss/corruption/absence controls passed under the direct wire cutover; see Outcome. |
| [F06](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f06) | S02 | Resolved | EFF04 | Selected closure preserves imports/refinements, shadowing, membership, provenance and taint; focused modeling and joined consumer controls passed; see Outcome. |
| [F07](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f07) | S03 | Resolved | EFF05 | Shared original coordinates and prepared complete predecessor/guard/provider access passed conditional/native and three bounded recycle/temporal controls; see Outcome. |
| [F08](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f08) | S06 | Resolved | EFF06 | Present/absent/changed actual-fact controls passed, original first caller traced and fresh PC-SAFT four-stage preparation passed; see Outcome. |
| [F09](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f09) | S05 | Resolved | EFF00/EFF07 | 30 focused runtime plus native/shooting/Python controls passed selective encoding, relation-local failures, bounded windows and durable sealing; see Outcome. |
| [F10](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f10) | S04 | Resolved | EFF08 | Prepared sparse refill, analytic gradient/Hessian, duplicate/shared addend, rank and transient demand controls passed in the joined runtime selection; see Outcome. |
| [F11](../design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md#f11) | S07 | Resolved | EFF00/EFF10 | 13 combined actual-backend race/initializer controls, public Python lifecycle and 28 real maintenance checks passed full deletion, stale refusal and fresh recreation; see Outcome. |

## Current checkpoint

All functional packets and selected investigation integrations are complete. The actual
maintenance journey passed all 28 controls, including interrupted initialization/recovery,
old-database nonaddressability, account/creation-authority refusal and preservation of
unknown owners. The final combined operations selection passed all 13 initializer/race
controls. Joined tooling passed 213 controls. Scope-end static findings were corrected
and the failing recipes rerun successfully.

Final same-source coverage passed: 29 runtime controls under the default profile and the
one timed-out fitting control under the local scientific profile; the current-extension
public Python lifecycle rerun also passed. Enduring meaning is at its existing owners;
this completed plan leaves current-work publication. The Outcome owns commands, conditions
and exclusions. No comparative speedup or whole
simulator qualification is claimed. Concurrent Plan 32 work is preserved; the cancelled
workspace matrix and Plan 28 E3/E4/E5/reference campaign remain paused.

### Investigation decisions

EI01 adopts selected touched-range forecasts and copies for supported Arrow layouts, with
conservative library take for unsupported layouts. Bounds, duplicate multiplicity, nested
offsets/nulls, allocation ownership and empty dictionary/view independence are explicit controls.
EI02 adopts exact selected scalar-cell verification at the existing protected reader; complete
relation admission remains complete. Its selected-cell controls and connected-reader journeys
passed at the joined acceptance boundary.

EI03 retains 4,096 nodes, 8,192 edges and 4,096 evidence rows for the current complete method.
The actual results-chain/results-publish-chain/source-growth workload has 128 states by 31
samples: 3,968 response rows, 7,936 quantitative/provenance edges and at most 65 source edges.
Root selection still consumes the complete admitted graph. Early root/control validation and
exact-cap/+1 refusal are integrated; no prefix graph or arbitrary cap increase is selected.

EI04 retains complete authored checking and the existing exact unchanged-publication,
admitted-revision and Salsa document-table reuse. `workspace/modeling.rs` publishes a complete
`CheckedPackage`; changed publication calls `check_with` before selection. The unresolved
incremental domains are earlier absent lexical/import/visibility candidates, complete package
and effective-member inventories, deletion/recreation, duplicate/owner constraints and global
inheritance/call-cycle merge/split; exact physical/document interpretations, provenance and
current diagnostic witnesses; staged failure/cancellation/limit atomicity and bounded memo
ownership. A selected positive-edge graph cannot establish these obligations. Revisit when
complete typed input/query domains satisfy them with independent accepted-package and
classified-current-error controls. Compare full publication/preparation and retained memory;
no historical scalar threshold or minimum speedup applies. No second production checker or
whole-revision cache is selected: admitted-revision handles already supply exact reuse.

EI05 retains semantic BLAKE3, exact equality behind bucket prehashes and the existing exact
factorable-node interner. Current FlowProjectionV2 already frames roles and counts; the older
V1 segmentation concern is resolved in the current source. EFF03 adds the selected compact
body ownership and Recipe v2/runtime envelope v4 cutover. The bounded compiler-issued Recipe
is the pure persisted product: it carries selected mathematical/physical meaning and exact
receipt/proof streams. Process-local BasisKey/frontier/Salsa IDs, native handles and mutable
effect owners are not additional serialization targets. Local receiving admission is selected
without an explicit producer receipt; QualifiedProducer applies when explicitly selected.
The actual Local reconnect, cold-miss, corrupt-acquisition and additional absence-recheck
controls passed, preserving fresh provider demand, source attribution, generation fences,
cancellation/drain and allocation ownership. No separate persisted runtime is selected
because no uncovered pure product has been identified.

EI06 adopts Cargo jobs bounded by admitted logical CPUs and actual selected-root build
measurement, removing incidental workspace-hack Arrow observations as validation authority.
Retain checkout-local targets/shared sccache, non-direct sccache, mold, workspace incremental
compilation and the existing dependency/scientific optimization profiles for their present
fit; unwind and floating-point guarantees remain. Explicit caller settings remain authoritative.
The changed tooling controls pass; no wall-clock speedup is claimed from concurrent runs.

### EFF06 original refusal attribution

The original PC-SAFT probe executable (SHA-256
`a118fc57aac18bc8f96ee9c566e473171a7875105d1a458155038306678975f1`,
source baseline `4c24721e691187e1a5b28398b29722fbde671da8`) was traced with
GDB against a disposable diagnostic v2 substrate using the same pinned SurrealDB
3.3.0 binary. The first `NoInvariantFacts::check` call comes through
`engineering_error_quantity`, candidate filtering in `engineering_default`,
`pse_math::numerics::resolve`, and the outer modeling `cases` preparation. A later
distinct stack enters engineering magnitude resolution through the same outer owner.
This locates the original missing-facts boundary; it does not establish a historical
scientific result or implicate inner implicit preparation as the first caller.
The diagnostic profile was normally drained and stopped; its four refused stages
remain diagnostic observations rather than numerical expectations.

**Tested:** the current native, force-validated preparation smoke command
`python -m scripts.case_measure build/eff33-pcsaft-02-current-retry --smoke --case pcsaft-02`
completed with cold, warm, value-change and structural-change stages all explicitly
reporting `prepared` (17 bodies and a 4-by-4 fixture). A single smoke observation on
the concurrent host does not qualify comparative performance or scientific parity.
The lawful authored fixture and numerical allowances were not relaxed.

## Outcome (recorded after implementation)

### What was built

**Implemented:** the standalone remediation replaces the reviewed amplification paths:

- Applicability includes consumed scripts, selectors, benchmarks and effective native
  configuration under a new interpretation. Native-local execution consumes explicit
  effects without acquiring canonical workers/observers. Arrow validation follows the
  actual Cargo target/host/dev-dependency closure rather than adding unrelated roots.
- Physical occurrences share body syntax and deterministic body-relative positions;
  selected products use declaration ranges and reverse dataset relationships. Conditional
  blocks share their original coordinate universe and prepared predecessor indexes.
  Fresh numerical inference receives actual immutable operand facts. Fitting refills use
  prepared observation/response/binding relationships while retaining final rank checks.
- Rust/Python name discovery does not encode tables. Relation-local encoding caches and
  bounded native-completion cursors serve selective and complete consumers through the
  same producers. Intrinsic encoding failures are sticky per relation; request allocation
  and delivery failures remain retryable. Durable activation still requires a sealed
  complete manifest. Complete-map-first production export was removed.
- Resource coordination scans live obligations, with exact cold evidence lookup and
  one-pass native pin discovery. Analysis occurrences have fresh identities, finite
  creation authority, source-incarnation/floor fencing and settlement-aware bounded
  deletion. Completed deletion leaves no per-analysis header, graph, lineage, roots or
  permanent retirement record. Late creation/mutation/read requests refuse.
- Closed rebuild/restore/recovery uses a configured, lifecycle-owned temporary backend,
  fresh physical identities, production initialization, explicit input preservation,
  authenticated catalog checks, account rotation and verified old-database disposal.
  Initializer authority requires the exact live owner; zombie/dead owners refuse.
  Failed or uncertain maintenance stays closed and charged until authentic drain.

**Implemented:** EI01/EI02 adopt selected gather and selected scalar verification. EI03
retains the complete method's supported graph limits with early validation and exact-boundary
controls. EI04 retains complete authored checking because the proposed partial dependency
model cannot establish complete publication semantics; its concrete reconsideration domains
are recorded above. EI05 retains semantic BLAKE3, exact equality, the existing node interner
and bounded pure Recipe persistence, integrating compact body ownership and the direct
Recipe v2/runtime envelope v4 cutover. EI06 adopts admitted-CPU Cargo jobs and selected-root
measurement while retaining the justified compiler/cache/floating-point settings. These are
completed bounded decisions, not six new implementation backlogs.

**Implemented:** enduring contracts and operation belong to blueprint §5/§14, §19.2/§21,
§20.4/§20.5 and §24, and the build, validation, environment and
[substrate guide](../dev/surreal-substrate.md). ADR-0169–0172 record the changed targets and
their scoped target reviews. They remain **Proposed** pending the repository's ADR adoption
workflow; implementation and targeted acceptance do not change their status automatically.
There is no compatibility decoder, historical-result oracle or second production checker.

**Tested — execution conditions:** controls ran on 2026-10-09 America/New_York
(scope-end completion on 2026-10-10 UTC), from the joined dirty checkout based on
`4c24721e691187e1a5b28398b29722fbde671da8`. Commands used `scripts/pse-env --`, the pinned
nightly, checkout Python and pinned native libraries. Canonical journeys used an explicitly
selected disposable v3 profile with actual backend ownership; actual maintenance used fresh
private profiles after other journeys drained. The final allocation was exclusive within
this repository's admission system; unrelated repositories were still active. Actual Arrow
consumers carried explicit force-validation. Every reported pass has a zero-failure baseline;
overlapping selections are not additive test coverage.

| Scope and actual command | Result against zero failures |
|---|---|
| `just check-package pse-runtime -p pse-operations -p pse-py --features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-tests` | Passed joined native/canonical compile. |
| `just unit-package pse-modeling 'test(body_inventory_) \| test(physical_admissions_) \| test(selected_closure_) \| test(checked_occurrences_) \| test(generic_physical_admission_) \| test(static_reduction_and_fold_) \| test(process_contract_static_inline_)'` | 24 compact-occurrence, physical admission and selected-closure controls passed. |
| `just unit-package pse-compiler 'test(portable_body_local_) \| test(portable_admitted_body_) \| test(checked_occurrence_handoff_) \| test(checked_compound_guards_) \| test(checked_finite_reduction_) \| test(typed_source_identity_)'` | 11 compiler occurrence/portable/admission controls passed. |
| Native/canonical `just unit-package pse-runtime` selections specified below | 30 export/projection/fitting controls, 18 native/reader/Local receiving controls and 14 conditional/shooting/trajectory/analysis controls passed. |
| `just unit-package pse-runtime 'test(authored_causal_recycle_retains_topology_and_refuses_hidden_inputs) \| test(indexed_mixer_holdup_separator_executes_conditional_recycle_and_external_closure) \| test(indexed_mixer_holdup_separator_temporal_closure_is_independent_and_requires_initial_inventory)' --features pse-runtime/native-solvers,pse-runtime/canonical-tests --profile local` | Three bounded authored recycle/temporal controls passed. |
| `shooting_trajectory_projection_retains_completion_diagnostic_and_lease`, selected through `just unit-package pse-runtime` | Passed nonempty sample-major windows of sizes two/four against the same current immutable completion, preserving rows, values, ranges and completeness. The broader command containing reference cases failed, as recorded below. |
| `just unit-native-capability-package pse-backend-native ipopt solver status_scope_and_finite_infinity_are_not_conflated` | One native-local control passed without canonical observer/worker selection; positive canonical journeys separately passed. |
| `just unit-package pse-operations 'test(canonical_analysis_) \| test(canonical_admission_unit::)' --features pse-operations/canonical-tests` | Final joined run: all 13 passed (nine actual-backend deletion/creation/expiry/lost-ack controls plus four initializer controls), including zero residue after stale retries and an actual unreaped dead child. |
| `.venv/bin/python -m scripts.arrow_validation nextest run --no-fail-fast -p pse-math --lib --locked -E 'test(actual_numerical_facts_check_affine_operands_and_preserve_exact_allowances)'` | Passed the pure numerical-facts closure without unrelated Arrow roots. |
| `just py-sync-native`; `just py-test` with the eight explicit nodes below and `-m 'unit or component or integration' -q` | Current native extension built; all eight selected Python journeys passed through the six initial passes and two corrected-case reruns. |
| `just parity -q -k 'test_00_preflight or test_fit_covariance_agrees_with_parmest or test_sensitivity_agrees_with_ipopt_sens or test_degenerate_sets_agree_with_degeneracy_hunter'` | Eight checks passed: five preflights and three external comparisons, isolated parity Python 3.13.15, IDAES 2.13.0; original reference pins/tolerances retained. |
| `.venv/bin/python -m unittest scripts.tests.test_surreal_server scripts.tests.test_arrow_validation scripts.tests.test_efficiency_applicability scripts.tests.test_efficiency_native_pins scripts.tests.test_execution_contracts scripts.tests.test_native_tests scripts.tests.test_plan30_routing` | 213 joined tooling controls passed, including namespace-only disposal and confirmed absence. |
| `.venv/bin/python scripts/tests/efficiency_maintenance_actual_check.py --directory /home/paul/.local/state/pse-arrow/eff33-maintenance-acceptance-20261009-retry5 --port 18243` | All 28 actual-backend controls passed; no final drain errors. Repeat only with a fresh directory. |

The three runtime selections above used
`--features pse-runtime/native-solvers,pse-runtime/canonical-tests`. Their exact filters were:

```text
# 30 export/projection/fitting controls
test(result_export::tests) | test(result_projection_unit::) |
test(uncertainty::projection_tests::) | test(strategy_export_windows_) |
test(strategy_windows_) | test(native_certificate_and_text_copy_) |
test(canonical_failed_export_prefix_) | test(prepared_fitting_relationships) |
test(compiled_weighted_loss_gradient_and_exact_hessian) |
test(gauss_newton_hessian_matches_jtwj) |
test(sparse_fit_admission_tracks_support_and_refills_duplicates) |
test(bounded_rank_diagnostic_does_not_disable_sparse_candidate_evaluation) |
test(steady_response_solves_the_compiled_implicit_closure) |
test(all_fixed_fit_uses_joined_direct_evaluation_and_explicit_source_export) |
test(library_parameter_rank_has_independent_controls) |
test(mixed_shared_parameter_gradient_uses_inline_forward_sensitivities) |
test(transient_fit_demand_cache_and_coherent_upgrade)

# 18 native/reader/Local receiving controls
test(trajectory_transport_shares_completion_retries_budget_and_retains_escaped_batches) |
test(uncertainty_propagation_linear_exact) | test(propagation_withheld_when_upstream_withheld) |
test(linear_regression_covariance_analytic) |
test(profile_likelihood_matches_wald_on_linear_model) |
test(sensitivities_published_with_local_validity) |
test(sensitivity_withheld_without_local_analysis) | test(infeasibility_certificate_published) |
test(kernel_diagnostics_retain_requested_structure_without_objective_or_solve_admission) |
test(kernel_diagnostics_name_sources_and_keep_numerical_rank_distinct_from_structure) |
test(root_response_publication_has_physical_primal_and_no_kkt_fields) |
test(canonical_decoded_arrow_survives_reader_drop_and_result_reclamation) |
test(canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks) |
test(canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation) |
test(canonical_portable_body_explicit_local_receiving_acquisition_survives_reconnect) |
test(canonical_portable_body_cold_miss_skips_local_runtime_observation) |
test(canonical_portable_body_corrupt_explicit_acquisition_refuses_before_retention) |
test(ordinary_preparation_rechecks_additional_acquisition_absence_before_basis_reuse)

# 14 conditional/shooting/trajectory/analysis controls
test(first_block_binding_retains_shared_parents_until_last_alias) |
test(block_commit_is_atomic_and_requires_native_success_plus_original_quality) |
test(conditional_failure_preserves_shared_terminal_causes_and_actual_native_status) |
test(automatic_blocks_execute_complete_nonport_coupled_original_and_keep_actual_components) |
test(automatic_blocks_merge_domain_control_cycle_and_refuse_single_block) |
test(automatic_blocks_refuse_original_objective_coupling) |
test(actual_multiple_root_suppliers_consume_nonzero_authored_offsets_and_chain_actions) |
test(automatic_authored_supplier_actions_match_complete_original_equations) |
test(compiled_flow_keeps_original_offsets_scope_and_auxiliary_role) |
test(compiled_original_guard_refuses_invalid_flow_initial_without_proposal) |
test(actual_compiled_guard_inventory_and_deadline_are_mandatory) |
test(trajectory_transport_shares_completion_retries_budget_and_retains_escaped_batches) |
test(shooting_trajectory_projection_retains_completion_diagnostic_and_lease) |
test(canonical_original_dependencies_and_retained_sensitivity_persist_exact_sources)
```

The eight Python nodes were:

```text
python/pse/tests/test_native_workflow.py::test_explicit_cone_strategy_preserves_native_qualification
python/pse/tests/test_native_workflow.py::test_fixed_fitting_sources_round_trip_and_use_shared_result_lifecycle
python/pse/tests/test_plan14_acceptance.py::test_public_dynamic_and_transient_fit
python/pse/tests/test_modeling_kernel.py::test_modeling_simulation_events_checks_and_terminal_reports
python/pse/tests/test_modeling_kernel.py::test_modeling_diagnostics_inspect_singular_case_without_solver_admission
python/pse/tests/test_modeling_kernel.py::test_modeling_authored_fixture_shared_checks_and_owned_tables
python/pse/tests/test_studies.py::test_durable_study_retains_exact_results
python/pse/tests/test_canonical_results.py::test_canonical_result_selection_and_progress_reopen
```

**Tested:** the final current-extension rerun of
`just py-test python/pse/tests/test_canonical_results.py::test_canonical_result_selection_and_progress_reopen -m 'unit or component or integration' -q`
passed its one selected public lifecycle journey in 83.26 seconds, against zero failures.

**Tested:** the final same-source 30-control runtime rerun under the default profile
reported 29 passed and one timeout at its 120-second limit; that command failed.
`just unit-package pse-runtime 'test(prepared_fitting_relationships_preserve_mixed_gradients_and_rank_upgrade)' --features pse-runtime/native-solvers,pse-runtime/canonical-tests --profile local`
then passed the remaining control in 90.836 seconds, against zero failures, without
changing the fixture or its assertions. Final coverage is the 29 passes plus this local
pass, not a successful default-profile rerun of all 30. The earlier 30-control run passed
as recorded above; concurrent host timing establishes no regression magnitude.

**Tested:** actual maintenance installed the production current schema and typed cleanup
rows; restored into a fresh physical database; retained the closed-through floor while
rotating source authority; removed analysis payload/roots; preserved explicit authored
input hashes, external content and the original backup; refused old ROOT credentials and
the original creation incarnation; and proved the retired database absent. A real
initializer child acknowledged its exact target and then exited 23. Recovery selected a
fresh target, removed both superseded databases and stayed closed until validation/start.
An unknown nonempty database refused global account rotation without changing its content
or original accounts; a known empty database remained intact. These controls exercised
the pinned SurrealDB 3.3.0 backend, not just mocked SQL responses.

**Tested — scope-end checks:** `just hygiene` ran once after functional integration. Its
first result was four failed recipes, not a pass: Python lint/type checking and both
Clippy modes. Findings were corrected; `just lint-py`, `just typecheck`,
`just clippy-default --features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-operations/canonical-tests`
and `just clippy-no-default` then passed with zero findings. The other hygiene steps,
including generation equivalence and warnings-as-errors rustdoc, passed. Manual
`just governance`, `just docs`, `just lint-native-contracts` and `just lint-native-data`
passed. The bounded powerset command below passed all 15 selected combinations; it is
not the cancelled workspace matrix:

```bash
CARGO_HACK_CARGO_SRC="$PWD/scripts/cargo_feature_check.py" \
PATH="$PWD/.venv/bin:$PATH" cargo hack --keep-going check \
  -p pse-columnar -p pse-operations --feature-powerset --depth 2 --locked
```

**Tested:** final `just turn-end` passed both ADR indexing and formatting, with zero
failures. No product qualification was repeated for the subsequent documentation-only
closure edits.

**Tested, not Measured benefit:** the fresh PC-SAFT preparation smoke completed all four
stages, as recorded above. No matched complete-operation performance comparison was
established on this concurrently loaded host; no quantitative speedup is claimed.

### A mistake made and corrected

The first real maintenance journey exposed assumptions that mocked phase tests could
not establish. Embedded CLI access dropped the declared RocksDB configuration; maintenance
now uses a configured temporary server under the exact lifecycle owner. SurrealDB 3.3
emitted extra result slots for plain deletion and required a namespace for assigned INFO
expressions. Producers now suppress extra write results with LET-bound operations, and
catalog reads use direct INFO with a separate exact nonce acknowledgment and strict shape
checks. Generic write acknowledgment was not loosened. Selecting the database while
removing it also left it addressable in the actual control. Disposal now selects only its
existing namespace and requires a catalog absence postcondition before completion.

An earlier analysis assertion equated semantic content with occurrence identity; it now
requires a fresh key and independently checks unchanged content fields. Other controls
incorrectly assumed nonempty batches or ignored default Python markers, uninitialized
CLI default state and a still-valid creation grant. The repaired controls accept legal
empty batches, select the intended nodes, initialize the actual default target and wait
for the recorded server deadline before attempting settlement-aware cleanup. They do not
treat expiry, cancellation or socket drain as transaction settlement. Static fixes included
test environment isolation and inventory mocks that now model actual per-state catalogs
rather than fabricating a fresh target before initialization succeeds.

### Deviations from the plan, deliberate

EI03/EI04/EI05 completed by retaining the concrete supported complete graph method,
complete authored checking and bounded pure Recipe product where no correct broader
replacement was established. The reasons and reconsideration triggers are recorded in
their investigation decisions; marginal timing did not reject an applicable change.
Maintenance's temporary server replaces the technically incapable embedded-CLI route.

Affected integration used bounded authored recycle controls under the local profile.
An accidentally broader reference selection produced one passing shooting control,
one timeout, one interrupted case and one unrun case; it does not qualify that selection.
Its exact owned invocation was normally interrupted/drained. The cancelled workspace
matrix and Plan 28 E3/E4/E5/reference campaign remain paused and are not Plan 33 closure
requirements. No full simulator qualification, whole-review PS-G1/PS-G3 pass, new
first-principles thermodynamic framework or matched performance improvement is claimed.

The actual maintenance driver preserves `.pse` hashes but does not compile/reimport them
into a scientific model. Its authority controls cover physical disposal, authentication
and source incarnation; delayed creation replay is covered by the separate actual-backend
Rust races. Its injected failure is a real initializer exit after initialization, while
SIGKILL/fsync power-loss scenarios have targeted phase-control coverage only. These limits
do not convert preserved bytes into scientific reconstruction evidence.

Enduring meaning is in its existing owners. This highest-numbered completed plan stays
under ADR-0096 and leaves current-work publication; reviews still consumed by the proposed
ADRs and the source review's unresolved whole-simulator judgments remain available.
Concurrent Plan 32 work and the paused Plan 28 ownership are preserved.
