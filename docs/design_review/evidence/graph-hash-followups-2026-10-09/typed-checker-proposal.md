# Typed authored checker domains

**Proposed — 2026-10-09.** This is a concrete candidate for the investigation owned
by [Plan 28f/N0](../../../plans/28f-shared-numerical-preparation.md#library-and-lifecycle-decisions).
It is not an adopted checker, an implementation authorization or a new authority
for canonical eligibility. The [measured authored-edit fixture](README.md#results)
found changed publication around 5.1–5.5 ms versus subsequent preparation around
1.3–1.7 ms. This identifies repeated upstream checking to address in the complete
typed design. It does not predict an incremental implementation's benefit. The
followup direction is to integrate correct optimizations unless they are clearly
substantially regressive; small timing regressions or concurrent repository test
noise do not exclude them. The existing full checker remains authoritative until
the complete replacement design is selected and independently qualified.

## Candidate boundary

Replace the current full `pse_modeling::check_with` implementation with composable,
typed checking operations, then invoke those same operations through the compiler's
existing Salsa ownership. Assemble the same complete `CheckedPackage` for the
existing catalog and downstream consumers. Preserve one authoritative checker;
do not add a parallel manual dependency graph or a positive-only result cache.

Keep `publish_modeling_revision`'s already-admitted path and exact unchanged
publication's fast path. Changed publication must first produce a completely
admitted package and then atomically select it. An error or cancellation leaves
the previous published revision usable with its current witnesses. Resource
reservation precedes construction and accounts for all retained query products.

## Input and query domains

| Domain | Candidate owning input and derived operation | Completeness requirement |
| --- | --- | --- |
| Declaration identity and source | Typed declaration input keyed by authored identity, including tombstoned absence; separate source-attribution input only if the source/witness contract permits it | Delete/recreate, kind changes, source positions and current diagnostic witnesses remain visible. A removed input cannot leave an old declaration available. |
| Inventory membership | Package and parent/kind membership inputs derived from the current complete inventory | Insert/delete, duplicate identities, sibling-name conflicts and owner-shape changes invalidate global constraints and consumed membership. |
| Name lookup and visibility | Namespace/scope input plus typed `lookup_name(scope, name) -> Option<DeclarationId>` and logical-reference operations | Track absent answers, visibility and shadowing as well as positive references. Changing a namespace must invalidate an earlier absent lookup. |
| Physical interpretation | Current quantity registry, units, preconditions and physical scope at their existing owners | A result depends on every physical meaning it consumes, including denied visibility. IDs alone cannot stand in for changed physical definitions. |
| Data documents | Existing typed document inputs, interpreted plans and admitted rows | Track positive/absent documents, membership and deletion; preserve document admission limits and failure behavior. The scalar probe did not qualify this domain. |
| Checked declarations | Pure typed semantic operations over declaration, lookup, physical and document products | Functions, schemes, entities, validity, contextual/temporal constructs, provenance and scientific knowledge must retain their current contracts. |
| Package-wide constraints | Complete typed edge/membership summaries consumed by duplicate/ownership, inheritance, call-cycle and related global checks | Per-declaration reuse cannot skip constraints that depend on the complete package or on a newly inserted edge. Preserve current refusal attribution and deterministic diagnostic precedence. |
| Complete package assembly | Existing `CheckedPackage` meaning assembled from all required checked products and global checks | The output remains sufficient for every current selected/specialized/executable consumer; downstream equality cutoff is preserved. |
| Adjacent execution context | Provider, policy and structural-demand inputs remain at the existing consumer that reads them | Do not invent these as checker dependencies where the pure checker does not consume them, or omit them when a derived check/consumer does. |

Names above describe contracts for review, not committed Rust type or query names.
Start with the current complete inventory input where a narrower domain's
completeness is unsettled. Returning equal typed lookup/membership products allows
Salsa backdating without asserting an incomplete premise set.

## Design questions that must be settled

1. Which existing `check_declarations` passes can be extracted as pure typed
   operations without changing diagnostic precedence or their access to complete
   namespace, membership and physical context? Global cycle and inheritance
   validation need explicit complete summaries, not a graph inferred from successful
   local checks alone.
2. Can source attribution be separated from semantic results while errors always
   report the current source witness? The span-edit measurement motivates this
   question; it does not authorize ignoring expression spans in equality or reusing
   stale `Located` errors.
3. Which products are small and stable enough to retain, and what checked bounds
   cover initial construction, all memoized products, LRU eviction, cancellation
   and failed-publication intermediates? A query counter alone cannot establish
   memory or execution savings.
4. How will data-document admission remain authoritative and budgeted when checking
   operations become tracked? Existing document inputs already use Salsa; wrapping
   the full checked package in another query does not by itself remove document or
   authored admission work.
5. What are the complete publication and subsequent preparation costs after
   package assembly, source rebinding, dependency validation and accounting are
   included? Compare complete operations on representative real editing workloads,
   including ones dominated by selected mathematical changes. Use that evidence
   to detect clearly substantial regressions and improve integration; a required
   minimum speedup is not an adoption condition.

## Independent acceptance

During development, use the existing full checker as an independent test oracle
for equivalent complete authored inputs. Compare accepted `CheckedPackage`
meaning and classified failures with current witnesses across unrelated and
selected edits, edit/revert, deletion/recreation, absent-to-present references,
scope visibility changes, duplicate/owner changes, cycles, physical changes,
document admission and consumed provider/policy/structural context. Include limits,
cancellation and failure atomicity. This oracle is a test route, not a second
production checker to retain after replacement.

Measure publication plus subsequent preparation using stable artifacts and
matched functional evidence. Include key/domain construction, lookup validation,
assembly and allocations. If full typed extraction produces no repeatable complete
operation benefit, that alone does not exclude a correct complete replacement.
Slight regressions and concurrent test noise are also not exclusions. Investigate
clearly substantial regressions before adopting the affected integration, while
preserving every checker contract. Local hashing, key reuse and complete typed
checking can compose in the existing compiler ownership; none substitutes for
complete tracked premises or independent correctness evidence.

The current canonical selection path continues to recheck complete positive and
absent name/logical premises, scope/kind/reference membership and interpretation
under a fresh protected read. Finer compiler checking cannot bypass or silently
weaken that receiving contract. Any changed durable interpretation or new checker
kernel contract follows the existing decision/design route before implementation.
