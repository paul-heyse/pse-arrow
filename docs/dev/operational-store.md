# Canonical operational store

The canonical SurrealDB store owns source revisions, reusable compilation products,
run and attempt records, studies, independent scientific artifacts, result blocks and
analysis graphs. Blueprint §20.3–§20.6 and §21 own the contracts; the
[current-work index](../plans/README.md) routes to their implementation and qualification
owners. [Plan 37](../plans/37-surrealdb-lifecycle-and-integration.md) owns the lifecycle
transition and its affected acceptance. The [local substrate guide](surreal-substrate.md)
owns server setup, resource allocation, maintenance, quiescing, backup and restore.

## Setup and runtime ownership

```bash
just surreal setup --state /path/to/private/state --interpretation pse.substrate.v5
just surreal start --state /path/to/private/state
just canonical-init /path/to/private/state
```

The current generated interpretation is `pse.substrate.v5`. Initialization installs
its complete schema and marker atomically in a fresh database. Ordinary opening validates
the selected interpretation and schema digest without DDL. Design-phase cutover regenerates
owned state from current definitions and fixtures; it requires no old-data migration,
backfill or historical reader. Current scientific document versions and current-version
backup/restore retain their own contracts. Explicit reconstruction and frozen forward
maintenance follow the [operator route](surreal-substrate.md#explicit-design-phase-reconstruction-and-recovery).
Python attaches with `pse.Runtime(settings, substrate="/path/to/private/state")`.
The engine, mathematical preparation, native work and returned buffers share the
admitted deployment budget. Managed workers use the supervisor's finite capped slots.
An optional qualified `producer` receipt enables eligible stored-product reuse. An
absent receipt permits admission and persistent execution while refusing cross-build
scientific reuse. `ephemeral=True` explicitly selects process-local numerical outcomes.

`runtime.close()` releases that logical caller and its aliases. Independently created
runtimes retain their own admission. Already admitted reads, durable publication and native
joins keep their continuation authority through drain. Logical release does not clear shared
program caches or disconnect transport drivers. Only the deployment/fixture owner performs
physical shutdown, which refuses while admitted activities or protected readers remain.

## Revisions and selected compilation

Source objects have immutable versions and revision membership. Authoring checks the
expected head and relevant named guards atomically; untouched objects keep their identities.
Selected compilation records actual dependencies, including absence and name-resolution
premises. Scientific admission remains the compiler's responsibility. Stored products
retain compact construction descriptions with exact source, interpretation and qualified
producer identities. Reopening reconstructs products through their library owners;
solver factors and process-local handles are not serialized as scientific authority.

## Runs and terminal results

Ordinary execution records selected source roots and an acknowledged attempt before
numerical work. Claims, cancellation, ingestion closure and manifest admission check
generation fences. Lost acknowledgments settle through immutable operation identities.
Successful, failed, partial and cancelled outcomes retain their actual scientific decisions.
Unsealed staging, incidental incumbents and another attempt's values are not admitted results.

Save `result.canonical_run_key` and `result.canonical_attempt_key`. Use
`runtime.results(run, attempt, "runtime.solve_variables")` to stream the exact original
relation. `run_record`, `attempt_record` and `result_manifest` return narrow registry Arrow
tables. `latest_results(problem, relation, classes=...)` selects an admitted semantic run
in explicit terminal classes; filtering does not reinterpret scientific quality.
`output_results` additionally selects output, field, partition, recorded row coordinates
and optional value or missingness predicates.

## Connected queries and Arrow export

Scientific results remain in bounded, self-contained uncompressed Arrow IPC blocks.
Sparse cells and dense output-group indexes point into original blocks rather than
replicating payloads or creating one graph vertex per sample value. Selectors retain
identities, units, basis, missingness and exact IEEE payloads.

A reader pins its exact source and terminal manifest before paging. Fully decoded copied
Arrow arrays retain accounted memory independently of the reader, runtime and database.
They remain usable after eligible storage reclamation and release their allocation on final
drop; they do not retain an indefinite database pin. Expiry refuses further reads. Cancellation
or a failed statement is an error, not successful stream exhaustion.
`export_results(run, attempt, relation, destination)` writes IPC with exact source and
terminal lineage. The destination appears after successful completion; interruption leaves
a sibling `.incomplete` file. Existing destinations are refused.
`progress(run, attempt)` reads admitted terminal events; live handle observations remain
separate. Progress never grants scientific seed permission.

## Studies across workers

Studies record distinct occurrences, dependencies, bindings and start policies. Each
point's authored `attempt_limit` owns its retry limit. Readiness uses immediate premises
and prepares only ready numerical consumers. Equal bindings remain distinct occurrences.
Use `just pse-worker --until-idle --maximum-actions 100`, or in-process
`runtime.work(maximum_actions=...)`, to perform bounded canonical actions.

A recovered closed attempt can reconcile and seal its existing observations without
rerunning numerics. A lost summary worker can be replaced without repeating point solves.
Finalization checks cancellation and every point outcome. External explicit starts are
admitted as copied exact study inputs before activation. Dispatch, retry and replacement
workers use those admitted artifacts, preserving occurrence role and provenance without
late acquisition from the producer. Admission before producer retirement survives it;
retirement before admission refuses without a fresh-start fallback. Portable prediction
anchors retain exact coordinates and permissions; the ready consumer reconstructs any
required library factor. Missing requested seeds are typed refusals. Killed workers do not
promote arbitrary progress or incumbents.

## Independent scientific artifacts

Scientific promotion preserves sufficient original content and interpretation independently
of the producing run. `promote_seed_artifact(solution, owner_kind=..., owner=...)` copies an
original qualified seed and assessment. `promote_scientific_artifact(run, attempt, kind,
owner_kind=..., owner=...)` admits `final-state`, `trajectory` or `local-analysis` content.
Final states use actual endpoints with mode, integral and input interpretation; trajectory
promotion retains the recorded prefix and its transitions; local analyses retain validity,
conventions and required upstream context. Insufficient context refuses promotion. Copying
preserves original identities, units, basis, exact bits, assessment and partiality; it grants
no stronger scientific qualification or numerical restart permission.

Reopen an active occurrence with `runtime.scientific_artifact(key)`, inspect `metadata()`
and read its bounded IPC members with `block(ordinal)`. Independent user, project, audit,
study, analysis and live-read obligations protect content and sources. `retain(kind, owner)`
and `release(kind, owner)` address one exact attributed owner; a released owner identity
cannot be reacquired. One owner's release cannot invalidate another owner's future reads.
After the last obligation and creation/read windows settle, repeat `collect()` until true.
Producer payload retirement does not remove an independently retained artifact.

Keep `artifact.original_intent` or the exact artifact intent carried by a typed error or
interrupt. `runtime.recover_artifact_intent(intent)` returns release/collection authority
bound to the original creation, including absent or inactive headers. It cannot
create a new occurrence, renew the original creation window, replay promotion or grant
content access. Unknown creation remains protective until exact settlement.

## Persisted scientific analyses

`prepared.dependency_analysis(controls)` persists original incidence and execution dependency
edges with explicit meanings. `runtime.result_analysis(run, attempt, controls)` persists
reported sensitivity evidence referring to the original row and field. `runtime.analysis(key)`
reopens the exact active graph; `header`, `nodes` and `edges` return bounded Arrow pages.
Controls select semantic roots and upstream or downstream traversal. Method, configuration,
source closure and producing attempt remain recorded. Result evidence uses typed artifact
block, row and field locators; copied graph inputs remain readable after eligible producer
retirement. Reachability does not establish a quantitative derivative or certificate;
original scientific rows retain their validity fields.

## Explicit retention and bounded reclamation

Scientific payload retention and detailed run/study history have separate release operations.
Live attempts/readers, retained studies, analyses and artifact obligations protect their exact
sources and content. `forget_study_results` withdraws the terminal study's result obligation;
`forget_analysis_results` performs one bounded retirement page and must be repeated until
true. `reclaim_run_results(run)` retires an eligible terminal producer run and performs one
bounded payload cleanup page; repeat until `complete`. Independently retained artifact
content survives producer collection. Persistent cursors make cleanup restartable.

Use typed `HistoryOwner`, `HistoryObligation` and `HistoryPurpose` documents for detailed
history. `retain_history_obligation` and `release_history_obligation` address exact attributed
`audit` or `unsettled` obligations. After payload retirement, borrowers, creation windows
and obligations settle, `release_history(owner)` irreversibly fences that run or study.
Repeat `reclaim_history_page(owner)` until its `complete` field is true. Study collection
includes settled member and summary-run detail in bounded pages. Terminality or elapsed
age alone never authorizes history release.

Permanent owner and operation identities survive detail collection. Released history reports
retired/history-unavailable or its defined compact settled attribution; it does not make an
old identity unused, reconstruct a successful original response or authorize redispatch.
Unknown effects retain the exact detail needed for settlement. Compact identity state can
still grow with issued operations. Logical collection establishes neither returned disk
space nor a constant-size database. Application retention does not remove supervisor state;
backup and restore follow the separate operator route.
