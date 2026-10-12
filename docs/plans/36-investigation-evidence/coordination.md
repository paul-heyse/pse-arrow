---
title: Plan 36 coordination investigation evidence
status: draft
date: 2026-10-11
doc_owner: docs/plans/36b-studies-coordination-and-results.md
doc_topics: [architecture, preparation, storage, validation, environment]
---

# Coordination investigations

This evidence supports [36b](../36b-studies-coordination-and-results.md); Plan 36 alone owns finding dispositions. Source baseline `2d6bf6ae7`, inspected 2026-10-11. The assigned effects are investigation and plan authoring, not product implementation. Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 govern the bounded assessment. **Interface-checked** denotes exact source/API inspection below; **Tested** denotes the explicitly described disposable controls. All replacement interfaces remain **Proposed**. No original finding closes here.

## Exact canonical authority and settlement

Inspected owners: operations `canonical.rs::{protected_query,product_acknowledged,release_protection}`, `canonical_execution.rs::{result_seed_candidates,renew_attempt}`, `canonical_results.rs::{latest_results,read_results}`, selection/staging/analysis/retention adapters; codegen `surreal_execution.rs`, `surreal_retention.rs`, `surreal_studies.rs`; schema `catalog/substrate.rs`. Generated files were read only to identify their schema consumer, never edited. Current problem/study touches serialize genuine and incidental effects together. The exact replacement is a transaction capability carrying immutable store/problem/run/study association plus current attempt/generation; current mutable authorities remain server-checked on every effect.

Context7 resolved `/surrealdb/docs.surrealdb.com` and queried SELECT FOR UPDATE and the local RocksDB CLI. The official [transaction documentation](https://surrealdb.com/docs/surrealql/transactions) describes optimistic record registration. Exact-release authority is `surrealdb-kvs-rocksdb-3.3.0/src/lib.rs::getu`: `get_for_update_opt(key,true,&self.read_options)` uses the transaction snapshot. It registers an absent exact key as well as a present key; it does not register predicate phantoms. Two readers of unchanged cancellation authority writing disjoint attempts can commit. Every run association must consequently be immutable after registration or have a named absent association key registered by its readers/creator. Summary runs and study claim/start/observe/finalize are included; removing only the append writer touch is incomplete.

Select the following semantic interfaces within existing operations owners, not a second coordinator:

- `EffectContext`: immutable exact store incarnation, problem, run, optional immutable point/study association, current attempt and generation, absolute expiry/clock, and operation identity. Server functions validate every supplied identity; possession alone grants nothing.
- `SourceAdmission`: registers reclaim authority and exact root/pin key including absence, checks reclaimed intervals, creates root/pin and writes the source creator fence in one transaction. Collection registers creator fence before scanning, registers affected exact live pins/roots, then writes reclaimed intervals and reclaim authority atomically. Creator-first invalidates the collection scan; collector-first invalidates creator admission. Root release, product/stage activation, source revision creation and analysis roots use that same protocol.
- `ResultAdmission`: the analogous attempt/manifest/result-creator versus monotone retirement protocol. Analysis-input installation is a creator, so retirement cannot miss a new borrower. Renewals register exact pin plus retirement/reclaim authority and refuse expired/released pins; they never revive them.
- `EffectKnowledge = Acknowledged | DefiniteConflict | Uncertain(identity)`: rebuild only a definite conflict, releasing local pacing and backoff permits first. Resolve uncertainty by its original receipt/fence under the original clock, without blindly replaying science or a mutation. Timeout/disconnect is not noncommit or drain evidence.
- `GrantSettlement`: stable incarnation, original grant nonce/deadline, exact absent-or-present grant registration, and a monotone closed-through/replay floor. Settlement observes/drains any prior committed grant and advances its floor atomically before removing grant residue. A stale registered acquisition conflicts; a newly submitted acquisition checks the durable floor and refuses. Restore must preserve that floor or create a new incarnation. Wall-clock age alone cannot collect a tombstone.

Cancellation is ordered by commits: append first can remain a valid retained prefix; cancellation first invalidates/refuses append, heartbeat, claim and close/seal. Joined complete original export can remain original evidence under a cancelled outcome; an incomplete prefix confers no science/seed permission. Local pacing follows actual write domains with stable cross-owner ordering, never hydration/native work/backoff. Original absolute clocks include all retries and settlement.

## Disposable RocksDB control, 2026-10-11

**Tested:** `scripts/pse-env -- .venv/bin/python /tmp/plan36-coordination-probe.py`, exit 0, zero unexpected query/control failures. The script exclusively created a temporary RocksDB directory, a loopback server at an OS-selected port, namespace/database `plan36/probe`, and joined that exact subprocess with exit 0. No canonical deployment or shared cache was used. Binary: `/home/paul/.local/share/pse-arrow/tools/surreal/v3.3.0/linux-amd64/surreal`; `surreal version` reported 3.3.0. The first setup attempt failed because the explicit namespace/database did not exist; the corrected script creates both before fixtures. That setup failure is retained as a failure, not a passing control.

Canonical reproducibility: [probe source](coordination-probe/probe.py), [full query/EXPLAIN/race receipt](coordination-probe/receipt.json), [server output](coordination-probe/server-output.txt). The published source only parameterizes the installation location with `PSE_PLAN36_SURREAL_BINARY`; the tested receipt retains the exact path. Run `scripts/pse-env -- .venv/bin/python docs/plans/36-investigation-evidence/coordination-probe/probe.py`; it creates and drains a new exclusively owned temporary fixture. Never point it at a canonical store.

Original disposable sources/receipts: `/tmp/plan36-coordination-probe.py`, `/tmp/plan36-coordination-z9xhjctu/receipt.json`, `/tmp/plan36-coordination-final.log`. The substantive queries/results are retained below; these temporary artifacts are not a new permanent receipt system.

| Bounded control | Observed result |
|---|---|
| Register absent `race:absent`, sleep, create a disjoint record; concurrent creator writes `race:absent` | Reader transaction COMMIT returned `TransactionConflict`; the exact absent-key premise was protected |
| Two transactions register unchanged `race:fence`, then create disjoint records | Both committed |
| Creator reads reclaim/exact absent root, writes root/creator fence before collector commits | Creator committed; collector's registered creator fence conflicted |
| Collector reads creator fence/scans roots, writes reclaim before creator commits | Collector committed; creator's registered reclaim fence conflicted |
| Append registers cancellation, commits before cancellation update | Both committed; earlier append remained |
| Cancellation commits before registered append | Cancellation committed; append conflicted; no append remained |
| Settlement registers absent grant and advances floor before acquisition commits | Settlement committed; acquisition conflicted |
| Fresh acquisition after floor advance | Explicit `closed grant` refusal |

For race controls the script issues concurrent HTTP transactions with 100/700 ms or 200/800 ms overlap windows. Observed conflicts establish overlap in this run; this is not systematic scheduler/model checking. The tests exercise abstract proposed authority rows, not generated canonical functions, authenticated protocol capabilities, clock exhaustion, restore or native drain. Those remain mandatory P-B1/P-B7 implementation controls.

Representative queries (the inverse commit order changes only delays):

```sql
BEGIN;
SELECT * FROM ONLY race:reclaim FOR UPDATE;
SELECT * FROM ONLY race:root FOR UPDATE;
SLEEP 100ms;
CREATE race:root SET v=1;
UPDATE race:creator SET v=1;
COMMIT;

BEGIN;
SELECT * FROM ONLY race:creator FOR UPDATE;
SELECT * FROM race WHERE id=race:root;
SLEEP 700ms;
UPDATE race:reclaim SET v=1;
COMMIT;
```

## Access effort, preparation handoff and seed completeness

The isolated fixture reproduces existing index/query shapes with schema-compatible surrogate rows: 128 current memberships plus 1,280 superseded rows (ten old versions each), 1,000 runs with 100 terminals, 65 seed headers and one eligible old attempt, and 1,000 exact acknowledgement rows. It does not fabricate an admitted scientific seed or qualify preparation content. Its EXPLAIN metrics are operator-output counts, not hidden RocksDB key reads or RSS. `records_fetched=0` does not mean zero records were examined.

| Query shape / index | Returned | Reported index output | Decision |
|---|---:|---:|---|
| Current membership interval, scope/key order, LIMIT 64 / `membership_scope_pages` | 64 | 768 | Retain interval semantics; returned bound does not bound history effort |
| Historical sequence 5, same shape/index | 64 | 768 | Preserve historical selection; index cannot simply become current-only |
| Proposed registered current-head, scope/key order, LIMIT 64 / `membership_scope_head_pages` | 64 | 64 | Select guarded head-only acceleration; historical requests retain interval path |
| Exact version plus interval / `membership_version` | 1 | 1 | Use exact membership where the consumer already knows the version |
| Latest terminal result, sequence DESC LIMIT 1 / `run_order`, Backward | 1 | 24 | Existing planner pushes ordered scan; no unconditional sort-removal rewrite is justified |
| Exact proposed descriptor/store acknowledgement / `RecordIdScan` | 1 | Exact record | Select exact lookup; preparation owner retains full content/root identity validation |
| Proposed unique `(problem,descriptor,store)` acknowledgement index | 1 | 1 | Viable alternative when a deterministic record ID is unavailable |
| Existing seed selector LIMIT 64, no eligibility predicate | 64 | 65 | Old eligible seed was absent from the returned page |
| Attempt eligibility dereference before LIMIT | 1 | 65 | Old eligible seed was selected; correct semantics alone do not prove bounded examined effort |

Both membership indexed queries produced exactly the same ordered rows as `WITH NOINDEX`. Exact current/historical query:

```sql
SELECT * FROM canonical_memberships
WHERE problem='p' AND scope='s' AND key>''
  AND from_sequence<=10 AND (to_sequence=NONE OR to_sequence>10)
ORDER BY key LIMIT 64 EXPLAIN FULL;
```

For current membership the final reported tree was `IndexScan(membership_scope_pages,Forward,output_rows=768) -> Filter(output_rows=69) -> Limit(64) -> SelectProject(64)`; historical used 768/70/64. An earlier current fixture execution reported 1,024 index-output rows, so these batched operator counts are observations, not an exact scan-cost contract. Latest result was `IndexScan(run_order,Backward,output_rows=24) -> Filter(1) -> Limit(1) -> ProjectValue(1)`, without a separate sort. Adding a deterministic seed-key tie breaker produced a `SortTopKByKey` in this fixture; a proposed index must cover the full actual ordering before claiming its removal.

The additional proposed indexes are `(problem,to_sequence,key)` and `(problem,scope,to_sequence,key)`. With both available, the no-hint scoped head query chose `membership_scope_head_pages`; forcing that scoped index produced the same plan: `IndexScan(access=['p','s',NONE] MoreThan '',Forward,limit=64,output_rows=64) -> SelectProject(64)`, without Filter or Sort. The head page equals the current interval page and its `WITH NOINDEX` control, in original key order. This is a concrete supported fixture path, rather than an assumption that adding an index eliminates history work. The unscoped index was declared but not independently exercised by an unscoped query in this run.

```sql
BEGIN;
LET $head = SELECT * FROM ONLY canonical_problems:p FOR UPDATE;
IF $head.sequence != 10 { THROW 'head changed'; };
SELECT * FROM canonical_memberships
WHERE problem='p' AND scope='s' AND to_sequence=NONE AND key>''
ORDER BY key LIMIT 64;
COMMIT;
```

Select head-only acceleration only when the requested revision equals the registered authoritative head in the same transaction; revision writers must update that exact head authority, so an overlapping revision invalidates this reader at commit. A changed head retries under the original deadline or uses the historical interval query; it must never silently substitute the new head. Retain historical interval queries and their exact protection/admission checks. An equality-success transaction was executed; an emitted protected-page revision race and authentic 64-row source-selection payload remain product acceptance, not inferred from the separate absent-key race. Preparation owns schema/index declaration, byte bounds, keyset continuation and the actual source page consumer. The fixture demonstrates operator limit pushdown under ten superseded generations, not hidden storage-key reads, wall-time scaling, or a general planner guarantee.

The successful seed query used `type::record('canonical_attempts',attempt).terminal_eligible=true` before LIMIT; the original selector lacks that predicate. `terminal_eligible` in the fixture represents a proposed seal witness, not a current field. The existing result-seed index contains layout/preparation/backend/run-sequence/generation/step, not that witness. Select correctness first: one snapshot filters authoritative seal eligibility, retirement and exact compatibility before cutoff, with seed key completing the total order. Do not implement one seal-time write per seed. A per-attempt witness refers to its frozen inventory; a seed has to prove its particular complete-step membership on protected admission.

Select an indexed eligible-attempt discovery relation if populated evidence shows the dereference scan is ordinary limiting work. Its activation is transactional with the attempt seal and contains bounded compatibility partitions plus frozen-inventory identity; all headers for one attempt are not assumed to have identical layout/backend. If the number of distinct partitions exceeds the admitted manifest bound, seal refuses before effects or uses bounded staged activation under the same close identity. This relation is derived discovery only; current protected admission checks retirement, manifest and individual completion permission. A keyset fallback must distinguish exhausted absence from incomplete discovery under the original clock. The fixture has not qualified that relation or a joined production selector.

Preparation handoff: I-A2 owns descriptor/store identity, complete expected acknowledgement bytes, root/revision validation and candidate dependency semantics. The above establishes only physical access feasibility and historical-page effort. Candidate E1/E8 content windows, inverse suppliers and a schema-exact prepared acknowledgement remain **not_run** here; their decisive premise is the preparation owner's actual emitted declarations plus admitted fixture content. Source read-registration alone cannot substitute for those controls. Reuse neither a fixed 64-row count nor EXPLAIN output cardinality as an absence/completeness argument.

## Admission, fitting and native coexistence

Select finite population, resident allocation and active-stage job/CPU owners separately. A proposed `ResidentSession` holds population/native thread and buffer allocation plus drain ownership. `StageRequest` declares checked construction/transient demand, job count, CPU width and original scope. Admission reserves all next-stage memory before job/CPU acquisition; tentative acquisitions release before waiting, while real resident memory remains charged. Oversize, closed, deadline and cancellation are typed operational outcomes. Optional idle retained state yields before it blocks every stage's required growth. Unknown foreign/provider allocation retains its conservative guard; no nnz or observed peak masquerades as a hard bound.

Current `MathPolicy` defaults are jobs=4, compiler workspace=4 GiB and worker=8 GiB. **Retain** those defaults during this change. They are policy caps/charges, not observed peaks; the reference jobs=32 fixture is not a recommended deployment default. Revisit defaults only with the runtime owner's representative concurrent native/fit/dynamics population and actual memory observations. Remove blanket charges only for individually derived known layouts; unknown provider/compiler/native construction stays explicitly bounded/refused.

Fitting interface: estimation owns only its actual admitted team; at completion it hands `QualifiedEstimate` plus immutable fit problem and original clock to a quiescent profile coordinator, releases active permits, then admits parameter/end chains independently. Each chain retains adaptive pin/seed policy; deterministic public assembly uses parameter/end order, not finish order. Chain starts record the last accepted point, falling back to the estimate; numerical failures alone may subdivide. Resource/contract/operational failures stop truthfully. Covariance/observation rank remains its statistical matrix, not Root Jacobian rank. Concurrent experiments are **deferred** until a profile demonstrates material serialization and callback/provider independence; preserve deterministic addend order and a single common capacity owner.

Native coexistence: POUNCE local pools, HiGHS/Uno exclusion, retained KINSOL/IDAS and IBEX may retain allocation while quiescent, but cannot retain CPU needed by independent child work. Active team callbacks borrow their team's existing allowance and may keep it while waiting; they must not pause siblings. The current `ComputeAdmission` exposes only `pause` and blocking `resume`; C8 requires `try_resume(execution) -> Acquired | Unavailable` plus cancellable readiness/retry. A proof waiter may test the library guard, but must drop it immediately if compute cannot be reacquired; no blocking resume under IBEX. Every retry uses original deadline/stop; every acquired resource has one drop/join owner. Extend feature routing to root-isolation and cover legacy optional-Execution entries through explicit enclosing scope, not a fresh clock.

Chunk result-copy reservation is **selected**; notification suppression is **retained unchanged**. DataFusion spillable-participant departure can increase another participant's allowance even at zero departing bytes. FairSpillPool accounting does not permit a generic bytes-only notification rule. CPU-width/default measurements and native lifecycle fairness are acceptance/prioritization observations, not grounds to weaken these ownership rules.

## Result demand, acknowledged close and horizon interfaces

Select a typed `ResultDemand` carried from request/preparation through execution, publication, completion and readers: exact relation/column/scalar projection, seed demand and requested public materialization. It changes product extent, never original scientific assessment. `ScalarRowLocator` names exact attempt/set, registry-typed original primary-key tuple, block identity and row offset once per scientific row. Floats/sentinels and field names are not a replacement primary key. Readers validate the original Arrow row, schema, units and typed key before extracting fields; optimizer filtering sees exact predicates. A locator cannot survive result retirement as a live storage permission.

`SetClosed` acknowledges requested zero-row sets with schema, demand and explicit empty coverage; unrequested, absent, partial and empty remain distinct. `AcknowledgedCoverage` records actual set/batch identities, ordinals, counts and digests with bounded gaps or monotone ordered ingestion. `CloseAck` freezes the complete advertised inventory and returns recovered close authority; `SealAck` returns authoritative terminal outcome/coverage/seed permission. Normal return assembles digest-acknowledged original completion bytes plus those acknowledgements. Recovery settles exact original append/close/seal identities and reconstructs server evidence; it never estimates success from local totals. Heartbeat remains owned through uncertainty and joins before terminal admission.

Connected key handles use a slim protected completion-only read; owned admitted seed bytes can be passed through the already-authorized start/prediction operation after one protection/decode. Subsequent independent use must obtain new permission. Public `SolutionId` remains identity and resolves once. Numeric data and solver options may deliberately differ while exact coordinate/layout/backend compatibility and provenance remain checked. A copied completion snapshot surviving retirement is a **separate proposal**, triggered by an actual public snapshot consumer, not a silent handle semantic change.

Horizon interface: one attempt owns rolling measurement/input windows and separate controller/estimator/background last-accepted products and permissions. A failed newer role step must not erase an older accepted seed. Stream private immutable step products under the one ingestion fence; keep only outcome/identity history demanded by the final report. Full in-memory history is explicit demand with reservation. General `Staged::Start::Accepted(index)` and `predecessor_at(index)` remain supported outside the bounded horizon owner. Close/seal happens once; prefixes confer no independent success/resumability. IDAS forward sensitivities and replay remain selected per numerical consumers, never removed merely because a report is streamed.

Ordinary telemetry uses bounded nonblocking samples/coalescing with exact drop counters; strict audit interruption is a separately selected policy. Original assessment, mandatory scientific/seed terminal transport and native cancellation/drain remain independent of telemetry loss.

## Remaining opportunity decisions and migration

| Question | Decision and exact reopening owner/trigger |
|---|---|
| Ready hints/frontier | **Select** dependency/version hints and continuous durable refill; claim rechecks policy. **Retain** cheap ephemeral scan until study-policy profiling shows material effort |
| Exact repeated preparation/admission | **Select** 36a engagement/ack ownership; JSON ingress remains a real trust boundary |
| Same-chain session affinity | **Select** optional compatible handoff within resident capacity; loss of affinity/restart must preserve allowed starts |
| Dense uncertainty GEMM/diagonal-only | **Defer** to fitting owner when a large-output or diagonal-only consumer specifies the product and scalar-sum tolerance contract |
| Result analysis repeated protected opens | **Select** one explicit live-pinned read operation where callers share an operation; no timeless lease caching |
| BatchCoalescer/buffered IPC file writer | **Defer** to result/assurance owner after payload-shaped blocks expose remaining tiny-write effort |
| Seed binary encoding | **Retain** exact JSON u64 bits; separate version/compatibility proposal only if measured payload cost warrants it |
| Polling versus LIVE SELECT | **Retain** bounded polling; subscription lifecycle only when actual identical-edit wait traffic is material |
| Flights versus async cache guard | **Retain** Flights cancellation/completion ownership; replacement needs those exact semantics, not a cache API resemblance |
| SDK request-context patch/thin upstream hook | **Retain** patch/hold; upstream candidate must preserve original deadline, admission, cancellation and remote/local drain |
| ensure_writes advisory settings | **Retain** advisory role; supervisor ownership/drain decides quiesce |
| Digests/copies/IPC preflight and metadata wrappers | **Hand off** to 36d/36a; repetition can be removed before hashing changes; untrusted read-side preflight remains |
| Compression/inbound Arrow/new DB/cross-output product | **Separate proposal** when a named consumer defines the changed guarantee; no implicit adoption |

Migration is producer-and-consumer complete: registry declarations and generators first, operations adapters and every shared-authority writer/reader next, runtime and Python demand/seed/close/handle consumers together, then durable format activation/rebuild under explicit version authority. Fence old writers so they cannot recreate old bookkeeping after cutover. Preserve retained source/run lineage, exact uncertain effect receipts, live study/analysis borrowers and historical result access; no production deletion is authorized. Remove superseded broad touches and per-cell/per-row/full-history paths only after replacements and all callers pass targeted controls. Emitted query races, original-clock interruption/restore, multi-fit saturation, both native admission contexts, typed locators, empty/gap coverage, lost acknowledgements, retired handles, and long failed-role horizons remain implementation acceptance obligations. No source or disposable fixture establishes integrated scientific/product acceptance.
