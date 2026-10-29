---
title: Qualification commands
status: current
---

# Qualification commands

Select checks that reveal the changed behavior. Native Cargo/nextest and the Rust performing
owners now supply correctness execution; the Python product, assessment/result ledger,
archive/reuse controller and measurement drivers are retired. Required remaining campaigns
belong to [Plan58](../plans/58-rust-owned-system.md), especially H. This guide describes
commands and evidence limits, not a claim that full qualification has completed.

## What each command establishes

| Command | Scope established by successful execution | Exclusions |
|---|---|---|
| `just check-package PACKAGE` | Selected package targets compile under the actual admitted graph | Test execution and scientific assertions |
| `just unit-package PACKAGE FILTER` | Selected source-built library tests under native nextest | Unselected targets, providers and services |
| `just unit-native-package PACKAGE FEATURES FILTER` | Selected native library tests with explicitly prepared providers | Canonical fixture ownership unless explicitly supplied |
| `just native-test MODULE_RECEIPT ...` | Selected source-built tests under one fresh native canonical fixture | Other deployments, historical state and unselected scientific cases |
| `just modeling-conformance ...` | Selected authored conformance runs in the manifest's supported native/pure mode | Unselected packages and independent reference execution |
| `just parity` | Four native candidate exchanges against the isolated pinned IDAES/Pyomo environment | Other cases, versions and exhaustive parity |
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

### Inspecting a product invocation

Nextest owns selection, execution and reporting. Retain the exact native command, source/build
selection and original output/report when evidence is needed. There is no current `just result`
reader or Python invocation reconciliation ledger. A readable report does not establish test
success, fixture drain or scientific assertion coverage. A complete failing population remains
a failure; missing selected outcomes cannot be inferred from a zero-exit helper.

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
just check-package pse-runtime
just unit-package pse-host 'test(correctness::)'
just test-package pse-ids --lib
PSE_NEXTEST_ACTION=list just unit-package pse-ids scientific_replay
just doctest
```

Nextest does not run doctests. `test-release` and `doctest-release` select the release profile;
that profile alone establishes no production performance. `check-native-test`,
`check-native-package`, `unit-native-package` and `unit-native-capability-package` expose the
current explicit feature/provider selectors. `just --usage RECIPE` is the grammar owner.

For a canonical consumer, first explicitly prepare/select the patched server and production
module using the [environment guide](agent-environment.md#explicit-compilerklu-preparation).
Save the absolute immutable module receipt. Then, for example:

```bash
just native-test /absolute/module.json \
  --build-arg=-p --build-arg=pse-operations --build-arg=--lib \
  --build-arg=--features --build-arg=canonical-tests,native-service \
  --build-arg=--locked -- -E 'test(canonical_server_unit)'
```

Choose the actual target/features/filter for the intended caller; a native solver target also
requires its actual solver features. The recipe explicitly builds the service/test helper.
`NativeTestCommand::prepare` runs source build/list through this checkout's prepared
correctness adapter before starting the fixture, then executes the exact resulting binary/Cargo
metadata. Build selectors are repeated `--build-arg`; arguments after `--` are run-only filters
and reports. Caller-supplied archive/metadata paths and run-time build selectors refuse.
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
order and keep-going; the adapter preserves selected features, targets, profiles and force-validation.
There is no combination resume cache or copied working tree. One driver owns overlapping Cargo
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
imports. `just parity` explicitly provisions `.venv-parity` from the parity group and exercises
native exchange callers. The pinned IDAES release is `idaes-pse==2.13.0`. `just reference-unit`
checks retained independent reference helpers. Missing/wrong reference prerequisites fail;
they are not skipped parity. Analytic/reference checks establish only their stated cases and
conditions; a local flash root is not a global stability certificate.

## Outside this scope

Other platforms, release/publication, full native deployment, complete powerset and performance
campaigns require their own acceptance. Architecture/scientific reviews are judgment at their
owning boundary. Retiring an orchestration mechanism does not discharge an unrun scientific or
H acceptance scenario. Use the [current work index](../plans/README.md) for remaining ownership.

## Scientific test input and support ownership

The native bounded document reader/package resolver derives exact package/data membership and
absence facts from the authored scientific input catalog. A synthetic declaration projection
is separately identified; it cannot qualify external data or an independent oracle. Tests
consume their declared input owner directly; no Python invocation-wide capture controller or
`PSE_SCIENTIFIC_INPUTS` preparation command remains.

Compatible compiler tests may share an immutable admitted revision within one process and
publish it into fresh workspaces. Mutable runtimes, attempts, databases and borrowers retain
independent ownership. `canonical-tests` selects actual test participation and `test-support`
selects fixture/hooks where declared. Native fixture descriptors authenticate state identity
and private ownership before effects; a copied path or a missing fixture never grants admission.
