# Canonical operations and policy

Reviewed 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` plus the concurrent
working tree. Guarded scripts, retry, generator, module and supplier-patch sources were
inspected directly. The current dirty migration/service files remain live implementation
context, not a refreshed qualification claim. Mechanisms are **Interface-checked**;
the recommendations below are **Proposed**. No concurrency or fault probe ran.

The [principal review](../surrealdb-functional-improvements-2026-11-09.md) owns the combined
dispositions. Core 3.4, efficient architecture 1.0, process-simulator 1.5 and the binding
shape this assessment. AP-04 preserves semantic authority, AP-07 judges complete physical
work, and DP-18/DP-19 require truthful effects, custody and outcomes. Native solver state
and scientific admission do not become database policy merely because data is nearby.

<a id="O04"></a>
## O04 — Complete named operations with explicit isolation

**Recommend explicit isolation and coherent typed procedures; condition removal of witnesses
on their precise role.** A project/study caller should submit one meaningful operation and
receive its original receipt or an honestly unresolved outcome. It should not assemble
transaction framing and interpret multiple incidental result positions to obtain that behavior.

[GuardedScript](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_script.rs)
already centralizes envelopes, authority bindings, deadlines and typed extraction; generated
project/research functions already perform multi-record effects. The target is consolidation
of the remaining physical composition, not first adoption of stored functions. Emit one
typed `fn::` operation around the necessary checks and publication and return its final
result. Keep an explicit owner for semantic input, effects and original operation identity.

Use explicit serializable write transactions where a decision reads membership, absence
or ranges; use genuine read-only snapshots for pure multi-query observations. Do not classify
a protected read as read-only when it acquires or renews a protection. The selected runtime
patch changes ordinary isolation defaults, so moving an existing envelope to an implicit
function transaction can weaken protection unless the effective route is checked.
[Runtime patch](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/scripts/surrealdb-source/surrealdb-v4-runtime-controls.patch).
The public [transaction guide](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions)
distinguishes atomic implicit execution, explicit isolation and commit-time conflict protection.

Prefer native predicate tracking over a blanket generation used solely to protect a database
read set. In particular, the existing capture witness increments on selected changes across
many tables and can conflict unrelated captures. Compare scoped native reads with that
conservative protection. Keep generation/incarnation/lease values that carry semantic or
authority meaning; serializability does not identify a stale owner or acknowledge an operation.
[Witness generator](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-codegen/src/codegen/surreal_study_capture_witness.rs).

**Alternatives and choice.** Keeping current envelopes is safe while native dependency
coverage is unsettled, but a permanent universal guard can amplify conflict beyond useful
work. Fully moving all orchestration into extensions hides queries and adds runtime custody.
Prefer visible generated SurrealQL for relational checks/updates, shared Rust for genuine
policy, and a narrow native execution envelope. Delete result-slot arithmetic, duplicated
framing, guard writes or round trips only where the complete operation no longer needs them.

Keep one retry owner. The [current retry layer](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/transport_retry.rs)
repeats only settled conflicts under the original clock. Supplier standalone retries may
replace it for qualifying single statements; they do not justify replay of an explicit
multi-statement effect or a lost acknowledgement. Native unique indexes can replace eligible
check/create coordination. `sequence::nextval` can serve allocation allowing gaps, but cannot
replace commit-ordered or gap-free history counters; bounded-staleness DS reads do not fit
exact publication selection.

**Delivery witness.** Exercise two disjoint and two conflicting writers, absent/range
phantoms, changed head/epoch, expired authority, deadline exhaustion and lost commit reply.
Observe equal useful outcomes and fewer unrelated conflicts before removing the broad witness.
A function's early return is normal completion; test refusal/rollback explicitly. Changes to
commit, metadata, placement or D14 dependencies enter the normal ADR/design route before adoption.

<a id="O05"></a>
## O05 — Typed readiness policy and deliberate database-local execution

**Recommend typed policy carriers and visible transactional acquisition; retain the pure shared
Rust decision as the default.** A study user gains consistent readiness/claim explanations
across native inspection and database progression, with less marshaling and fewer opportunities
for interpretation drift. This is a functional improvement even without changing numerical policy.

The [current module](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-study-module/src/lib.rs)
uses typed `SurrealValue` page carriers but embeds policy/facts/outcome JSON bytes and converts
Decimal generations back to integers. It preflights aggregate content before decoding and invokes
the shared native `candidate_action`. Surrounding SQL acquires facts and publishes changes.

Generate typed native carriers from the policy owner, aligning with O01 rather than introducing
a schema-first generic decoder. Remove JSON-in-bytes and repeated manual adaptation where native
types preserve all required distinctions. Retain aggregate admission at the uncertainty boundary;
typed conversion does not bound allocation. Keep query-visible fact acquisition and guarded
publication in one explicit operation under O04, with the pure extension supplying the decision.

The v4 extension model adds call-scoped `Transaction`, fresh invocation instances and build-time
`init`. Use `init` for genuinely immutable reusable policy/descriptor metadata. A transactional
extension becomes preferable only when it eliminates substantial selected acquisition/marshaling
that visible SQL cannot express economically; it must not conceal undeclared dependencies.
Neither call-local memory nor initialization owns durable acknowledgements.
[Extension architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture).

**Alternatives and choice.** Caller-side Rust plus one transaction can simplify testing but
requires complete conflict protection over acquisition-to-publication. A transaction-capable
extension can reduce transfer but hides planning and lengthens WASI/database lifetimes. Prefer
the current pure shared policy with better typed composition first. This preserves independent
native testing while leaving an explicit upgrade path for expensive acquisition. ABI-5
`BINDINGS` is conditional: confirm selected Surrealism 0.7.0 builder/runtime support before
using it for per-definition configuration; a supplied overview is not that confirmation.

Retain artifact digest/signature and explicit capabilities. Public runtime resource limits
do not yet establish equivalence to the retained process-wide fail-fast admission and original
deadline through physical Store release. Remove runtime/build patches only on that complete
contract; experimental enablement is a qualification condition, not a licence/eligibility veto.
Do not move native solvers or external side effects into the extension to reuse a transaction.

**Delivery witness.** Native and WASI execution return independently expected readiness actions,
wait reasons and refusal non-effects for the same typed facts, including missing predecessors,
changed revisions, cancellation and unknown effects. Test oversized aggregate input and cold
initialization/deadline overlap. Any transaction-access variant must expose its declared reads
and preserve publication/acknowledgement outcomes under concurrency. Blueprint §14/§20 and D10
placement changes require their normal decision route; codec-only simplification may stay within it.

<a id="O06"></a>
## O06 — Controlled operation access without duplicated business policy

**Recommend one typed operation model with optional native API projection; do not add a second
business execution path.** Existing direct callers and future inspection/automation clients
should invoke the same meaningful project/study operations and interpret the same outcomes.
A database-defined endpoint is worthwhile where it can replace a narrow actor ingress or make
a useful existing operation independently accessible with less client machinery.

Generate stored-function arguments/results and any `DEFINE API` projection from that operation
contract. Keep direct Rust/gRPC access as eligible physical realizations; adding HTTP is not
required for every command. For ordinary actor-scoped data access, declare invocation permissions
and `SECURITY INVOKER` explicitly. Deliberate privileged operations need a named authority and
constrained inputs instead of accidental definer execution. A binary body requires an actual
decoder; it is not automatic application Proto support.
[API reference](https://surrealdb.com/docs/reference/query-language/statements/define/api).

**Alternatives and choice.** Existing native methods may be the simplest interface for local
composition. Generated GraphQL/SQL can help inspection but do not expose all guarded domain
operations or rich value semantics. A custom external service is justified by a real isolation
or client requirement, not by the existence of HTTP. Recommend consistent typed operation
contracts now, with the database API adapter conditional on replacing an ingress or serving
a plausible independent operator workflow. Retire that superseded adapter and its duplicate
policy if the native route is selected.

Middleware, callable permission and table execution authority are separate. Handler rollback
must include the required enclosing effects; the existing patch's stronger rollback behavior
cannot be deleted merely because v4 documents handler atomicity. Network/file transfers and
native work remain outside SQL rollback. Deadlines, original-operation receipt lookup and
connection/SQL settlement survive adapter replacement.

**Delivery witness.** The same operation through chosen direct/API projections produces equal
receipts and effects for authorized, denied and malformed inputs. Include an earlier enclosing
write before a failing handler, intentional non-success responses, lost acknowledgement and
revoked actor authority. Endpoint reachability is not permission evidence. Changing blueprint
§20/§23 authority or commit contracts requires its decision route; interface generation within
an accepted contract does not independently authorize a new deployment.

## Discovery correspondence and boundaries

| Captured directions | Assessed disposition |
| --- | --- |
| Native isolation replacing incidental guards, complete procedure composition, read-only snapshots, retry ownership, unique constraints and sequences | O04 recommended explicit operation/isolation design; guard deletion and supplier retry substitution conditional on exact obligation coverage; sequences unsuitable for commit-ordered counters |
| Acquisition/policy/publication placement, JSON carriers, immutable init, ABI bindings, runtime/build patch replacement | O05 recommended typed pure policy composition; transaction extension/BINDINGS/patch removal conditional on specific complete benefit and contract evidence |
| Stored functions/closures and native API ingress, security invoker/definer, binary bodies, rollback | O06 recommended shared typed operation meaning; additional API adapter only where its complete consumer/ingress lifecycle earns it |

Sources include the discovery [transaction](../../library_leverage/libraries/surrealdb/briefings/transactions-and-guarded-state.md)
and [programming](../../library_leverage/libraries/surrealdb/briefings/database-programming-and-extensions.md)
briefs plus [runtime contracts](../../library_leverage/libraries/surrealdb/v4-runtime-contracts.md).
The broader review may combine these opportunities with observation, graph and delivery choices.
No performance, concurrency, ABI compatibility or physical-cancellation guarantee was newly tested.
