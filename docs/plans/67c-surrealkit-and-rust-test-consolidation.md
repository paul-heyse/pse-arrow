---
title: "Plan67c: SurrealKit and Rust test consolidation"
status: proposed
date: 2026-11-06
doc_role: plan
doc_owner: docs/plans/67-storage-lifecycle-and-build-reuse.md
doc_topics: [design, validation, storage, current-work]
review_sources: [docs/functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md]
---

# SurrealKit and Rust test consolidation

## Target and baseline

Deliver [O05](../functional_reviews/storage-lifecycle-and-build-reuse-2026-11-06.md#O05).
Exercise schema, SurrealQL, permissions, and database HTTP API contracts on deliberately seeded
logical suite databases. Select application scenarios as Rust tests and runtime inputs within a
compatible compiler context. Build helpers and prepare physical services only for cases whose
assertion requires that boundary. Preserve Plan66's native admission, exact scientific assertions
and force validation. The number of data, actor, and fault scenarios can grow without growing the
ordinary Cargo variant set or leaving a physical database or retained helper image per scenario.

**Interface-checked at `c2126df45`.** The repository pins standalone SurrealKit
`1.0.0-beta.6` at `92ebc3cc89b99f00d21c7d2cbf5136d728367061`. The existing
`production_generated_contract` suite already checks deadline behavior, exact decimal and
binary payload round trips, relation endpoint refusal, activation absence, and a restricted
namespace actor. Its source can grow under the existing generated schema owner. Native
fixture declarations already distinguish target-owned, runner-borrowed and preparation-only
inputs. Borrowed cases currently consume the admitted native inventory through
`NativeTestCommand::from_source_capture` and use isolated UUID databases after readiness.
Current runtime fixture preparation still captures the `xtask` capsule builder and the
`pse-native-lane` worker, each in a distinct feature composition. Selection checks the
capsule against the original builder and current inputs. A move of SQL assertions to SurrealKit
does not by itself remove those scientific runtime captures. [SurrealKit pin](../../.config/surrealkit.toml) ·
[Existing TOML suite](../../scripts/tests/fixtures/surrealkit/canonical.toml) ·
[Fixture declarations](../../crates/pse-operations/Cargo.toml) ·
[Runtime capture owner](../../crates/pse-host/src/fixtures/runtime.rs) ·
[Fixture settlement](../dev/surreal-substrate.md#targeted-fixture-controls)

## Database contract lane

Select the existing canonical schema generator and stage its definitions as SurrealKit schema
input under a test owner. Keep the canonical schema and generated native representations single
source. Author only the small test seeds, fixtures and assertions. Use SurrealKit's separate
setup, sync or rollout, seed, and fixture stages with a deliberate authored input for each.
The pinned runner creates unique logical database names, applies setup, schema, seeds and
fixture inputs in its supported order, and executes cases in the suite context. Use separate
suites where mutations would invalidate another case's assumptions. Use suite, case, and tag
selection for targeted checks. Run ordinary schema suites from sync. Exercise
`--schema-from both` for a selected authored rollout chain, with parity enabled. Replay begins
in an empty logical database and compares definitions to sync. A native physical upgrade or
reopen test still exercises the specific old state and service boundary.
[Pinned suite runner](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs) ·
[Test kinds and setup](https://surrealdb.com/docs/manage/schema-migration/testing) ·
[Rollouts](https://surrealdb.com/docs/manage/schema-migration/rollouts)

Use `sql_expect` for query values and refusals, `permissions_matrix` for actor coverage,
`schema_metadata` and `schema_behavior` for declarations and constraints, and `api_request`
for actual HTTP behavior. Carry the native expected-refusal versus assertion-failure distinction
into the receiving report. Assert exact values and actual non-effects on denied operations.
Take `--json-out` as the suite's original evidence and send selected facts to
[67b](67b-invocation-custody-and-developer-history.md) once its schema is available. A
wrong or missing generated artifact, setup failure, and partial/invalid result must stay
distinguishable from a test failure. Keep the original authored schema, suite and seed
identities and physical settlement separately from the selected JSON evidence.

Provision only on a test-owned endpoint. Prefer a dedicated test namespace and namespace-scoped
setup credentials. Root credentials are reserved for a disposable test-owned server when a case
needs separate namespaces. Select actor credentials explicitly and keep them outside evidence.
Refuse the configured durable catalog or an endpoint whose test ownership is unverified.
Treat `--keep-db` as a named investigation with a release owner and purpose. The pinned runner
can warn on cleanup error, drops the statement-level cleanup checks, and can leave state on
setup failure or interruption. Require native readback of absence for the exact suite databases
or verified settlement and disposal of the entire test-owned endpoint. Where the runner cannot
reveal identities for an interrupted suite, use a privately owned disposable endpoint as the
release unit. A tool exit alone cannot settle service, suite data, or scientific children.
Scope the selected lane to passing, failing, setup-failed and interrupted journeys and exercise
the wrong-endpoint refusal before allowing ordinary reuse of a test endpoint.
[Pinned cleanup and report behavior](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/crates/surrealkit/src/tester/runner.rs)

## Rust and physical test lanes

Inventory the real cases and callers of the `canonical-borrowed`, `maintenance-borrowed`,
`publication-owned`, `operator-evidence`, `external-server-restart`, and native-service
declarations in `pse-operations`. Follow `canonical-tests` and `test-support` through
`pse-runtime` and `xtask`. For each case record the assertion and its required boundary.
Transfer database-only assertions to SurrealKit. Move remaining test selection into ordinary
Rust unit tests or consolidated integration targets, with Nextest filters and authored runtime
data for actors, seeds, faults, and fixture roles. Use dev-dependencies for test-only consumers
where their actual compilation context fits. Preserve actual product linkage and native
maintenance modes and explicit Arrow force validation. Transfer each consumer and delete its
replaced test or feature-only path together after the replacement proves the same behavior.
The feature and target census shared with [67d](67d-cargo-variants-and-sccache-reuse.md) must
distinguish test-selection variants from real compatibility modes.
[Runtime features and fixtures](../../crates/pse-runtime/Cargo.toml) ·
[Fixture declaration and proof routes](../dev/validation-assessment.md#individual-commands) ·
[Pinned Nextest selection](https://nexte.st/docs/selecting/)

For the runtime fixture class, map actual consumers of `scientific-inputs` and
`scientific-inputs-check`, the authored scientific catalog, and native lane. Move the
scientific checks into the normal test-owned path where they do not require a subprocess
boundary. Retain the current source/provider correspondence and validated capsule semantics.
Where a case genuinely exercises a separate worker or prepared executable, borrow the admitted
image under [67b](67b-invocation-custody-and-developer-history.md) or use the existing
target-owned native admission. Once all affected consumers have moved, remove the corresponding
routine builder/lane preparation and per-invocation capture path. Keep readiness, stale or
missing inputs, and original joined failures observable. Repeated preparation must cease
producing redundant retained copies even when the required worker remains.
[Scientific inputs catalog](../../tests/fixtures/scientific-inputs.toml) ·
[Current runtime owner](../../crates/pse-host/src/fixtures/runtime.rs)

Keep physical lifecycle, restart, persistent migration, recovery, real protocol, and scientific
execution controls in their native owners. Generate the smallest synthetic state that exercises
each real transition. Where compatible, borrowed Rust cases can use separate logical databases
on one bounded test-owned service. Use a supported test-owned in-memory server for ordinary
database cases only where isolation and tested semantics match. A physical copy belongs to a
named backup or restore assertion. Release the original service and state after physical drain
and protocol settlement, preserving explicit holds or unresolved evidence for the owner.
[Selected canonical server profile](../../scripts/surrealdb-source/source-build.md) ·
[Plan66 service ownership](66c-preparation-execution-and-service-lifecycles.md)

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| C01 | planned | Existing SurrealKit pin, generated schema and native fixture selection | Deliver the bounded test-owned endpoint and real TOML lane. Exercise seed and actor isolation, expected refusal, schema or rollout parity where selected, explicit wrong-endpoint refusal, and verified cleanup through success, failure and interruption. |
| C02 | planned | C01 database assertions and complete owned case map | Migrate database-only cases, consolidate the Rust cases and remove each replaced test-selection variant and duplicate target. Independently compare named assertions and run representative isolated Rust, native and force-validated controls. Supply the actual variant delta to 67d. |
| C03 | planned | Current scientific inputs and native lane contracts. B02 active custody for consumers that still need executable borrowing. | Move the ordinary capsule and lane consumers onto the chosen test-owned path and retire superseded preparation/capture. Verify changed input refusal, native worker failure and settlement, and repeated ordinary runs with zero added retained helper copies or physical database copies. |

Record original test outcomes and coverage separately from settled resource counts. Compare
repeat runs in the same compatible compiler context, showing retained state after settlement and
the changed unit count. Preserve independent scientific/reference assertions when a case moves.
Use the targeted product routes before scope-end qualification, with the accepted integration
population still owned by Plan66.
