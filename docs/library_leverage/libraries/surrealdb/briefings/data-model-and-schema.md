# Data model and schema

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283`, including the
current dirty Plan66 working tree. Representative local mechanisms are
**Interface-checked**; improvement directions are **Proposed**. No schema change,
database probe or product qualification was performed. See [coverage](../coverage.md).

## Current responsibility and representation

The [Surreal lowering](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal.rs)
derives schemafull tables, relation endpoints, field assertions, codecs and indexes from
registry declarations. It emits repeated literal unions for enums. `UInt64` lowers to
Decimal with an integral range assertion; a finite `Float64` lowers to an object with
authoritative eight-byte bits and a queryable floating projection. Nullable fields use
`option<...>`. Endpoint links are checked against declared scalar keys.

The [canonical codec](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_codec.rs)
checks actual native value kinds and preserves the distinction between string keys,
record IDs, semantic bytes, diagnostic bits and scientific finite values. These are
existing deliberate representations. A v4 scalar type is useful only if the complete
write, query, transport and read interpretation supports the intended replacement.

Blueprint §4 and §20 own current contracts. This discovery leaves their placement open
to redesign; it does not infer that a registry, handwritten adapter or current field shape
must remain merely because it already exists.

## Capability territory

| Mechanism | Origin and availability | Research relevance |
| --- | --- | --- |
| `DEFINE TYPE`, ordered unions and named object/scalar constraints | New in v4; schema definition facility | Consolidate repeated enum and structural database declarations. |
| Sized integer/float types, `bigdecimal`, date/time/timestamp/interval | v4 additions; end-to-end codec support differs | Replace representational work where the native type carries the actual required meaning. |
| `NONE`, `NULL`, optional/union types, field-scoped `FLEXIBLE` | Earlier foundations retained | Express absence and extensible metadata without weakening the entire record. |
| `VALUE`, `ASSERT`, `READONLY`, default and computed fields | Earlier foundations, stronger v4 effect rules | Delegate storage invariants at the point where state can change. |
| Named record-ID components and ordered composite keys | v4 extension to earlier structured identity | Align storage locality with existing revision/occurrence access. |
| Native field migrations and compatibility levels | v4 | Change representation without making permanent mixed-format readers the default. |

The official [named-type definition](https://surrealdb.com/docs/reference/query-language/statements/define/type)
documents reusable constrained types and declaration-ordered string unions. Such a union
can change ordering relative to ordinary strings; reducing repeated DDL is not sufficient
reason to change the ordering observed by a cursor or query.

## Priority directions

**First: generate a shared native domain vocabulary.** Emit named database types once
from the same semantic owner that produces native Rust vocabularies, then reference them
from fields and function arguments. This could remove repeated union rendering and reduce
schema/decoder disagreement. Keep transition legality separate: membership in a state enum
does not prove that a particular transition is allowed. A reusable constrained finite-value
object is also worth comparing with repeated nested field definitions.

The countercondition is material: casts can coerce values, enum ordering is meaningful,
and named-type introspection may not be represented by every selected SurrealKit schema
document. A shorter declaration can therefore change behavior or weaken tooling coverage.
The discriminating question is whether one generated vocabulary can drive installed
definition checking and exact Rust decoding without introducing another authority.

**First: compare native `u64` with the Decimal encoding as complete alternatives.** The
candidate is native unsigned storage plus generated native conversion, assertions and
protocol handling, with deletion of Decimal normalization/parsing and range DDL where
the replacement actually discharges those obligations. It is not a cast inserted into
the existing adapter. Check counts, generations, ordinals and identifiers separately.

The existing representation may remain preferable where supplier Protobuf conversion
does not preserve the new native kind, an inspector expects Decimal, or a query mixes
float and decimal values. [Transport/value discovery](native-values-transports-and-clients.md)
owns those codec questions. Boundary cases are zero, `i64::MAX`, the next value,
`u64::MAX`, overflow, fractional input, wrong kinds and round-trip equality.

**Next: eliminate duplicated key/reference facts selectively.** Current generated rows
often retain a scalar key and a checked record link. Compare one native record identity
with generated domain projection against the current pair. Structured keys might also
remove a secondary access index for entity-local ordered history. Preserve identity
meaning, rename behavior and occurrence multiplicity; physical key layout is replaceable.
The useful question is which actual reader avoids a conversion or index with the new key.

**Next: consolidate enforceable invariants.** Review generated assertions and Rust checks
for the uncertainty each protects. A native schema may eliminate repeated checks on
controlled writes, while readers of corrupt or differently interpreted data still need
an admission boundary. `FLEXIBLE` could simplify variable metadata already carried by an
existing record, but supplies no reason to introduce an unrestricted metadata product.

## Counterconditions and neighboring work

Do not replace authoritative IEEE bytes with a native float merely because it is narrower
or more direct. Signed zero, diagnostic payloads, finiteness and physical admission are
distinct requirements. Likewise, a database `timestamp` and the current microsecond
integer are not interchangeable until timezone, precision and interval semantics agree.

The [field reference](https://surrealdb.com/docs/reference/query-language/statements/define/field)
distinguishes stored normalization, defaults, assertions and read-time computation. New
definitions do not necessarily backfill existing records. V4 rejects arbitrary writes in
calculation/authorization clauses, so moving mutation into a helper does not justify it.
Use [rollout research](deployment-rollouts-and-migrations.md) for representation conversion,
dependent indexes and compatibility-level completion.

## Next discriminating work and evidence limits

Start with one repeated enum and one actual full-range unsigned field, tracing generated
DDL, installed definition, native binding, query predicate and decoded result. Compare the
expanded machinery of both designs, including schema inspection and migration. Then
inspect whether record-link duplication purchases any consumer behavior beyond validation.

The supplied [v4 reference](../../../../libraries/surrealDB/SurrealDB_v4.md), original §3
and addendum §§7–8/13, supplies breadth. Primary documentation was inspected on 2026-11-09;
it is a moving reference, not proof of every exact tagged conversion. The older
[parser/type account](../syn-and-sql.md) retains its original baseline and does not establish
v4 parser or rendering coverage. No repository-wide semantic census was prepared.
