# Canonical operational store

The canonical SurrealDB store owns source revisions, reusable compilation products,
run and attempt records, studies, scientific result blocks and analysis graphs.
[Plan 28](../plans/28-surrealdb-unified-substrate.md) owns implementation and qualification
status. The [local substrate guide](surreal-substrate.md) owns server setup, resource
allocation, quiescing, backup and restore instructions.

## Setup and runtime ownership

```bash
just surreal setup --state /path/to/private/state --interpretation pse.substrate.v1
just surreal start --state /path/to/private/state
just canonical-init /path/to/private/state
```

Initialization installs the generated scientific schema in a new compatible database.
Ordinary opening checks its interpretation; it does not migrate an unknown database.
Python attaches with `pse.Runtime(settings, substrate="/path/to/private/state")`.
The engine, mathematical preparation, native work and returned buffers share the
admitted deployment budget. Managed workers use the supervisor's finite capped slots.
An optional qualified `producer` receipt enables eligible stored-product reuse. An
absent receipt permits admission and persistent execution while refusing cross-build
scientific reuse. `ephemeral=True` explicitly selects process-local numerical outcomes.

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
Finalization checks cancellation and every point outcome. Continuation consumes explicitly
qualified predecessor solutions. Portable prediction anchors retain exact coordinates and
permissions; the ready consumer reconstructs any required library factor. Missing requested
seeds are typed refusals. Killed workers do not promote arbitrary progress or incumbents.

## Persisted scientific analyses

`prepared.dependency_analysis(controls)` persists original incidence and execution dependency
edges with explicit meanings. `runtime.result_analysis(run, attempt, controls)` persists
reported sensitivity evidence referring to the original row and field. `runtime.analysis(key)`
reopens the exact active graph; `header`, `nodes` and `edges` return bounded Arrow pages.
Controls select semantic roots and upstream or downstream traversal. Method, configuration,
source closure and producing attempt remain recorded. Reachability does not establish a
quantitative derivative or certificate; original scientific rows retain their validity fields.

## Explicit retention and bounded reclamation

History remains retained until explicit withdrawal. Live attempts/readers, retained studies
and active analyses protect their sources and results. `forget_study_results` and
`forget_analysis_results` withdraw those obligations while preserving lineage receipts.
`reclaim_run_results(run)` retires an eligible terminal run and performs one bounded cleanup
page; repeat until `complete`. Persistent cursors make cleanup restartable. Child indexes
are removed before payloads. Tombstones fence writes, replay and uncertainty settlement.
Application retention does not remove supervisor state; backup and restore use its operator route.
