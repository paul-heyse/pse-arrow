---
title: Native cache and schema pivot implementation acceptance
date: 2026-09-17
status: done
evidence: Tested
---

# Native cache and schema pivot implementation acceptance

## 1. Decision and scope

**Decision: Accept for Plan 09's stated local/current-function scope — 2026-09-18.** This is the implementation
acceptance review of [Plan 09](../../plans/09-native-caching-and-pivot-completion.md),
including its carried Plan 08 E00–E10 obligations. It does not certify planned future
simulator functions, numerical IDAES parity, remote destructive coordination or an
unmeasured performance improvement.

**Method and coverage:** inspect actual registry, provider, session, rule, dependency,
Delta, resource and Python routes; use the pinned DataFusion/Arrow and Delta skills,
exact upstream source and the reproducible Delta override; associate each assertion
with current target tests and measurements. The [implementation inventory](../../plans/09-execution-inventory.md)
records concrete ownership, deletion and oracle paths. This review is reconciled to
the final execution receipts. The method attacks forged declarations, hidden null
children, conflicting keys, false support, stale/unknown cache identity, interrupted
commits, missing CDF history, maintenance races and escaped buffer lifetimes.
Evidence is aggregate across recorded source boundaries; it is not a single untouched
final full-suite run. Universal allocator coverage and numerical IDAES parity were
not established. Isolated units and compiled APIs alone are not terminal evidence.
No Context7 was used for DataFusion, Arrow or Delta. No predecessor implementation is
retained as an oracle. Scientific assertions, exact native recomputation and independent
finite-graph/row-count assertions are retained.

## 2. Authority and lifecycle map

| Meaning | Authority | Derived owner / lifecycle |
|---|---|---|
| Relation fields, semantic alternatives, keys, consumption and publication policy | `pse-schema` registry declarations | Generated Rust/Python/reflection and admitted Arrow fields; direct regeneration |
| Actual engine semantics | Captured functions, rules, planners, policies and conservative semantic settings | Owned implementation generation; unrelated opaque generations refuse reuse |
| Published state | Native Delta member logs and complete publication selection | Native snapshots, selected providers and typed receipts; current admission precedes consumption |
| Cached file metadata | Actual store/root binding and native validity checks | Native DefaultCache inside a reserved capacity envelope; no path rewriting of storage IO |
| Cached snapshots and decoded values | Exact native selection, schema, load class and semantic identity | Reservation-owned values; eviction drops the cache reference, not reader ownership |
| Fixed-point work | One admitted invocation and its prepared native plans | Immutable input epochs, drained execution, native reset; failure ends reuse |
| Inspection | Actual owners and observable native counters | Read-only typed projections; unavailable values remain NULL |

## 3. Semantic contracts and invariants

The target enforces complete tagged values, exact nested fields, coherent math and
numerical dimensions, typed source witnesses and explicit member selection. Whole
consumption remains conservative. A narrower argument must make omitted fields
inaccessible; native expressions must also retain keys, filter fields, membership,
multiplicity and absence obligations.

Cache identity separates resource ceilings from semantic settings without weakening
current admission. Unknown settings remain semantic. Metadata snapshots cannot satisfy
query/file-list requirements. A cache entry never supplies missing CDF history or
proves an unknown implementation identity. Projected reuse compares bag contents,
including NULLs and duplicates, under current policy.

## 4. Derivation and execution design

Native providers, expression plans, invariant plans and schema policies form the
common route. Reusable strata prepare candidates, deltas, support, probes and merge
plans once, then advance stable round inputs. Useful canonicalization/graph algorithms,
source parsing, numerical callbacks and FFI remain explicit native extension or boundary
work, without a second model authority.

Shared native caches are owned by the deployment runtime. One bounded admission gate
covers snapshot and resident population. Namespaces, actual load requirements, fresh
maintenance fences and read leases precede reuse. CDF uses the injected namespaced
metadata cache. Missing or unusable acceleration takes the normal exact native route;
an unavailable required semantic proof refuses reuse.

## 5. Representative journeys

### Ordinary extension

Registry declarations generate an algorithm's projected argument. The compiler's
`passes::argument::tests` verifies that omitted fields and the full owner cannot be
recovered. Native expression/provider extension fixtures exercise shared admission,
effects, persistence and inspection; their final results belong to the campaign.

### Meaningful change

`native_cache_lifecycle` performs value-only and key changes. It compares qualified
projected reuse with fresh native recomputation and an independent key oracle. Key
changes must invalidate; an unrelated opaque implementation generation must refuse.

### Boundary or alternate representation

Rust-produced current engineering publications are opened in a separate Python
process. Raw Arrow and registered extension consumers retain exported buffers through
handle close. Native solver tests run with the pinned solver image, while unlinked
capability refusal remains exercised in the ordinary workspace.

### Interruption or failure

Object-store fault injection covers actual member/root publication and post-commit
checkpoint/checksum effects. Live-flight cancellation cannot leave a durable negative
entry. A child process performs actual maintenance while idle parent caches remain;
the parent must reacquire a lease and validate the native fence before reuse.

## 6. Acceptance gates

Each gate is assessed independently against the exact Rust coverage ledger,
complete engineering receipts, final Python checks and cache measurements in §9.
All verdicts apply only to the scope declared in §1, with the limits in §10.

| Gate | Assessment | Mechanism, refusal and evidence |
|---|---|---|
| G1 Authority | **Pass** | Registry declarations generate exact fields; native Delta commits select durable state. Disposable caches cannot mint authority. Source archive 25 verifies legacy deletions; generated equality and declaration-impersonation tests pass. |
| G2 Semantic fidelity | **Pass** | Q01–Q06/Q08/Q11–Q12 preserve nested values, provenance, math, units and ownership. All 390 kernel outputs survive publication/reopen; four complete engineering publications and final independent Python readers pass. |
| G3 Validity | **Pass** | Common admission rejects forged fields, invalid variants/references, duplicate keys, unavailable native constraints and malformed visible values. Final 268 schema/rule/compiler and 253 provider/source checks pass; solver residual/derivative checks retain independent assertions. |
| G4 Hidden behavior | **Pass** | Actual caller policies and native effect contracts precede execution. Cache planning and inspection remain non-mutating; deferred DDL/DML and EXPLAIN sentinels are qualified in the final catalog selection. |
| G5 Consistency and recovery | **Pass** | Actual member/root receipts, native transaction witnesses, exact parent versions, reader leases and fresh maintenance fences govern recovery. Failure injection and cross-process local maintenance pass; all four publications reopen independently. Native acceleration failure never implies data-write replay. |
| G6 Transformation and reuse | **Pass** | Original semantic dependencies survive optimization; enforced column projection and exact bag comparison retain NULLs/duplicates/absence. Unknown generations refuse reuse. Prepared round/reset and clean-recomputation controls pass; fixed-point preparation counts remain constant in the measured matrix. |
| G7 Truthful capabilities | **Pass within the declared scope** | Feature matrix, provider/codec refusal, exact Delta source overlay and read-only native counters are qualified. Opaque allocator/replay counters remain explicitly unknown. Memory reservations/envelopes are distinguished from RSS and total heap; remote destructive coordination is not admitted. |

## 7. Principle findings

**Applicability:** authority, semantic structure, derivation, validity, boundaries,
reuse, effects, resources and evidence groups apply to this cross-cutting change.
Remote/distributed deployment and new process-simulator functions are outside the
requested implementation; no favorable verdict is inferred for them.

| Finding | Principles | Evidence / gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| F1 — Native leaf projection can misresolve an unqualified nested `value` | DM-24, DM-40, DM-60 | `artifact/consumption.rs`: explicit `__pse_cdf_input` alias and qualified nested field; isolated regression passes | A payload containing its own `value` field can fail exact projected comparison | **Implemented:** qualify the input; retain native optimization | `projected_changes_preserve_nulls_and_bag_multiplicity`, final CDF/reuse journey |
| F2 — Native opaque allocations/counters are not exact application pool accounting | DM-39, DM-59 | Native file-cache envelopes and replay staging bound admission; escaped upstream Arc counts and replay-action totals remain unavailable | Treating reservations as total heap or NULL counters as zero would overstate the resource proof | **Implemented:** expose separate capacities/retained/pinned/staging, NULL unknowns and measured RSS; conservative fanout | K02/K09 pressure, cancellation, owner lifetime and IO/reader measurements |
| F3 — Static completion does not establish the current process-model outcomes | DM-59, DM-60 | Initial static receipts lacked complete source/compiler/storage/solver/Python journeys | A declaration-correct pivot could still lose numerical, source or recovery behavior | **Tested:** complete the current-function oracles after the implementation barrier and repair failures in the target | Q01–Q14, K01–K11 and each gate independently |
| F4 — A transparent requirement barrier omitted the native order-preservation hook | DM-40, DM-43, DM-60 | `session/contract.rs`, `ContractExec::maintains_input_order`: the value stream is unchanged; sidecar validation does not contribute value rows | Physical optimization could discard an explicitly requested sort; physical prerequisite lookup then rejects reversed identities | **Implemented and Tested:** report the actual input-order contract, retain the algorithm's strict identity check | `just test-package pse-catalog --test native_output`: 4 passed, 0 failed, force-validation, including exact order; full physical fixture regeneration succeeds |
| F5 — Nested correlated closure query lost its outer alias at this pin | DM-24, DM-40 | `pse-schema/src/catalog/invariant_closure.rs`: stoichiometric eligibility now uses native joins and explicit presence/UNKNOWN handling | Correct reference packages could fail admission before physical compilation | **Implemented and Tested:** simplify the relational query while retaining NULL defaults and phase restrictions | `stoichiometry_phase_policy_preserves_defaults_restrictions_and_missing_members`: 1 passed, 0 failed, force-validation; physical generation succeeds |

Additional qualification findings:

| Finding | Principles | Evidence / gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|
| F6 — Shared native producers were repeatedly expanded during planning and diagnostics | DM-32, DM-39, DM-59 | `session/cache/logical.rs`, `session/preparation.rs`, `session/cache.rs`, `session/observation.rs`: actual producer identity now bounds preparation and diagnostic expansion; completed values use native streaming leaves | Diagnostic duplication exhausted the pool; repeated physical optimization obscured execution-cache benefits | **Implemented and Tested:** once-per-invocation native preparation, complete executable plans, explicit diagnostic references and successful current-epoch native leaf readers | 10 cache units pass, including stale epochs and ownership; representative P3 case falls from 349.989 to 101.674 seconds, CI/force-validation, same assertions and timeouts; final exact coverage and all four engineering publications pass; the two elapsed observations are not a controlled comparative benchmark |
| F7 — Enum references targeted repeated member rows | DM-01, DM-17, DM-60 | `pse-schema/src/builder.rs`: registry-derived `reference.schema_enum_types` and member parent FK | `balance_enum_id` was not unique in `schema_enums`, so correct source publications failed reference validation | **Implemented and Tested:** one registry-derived enum declaration row, distinct from its ordered member rows; law bindings target the declaration key | Registry FK target test passes; fresh complete inspection fixture publishes and reopens; conformance regeneration includes both relations |
| F8 — Native deferred CREATE reconstruction dropped constraints | DM-17, DM-24, DM-60 | DataFusion 55 `with_new_exprs(CreateMemoryTable)` resets declared constraints | A private table lost its primary key and admitted a duplicate insert | **Implemented:** preserve constraints only across an unchanged child field layout | Strengthened native DDL/default/generation regression passes; duplicates remain refused |
| F9 — Python's blocking-thread restriction deadlocked native Delta opening | DM-39, DM-60 | Python worker sample and the pinned kernel executor's explicit nested-blocking warning | A single open could wait indefinitely while completion delivery was queued behind its own blocked worker | **Implemented and Tested:** retain Tokio's native blocking capacity and control admitted work through the existing application limits | Continuation 13: extension rebuild, quality and all 72 Python unit/component tests pass, baseline zero; all four final engineering cold-process inspections pass |
| F10 — Repeated field restoration expanded shared graphs | DM-32, DM-39, DM-60 | Actual native extension owners, unchanged nodes and recursive projections were reconstructed repeatedly | Composition cost and retained plan memory grew independently of domain work | **Implemented:** exact-owner admission keys, native transformation flags, idempotent recursive transport and sibling query/diagnostic batching | 18 focused admission/rules/compiler checks pass; both heater and both mixer executions/publications now pass |
| F11 — Python bypassed deployment cache construction | DM-01, DM-39, DM-60 | `pse-py/src/inspection/runtime.rs` directly constructed a session factory instead of using the shared runtime factory | Resident reads missed the shared service and tiny-budget opening bypassed its controls | **Implemented and Tested:** use `SharedRuntime::session_factory`; actual store-binding ownership is included in resource inspection | Continuation 13: all 72 Python tests pass, including cache-retention/drop/budget assertions; 84 cache matrix records include explicit binding ownership |
| F12 — Physical planning expanded shared producers | DM-32, DM-39, DM-60 | `session/cache/physical.rs` plans each closed native producer once; public logical plans retain the complete graph | Native whole-tree discovery and optimization multiplied the same producer inputs | **Implemented and Tested:** separately planned lazy cache boundaries; actual producer reset eligibility and native state reset remain enforced | Campaign 12: both complete heater configurations pass with independent model assertions, 482.819/484.802 seconds, CI/force-validation; fixed-point matrix preserves constant preparation counts |
| F13 — Native tree diagnostics duplicated shared nodes and indentation | DM-32, DM-39, DM-59 | Heater receipt 46 exhausts the 32 GiB pool after 501.117 seconds; many retained plan observations are 128 MiB | Diagnostic formatting prevents an otherwise progressing model from completing | **Implemented and Tested:** public DataFusion `PgJsonVisitor` renders each actual node once with its output fields; ordered graph edges retain every input and subquery occurrence. No live execution owner is retained. Delete the cache-only diagnostic clone and duplicate rule-result text | Receipt 48: six passes including the heater; campaign 12 repeats the units and both heaters successfully. Per-node upstream rendering still has transient allocations outside the application's allocator hooks |
| F14 — Internal nested-field aliases shadowed an actual source field | DM-24, DM-43, DM-60 | Both element-molar conservation publications failed in native `push_down_leaf_projections`; `nested_values` repeatedly introduced an unqualified `value` beside the actual qualified source `value` | Correct quantity-bearing rows could not complete publication admission | **Implemented and Tested:** source-disjoint occurrence names, actual-column binding in quantity/reference/source-span consumers; retain NULL/container masks and duplicates | The new in-memory test first reproduces the native ambiguity, then passes with all 12 affected units; 32 storage/reuse tests pass. Both element-molar cases and the complete conservation selection pass in receipt 71 |
| F15 — Native casts visited bytes hidden beneath null storage parents | DM-24, DM-43, DM-60 | Continuation 16 and the element-molar cases failed when a hidden empty binary child was decoded as a fixed-size identity | Semantically null values could fail exact publication admission | **Implemented and Tested:** native struct-mask propagation and null-list/map compaction before the strict cast; internal inverse-comparison nullability never changes output declarations | Four focused exact-boundary tests pass, including struct/list/fixed-size-list/map hidden children and malformed visible-value refusal; all 253 final catalog/source checks and four complete engineering publications pass |

**F16 — Repeated derivation heads (DM-01, DM-17, DM-24, DM-60):** continuation 17
persisted 17,422 derivation rows with 17,183 identities. Repeated rule invocations
can contribute additional support for the same head; raw concatenation violates
the declared primary key. **Implemented and Tested:** native aggregate/flatten/
distinct/sort combines support by the complete head. Conflicting scalar head fields
remain separate and subject to ordinary uniqueness refusal. All 29 compiler units
pass, including exact overlapping-support union, empty heads and preserved conflicts;
the engineering ledger qualifies all four complete publications/reopens.

**Fixture corrections:** continuation 18's heater FTPx publication and Rust cold
reader pass. Python initially failed before opening because its explicit spill
directory did not exist; after creating it, the independent cold check passes in
9.48 seconds. Its ownership assertion now reuses the already qualified exact
cache-ownership helpers: no pins/loads remain, and all remaining reservations equal
reported native cache ownership. The same read leaves persisted file hashes unchanged.
One finite-inference fixture expected the predecessor unsigned depth; its repaired
fixture requires the declared signed value with the identical ancestor/depth assertion
and passes in `/tmp/pse-plan09-signed-depth80.log`.

Further F6 qualification found three publication preparation traversals expanding
shared graphs. Conservative dependency discovery now stops at its first opaque
extension. Durable/publication expressions use native typed constructors because
their fields are already resolved. Private DML isolation and namespace deferral use
one reservation-owned native visitor, retaining unchanged owners and cancellation.
The deep shared-plan construction, nested-command, no-effect and ownership units
pass; `/tmp/pse-plan09-effects-storage64.log` records **52 provider/storage/reuse
passes, zero failures, baseline zero, 51.848 seconds**, default/force-validation.
These repairs neither disable an optimizer nor narrow the scientific assertions.

**F17 — Constraints reran native producers (DM-24, DM-26, DM-28, DM-39):**
construction, branch and support collision queries previously planned/executed their
producer before a separate execution returned its values. `strata/native_input.rs`
now completes that producer once, binds its owned values through the existing
immutable candidate provider, and runs the same native collision query over those
values. No key/FK promise is advertised before checking, and no result escapes on
failure. **Tested:** all 107 affected rule/compiler cases pass in CI/force-validation;
`constraints_check_one_completed_execution_and_never_export_conflicting_values`
additionally counts actual volatile-UDF input rows and attacks duplicate keys
(`/tmp/pse-plan09-materialization-unit90.log`, one pass, zero failures, baseline zero).
The counter fixture uses the explicit non-null UDF: at this DataFusion pin, native
`coalesce` simplifies to CASE and may evaluate its argument for both condition and
value; counting its calls would not isolate producer re-execution.

**F18 — Source fixture bypassed common services (DM-25, DM-43, DM-60):**
`unified_sources` constructed a physical publication directly from a raw SessionState,
which had no bound domain execution services. It now binds exact document providers
and prepares an `ArtifactPlan` publication through the normal session route.
**Tested:** `exact_source_text_reopens_and_reparses_from_delta_alone` passes with
unchanged UTF-8/CRLF, reparsing, source support, cancellation, policy and stream-owner
assertions (`/tmp/pse-plan09-source89.log`, CI/force-validation, one pass, zero failures,
4.547 seconds, baseline zero).

Borrowed recursive Arrow field traversal avoids metadata copies during admission.
Existence-only violation queries now explicitly project a literal and apply native
LIMIT 1. All 46 affected library units and 119 declaration/admission/construction
cases pass. These two changes alone did **not** resolve the kernel deadline: the
unchanged 360-second case still timed out, recorded in the Rust ledger. No performance
benefit is inferred from their passing correctness checks.

**F19 — Construction retained unused join payloads and reevaluated witness conditions
(DM-24, DM-28, DM-39):** `crates/pse-rules/src/strata/native_input/projection.rs:15` materializes actual
output fields, witness keys and witness conditions together. Subsequent membership
and support queries consume the same Boolean outcomes. Native projection owns
column pruning; no custom expression dependency analyzer or function whitelist is
introduced. **Tested:** 108 rule/compiler cases pass; the added volatile-condition
regression counts two evaluations for two actual rows, one positive `facts` support
edge and retained absence support. The first regression expected the nonexistent
`positive` kind; the corrected assertion uses the declared `facts` kind.

**F20 — Derived declaration cache was bypassed (DM-01, DM-32, DM-39):** the kernel
trace reaches complete recursive declaration serialization through
`FieldCheckedBatch::check_declaration`. At `crates/pse-relations/src/columnar.rs:351`,
the immutable registry already owns the exact
lossless declaration, but the relation boundary rebuilt it twice on every transfer.
**Implemented and Tested:** borrow `Registry::compiled_declaration`
(`crates/pse-schema/src/builder.rs:182`) for an actual registry-owned
declaration; independently reconstruct external candidates. Identity/fingerprint
equality never substitutes for the full comparison. Existing generated-contract
fixtures attack altered declarations with unchanged fingerprints; all 268 affected
schema/relation/rule/compiler cases pass, followed by all 253 provider/source cases.
The complete kernel test passes in 309.658 seconds under its unchanged 360-second
deadline. Its full output, conversion and durable-reopen assertions are unchanged.

**F21 — Repeated expanded producers and receipt copies (DM-32, DM-39):** native cache
expansion (`crates/pse-catalog/src/session/cache/logical.rs:259`) now memoizes
already-restored producer identities as well as temporary
optimizer leaves, retaining one native owner per shared producer. Sealed original
producers keep their structural admission owner. Native Delta `CommitInfo` values
are moved into typed receipt decoding instead of cloned
(`crates/pse-catalog/src/delta/attempt.rs:170`). Native action parsing,
transaction witnesses, exact request comparison, phase refusal and receipt format
remain unchanged. **Tested:** all 376 catalog/rules/compiler cases pass in
CI/force-validation, 84.456 seconds, zero failures against baseline zero
(`/tmp/pse-plan09-affected96.log`). These mechanism tests do not establish a model
latency improvement.

### Applicable principle verdicts

This architecture replacement spans all twelve charter groups. Each listed principle
is **Satisfied for the stated scope** by the enforcement and attacks below; this is
not a formal proof or an assurance about unimplemented simulator/deployment behavior.
DM-51's predecessor migration workflow is **not applicable**: the authorized hard
pivot discards predecessor data, while exact incompatible current revisions refuse.

| Principles | Enforcement and observable refusal / evidence |
|---|---|
| DM-01–DM-05 | Registry/provider/Delta authority map in §2; derived caches retain actual owners and explicit domain boundaries. Forged declarations, foreign sources and unsupported codecs refuse. Deletion inventory and generated equality exclude parallel model authorities. |
| DM-06–DM-10 | Complete typed alternatives, explicit NULL/UNKNOWN/error states, native keys/references and recursive Arrow fields. Q01–Q03/Q07 malformed-arm, visible-value, singleton/ordinal and reference attacks pass. |
| DM-11–DM-15 | Entity, selected revision, artifact and invocation identities are distinct; resource limits are not semantic identity. Exact declarations and selected Delta versions govern equality. Same-fingerprint impersonation, stale-parent and opaque-generation reuse refuse. |
| DM-16–DM-19 | Registry algorithms, input consumption, policies and complete method alternatives generate native bindings. Projected arguments cannot recover omitted fields; source-to-template and method/kernel fixtures exercise actual bindings. |
| DM-20 | Inspection and preparation neither publish nor mutate values. Provider EXPLAIN/deferred-write sentinels and unchanged-file hashes across all four Python readers pass. |
| DM-21–DM-25 | Native logical/physical plans and coherent math IR expose lowering; specialized graph/quantity/solver algorithms have typed boundaries. Q04–Q06 support, dimensions, ordered children, exact conversions and native recomputation pass. |
| DM-26–DM-30 | Once-prepared strata, actual round reset, declared effects, private native mutation and member/root phase receipts separate preparation, execution and commit. Repeated/volatile witness, duplicate-key, cancellation, lost-acknowledgment and no-replay attacks pass. |
| DM-31–DM-35 | Complete dependencies retain original reads and negative scopes; consumed-column narrowing requires enforced access and exact bag equality. Native role/source identities distinguish graphs and constrain ordering. Changed keys, missing CDF history, unknown implementations and cross-process maintenance are tested. |
| DM-36–DM-40 | Arrow columns and native kernels provide actual layouts; numerical callbacks retain contiguous owned vectors. The 84-record matrix and complete publication timings report end-to-end costs. Independent quantity/conservation assertions and ordered output tests retain precision and deterministic semantics. |
| DM-41–DM-45 | Mechanical generated adapters, exact storage codecs, actual provider hooks, versioned native capabilities and current policy admission govern boundaries. Narrowed fields, hidden/visible storage children, extension/field mismatches and foreign ownership are attacked in the final 253 provider/source cases. |
| DM-46–DM-50 | Typed source selections and complete-head support aggregation retain lineage; bounded native plan nodes/edges and typed diagnostics expose lifecycle state. Q04/Q11 source edits and original UTF-8 spans survive complete publication; exact pins/source archives define the supported reproducibility boundary. |
| DM-52–DM-55 | Registry-generated contracts, regeneration equality, exact schemas and adversarial boundary/lifecycle fixtures are discoverable through provider inspection and the execution inventory. Generator, feature, documentation and Python quality checks pass at named source boundaries. |
| DM-56–DM-60 | Existing native mechanisms and the existing registry projection replace parallel code and duplicate work. The simpler cache-disabled native configuration remains an oracle. Measured defaults, explicit unknowns, source-specific receipts and independent gates prevent unbacked performance/capability claims. |

## 8. Alternatives and architectural leverage

| Alternative | Consequences | Decision |
|---|---|---|
| Retain predecessor stages/stores plus separate application memoization | Two authoritative paths, duplicated invalidation and transition policy | Delete; incompatible with the authorized hard pivot |
| Shared native caches and prepared strata with exact disposable ownership | Less repeated planning/replay/decoding; explicit lifecycle and proof obligations | Implemented target, subject to correctness and cost evidence |
| Simpler native execution with caches disabled and fresh planning/opening | One correct authority and fewer retained owners; repeats work | Retained as a **configuration of the same native route**, not a fallback architecture; exact recomputation oracle and pressure bypass |

The simpler option is preferable where cache population costs exceed reuse or where
identity cannot be proved. No persistent snapshot serialization or second eviction
algorithm is justified. Native DefaultCache owns TTL/LRU; native Delta owns replay,
transactions, checkpoints and checksums. The selected source patch exposes six narrow
integration seams, with all 142 selected files reproducibly verified.

## 9. Verification and measurements

### Obligation evidence

The [Rust coverage ledger](../evidence/native-cache-rust-qualification-2026-09-18.json)
identifies individual tests, original failures, repair receipts and the current
source archive. Counts from overlapping selections are not added together as if they
were distinct tests. Baseline is zero. The original inventory contains 993 tests;
nine added regressions bring the current inventory to 1,002, with no deleted test.
All 1,002 have passing receipts. Final source snapshot 25 is qualified by the
268-case schema/relation/rule/compiler selection, 253-case provider/source selection,
309.658-second kernel roundtrip, 72 Python tests and four independent cold readers.

| Obligation | Concrete enforcement and accepted evidence | Disposition |
|---|---|---|
| Q01 | `pse-schema` exact field/declaration tests; catalog captured/native/tagged layouts; nested durable admission rejects relabeling before physical planning | Closed for the stated scope; see exact ledgers and source limits below |
| Q02 | Generated selected payloads, native value contracts and conformance invariants; invalid arms, ordinals, containers and visible children refuse | Closed for the stated scope; see exact ledgers and source limits below |
| Q03 | Row-key and composite-reference tests; native anti-join reference checks bind exact selected members; nested occurrence units retain masks/multiplicity | Closed for the stated scope; see exact ledgers and source limits below |
| Q04 | Native rule support units cover filter/join/UNION/window/unnest; source-span checks use selected UTF-8 documents and retained owners | Closed for the stated scope; see exact ledgers and source limits below |
| Q05 | `mathir::relation_roundtrip`, coherent compiler relations and native numerical expressions preserve ordered children, typed payloads and identity | Closed for the stated scope; see exact ledgers and source limits below |
| Q06 | Twelve linked solver cases pass in continuation 13, including derivatives, multipliers, cancellation, callback panic containment and drop ownership; 256-callback reservation measurement is recorded separately | Closed for the stated scope; see exact ledgers and source limits below |
| Q07 | Final storage/provider/source selection passes all 253 cases; standalone native Delta scalar `CHECK` constraints, UNKNOWN handling, defaults and SQL mutations retain their assertions | Closed for the stated scope; see exact ledgers and source limits below |
| Q08 | Native durable value/property/layout tests and Python raw/registered Arrow tests pass; field and timestamp mismatch refusal remains explicit | Closed for the stated scope; see exact ledgers and source limits below |
| Q09 | Current `provider_contracts` and nested-write cases verify actual caller functions/rules, namespace discovery, metadata, no-scan policy and deferred DDL/DML | Closed for the stated scope; see exact ledgers and source limits below |
| Q10 | Exact member/root retries, stale parents, lost acknowledgments and failure-at-each-object tests pass; unchanged-member selections are checked | Closed for the stated scope; see exact ledgers and source limits below |
| Q11 | Both heater and both mixer graph journeys/publications pass; all conservation, source/edit/compiler cases and the exact kernel publication/reopen now pass | Closed for the stated scope; see exact ledgers and source limits below |
| Q12 | Seventy-two Python tests and all four independent engineering cold readers pass on the final rebuilt extension, covering stream/buffer lifetime and codec refusal | Closed for the stated scope; see exact ledgers and source limits below |
| Q13 | Actual selected-cache/projection-reuse test passes with native clean recomputation, changed keys, opaque-generation refusal and child-process maintenance; bounded/gapped CDF and protected versions are covered | Closed for the stated scope; see exact ledgers and source limits below |
| Q14 | No predecessor stage/controller/store/rule authority remains; generated restricted arguments prove extension locality; 84 cache measurements and exact callback reservations are recorded | Closed for the stated scope; see exact ledgers and source limits below |

**Tested:** K01–K11 are closed by the following mechanisms and the exact receipts
above. **Measured:** the 84-record matrix qualifies mechanism counts and conservative
defaults where applicable; it does not establish a universal speedup.

| Cache obligation | Accepted evidence mechanism |
|---|---|
| K01 | Semantic setting classification units and actual requesting-session reuse/refusal |
| K02 | Accounted metadata/snapshot/resident/binding owners, bounded fill coordination, pressure/eviction/cancellation and exported-buffer lifetime |
| K03 | Actual store/root namespace tests and instrumented native Parquet/CDF metadata reads |
| K04 | Empty/nonempty round hash/cross joins, native reset refusal/cleanup and constant preparation counts across 5/10/18 fixed-point rounds |
| K05 | Exact old/latest and Metadata/Query snapshots, native seeded incremental refresh and independent native Add-action comparison |
| K06 | Cache-off/on selected reads, NULL/duplicate/revision semantics, incomplete-stream refusal and Rust/Python owner retention |
| K07 | Native consumed-column/CDF/endpoint multiset comparison, independent clean output oracle, changed-key and unknown-generation refusal |
| K08 | Actual child-process maintenance with idle parent caches, fresh lease/fence checks, protected versions and late-fill/cancellation units |
| K09 | Bounded non-mutating inspection, actual native scan metrics and NULL unavailable counters; telemetry excluded from semantic inputs |
| K10 | Failure injection after native member/root/checkpoint/checksum writes; missing/corrupt/stale CRC native replay controls; measured checkpoint interval and partition choices |
| K11 | Current deletion audit, exact 142-file Delta overlay verification, native ordinary extension paths and explicit unsupported-codec refusal |

### Final commands and source boundaries

**Tested, baseline zero, no retries:**

- `just test --profile ci -E '"package(=pse-relations) | package(=pse-schema) | package(=pse-rules) | package(=pse-compiler)"'`: 268 passed, zero failed, 21.884 seconds; force-validation via recipe, `/tmp/pse-plan09-declarations97.log`.
- `just test --profile ci -E '"package(=pse-catalog) | binary(=unified_sources)"'`: 253 passed, zero failed, 81.656 seconds; `/tmp/pse-plan09-provider-final99.log`.
- `just test --profile ci --nocapture -E '"binary(=method_realization) & test(=kernel_descriptor_compiles_exact_natural_units_without_inventing_execution)"'`: one passed, zero failed, 309.658 seconds; `PSE_TEST_PHASE_TIMINGS=1`, 390 outputs and complete publication/reopen, `/tmp/pse-plan09-kernel98.log`.
- `just py-sync`, `just doctor`, `just py-test`: fresh extension, ready environment, 72 passed/zero failed in 7.12 seconds, 32 Python workers. Four one-worker integration-marked Python readers independently reopen the existing full publications with unchanged-file and final-owner assertions.
- `just clippy` (default and no-default), `just codegen-check`, `just quality`, rustdoc/book: pass on the final production source. Source archive 25 captures it. Final prose receives a separate documentation/static check.

The [engineering ledger](../evidence/native-cache-engineering-2026-09-18.json) records
four successful 390-output publications, Rust reopening and independent Python
inspection. Rust publication receipts precede the final admission/construction repairs;
all four Python inspections use the final extension. The final kernel and affected
Rust suites qualify those later repairs. Doctests, 12 linked native solver tests,
84 feature checks and the 84-record cache matrix keep their earlier recorded source
boundary; late changes do not change their feature, FFI, reset or cache-storage contracts.

The kernel timing includes concurrent checks and one 0.52-second debugger sample.
No timeout, retry, memory ceiling, numerical tolerance or independent scientific
assertion was relaxed. Capability-gap scans report no production gap in the final
changed scope; two findings name tiny planning-only test SessionContexts. Cargo's
upstream `proc-macro-error2` future-compatibility notice remains visible; strict
workspace Clippy has zero findings. No formal correctness or total-heap bound is claimed.

### Historical campaign boundaries

Baseline is zero. The inventory names exact isolated-unit receipts and the final
target fixtures. `just architecture-acceptance <new-directory>` captures exact source
hashes, tracked diff, untracked source archive, dependency/toolchain provenance,
command logs and elapsed times. It stops on the first failed command and does not
assign gate verdicts.

The initial campaign stopped at physical generation. After F4/F5 were corrected,
the second campaign passed complete generated equality and workspace compilation,
then caught a test-only primitive-sort lint. That lint is fixed. A continuation may
use `--start-at <gate>` in a new evidence directory, preserving a new exact source
snapshot; its receipt never certifies earlier commands. The Rust suite uses
`--no-fail-fast` to collect all current failures in one campaign without retries or
larger timeouts. Earlier source receipts remain separately assessed.

The cost matrix includes actual cyclic strata and reusable round plans, M/K Delta
open dimensions, independent fresh processes, retained/incremental snapshots,
selective/wide reads, CDF images, CRC load classes, resident pressure and exact
projection reuse. Setup and compilation are separate from execution. OS cache is
uncontrolled; process VmHWM is process-wide, not a per-case allocator trace.
Numerical callback tests report reserved memory and release, not an invented count
of all allocator calls. No universal latency target or controlled speedup is established; measured defaults
and absolute kernel/publication timings are recorded with their conditions.

## 10. Exceptions and unresolved decisions

Unknown upstream allocation/counter detail is explicitly unclaimed, not a fabricated
zero. Remote destructive reader coordination is not implemented or admitted as safe;
the local provider framework and ObjectStore seam remain the supported scope.
Unrelated cold implementation generations conservatively refuse semantic reuse.
Proposed ADR-0068/0069/0070 remain proposed; this review does not accept records or
amend the blueprint. Numerical cache/scan/checksum defaults are selected conservatively
in [the measurement receipt](../../plans/09-cache-measurements.md). These are explicit
resource ceilings, not a measured optimum or a claim of universal cache speedup.

## 11. Decision and implementation changes

| Item | Status | Evidence and scope |
|---|---|---|
| Native schema/provider/cache pivot and deletion cut | **Implemented and Tested; complete** | C00–C12, Q01–Q14 and K01–K11; exact aggregate coverage and affected final source receipts |
| Native CDF/nested/visibility/support corrections | **Tested; complete** | Final provider/source suite, conservation cases and four complete engineering publications |
| Runtime resource claims and opaque implementation refusal | **Accepted within the declared bounds** | Pressure, ownership, inspection and refusal tests; unavailable allocator/replay details remain unknown |
| Overall acceptance | **Accept for the stated scope** | G1–G7 independently pass; measured defaults recorded; future/remote/numerical-parity claims excluded as originally scoped |
