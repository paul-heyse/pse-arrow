# Execution fit

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.** Decisive
files were clean.

**Method.** This is qualitative structural assessment under the efficiency heuristics. No
benchmark, timing or contention measurement was run, so every cost statement here describes
structure, not magnitude. The maintainer premises apply:
- canonical SurrealDB/MX;
- up to 16 SurrealDB-coordinated lanes;
- durable work that outlives its caller without a daemon.

The findings assess how these premises are realized, not the premises themselves.

This page holds:
- the fine-grained durable protocol ([DA-055](#da-055));
- the retention-protection families ([DA-056](#da-056));
- the narrowed publication and population residues ([DA-034](#da-034), [DA-035](#da-035));
- the collection scope record ([DA-025](#da-025)).

Build-unit and per-invocation fit appear under [DA-020](boundaries-and-composition.md#da-020),
[DA-059](boundaries-and-composition.md#da-059) and
[DA-043](development-tooling.md#da-043).

## <a id="da-055"></a>DA-055 — Each study point runs the generic durable lifecycle as many guarded commits, each with a fixed prologue, against shared per-problem keys

**Principles:** AP-07, DP-10, DP-19, DP-20; H2, H5, H8, H19, H22, H23, H25.

**Status:** new on 2026-10-29 (U3). It succeeds the population-loop diagnosis that Plan56 corrected
for creation and finalization ([DA-035](#da-035)). This finding concerns per-point execution.

**Claim at risk.**
- Durable studies of up to `MAXIMUM_STUDY_POINTS = 100_000` (`workflow/study.rs:26`) on 16
  coordinated lanes.
- PS-11: studies over many cases bind only values per case.

**Owner intent.** This finding challenges a granularity that the owners selected; it is not an
unnoticed defect.
- Plan58's SurrealDB boundary disposition O01 routes to 58d D03, which is scheduled and in
  progress: "complete bounded progression/action/premise handoff; competing workers, sparse
  continuation, stale preparation".
- 58d's state-placement table assigns candidate discovery to "bounded applicable action … no
  local durable scheduler truth".
- The D03 checkpoint records that `CandidateGroup` still suppresses only local duplicate
  preparation.
- The global capture gate in every prologue is §19.3's deliberate correction after an unguarded
  10,000-member SSI surrogate conflicted and rolled back.
- The 100k study ceiling and the 10k cohort cap are declared limits.

**Structural cause.**
- **Many guarded commits per point.** The unit of durable work is
  point → run → attempt → result sets → manifest → seal → point observation, and each transition
  is its own guarded RPC transaction. On the success path one point visits roughly a dozen or more
  synchronous AOL/EveryAppend commits:
  - `progress_studies` (`worker.rs:1665-1722`);
  - `claim_prepared_study_point` (`:1963`);
  - started and renewed;
  - `register`, which re-reads the run (`durable.rs:1319-1328`);
  - heartbeat and progress chunks;
  - queued result appends (`durable.rs:1925-2012`);
  - seeds;
  - `terminate`, which re-reads the attempt and run (`:2030-2066`);
  - completion chunk, close and seal;
  - a `canonical_study` read, then `observe` (`study.rs:2159-2170`).
- **A fixed prologue on every commit.**
  - The deadline.
  - A `FOR UPDATE` read of the global study capture gate (`fn::pse_workbench_v1::publication`,
    `surreal.surql:3440-3443`).
  - The issuer authority and module hash.
  - History-owner registration.
  - Schema admission appended to every script: `fn::pse_schema_v1::admit` locks the interpretation
    marker (`surreal.surql:2375-2390`).
  - A receipt lookup and an operation-record write.
  - Client-side, a synchronous `config.json` read and parse ([DA-052](boundaries-and-composition.md#da-052)).
- **A shared write guard on every claim.** Each claim calls `source_creator`, which touches
  `source-creators:<problem>` (`pse-codegen/src/codegen/surreal_execution.rs:124-127, 389`). Every
  point of a study shares that problem, and surrealmx validates write sets first-committer-wins.
  `ProblemTurns` (`pse-operations/src/canonical.rs:1072-1110`) serializes mutations per problem
  only within one process.
- **Discovery restarts each time.** Each lane action restarts progression with
  `continuation = None` (`worker.rs:1665`), and progression returns one action per RPC. Because
  preparation precedes the claim, other lanes skip points that are being prepared, one RPC at a
  time, and separate processes can duplicate preparation.
- **Retained units are per point.** Each point retains 11 always-on summary relations
  (`results.rs:64-83`) as separate result sets, because packing happens only within one attempt
  and relation.
- **The analysis bound is smaller than the study.** Cohort capture is capped at
  `STUDY_COHORT_MAX_MEMBERS = 10_000` (`pse-schema/src/catalog/study_workbench.rs:12`), and a
  cohort scan reads one small body per member per relation (`cohort_provider.rs:420-515`).

**Consequence.**
- Fixed protocol cost scales with the number of points, not with the scientific work, so
  many-cheap-case sweeps are dominated by commits.
- Additional lanes or processes add conflict on the shared per-problem key rather than
  throughput.
- 58d records that the 100k ceiling is unqualified and that no database speedup is measured. This
  finding adds a static reason to doubt that ordinary large sweeps have a credible route under the
  selected per-point unit.
- In context, not as a separate defect: studies larger than the declared 10k cohort cap cannot be
  analysed whole (H11).

**Counterevidence and legitimacy.**
- Science stays outside transactions (§20.6).
- Run installation is batched, which amortizes the sequence counter (`surreal.surql:1180-1196`).
- Result blocks are queued and batched (`result_projection.rs:138-173, 495-533`).
- Lanes retain preparation and sources.
- `FOR UPDATE` on the capture gate arms read-lock validation only, so it does not serialize
  ordinary writers against each other.
- Lost-ACK safety genuinely needs immutable receipts (§20.2).
- 58d deliberately rejects a performance ladder.

The commit count per point is approximate, and actual contention is unmeasured.

## <a id="da-056"></a>DA-056 — Retention safety is spread across many owner-specific protection families, each with its own fence and settlement

**Principles:** AP-05, DP-19, DP-01; H1, H25, H27.

**Status:** new on 2026-10-29 (U3). The proportion judgment is interpretive.

**Claim at risk.** One explicit lifecycle per durable fact, and a reclamation answer a reader can
discover (§20.4; §0.6 "Retention and exact selection").

**Structural cause.**
- About 18 of the 90 canonical tables hold protections, obligations or reclamation state:
  - result, capture and manifest protections;
  - artifact obligations, reads and releases;
  - file owners, borrowers and admission;
  - history obligations and retirements;
  - result and study retirements;
  - reclaimed ranges, roots and guards.
- Each family has its own creator/reclaim fence pair, grant/incarnation/nonce binding or
  monotone floor.
- `pse-operations/src/canonical_lifecycle.rs:11-55` lists seven release kinds, "each effect has a
  different original owner". The explanation facade exists because no single model answers "why
  is this retained?".
- Liveness is decided two ways:
  - attempts use leases;
  - file owners persist boot, PID namespace, PID, start time, systemd unit and cgroup inode in
    the canonical store (`canonical_files.rs:294-331`). That sits awkwardly with 58d's rule that
    process state in the store is "logical intent/status only".
- Read tickets are canonical writes, released by flag rather than deleted
  (`surreal_files.rs:110-126`).

**Consequence.**
- Each new producer or consumer kind registers with several fences.
- Creator fences are writes on shared keys, which feeds [DA-055](#da-055).
- Readers pay durable writes.
- Extension and diagnosis need several mechanisms (H27).

**Counterevidence.**
- ADR-0164 has a principled three-lifetime table.
- DP-05 favors separate owners where release semantics genuinely differ.
- Collection is bounded and restartable.
- The explanation facade gives one read view.
- Under the daemonless premise, a persisted owner identity is a reasonable way to detect crashed
  owners.

It was not established that one reachability model could honor all of these semantics.

## <a id="da-034"></a>DA-034 — Above the inline threshold, each relation-sized publication unit carries the full file-intent protocol

**Principles:** AP-07; H2, H5.

**Status:** narrowed. Small units are inline at or below `RESULT_INLINE_BYTES`
(`workflow/result_projection.rs:374-377`). Above it, units are still sealed at relation
boundaries (`:264-271`), each with the intent/fsync lifecycle. The per-point consequence is
absorbed into [DA-055](#da-055).

## <a id="da-035"></a>DA-035 — Study reopen and admission reconstruction page full descriptors four rows at a time

**Principles:** AP-07; H5.

**Status:** narrowed.
- Creation is cursor-resumed (`canonical_studies.rs:1006`), and finalization and cohort premises
  are corrected.
- `LIMIT 4` full-descriptor paging (`canonical_studies.rs:55`) survives for reopen and admission
  reconstruction (`workflow/study.rs:570, 1015`), which may legitimately need descriptors.
- Per-point execution is [DA-055](#da-055).

## <a id="da-025"></a>DA-025 — Built-in retirement stops at logical cleanup; physical collection remains an explicit scope limit

**Status:** retained as a scope record, with no active violation.
- The public collector (`pse-operations/src/canonical_files.rs:799`) has no built-in composition.
- Plan58 confirms "no automatic collection".

## Considered and not raised

- **Per-case solve preparation** re-resolves policy, class discovery and routing
  (`math/solves.rs:3148-3260`). Class discovery is value-dependent, and a defect was not
  established.
- **Parallel preparation-reuse trackers** beside Salsa: runtime premise traces, `ModelingCache`,
  a parsed-fields cache, thread-local quantity premises, and duplicated `BasisKey`/`request_hash`
  composition. These are within [DA-030](#da-030)'s decision scope, and no stale reuse was found,
  so DA-030 is not reopened.
- **Per-command supervised scopes.** The native host's scope-per-command containment is
  deliberate (ADR-0233) and unmeasured.

## Closed by decision, resolved and merged identities

| ID | Outcome at `f83bac039` | Decisive evidence |
|---|---|---|
| <a id="da-022"></a>DA-022 | Closed by decision (I16) | No new unsupported-allocation evidence sought or seen |
| <a id="da-030"></a>DA-030 | Closed by decision (I08–I10) | The always-cold MCP worker population that made it a candidate trigger was deleted (`f321e9e0f`). The 2026-10-29 preparation stream found several premise/key mechanisms but no stale reuse; revisit trigger unchanged |
| <a id="da-042"></a>DA-042 | Resolved | Explicit owner-stage/request-span export filter and SDK loss layer (`tooling/telemetry.rs:33-37, 231, 276`) |
| <a id="da-044"></a>DA-044 | Resolved; mechanism on a retiring path | Checkpointed recovery with a startup deadline (`scripts/surreal_server.py:7148-7235`). Backup remains in the Python supervisor; persistent backup/restore qualification belongs to 58h |
| <a id="da-045"></a>DA-045 | Resolved | `ResolvedClosure` holds no aggregate document vector (`workflow/project_closure.rs:13-18`) |
| <a id="da-026"></a>DA-026 | Resolved (2026-10-25) | Per-row locators removed; DataFusion pruning |
| <a id="da-027"></a>DA-027 | Resolved (2026-10-25) | Group/re-expand cycle deleted |
| <a id="da-028"></a>DA-028 | Resolved structurally (2026-10-25) | Multi-output guarded regions |
| <a id="da-029"></a>DA-029 | Resolved in the compiler (2026-10-25) | Runtime scan residue later resolved with DA-006's original manifestations |

Earlier full text is at source `682fc3830`.
