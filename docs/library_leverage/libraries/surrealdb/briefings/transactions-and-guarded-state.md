# Transactions and guarded state

Inspected 2026-11-09 at `053ac1c93c84056456f60557dca73256ccb5b283` plus the current
dirty Plan66 working tree. Representative local guards, retries and supplier patch are
**Interface-checked**. Simplification and placement alternatives are **Proposed**. No
concurrency, fault-injection or recovery test ran during discovery.

## Existing functional contract and implementation

[GuardedScript](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_script.rs) composes
transaction framing, deadline checks, authority/selection bindings, stage fences,
postludes and typed result positions. Installation explicitly selects snapshot isolation;
other envelopes often emit an ordinary `BEGIN`. The
[maintenance generator](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal_maintenance.rs)
registers interpretation and lifecycle records with `FOR UPDATE` and coordinates admission,
drain and activation. Its comment correctly distinguishes commit-time validation from a
blocking row lock.

[Research publication functions](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal_research.rs)
combine selected scientific source, context, manifest, retention and acknowledgement facts.
[Project functions](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-codegen/src/codegen/surreal_projects.rs) guard
mutable branch/draft heads. These are existing database-local transactions; discovery
should ask which surrounding work can disappear, not propose a first stored procedure.

The [retry owner](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/transport_retry.rs) retries only
settled conflicts under the original enclosing clock. An uncertain effect must first be
reconciled by its owner. The
[transport adapter](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/crates/pse-operations/src/canonical_transport.rs) retains
response allocation and protected-selection custody across nested requests and tasks.
Transactional atomicity alone does not discharge those physical lifetime obligations.

## Distinguish upstream defaults from this deployment

The official [transaction guide](https://surrealdb.com/docs/learn/querying/concepts-and-guides/transactions)
documents snapshot as the general default, explicit serializable transactions on MX/DS,
and read-only snapshots. `FOR UPDATE` registers named records; serializable isolation
also checks read ranges. Implicit function/block atomicity does not itself establish
serializable predicate protection.

The selected [runtime patch](https://github.com/paul-heyse/pse-arrow/blob/053ac1c93c84056456f60557dca73256ccb5b283/scripts/surrealdb-source/surrealdb-v4-runtime-controls.patch)
changes unspecified write-transaction isolation to the datastore's ordinary policy and
documents serializable writes with current snapshot reads in the MX adapter. Therefore
neither upstream default prose nor the text `BEGIN` alone describes local isolation.
Any replacement must inspect the effective patched path or deliberately select an explicit
isolation level. This is also an opportunity to remove dependence on a customized default
if supported explicit selection carries the same complete behavior.

## Capability map

| Mechanism | Origin/availability | Leverage question |
| --- | --- | --- |
| Atomic statements, blocks and stored functions | Earlier foundation retained | Can one operation eliminate intermediate state and client orchestration? |
| Serializable isolation and explicit `BEGIN` selection | V4 public engine-dependent surface | Can native predicate tracking replace custom conflict-generation work? |
| `SELECT ... FOR UPDATE` | Existing named-record protection | Is explicit protection sufficient for the smaller decision footprint? |
| `BEGIN READ ONLY` | V4; adopted for private knowledge | Can genuine read snapshots avoid write-transaction overhead and accidental effects? |
| Automatic standalone conflict retries | V4 improvement; exact qualifying path matters | Can an existing caller retry layer disappear safely? |
| Native unique constraints and sequences | Established capabilities | Can native allocation/uniqueness remove application coordination with the same semantics? |
| Bounded-staleness reads | SurrealDS-specific | No reason to introduce staleness into exact publication or source selection. |

## Priority directions

**First: replace incidental conflict coordination with native isolation where complete.**
Select one branch-head or membership-dependent publication. Inventory its actual positive,
negative and range reads and the shared guard records it writes or registers. Compare
explicit serializable execution with the current generations and guard rows. Native read
tracking may eliminate blanket conflict keys, reduce unrelated serialization and make
the decision's true dependencies clearer.

The countercondition is important: generation/incarnation values may also carry lease,
stale-owner or semantic interpretation meaning. Serializable isolation does not replace
an expired authority fence or identify which logical attempt owns an acknowledgement.
Large read sets can cause more aborts. The discriminating question is which existing
guard exists only to make a database concurrency fact safe, rather than convey domain
meaning or original-operation identity.

**First: simplify a complete guarded database operation.** Compare the current script and
stored-function composition with one typed procedure returning the final receipt/result.
This may remove repeated statement construction, extra result slots or round trips.
Keep authorization and semantic owners explicit while changing physical placement.

A function call's transaction must have the required isolation. Early `RETURN` is normal
completion; it cannot substitute for failure/rollback. External files, solver/native work
and network effects do not become transactional by moving a call into a database function.
Compare API/extension execution in [programming research](database-programming-and-extensions.md)
and protocol completion in [transport research](native-values-transports-and-clients.md).

**Next: broaden genuine read-only snapshots selectively.** Private knowledge already
uses `BEGIN READ ONLY`; it is a starting example, not a proposed first adoption. Compare
other multi-query metadata reads with explicit read-only mode where they need one cut.
A protected result read may write/renew a pin or consume a ticket and cannot be classified
read-only by its public name. Separate pure selection from acquisition only when the
complete race and custody contract becomes simpler. Preserve current required freshness.

**Next: reconcile retry ownership instead of stacking retries.** Compare supplier
standalone retries with local explicit-transaction retries for one actual guarded effect.
The candidate deletes the redundant layer only where conflict classification, original
deadline and returned outcome remain correct. Preserve original receipt lookup after an
uncertain reply; a new attempt key is not conflict recovery. Check both upstream behavior
and the selected patched server, because typed failures drive local classification.

**Conditional: sequences for allocation, unique indexes for invariants.** The official
[sequence reference](https://surrealdb.com/docs/reference/query-language/statements/define/sequence)
describes batched unique allocation. Compare it only with identities whose semantics allow
gaps and allocation order distinct from commit order. Current history sequence/cut meaning
must be understood before replacing a transactional counter. A unique index may replace a
check-then-create lookup, but cannot represent arbitrary multi-record predicates.

## Next discriminating evidence and limits

Focused follow-up should use two independent writers to exercise lost head, absent lookup,
phantom range, disjoint updates and failed retry. Add lost commit acknowledgement,
deadline expiration during contention, schema epoch change and stale original custody.
Compare useful concurrency, not only whether an invariant test fails. Prove which old guard,
retry or script wrapper can be removed before crediting simplification.

Blueprint §20 and §21 own the current publication/custody meaning; breaking physical or
public interface changes remain eligible. [Storage research](storage-durability-and-artifacts.md)
must carry engine isolation differences into any MX/KV/DS comparison. Discovery creates
no new adoption or qualification status in Plan66/68.

Coverage includes original v4 reference §6 and addendum §§8–9. The older
[SurrealQL type account](../surrealql-server-constructs.md) preserves historical 3.3 research;
its contracts are not automatically refreshed by this document. Primary docs and local
patch/source reads establish candidates and relevant distinctions, not measured retry
rates, cancellation safety or assembled recovery of a redesigned implementation.
