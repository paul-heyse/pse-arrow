---
title: "Plan59c: Tests and fixtures"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, validation]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
---

# Tests and fixtures

Part of [Plan59](59-agent-cli.md). This plan adds intent-level `pse test`, `pse check` and
`pse tests` verbs that work out their own features, native capabilities, force-validation,
fixtures and filters. It replaces caller-assembled fixture inputs with resolution through the
actual fixture owners, and gives blueprint §24.1's reconciled-pass rule a Rust owner.
The remaining targets below are **Proposed**; the coordinator's paused 2026-10-30 checkpoint
and completed evidence below remain unchanged by this documentation amendment.

Sources: [O04](../functional_reviews/rust-agent-cli-2026-10-29.md#O04),
[O05](../functional_reviews/rust-agent-cli-2026-10-29.md#O05),
[O19](../functional_reviews/rust-agent-cli-2026-10-29.md#O19), and the fixture and test-route
items of [O11](../functional_reviews/rust-agent-cli-2026-10-29.md#O11).

The amendment adopts [new O05–O06](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O06)
and [I03](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I03),
[I07](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I07) and
[I08](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I08).

The 2026-10-30 revision carries the library-integration review's
[I01](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I01),
[I13](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I13),
[I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14) and
[I15](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I15).
The coordinator records the operator's 2026-10-30 RC01–RC04 decisions and decision routing;
this packet changes the remaining selection and consumer targets, not execution status.

Evidence from the review: in one post-retirement morning, agents made 194 inline `PSE_*`
assignments and chose between eight hand-picked `--native` capability sets. Tests ran through
four different entry forms, including 73 bare `cargo test` runs that skip force-validation.

## Baseline

The original code map is at `68e075493`; the 2026-10-29 amendment corrects fixture ownership
against `crates/pse-operations/src/native_fixture.rs`, its `pse-canonical-test` caller and
`crates/pse-runtime/tests/native_study.rs`.

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

**Two fixture ownership modes.** `pse-canonical-test` delegates to `NativeTestCommand`.
Its `prepare` source-builds/lists through the correctness adapter and captures Cargo/binary
metadata before creating a service; `run` lends `FixtureSelection` through
`PSE_NATIVE_CANONICAL_FIXTURE`. `ChildExitPremise` protects retained state after signal or
unknown scientific joins. Separately, `native_study.rs` and other targets construct their own
`NativeCanonicalService` from helper/module variables. A runner-owned service cannot replace
those preparation inputs. Neither ledger terminality nor child exit authorizes fixture disposal.

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
- **Staged selection.** C07 normalizes intent against captured declarations, admits compatible
  package/target roots through correctness, and later binds explicit build/list observations.
  Explain and execution consume the same value at the stage they can honestly expose.
  Compatibility permits one Cargo closure for several roots with shared metadata, not one
  scalar admission per package hidden behind concurrency. Preserve per-root required/default/
  absent-feature policies, host/target context, filters and ignored policy; split incompatible
  selections rather than union their features or approximate them with all-features.
- **Native capabilities.**
  1. Ask the adapter's unit graph which `pse-backend-native` features the selection reaches.
  2. Map them through the single declaration (below).
  3. Close the set with `providers::capabilities()`.
  4. Select the prepared providers. A missing provider is a typed `native.unprepared`
     diagnostic naming `pse native prepare --capabilities …`.
- **Force-validation.** Run through the correctness adapter, which adds the Arrow closure as
  today.
- **Fixtures.** Resolve declarations into the target's actual owner mode. Borrowing targets
  use `NativeTestCommand` and its typed selection; targets constructing their own service
  receive the captured helper/module inputs. Never create a redundant runner service or a
  second global fixture-permission authority.
- **Preparation.** Source build/list and immutable tool/helper capture precede service creation.
  A mutable `target/debug` path or fixture current pointer is discovery, not a pinned executable.
- **Running.** Through the [59b](59b-runs-and-continuity.md) run modes: foreground by default,
  `--detach` available, nextest libtest extraction.
- **Reconciled verdict** (blueprint §24.1 P00). A reconciled pass requires successful setup,
  the actual selected binary/test population, complete required records and terminal runner
  evidence. Missing evidence is `incomplete`; a known test/setup failure remains that failure.
  Preserve child exit, observation completeness, settlement and scientific acceptance separately.
  Launch acknowledgements and nonterminal waits may exit 0 under A03/B's command contract;
  they do not assert a reconciled workload pass. Raw adapter exits keep their existing meaning.
- **Rerun.** `--rerun-failed` uses recorded package, binary/target identity and exact test ID,
  with the original features/platform/policy. A same-named test in another binary is distinct.
  No ambiguous name-only filter or nextest `-R` substitutes for the captured selection.
- **Explain.** `--explain` renders the available resolved stage, redacts secrets while retaining
  captured configuration/value-source provenance, and distinguishes requirements, admission
  and actual observed population. It never builds/lists, prepares a provider or starts a
  fixture. Execution checks the captured premises before effects; a change requires explicit
  re-resolution or refusal rather than silently reinterpreting the request.

`pse check` does the same selection and features for `cargo check --message-format=json`, through
the adapter with cargo-message extraction.

`pse tests <pattern>` runs `nextest list --message-format json` through the adapter with the same
features `pse test` would use. Listing explicitly builds binaries; help/explain do not.
Reuse by a following test requires the captured immutable build identity to remain valid.
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
- Targets that need prepared state declare inputs and runner-borrowed or target-owned service
  mode in package metadata, keyed by target kind/name and feature context under
  `[package.metadata.pse.test.<target>]`, with an optional `when-features = [...]`. That
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
    native-service --bin pse-canonical-service`) and captures its immutable artifact identity;
  - `scientific-inputs` dispatches the federated `xtask scientific-inputs` and records the
    capsule;
  - evidence and tool directories come from the store.
- `pse test` supplies `PSE_NATIVE_FIXTURE_*` for target-owned services and other declared
  inputs from these records; runner-owned services lend `PSE_NATIVE_CANONICAL_FIXTURE` through
  the existing owner. Prepared tools, including xtask outputs, are captured before a later
  Cargo feature selection can overwrite their original output paths. Retention follows A05
  generation-use pins and the fixture owner's positive disposal decision.
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
| [C01](#c01) | complete | 59c | Declaration and actual reached-unit capability derivation integrated; targeted real no-build Cargo graphs cover every mapped feature and pure/full runtime; no provider preparation or solver build |
| [C02](#c02) | in-progress | 59c | Fixed authenticated managed helper, complete required raw capture and native reconciliation integrated; actual public listing succeeds. Public test/check execution and detach integration, fixture lifecycle, richer verdict/rerun consumers and recipe retirement remain |
| [C03](#c03) | in-progress | 59c | Native Gitoxide preview and conservative captured declaration closure pass6 controls; prepared CLI preview works and replaced xtask implementation/tests/recipe are deleted. `pse test --changed` integration remains |
| [C04](#c04) | in-progress | 59c | Source-audited runtime/operations declarations and passive existing-owner prerequisite selection integrated for both service modes; fixture CLI, immutable helper source-build admission, worker variable migration and actual service journeys remain |
| [C05](#c05) | in-progress | 59c | Source-build capture, complete phase custody and exact native binary/test inventory integrated; actual public `tests` journey retains the original inventory. Useful paged discovery/rerun consumers and `list-native-contracts` retirement remain |
| [C06](#c06) | deferred | 59c | Bindeps for the cross-package service helper; trigger below |
| [C07](#c07) | in-progress | 59c | Compatible multi-root Cargo groups, per-root absence policies, typed graph-driven splitting and invocation-local metadata reuse tested; staged CLI/list/execution consumers remain separate integration obligations |

<a id="c01"></a>
### C01 — One capability declaration

**Consumes:** [59a A02](59a-cli-foundation.md#a02) and C07's resolved feature/platform selection.

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

**Consumes:** C07, C01, [59a A03](59a-cli-foundation.md#a03), and [59b B01–B02](59b-runs-and-continuity.md)
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
- The reconciled verdict, constructed by the owning reconciler from actual setup, population,
  terminal and completeness evidence before A03 renders it. Typed decode and directional
  schemas do not independently establish a pass or permit invalid outcome combinations.
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
- One actual explain→execute journey with unchanged premises and one changed-premise refusal;
  an unavailable fixture/provider is explained without preparation.
- Reconciliation cases: nextest exit 0 with a missing selected record is incomplete;
  setup failure and zero-match selection remain explicit; duplicate test names across binaries
  and ignored-policy changes cannot produce an incorrect pass or rerun.
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

**Consumes:** C07, C02, [59a A04](59a-cli-foundation.md#a04) (store) and A05's immutable
artifact capture/generation-use pin contract before retaining helpers or pruning their generation.

**Delivers:**
- Fixture declarations for the targets and lib tests listed in the baseline, including both
  service ownership modes and their actual preparation inputs.
- `pse fixture prepare|show`.
- The study-module current pointer, written by `study-module prepare` through the store resolver.
- `pse test` setting fixture variables from the records.
- Borrowing targets use `NativeTestCommand` through `pse-canonical-test`, preserving source
  build/capture before service creation and `ChildExitPremise`. Target-owned service tests
  receive helper/module records without another runner service. Original owners settle and
  decide retained-state disposal even if the host invocation has become terminal.
- Bind the source-build correctness adapter and helper/tool metadata to A05's authenticated
  checkout/generation identity. Migrate current-pointer path checks where necessary so an
  immutable selected generation remains identifiable after a later build advances `current`;
  preserve the source-build admission check rather than trusting arbitrary recorded paths.
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
- Both runner-borrowed and target-owned service journeys receive their required inputs.
  Source-build failure creates no service; signal/unknown-join evidence preserves owner state.
- A later different-feature Cargo build cannot change an already selected helper/tool.

<a id="c05"></a>
### C05 — Test discovery

**Consumes:** C07, C01, C02 and C04's declaration/immutable-input slice. Actual fixture startup
is not a prerequisite to listing.

**Delivers:** `pse tests <pattern> [--package P]` over adapter `nextest list --message-format json`
with derived features, joined with declarations. Official nextest filtering/listing remains the
population authority. Map each listed binary back to its exact package/target root, admitted
context and immutable build identity, including multi-root groups; a package index or test name
alone cannot establish that mapping. Results are reusable only under that captured identity.

**Deletes:** the `list-native-contracts` recipe.

**Acceptance:** a pattern matching one integration test returns its package, target, features,
capabilities, fixtures and a `pse test` command that runs exactly that test.
Add compatible multi-package discovery with duplicate test names, an incompatible feature
context and a stale captured binary; matching/rerun must preserve the original binary/context.

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

<a id="c07"></a>
### C07 — Shared resolved selection foundation

**Consumes:** [59a A02](59a-cli-foundation.md#a02)'s typed command/configuration inputs and
the existing correctness adapter's selection/force-validation contract.

**Delivers:** staged test/check selection with three distinct responsibilities:

1. Normalize intent once from typed arguments/configuration and captured Cargo declarations:
   exact package/target roots, platform, features/defaults/absence, filter and ignored policy,
   provenance and preparation requirements. A missing provider or fixture remains a requirement.
2. Admit a set of compatible roots through the correctness adapter using shared metadata and
   selected unit-graph work. Adapt the existing single-package root validation so every admitted
   root maps to its intended package/target, with no unrelated roots and with each root's
   feature-absence constraints checked. Cargo's feature unification and actual Arrow closure
   govern admission. If combined admission would violate one intent, retain separate groups;
   running scalar calls in parallel does not remove the repeated preparation.
3. On explicit build/list, bind actual immutable compiler artifacts, binary identities and
   official nextest test population to that admitted context. C05 owns discovery mapping;
   B02 consumes the actual selected population and setup policy for reconciliation. Intent or
   a unit graph never fabricates a built binary or a test ID.

C01 derives capabilities from admitted units; C02 executes/checks the admitted selection;
C04 resolves declared fixture inputs without changing their owner; C05 discovers actual tests.
[59a A06](59a-cli-foundation.md#a06)'s explain consumes the same available stage and renders
later stages as pending. Generic non-test explain uses its own resolver rather than all C07/C04.
No universal operation registry or eagerly prepared provider set is introduced. Checked owner
constructors enforce valid stage transitions and selected-root mappings before execution;
Serde/Schemars views describe those values, not an independent selection interpretation.

**Failure closure and acceptance:** parser cases cover workspace exclusions, explicit targets,
incompatible required features, feature-absence controls and ignored tests. Exercise compatible
multi-root admission, mixed compatible/incompatible groups and host/target roots against independent
expected selections; retain negative-feature cases after actual Cargo unification. Explain and
execution agree for unchanged premises and refuse/re-resolve changed ones before effects. Dry
explain performs no build/list/provider/service preparation. C05's duplicate-name binary journey
challenges discovery identity independently of normalization. Structural repeated work is removed
when compatible roots share admission; elapsed-time improvement remains unmeasured.

## Verification and handoff

### Code-intelligence selection and candidate handoff

**C07** supplies the context slice used by G03/H/I. Analyzer cfg/metadata is not every compiler
unit: expose differences and ambiguous instances instead of silently selecting all features.
Actual compiler/probe actions retain correctness admission and force-validation obligations.
**C05** resolves G06 related-test candidates to actual binaries/features/ignored policy and
fixture-capable execution; raw analyzer runnables cannot bypass it. **C03** remains conservative
reverse reach; missing semantic references never establish sufficient reduced verification.

Acceptance adds duplicate-name/cfg-dependent candidates, missing macro preparation and a
candidate absent from the selected binary. Candidate, listed, selected and executed facts remain
distinct. Discovery need not wait on fixtures; actual execution requires working C05/C07.

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
