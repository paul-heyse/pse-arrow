---
title: "Plan72c: Canonical operations and policy"
status: active
date: 2026-11-09
adrs: [ADR-0262]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, design, libraries, validation, preparation]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/canonical-operations-and-policy.md]
---

# Canonical operations and policy

## Target and foundation assessment

**Proposed.** A native caller supplies one typed project/study intent and receives its original
receipt or an honestly unresolved outcome. Generated native procedures perform relational
selection/checks/publication under explicit isolation; shared Rust owns genuine study policy.
Typed readiness facts/decisions replace JSON-in-bytes. Reduce unnecessary framing, broad conflict
writes, conversion and repeated calls while retaining meaning, authority and physical custody.
The [coordinator](72-surrealdb-functional-improvements.md) owns O04–O06 and O11's shared
disposition; C01–C04 below own the implementation state. No historical-plan scope is inherited.

**Interface-checked baseline:** `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the
concurrent working tree inspected on 2026-11-09. Current
[GuardedScript](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/src/canonical_script.rs)
already types installation, protection, stage and authority envelopes and result slots.
Generated procedures already perform database-local multi-record effects. This is a usable
integration foundation; target complete-operation boundaries rather than invent first adoption
or mechanically move every check into a function. The
[readiness module](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-study-module/src/lib.rs)
has native page types but JSON policy/facts/outcomes in bytes and Decimal integer adaptation.
Those are concrete representation targets.

The selected core/heuristics/profile/binding apply. RC02's direction was explicitly accepted
for planning on 2026-11-09. Complete the required decision/design route for D10 placement,
metadata/commit or authority changes before implementing them. RC01's declared mapping supplies
72a inputs. Plan66 retains scientific policy and acceptance; Plan71 retains test infrastructure.

## Chosen operation model

Keep semantic input, original operation/issuer, effects and expected result in an identifiable
typed operation contract. Emit coherent `fn::` procedures for database checks/publication and
minimize incidental statement slots. Select serializable writes explicitly for read-dependent
decisions and read-only snapshots for pure observations; a protected acquisition that writes
custody is not a pure read. The patched ordinary default is not a substitute for explicit intent.
[Transaction guide](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions),
[selected runtime patch](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/scripts/surrealdb-source/surrealdb-v4-runtime-controls.patch).

Use the database's predicate/range conflict tracking instead of blanket generations used only
for concurrency. Where the decision spans more than one database transaction, use scope-specific
witnesses with complete absence/membership meaning. Domain incarnation, lease and authority
fences and original acknowledgements stay where they convey those meanings. One retry owner
repeats settled conflicts under the original clock; unknown effects are reconciled first.

Keep visible SQL acquisition and guarded publication around the pure shared Rust readiness
decision. Generate typed carriers from its semantic owner and use native conversions only where
they preserve meaning. This is the selected target, not a benchmark candidate. A transaction-
capable extension is an alternative only if a concrete acquisition/transfer problem defeats
that simpler composition. `init` can precompute real immutable metadata; call-local memory and
ABI-5 `BINDINGS` do not own durable state. [Extension architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture).

## Packages

### Execution checkpoint — 2026-11-10

Execution is authorized under proposed ADR-0262. Explicit transaction rendering,
transport-owned single-operation result projection, owner-derived native readiness and
consumer migration are implemented together in the clean v12 contract. The composing
owner supplies the projection graph, preserving the generator's ordinary dependency
ceiling. Portable Arrow rows and native carriers retain separate boundary roles.
The global study-publication row/event fanout and JSON policy adapters are deleted.
Actual SSI/protection races, native/WASI policy and shared direct/API receipt controls
have passed. The constrained definer API and runtime patches remain selected because
they still discharge rollback, authority and physical custody obligations.

Implementation is committed in `819b01bc6`. The maintainer requested moving on before
final receiving/closeout; this plan remains active for that verification, with no further
feature work identified. The compiler dependency-ceiling discussion is outside current
work: use of Tokio alone does not establish dependency on product runtime code or ownership
of product execution responsibilities. No library split or rule change follows from the
transitive package name alone.

Final receiving at handoff is **not complete**. The unchanged 69-case command finished
with 67 passes and two 360-second timeouts in the 10,000-member capture cases; the three
earlier initialization failures passed. Its fixture settled. Final pure module tests passed
10/10. The native six-case rerun did not start: discovery compilation found the new bootstrap
parser test referencing optional `surrealdb_expr` without enabling it in a `native-service`-only
selection. Correct that test/dependency configuration before rerunning. These are remaining
verification/repair obligations, not passing evidence. The maintainer requested no further
investigation or new runs in this engagement.

A repeated default-concurrency selection exposed
the engine's global index-build barrier: schema setup in one logical database can wait
behind long writes in another. The initialization owner is being corrected to submit
concurrent native index builds and validate exact required readiness before admission,
under the original deadline. The earlier selected startup failures are not passing evidence.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| C01 | Implemented; receiving pending | Actual operation/protection/read owners; required contract decisions | Explicit isolation and one transport-projected result; closed fresh bootstrap requires exact native readiness. |
| C02 | Implemented; receiving pending | A01/A02 carrier meaning; C01 enclosing transaction | Source-owned native/WASI projections replace JSON policy adaptation across storage and runtime consumers; bounded raw admission precedes decoding. |
| C03 | Implemented; receiving pending | C01 decision footprint; 72e dependency/freshness contract | Actual SSI capture replaces the global publication witness; one original-clock retry owner and semantic/authority fences retained. |
| C04 | Implemented; receiving pending | C01 operation contract; C02 policy where consumed | Shared direct/API original receipts and explicit authority; constrained definer API and runtime controls deliberately retained. |

<a id="C01"></a>
### C01 — Expose a complete operation and its isolation

Trace actual project, publication, protection and stage callers through the existing typed
envelopes and generated procedures. Group check/selection/update work into a named typed
operation with one result contract. Keep separate effects and physical owners where they buy
real cancellation/custody guarantees; do not turn every semantic operation into another task,
transaction or service. Install and inspection paths retain their deliberate snapshot meaning.

Make serializable dependency protection explicit where correctness needs positive/negative/
range reads, instead of relying on an unspecified `BEGIN` or implicit function isolation.
Return original receipt/outcome and avoid caller arithmetic over incidental result positions.
Migrate affected callers and delete superseded script framing/extraction helpers in the same
change, retaining shared ownership helpers that still discharge a concrete obligation.
Targeted cases cover lost head, wrong issuer, authority expiry, multi-statement refusal and
complete output. Early `RETURN` is normal completion and must not be used as rollback.

<a id="C02"></a>
### C02 — Generate typed readiness and stable preparation

Replace module-local JSON policy/facts/outcome bytes and repeated Decimal adaptation with
owner-derived typed carriers using 72a's selected representation. Keep aggregate admission before
owned decoding; type safety alone does not limit allocation. Use the same pure decision owner
for native and WASI paths, retaining independent expected outcomes and refusal non-effects.
Preserve the relevant generation, occurrence, original revision and unknown-effect semantics.

Move immutable repeatedly decoded policy/descriptor metadata into build-time `init` where it
actually removes work. Do not create a generic descriptor runtime for a fixed local policy.
If per-definition configuration benefits from `BINDINGS`, check the exact selected builder/ABI
support first; unsupported ABI is a bounded preparation choice, not a reason to hold typed
readiness work. Retain archive signature/digest, capabilities and original preparation identity.

No solver/native mutable work enters the database transaction. A transaction-access extension
is reopened only for a specific acquisition/transfer limitation, with declared reads and a
smaller complete operation. Tests exercise missing predecessors, changed revisions, cancelled
and unknown outcomes, oversized aggregate input, and cold initialization/deadline overlap.
Delete replaced JSON adapters/fixtures rather than preserve both normal policy routes.

<a id="C03"></a>
### C03 — Narrow conflicts and unify retries

Classify each shared generation/witness by role: database read-set conflict protection,
semantic interpretation, source freshness, or issuer/attempt authority. Prefer exact SSI
tracking for the first role inside one transaction; otherwise narrow the witness to the actual
scope with complete positive, absent/future and membership dependencies. Coordinate with 72e
so summary maintenance and publication use one chosen dependency model.

The [capture witness generator](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs)
currently reacts across many source tables. Removing unrelated increments has a clear design
benefit; correctness of the narrowed footprint is the serious risk. Targeted independent
writers must cover disjoint and overlapping captures, phantoms, absence, deletion and changed
epochs before broad guards disappear. No contention speedup campaign is required.

Use supplier retries only when their exact standalone operation covers the same conflict unit
and original deadline. Otherwise keep the existing explicit-transaction owner; remove stacked
retry policy. Native unique indexes replace suitable check/create coordination; sequences serve
allocation only when gaps and allocation order are allowed. Do not replace commit-ordered
history or exact publication freshness with sequences/bounded staleness. Test lost commit reply
and deadline exhaustion, recovering original acknowledgements before replay.

<a id="C04"></a>
### C04 — Share operation authority across selected access surfaces

Generate callable arguments/results and any selected `DEFINE API` adapter from C01's operation
meaning. Keep direct native access as the ordinary local realization. Use a database API only
where it replaces an existing narrow ingress or supplies a useful independently accessible
operator operation with less lifecycle machinery; it must not duplicate business policy.
Declare invocation permission and `SECURITY INVOKER` for actor-scoped access. Privileged
operations have explicit constrained authority instead of accidental definer defaults.
[API reference](https://surrealdb.com/docs/reference/query-language/statements/define/api).

Compare the exact retained runtime/build patches against public mechanisms at their obligation,
not by patch-line count. Remove a patch only if native admission/deadline/rollback behavior covers
the required scope. A serious rollback/custody risk warrants the paired earlier-enclosing-write
handler test, intentional error responses and physical cancellation/release controls. Network,
file and numerical effects remain outside SQL rollback. Binary API bodies need a real decoder.

Migrate selected callers and remove replaced ingress, policy duplication and its obsolete tests.
If an API or patch alternative supplies no complete improvement, record retention as the settled
choice and finish that branch; do not leave an indefinite comparison gate. Delivery exercises
authorized/denied/malformed calls, revocation, original acknowledgements and equal direct/API
effects through the selected projections.

## Coverage and verification

| Review direction | Owner and decision |
| --- | --- |
| O04 procedures, explicit SSI/read-only, guard granularity, retry, unique/sequence primitives | C01/C03; select coherent operations directly, investigate only unresolved corruption/authority risks |
| O05 typed readiness, acquisition placement, init, BINDINGS and runtime/build controls | C02/C04; typed pure policy selected, transaction access only for a concrete limitation |
| O06 shared operation meaning, API ingress, invoker/definer and rollback | C04; one policy with a selected useful adapter, no compulsory additional server |
| O11 precise invalidation | C03 supplies canonical conflict footprint; 72e owns derived/follow freshness and consumer behavior |

Implementation uses scoped compile checks and targeted procedure/module/client tests with
applicable force-validation. Move all affected consumers and delete replaced paths as they are
proven, not in a later compatibility phase. Additional proof addresses serious degradation risks;
no general performance proof is required for removing crossings, JSON conversion or broad conflict
writes. Integrated series receiving belongs to 72i. Execution evidence follows separately
from the original authoring baseline.

## Outcome

**Implemented, 2026-11-10.** Canonical mutation/custody and guarded-read operations use
explicit serializable transactions. Pure observations use readonly snapshots; installation
retains its deliberate writable snapshot exception. The transport checks all body and commit
errors before exposing one operation result; caller statement-position arithmetic is removed.
Protected history acquisition remains a guarded read, including authority registration.

Owner-derived native projections now cover stored policy, facts, outcomes, action/start and
readiness premises across SQL, native/WASI policy and runtime consumers. The generator receives
the source projection graph from its composing owner without adding a normal model dependency.
Generated encode/decode layout guards protect source correspondence. Raw tree admission precedes
owned decoding and charges every node, payload and key; full unsigned values retain exact native
U64 representation with checked wire widening. JSON-in-bytes policy adapters are deleted.
Portable Arrow rows retain their own representation and do not become another semantic owner.

The concurrency-only study-publication row, event fanout, helper calls and mechanism-specific
fixtures are deleted. Exact selected-study lookup and native serializable positive/negative/range
tracking protect atomic frozen capture. Semantic/issuer/lease fences, history, file-owner custody,
original receipts and one original-clock retry owner remain. Empty selections avoid unrelated
point scans. Captured-member paging uses an optional unsigned cursor, so the initial page includes
ordinal zero without a signed sentinel.

Fresh initialization now derives concurrent bootstrap index declarations and exact expected
observations from the same source index registry. It commits definitions with admission
closed, validates every actual index and the selected module, then atomically opens admission
against the original marker/custody. Reentry resumes that same installation rather than
replaying DDL. This replaces sequential global index-build waits during fresh setup; ordinary
maintenance retains its blocking contract. A source-count-derived bootstrap metadata profile
admits the complete index inventory within finite node/depth/byte limits and admits no blobs.

**Implemented finite decisions.** The pure readonly Rust policy with visible SQL acquisition
and publication is sufficient. Fixed generated adapters remove the descriptor decoding that
might justify `init`; no immutable per-definition configuration requires ABI-5 `BINDINGS`.
There is no selected transaction-access extension. The constrained definer control API and
runtime admission/deadline/rollback/settlement patches remain because their required guarantees
are not replaced by ordinary native HTTP response handling. Direct/API operations share original
receipt meaning. Native engine constraints backstop representable structure; recursive diagnostics
use native object edges with full generated Rust checks because recursive named types are
unsupported by the selected engine.

**Tested.** Selected controls used Linux, the pinned source-built v4 server/module and native
correctness routes with explicit force-validation where applicable. The target is zero failures:

| Command / selected scope | Result |
| --- | --- |
| `scripts/pse test pse-codegen --lib` | 107 passed, 0 failed; generation, native projections, source layout/kind checks and deterministic full-registry lowering. |
| `scripts/pse test pse-study-module --lib` | 10 passed, 0 failed; native/WASI shared policy, aggregate limits, opaque input refusal and exact typed products. |
| `scripts/pse test pse-operations --lib --features canonical-maintenance -E 'test(~atomic_capture::) or test(~canonical_control::tests) or test(~canonical_script::tests) or test(~canonical::transport::tests) or test(~operation_return_checks_all_errors) or test(~native_sql_serializable)'` | **Failed:** 67 passed, 2 timed out at the unchanged 360-second limit. Default 16 lanes, unbounded placement, prepared module `bb7ddaed…`; actual fixture settled. Both large capture cases remain unresolved. |
| `scripts/pse test pse-operations --lib --features canonical-maintenance -E 'test(~canonical::bootstrap::tests)'` | 5 passed, 0 failed; actual closed bootstrap reentry keeps issuer/epoch and blocks ordinary opening, missing index remains closed, changed/failed/pending inventory refuses readiness, and the complete generated observation parses. |
| `scripts/pse test pse-operations --lib --features canonical-maintenance -E 'test(~canonical_server_unit_exact_conflicts)'` | 20 passed, 0 failed; real native serializable point/range/absence races, protected reads, snapshot observations, whole-decision retry and disjoint writers. |
| `scripts/pse test pse-operations:native_service --features canonical-maintenance -E 'test(~native_control_http_actor_grants_original_receipts_and_raw_envelopes)'` | 1 passed, 0 failed; direct/API original receipts, grants, denied/malformed/revoked authority and raw envelopes. |
| Native operations typed full-u64 and nested diagnostic roundtrips | 2 passed, 0 failed against the installed module/schema. |
| Native operations readiness/progression/refusal selection | 6 passed, 0 failed; existing policy and guarded publication behavior. |
| `scripts/pse test pse-operations --lib --features native-service --run-ignored all` selecting the four module qualification/open cases, with exact prepared qualification receipt and `SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS=4` | 4 passed, 0 failed; missing/tampered module refusal, fault rollback/recovery, original pool expiry and cancelled apply receipt. The four-controller setting selects this fixture's bounded cold/deadline test, not a workstation resource cap. |

### Mistakes made and corrected

Native `FOR UPDATE` registration is not permitted in a readonly transaction, and the engine's
lockset alone does not create a write conflict. Actual native tests corrected the initial
observation/read classification while preserving mutation authority registration. Native parser
receiving exposed recursive named-type and `FLEXIBLE` placement limitations; generated acyclic
aliases plus literal recursive roots now respect the engine and retain full Rust validation.
Two general renderer tests still supplied the full registry without its semantic graph; their
composing calls were corrected and the complete 107-test suite passed. Early fixture setup
failure was mistakenly reported as unsettled solely because Nextest never started; settlement
now reads the actual fixture producer fact, with a regression control. A failed setup remains
a failed/incomplete test execution.

The repeated 69-case native selection initially had 66 passes and three fixture-initialization
timeouts at normal 16-lane concurrency; the affected three passed alone, but that did not
qualify the complete selection. Exact engine source showed each blocking index could wait
behind unrelated datastore writers. Concurrent bootstrap plus closed admission addresses
that mechanism without changing deadlines or lanes. Independent review also caught the
new observation exceeding the small header budget and an ordinary open-only fence rejecting
the deliberately closed marker; bootstrap-specific bounded admission and identity validation
correct both defects before acceptance.

### Deliberate choices and receiving limits

**Implemented.** This is a direct clean-v12 cutover, with no JSON compatibility policy path or
v11 result-preserving reader. Scientific policy still belongs to its existing owner; no solver
or external side effect enters SQL rollback. No configured deployment was reset. 72e consumes
the qualified canonical conflict model and still owns derived/follow freshness; 72f owns
transport/lifetime and protected storage scope. 72i owns assembled journeys and eventual
retirement. Expected reductions in interpretation, conversion and broad conflict writes are
qualitative; no measured speedup is claimed.
