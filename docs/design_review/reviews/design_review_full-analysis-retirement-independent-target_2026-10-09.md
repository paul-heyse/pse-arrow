# Full analysis retirement — independent target review

**Date:** 2026-10-09  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** Analysis occurrence creation, admission, staging, page acquisition, retirement,
settlement and physical deletion, including their source/run retention suppliers.  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and
the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** `4c24721e691187e1a5b28398b29722fbde671da8`, inspected working-tree source and
[ADR-0172](../../adr/0172-full-analysis-retirement.md) on 2026-10-09; pinned SurrealDB 3.3.0
source. Concurrent Plan 32/33 work is outside this review's write scope.  
**Reviewer:** Independent delegated reviewer; did not author ADR-0172 or its
[author assessment](design_review_full-analysis-retirement-target_2026-10-09.md).  
**Disposition owner:** [Plan 33 EFF00/EFF10 and RC03](../../plans/33-efficiency-principles-remediation.md#eff10).

**Behavioral/semantic adequacy: Accept at Proposed target-design strength. Architectural
fitness: Accept at Proposed target-design strength. Overall decision: Accept.**

No blocking defect was identified in the proposed target. The two conflict barriers address
different races: the header fences an existing occurrence; the primary retention authority
fences creation that has not produced a visible header. A durable monotone expiry floor closes
future replay after that header disappears. This is an adequate constant-size replacement for
per-analysis retirement history, provided its existing source authority retains its incarnation
and floor. Acceptance does not establish an implemented or tested protocol.

## 1. Scope, drivers and coverage

The functional target is complete deletion of retired analysis headers, graphs, lineage input
relations, roots, pins and analysis-specific guards/admission/cleanup/audit state. Explicit new
creation of identical scientific content remains supported. Cleanup must survive interruption
and limit each transaction's work, including high-degree graph nodes. Socket closure, expiry
and client cancellation cannot establish remote transaction settlement.

Direct inspection covered `crates/pse-operations/src/canonical_analyses.rs`,
`canonical_retention.rs`, `canonical_result_retention.rs`, relevant guard writes in
`canonical.rs`, `canonical_transport.rs`, `crates/pse-codegen/src/codegen/surreal_analyses.rs`,
`surreal_execution.rs`, `surreal_retention.rs`,
`crates/pse-schema/src/catalog/substrate.rs` and
`crates/pse-runtime/src/workflow/analyses.rs`. Blueprint §5.2 and §20.3–§20.6 supply the
current neighboring contracts. The author assessment was read as a claim to examine.

The concrete backend uncertainty was examined in pinned `surrealdb-core`,
`surrealdb-datastore` and `surrealdb-kvs-rocksdb` 3.3.0 source and upstream locked-read tests.
No builds, tests, probes, benchmarks or production queries were run. Numerical algorithms,
general source/run retirement redesign, backend deployment qualification and historical-reader
leases are excluded. These exclusions do not weaken the reviewed retention boundary.

## 2. Decomposition, ownership and dependencies

| Responsibility | Owned meaning and consumed contract |
|---|---|
| Creation intent | One fresh occurrence and immutable original authority, expiry and exact request; transport retries reuse it |
| Primary source authority | Existing retention record and incarnation; monotone aggregate creation closure |
| Atomic admission | Actual source/result validity plus durable roots/inputs and staging header in one transaction |
| Header | Staging, Active and Retiring transitions; exact point-record conflict barrier |
| Physical cleanup | Indexed bounded deletion and temporary progress; existing retention owners release roots |
| Run retention | Every surviving analysis-input relation conservatively retains its run |
| Transport | Definite-conflict retry and uncertain-delivery recovery without issuing a new occurrence |

The integration owner can expose these operations through ordinary checked types/functions.
The design needs neither a generic lifecycle coordinator nor a transaction wrapper that hides
the backend's actual conflict capability. Tests of occurrence/key validation need no solver;
tests of commit ordering necessarily need the selected store.

## 3. Contracts, authority and constraints

Scientific equality and lifetime identity are distinct. Current runtime content hashing can
remain a scientific digest, but cannot remain the sole occurrence key. ADR-0172 binds authority
incarnation, nonce and absolute expiry into a server-checked occurrence key. A retry cannot
change the expiry or primary authority while retaining that key, and a separate transport
identifier cannot silently become permission to issue another intent.

Source and result validity remain with their current owners. Admission checks actual revision
receipts, reclaimed intervals and exact run/attempt/manifest protections under the corresponding
source/run conflict registers, then atomically creates roots and input relations. Removing
the analysis-specific preliminary `protect()` loop avoids fresh pin issuance by a delayed
analysis retry; cached bytes cannot restore reclaimed authority.

Page reads return owned copied Arrow data. The existing `AnalysisHandle::nodes` and `edges`
acquire each page separately; they carry no durable historical-read entitlement. Refusing later
pages once retirement starts is consistent with that consumed contract. Already returned pages
may remain usable and arrive after settlement without leaving persistent analysis state.

Physical semantics and well-posedness: no quantity, physical convention, equation system,
derivative or numerical outcome changes in this boundary. Exact scientific source/result
attribution remains required while an occurrence is retained. No new numerical stage is added.

## 4. Representative scenarios

| ID | Discriminating case | Required response |
|---|---|---|
| S01 | Original creation dispatches before expiry, but transaction commit is held beyond expiry | Final source-authority write orders its effect before settlement or forces conflict; elapsed time alone never suffices |
| S02 | Settlement sees no header; concurrent creation writes it | Exact absent-header registration prevents the settler from committing an empty-state conclusion across that creation |
| S03 | Append, activation or page acquisition overlaps retirement | Earlier transaction commits before the phase fence or fails validation; later work refuses without upserting state |
| S04 | Cleanup/reply is interrupted or acknowledgement is lost | Retiring header or original intent supplies recovery; settlement can be resolved without an audit receipt |
| S05 | Same key is retried with changed expiry/authority, or time moves backward | Server binding and durable floor prevent renewed original authority |
| S06 | Same content is explicitly created again | Fresh valid occurrence succeeds with independent lifetime |
| S07 | Source collection or run retirement overlaps admission/partial cleanup | Shared conflict authorities and physical input claims prevent a retention gap |
| S08 | One node has most edges | Edge pages drain before node deletion; native cascade cannot hide remaining graph-sized work |
| S09 | Authority reset/restore or backend substitution | Old incarnation cannot become addressable with a cleared floor; substitute backend demonstrates the exact consumed capability or refuses |

S09 tests mechanism substitution. Another analysis algorithm should compose with the same
occurrence protocol; it need not introduce new retirement rules. New physical models and solver
substitution are outside this bounded lifecycle decision.

## 5. Mechanisms and execution

Consider a creator and final settler using overlapping snapshots. Creation registers the
existing primary retention guard and writes the exact header. Settlement registers that exact
header key, including absence, and writes the primary guard while advancing its floor.
If creation commits first, a settler that observed absence cannot validate its registered
header key. If settlement commits first, a creator using the older guard snapshot cannot
validate. A creator starting afterward observes the closed expiry and refuses. These cases
cover the original dispatched operation, rather than only retries after a visible header.

The floor may advance only after server time reaches the immutable expiry. Under normal time,
another currently valid intent expires above that boundary; advancing the floor does not
invalidate it. Backward time can cause a fresh short window to refuse, as explicitly described,
but cannot reopen an old grant. Source-authority reset/restore must change the fenced incarnation
instead of clearing this scalar under an addressable old authority.

Pinned source supports the narrow backend premise. `dbs::processor::process_record` routes
point-record locked reads to `get_record_for_update`; that datastore method always reaches
`getu_key`, returning a default absent record only after the underlying read. `getu_key` rejects
read-only transactions. RocksDB `getu` calls `get_for_update_opt(key, true, &self.read_options)`;
its writable commit path reaches the underlying optimistic transaction's `commit()`. This is
commit conflict registration, not blocking or predicate serializability. Current
[SurrealDB transaction documentation](https://github.com/surrealdb/docs.surrealdb.com/blob/main/src/content/learn/querying/concepts-and-guides/transactions.mdx),
retrieved through Context7, is consistent with this interpretation. The pinned source controls
the version-specific assessment.

The upstream `select_for_update_test` cases inspect existing records, both planner strategies,
and locked/plain-read controls. They were not run and do not settle the exact absent-header,
delete or function-wrapped emitted-query cases. In particular, a read-looking page query must
actually use and commit a writable transaction; writing `FOR UPDATE` inside a helper does not
by itself establish its enclosing transaction mode.

Existing `root_owner`, `analysis_nodes`, `analysis_edges`, `analysis_inputs` and
`analysis_input_run` indexes match the proposed access paths. Native `doc/purge.rs::purge_edges`
can delete connected relations when deleting a document. Edge-first cleanup is consequently
necessary, with both endpoint membership checks preserved. Inputs must precede final header
deletion so its native cascade does not perform hidden cleanup or prematurely release run
retention. Temporary progress belongs in the retiring header and disappears with completion.

## 6. Architectural assessment and gates

All verdicts concern the Proposed target, not the unchanged implementation.

| Foundation | Evidence and judgment | Verdict |
|---|---|---|
| AP-01 | Lifetime authority is separated from scientific content, transport and source/result ownership | Satisfied |
| AP-02 | Explicit occurrence, phase, pending settlement and backend capability contracts cover S01–S09 | Satisfied |
| AP-03 | Atomic roots/inputs and existing conflict authorities compose without another coordinator | Satisfied |
| AP-04 | Content, occurrence, dispatch, committed effect and settlement have distinct governing meanings | Satisfied |
| AP-05 | Checked key fields, header phase and monotone scalar expose enforceable constraints | Satisfied |
| AP-06 | Bounded owners and exact conflict keys support deterministic tests without numerical infrastructure | Satisfied |
| AP-07 | Indexed pages, edge-first deletion and constant-size aggregate state fit growth/interruption qualitatively | Satisfied |

| Gate | Judgment | Basis / limit |
|---|---|---|
| G1 Authority | Pass | Actual source/result owners and server key binding govern admission |
| G2 Semantic fidelity | Pass | Fresh lifetime does not reinterpret scientific content or exact inputs |
| G3 Validity | Pass | Finite immutable admission and retention checks precede durable authority |
| G4 Hidden behavior | Pass | Missing state refuses; neither transport retry nor cleanup bootstraps analysis guards/pins |
| G5 Consistency and recovery | Pass at Proposed strength | Dual conflict keys, durable floor and restart progress cover S01–S09; execution remains unqualified |
| G6 Transformation and reuse | Pass | Explicit fresh occurrence permits same-content recreation |
| G7 Truthful capability claims | Pass | No task/socket drain, global serializability or measured benefit claimed |
| G8 Library leverage | Pass | Native conflict registration, indexes and relation deletion supply the fitting mechanisms |
| G9 Architectural fitness | Pass | All seven foundations satisfied in the declared boundary |
| PS-G1 / PS-G2 / PS-G3 | Not applicable | No physical, formulation or numerical transformation is proposed |

## 7. Findings

No blocking target finding or SHOULD exception is identified. The existing implementation
does fail the selected target: `surreal_retention::forget_analysis` drops roots and upserts a
permanent retirement marker; header, nodes and inputs remain. `forget_run` excludes inputs
through that marker. `surreal_analyses::available` also uses `touch('analysis:'+key)`, which can
create an absent analysis guard. These are observed replacement obligations, already owned by
Plan 33 EFF10 and the author review's F01; they do not contradict the explicitly Proposed
replacement.

The acceptance requirements below are concrete implementation constraints of ADR-0172, not
claims that tests already pass or a second finding-disposition ledger.

## 8. Library fit and ownership cost

SurrealDB supplies atomic effect publication, exact-key optimistic conflict registration and
native relation maintenance. The canonical store must own the meaning of occurrence authority
and complete retirement; no built-in retention feature removes that domain responsibility.
The design retains one scalar per existing source authority rather than records per retired
analysis. Shared-guard contention and preserving its lifetime are real costs. They are localized
to the authority that already coordinates source retention. No quantitative performance claim
is established. Backend replacement must preserve this capability explicitly.

## 9. Alternatives and tradeoffs

Permanent per-analysis tombstones or operation ledgers violate full deletion. Expiry without a
durable floor permits revival after clock rollback and cannot settle an older snapshot.
Client task/socket draining cannot establish remote abort after process loss. A global lifecycle
coordinator introduces another authority and recovery path. The proposed existing-header barrier
plus existing-parent floor is the simplest viable examined alternative using the chosen store;
its transaction realization is library-owned. Revisit if exact absent-key conflicts cannot be
consumed or primary authority lifetimes cannot honor incarnation fencing.

## 10. Verification and acceptance requirements

| ID | Required control | Evidence now / acceptance result needed |
|---|---|---|
| <a id="c01"></a>C01 | Execute exact emitted creation, page/header read and absent/existing settlement queries in writable committed transactions; hold server transactions through both commit orders, including original dispatch before expiry and commit afterward | Interface-checked pinned source only; deterministic conflicts/ordering required, not timeout inference |
| <a id="c02"></a>C02 | Exercise canonical key binding, changed expiry/authority refusal, original-intent retry after lost acknowledgement, backward time and fresh same-content creation | Proposed; old occurrence cannot regain authority, fresh valid occurrence succeeds |
| <a id="c03"></a>C03 | Enumerate relevant source-guard mutators in `canonical.rs`, `canonical_retention.rs`, generated execution/retention and schema evolution; exercise preserve-floor and reset/restore fencing | Current writers inspected; every writer must preserve the scalar, with no old-incarnation bootstrap/reset path |
| <a id="c04"></a>C04 | Race source GC and run retirement with admission; interrupt cleanup before and after input/root removal | Proposed; no admission retention gap and every surviving input remains a run claim |
| <a id="c05"></a>C05 | Resume multi-page/high-degree cleanup, then query all analysis-owned row/root/pin/guard namespaces after delayed operations settle | Proposed; bounded edge-first work, no header/input/lineage/admission/audit/tombstone residue |
| <a id="c06"></a>C06 | Verify public/runtime/Python outcomes for pending expiry, absent unknown intent and acknowledged final completion | Proposed; `Result<()>` success must not imply settlement while cleanup or original grant remains open |

No test failure count or pass claim is supplied: all execution checks are **not_run** by
assignment, against the repository's zero-failure target. No numerical conformance suite is
applicable to this documentation review. Implementation acceptance requires named executed
controls, particularly C01; source inspection alone cannot establish their emitted-query behavior.

## 11. Authority changes and disposition

<a id="rc01"></a>**RC01:** Blueprint §5.2 must separate content identity from occurrence;
§20.3–§20.4 must describe complete analysis deletion and physical input-row retention;
§20.6 must state the exact conflict capability, pending/unknown outcomes and incarnation/floor
lifetime. The neighboring persisted-analysis descriptions in blueprint §19.2 and Python-facing
§21.1 must agree with that change. ADR-0172 owns rationale; Plan 33 EFF00/EFF10 owns adoption,
implementation and current dispositions. The coordinator owns the architecture revision route.

No architecture section or ADR was edited by this reviewer. No source/run scientific receipt
retention exception is implied by complete deletion of an analysis occurrence. Existing Plan 32
and other Plan 33 work remains outside this assessment's ownership.

## 12. Decision

Accept ADR-0172's bounded target at **Proposed** strength. Its model and physical route satisfy
the reviewed semantic and architectural requirements, including original creation crossing
expiry, absent-header settlement, fresh same-content recreation and no permanent per-analysis
history. The shared floor is acceptable precisely because it is constant-size source-authority
closure, with explicit immutable key/incarnation binding and preservation obligations.

No further target redesign is required by this independent assessment. C01–C06 remain required
before implementation acceptance; accepting this document neither closes EFF10 nor qualifies
the product, the backend deployment or numerical/performance behavior.
