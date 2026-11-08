---
title: Qualification commands
status: current
---

# Qualification commands

Select checks that reveal the changed behavior. Native Cargo/nextest and the Rust performing
owners now supply correctness execution; the Python product, assessment/result ledger,
archive/reuse controller and measurement drivers are retired. Required remaining campaigns
belong to the active [Plan64 and Plan65 owners](../plans/README.md), including their H
qualification companions. This guide describes
commands and evidence limits, not a claim that full qualification has completed.

The local `pse-cli` skill (source: `.codex/skills/pse-cli/SKILL.md`) helps select useful native
checks, investigations and original-result readers. Prefer those routes when their actual
correctness selection, fixtures and native process ownership fit the task; raw tools remain
available under the same environment and force-validation obligations. This guide owns the
qualification meaning of the resulting evidence.

## What each command establishes

| Command | Scope established by successful execution | Exclusions |
|---|---|---|
| `scripts/pse check PACKAGE --all-targets` | Selected package targets compile under the actual admitted graph | Test execution and scientific assertions |
| `scripts/pse test PACKAGE --lib -E FILTER` | Selected source-built library tests under native nextest | Unselected targets, providers and services |
| `scripts/pse test PACKAGE --lib --features FEATURES -E FILTER` | Selected native library tests with required already prepared providers and declared fixture inputs | Unselected targets, provider preparation and scientific cases |
| `scripts/pse test PACKAGE:TARGET ...` | Selected source-built tests with their declared Rust fixture ownership | Other deployments, historical state and unselected scientific cases |
| `scripts/pse test pse-tests-lifecycle:query_memory_budget` | Actual scan and operator pressure, cancellation, recovery and release under the shared memory budget | Native service lifecycle and result publication |
| `scripts/pse test pse-tests-engine:pushdown_vs_unpruned` | Selected engine provider results against complete unpruned inputs, including a deliberately defective provider | Canonical snapshot pruning and statistics custody |
| `just modeling-conformance ...` | Selected authored conformance runs in the manifest's supported native/pure mode | Unselected packages and independent reference execution |
| `just parity` | Four native candidate exchanges against the isolated pinned IDAES/Pyomo environment | Other cases, versions and exhaustive parity |
| `just reference-unit` | Independent helper, wire and thermodynamic controls in the prepared parity environment | Solver execution and native candidate contracts |
| `just reference-solver-unit` | Independent exchanger and recycle solves and the exchange failure-witness control using prepared Ipopt | Native candidate contracts and full parity |
| `just reference-contract-unit CONTRACT_DIR` | Independent receiver checks against five original Rust-produced contracts | Regeneration or provenance of the supplied contracts |
| `just docs` / `just adr-lint` | Rendering/search or authored decision/register consistency respectively | Truth or quality of the scientific/design prose |

Report the command, mode, scope, baseline and actual failures. The failure baseline is zero.
A focused pass is not a full product pass. Interrupted, unattempted and unavailable cases remain
explicit; tool completion does not turn them into acceptance.

## Common conditions

Use the checkout's prepared host and pinned toolchain. `scripts/pse-env -- COMMAND` supplies
actual environment/placement; provider consumers select `--native=CAPABILITIES`. Ordinary
configured unbounded work uses 32 Cargo jobs and 16 nextest lanes without a new memory cap.
Explicit finite/scientific/reference/timing requests retain native admission and physical
containment. `--store` and allocation-binding markers are retired; Rust applications and
fixture owners construct their actual services. See [the environment guide](agent-environment.md).

Every Arrow-consuming correctness closure explicitly enables validation in actual target,
host/build and selected-root dev units. `cargo c`, `cargo t` and
`scripts/cargo-bin/cargo-pse-correctness` admit the actual Cargo unit graph and verify leaves.
Pure closures need no unrelated scientific root. Plain Cargo commands retain the same obligation;
feature spelling alone is not proof. A binary built earlier or an environment marker does not
prove the current source or compiled feature closure.

Selected host unit journeys use the shared ordinary-application-scope fixture before preparing
independent persistent sessions, so they can run through normal `pse test` ownership. That fixture
retains original stop/drain and does not permit independent detachment directly from a workload.
For direct fixtures without this adaptation, use the top-level `cargo t` route after
`scripts/bootstrap-host`. For example, use
`cargo t -p pse-host --test composed_investigations` or select the native session tests with
`cargo t -p pse-host --lib --features native-hir -E 'test(semantic::tests::native_snapshot)'`.
Each fixture explicitly prepares its helper and session, then verifies the original stop/drain.
Its child environment removes Cargo's injected `RUSTUP_TOOLCHAIN` so the authored project pin
selects the toolchain, while retaining every native ownership and placement input. Persistent
independent detachment remains unsupported inside an enclosing `pse test`/`pse-env` workload;
ordinary tests continue to use the native managed route.

### Inspecting a product invocation

Nextest owns selection, execution and reporting. Retain the exact native command, source/build
selection and original output/report when evidence is needed. There is no current `just result`
reader or Python invocation reconciliation ledger. A readable report does not establish test
success, fixture drain or scientific assertion coverage. A complete failing population remains
a failure; missing selected outcomes cannot be inferred from a zero-exit helper.

Group exact Nextest filters with `|` in one `pse test` command when target, features, profile,
and fixture prerequisites agree. The native runner can prepare the shared reader once while
retaining each test's verdict and the fixture's own settlement.

Native observation, capture, frozen inspection and review documents remain available through
Rust APIs; see [diagnostics](simulation-diagnostics.md). Keep copied observation scope distinct
from actual performing handles. Cancellation and deadline results retain the original native
work owner's join and physical drain obligations.

## Full local assessment

There is no replacement monolithic assessment command. At the authorized scope end, choose the
relevant unit/public caller, component, native process/fixture, solver and independent reference
journeys, then the existing `just hygiene`, `just governance`, `just docs`, native lints and
powerset commands. `just turn-end`, `ready` and `hygiene` are keep-going recipe aggregates, not
an assessment database. Follow `AGENTS.md` for their timing and report individual failures.

Manual GitHub workflows cover their selected compile/docs/governance and pure test scopes.
They do not qualify local Linux user-systemd placement, canonical deployment, full native
scientific journeys, release distribution or H performance. Current status stays with the
owning plan and architecture qualification basis in
[§24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).

## Representative functional selections

Select the public outcome and relevant failure branches, including real owner/placement
controls when ownership is changed. Generic library units are useful for pure transformations;
service construction needs its real private fixture. Preserve boundary sizes, races and orderings
that carry the assertion. The default native fixture route is not a promise to run all packages
or full deployment capacity. Broader campaigns require their explicitly selected consumers.

## Individual commands

```bash
scripts/pse check pse-runtime --all-targets
scripts/pse test pse-host --lib -E 'test(correctness::)'
scripts/pse test pse-ids --lib
scripts/pse tests pse-ids --lib -E 'test(scientific_replay)'
scripts/pse test pse-authoring:dsl_roundtrip -F pse-authoring/arbitrary \
  -E 'test(=arbitrary_ast_render_parse_preserves_structure)'
just scientific-replay-miri
just doctest
```

The local Miri route exercises the focused scientific replay authority cases. Run the
large-description stress case natively with
`scripts/pse test pse-ids --lib -E 'test(=scientific_replay::tests::scientific_replay_large_logical_description_preserves_record_binding)'`.

Nextest does not run doctests. `test-release` and `doctest-release` select the release profile;
that profile alone establishes no production performance. The CLI's `--features` and actual
target selection derive required capabilities and select their already prepared providers;
they never prepare a missing native provider implicitly. Use `scripts/pse COMMAND --help` for
the CLI grammar and `just --usage RECIPE` for retained specialized recipes.

For a canonical consumer, first explicitly prepare/select the patched server and production
module using the [environment guide](agent-environment.md#explicit-compilerklu-preparation).
Save the absolute immutable module receipt. Then, for example:

```bash
PSE_NATIVE_FIXTURE_MODULE_RECEIPT=/absolute/module.json \
  scripts/pse test pse-operations:pse_operations --target-kind lib \
  --features native-service -E 'test(canonical_server_unit)'
```

The Study qualification cases are ignored by ordinary selection. Prepare the production
receipt as above, then use `just surreal-module --qualification` for the companion receipt.
Select the fault, expiry and cancelled-apply cases together on a four-controller service.
The expiry case reserves that service's Nextest store group for its run.

```bash
PSE_NATIVE_FIXTURE_MODULE_RECEIPT=/absolute/production/module.json \
PSE_NATIVE_FIXTURE_MODULE_QUALIFICATION_RECEIPT=/absolute/qualification/module.json \
SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS=4 \
  scripts/pse test pse-operations:pse_operations --target-kind lib \
  -F pse-operations/native-service --ignored only \
  -E 'test(/::canonical_studies_surrealism_qualification_(faults_rollback_and_recover|pool_original_expiry_cannot_start_late|cancelled_apply_settles_original_receipt)$/)'
```

For the capacity case, prepare a long-hold companion with
`just surreal-module --qualification --long-hold` and select it separately.

```bash
PSE_NATIVE_FIXTURE_MODULE_RECEIPT=/absolute/production/module.json \
PSE_NATIVE_FIXTURE_MODULE_LONG_HOLD_RECEIPT=/absolute/long-hold/module.json \
SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS=8 \
  scripts/pse test pse-operations:pse_operations --target-kind lib \
  -F pse-operations/native-service --ignored only \
  -E 'test(=canonical_studies::canonical_studies_server_unit::surrealism_qualification::canonical_studies_surrealism_qualification_global_capacity_survives_source_churn)'
```

Choose the actual target/features/filter for the intended caller; a native solver target also
requires its actual solver features. The host admits and captures the source build/list through
this checkout's prepared correctness adapter. It selectively captures original service/test helpers
only when selected test declarations require them. Borrowed cases delegate the exact captured
compiler context to `NativeTestCommand::from_source_capture`, which starts their fresh fixture
and executes nextest without repeating the build/list. Target-owned cases receive immutable
inputs and create their own fixture through the original Rust owner. Pure selections prepare
neither a database nor unused helpers. Caller-supplied archive/metadata paths and run-time build
selectors refuse.
The private descriptor is issued only after readiness. Children borrow isolated UUID databases;
they gain no service shutdown/disposal authority. Exact service/child settlement remains with
the performing owner. Failed, interrupted or unproved scientific joins retain the fixture for
diagnosis rather than infer safe disposal. See [targeted fixture controls](surreal-substrate.md#targeted-fixture-controls).

Independent Python reference typing uses the actual environments, with no product extension:
`just typecheck` checks ordinary/oracle sources through `.venv-parity` (Python3.13), then the
three teqp reference files through `build/plan14-reference` (Python3.12). Explicitly prepare
those environments with `UV_PROJECT_ENVIRONMENT=.venv-parity uv sync --locked --only-group
parity --python 3.13` and `UV_PROJECT_ENVIRONMENT=build/plan14-reference uv sync --locked
--only-group thermo-reference --python 3.12`. These checks do not run a reference campaign.

For optional native CLI consumer controls, build the explicitly selected xtask features and
capture that executable before a later Cargo selection. `target/debug/xtask` is mutable Cargo
output: building a smaller feature set can replace it. `PSE_CANONICAL_TOOLS` selects the
captured executable for graph/query operator tests; the enclosing native fixture still owns
its private service and physical settlement. This is caller-owned test setup, not a second
production installer or an executable-reuse claim.

### Selected behavioral investigations

`just features-combinations` and `just features-no-default` use pinned cargo-hack 0.6.45 with
`CARGO_HACK_CARGO_SRC` set to the existing native correctness adapter. Cargo-hack owns enumeration,
order, keep-going and explicit partition selection; the adapter preserves selected features,
targets, profiles and force-validation. Feature permutations select nonincremental compilation
through Cargo's configuration argument, avoiding a retained incremental cache for every
one-shot combination. Ordinary editing profiles and compiler optimization levels are unchanged.
When partitioning a campaign, retain the printed command population and demonstrate complete
coverage; stopped or failed original captures remain stopped or failed. There is no combination
resume cache or copied working tree. One driver owns overlapping Cargo
graphs in a checkout. Focused adapter controls do not qualify the full powerset campaign.

Optional external coverage, mutation, fuzz, perf and rr tools can be selected explicitly for a
concrete question. The old Python campaign wrappers are retired; their scenario/acceptance
handoffs remain with H. A finite clean run does not establish exhaustive correctness, and an
unchanged baseline, setup failure and a caught behavioral mutation have distinct meanings.

## Case measurements

The former `case-measure`, `reference-run`, experiment and assessment measurement controllers
are retired; they have no automatic native substitute. H owns their exact scenario handoffs
and remaining measurement work. The Rust benchmark sources and scientific assertions remain
with their callers. No measurements are implied by this command migration.

Retain the measured operation, actual process/placement, sample interval and competing load.
Separate compilation, preparation, numerical execution, publication and serving. A fresh
process/database is not a cold filesystem; process VmHWM is not one case's allocated memory.
Pool-accounted bytes and OS RSS have different scopes. Configured quotas/masks do not prove
quiet or timing isolation. Report uncontrolled caches/load and failed/missing samples explicitly.
Functional correctness evidence may be cited separately; it does not establish equivalent
benchmark conditions or release-profile performance.

## Shared process fixtures and independent references

`tests/fixtures/plan14` remains a live authored test input despite its historical name. Native
public callers, conformance/benchmark consumers and Rust scientific units own its assertions.
The catalog `tests/fixtures/scientific-inputs.toml` declares scientific families and projections.
Independent references remain distinct from native model execution.

`just plan14-reference` generates frozen teqp PC-SAFT and Decimal caloric reference files from
the published parameter banks in an isolated locked CPython 3.12 environment, without product
imports. `bash scripts/setup-env reference` prepares `.venv-parity` from the locked parity
group. `just parity` checks that prepared environment and exercises the four native exchange
callers with the pinned `idaes-pse==2.13.0`. The narrower reference routes distinguish
helper and analytic checks, independent solves, and checks that require original native
contracts. Missing or wrong prerequisites fail the selected route. Analytic and reference
checks establish only their stated cases and conditions. Global stability requires its own
certificate. [Reference entry points](native-workflow.md#independent-native-reference-exchanges)

## Outside this scope

Other platforms, release/publication, full native deployment, complete powerset and performance
campaigns require their own acceptance. Architecture/scientific reviews are judgment at their
owning boundary. Retiring an orchestration mechanism does not discharge an unrun scientific or
H acceptance scenario. Use the [current work index](../plans/README.md) for remaining ownership.

## Scientific test input and support ownership

The native bounded document reader and package resolver derive exact package and data membership
from the authored scientific input catalog. Runtime tests capture and verify the inputs in their
own process. The explicit `xtask scientific-inputs` and `scientific-inputs-check` commands
export and verify the same capsule for separately selected consumers. Synthetic declaration
projections and independent reference executables retain distinct owners.

Compatible compiler tests may share an immutable admitted revision within one process and
publish it into fresh workspaces. Mutable runtimes, attempts, databases and borrowers retain
independent ownership. Native service linkage follows the declared capability. Native fixture
descriptors authenticate state identity and private ownership before effects.
