# Full analysis retirement target — author review

**Date:** 2026-10-09  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** Analysis occurrence identity, original creation admission, mutation/read fencing,
settlement and full physical deletion proposed by
[ADR-0172](../../adr/0172-full-analysis-retirement.md), including source and result retention.  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and
the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** Product baseline `4c24721e691187e1a5b28398b29722fbde671da8`, the proposed ADR,
inspected working-tree owners on 2026-10-09, and pinned SurrealDB 3.3.0 source. Concurrent
Plan 32 and Plan 33 work is preserved.  
**Reviewer:** Delegated author reviewer. The same agent supplied the protocol and authored
ADR-0172. This is a scoped formal-format author assessment, **not an independent review**.  
**Disposition owner:** [Plan 33 EFF00/EFF10 and RC03](../../plans/33-efficiency-principles-remediation.md#eff10).

**Behavioral/semantic adequacy: Accept at Proposed target-design strength.
Architectural fitness: Accept at Proposed target-design strength.
Overall author assessment: Accept. Independent target judgment remains a separate obligation.**

The target separates a scientific content digest from a particular analysis lifetime, then
closes that lifetime through database conflict authorities. It does not mistake an elapsed
deadline for settlement. A header fences work that already has an analysis; a shared source
authority fences the original creation that might not yet have produced a header. Full cleanup
can therefore remove analysis history while preserving a constant-size creation-closure fact.

The decisive tradeoff is explicit: the existing source-retention guard permanently preserves
one monotone expiry scalar, but no retired analysis identity or payload. That guard cannot be
silently reset under a still-addressable authority. This is an enduring source-authority
contract, not an implementation detail or a disguised per-analysis tombstone list. The proposal
is credible only with that contract, exact absent-record conflict registration, immutable
key-bound authority/expiry and deterministic tests of the emitted transactions. None has been
qualified as implemented by this review.

## 1. Scope, drivers and coverage

The functional target is complete retirement of an explicitly created analysis, including
staging or partially populated graphs, after interruption or uncertain remote delivery. The
graph is currently bounded at 4,096 nodes and 8,192 edges; admission has at most 65 source
revisions and 64 result inputs. Cleanup must remain bounded per transaction even for skewed
high-degree graphs. There is no asserted latency or throughput result.

Inspection covered runtime analysis identity/handles; store admission, protection and transport
retry; generated analysis, execution and retention functions; canonical schema declarations;
and source-retention guards/roots. The concrete uncertainty requiring upstream inspection was
whether an earlier dispatched issuer can commit after its local caller and temporary analysis
state disappear. Pinned datastore, processor and RocksDB transaction code were read to settle
the capability argument. Current transaction documentation was retrieved through Context7 as
a lead; the pinned source controls the version-specific conclusion.

This review does not repeat the whole-code efficiency review, review numerical graph analysis,
or qualify the server/backend deployment. No build, product test, backend probe or benchmark
was run. Source and upstream test inspection are not executed evidence.

## 2. Ownership and composition

| Owner | Decision hidden / contract exposed | Dependency and local test context |
|---|---|---|
| Runtime creation intent | Fresh occurrence; immutable original authority, expiry and request | Checked key representation and store admission; deterministic intent/retry tests |
| Existing primary retention guard | Monotone aggregate creation closure; lifetime/incarnation | Store transaction capability; guard-writer preservation tests |
| Atomic admission | Valid sources/results become durable roots/inputs with the staging header | Existing source and execution conflict authorities; controlled GC/run-retirement races |
| Analysis header | Staging, active, retiring and absent meanings; operation authority | Point-record commit registration; held append/read/retire transactions |
| Cleanup | Bounded physical release and restart progress | Indexed ownership, native relation deletion, source retention guards |
| Run retention | Any surviving input relation is a claim on results | Existing execution-run guard and run-index predicate |
| Transport | Definite conflict retry versus uncertain delivery | Original intent; no renewal through per-RPC deadlines |

The composition uses existing canonical-store concepts. There is no new global lifecycle
service, historical-reader service or neutral transaction abstraction that hides the required
backend capability. Scientific source validity remains with canonical revision/result owners.
The analysis lifetime owns only its derived graph and references. The new scalar's authority
belongs with source retention, where delayed initial creation must be ordered against cleanup.

## 3. Contracts and authority

An occurrence is not content equivalence. Its checked key binds primary authority incarnation,
nonce and absolute expiry. The server verifies this binding, so a missing-header retry cannot
keep its occurrence identity while selecting a new expiry or a different, unfenced parent.
The bound occurrence key is also the creation operation identity; an independent reusable
operation identifier must not become a backdoor to fresh admission after deletion.
The exact request is retained and compared while the header exists. After full deletion, the
key-bound expiry and parent's scalar permanently deny that original creation authority.

One owned intent is constructed before the first await. This alone is not server fencing;
the key validation, existing-parent registration and closed-expiry predicate supply that
fencing. The separate transport deadline is allowed to change per query but cannot issue a
new creation window. Fresh explicit recreation obtains a new occurrence and valid expiry,
including when its scientific content is identical.

Absence has three distinct meanings: unavailable for ordinary mutation/read; potentially
in-flight creation for settlement; and settled when the original intent is closed and all
owned rows are absent. A bare key cannot prove an unknown grant settled. The protocol does
not retain a completion receipt merely to answer that question. A lost final acknowledgement
can be resolved with the original intent and shared closure authority.

Source validation and source-root creation occur atomically under existing retention guards.
This replaces the analysis-specific `protect()` issuance loop, whose absent-pin creation
currently recomputes expiry. Existing result-read protections are still checked at input
admission. Analysis-input rows then retain runs independently until physically removed.
No generic protection redesign is needed to meet this analysis target.

## 4. Representative scenarios

| ID | Stimulus | Expected boundary / discriminating control |
|---|---|---|
| S01 | Original issue dispatches before expiry, server commit is held through cleanup | Existing-parent conflict registration orders commit before the closure write or aborts it; expiry alone never acknowledges settlement |
| S02 | Settler observes no header, issuer creates one before settler commits | Exact absent header `FOR UPDATE` registration forces conflict/retry; new header is then retired and drained |
| S03 | Append, activation or page acquisition is held while retirement writes phase | Earlier operation commits before the fence or aborts; new operations refuse Retiring/absence without creating guards |
| S04 | Cleanup stops after several pages and client/server restarts | Retiring header supplies original intent/progress; bounded cleanup resumes; no local task registry supplies authority |
| S05 | Same intent retries after final deletion; clock moves backward | Bound original expiry remains below the durable scalar; no header, pin or guard is recreated |
| S06 | Fresh explicit operation uses the same content after settlement | Fresh key and valid expiry above the closure scalar admit normally; content identity does not retain retirement |
| S07 | Source GC or run retirement races first admission | Shared source/run guard registration plus atomic roots/inputs orders the competing effects; no retention gap |
| S08 | One node has most graph edges | Indexed edge pages drain first; node deletion does not cascade across the entire remaining graph |
| S09 | Backend changes, or primary authority is reset/restored | Equivalent absent-key/commit conflicts must be demonstrated; a reset requires a fresh fenced incarnation |

S09 is the relevant mechanism-substitution case. Numerical domain extensions do not change
the lifecycle semantics and are outside this bounded decision. Additional analysis algorithms
reuse the same occurrence/admission/retirement protocol rather than defining new fencing rules.

## 5. Mechanisms and execution

The safety argument has two necessary conflict points. Initial creation registers the existing
primary retention record before writing any analysis-owned rows. Settlement writes that record
to advance its closure scalar. If creation used an older snapshot, commit validation prevents
it from crossing that write. Creation that commits first may leave a header for cleanup.
Therefore settlement also registers the exact header key, **including absence**: an earlier
missing-header observation cannot commit as complete after an issuer wins the creation race.

The scalar prevents a later new snapshot from reopening the original grant, including after
clock rollback. A finite admission expiry alone would not provide that permanent refusal.
Advancement occurs only once `now >= E`; at a normal clock, another still-valid intent has
expiry above that boundary and remains eligible. Backward time may reduce availability of a
new short window but cannot reopen an old one. Guard reset is forbidden rather than relying
on a clock guarantee the backend does not supply.

The source-backed capability is narrower than serializability. In pinned 3.3.0:

- `surrealdb-core/src/dbs/processor.rs::process_record` gives a point `SELECT FOR UPDATE`
  priority over skipping record-value fetches and calls `get_record_for_update`.
- `surrealdb-datastore/src/tx.rs::get_record_for_update` reaches underlying `getu_key` even
  for an absent record, then returns a default absent record. It does not create a row.
- `surrealdb-kvs-rocksdb/src/lib.rs::getu` requires a writable transaction and calls
  `get_for_update_opt(key, true, snapshot_read_options)`. Commit validates registered keys.
- Upstream `select_for_update_test` exercises concurrent updates and locked versus plain
  reads, but was not run here. Its existing-record controls do not replace local tests of
  the exact absent-header/delete/query path.

Consequently every application barrier uses an explicit writable point-record transaction.
Ordinary predicate scans do not fence phantom creation. The generated SQL must preserve this
path and the deployment must use the examined RocksDB capability. A server task may remain
alive after losing commit authority; no claim of network/task drain is needed.

Cleanup uses existing analysis ownership indexes, bounded page limits and temporary progress
in the retiring header. Edges precede nodes because pinned document purge deletes connected
edges. Current append already validates both endpoints belong to the same analysis; preserve
that invariant so cleanup ownership also bounds cascade work. Input deletion and source-root
release happen before the final header delete. Root release uses the corresponding retention
guards. An input row remains a conservative run-retention claim throughout partial cleanup.

Normal acquisition returns copied Arrow pages, not durable analysis-reader leases. Closing
future pages during retirement is therefore faithful to the current handle contract; already
owned buffers need no database lifetime. The proposal explicitly declines to imply a reusable
snapshot across independent pages after retirement starts.

## 6. Foundations and gates

All judgments below concern the Proposed target, not current implementation.

| Foundation | Argument | Verdict |
|---|---|---|
| AP-01 Separation of concerns | Scientific inputs, lifetime, transport and source authority retain separate decisions | Satisfied |
| AP-02 Stable contracts | Key-bound finite authority, phase and closed-expiry semantics are explicit | Satisfied |
| AP-03 Composition | Existing roots, input relations and transaction conflicts compose without a new coordinator | Satisfied |
| AP-04 Domain model and semantic authority | Content, occurrence, dispatch, committed effect and settlement are distinct and govern behavior | Satisfied |
| AP-05 Explicit structure | Durable header phases and one shared scalar expose recovery authority | Satisfied |
| AP-06 Local reasoning/testability | Two exact conflict keys and deterministic held transactions distinguish safety from timeout behavior | Satisfied |
| AP-07 Execution fits workload | Bounded admission/pages, edge-first deletion and constant-size shared closure avoid history growth | Satisfied |

| Gate | Judgment | Evidence / limit |
|---|---|---|
| G1 Authority | Pass | Source/result owners validate admission; no cached reconstruction restores reclaimed authority |
| G2 Semantic fidelity | Pass | Explicit new occurrence replaces content-lifetime conflation; scientific inputs stay exact |
| G3 Validity | Pass | Original finite admission, actual revision/result protections and phase checks are required |
| G4 Hidden behavior | Pass | Missing guards refuse; expiry renewal, implicit reactivation and absent upserts are prohibited |
| G5 Consistency and recovery | Pass at Proposed strength | Dual conflict barriers, durable closure and retiring progress cover S01–S07; execution controls outstanding |
| G6 Transformation and reuse | Pass | Fresh same-content recreation is explicit; existing valid scientific material remains reusable |
| G7 Truthful capability claims | Pass | No task-drain, global serializability, performance or implemented claim |
| G8 Library leverage | Pass | Uses native transaction conflict registration and graph deletion; backend capability remains explicit |
| G9 Architectural fitness | Pass | All foundations satisfied for the bounded target; no standing lifecycle framework added |
| PS-G1 physical typing / PS-G2 structural well-posedness / PS-G3 numerical soundness | Not applicable | No scientific model or solver transformation; source/result authority and existing qualification remain in force |

The efficiency heuristics concerning necessary state, bounded crossings, late hydration and
effect/recovery lifetimes support this mechanism qualitatively. They do not establish a
measured benefit. A shared per-problem conflict register can contend with retention operations;
that is the actual authority boundary, not a reason to weaken fencing or invent a broad queue.

## 7. Findings and obligations

<a id="f01"></a>**F01 — Existing marker-based retirement cannot satisfy complete deletion.**
Current `surreal_retention.rs::forget_analysis` releases roots and upserts a retirement marker;
the header, graph and inputs remain. `forget_run` interprets input rows through that marker.
Deleting the marker alone loses the existing creation fence. AP-04, AP-07, G4 and G5 apply to
S01–S07. ADR-0172 supplies the replacement ownership and closure protocol. Verify complete
cleanup followed by stale requests and fresh recreation, and require zero per-analysis rows.
Implementation/disposition remains with Plan 33 EFF10; this author review does not resolve it.

<a id="f02"></a>**F02 — Proposed target requires shared-authority preservation, not only new analysis code.**
The scalar must survive every existing retention-guard update and cannot reset under a reused
authority. Existing `canonical_retention.rs` and generated execution `touch` paths update
generation and sometimes create guards. S05/S09, AP-02 and G5 require inspection of all writes
to this specific guard namespace, schema evolution and reset/restore lifecycle. Analysis calls
must use existing-record refusal rather than generic bootstrap. Verify that no relevant writer
replaces guard content, decreases the scalar or reuses an old incarnation. This is a required
implementation scope in the ADR, not an unacknowledged target gap.

<a id="f03"></a>**F03 — Author assessment does not supply independent review.**
The authoring assignment deliberately combines proposal and review artifacts. The
[design-review skill](../../../.codex/skills/design-review/SKILL.md#shared-review-roles) says,
“Give a fresh design reviewer the target, requirements and source evidence for an independent
judgment.” The shared reviewer role likewise requires independent assessment. This document
discloses its author status and supplies the requested bounded analysis; an independent target
judgment remains with the coordinator before treating this as independently reviewed. No
implementation acceptance or settled independent verdict is inferred from the author verdict.

## 8. Library fit and ownership cost

SurrealDB supplies the commit registration, atomic transactions, ownership indexes and native
relation-pointer cleanup. It does not supply the domain notion of a fully retired analysis or
immutable creation grant. Those meanings belong in the existing canonical-store contract.
The smallest custom mechanism is the checked occurrence intent, header phase and shared scalar.
Replacing it with a generic lock wrapper or client-drain manager would hide the critical
absent-record/commit capability without eliminating the domain obligation.

The local server launcher selects RocksDB with versioned reads disabled and synchronous writes.
That is source evidence about configuration, not a new durability test. Other backends or
restore behavior must be assessed against the exact consumed capability. No dependency bump,
new crate or licence restriction is needed.

## 9. Alternatives and tradeoffs

| Alternative | Benefit | Material limitation / judgment |
|---|---|---|
| Current permanent per-analysis marker | Simple persistent refusal | Leaves history and graph/input residue; fails target |
| Expiry plus cancellation/socket drain | Little database state | Does not settle delayed transactions; process loss and clock rollback break the claim |
| Permanent per-operation admission ledger | Durable exact authority history | Another per-analysis tombstone; fails zero-residue target |
| New global lifecycle coordinator | Centralizes ordering | Extra durable owner/recovery path; existing conflict authorities suffice |
| Proposed scalar + exact header barrier | Constant-size source authority, no retired occurrence payload | Requires monotone preservation, bound key fields and fresh incarnation on authority reset |

The proposed realization is the simplest viable alternative using the chosen store. Revisit
if its backend cannot honor missing-key commit registration or the source authority is allowed
to reset without fencing old requests. Do not compensate with an indefinite retired-analysis
marker while claiming the same full-deletion scope.

## 10. Verification and evidence limits

| Claim / risk | Evidence | Required settling control |
|---|---|---|
| Current residue and marker predicate | Interface-checked/source inspection | Baseline behavior identified, not rerun |
| Pinned point-record conflict capability | Interface-checked/source inspection | Execute exact emitted-query missing/existing/deleted-key races on selected backend |
| Original issuance versus final fence | Proposed | Hold both transactions; exercise each commit order, dispatch-before-expiry/commit-after-expiry and absent first observation |
| Mutation/read/cleanup fencing | Proposed | Hold append, activation, page acquisition and cleanup at commit; only ordered commit or conflict is permitted |
| No renewed same occurrence | Proposed | Change E or primary authority while retaining key and require server refusal; same-intent retry never allocates new intent |
| Restart and acknowledgement loss | Proposed | Resume partial pages from header; settle absent original intent; resolve lost acknowledgement without an audit row |
| Retention continuity | Proposed | Race source GC and run retirement against admission; input-row retention remains until physical deletion |
| Bounded skew cleanup | Proposed | High-degree graph with more than one page of edges; verify edges drain before node cascade |
| Aggregate closure preservation | Proposed | Exercise all relevant guard writers, backward time and authority reset refusal/new-incarnation path |
| Complete retirement and fresh recreation | Proposed | After delayed requests settle, inspect every owned row/root/guard/pin namespace for zero residue; new valid occurrence succeeds |

Tests must control server transaction ordering, not infer settlement from client timeout or
an empty socket. An instrumented server/datastore harness may supply deterministic holds;
its observations do not justify changing production semantics. This review proposes those
controls and does not claim that the current test harness already exposes every hold point.

## 11. Authority changes and disposition

Blueprint §5.2 needs the content/analysis-occurrence distinction. Blueprint §20.3–§20.4 needs
physical input-row retention, full analysis deletion and the source authority's aggregate
creation-closure/lifetime contract. Blueprint §20.6 needs the exact transaction capability,
refusal and restart semantics. ADR-0172 owns the rationale; the coordinator owns adoption and
the architecture revision route. No architecture or accepted ADR was changed in this task.

The implementation review must include runtime/Python creation handles, schema/generator
declarations and emitted queries, store admission/transport retry, all relevant retention-guard
writers, run-retirement predicate, cleanup indexes/native cascade behavior and reset/restore
authority handling. Narrowing it to `forget_analysis` would omit the original issuance race.
No SHOULD exception is requested. F01–F03 and S01–S09 have one current disposition owner in
Plan 33; this review retains its original observations and does not maintain a second status.

## 12. Decision

Behavioral/semantic adequacy and architectural fitness are acceptable **as a Proposed target
in this author assessment**. Exact pinned source supports the required conflict capability;
the protocol addresses both an existing analysis and a dispatched original creation, while
leaving no retired per-analysis payload. It neither treats time as settlement nor introduces
a permanent per-occurrence receipt to hide that gap.

Independent target review remains due because this reviewer authored the ADR. Before any
Implemented/Tested acceptance, the coordinator must settle the actual generated-query races,
key/expiry/authority validation, complete guard preservation scope, restart behavior and
zero-residue oracle. No product, numerical or performance qualification is implied.
