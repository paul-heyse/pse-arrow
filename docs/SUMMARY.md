# Summary

[pse-arrow](README.md)
[Relationship to IDAES](relationship-to-idaes.md)

# Authoritative design

- [What "authoritative" means](authoritative_design/README.md)
  - [Architecture blueprint](authoritative_design/blueprint.md)
  - [Proposal (historical)](authoritative_design/proposal.md)

# Decisions

- [Decision records](adr/README.md)
  - [Deferred-decision register](adr/register.md)
  - [ADR template](adr/template.md)
<!-- adr:begin -->
  - [ADR-0001: Record decisions as ADRs with the charter §H fields](adr/0001-adr-process-and-decision-records.md)
  - [ADR-0002: Name the project pse-arrow, prefix crates pse-, dual-license MIT OR Apache-2.0](adr/0002-project-name-license-and-distribution.md)
  - [ADR-0003: Re-implement IDAES clean-room and pin parity to idaes-pse 2.12.0](adr/0003-clean-room-relationship-and-parity-pin.md)
  - [ADR-0004: Adopt D1: typed Arrow relations are the only model authority](adr/0004-typed-relations-are-the-only-authority.md)
  - [ADR-0005: Adopt D2: authors write primitive facts, passes derive everything else](adr/0005-author-causes-derive-consequences.md)
  - [ADR-0006: Adopt D3: one catalog holds CanonicalModel, CanonicalMathGraph and CanonicalMathProblem](adr/0006-three-artifact-levels-one-schema-system.md)
  - [ADR-0007: Adopt D4: semantic IDs, artifact ordinals and content hashes are three different things](adr/0007-identity-kinds-are-distinct.md)
  - [ADR-0008: Adopt D5: every quantity carries a quantity_type_id, and pse.* extension types are registered](adr/0008-physical-type-is-more-than-a-unit-string.md)
  - [ADR-0009: Adopt D6: the math IR is relations, and DataFusion Expr computes over it](adr/0009-math-ir-is-richer-than-datafusion-expr.md)
  - [ADR-0010: Adopt D7: balance, costing and utility laws are templates expanded over a contributions substrate](adr/0010-laws-are-templates-over-contributions.md)
  - [ADR-0011: Adopt D8: a pass computes the property closure; inspection never constructs physics](adr/0011-property-demand-is-resolved-explicitly.md)
  - [ADR-0012: Adopt D9: every constitutive computation is a KernelSpec with generated bindings](adr/0012-kernels-have-one-contract-and-generated-adapters.md)
  - [ADR-0013: Adopt D10: DataFusion has four roles and never runs inside a Newton iteration](adr/0013-datafusion-has-four-roles.md)
  - [ADR-0014: Adopt D11: execution layouts are compiled artifacts and zero-copy is a preference](adr/0014-native-numerics-own-execution-layouts.md)
  - [ADR-0015: Adopt D12: one generated Pyomo adapter, no IDAES class hierarchy](adr/0015-pyomo-is-a-generated-coarse-grained-backend.md)
  - [ADR-0016: Adopt D13: initialization, homotopy and fix-then-release are immutable case overlays](adr/0016-cases-and-results-never-mutate-the-model.md)
  - [ADR-0017: Adopt D14: memoize on declared inputs, the pass version and the engine profile](adr/0017-incrementality-follows-declared-dependencies.md)
  - [ADR-0018: Pin both families with =, commit the lockfile, run cargo deny, and set MSRV to the pinned toolchain](adr/0018-dependency-pins-lockfile-and-msrv.md)
  - [ADR-0019: Record the DataFusion plan fingerprint as evidence; keep it out of the memo key](adr/0019-plan-fingerprint-is-evidence-not-a-memo-key.md)
  - [ADR-0020: Defer salsa; the artifact-hash memo is the only memoization mechanism](adr/0020-salsa-deferred-behind-the-artifact-hash-memo.md)
  - [ADR-0021: Load YAML with serde-saphyr; serde_yaml is banned](adr/0021-serde-saphyr-replaces-serde-yaml.md)
  - [ADR-0022: Write every kernel against num-dual 0.15; FeOs stays conditional](adr/0022-num-dual-0-15-and-conditional-feos.md)
  - [ADR-0023: Give pse-ids sole ownership of hashing, with derive_key contexts and pse.canon.v1](adr/0023-blake3-derive-key-in-pse-ids-and-pse-canon-v1.md)
  - [ADR-0024: Cross the Python boundary with pyo3-arrow and the PyCapsule stream protocol](adr/0024-pyo3-arrow-over-arrow-pyarrow.md)
  - [ADR-0025: Defer LogicalPlan::Extension rule nodes; derivations already answer the attribution question](adr/0025-logicalplan-extension-rule-nodes-deferred.md)
  - [ADR-0026: Drop uom and arrow-flight; pint validates what the adapter emits and never defines a quantity](adr/0026-drop-uom-and-arrow-flight-pint-validates.md)
  - [ADR-0027: Define the commit contract as P2-level validity; compilability is gated separately](adr/0027-commit-contract-is-p2-validity.md)
  - [ADR-0028: Build Ipopt 3.14.20 with MUMPS and ASL from pinned sources in a container; probe HSL at runtime](adr/0028-solver-acquisition-source-built-ipopt.md)
  - [ADR-0029: Give every session a FairSpillPool sized from a declared memory limit](adr/0029-fair-spill-pool-with-an-explicit-limit.md)
  - [ADR-0030: Canonicalize NaN and preserve -0.0 on the hashing path; never use Float64 as a distinct or join key](adr/0030-canonical-float-hashing-and-no-float-keys.md)
  - [ADR-0031: Commit every generated source and diff it in CI; never generate in build.rs](adr/0031-generated-sources-committed-and-diff-checked.md)
  - [ADR-0032: Fetch external reading copies into a gitignored external/ rather than vendoring them](adr/0032-external-checkouts-are-not-vendored.md)
  - [ADR-0033: Keep one blueprint file, revised in git, with section numbers as stable citation targets](adr/0033-one-blueprint-file-with-stable-section-numbers.md)
  - [ADR-0034: Govern with a single maintainer, PR-only squash merges onto a linear signed main](adr/0034-governance-single-maintainer-pr-only.md)
  - [ADR-0035: Update dependencies with Dependabot, grouped by the pinned families](adr/0035-dependabot-with-family-groups.md)
  - [ADR-0036: Publish the documentation as an mdBook on GitHub Pages](adr/0036-mdbook-on-github-pages.md)
  - [ADR-0037: Defer datafusion-tracing, instrumented-object-store, datafusion-ffi and egglog, each with a trigger](adr/0037-remaining-deferred-items-with-triggers.md)
  - [ADR-0038: Add pse-ipopt-sys, pse-buildinfo, xtask, benches and five test crates to the workspace layout](adr/0038-workspace-layout-additions.md)
  - [ADR-0039: Enforce semantic admission and complete physical quantity operations](adr/0039-active-semantic-admission-and-quantity-algebra.md)
  - [ADR-0040: Resolve property demand from normalized seeds and name immutable stage outputs](adr/0040-demand-seeds-and-stage-bundles.md)
  - [ADR-0041: Reuse complete stage inputs before introducing finer memoization](adr/0041-complete-stage-cache-inputs.md)
  - [ADR-0042: Defer finer memoization until whole-stage measurements justify it](adr/0042-defer-fine-grained-memoization.md)
  - [ADR-0043: Declare kernel outcomes and generate only supported bindings](adr/0043-complete-kernel-outcomes-and-bindings.md)
  - [ADR-0044: Record plan encodings as noncanonical diagnostic evidence](adr/0044-noncanonical-plan-evidence.md)
  - [ADR-0045: Separate canonical logical content from encoded artifact integrity](adr/0045-canonical-content-and-encoded-integrity.md)
  - [ADR-0046: Share runtime budgets and reserve platform allocations explicitly](adr/0046-shared-accounted-runtime-memory.md)
  - [ADR-0047: Preserve guarded numerical semantics and borrow through safe Arrow views](adr/0047-guarded-numerics-and-safe-arrow-borrows.md)
  - [ADR-0048: Use standard relational operators and explicit-schema readers](adr/0048-standard-relational-and-import-operators.md)
<!-- adr:end -->

# Plans

- [Plans](plans/README.md)
  - [01 — Repository configuration](plans/01-repository-configuration.md)
  - [02 — Blueprint revision 5 contracts](plans/02-blueprint-revision-5-contracts.md)

# Design reviews

- [Data model design charter](design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md)
  - [Design review template](design_review/design_principles/DESIGN_REVIEW_TEMPLATE.md)
  - [Agent design directive](design_review/design_principles/AGENT_DESIGN_DIRECTIVE.md)
- [Review: blueprint revision 1](design_review/reviews/design_review_arrow-native-idaes-core-blueprint_2026-09-13.md)
- [Review: blueprint revision 2](design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md)
- [Review: blueprint revision 4 and library contracts](design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md)
  - [Revision-4 library evidence: probe receipts](design_review/evidence/blueprint-rev4-2026-09-13/README.md)
- [Review: blueprint revision 5 contracts](design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md)

# Capability maps

- [Arrow (Rust)](capability-maps/arrow-rust.md)
- [DataFusion (Rust)](capability-maps/datafusion-rust.md)
- [Supporting Rust libraries](capability-maps/supporting-rust-libraries.md)
- [Python libraries](capability-maps/python-libraries.md)
- [Evidence](capability-maps/evidence/README.md)
  - [Rust evidence](capability-maps/evidence/rust/README.md)
  - [Python evidence](capability-maps/evidence/python/README.md)

# Development

- [CI job graph](dev/ci.md)
