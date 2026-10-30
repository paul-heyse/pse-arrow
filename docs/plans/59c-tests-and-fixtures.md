---
title: "Plan59c: Tests and fixtures"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, validation]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md]
---

# Tests and fixtures

Part of [Plan59](59-agent-cli.md). This plan adds intent-level `pse test`, `pse check` and
`pse tests` verbs that work out their own features, native capabilities, force-validation,
fixtures and filters. It also adds fixture resolution that removes hand-plumbed environment
variables and `/tmp` receipt paths, and it gives blueprint §24.1's reconciled-pass rule a Rust
owner.

Sources: [O04](../functional_reviews/rust-agent-cli-2026-10-29.md#O04),
[O05](../functional_reviews/rust-agent-cli-2026-10-29.md#O05),
[O19](../functional_reviews/rust-agent-cli-2026-10-29.md#O19), and the fixture and test-route
items of [O11](../functional_reviews/rust-agent-cli-2026-10-29.md#O11).

Evidence from the review: in one post-retirement morning, agents made 194 inline `PSE_*`
assignments and chose between eight hand-picked `--native` capability sets. Tests ran through
four different entry forms, including 73 bare `cargo test` runs that skip force-validation.

## Baseline

The following facts come from a code map at `68e075493`.

**`affected`.**
- `xtask/src/development/affected.rs` has 538 lines plus 264 lines of tests, and is nearly
  self-contained: it uses `anyhow`, `cargo_metadata` (`no_deps`), `serde`, `serde_json`, `toml`,
  git subprocesses and `tests/fixtures/scientific-inputs.toml`.
- It has hard-coded generator and Python path lists, a `NATIVE_PACKAGES` exclusion list, and the
  `affected-preview-v1` output.
- It emits a ready nextest command without native capabilities, fixtures or integration targets.
- Its only consumer is the `just affected` recipe, which has had no measured use.

**Native capability knowledge.**
- `pse-host`'s `native/providers.rs` owns the capability vocabulary and its closure:
  - the vocabulary is `ORDER`, `required` and `capabilities()`;
  - `klu`, `isolation`, `uno` and `petsc` each imply `compiler`;
  - `uno` and `petsc` each imply `solver`.
- `pse-backend-native/build.rs` reads `CARGO_FEATURE_*` (`ROOT_ISOLATION`, `IPOPT`, `SDP`,
  `CLARABEL_PARDISO`, `UNO`, `PETSC`, `SCIP`, `KINSOL`, `IDAS`, `DIFFSOL`) and the matching
  provider environment.
- `pse-runtime`'s `solver-*` features map one-to-one onto backend features.
- The feature→capability pairing is declared nowhere. It lives as literals in about twenty
  justfile lines, for example `solver,klu,isolation,uno,petsc`.

**Integration targets.**
- `pse-runtime` has about thirty `[[test]]` targets with `required-features`, and `pse-operations`
  has `native_service` (`native-service`).
- Fixture variables are read with hard `expect`s:
  - `PSE_NATIVE_FIXTURE_{SERVICE_HELPER,MODULE_RECEIPT,MODULE_STATE}` in seven `pse-runtime`
    integration targets, two `pse-runtime` lib tests and one `pse-operations` lib module;
  - `PSE_WORKER_BINARY` in two integration targets and one lib test;
  - `PSE_NATIVE_STUDY_WORKER` in `worker`;
  - `PSE_SCIENTIFIC_INPUTS` through the included `tests/fixtures/scientific-inputs.rs`;
  - `PSE_CANONICAL_TOOLS` and `PSE_DIAGNOSTIC_EVIDENCE` in `pse-operations` lib tests.
- Only `worker-test` sets anything: `PSE_NATIVE_STUDY_WORKER`, from a preset `PSE_WORKER_BINARY`.

**Built binaries.**
- `native_service` already uses `env!("CARGO_BIN_EXE_pse-canonical-service")`.
- `worker`, `native_worker_lifecycle` and `native_boundary_acceptance` already require
  `product-operators`, so `env!("CARGO_BIN_EXE_pse-worker")` works for them on stable Cargo.
- The pse-runtime helper consumers are cross-package: `pse-canonical-service` lives in
  `pse-operations`.

**Receipts.** `pse-host study-module prepare` is idempotent and content-addressed. It prints
receipt JSON but no path, and keeps no current pointer.

**Database runner.** `pse-canonical-test` (clap) owns fresh-service-per-run database tests and
sets the fixture variables for its child. It takes `--root`, `--correctness`, `--nextest`,
`--helper`, `--module-receipt`, `--state-parent`, `--build-arg…`, the timeouts, and
`-- run_args`. Only `just native-test`/`canonical-test` compose it.

**Correctness adapter.**
- `resolve_with` returns the force-validation feature splice and resolved packages.
- It supports `nextest run|list` with features.
- It refuses `-R`, `--rerun` and `--tool-config-file`.

**Entry forms.** `just unit-package`, `test-package`, `unit-native-package`,
`unit-native-capability-package`, `unit-native-selected`, `check-package`, `worker-test`,
`native-test`, `canonical-test` and `list-native-contracts`, plus `cargo t` and bare
`cargo test`.

## Target

`pse test` takes a selection and does the rest:
- **Selection.** Positional `pkg[:target]` targets (combinable with `clap-cargo`'s `-p`), or
  `--workspace [--exclude …]`, or `--changed [--base REF]`. Bare words become `test(word)`
  filters; full filtersets pass through `-E`.
- **Features.** Read each target's `required-features` from Cargo metadata and add the caller's
  `--features`.
- **Native capabilities.**
  1. Ask the adapter's unit graph which `pse-backend-native` features the selection reaches.
  2. Map them through the single declaration (below).
  3. Close the set with `providers::capabilities()`.
  4. Select the prepared providers. A missing provider is a typed `native.unprepared`
     diagnostic naming `pse native prepare --capabilities …`.
- **Force-validation.** Run through the correctness adapter, which adds the Arrow closure as
  today.
- **Fixtures.** Resolve and prepare what the target declares (below), and set the variables for
  the child.
- **Database tests.** Targets that need a fresh canonical service go through `pse-canonical-test`
  rather than reimplementing its lifecycle.
- **Running.** Through the [59b](59b-runs-and-continuity.md) run modes: foreground by default,
  `--detach` available, nextest libtest extraction.
- **Reconciled verdict** (blueprint §24.1 P00). The verb exits 0 only when nextest exits 0 and
  the libtest records cover every selected test, ending in a terminal record. Anything else is
  `incomplete` (exit 1), with `child_exit` recorded.
- **Rerun.** `--rerun-failed` reruns the failed IDs from the run's own ledger as an exact
  `test(=…)` filterset, since nextest's `-R` is refused and races.
- **Explain.** `--explain` prints all of the above without effects.

`pse check` does the same selection and features for `cargo check --message-format=json`, through
the adapter with cargo-message extraction.

`pse tests <pattern>` runs `nextest list --message-format json` through the adapter with the same
features `pse test` would use. Listing builds the binaries, so a following `pse test` reuses them.
It joins the results with target and fixture declarations, and returns each match with its
package, target, required features, capabilities, fixtures and exact `pse test` command.

**Capability declaration (one authority).**
- `crates/pse-backend-native/Cargo.toml` gains `[package.metadata.pse.native-capabilities]`,
  mapping each backend feature that needs a prepared provider to its capabilities:
  - `ipopt`, `scip`, `sdp` and `clarabel-pardiso` → `solver`;
  - `kinsol`, `idas` and `diffsol` → `klu`;
  - `root-isolation` → `isolation`;
  - `uno` → `uno`, and `petsc` → `petsc`.

  The final mapping is confirmed against `build.rs`.
- `pse` reads the table through `cargo_metadata`. It is the one declaration of which
  *capabilities* to prepare.
- `build.rs` stays keyed on `CARGO_FEATURE_*`, because it decides link inputs per feature. Having
  it read the table would only add a `Cargo.toml` rerun trigger on the heaviest crate.
- The justfile's literal capability lists retire with the recipes they belong to.

**Fixture declaration.**
- Targets that need prepared state declare it in their package's metadata, keyed by target *and*
  feature: `[package.metadata.pse.test.<target>]`, with an optional `when-features = [...]`. That
  keeps canonical database lib subsets, selected by `canonical-tests` plus a filter, apart from
  ordinary `--lib` runs.
- Fixture names include:
  - `canonical-service`;
  - `study-module` (`production` or `qualification`);
  - `scientific-inputs`;
  - `worker`;
  - `diagnostic-evidence`;
  - `canonical-tools`;
  - `producer-identity` (the `xtask producer-identity-fixture` receipt that
    `canonical-portable-test` needs).
- `pse fixture prepare|show <name>` owns preparation:
  - `study-module` wraps the idempotent `study-module prepare` and records a current pointer at
    `<store>/fixtures/study-module.json`;
  - `canonical-service` builds the bin through the adapter (`-p pse-operations --features
    native-service --bin pse-canonical-service`) and records its path;
  - `scientific-inputs` dispatches the federated `xtask scientific-inputs` and records the
    capsule;
  - evidence and tool directories come from the store.
- `pse test` sets `PSE_NATIVE_FIXTURE_*`, `PSE_SCIENTIFIC_INPUTS`, `PSE_DIAGNOSTIC_EVIDENCE` and
  `PSE_CANONICAL_TOOLS` from these records.
- Historical module-state selectors (`…_MODULE_STATE`, `…_PREDECESSOR_…`, `…_TARGET_…`) are
  recorded inputs. They are explicit `pse fixture` and `pse test` arguments, never defaults.

### Decisions with reasons

- **`affected` moves into `pse-host`.** It needs only crates the host already has, plus git. An
  xtask dispatch would pull xtask's unconditional `pse-schema`/`pse-columnar`/`pse-codegen`
  closure into every `pse test --changed`.
  - Its hard-coded generator path list is replaced by the generated-outputs map
    ([59d D06](59d-knowledge-and-workflow-verbs.md#d06)), so the guard, `owner` and `affected`
    share one declaration.
  - Its `NATIVE_PACKAGES` exclusion is replaced by the capability declaration.
  - `anyhow` becomes `HostError`.
- **The unit graph decides capabilities, not package names.** It reports which backend features a
  selection actually activates. Package membership overstates the need: `pse-runtime` without
  solver features needs no providers.
- **Use `CARGO_BIN_EXE` wherever Cargo provides it.** That covers same-package integration
  targets on stable Cargo. Cross-package helpers and lib tests use fixture records. Cargo
  artifact dependencies (`-Z bindeps`) come last and only for the cross-package helper, because
  their features must also pass ADR-0170's force-validation unit-graph check and interact with
  ADR-0122 feature unification.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [C01](#c01) | planned | 59c | Capability declaration on `pse-backend-native` (build.rs unchanged); unit-graph capability derivation |
| [C02](#c02) | planned | 59c | `pse test` and `pse check` with selection, features, capabilities, force-validation, reconciled verdict, explain and rerun; replaced recipes retired |
| [C03](#c03) | planned | 59c | `affected` in `pse-host`; `pse test --changed`; xtask `affected` and its recipe retired |
| [C04](#c04) | planned | 59c | Fixture declarations, `pse fixture`, receipt pointer, `CARGO_BIN_EXE_pse-worker`; database targets through `pse-canonical-test`; `worker-test`/`native-test`/`canonical-test` retired |
| [C05](#c05) | planned | 59c | `pse tests <pattern>` discovery; `list-native-contracts` retired |
| [C06](#c06) | deferred | 59c | Bindeps for the cross-package service helper; trigger below |

<a id="c01"></a>
### C01 — One capability declaration

**Consumes:** [59a A02](59a-cli-foundation.md#a02).

**Delivers:**
- The `[package.metadata.pse.native-capabilities]` table on `pse-backend-native`.
- A `pse-host` reader joins it with the adapter's unit graph (`resolve_with` and the graph
  capture) and closes the result with `providers::capabilities()`.
- `build.rs` is unchanged.

**Acceptance:**
- **Per-feature derivation.** One derivation test for each backend feature in the table, plus
  these combinations:
  - `pse-runtime` with `native-solvers` derives the full set;
  - `pse-runtime` without solver features derives none.
- No solver build or provider preparation is part of this package's acceptance. The derivation
  tests establish the mapping, and Plan59 does not change the solver build. A real build run
  during execution had to prepare providers from a worktree, which repointed the shared provider
  cache away from main; it was removed from acceptance.

<a id="c02"></a>
### C02 — `pse test` and `pse check`

**Consumes:** C01, [59a A03](59a-cli-foundation.md#a03), and [59b B01–B02](59b-runs-and-continuity.md)
(the foreground runs and extractors). It also uses [B03](59b-runs-and-continuity.md#b03) for
`--detach` when present.

Its recipe-retirement documentation edits include the architecture section
`operations-and-validation.md` (§24.1 Commands), so they follow
[59d D06](59d-knowledge-and-workflow-verbs.md#d06) (R4).

**Delivers:**
- Both verbs with `SelectionArgs` (positional targets, `clap-cargo` `Workspace`/`Features`, an
  explicit selection group that keeps `--workspace --exclude` and `-p … -E` legal) and
  `PlacementArgs`.
- Required-feature and capability derivation.
- The reconciled verdict.
- `--rerun-failed`.
- `--explain`.

**Deletes:**
- The `unit-package`, `test-package`, `check-package`, `check-library`, `unit-native-package`,
  `unit-native-capability-package` and `unit-native-selected` recipes, plus their literal
  capability lists.
- The documentation and skill routes that name them, which [59e E03](59e-environment-and-adoption.md#e03)
  updates in the same change.

**Acceptance:**
- `try_parse_from` selection cases.
- An `--explain` snapshot for `pse-runtime:native_study`.
- A reconciliation unit test: a nextest exit 0 with a missing selected record yields
  `incomplete` and exit 1.
- **Force-validation.** It is present for a small Arrow-consuming target and absent for
  `pse-host`, which has no Arrow, both through the real adapter. A passing run yields the
  reconciled verdict.

<a id="c03"></a>
### C03 — `affected` in the host

**Consumes:** C02, and the generated-outputs map from [59d D06](59d-knowledge-and-workflow-verbs.md#d06).
That map lists generator *inputs* per target as well as outputs, so `affected` can widen on a
generator change.

**Delivers:**
- `affected.rs` and its tests move into `pse-host`, with `HostError` in place of `anyhow`.
- `pse test --changed [--base REF]` and `pse check --changed` use its reverse-dependency
  selection. They widen to integration targets and native packages through C01, instead of
  listing them as exclusions.
- `pse affected --json` keeps the preview, versioned as an envelope.

**Deletes:** xtask's `affected` subcommand, `xtask/src/development/affected*.rs`, and the
`just affected` recipe.

**Acceptance:** the moved tests pass in `pse-host`, and a change in `pse-backend-native` selects
its dependents with the derived capabilities.

<a id="c04"></a>
### C04 — Fixtures

**Consumes:** C02, and [59a A04](59a-cli-foundation.md#a04) (store).

**Delivers:**
- Fixture declarations for the targets and lib tests listed in the baseline.
- `pse fixture prepare|show`.
- The study-module current pointer, written by `study-module prepare` through the store resolver.
- `pse test` setting fixture variables from the records.
- Database-backed targets run through `pse-canonical-test` with its arguments filled from the
  records.
- `worker`, `native_worker_lifecycle` and `native_boundary_acceptance` switch to
  `env!("CARGO_BIN_EXE_pse-worker")` and drop `PSE_WORKER_BINARY`/`PSE_NATIVE_STUDY_WORKER`. The
  lib test in `study_tests.rs` keeps its variable, supplied by its `lib` fixture declaration.

**Deletes:** the `worker-test`, `native-test`, `canonical-test` and `canonical-portable-test`
recipe bodies. Their scenarios become `pse test` invocations.

**Acceptance:**
- `pse test pse-operations:native_service` and `pse test pse-runtime:native_study` run with no
  caller-set fixture variables.
- A missing preparation yields `fixture.unprepared` with the exact prepare command in `next`.
- A second `pse fixture prepare study-module` reuses the cached module.

<a id="c05"></a>
### C05 — Test discovery

**Consumes:** C01, C02, C04.

**Delivers:** `pse tests <pattern> [--package P]` over adapter `nextest list --message-format json`
with derived features, joined with declarations. Results are cached per build identity in the
store.

**Deletes:** the `list-native-contracts` recipe.

**Acceptance:** a pattern matching one integration test returns its package, target, features,
capabilities, fixtures and a `pse test` command that runs exactly that test.

<a id="c06"></a>
### C06 — Cross-package helper by artifact dependency (deferred)

**Trigger.** Revisit if the `canonical-service` fixture from C04 proves burdensome in use, or if
Cargo stabilizes artifact dependencies.

**Consumes:** C04.

**Delivers, when triggered:** a focused probe of `-Z bindeps` for `pse-runtime`'s dev-dependency
on the `pse-canonical-service` artifact.
- **Criteria for adoption:**
  - the correctness adapter's unit-graph check and force-validation splice accept artifact edges;
  - workspace feature unification gives one feature set;
  - sccache and nextest behave normally.
- **If the criteria hold:** helper consumers switch to
  `env!("CARGO_BIN_EXE_pse-canonical-service")` (artifact form) and drop the
  `canonical-service` fixture.
- **Otherwise:** the C04 fixture remains the route, and this package records why.

**Acceptance:** the decision and its evidence are recorded here. If adopted, the cross-package
targets run without the fixture variable.

## Verification and handoff

During packages:
- compile checks and targeted tests on `pse-host`, plus small real adapter runs on non-solver
  targets;
- the integration targets named in C04, run once each through `pse test` in main against the
  providers already prepared there, to show fixture resolution. Nothing in this series prepares
  native providers or re-verifies the solver build.

The broader scientific and integration journeys belong to series scope end, as AGENTS.md
describes.

[59e](59e-environment-and-adoption.md) carries the routing and documentation updates for every
retired recipe in the same change as its retirement.
