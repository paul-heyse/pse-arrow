---
title: Source checking and incremental preparation
status: draft
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
All target changes below are **Proposed**. The operator accepted RC01–RC12 as written on
2026-10-20; the coordinator owns that record and the required decision/design routing.
A05 adoption still depends on the bounded I01/I02 outcomes and an effective RC02 contract.

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

`CheckedPackage::named_types_with` currently walks lexical ancestors, collects visible
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

`check/phases/expressions.rs::Environment::new` currently serializes typed declarations,
masks expression fields by role/position and deserializes them. Failed normalization keeps
the original row, deliberately preserving conservative dependencies and signed zero.
A04 removes that internal crossing while retaining parser-owned occurrence meaning. JSON
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
| A01 — lazy indexed lookup | Existing lexical/physical visibility semantics and exact source inventories | Resolve actual requested names through indexed scope/child/import paths, retain complete enumeration and independent precedence cases; delete eager lookup-only environment construction after all affected callers move. | open |
| A02 — mutable phase overlays and memo ownership | A01 resolution view usable by checking and admission evaluation | Share immutable lexical bases with current type/entity/extent/reader overlays; prove same-owner evolving-phase lookups remain fresh, reject owner-only memo keys and retire redundant phase environment rebuilding. | open |
| A03 — granular sharing and bounded retention | A02 current phase ownership; existing granular query roots | Narrow consumed structural dependencies and exact shared products, retain absence/order/global obligations and byte accounting; compare incremental results with fresh checking and remove replaced broad inputs. | open |
| A04 — typed declaration masks | Parser-owned role/ordinal mapping and generated declaration source | Replace the JSON masking round trip with a typed/generated projection preserving signed zero and unknown-field conservatism; verify changed versus masked fields independently and remove the internal JSON helper. | open |
| A05 — checked artifacts | I01 complete artifact/import/body/data contract, I02 storage choice, accepted RC02 routed into its owner and current receiving reconstruction | Conditionally deliver a disposable complete checked-artifact accelerator with atomic publication, bounded retention and fresh-check fallback; otherwise retain fresh-process checking and deliver the inquiries' explicit no-adoption result. | open |
| A06 — read-only parallel phases | A02 immutable inputs, I03 ownership/cancellation outcome and dependent source-order stages complete | Conditionally execute genuinely independent read-only work under existing capacity, with deterministic failures and joined cancellation; migrate selected consumers and retire the replaced serial scheduling where qualified. | open |

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

This authoring task ran no tests, builds, services or environment qualification. A future
implementation checkpoint reports the command, mode, scope and failures against zero, plus
what was removed. A completed A01–A04 does not resolve all of AF02, establish A05 eligibility
or qualify assembled numerical behavior. Enduring contract changes follow the decision/design
route and transfer to their existing owners when the coordinator closes the series.

## Outcome (recorded after implementation)

### What was built

**Proposed; not implemented.** This document supplies the execution design and conditional
inquiries only. No source checking, caching, persistence or scheduling behavior changed.
Implementation outcomes, a mistake made and corrected, deliberate deviations and actual
verification will be recorded when the work is performed; current inquiry and rule outcomes
remain solely in the coordinator.

### A mistake made and corrected

**Proposed; not implemented.** No implementation mistake or correction is recorded yet.
Record the actual lesson when the work is performed.

### Deviations from the plan, deliberate

**Proposed; not implemented.** No implementation deviation is recorded yet. Record deliberate
changes and their consequences when the work is performed.
