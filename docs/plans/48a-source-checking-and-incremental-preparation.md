---
title: Source checking and incremental preparation
status: done
date: 2026-10-20
adrs: []
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [current-work, preparation, performance, design]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
---

# Source checking and incremental preparation

## Purpose and authority

This companion develops the source-checking part of [Plan 48](48-execution-performance-and-numerical-efficiency.md).
It consumes the completed [recommendations audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md),
particularly AF01, AF02 and AF18 and the corrected R2 P01/P02/P18 remedies.
The coordinator owns all finding dispositions, rule decisions and inquiry state. Packet
descriptions here specify delivery; they do not accept a rule change or resolve an inquiry.
The delivery descriptions below preserve the accepted implementation target; package states and the Outcome record adopted scope and settled no-adoption choices. The operator accepted RC01–RC12 as written on
2026-10-20; the coordinator owns that record and the required decision/design routing.
I01/I02 are settled as complete-payload/fresh-check design and no persistent adoption;
I03 retains serial production under the current one-core owner. The coordinator records
these bounded choices; completed selected assembled qualification is recorded in48h.

The target serves cold package admission and repeated edit/re-solve without making every
declaration enumerate every visible name. A value-only change should retain mathematics
whose actual inputs did not change. A new import, ambiguous name or physical declaration
must still invalidate every consumer whose meaning changes. These are dependencies of the
operation, including absence and ordered membership, rather than an environment-proof or
provenance-attestation project.

## Current foundations and intended boundary

The baseline assessed by the audit is commit `1ee81798a9b884f3054b685ee9629e2c8d124498`.
Plan authoring also inspected the current checkout's `pse-modeling::check`, expression
phase inputs and `pse-compiler::workspace::modeling`. This is a focused foundation assessment
under Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5, reusing the audit rather
than restarting its review. The enduring owners are [blueprint §14.3 and §14.4](../authoritative_design/sections/mathematics-and-compilation.md#section-14-4)
and the source/model contracts in [§6.1](../authoritative_design/sections/schema-and-relations.md#section-6-1).

At the assessed baseline, `CheckedPackage::named_types_with` walked lexical ancestors, collects visible
prefixes and names and repeatedly resolves candidates against current types. Its visibility
and shadowing behavior is useful authority, but building a broad `BTreeMap<String, Type>`
for a small lookup is the physical work A01 removes. This is narrower than claiming that
all scope indexes or incrementality are absent. Existing ordered source-selection indexes
and checked package child/name maps should be used where they already supply the required
access; add only missing lexical paths.

The compiler already separates source, signatures, expression environments and individual
expression inputs. `Namespace` and expression metadata strip nonlexical content, and
`Environment::reuse_metadata` preserves equal owners. Granular tests already exercise consumed
body edits, absent-name/shadow/import changes, cancelled staging and atomic live-head refusal.
A03 extends those mechanisms; replacing Salsa or doubling cache entry limits would bypass
the actual broad-key/equality and representation problem. Exact equality is the contract:
structural sharing and pointer equality can accelerate it, but an unqualified digest cannot
silently stand in for it. Stable independently versioned inputs may receive high durability;
durability supplies neither a dependency partition nor persistence.

At the assessed baseline, `check/phases/expressions.rs::Environment::new` serialized typed declarations,
masks expression fields by role/position and deserializes them. Failed normalization keeps
the original row, deliberately preserving conservative dependencies and signed zero.
A04 now removes that internal crossing while retaining parser-owned occurrence meaning. JSON
at external import, semantic witness and transport boundaries has different consumers and
is outside that deletion.

The resulting ownership remains ordinary source and compiler operations. An immutable
lexical base resolves a requested spelling against a phase's current type/entity/extent
and reader overlays. A dependency-aware checked product supplies current compilation and
specialization; its receiving shell binds the current source/physical/invocation owners.
Mutable candidate assembly and native evaluator construction do not enter a shared immutable
cache. New declarations extend authored definitions; a mechanism change remains inside
lookup/checking rather than spreading lexical policy into workflows or native adapters.

## Delivery sequence

Package prerequisites identify usable behavior, not merely an agreed interface. A01 and
A04 can be implemented independently with coordinated edits to expression inputs. A02
makes current-phase ownership explicit before A03/A06 share more work. A05's inquiries can
start alongside that work, but persistent adoption waits for complete dependency and receiving
contracts. [48b](48b-registry-and-receiving-validation.md) owns receiving validation improvements;
A05 must consume its current bindings rather than export cached admission permission.
[48c](48c-mathematical-products-and-compiled-execution.md) consumes admitted source and bodies;
none of its numerical-state assessment can be replaced by a checked-source cache hit.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A01 — lazy indexed lookup | Existing lexical/physical visibility semantics and exact source inventories | Resolve actual requested names through indexed scope/child/import paths, retain complete enumeration and independent precedence cases; delete eager lookup-only environment construction after all affected callers move. | done: implemented; focused tested; selected assembled acceptance recorded in48h |
| A02 — mutable phase overlays and memo ownership | A01 resolution view usable by checking and admission evaluation | Share immutable lexical bases with current type/entity/extent/reader overlays; maintain the package-owned kind/extent projection at inventory mutation, prove same-owner evolving-phase lookups remain fresh, reject owner-only memo keys and retire redundant phase environment rebuilding. | done: implemented; inventory projection correction focused tested; selected assembled acceptance recorded in48h |
| A03 — granular sharing and bounded retention | A02 current phase ownership; existing granular query roots | Narrow consumed structural dependencies and exact shared products, retain absence/order/global obligations and byte accounting; compare incremental results with fresh checking and remove replaced broad inputs. | done: implemented; focused tested; selected assembled acceptance recorded in48h |
| A04 — typed declaration masks | Parser-owned role/ordinal mapping and generated declaration source | Replace the JSON masking round trip with a typed/generated projection preserving signed zero and unknown-field conservatism; verify changed versus masked fields independently and remove the internal JSON helper. | done: implemented; focused tested; selected assembled acceptance recorded in48h |
| A05 — checked artifacts | I01 complete artifact/import/body/data contract, I02 storage choice, accepted RC02 routed into its owner and current receiving reconstruction | Conditionally deliver a disposable complete checked-artifact accelerator with atomic publication, bounded retention and fresh-check fallback; otherwise retain fresh-process checking and deliver the inquiries' explicit no-adoption result. | done: settled; fresh checking retained, no persistence adoption; selected assembled acceptance recorded in48h |
| A06 — read-only parallel phases | A02 immutable inputs, I03 ownership/cancellation outcome and dependent source-order stages complete | Conditionally execute genuinely independent read-only work under existing capacity, with deterministic failures and joined cancellation; migrate selected consumers and retire the replaced serial scheduling where qualified. | done: settled; production serial retained; selected assembled acceptance recorded in48h |

### A01 — resolve the names the operation reads

Given a checked source revision, lexical owner and requested spelling, lookup returns the
same declaration/type or absence/ambiguity required by current visibility and shadowing.
Qualified spellings and physical names remain available for diagnostics and public lookup.
Kind/extent enumeration still returns complete ordered populations for consumers that need
them; it is not implemented as repeated lazy lookups or omitted because most expressions
are small. Inserting a previously absent name, deleting a shadow or changing an import
alias updates the relevant dependency even if the previous successful target still exists.

Migrate the actual checking, type-context and admission-time evaluation callers that build
whole named-type maps only to answer lookups. Retain necessary complete physical and global
checks. The existing exhaustive enumeration oracle can verify enumeration compatibility,
but it uses production resolution and cannot independently establish precedence. Add small
explicit expected outcomes for nested shadowing, relative/qualified names, duplicate/ambiguous
imports, absent names and physical visibility. Short package-size controls can inspect lookup
visits without a promised universal exponent. Once targeted cases pass and callers move,
delete replaced eager construction paths and their mechanism-specific fixtures/tests.

### A02 — share a base without freezing an evolving candidate

Admission receives the immutable lexical base plus borrowed current phase state. Types,
entity membership, finite extents, reader refinements and finite functions can evolve while
the declaration owner remains the same. Memoization must either consume the relevant state
versions or remain local to an immutable phase boundary; candidate lifetime alone is not a
sound key. Failures must not install a half-checked overlay or publish candidate state.

Migrate signature/type/expression checking and admission evaluation together so no caller
quietly reconstructs its own lexical policy or holds an obsolete type map. Exercise a lookup
before and after adding a type, changing a finite extent and refining a reader in one
candidate; also exercise a refusal followed by repair. Compare actual resolved outcomes
with explicit expectations and a fresh candidate. Delete redundant base rebuilding once
its final consumers use the current resolution view; do not introduce owner-only caches.
Immutable
specialization bases remain reusable with occurrence-local bindings, as §14.4 requires.

### A03 — let actual dependencies control shared products

Expose the missing consumed name/type/member inputs to existing granular queries. Equal
results can backdate and share storage; changed bodies, defaults, table contents and physical
interpretations must reach every consumer that reads them. Negative lookup and ordered
candidate-set dependencies are first-class inputs. A whole-population rule remains broad
until its sound partition is known. An unchanged signature does not establish an unchanged
implementation or data dependency.

Retained roots cover actual reachable shared allocation without an ancestral ownership chain
or double charging shared products. After representation reduction, select bounded useful
retention in bytes through existing compiler/service owners; pressure may evict acceleration,
not corrupt detached products or create a new scientific refusal for otherwise valid work.
Existing `granular_tests.rs` fresh-check comparisons are a foundation, especially
`consumed_body_edit_executes_one_source_and_cuts_off_equal_sibling_phases`,
`namespace_add_delete_shadow_import_and_absence_match_fresh_complete_admission`,
`expression_inputs_retain_signature_type_guard_and_missing_name_changes` and pressure rotation.
Extend them for the newly narrowed inputs and inspect retained owners after replacement.
Delete broad duplicate keys/products only when every corresponding consumer has moved.

### A04 — mask typed expression occurrences directly

The operation receives the typed declaration and parser-owned expression occurrences and
produces the same nonexpression comparison product now obtained by masking JSON. A role and
ordinal select precisely the expression-bearing field; unrelated source and semantic fields
retain their consumed meaning. Missing/unrecognized mappings conservatively keep the full
dependency rather than discarding a field. Generated support derives from existing declarations
and parser metadata; it does not hand-maintain a second row shape.

Verify scalar, nested and repeated occurrences, signed positive/negative zero, unknown fields,
and pairs that differ only in a masked expression versus an unmasked metadata field. Explicit
expected typed products and retained dependency behavior are the oracle, rather than comparing
only two implementations of the same masking path. Regenerate through `just codegen` if a
generator changes. Remove `serde_json` use and helpers only for this internal mask after its
consumers migrate; preserve externally consumed JSON contracts and dependencies still used
elsewhere.

### A05 — decide and, conditionally, adopt complete checked artifacts

**I01 asks what a reusable checked artifact must contain.** Compare an explicit type-interface
product with a complete checked-package product and its recorded consumed implementation/data
dependencies. Trace a short imported function-body change, unchanged-signature default/table
change, physical interpretation change and negative/member lookup edit through the actual
consumer. Evidence must distinguish required global completeness from selected dependencies,
source attribution from mathematical equality and immutable content from receiving eligibility.
Stop when the complete supported payload and dependency comparison are specified and these
revealing edits have unambiguous hit/miss/readmit outcomes. If any dependency cannot be captured
soundly, choose conservative fresh checking for that scope. Reopen when a new imported
declaration/data kind or interpretation introduces a consumed premise outside the payload.

**I02 asks whether saving the whole Salsa database is preferable to smaller artifacts.**
Compare a disposable immutable checked artifact with whole-database persistence, including
transitive serializable types, interned identity/collection, accumulated diagnostics, restoration,
retention and current receiving construction. Salsa 0.28.4 persistence exists; it is not
automatically suitable for this compiler. Use the pinned skill's persistence contracts and
current compiler types rather than adding an alternate persistence subsystem by analogy.
Stop after a bounded representative round-trip design/probe establishes the materially
different obligations and cold-path usefulness. Adopt one justified granularity or adopt
neither; a smaller artifact is not a prerequisite for eventually choosing whole-database
persistence. Reopen for a concrete unsatisfied restart workload or a library change affecting
the deciding restoration constraints. The coordinator records I01/I02 decisions and evidence.

The accepted RC02 boundary permits this investigation; its effective contract and a sound
selected outcome precede persistent production adoption. A local cache is derived, disposable
acceleration. Complete consumed source/import,
body/default/data, physical interpretation and membership dependencies govern reuse; an ABI
or cache-format compatibility key is not an environment-proof service. Atomic publication
prevents partial products becoming hits. Corrupt, absent, incompatible or stale artifacts
fall back to fresh checking with the current clock and receiving owners; interrupted writers
leave no accepted artifact. Do not persist mutable compiler lanes, loaded provider/native
handles or prior receiving permission. Test these failures plus independent-process restore
and changed-premise misses before switching callers. Remove the replaced artifact-building
or load path at cutover; fresh semantic checking remains the authoritative fallback, not a
compatibility API to retire.

### A06 — bound parallelism to immutable phase inputs

**I03 asks which checking/import operations are genuinely independent.** Compare existing
serial ordering with cloned-handle read-only execution over immutable phase inputs, retaining
ordered signature prerequisites and one mutable candidate publisher. Salsa's `rayon` feature
is not the enabling contract. Inspect source-order reads, diagnostics, thread safety, query
cancellation and candidate rollback for the specific phase proposed. A bounded fan-out with
one failing and one cancelled child must end with all readers drained before mutation or
publication. Stop when one useful phase has an explicit dependency/ownership boundary and
deterministic outcome; choose serial execution if no qualifying phase warrants the added
coordination. Reopen for a new independent phase or evidence that qualifying serial work
still dominates after A01–A04. Adoption and no-adoption both return to the coordinator.

Any selected parallel phase uses existing population/CPU/memory admission and original clocks.
It retains source-order diagnostics and atomic candidate publication, with private temporary
assembly and complete drain on failure/cancellation. Migrate actual phase consumers as one
coherent change; do not leave a second production scheduler for the same phase. Targeted
tests cover source-order dependence, independent results, failure before publication and
cancelled-reader retirement. Parallel success alone does not qualify all compiler concurrency.

## Verification and handoff

**Proposed verification:** packages use `just check-package pse-modeling` and/or
`just check-package pse-compiler` for touched responsibilities and targeted
`just unit-package <pkg> <filter>` runs for the cases above. Existing granular tests give
fresh semantic comparisons; independently authored lookup outcomes cover resolver changes.
Actual Arrow-consuming closures retain explicit force-validation through repository routes.
Generation belongs to the functional change. Formatting/static checks and assembled integration,
Python journeys and representative cold/warm measurements occur once at the coordinator's
scope end, selected for the implemented scope. No universal near-linear bound, cache hit
ratio or latency SLA is introduced.

The original authoring phase ran no tests, builds, services or environment qualification.
The implementation checkpoint below records the current focused verification boundary.
A01–A04 implementation and the settled A05/A06 conditional choices do not resolve all of
AF02 or qualify assembled numerical behavior. Enduring contract changes follow the decision/design
route and transfer to their existing owners when the coordinator closes the series.

## Execution checkpoint — 2026-10-20

A01–A04 are implemented in the assigned source/compiler scope. Checking and admission-time
evaluation now use indexed lazy lexical views, borrowing the current type population and
extending occurrence-local bindings instead of building a complete visible-name map.
The immutable specialization cache retains only local formal/scope bindings; admission
readers do not use an owner-only cache. Kind/entity extent enumeration remains complete.
The subsequent A02 completeness pass found that `with_extents` still rebuilt the whole
kind-to-extent map for each view. `KindInventory` now owns current kind records and their
derived extent types; key insertion updates both, mutable access is limited to record
values, and no mutable map dereference can bypass the projection. Extent views now only
select a mode and borrow those stored types. Selected packages rebuild only their selected
kind projection, and retained-byte accounting includes its map entries and boxed types.
The old production eager lookup builder and JSON mask helper are removed. One exhaustive
lookup oracle remains in tests alongside independently expected precedence and physical/
quoted-name cases; redundant prefix-builder test helpers have been retired.

Compiler signature inputs now retain the exact consumed resolved-name projection, rather
than a persistent complete preceding type root per declaration. Namespace equality shares
exact equal allocations at phase boundaries; reduced-law signatures alone read the full
coordinate namespace. Body/default/data and expression dependencies retain their existing
incremental owners. Shared physical registries, prerequisite/scope and lexical-index roots
are charged once per live input census; owned maps retain conservative per-owner bounds.
Generated typed role/ordinal dispatch masks only the exact matching string coordinate and
keeps unknown/ambiguous mappings conservative, without rewriting scientific float cells.

Focused verification at this implementation baseline: `just check-package pse-modeling`
and `just check-package pse-compiler` passed their all-target compile scopes; the compiler
check reported no warnings. `just unit-package pse-modeling 'all()'` passed 248 library tests
with 0 failures/0 skipped, and `just unit-package pse-compiler 'test(workspace::modeling::granular_tests)'` passed 17 selected tests with 0 failures and
259 filtered. These were the pinned repository routes, with actual closure validation
selected by those routes. They exercise typed masking, fresh/current lookup semantics,
consumed and unrelated earlier type edits, fresh-versus-incremental agreement, absent names,
shadow/import changes, candidate rollback and bounded retention/history. The subsequent oracle retirement, current entity-extent/read-mode evolution and bounded A05
artifact controls passed the expanded 249-test modeling library scope, with zero failures
and no compiler warnings. A subsequent focused `just unit-package pse-modeling
'test(indexed_import_lookup_preserves_nested_alias_shadowing_and_refuses_conflicts)'`
passed its one added import/alias control with zero failures and no warnings. Its first
attempt used an infallible test parser for a deliberately duplicate named identity; the
control now asserts that refusal at the parser owner before checking alias conflicts.
The later A02 inventory correction passed `just check-package pse-modeling` across all
targets with zero errors and warnings, followed by `just unit-package pse-modeling
'test(check::named_types_tests)'`: 12 selected tests passed, zero failures, 241 skipped and
zero compiler warnings. Three new controls verify shared borrowed projection values,
current schema replacement/removal/reinsertion with exact retained storage, and selected
kind closure boundaries; the independent lexical oracle and precedence controls remain.
This focused correction does not enlarge the earlier 249-test modeling or 19-test compiler
qualification scopes. These focused receipts do not establish scope-end hygiene, assembled
journeys, numerical qualification or whole-workload performance; 48h owns their current
selection and acceptance.

A05 is settled as a bounded no-adoption outcome: complete-artifact receiving dependencies
are specified below, the existing portable function record is insufficient, and production
retains fresh-process checking. A06 is settled as no parallel adoption under the runtime's
one-core source-compiler owner, with the tested independent read boundary and concrete
revisit trigger below. The coordinator accepted I01/I02/I03 on 2026-10-20 and owns aggregate
acceptance; these are not remaining implementation promises. Scope-end assembled
qualification remains owned by 48h.

### A05 — bounded complete-artifact comparison

**Tested:** `check::artifact_contract_tests` within the expanded 249-test modeling library
run on 2026-10-20 round-trips the existing `portable::FunctionRecord`. The fixture is an
imported unit/function consuming a constructor default, an admitted complete-over table
and a constant interpreted through a custom physical unit. Changing the function body
preserves its signature but changes the record. Changing the consumed default, table value
or custom-unit scale leaves that function record equal while changing its actual specialized
equations/functions/symbols. An absent caller binding is still refused until supplied; the
imported function record remains equal after repair. Restoring the raw deserialized record
outside the qualified reconstruction recipe remains refused. Thus the existing function
record is useful subpayload and is not a complete checked-artifact/reuse key.

**Proposed complete record, not a new production API:** capture the complete authored
source/import/declaration population and order, package/document ownership and actual
physical scope, immutable physical declarations/prerequisites, and exact data-document
identities/content/decoded typed rows. The semantic payload covers checked types/functions
(including `FunctionRecord` admissions, guards, effects and physical roles), effective
members/interfaces/temporal contracts, kinds/entities/identifier scope/constants/tables,
provenance/oracles/test-only taint, source occurrences and scientific selection closures.
Derivable name/child/selection/physical-type indexes can be rebuilt from those complete
owners. It must retain negative lookup and ordered candidate membership, not only successful
selected targets or type signatures. Exact consumed input comparison remains authoritative;
current source/span attribution and resource `DataOwner` lifetimes are rebound by their
current receiving owners. Native/provider handles, mutable compiler lanes and prior receiving
permission are not payload. `pse-modeling` owns capture/reconstruction; the compiler's current
physical/context/resource shell must qualify the complete record before existing strict
record restoration can run. Unknown/incomplete payloads fall back to fresh source checking.

**Interface-checked comparison:** Salsa 0.28.4 supports persistence; the pinned skill's
B025–B027/B034–B036 records demonstrate memo/interned restoration, transitive persistability
requirements and renewed execution after intern collection. The current workspace feature
selection omits `persistence`; compiler ingredients are not marked `persist` and their
`CheckedPackage`, type and document inventories do not expose a complete serializable wire.
`Arc<dyn DataOwner>` is allocation lifetime, not serializable scientific content. Persisting
Salsa requires the entire selected transitive ingredient/key/value closure plus fresh
cancellation/retention/receiving construction; accumulated diagnostics would require their
own reconstruction if introduced. A whole-database design can supply this without requiring
a separate smaller-artifact API, but it does not remove these obligations.

The coordinator accepted no persistent production adoption on 2026-10-20: retain
fresh-process checking and the implemented in-process sharing. No complete checked-package round trip,
receiving restore, atomic storage publication or cold restart benefit has been established.
This is the settled conditional no-adoption outcome; it creates no outstanding artifact
implementation promise. Reopen for a concrete restart workload whose
source-checking cost remains material after A01–A04, or a new source/data/physical kind that
changes the complete supported payload and capture/restore choice.

### A06 — bounded immutable-read boundary

The useful independent subset is parsing after serial `Source` staging and before global
inventory/signatures. All preflight accounting and input setters stay on the sole mutable
candidate owner; cloned Salsa handles execute only `parsed` over staged immutable inputs.
Every handle must be joined/dropped before inventory or repair setters. Results/errors are
collected in the existing canonical serial traversal order, never completion order; local
cancellation cannot be mistaken for a successful package. Signature prefixes, inherited
members and `finish` retain their dependency order. The current `Driver`'s `RefCell`/`Cell`
state is not shared. No `rayon` feature or separate resource configuration is needed to
identify this boundary.

The actual runtime owner `math/jobs.rs::compiler_job_retained` admits and executes one
entry/core (`admit_entry(1, ...)`, `job_retained_on_entry(1, ...)`). Optimizer `compiler_cores`
is a different control. Adding read threads without that owner's grant would spend
ungranted capacity. **Tested:** `workspace::modeling::granular::parallel_probe_tests` passed alongside the
17 granular controls (19 selected compiler tests, zero failures, 262 filtered) on 2026-10-20.
Three cloned read handles completed success, malformed-source error and local cancellation;
all readers were drained before repair setters and the cancelled query recomputed normally.
**Measured:** one synthetic cold development-profile, one-worker run with 64 scalar functions
and 68 declaration rows took 636 microseconds for parsing plus `Source` staging and 9,836
microseconds for complete candidate admission. This is one revealing fixture, with no timing
threshold, fan-out speed result or wider workload claim.
The coordinator accepted retaining the production serial path on 2026-10-20. Reopen only with an actual
multi-core source-read grant and a parse-dominant workload showing a useful bounded subset;
any adoption must retain the same accounting, original clock and complete reader drain.

### H01 scoped completeness handoff

The linked recommendation coverage is accounted for within this package: P01 lookup,
kind/extent indexes and the independent eager oracle use the implemented lazy views;
P01 mutable-candidate ownership uses current borrowed phase populations. P02 tracked-input,
exact-equality and retention recommendations use consumed signature projections and live-root
accounting while retaining the existing Salsa query/durability owners. The expanded modeling
and compiler controls above exercise the actual checking and admission consumers, including
imports, absence, shadowing and mutation; they do not replace complete namespace obligations.
P18 typed masking is consumed by expression phase comparison through generated string-leaf
dispatch, with unknown paths and scientific numerical cells preserved.

The final scoped pass corrected the remaining per-view kind-map reconstruction through
A02's current inventory projection. After that retained-footprint change and normal
generation, six existing compiler controls passed in the canonical native runtime/compiler
selection: consumed body edits, namespace add/delete/shadow/import/absence, pressure rotation,
current input roots, unrelated preceding type sharing and consumed preceding type refusal/
repair. They compare against fresh checking and preserve detached products and exact sharing;
the selection's separately owned runtime failures do not turn these six passes into an
enclosing qualification. The earlier 249-test modeling and 19-test compiler results retain
their earlier baseline. No additional source production gap or superseded lookup/mask path
was found in this bounded A01–A06 consumer pass; 48h still owns assembled qualification.

The remaining P02 persistence and parallel-read rows have the settled I01–I03 outcomes above:
a portable function record is not a complete receiving package, and the actual compiler job
owns one admitted core. Neither a signature-only cache, whole-database persistence nor an
unused parallel feature was introduced. These are explicit no-adoption boundaries with
concrete reopening conditions, not missing implementation promises. P18 cross-package hashing
and registry work remains owned by48b. This handoff accounts for A's linked producer/consumer
scope;48h retains assembled qualification and the coordinator retains aggregate coverage state.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested, completed adopted scope (2026-10-20):** the package table records
all delivery/inquiry outcomes. [Plan48h](48h-qualification-and-design-publication.md#final-qualification-accounting)
records completed selected assembled acceptance and scope-end checks. The individual
observations below retain their original conditions and partial-run boundaries; they do
not assert a universal speedup or extend unsupported capabilities. There is no outstanding
authorized package work in this companion.

**Implemented:** A01–A04 source lookup, current phase views, narrowed signature products,
shared-root accounting and generated typed expression masking described above, including
the A02 current kind-inventory projection replacing per-view extent-map reconstruction. A05/I01–I02
and A06/I03 are settled conditional no-adoption outcomes with retained fresh checking and
serial production, not deferred implementation promises.
**Tested:** the expanded 249-test modeling and 19-test compiler scopes above, against a zero
failure baseline on 2026-10-20; both final runs emitted no compiler warnings. This does not establish assembled numerical qualification or a measured whole-workload
performance claim. The later A02 projection correction separately passed the all-target
modeling compile check with zero errors/warnings and all 12 selected named-type controls
with zero failures/warnings; it does not claim a repeated full modeling/compiler run.
After normal generation, the six affected compiler freshness, absence, exact-sharing and
retention controls also passed in the canonical runtime/compiler selection, as recorded in
the H01 handoff. Its separately owned runtime failures remain failures against zero.
48h owns the remaining assembled qualification.

### A mistake made and corrected

Sparse signature projection exposed the source-requirement view's omitted inventory-balance
type. The full modeling library tests refused `Time` in inventory/temporal cases. The source
owner now includes inventory type/axis/index/flux/tolerance/transfer dependencies, and the
remaining temporal constructor consumer was migrated from the old complete cached type map
to the current lazy view. The subsequent 248-test run had zero failures.

The first A02 implementation removed eager lexical environments but left whole-kind extent
map reconstruction in each extent view. The completeness pass exposed that remaining work
amplification; the package-owned current projection and three targeted mutation, sharing
and selection controls now replace it. Compilation also caught the signature-resolution
constructor still initializing the former map field; both current constructors now use the
mode flag, and the final all-target check and focused controls passed without warnings.

### Deviations from the plan, deliberate

Full namespace reads remain for reduced-law coordinate admission and global population
obligations; they are not replaced by signatures-only reuse. A06 will preserve the existing
canonical declaration-id result/error ordering if parallel execution is adopted. Its
original source-order wording does not authorize changing diagnostic selection during a
scheduling optimization. A05 no persistence and A06 no parallel production adoption are deliberate settled choices
under their stated complete-payload/resource limits and concrete reopen triggers.
