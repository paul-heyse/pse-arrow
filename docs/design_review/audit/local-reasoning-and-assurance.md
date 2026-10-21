# Local reasoning and assurance

Findings about what a developer or agent must read, instantiate or keep consistent by hand
to change a responsibility safely. Baseline and limits are in the
[collection README](README.md). Citations are `path:line` at source commit `a791c533e`.

## <a id="da-021"></a>DA-021 — Domain values carry the whole composition root, so effects are reachable from every value and policy is tested through the full runtime

**Principles:** AP-01, AP-05, AP-06, DP-17, DP-18, DP-23; PSE-S05.

**Diagnosis.** `workflow::Runtime` bundles the shared pool and `MathService`, the bound
DataFusion model, the canonical SurrealDB deployment, durability and operations, and the
physical cache (`crates/pse-runtime/src/workflow/mod.rs:370-380`). Domain values store it
as a field, among them:
- `ModelingPackage` (`workflow/modeling.rs:117-129`);
- `FitProblem` (`workflow/fitting.rs:374-378`);
- `RunResult`, studies, simulations and diagnostics.

Any method on a package or result can therefore reach storage, engine, math admission and
native pools. Effect boundaries are conventions inside each method (`admit_activity()`),
not something the types declare.

`ModelingPackage` is in effect a service locator. Its inherent impls are spread over 31
files covering source persistence, preparation, dynamics, diagnostics, conformance,
fitting, shooting, studies and strategies. `workflow` itself has 74 flat sibling modules
and about 301 `pub(in crate::workflow)` items.

Route and policy decisions read service handles from the value. For example, LM
eligibility reads `p.runtime.shared.budget()`
(`workflow/fitting/oracle/residual.rs:37-57`). A test that a fit routes to LM must
therefore build a runtime, physical context, package admission and compilation
(`fitting/oracle/lm_inquiry.rs:136-175`). Other tests follow the same pattern:
- Study-admission tests build sources and, under `canonical-tests`, a durable runtime
  (`study_tests.rs:55-90`).
- Study-operation admission tests require all native solvers (`study_operations.rs:1749-1751`).
- Covariance withholding rules are methods on the full `FitProblem` over a solved
  `FitReport` (`workflow/fitting/covariance.rs:157-345`), so every test of them needs
  `prepare_fit` and linked native solvers (`covariance_tests.rs:25-30`).

**Module coupling.** Fitting is mostly separable from modeling. The one inversion is that
modeling's canonical persistence stores `FitDeclarations` typed in the fitting execution
module (`modeling/canonical.rs:564`, `:1634`). That creates a module cycle with inverted
ownership.

**Consequence.**
- Changing a policy safely requires reading an owner's whole service context.
- Policy and admission cannot be exercised with only their actual inputs.
- The effect profile of an operation is not discoverable from its signature.

**Counterevidence.**
- Pure policy cores exist and are tested in isolation:
  - `pse-model/src/study_policy.rs` (effect-free, 17 tests);
  - `math/strategy.rs` (pure admission and transition with an injected effect boundary);
  - backend routing takes an injectable `Table`.
- `Runtime` clones are cheap `Arc`s.
- §1.1 declares the runtime as composition root; the issue is how far it reaches into
  values, not that it exists.

**Evidence strength.** *Implemented*. The cited tests show the test-setup consequence.

**Related.** [DA-015](boundaries-and-composition.md#da-015),
[DA-010](structure-and-invariants.md#da-010).

## <a id="da-022"></a>DA-022 — Memory admission depends on hand-maintained size models of the project's own structures and of native libraries' internals

**Principles:** DP-20, DP-16, DP-15, AP-02, AP-07 (H26, H27); PSE-S06.

**Diagnosis.** Admission budgets and Salsa `heap_size` are fed by hand-written estimators
with ad hoc per-entry constants. These are pervasive across `pse-math`, `pse-compiler`,
`pse-modeling` and runtime math: over a hundred functions named `heap_size`,
`retained_bytes` or similar, plus many inline estimates. Examples:
- `crates/pse-compiler/src/workspace/modeling/executable/grouped.rs:57-97`
- `executable.rs:2063-2110`
- `executable/solve.rs:394-405`, `:567-571`

Some runtime estimators run serialization passes only to size buffers (for example
`crates/pse-runtime/src/workflow/replay.rs:1026-1068`, per study point). Runtime code also
charges DataFusion pool memory down to individual metadata records and futures, with 145
`MemoryConsumer::new` sites in `pse-runtime` (for example `workflow/durable.rs:615-645`;
`workflow/connected_results.rs:29-57`).

Pre-charging memory before native allocation also uses project-written models of library
internals:
- SUNDIALS allocation shapes (`crates/pse-backend-native/src/kinsol.rs:1070-1135`);
- the Uno allowance projection ([DA-018](boundaries-and-composition.md#da-018));
- IDAS BBD allowances (`dynamics/idas/bbd.rs:40`, `:77`);
- the FERAL storage profile (§18.8).

**Consequence.**
- Every structural change and every native library upgrade must re-derive these formulas.
- Nothing detects drift: there is no measuring-allocator comparison.
- A library upgrade can make a pre-charge non-conservative without any typed check failing.

**Counterevidence.**
- Some native allowances are tested against the ABI (`bbd.rs:559`; `kinsol.rs:2264`, `:2385`).
- Library size queries are used where they exist (`KINGetWorkSpace`, `N_VSpace`).
- The declared design is candid that allowances do not bound RSS (§14, §18.8).
- There is a concrete need, given past editor-wide OOM incidents.

**Evidence strength.** The duplication is *Implemented*. Staleness risk is a hypothesis.

**Open library question.** Whether a derive-based heap-size crate can serve the Salsa
`heap_size` and admission estimates for the project's own structures has not been
researched. Salsa users such as ruff and ty are believed to use `get-size2`; unverified.

## <a id="da-023"></a>DA-023 — The architecture contract interleaves current design with amendment and plan status, and lags the code in places that guide extension

**Principles:** DP-24, DP-01, AP-05.

**Diagnosis.** Several architecture sections have grown into amendment logs.
- **§14 preparation contract.** It carries 28 Decision/Supplement/Proposed/Implemented
  blocks and about 40 references to plan packets and review IDs (for example
  `docs/authoritative_design/sections/mathematics-and-compilation.md:559-617`, `:1172-1194`).
- **§13, §19 and §18.6–§18.8.** They interleave proposed-target ADR blocks with implemented
  contracts.

The current contract, proposed targets and live status cannot be read as a compact map.
The text diverges from code in places that guide extension:
- §0.5, §1.3, §3.3 and §18.9 omit linked backends and runtime-held libraries
  ([DA-017](boundaries-and-composition.md#da-017)).
- §3.3 calls nalgebra an optional conformance dependency, but it is a production runtime
  dependency for LM. levenberg-marquardt, argmin, egobox, statrs and ndarray appear in no
  section.
- §3.2 misdescribes `pse-py`'s dependencies ([DA-015](boundaries-and-composition.md#da-015)).
- The §4.2 generated-tree table places document adapters in an empty, untracked
  `pse-runtime/src/generated/`; they now live in
  `crates/pse-admission/src/generated/document_adapters.rs`. The table also omits the
  admission, schema and columnar generated trees.
- §13.5 names a nonexistent `PreparedSimulation::rebind` ([DA-014](boundaries-and-composition.md#da-014)).
- §7.1 states one body per definition ([DA-028](execution-fit.md#da-028)).
- §7 says there is no project-owned arithmetic interpreter. But
  `crates/pse-modeling/src/specialize/value.rs:1010-1090` is a static f64 interpreter that
  computes parameter defaults used as case inputs. That is defensible for static data, but
  undeclared.

**The set of generated paths is stated in six places that disagree.** These are codegen
`roots()` (`crates/pse-codegen/src/codegen/mod.rs:77-97`), AGENTS.md prime directive 2,
`.claude/rules/generated.md`, `scripts/agent-hooks.py:59`, `.claude/settings.json:29` and
§4.2. Several of them still list the absent `crates/pse-operations-queries/`, and codegen
still declares a `pse-catalog` root.

**Consequence.** A newcomer or agent following the contract for PSE-S02 or for an IR
change cannot find live owners. A change to one statement of the generated-path set leaves
the others stale.

**Counterevidence.** In code, the capability rows and generated trees each have one
authority, checked by tests or by `codegen-check`. The text lags behind sound code
authorities.

**Trivial drift, recorded without findings.**
- §3.1 says three dependency ceilings; `Cargo.toml` has four.
- `mathematics-and-compilation.md:535` mentions Delta `CHECK`s, but Delta is retired.
- §1.3 lists petgraph only under `pse-structural`.
- The §3.2 ceiling prose names crates the ceiling roots omit. The lockfile shows no
  violation.
- `tests/engine` declares `pse-runtime` and `pse-authoring` as dev-dependencies used only
  through its `force-validate` feature list (`tests/engine/Cargo.toml:19-47`).

**Evidence strength.** *Interface-checked*: text compared with manifests and code.

## <a id="da-024"></a>DA-024 — Bespoke machinery is maintained without a production consumer

**Principles:** DP-16 (bounded exploration purpose), §C (flexibility without a credible
variation axis), H27.

**Diagnosis.**
- **Native child-process lane.** `crates/pse-runtime/src/math/native_child.rs` (918
  lines: child-process protocol, authenticated IPC, KINSOL fingerprint gating, reaping)
  has a parent side constructed only by its tests (`native_child_tests.rs:258`, `:378`,
  `:600`). Its child side ships as the product binary `xtask/src/bin/pse-native-lane.rs`
  (declared at `xtask/Cargo.toml:135-136`), which serves `native_child::serve_private`.
  `NativeChild` and `solve_portable_root` have no other caller in `crates/`, `python/` or
  `xtask/` (coordinator-confirmed for `crates/`; the `xtask/` binary was identified in
  challenge review). §18.8 and ADR-0185 (proposed) declare it as a target.
- **Relation canonical hashing.** `pse.canon.v2` (about 1,400 lines in
  `crates/pse-columnar/src/canon` plus about 1,300 test lines) and its `pse-relations`
  wrapper (`crates/pse-relations/src/columnar.rs:1531-1566`) have no production caller. §5.3
  acknowledges this under "Limits".

**Consequence.** These are bespoke protocols and codecs that carry build, test and upgrade
obligations without delivering a supported behaviour. The library-eligibility rule ("no
consumer needed") applies to adopted libraries, not to bespoke machinery.

**Counterevidence.** Both are declared targets with stated purposes. Whether either has a
bounded exploration purpose is a maintainer judgment that this audit does not settle.

**Evidence strength.** *Implemented*, from a production-caller search across `crates/`,
`python/` and `xtask/`.

**Related.** [DA-025](execution-fit.md#da-025) (the online file collector likewise has no
production caller).
