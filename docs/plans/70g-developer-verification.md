---
title: "Plan70g: Developer verification"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, validation, preparation]
review_sources: [docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md]
---

# Developer verification

**Interface-checked at `1835fcf3a`.** Grouped codegen already renders each selected target
once. Its comparison publishes the rendered bytes to a temporary tree and reads them back.
Hosted jobs have different preparation needs, but their shared action always builds and
authenticates the native host. The Buffa control executes two isolated Cargo projects, and the
recommended scope-end aggregates repeat two checks. The [coordinator](70-test-assurance-architecture.md)
owns finding dispositions and assembled acceptance. [Improvement review](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md)

**Proposed.** Keep each assertion with its existing owner and remove repeated work within the
same applicable inputs. A changed checkout, toolchain, feature selection, or provider context
requires evidence for that context. The savings in materialization and invocation count are
structural hypotheses. Measure elapsed benefit only for a named route and workload.

## <a id="g01"></a>G01 — Compare fresh generated output directly

For [I07](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i07), make the
validated `GeneratedTree.files` map the expected side of `xtask codegen --check`. Check candidate
paths before inventory so an invalid file/ancestor pair still fails after removing the temporary
writer. Apply the same exact-file and nested-root ownership rules to the expected path set and
the checkout inventory. Compare checkout bytes to the map by path. Keep missing, stale and
byte-different diagnoses, and preserve refusal of symlinks and non-file destinations. Git tracking
and later producers still report independent results when a tree drifts. The checkout comparison
has no publication effects. Production publication keeps its lease, staging, preimage checks,
durable writes and separate writer controls. Delete the comparison-only scratch writer and
readback once the direct comparison exercises the same cases. [Comparison and inventory](../../xtask/src/codegen.rs) ·
[Publication](../../xtask/src/codegen/publication.rs)

Consider three independent reductions once the comparator is available. Each has its own
adoption condition.

- **Git inventory.** Retain each tree's effective `git ls-files --others --exclude-standard --`
  scope and Git failure attribution. A common query can use the union of eligible pathspecs
  and `-z` to return verbatim NUL-terminated names within one checkout invocation. Attribute
  each returned path to all equivalent per-tree tracking scopes, including overlaps. Tracking's
  pathspec scope can differ from the checkout comparison's nested-root pruning. Compare
  per-tree outcomes for nested and exact roots, ignored and untracked entries, unusual path
  bytes and a failed Git invocation. Keep the individual queries if the union cannot preserve
  those scopes or the attribution would cost more machinery than it saves. [Git path output](https://git-scm.com/docs/git-ls-files#_output) ·
  [Current tracking query](../../xtask/src/codegen.rs)
- **Hosted projections.** Let one no-default `xtask` check invocation assemble the source
  registry once and run the existing Rust-contracts, docs and Surreal projections, each with
  its own generation, provenance, comparison and failure attribution. Prove equivalence with
  the three focused invocations, including a drift or generation failure in one projection
  while later projections still run. The Markdown projection must still run the source-built
  `pse commands --markdown` export. Adopt the grouped route for hosted `codegen-diff` if it uses
  the same locked correctness graph and each projection retains its inputs and checks. Keep
  individually selectable routes and the separate full native graph. Inspect the local
  `codegen-contracts-check` consumer before any migration. Its Rust recipe also checks recorded
  results under a different feature selection. [Hosted job](../../.github/workflows/rust.yml) ·
  [Projection loop](../../xtask/src/codegen.rs) · [Local recipes](../../justfile)
- **Physical fixture registry.** `pse_schema::catalog::assemble` creates the fresh authored
  registry for generation. When both sides of the physical loader currently assemble that same
  declaration and scope, give the engine binding the caller's already owned registry and keep
  package admission fresh. Retain the fingerprint comparison when the supplied registry really
  comes from the independently hydrated generated products through `pse_schema::registry()`.
  Exercise both receiving paths, including independent registry disagreement and physical
  inventory and package-refusal controls. [Source assembly](../../crates/pse-schema/src/catalog/mod.rs) ·
  [Generated registry](../../crates/pse-schema/src/lib.rs) · [Physical receiver](../../xtask/src/codegen/physical.rs) ·
  [Physical controls](../../xtask/src/codegen/physical/tests.rs)

Verify the direct comparison on an equivalent tracked tree, missing and stale outputs, byte
drift together with untracked output, invalid candidate roots or path conflicts, nested ownership
and symlinks. Verify focused `just codegen-bindgen-check` and use Q01's full `just codegen-check`
with their original native prerequisites so Ipopt and CoolProp remain in scope. Update
stale scratch-comparison descriptions with the replacement. Plan69e E05 retains full fresh
generation and regeneration comparison for guarded optional-field previews. That owner still
requires complete producer and output coverage, including aggregate ownership, in its private
snapshot. G01 changes how freshly rendered output is compared, independent of that selection
decision. [Plan69e E05](69e-source-model-and-live-schema-operations.md#e05--select-a-complete-generator-route-for-guarded-previews)

## <a id="g02"></a>G02 — Select hosted preparation by consumer

For [I08](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i08), split the
composite action's setup into independent selections for the pinned toolchain, platform linker,
Rust target cache and native host. Use composite step conditions on explicit caller inputs and
audit all current callers before selecting those inputs. `rust-toolchain.toml` remains the
toolchain authority. Direct Miri and direct `cargo doc` skip native host preparation while
retaining their actual linker, cache, toolchain and feature needs. The docs publisher still uses
the correctness adapter. Every adapter or host-backed tool resolver keeps explicit host
preparation. Keep cache keys separated by compatible graph and profile. More effective cache
restore could improve required setup, but removing preparation a job never consumes also helps
on a cold runner. [Composite action conditions](https://docs.github.com/en/actions/reference/workflows-and-actions/metadata-syntax#runsstepsif) ·
[Current setup](../../.github/actions/setup-rust/action.yml) · [Host bootstrap](../../scripts/bootstrap-host) ·
[Pinned toolchain](../../rust-toolchain.toml) · [Adapter launcher](../../scripts/cargo-bin/cargo-pse-correctness) ·
[Miri](../../.github/workflows/rust-scheduled.yml) · [Docs publisher](../../.github/workflows/docs.yml)

The `fmt` job needs `cargo fmt --all --check` and a pinned Taplo. Use the existing hosted
locked uv setup to install the declared `quality` group and invoke Taplo from that environment.
`uv sync --locked --only-group quality --no-install-project` checks the lock and selects the
tool group without installing the workspace project. The Taplo declaration and lock remain
the version authority. Use that path for the hosted Taplo-only lint consumer as well so it
uses the same pin. The format job can omit the linker, Rust target cache and host after cold
receiving checks prove both tools run and their respective drift is rejected. [uv lock and group semantics](https://docs.astral.sh/uv/concepts/projects/sync/) ·
[Hosted uv precedent](../../.github/workflows/governance.yml) · [Tool declaration](../../pyproject.toml) ·
[Tool lock](../../uv.lock) · [Hosted TOML lint](../../.github/workflows/repo-hygiene.yml)

The hosted family job currently repeats locked workspace metadata after `xtask family-check`.
That command already calls the pinned `cargo_metadata` 0.23.1 `MetadataCommand::other_options`
with `--locked` and uses the resolved graph. Cargo metadata fails for a missing lock or when
resolution requires changing it. Exercise the family owner against a deliberately stale
workspace lock with the xtask binary already built, then prove the retained hosted step also
fails and leaves the lock unchanged. On the valid path keep the family, ceiling and evidence-lock
comparisons. Remove the duplicate step only after the stale-lock case reaches the job verdict.
Check cold fmt, Miri and rustdoc outcomes and real adapter admission on the identity job.
Measure time by job phase only if claiming an elapsed benefit. [Pinned Rust API](https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.MetadataCommand.html#method.other_options) ·
[Cargo lock contract](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html#manifest-options) ·
[Family check](../../xtask/src/main.rs) · [Hosted Rust jobs](../../.github/workflows/rust.yml)

## <a id="g03"></a>G03 — Compare stock Buffa preparation on one control

For [I15](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i15), compare
the existing scratch stock-generator workspace with a small checkout-owned, explicitly locked
fixture for that stock tool alone. Keep it an isolated Cargo root so the parent workspace's
Buffa patches cannot replace the stock crates. A stock `buffa-codegen` 0.9.2 registry package
and its checksum establish which generator supplies the independent expectation. Check the
lock entry for that package's exact source and checksum. `cargo run --offline --locked` may
reuse an eligible fixture build in its own target directory, while every invocation passes the
current descriptor and runs the stock tool. Compare the stock default's output names and bytes
to those from the maintained `buffa_codegen::generate` with the same descriptor bytes,
`files_to_generate` and configuration. Exercise a changed descriptor or generation input and a
stock-default mismatch. [Stock generator API](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/fn.generate.html) ·
[Maintained API](../../vendor/buffa-codegen/src/lib.rs) · [Workspace patches](../../Cargo.toml) ·
[Current supplier control](../../vendor/buffa-codegen/tests/allocation_compile.rs)

Continue compiling the maintained decoder from current generated source and executing all
seven decoder controls, with zero failures, skips or compilation warnings. Keep the default
toggle and schema-name controls. A stored decoder or reused verdict would remove the fresh
generated-code evidence. Separate Cargo roots keep separate target directories. Adopt persistent
stock-tool preparation only if equivalent cold and warm runs show recurring stock compiler
cost worth retaining the separate fixture and lock. Attribute that cost separately from fresh
generation and decoder compile/test. If the comparison cannot establish that benefit or source
identity, record the deciding evidence and keep the scratch route. This inquiry is limited to
the existing first-party supplier receiver. [First-party receiver](../../crates/pse-codegen/tests/buffa_allocation.rs) ·
[Maintained profile](../../vendor/buffa-codegen/pse-patch.md)

## <a id="g04"></a>G04 — Compose scope-end checks under one prepared context

For [I16](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i16), Q01's
combined hygiene and governance selection should use `scripts/pse run-all hygiene governance-tests`
when both cover the same unchanged checkout and prepared context. `hygiene` already includes
`codegen-check` and `family-check`. The native bundle attempts each independent child and
retains its original verdict even when a sibling fails. Keep `just governance` as the standalone
aggregate for callers selecting governance by itself. Prefer this composition to storing check
verdicts, which would require a separate proof of applicability and invalidation. [Recipes](../../justfile) ·
[Bundle owner](../../crates/pse-host/src/cli/bundle.rs) ·
[Blueprint §24.1](../authoritative_design/sections/operations-and-validation.md#section-24-1)

Compare the old and proposed leaf populations and exercise independent generator and family
failures. Require one occurrence of each duplicate leaf in the combined route, preserved
governance-test coverage and individual nonpass propagation, including when hygiene fails
before the governance child starts. If source or preparation changes, rerun checks whose
evidence no longer applies. Reconcile Q01 and the scope-end guidance in the validation guide
and AGENTS.md with the verified composition. The guide's separate advice on grouping exact
Nextest filters belongs to its ordinary selection route. [Scope-end guide](../dev/validation-assessment.md#full-local-assessment) ·
[Repository execution rhythm](../../AGENTS.md#execution-rhythm-functional-scope-first-static-checks-at-scope-end) ·
[Q01](70-test-assurance-architecture.md#q01)

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [G01](#g01) | planned | Fresh rendered trees, existing tracking and publication owners, Plan69e E05 full-route boundary | Direct comparison passes ownership, drift and refusal controls and replaces the scratch comparator. Each conditional reduction is integrated with equivalent receiving evidence or has a recorded reason to retain its current route. |
| [G02](#g02) | planned | Hosted consumer inventory, pinned Rust and uv tools, existing host and adapter contracts | Cold hosted paths select their required preparation, Taplo consumes the lock and stale-lock refusal survives removal of duplicate metadata. If adopting G01's hosted grouping, coordinate the shared workflow edit after its grouped invocation is qualified. |
| [G03](#g03) | planned | Registry Buffa 0.9.2 source and checksum, maintained decoder fixture, separate Cargo roots | Complete the bounded comparison and preserve fresh decoder results. Adopt the independent locked stock fixture only with equivalent receiving evidence and measured recurring cost worth the maintenance. |
| [G04](#g04) | planned | Current recipe leaves and native keep-going bundle in one applicable checkout/context | Equivalent coverage and independent nonpass controls establish the combined scope-end route for Q01. Guidance reflects the verified composition. |
