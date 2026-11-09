# Graph representation and traversal

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the current
dirty Plan66 working tree. Current uses are **Interface-checked** by representative source
reads. Local improvements are **Proposed**; no graph comparison or benchmark ran.

## Existing graph responsibilities

The [schema lowering](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal.rs) already
emits conventional relation tables, optional enforced endpoints, and selected vertex caches
(`INLINE EDGES 64`, `INLINE REFERENCES 32`). Its disposable generation-scoped family
also uses tracked record references with `ON DELETE REJECT` and same-generation assertions.
These are deployed uses of earlier capabilities, not first-adoption opportunities.

The [canonical explanation reader](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_explanations.rs)
uses explicitly indexed relation-table queries for study dependencies, run sources,
analysis inputs and analysis edges. It binds identities and cursors, orders by a stable
key, requests an extra row and retains source/evidence fields. A native graph label does
not imply that this reader currently traverses cached adjacency.

The [private knowledge reader](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/xtask/src/development/knowledge.rs) does
traverse `$root->(knowledge_subject_edges WHERE ...)` within a selected imported generation.
Its production lookup also follows typed reference fields. These two readers provide
different useful comparison workloads. Blueprint §20 owns authority and retention;
physical representation remains open to change for the same functionality.

## Candidate mechanisms

| Mechanism | Release/selection distinction | Concrete use to investigate |
| --- | --- | --- |
| Conventional relations and typed record links | Established; both used | Retain independent edge identity only where a consumer needs it. |
| Tracked references with explicit deletion policy | Established; selectively used | Replace manually coordinated inverse lookup or existence checks. |
| `LIGHTWEIGHT` relations | Introduced in 3.3; supported in v4 | Remove edge payloads and deduplication for truly propertyless endpoint-pair membership. |
| Vertex inline edge/reference caches | Introduced in 3.3; selectively generated | Improve batched low-degree traversal rather than assume all graph reads benefit. |
| Inline edge properties and graph-path predicates | Available in v4 reference | Avoid loading rejected edge records for small selective attributes. |
| Adjacency folding/packed blocks | Engine-maintained graph path | Reduce storage accesses without an application graph cache. |
| `REFERENCE ... MATERIALIZE INTO` | V4 reference addition; exact exposed contract needs confirmation | Replace repeated inverse collection assembly. |
| GQL pattern/path execution | V4 ecosystem; selected server closure leaves GQL disabled | Express an existing multi-hop explanation more directly if its complete contract fits. |

The official [table definition](https://surrealdb.com/docs/reference/query-language/statements/define/table)
is the primary route for relation/cache restrictions; the supplied v4 addendum §1
provides the detailed discovery leads. A timeout fetching that page in this pass leaves
its exact-release restrictions for focused source checking, rather than proving absence.

## Priority directions

**First: compare physical access for the existing explanation.** Take one study-neighbor
or analysis-source request and compare its indexed relation-table query with native
traversal from the same selected anchor. Keep authorized membership, order, pagination,
evidence attribution and completeness equal. Only then compare inline adjacency or an
inline small discriminator such as edge kind. This could remove secondary indexes,
record hydration or repeated round trips, rather than merely change query spelling.

Do not presume the traversal wins: edge payloads are required by several existing
explanations, a key-ordered range may favor the current index, and adjacency caches add
write amplification. A high-degree anchor can overflow the cache. The discriminating
question is which full operation performs less storage and transfer work at equal answers.

**First: reduce duplicated reference maintenance where identity is simple.** Current
generated same-generation record links offer a concrete starting point for native inverse
references. Compare them with explicit lookup indexes and copied scalar foreign keys.
The [reference documentation](https://surrealdb.com/docs/reference/query-language/language-primitives/record-references)
documents inverse traversal and explicit delete policies; default tracking alone does
not establish the required refusal or cascade behavior. An inverse lookup is not release
authority over an original protected payload.

**Next: classify relationship identity before trying lightweight storage.** A source
membership whose sole meaning is an ordered endpoint pair may eliminate its separate edge
record and uniqueness/deduplication logic. Analysis evidence, dependency kinds, provenance
and independent occurrences often require attributes or multiplicity and are poor blanket
targets. Classify each existing family from its writers and consumers, not its table name.
The first question is whether any current relation can lose its payload and independent
identity without losing information its consumers actually use.

**Next: compare materialized inverse fields against repeated incoming queries.** This
may suit frequently read low-degree parents. The countercondition is a hot referenced
record: maintaining a sorted inverse collection can increase contention, storage and write
work. The current official reference page did not contain `MATERIALIZE` in this read;
confirm tagged implementation and supported syntax before treating this lead as
Interface-checked. Compare index-based inverse lookup as the complete alternative.

**Conditional: use GQL for an existing multi-hop question.** Pattern, optional-path or
shortest-path syntax could replace caller-driven traversal and intermediate materialization.
Current one-neighborhood product reads do not alone establish demand for it. Check the
selected feature closure, supported grammar, authorization, occurrence identity, cycles,
path explosion and deterministic pagination. GQL and GraphQL serve different responsibilities.

## Discriminating evidence

Use low-degree and high-degree anchors, absent roots, duplicate endpoint occurrences,
mixed generations, deleted endpoints, denied records and edges carrying evidence. Compare
static plans first; execute analyzed plans only in an owned disposable fixture because
analysis performs work. Record actual cache/block/delta and payload-fetch counters where
reported; missing counters remain unknown. A final returned-row count is not scanned work.

[Plan68 C03](../../../../plans/68c-surrealdb-v4-query-improvements.md) owns the existing
comparison/status baseline. Its selected reader controls are inherited evidence, not fresh
measurements here. The proposed directions extend beyond that minimum compatibility lane.
Cross-link [query/index research](query-planning-indexes-and-search.md),
[derived-state research](derived-state-and-change-observation.md) and
[schema identity research](data-model-and-schema.md) rather than creating competing choices.

Coverage includes original v4 reference §4 and addendum §1; full-text/vector graph
composition belongs to the query brief. Broader graph algorithms, semantic graph meaning
and solver structure are not inferred from database graph support. No universal absence
claim or measured graph-performance improvement is established.
