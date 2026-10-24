---
title: "Plan 53d: Build, discovery and generation"
status: draft
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, preparation]
---

# Plan 53d: Build, discovery and generation

## Target and foundation

**Proposed:** eliminate demonstrated repeat preparation and incomplete reuse premises under [R05](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r05). The [coordinator](53-test-architecture-target.md#package-state-and-execution-route) owns state. Existing Cargo/nextest capture, `cargo_tools`, `python_extension`, `validation_scope`, codegen and actual Arrow coverage remain authorities. Prepared artifacts are reusable only under complete known consumed inputs; unknown scope prepares conservatively.

Plan52 already supplies literal cargo-hack enumeration, frozen ordered combinations/context and authenticated passing-origin resume. Preserve it and its failure/history/completeness distinctions; do not add an enumerator, cache or resume service. The closed `installation_context` used for server builds is useful precedent with a specifically qualified domain, not proof that Python preparation identity is solved. Ordinary `_rust_capture` already retains nextest binaries and Cargo metadata. ADR-0122 workspace feature unification is not broken; intentional validated/interactive/support variants remain legitimate.

## Work packages

<a id="d01"></a>
### D01 — Exact consumed tools and complete generation inputs

Extend `.config/diagnostic-tools.toml` and `cargo_tools.selected_identity` to own selected nextest version, executable path/digest and supported machine-schema compatibility. Migrate bootstrap, CI/installation, capture and reuse consumers to the same declaration/identity, preserving exact toolchain and Arrow family. A version string without authentic executable identity is insufficient. Test changed executable/version/config invalidates incompatible reuse and unsupported metadata refuses explicitly.

Correct `INPUT_SCOPES['generation']` to include the selected package inputs and fixture projection actually read by `xtask/src/codegen/physical.rs::load`, not just Rust/declaration inputs. B04's family closure can refine those edges when delivered; until then include the known sufficient package scope conservatively. Add/delete/package-only edits invalidate generation reuse. Preserve Plan52 combination identity semantics and actual Arrow leaf coverage; this package does not establish whole-campaign qualification.

<a id="d02"></a>
### D02 — One retained build/discovery path for equivalent correctness closures

Route the unprepared fallback through ordinary `_rust_capture`'s retained `--cargo-metadata`/`--binaries-metadata` path. Execution consumes the retained catalog/build artifact under its identity instead of repeating build/discovery. The [nextest archive interfaces](https://nexte.st/docs/ci-features/archiving/) are concrete enabling mechanisms already consumed locally, not a demand for a second archive service. Keep exact actual catalog/graph correspondence and incompatible-input refusal.

Use C01/C03 package/target/feature/validation facts to share genuinely equivalent correctness closures. B03 supplies intentional participation versus support distinctions; preserve interactive unvalidated extension and support-absent production-shaped checks. Assess actual Cargo graphs before deleting variant preparation. A matching feature spelling or file timestamp does not prove equivalent compiled behavior. Targeted script/selection controls show retained artifacts are consumed on repeat, meaningful feature/tool/config changes rebuild/refuse, actual Arrow leaves remain enabled and intentional variants remain distinct. Delete replaced fallback enumeration/build routes with their callers.

<a id="d03"></a>
### D03 — Split Python ensure, extension preparation, stubs and admission

Separate `python_extension` environment synchronization, extension build/install, stub generation and execution admission. Settle I06 by enumerating the actual supported installer-consumed Rust/declaration/tool/config/native inputs and installed binary identity, including unknown-input fallback and parent/worker mutable replacement windows. Use the existing source/provenance owner where its domain fits; do not assume the server's complete-context domain automatically covers Maturin/extension inputs.

When known consumed inputs and actual installed artifact are unchanged, pure Python execution need not rebuild the extension or regenerate unrelated stubs. A real Rust/declaration/tool/feature/config/native change or unknown scope prepares before use. Continue verifying mutable executable replacement at the real trust transitions; no stat/hardlink shortcut. C03 proves actual force-validation; preserve selected native/interactive and correctness distinctions, exact locks and generated `_native.pyi` ownership. Change the generator and regenerate through `just codegen`, never hand-edit generated outputs.

Acceptance uses targeted preparation/installer tests and an actual selected Python-boundary repeat: pure Python edit leaves compatible extension prepared; relevant Rust/config/feature/tool edit invalidates; changed binary between parent and worker refuses; unknown input prepares conservatively. Compare the actual admitted extension, not just a cached key. Remove redundant always-sync/install/stub paths and their duplicated validity decisions once migrated. This is not a full wheel/platform provenance campaign.

<a id="d04"></a>
### D04 — Necessary generation and narrow-before-read capture

Aggregate relation generation already includes physical/Rust contract work. Execute that overlap once with real root coverage; preserve independently callable recipes and Python candidate/native ABI checks, missing/extra/untracked outputs and nonmutating comparison. Keep mutating bootstrap ordering. Do not treat recipe names as evidence of independent required executions. Targeted codegen controls must expose a changed physical/package input and each retained output guard; delete only proven redundant aggregate calls.

In `validation.sources` and `rust_source_identity`, choose the semantically sufficient scope before opening or hashing bytes. Preserve whole-source provenance where promised and separate hashes at distinct mutable boundaries. Include symlink/mode, additions/deletions, absent lookups and `.envrc.local` meaning where consumed. B04 supplies scientific source edges; unknown scope remains broad. An out-of-scope file should not be opened, while in-scope content/membership/replacement must invalidate. Test these distinctions using bounded source fixtures, not a new repository census or per-file accounting system.

## Composition and completion

D01 may proceed before the whole C companion; C01/C03 then unblock D02/D03 correctness consumers. B04 precedes any narrowing that depends on scientific edges, but conservative sufficient scope permits earlier correctness repairs. D04 generation deduplication does not require Python reuse to be complete. Coordinate `.config/diagnostic-tools.toml`, manifests/locks, nextest, validation and codegen shared surfaces explicitly.

Completion requires actual retained-build/catalog and selected Python repeat evidence plus preserved generation/force-validation controls in 53g. Describe removed extra preparation/read/rewrite operations statically. Do not claim measured speedup, a universal build-equivalence interpreter or full wheel/scientific/platform qualification from these focused results.
