---
title: "Plan59c: Tests and fixtures"
status: active
date: 2026-10-30
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, validation]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md]
---

# Tests and fixtures

Part of [Plan59](59-agent-cli.md). This plan adds intent-level `pse test`, `pse check` and
`pse tests` verbs that work out their own features, native capabilities, force-validation,
fixtures and filters. It replaces caller-assembled fixture inputs with resolution through the
actual fixture owners, and gives blueprint §24.1's reconciled-pass rule a Rust owner.
The remaining targets below are **Proposed**; the coordinator's paused 2026-10-30 checkpoint
and completed evidence below remain unchanged by this documentation amendment.

The current target incorporates O02–O08/O12/O14 of the
[CLI, test and runtime workflow review](../functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md).
The operator confirmed native completed-report reconciliation on 2026-10-30: independent
event-by-event start/terminal verification is intentionally retired. A09 publishes that
contract change before dependent implementation. Research and test-target design can proceed
without waiting for publication. This document owns test integration, not another runner,
scientific authority or process supervisor.

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

The detailed original map below is historical. At the new review's source baseline
`26ac0b893`, C01 was complete and C02–C05/C07 were partially integrated as recorded in the
package table. The current Nextest event reconciler remains implemented until replaced;
planning its removal does not erase its focused evidence. The recorded reader selection
compiled `pse-runtime --lib` for 3m56s, then ran 18 selected tests in 6.454s. This establishes
a broad compile unit followed by narrow execution, not a cache miss or CPU-width defect.

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
  synchronous foreground by default and explicit `--detach` for persistent work. The selected
  test owner composes outcomes in either mode; ordinary tests do not require a generic service.
- **Reconciled verdict** (blueprint §24.1 P00). A reconciled pass requires successful setup,
  the actual selected binary/test population, original native exit, an invocation-owned
  completed report and required capture/fixture evidence. Use native Nextest JUnit rather
  than reconstructing its experimental event protocol. Missing evidence is `incomplete`;
  a known test/setup failure remains that failure. No independent event ordering or per-event
  byte-address guarantee remains; original raw streams retain their separate fidelity contract.
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
- Existing fixture inputs include:
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
  Cargo feature selection can overwrite their original output paths. A05 retains immutable
  prepared generations without automatic collection; actual fixture state and resource
  disposal remain with the fixture owner's positive decision.
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
  artifact dependencies (`-Z bindeps`) are a reopened C06 comparison, not an assumed improvement.
  Their unit graphs must preserve force-validation and feature-absence controls without
  preparing the helper for unrelated targets.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [C01](#c01) | complete | 59c | Declaration and actual reached-unit capability derivation integrated; targeted real no-build Cargo graphs cover every mapped feature and pure/full runtime; no provider preparation or solver build |
| [C02](#c02) | in-progress | 59c | Existing managed helper, phase capture and event reconciliation have focused evidence. Native completed-report replacement/isolation, synchronous ownership integration, fixture lifecycle, failure/rerun consumers and recipe retirement remain; prior event checks are not the new target |
| [C03](#c03) | in-progress | 59c | Native Gitoxide preview and conservative captured declaration closure pass6 controls; prepared CLI preview works and replaced xtask implementation/tests/recipe are deleted. `pse test --changed` integration remains |
| [C04](#c04) | in-progress | 59c | Source-audited runtime/operations declarations and passive existing-owner prerequisite selection integrated for both service modes; fixture CLI, immutable helper source-build admission, worker variable migration and actual service journeys remain |
| [C05](#c05) | in-progress | 59c | Source-build capture, complete phase custody and exact native binary/test inventory integrated. Public `tests` retains the inventory; source-built `result ID --tests` reads whole-record pages with byte-bound continuation and stale/foreign cursor refusal. Fixture/capability presentation, exact rerun consumers and `list-native-contracts` retirement remain |
| [C06](#c06) | planned | 59c | Reopened bounded comparison of selective helper construction and Cargo artifact composition; no adoption or timing benefit established |
| [C07](#c07) | in-progress | 59c | Compatible groups, absence policies and metadata reuse have focused evidence; remaining scheduling delegates to Nextest and explicitly sequences incompatible groups needing shared exclusion |
| [C08](#c08) | planned | 59c | Public reader feedback target and useful native compilation explanation; dependency-boundary investigation only where needed |
| [C09](#c09) | planned | 59c | Focused real-CLI assertions, meaningful case identities and adversarial framing assurance using established test utilities |

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

**Consumes:** C07, C01, [59a A03](59a-cli-foundation.md#a03), A09's completed-report contract,
and [59b B01](59b-runs-and-continuity.md#b01)'s shared foreground capture. B02 consumes
C02's completed domain result for presentation; its full delivery is not a prerequisite to
the report-reader decision. Integrate the actual result consumer in the same performing change.
It also uses [B03](59b-runs-and-continuity.md#b03) for
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
- Producer-isolated JUnit for every invocation/compiler group. The current profile-shared
  `junit.xml` path cannot identify its writer; copying it after execution is insufficient.
  Set report placement before launch and retain effective configuration/provenance. Resolve
  `junit.path` against the selected profile store, preserving repository configuration
  precedence. If a derived captured config is needed, change only placement and refuse
  unsupported precedence before effects; a tool-default file cannot silently override policy.
- Native inventory from `nextest-metadata` and exact `(compiler context, binary ID, test name)`
  reconciliation against the completed report. Setup suites and skipped populations remain
  distinct; retry/flaky details and original runner exit retain native policy.
- A bounded typed-reader decision: qualify a selected exact `quick-junit` release's `Report`,
  `TestCaseStatus`, `TestRerun` and deserialization against the pinned runner. Its API is a
  candidate, not an adopted pin. Unsupported/malformed/missing reports remain incomplete.
  Do not replace the event interpreter with a handwritten XML event interpreter or embed
  `nextest-runner` merely to retain unnecessary lifecycle reconstruction.
- Bounded failed-case/setup/runner pages and original stdout/stderr links under B02. A result
  read preserves the performing failure at the top level. Rerun is explicit; changed source,
  features or external fixture premises are reported, not silently reinterpreted. Retained
  binaries do not freeze scientific inputs. Native record/replay and timeline attachments
  remain conditional on a selected compatible runner; replay is not fresh-source validation.
- `--rerun-failed`.
- `--explain`.

**Deletes:**
- The replaced `NextestExtractor` suite/event state, retry-name reconstruction, ignored-tail
  compatibility and mechanism-only tests; retain Cargo diagnostic extraction and raw streams.
- Parallel failure briefs/history formats that merely restate the same native result.
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
- Two concurrent report producers cannot consume each other's report; crash/cancellation and
  malformed/partial XML never establish pass. Include setup, retry/flaky and native nonzero
  exit even where individual report entries look successful. Report completion never proves
  physical fixture teardown or scientific acceptance. These cases qualify the selected reader.
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

**Consumes:** C07's captured population, C01 requirements, [59a A04](59a-cli-foundation.md#a04)
(store) and A05's immutable artifact identity. C02 supplies actual execution/result integration;
passive declaration/input resolution need not wait for its completed report reader.

**Delivers:**
- Fixture declarations for the targets and lib tests listed in the baseline, including both
  service ownership modes and their actual preparation inputs.
- Build/list once, derive exact selected fixture demand and prepare only required inputs.
  Pure schema/framing/preflight tests need no canonical service or solver provider. Reuse
  `NativeTestCommand`/`NativeCanonicalService` performing owners; do not duplicate them in the
  host or rewrite ordinary test bodies to serve CLI control. Nextest setup scripts may export
  inputs where appropriate but do not automatically become an external service teardown owner.
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
- Pure and mixed selections show actual demand; a support-absent control retains absence.
  The existing build/list handoff reuses binaries/Cargo metadata rather than compiling again.

<a id="c05"></a>
### C05 — Test discovery

**Consumes:** C07's shared source-build/capture stage, C01 and C04's declaration/immutable-input
slice. Listing is independent of C02 execution/reconciliation and actual fixture startup;
C02 consumes this observed inventory rather than forming a reverse prerequisite.

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
### C06 — Selective helper construction or Cargo artifact composition

**Consumes:** C04's actual helper demand and existing correctness selection contract. Research
can begin before fixture CLI delivery; adoption needs its working selected consumers.

**Delivers:** a bounded decision comparing the existing selective helper build plus test build
with one Cargo DAG. Two Cargo invocations do not establish duplicate expensive compilation.
For artifact dependencies, examine `-Z bindeps`, `artifact = "bin:pse-canonical-service"` and
`lib = true` only if library use is needed. Cross-package access is
`CARGO_BIN_FILE_<DEP>_<NAME>`, not same-package `CARGO_BIN_EXE_*`.
See the [Cargo artifact contract](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies).

Compare a feature-selected optional artifact edge with retaining selective construction;
package-wide dev dependencies can build helpers for unrelated test targets. A dedicated harness
package is conditional on a responsibility that warrants a new crate and its decision route.
Do not combine arbitrary helper-bin/runtime-lib roots with global selectors or implement a
custom Cargo-equivalence planner. Preserve immutable captured binaries, source-built provenance,
host/target/artifact force-validation, feature unification and support-absent controls.

**Deletes if adopted:** only the replaced helper-build invocation/path handoff and duplicated
capture. Existing fixture service lifecycle remains where required. If exact selected demand
cannot be preserved, retain C04's selective build and record the limiting contract here.

**Acceptance:** a pure reader builds no helper/provider solely because of composition; the
canonical consumer receives the exact source-built helper; validation and negative-feature
cases remain effective. No timing saving or repeated Arrow-build elimination is presumed.

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

Delegate scheduling, groups, retries, threads, timeouts and fail-fast to one Nextest invocation
for compatible roots. Native named groups are not cross-process semaphores. For incompatible
contexts needing common exclusion, explicitly sequence only those affected groups; do not add
a second scheduler or host-wide serial policy. Keep ordinary 16 physical lanes/unbounded
placement, with actual scientific timing/exclusive fixture contracts preserved.

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
An exclusive fixture cannot overlap across split groups; independent tests retain native
parallelism. Existing semantics, not a general CPU reduction, justify any sequencing.

<a id="c08"></a>
### C08 — Reader feedback and compilation explanations

**Consumes:** existing public `tooling::reader::{encode,serve,schema}` and C07's selected target
context. C04 is needed only for canonical assertions; pure reader design is independent of
B03 or the new report reader. Boundary changes requiring a new crate consume A09's decision route.

**Delivers:** a named public-contract test target for reader encoding/framing/schema, with
independent decoding/expected values; retain genuinely private invariants beside implementation.
Inspect Cargo's automatic eligible-bin build for same-package integration tests before choosing
the exact target. Moving tests removes unrelated unit-test bodies, not ordinary runtime
compiler/mathematics/DataFusion/telemetry dependencies. Compare module/feature extraction only
where a real production responsibility narrows that closure. No new public test-only accessors,
`#[path]` recompilation or unproven unused-dependency removals.

Extend existing result presentation with phase durations, effective profile/features/targets,
raw Cargo links and an opt-in native `--timings` attachment. Explain check metadata versus
executable codegen/link versus running tests. Shared cache totals cannot attribute a miss to
one invocation. Keep workspace incremental/dependency nonincremental choices unless a selected
feedback profile's compile/runtime trade justifies another artifact population; that profile
cannot substitute for production-shaped scientific/performance evidence.

A05 owns the smaller optional bootstrap simplification: compare one source-coherent constructor
with the current shell build plus bootstrap build. It is not an explanation for the recorded
runtime compile. No pre-test Clippy warming, new cache ledger, smaller CPU partition or cache-hit
campaign is required. Native [Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html)
answer a selected critical-path question; sccache cannot cache incremental Rust compilations
or final linker outputs under its documented 0.19 restrictions.

**Deletes:** migrated duplicate unit tests, obsolete fixture identities and any duplicate
construction invocation whose coherence duties are preserved by A05's chosen producer.

**Acceptance:** edit framing and select the public target without a database/provider or the
unrelated runtime unit-test executable; independent malformed/truncated/identity cases reject.
Canonical history assertions still exercise their real service. A check→test journey explains
different compilation work without claiming cache failure. Structural reduction is sufficient;
numerical speed or hit-rate claims need a separate measurement.

<a id="c09"></a>
### C09 — Focused CLI and adversarial test assurance

**Consumes:** existing real-binary `pse-host` tests and the selected behavior delivered by
C02/B01–B03. Framing assurance can proceed independently using existing stream owners.

**Delivers:** selectively replace repetitive process assertion helpers with `assert_cmd` and
predicates; `Assert::new(Output)` can use an existing fixture's process output. Keep semantic
JSON/schema/exit assertions and existing insta presentation snapshots. Consider snapbox only
for meaningful filesystem effects, trycmd for an actual executable-example corpus, rstest for
usefully selectable static matrices, datatest-stable for per-file identities and libtest-mimic
only for genuinely dynamic harness cases. Do not convert cheap loops or economical Clap tree
snapshots into unnecessary process populations.

Extend existing Proptest at byte framing boundaries for arbitrary chunks, EOF, malformed data
and bounds, checking conservation/truncation against independent expectations. Keep useful
counterexamples. Do not invest in the disappearing Nextest event machine. Real kernel/fixture
tests retain process ownership assertions: assertion-tool timeouts do not prove descendant drain.
Scientific expectations/reference routes remain independent under PS-10/PS-13.

Loom/Shuttle require a small explicit modeled responsibility; Miri, coverage, mutation, CPU/
allocation profilers and tracing assertions remain selected inquiries, not new gates or CLI
commands. No test-history database or universal test-support registry is selected.

**Deletes:** replaced ad hoc assertion helpers, duplicate snapshots/case registries and one-off
journey scripts once durable tests cover their actual behavior. Preserve unique original evidence.

**Acceptance:** malformed invocation has the expected structured error/exit/streams and no
effects; reading incomplete/failed work preserves the performing failure; generated chunk/EOF
cases reject known bad input and retain useful minimized counterexamples when found. Actual foreground/detached cancellation
journeys stay with their physical owner. Utility adoption does not qualify unexercised science.

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
