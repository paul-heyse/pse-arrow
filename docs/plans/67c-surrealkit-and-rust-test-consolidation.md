---
title: "Plan67c: SurrealKit and Rust test consolidation"
status: in-progress
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
Use `--json-out` to interpret the suite during its native run. A wrong or missing generated
artifact, setup failure, and partial or invalid result stay distinguishable from a test failure.
Use the authored schema, suite and seed inputs to regenerate the result when needed. Observe
physical settlement separately. [Working evidence](67b-invocation-custody-and-developer-history.md)

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
| C01 | complete | Existing SurrealKit pin, generated schema and native fixture selection | Explicit locked tool preparation and the owned sync, rollout parity and deliberate drift controls passed with original endpoint disposal. |
| C02 | in-progress | C01 database assertions and complete owned case map | Migrate database-only cases, consolidate the Rust cases and remove each replaced test-selection variant and duplicate target. Independently compare named assertions and run representative isolated Rust, native and force-validated controls. Supply the actual variant delta to 67d. |
| C03 | in-progress | Current scientific inputs and native lane contracts. B02 active custody for consumers that still need executable borrowing. | Move the ordinary capsule and lane consumers onto the chosen test-owned path and retire superseded preparation/capture. Verify changed input refusal, native worker failure and settlement, and repeated ordinary runs with zero added retained helper copies or physical database copies. |

## Execution checkpoint — November 7

The existing Rust native-service target drives the pinned SurrealKit CLI against the current
generated schema. One physically owned endpoint hosts separate logical databases for the suite
controls. The native fixture settles and disposes the endpoint after the passing suite, expected
assertion failure, setup refusal and interrupted seed reader. The test verifies the original JSON
case population and refuses a foreign endpoint. The first native run passed all 11 suite cases.
Selection is explicit while the prepared SurrealKit release is a separate prerequisite.
The historical rollout's target differs from the current generated schema. Sync uses the
current schema. Parity compares sync and replay against the retained rollout's frozen target.
The pinned native installer publishes the tool, locked inputs and receipt in one verified
generation. The owned native-service test passed the 15 generated cases and the authored
rollout parity and drift controls, including agreement across 1,538 definitions in the
matching rollout. The service and private database directory were gone after settlement,
with zero retained executable captures. The rollout comparisons have a separate bounded
observation allowance because the real fixture exceeded the ordinary sync deadline.

The exact decimal cell and denied-mutation database assertions now use SurrealKit.
Ten pure gRPC peer and source-acquisition controls passed in the ordinary Rust library
test context without the `canonical-tests` selection. The library graph still had 498
units in each mode. The consolidation removes a separately selected feature context for
these cases. The broader borrowed, physical and runtime instrumentation cases still need
their named replacements.

The tooling evidence tests now select the live fixture by their exact test identity under the
existing `evidence-tools` capability. The `xtask/canonical-tests` feature is removed. The
remaining test-only fixture descriptor dependency is a dev dependency. It still enters the
ordinary xtask test graph and belongs in the post-migration unit comparison.

The ordinary KINSOL portable-root case now re-executes the admitted test image and enters the
production native receiver. The targeted case passed with complete settlement. Standalone
bootstrap and opt-in telemetry have their own explicitly prepared test selection. The four
ordinary native dynamics cases capture their one authored physical package in memory through
the existing package owner. All four passed with Diffsol and Ipopt, explicit Arrow validation,
complete settlement and zero prepared fixture requirements. Multi-package and reference
consumers remain in C03.

Explicit runtime preparation serializes selection per checkout. It reopens the exact capsule
and native lane, checks their current source and build context, and reuses them when the
original input owner confirms correspondence. Two explicit preparations under the same four
native profiles returned the same selection and original bytes. The second added no generation
and left the physical allocation unchanged.

Record original test outcomes and coverage separately from settled resource counts. Compare
repeat runs in the same compatible compiler context, showing retained state after settlement and
the changed unit count. Preserve independent scientific/reference assertions when a case moves.
Use the targeted product routes before scope-end qualification, with the accepted integration
population still owned by Plan66.
