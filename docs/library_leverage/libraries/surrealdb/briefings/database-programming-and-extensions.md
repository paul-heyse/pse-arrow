# Database programming and extensions

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the current
dirty Plan66 tree. Representative code and selected supplier interfaces are
**Interface-checked**; local improvements are **Proposed**. No extension build, invocation,
rollback control or performance measurement ran. Execution placement is open to redesign.

## Existing operation and its meaningful boundary

The [study module](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-study-module/src/lib.rs)
already executes the shared Rust readiness policy inside an invoking database operation.
Typed `SurrealValue` carriers describe a bounded page, but policy, facts and outcomes within
it are JSON bytes. The code preflights aggregate content before decoding, applies
`pse_model::study_policy::candidate_action`, and returns typed decision carriers with some
encoded results. The enclosing database operation owns state mutation and acknowledgement.

[Guarded scripts](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_script.rs)
compose authority binding, transaction framing, protection rechecks and result slots.
[Server patches](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/scripts/surrealdb-source/source-build.md)
add extension build provenance and physical runtime admission/deadline behavior beyond ordinary
supplier defaults. Study-module preparation, original receipts and deployment are existing
owners. This is an improvement inquiry into an established extension deployment, not a first
proposal to move any Rust into the database.

Blueprint §20 and §14 retain current scientific meaning and operation ownership. A better
placement may change the representation or framing without moving native solver state into
a transaction or requiring historical API compatibility.

## Capability families and version relevance

| Mechanism | Origin / availability | Design significance |
| --- | --- | --- |
| Typed stored functions, closures and parameterized calls | Pre-v4 foundations | Named bounded procedures can replace repeated caller-side query construction where they express the complete operation. |
| Synchronous events | Existing native use | Maintain a write-owned effect atomically; separate from caller-owned procedures. |
| Surrealism WASI execution | Existing deployment, v4 lifecycle/ABI change | Reuse native Rust policy in database-managed execution with call-scoped transaction access. |
| Build-time `init` and fresh instances | V4 execution model | Amortize immutable initialization while removing mutable warm-instance assumptions. |
| Explicit `Transaction`, typed write access | V4 | Gather selected state and apply a policy result within one enclosing transaction, reducing intermediate marshaling. |
| Typed `BINDINGS` | ABI-5 lead in supplied addendum | Per-definition configuration without rebuilding the same policy component; selected build support needs checking. |
| Signed/digested archives and capabilities | V4 extension definition | Supply immutable code/config identity and scoped permissions through native declarations. |
| `DEFINE API`, binary request bodies | API predates v4; authority/rollback enhanced | A native operation endpoint may replace custom ingress for an existing consumer. |

The [reusable contracts](../v4-runtime-contracts.md) separate call-local extension state,
transaction participation, resource controls and API authority. Those facts constrain candidate
compositions; they do not establish that any local wrapper can already be deleted.

## Priority compositions

**First compare data acquisition, policy execution and guarded publication as one operation.**
The current design moves a bounded fact page into pure Rust policy, then consumes its decision
in surrounding SQL. With transaction access an extension could gather only the facts required
by that policy and apply the resulting guarded change itself, preserving shared Rust meaning.
Compare this with a stored SurrealQL procedure around the existing pure extension and with
caller-side Rust plus one selected transaction. Potential removals include intermediate page
objects, duplicate shape checks and repeated argument/result conversion. The best choice
depends on selected fact volume, policy reuse, query optimization and transactional exposure.

**Compare owner-derived typed data with JSON-in-bytes.** Generate a compatible native carrier
or portable message mapping from the current policy owner, retaining explicit numeric/presence
semantics and bounded admission. This could remove repeated JSON encode/decode and checked
Decimal-to-integer adaptation. A general descriptor interpreter is justified only if many
actual schemas need it; compiling one fixed local policy can be simpler. Decoder compatibility
inside WASI and build-time snapshots must be established for the chosen library.

**Move stable initialization to the archive where it is real work.** Immutable descriptor,
lookup or policy metadata can potentially be prepared by `init`. Compare build-time snapshot
size, startup cost and per-call allocations against current decode. Per-invocation `BINDINGS`
may carry database-specific identity/configuration; it cannot replace durable acknowledgement
or a mutable inter-call cache. Prepared metadata must have an explicit interpretation identity.

**Reassess stored procedures and native API ingress together.** Existing project/control
operations can be expressed as a typed named procedure, potentially eliminating repeated
string assembly or a separate narrow HTTP actor. Compare invocation permission, effective
table authority, original-operation identity, complete rollback and server/client settlement.
`SECURITY INVOKER` is useful when caller permissions should govern the operation; a deliberate
privileged endpoint needs a different explicit authority contract. No new API consumer is
invented merely to exercise the feature.

## Reasons to retain a smaller integration

Pure bounded policy keeps database access visible in SQL and allows the same Rust decision
function to be exercised outside WASI. Transaction-capable extensions can hide too much access,
reduce optimizer visibility, lengthen transactions and increase module memory pressure.
Build-time preinitialization can enlarge every fresh instance when metadata is rarely used.

Resource limits do not prove the original deadline is honored through queueing, preparation
and physical runtime release. Native defaults may wait for admission while the selected
patched deployment requires a different refusal/custody policy. The retained patches remain
eligible for replacement only when the public mechanism covers the complete required behavior.

Extension execution is still documented with experimental enablement. Network access is
capability-scoped and external side effects do not roll back with database writes. A WASI HTTP
interface does not establish support for arbitrary Tonic/native solver dependencies. SurrealML
removal means it is not an available alternative inference subsystem.

## Next discriminating questions and evidence

1. Which bytes/objects/calls disappear in one current readiness claim if acquisition and policy
   are composed differently? Follow both accepted and refused decisions, including no effects.
2. Can generated native carriers replace JSON while preserving every current policy outcome,
   unknown-effect precedence, original revisions and aggregate allocation admission?
3. Does selected Surrealism 0.7.0 plus the prepared builder actually emit ABI 5 when requested,
   and how does definition configuration bind to policy interpretation and artifact identity?
4. Can public runtime admission/deadline APIs replace any patch while retaining overlap,
   timeout, cancellation and physical Store-release behavior?
5. Does a native handler/procedure remove a deployed ingress lifecycle, or merely add another
   invocation route that still needs all existing custody?

Selected primary [extension architecture](https://surrealdb.com/docs/learn/extensions/guides/extension-architecture),
[extension definition](https://surrealdb.com/docs/reference/query-language/statements/define/extension)
and [API reference](https://surrealdb.com/docs/reference/query-language/statements/define/api)
were inspected. The full ABI/bindings and schema-clause restriction details remain leads from
the supplied [Protobuf addendum](../../../../libraries/proto/Protobuf_SurrealDB_v4.md), requiring
tagged implementation checks for a selected design. The [v4 overview](../../../../libraries/surrealDB/SurrealDB_v4.md)
also distinguishes stored-function, event and API authority. Further work belongs to focused
applicability/comparison with the [transaction](transactions-and-guarded-state.md) and
[schema/deployment](deployment-rollouts-and-migrations.md) briefs, not automatic extension expansion.
