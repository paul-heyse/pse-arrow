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

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| C01 | Planned | Actual operation/protection/read owners; required contract decisions | Typed intent-level procedures and explicit isolation used by affected canonical callers; replaced framing and incidental result interpretation removed. |
| C02 | Planned | A01/A02 carrier meaning; C01 enclosing transaction | Typed shared readiness facts/decisions replace JSON adaptation across native/WASI consumers; preparation/custody remains explicit. |
| C03 | Planned | C01 decision footprint; 72e dependency/freshness contract | Scope-appropriate native conflict coverage and one retry owner replace concurrency-only broad guards; authority/semantic fences retained. |
| C04 | Planned | C01 operation contract; C02 policy where consumed | Direct/API access shares one operation policy with explicit authority; any replaced ingress or runtime patch is removed only at its covered boundary. |

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
writes. Integrated receiving belongs to 72i. **Proposed:** no implementation, runtime test or
measured benefit occurred during this plan's authoring.
