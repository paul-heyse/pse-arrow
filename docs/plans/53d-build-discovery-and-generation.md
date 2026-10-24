---
title: "Plan 53d: Build, discovery and generation"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, preparation]
---

# Plan 53d: Build, discovery and generation

## Target and foundation

**Proposed:** remove repeated preparation and incomplete reuse premises under
[R05](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r05) and
[PI01](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI01)/
[PI02](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI02).
The [coordinator](53-test-architecture-target.md#package-state-and-execution-route) owns progress.
Cargo owns compiled-artifact freshness; existing nextest capture, `cargo_tools`,
`python_extension`, `validation_scope`, codegen and actual Arrow coverage remain authorities.
Repeated Cargo invocation does not itself prove repeated compilation. Structural removal of
extra discovery, generation and byte reads needs no benchmark; speed claims require measurement.

**Interface-checked:** nextest 0.9.146 is already pinned and authenticated by
`cargo_tools.selected_identity`; capture, prepared inventory, archives, bootstrap and CI consume
that owner. B03/B04 and C01–C03 supply intentional support variants, sufficient scientific input
closures, normalized selections and actual validated Arrow leaves. Their delivery changes D01's
starting point, not its remaining compatibility/input obligations. Plan52's literal cargo-hack
combinations and authenticated passing-origin resume remain intact.

Completed 52k is the starting condition for subsequent execution. Loaded handler/schema/guidance,
current source facts, prepared/installed artifacts and invocation evidence are distinct identities.
The serving-source restart fence is not compiler-input completeness. Publish useful operations
through the canonical definitions, native contracts and derived references; add no handwritten
parallel CLI, second build cache or generic Cargo-equivalence interpreter.

## Work packages

<a id="d01"></a>
### D01 — Qualify existing tool identity and complete generation inputs

Retain the selected nextest declaration and executable version/path/digest checks. Qualify the
supported machine-metadata shapes consumed by preparation and replay; an exact supported pin
plus explicit shape checks is sufficient unless actual incompatible metadata establishes another
need. A generic schema/version registry adds no demonstrated value. Source tests that patch a
fixed identity are insufficient by themselves: changed executable bytes/version and unsupported
metadata must invalidate or refuse incompatible reuse through the real identity seam.

Complete `INPUT_SCOPES['generation']` with the package inputs and fixture projection consumed
by `physical::load`, including `packages/reference/fixture-projection.toml`. Start with known
sufficient package scope; B04 can justify narrower edges, but minimality is not a prerequisite.
Membership, absence and package-only changes invalidate. Preserve the exact toolchain, one Arrow
family, selected combination identity and C03's actual target/host/build/dev Arrow validation.
Delete duplicate declarations or validity decisions only where the existing owner replaces them.

**Acceptance:** targeted tool/prepared-inventory controls cover executable/version/config drift
and unsupported shape; a package-only edit and add/delete affect generation applicability.
No wider feature campaign is qualified merely by these controls.

<a id="d02"></a>
### D02 — One retained build/discovery path for equivalent correctness closures

Route the ordinary unprepared fallback in `select`/`test_run.run_rust` through
`ordinary_rust_capture`; execution consumes the retained Cargo metadata, binaries metadata and
exact catalogue instead of listing and then executing the original build/discovery command.
Separate preparation from canonical observer acquisition: local execution must not acquire a
store merely to reuse nextest's retained-build interfaces. The
[exact nextest 0.9.146 interface](https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.146/site/src/docs/ci-features/archiving.md)
supports metadata reuse, not proof of current source or auxiliary-input correspondence.

Use C01/C03's package/target/platform/feature/validation facts to share genuinely equivalent
correctness preparation. Preserve interactive unvalidated Python, support-enabled and
support-absent production-shaped checks. Matching feature names is not actual graph equivalence.
One preparation per invocation is the target; no persistent catalogue cache is selected.

**Acceptance:** ordinary local and canonical selections execute the retained catalogue; changed
source/config/tool/features prepare or refuse; actual Arrow leaves and executable associations
remain correct. Test intentional support variants. After callers migrate, delete the fallback
list→original-run route and its obsolete tests rather than preserve a compatibility runner.

<a id="d03"></a>
### D03 — Independent Python preparation operations and currency

Split the unconditional `python_extension.sync` composition at its real responsibilities. Expose
independently useful operations through the existing canonical operation definitions and derived
references, while retaining convenient explicit compositions for test and interactive workflows. Update the
existing `python-extension` inventory family and derived descriptions for independent native
routes; if a stage is callable, bind it at the existing canonical server/catalogue composition
owner. A native route does not become a callable tool merely by being discoverable.

| Responsibility | Currency and result |
|---|---|
| Dependency environment ensure/sync | Selected interpreter, locked dependencies and installation selection; determine currency before synchronization. Installed uv 0.13.5's [sync --check](https://docs.astral.sh/uv/reference/cli/#uv-sync--check) is an available mechanism, not extension freshness. |
| Extension prepare/install | Supported actual Cargo/Maturin selection and consumed Rust/declaration/config/tool/native inputs, bound to the actual installed artifact. |
| Stub generate/check | Generator and module/exception/recursive class declarations, dependency/tool/Ruff configuration, actual binary and output membership/content. Binary identity alone is insufficient. |
| Execution admission | Actual import origin, features and bytes at parent/worker transitions; admission remains fresh even after a valid preparation decision. |

Settle coordinator inquiry I06 before conditional skipping. Its decision product is the supported
ordinary editable dev/native correctness domain: interpreter/PyO3 and effective configuration,
consumed inputs including membership/absence, installed-byte identity/transformation, mutable
boundaries, and what remains unknown. Enumerate actual consumed inputs using existing owners;
do not claim precision from an all-Python key or whole-host scan. Missing receipts or unknown
inputs prepare conservatively; never mint a matching receipt from current state to bypass work.
Strict producer/diagnostic capture retains its own guarantees and is not reopened as a campaign.

Reuse `producer_deployment.bind_python_artifact`'s relevant editable RPATH transformation
semantics: raw Cargo and Maturin-installed bytes may differ legitimately. Bind imported/installed
bytes instead of requiring raw equality or importing the full strict producer campaign. Existing
Maturin 1.15.0 was verified locally in the review; no choice relies on unavailable exact-release
upstream source or newer main-branch options. Preserve C03's actual graph validation before
correctness installation, exact locks, native/interactive modes and generated stub ownership.
Generator changes publish through `just codegen`, never direct edits to generated outputs.

**Acceptance:** a pure Python edit preserves the compatible extension and unrelated stubs;
Rust/config/feature/tool/native changes prepare; stub-only input changes regenerate/check stubs;
unknown input prepares; parent-to-worker binary replacement refuses. Exercise a real selected
installed Python-boundary repeat as well as targeted installer/stage controls. Discovery stays
source-only; execution retains workload admission. A changed extension does not refresh a loaded
endpoint: consume completed 52k's supported snapshot/restart contract for relevant changes.
Remove unconditional stage coupling and duplicated decisions after all actual callers migrate.

<a id="d04"></a>
### D04 — Necessary generation and scope-before-read capture

In aggregate comparison, remove only physical/Rust-contract production already covered by the
relations route. Preserve independent recipes and real output-root coverage, Python candidate
versus published-output checks where distinct, actual API stubs, native ABI checks and
missing/extra/untracked-output guards. Preserve ordered mutating bootstrap. Replacing every check
with full-codegen would activate unrelated native capabilities and is not selected.

At the existing `validation.sources` boundary, select sufficient scope before opening or hashing
bytes; migrate `rust_source_identity` and other narrow callers. Whole-source provenance and
archives still capture their promised domain. Preserve modes, symlinks and consumed resolved
targets, additions/deletions, absence and `.envrc.local` where consumed. Enumeration may be shared
within one capture; observations at real mutable boundaries remain independent. B04 supplies
scientific edges, and unknown scope remains broad. No repository-wide incremental index is needed.

**Acceptance:** a changed package/physical input reaches the right generation root; aggregate
comparison does not repeat covered production; all retained guards and standalone capabilities
remain effective. Scoped capture never opens an unrelated fixture file, while relevant content,
membership and replacement invalidate. Delete only proven duplicate aggregate calls and
full-before-filter capture paths, migrating their tests to the retained behavioral distinctions.

## Composition and completion

D01's compatibility/input corrections and D04's sufficient-scope repairs can proceed independently
of Python reuse. D02 consumes delivered C01/C03/B03; D03's conditional skipping consumes I06 and
C03. Canonical exposure follows completed 52k, not an interim interface. Coordinate shared
manifests/locks, nextest, validation/codegen and capability declarations with their writers.

53g qualifies retained-build/Python repeats and the relevant schema/restart journey. Completion
requires migrated callers, replacement deletion and preserved input/graph/generation guarantees;
it establishes neither a universal installer domain nor full wheel/platform/scientific qualification.
