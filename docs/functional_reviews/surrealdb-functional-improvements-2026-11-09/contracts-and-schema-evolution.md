# Contracts and schema evolution

Reviewed 2026-11-09 against `4f07766942ff3b21274fdfb686ad483031439462` and the
concurrent working tree. The declaration/codec, SurrealKit preparation and frozen-rollout
owners inspected here are unchanged from that revision. Current mechanisms and selected
supplier contracts are **Interface-checked**; recommendations and benefits are **Proposed**.
No generator, migration, runtime probe or benchmark ran. The [principal review](../surrealdb-functional-improvements-2026-11-09.md)
owns integrated dispositions; the anchors below provide their detailed argument.

Assessment uses core 3.4, efficient architecture 1.0, process-simulator 1.5 and the
repository binding. AP-04/DP-01 govern meaning, AP-02/DP-24 evolution, and AP-07/DP-15
the complete representation and tool lifecycle. Scientific fidelity under PS-01/PS-13
is separate from a schema or codec agreeing with itself.

<a id="O01"></a>
## O01 — One generated native contract vocabulary

**Recommend shared named constraints and policy-aware conversions; condition native unsigned
replacement on the whole boundary.** Authors should change a declared vocabulary or storage
contract once and obtain consistent database fields, callable arguments, native carriers and
inspection. A developer inspecting a rejected value should see the same domain name and
meaning at each of those boundaries, rather than several independently rendered unions.

The [current lowering](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-codegen/src/codegen/surreal.rs)
already derives schema and codecs together. It emits literal unions repeatedly, `UInt64`
as integral Decimal, and finite floats as authoritative bits plus a query projection.
The [codec](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_codec.rs)
checks exact native kinds. This is a coherent starting authority, not a reason to add
another editable model or to declare all repeated checks redundant.

Generate `DEFINE TYPE` names for domain vocabularies and reusable constrained shapes, with
fields/functions referencing them. Keep transition legality and context-specific cross-field
assertions with the owning operation. V4 named types can constrain callable values as well
as stored ones; their coercion and declaration ordering require deliberate interpretation.
[Named-type reference](https://surrealdb.com/docs/reference/query-language/statements/define/type).

For unsigned counts, generations and ordinals, the preferred candidate is native `u64`
storage plus generated direct tagged-wire conversion, deleting Decimal parsing/range DDL
when that complete route works. The released native Proto conversion widens numbers through
`Number::wide`, degrades temporal types and can emit BigDecimal as text. Therefore replacing
`encode_uint` with a convenience native conversion is specifically **not** the recommended
implementation. The current Decimal representation remains the interim complete choice until
the selected outbound/inbound route preserves the required domain. Keep exact IEEE bits
where identity or signed zero matters; do not exchange them for a query float.
[Tagged v4 conversion](https://github.com/surrealdb/surrealdb/blob/v4.0.0/surrealdb/types/src/proto/value.rs).

Generate native conversion from declared policy rather than route every value through generic
Serde or a new descriptor interpreter. Consolidate key/link duplication only where one native
record identity plus generated projection serves all consumers. Ordered composite IDs can
replace an index for entity-local history, but must earn their persistence coupling through
a real lookup pattern. Field-scoped `FLEXIBLE`, native `VALUE`/`ASSERT` and explicit
NONE/NULL contracts can simplify existing metadata without making all tables schemaless.

**Alternatives and removal.** Keeping all current literal/Decimal lowerings is lowest
transition cost but leaves database-native vocabulary underused. A Proto-first database
model would add interpretation and graph/index policies not supplied by descriptors. Prefer
owner-derived native contracts; remove superseded literal rendering, conversion branches and
duplicate identity fields only after every selected consumer moves.

**Delivery witness.** One authored vocabulary changes across field/function/inspection without
independent edits. Independent malformed inputs exercise coercion, optional/nullable values,
enum ordering and invalid transitions. Unsigned cases include zero, `i64::MAX`, its successor,
`u64::MAX`, fractional/wrong-kind input and overflow through write/query/read. Temporal changes
must preserve authored precision and interval conventions rather than infer them from a type name.
Blueprint §4/§5/§20 representation changes need their normal decision route if contracts move.

<a id="O02"></a>
## O02 — Engine-normalized contract inspection and operation-shaped generation

**Recommend a strict semantic cross-check; retain one source-owned production generator.**
The desired developer workflow is: change the contract, inspect what the selected engine
understands, see a precise disagreement, and use operation-appropriate inputs. This improves
feedback and inspectability even when no generator bug has yet been demonstrated.

The existing generator hashes DDL, codec interpretation and refusal classification. SurrealKit
rc.1 adds engine-backed `compile`, live `typegen`, schema document v2 structured kinds/presence,
diagnostics and `semanticDigest`. Use live introspection of the selected patched server as
the installed comparison, and explicitly prepared embedded compilation as faster local
feedback where equivalent. The [prepared tool](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-host/src/surrealkit.rs)
currently selects `cli` without an embedded backend; offline compilation needs a deliberate
feature/build identity change, not an assumption that its command name makes it available.

V2 is draft and version-qualified. Its represented schema and digest do not cover the complete
application codec/history contract. Use strict diagnostics and explicitly report unrepresented
definitions, notably named types/extensions; do not equate lenient output with complete agreement.
The compiler and installed server must be compared at the selected engine/patch context.
[V2 contract](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md),
[compiler](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs).

Generate create, merge and replacement inputs from the existing owner where those operations
have different omission/default/removal semantics. Keep database-derived read shape distinct
from a writable command. This can remove repeated interpretation in existing writers without
making SurrealKit's schema document a competing semantic authority.

**Alternatives and removal.** Replacing the whole generator with an external Rust emitter is
not presently the best default: no Rust emitter is bundled, and the source owner already
generates exact codecs and multiple non-database consumers. External emission remains a useful
choice for a new downstream projection that benefits from v2. Its manifest, stale-file removal
and `--check` can replace a publication owner only for disjoint outputs; do not wrap the
current generator with another owner for the same files. Per-file replacement is not atomic
publication of an entire directory. [Emitter protocol](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md).

**Delivery witness.** The developer sees a deliberately introduced kind/presence/permission
disagreement, including a definition v2 cannot represent. Compilation and live introspection
agree on the represented subset; unsupported coverage remains visible. Existing operation
inputs preserve NONE/NULL/omission and exact values. Remove only normalization/parsing and
file-publication logic that the chosen supplier route actually replaces. AP-06 requires an
independent semantic expectation; comparing two projections from one generator is insufficient.

<a id="O03"></a>
## O03 — Owned schema transitions with native stored-value migration

**Recommend a typed transition capability using supplier rollout/migration machinery, with
fresh reconstruction remaining a first-class alternative.** Operators should be able to
preview an exact change, understand its data effect, apply it to the intended issuer, and
recover the original operation after interruption. This enriches the existing preview and
schema lifecycle; it does not require a universal arbitrary-SQL migration service.

[FrozenRollout](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/src/canonical_maintenance/frozen_rollout.rs)
currently admits bounded hashed definitions/removals and records an independent-data-witness
requirement. The selected preview binds listener, catalog, extension and tool identity. Retain
that authority while delegating frozen history, chain lint, ordered catch-up and tracking
recovery to SurrealKit rather than maintaining a parallel history interpretation.

For field changes fitting v4's supported transformations, use named `MIGRATE EXISTING ROWS`
with explicit value policy and a completion barrier such as `await_migrations`. SurrealKit
rc.1 checks migration identity, removes blocking indexes and restores them. Its migration
partition is distinct from ordinary definition identity. This saves a bespoke scan/rewrite
loop, but creates a uniqueness interval when a removed index enforced uniqueness: close or
appropriately coordinate writers, then establish data conformance and rebuilt-index readiness
before cutover. A fallback value is a scientific/domain decision, not an automatic repair.
[Tagged adapter](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/migrate.rs),
[migration notes](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md).

Keep compatibility inspection/raising, definition submission, background rewrite completion
and consumer activation separate. Admit data migrations as explicit effect-bearing operations
instead of widening the existing schema-only manifest invisibly. Modules/targets can provide
independent reconciliation partitions; seeds need tracked identity and idempotency. Bind
effective substituted bytes if variables become useful, since a source hash alone does not
identify deployed SQL. Retain current frozen explicit input when substitution adds no value.

**Alternatives and choice.** Reconstruct fresh reproducible design-phase state when cheaper
and semantically safe; do not build a historical migration estate merely to preserve disposable
data. Use native in-place conversion when retained originals or an actual ongoing deployment
benefit. Explicit backfill remains appropriate for cross-field or dependency-restricted
transformations. Reverse steps do not resurrect discarded information; unknown acknowledgement
requires original-operation reconciliation before any replay. Multi-target rollout is not a
distributed transaction. Compatibility shims are not a default design requirement.

**Delivery witness.** Change one real field representation with protected/read consumers;
demonstrate forward conversion, repeated named application, changed-name refusal, interruption,
unknown outcome, duplicate-value index refusal and independent old/new data witnesses. Destructive
undeclared-field cleanup needs an explicit loss argument. Recurring configured apply is currently
deferred in the operating guide; adopting this proposal changes that scope through plan creation
and the applicable blueprint §20/§24 decision route, not through this review.

## Discovery correspondence

| Captured directions | Assessment home |
| --- | --- |
| Named types, unsigned/numeric/temporal fidelity, presence, flexible metadata, stored invariants, record-key/reference duplication and locality | O01; recommend shared vocabulary and policy-derived conversions, conditional complete native representation replacement |
| Engine-normalized schema, live/embedded interpretation, v2 coverage/digest, mutation-shaped inputs, emitter/managed publication | O02; recommend semantic feedback and owned inputs, conditional external emitter for a useful downstream projection |
| Sync/modules/targets/seeds, frozen history/repair, effective variables, migration/conformance/compatibility and destructive cleanup | O03; recommend bounded transition capability and per-transition reconstruction/native/backfill choice |

All directions include earlier capabilities retained in v4. The source briefs are
[data/schema](../../library_leverage/libraries/surrealdb/briefings/data-model-and-schema.md),
[compilation/generation](../../library_leverage/libraries/surrealdb/briefings/schema-compilation-and-generation.md)
and [deployment/migration](../../library_leverage/libraries/surrealdb/briefings/deployment-rollouts-and-migrations.md).
No performance, installed-tool, rollback or assembled qualification claim follows from inspection.
