# Scoped Arrow validation target

**Date:** 2026-10-09  
**Tier / purpose:** DESIGN / TARGET  
**Boundary:** The correctness-invocation governance and Cargo feature composition proposed by
[ADR-0170](../../adr/0170-scoped-arrow-validation.md), covering selected package/target roots,
resolved dependency closures, opt-in propagation, recipes and bare-tool routes.  
**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and
the [pse-arrow binding](../design_principles/binding/pse-arrow.md).  
**Baseline:** `4c24721e691187e1a5b28398b29722fbde671da8`, proposed ADR-0170, and the inspected
working-tree command surfaces on 2026-10-09. `scripts/select.py` has concurrent execution-effects
changes; `justfile` and `scripts/validation_scope.py` also have concurrent changes. The existing
forced relation-root mechanism remains visible in those sources. Those changes are preserved
and are not implementation evidence for ADR-0170.  
**Reviewer:** Independent delegated design reviewer; no product implementation or checks performed.  
**Disposition owner:** [Plan 33 EFF00/EFF02](../../plans/33-efficiency-principles-remediation.md#eff02).

**Architectural fitness: Accept at Proposed target-design strength. Behavioral/semantic
adequacy: Accept for the specified closure and validation contracts. Overall decision: Accept.**

The target replaces an invocation-wide feature spelling with a guarantee about the selected
work: every actual Arrow-consuming correctness closure must receive full validation, while
pure responsibilities need not select an unrelated relation test target. That is the appropriate
assurance boundary. Cargo's declared and resolved dependencies remain authoritative; opt-in
features on actual owners carry activation. Unknown or uncovered consumption refuses rather
than silently receiving weaker checks.

The coordinator supplied two pinned Cargo refusals and a successful direct-dependency feature
observation. They support the conditional rule change, but were not rerun by this reviewer.
This review accepts the target design, not the current recipes, a completed implementation,
measured build improvement or scientific qualification.

## 1. Scope, drivers and coverage

The target serves local semantic, compiler, native-adapter and Arrow-boundary development
without enlarging Cargo's selected test roots solely to activate validation. It preserves
actual consumer validation, pinned dependency/toolchain authority, the one Arrow type universe
and intended feature sharing when routes alternate. It does not change native execution effects,
canonical service admission, numerical tolerance, resource ownership or test outcome handling.

Inspected sources include `scripts/select.py`, `scripts/affected.py`, relevant `justfile`
recipes, `.cargo/config.toml`, `.config/hakari.toml`, workspace manifest validation metadata,
quantity/modeling/math/model/columnar/schema/codegen/operations/runtime/relations/testkit manifests,
`xtask` governance composition and governance feature witnesses. Relevant owners are blueprint
§24.1, AGENTS.md's execution/invariant rules and the shared Rust/tooling rules. The selected
toolchain is `nightly-2026-09-29`; Arrow is pinned to 59.3.0. No dependency or toolchain change
is proposed.

The source review's [F04](design_review_efficiency-principles-codebase_2026-10-09.md#f04)
remains the current diagnosis. This is a bounded decision review, not a repeat of its codebase
assessment or a request to resume the cancelled feature matrix.

## 2. Ownership and composition

| Owner | Responsibility and invariant | Boundary |
|---|---|---|
| Package manifests and pinned Cargo resolution | Actual dependency kinds, active features, selected roots and target/host contexts | One authoritative dependency graph; no independent per-test Arrow classifier |
| Actual Arrow boundary owners | Opt-in validation propagation through existing dependencies | Feature wiring changes with a new consumed boundary; pure crates gain no Arrow dependency merely for testing |
| Command composition | Resolve the requested correctness scope, activate covering flags and refuse uncertainty | Preserve package/target selection and user features; no unrelated package root is appended |
| Recipes, affected selection and bare aliases | Expose the same scoped obligation through supported entry points | A command must enforce what its description claims; broad workspace selection remains explicit |
| Workspace unification/hakari | Share intended ordinary dependency features | Generated unifier edges do not certify semantic consumption or activate opt-in validation by default |

The split keeps dependency meaning with Cargo/manifests and invocation policy with the command
owner. A pure numerical or semantic unit can be tested locally without relation infrastructure.
Real Arrow consumers still require their actual data boundary. Additional explicit package
selections legitimately widen the operation; a Nextest execution filter does not by itself
remove the test targets Cargo selected for compilation.

## 3. Closure and assurance contracts

The ADR's "selected actual Arrow-consuming closure" means the resolved normal and build
dependency paths plus the dev-dependencies applicable to the selected test/check targets.
Resolution must reflect the invocation's package selections, target selectors, target platform,
default-feature choices and user features. It must neither walk every dependency's unrelated
dev graph nor omit a selected root's dev-only Arrow consumer. Unknown selection syntax or
unresolved coverage cannot silently fall back to a guessed pure closure.

Validation activation is an explicit property of the resulting consumed feature contexts,
not the presence of a conveniently named `force-validate` feature in a manifest. Propagation
must reach the validation-capable Arrow leaves in relevant target and host contexts. A feature
that exists but forwards nowhere cannot establish coverage. Command composition may use existing
declared propagation and resolved feature facts; it must not introduce another scientific or
dependency registry. An incomplete activation path refuses until its real owner is wired.

Pure means no semantic Arrow consumption in the actual closure, not a familiar package name,
a test's `unit` label or absence of a direct dependency. Generated workspace-hack edges may
cause incidental feature/build presence; they neither prove semantic consumption nor permit
ignoring an independent real path. Removing their authority from classification does not remove
them from Cargo's actual feature resolution.

All validation remains opt-in for correctness invocations. Neither defaults, the production
dependency declaration nor generated hakari features may turn it on routinely. The one-family
version rule, pinned nightly workspace unification and ordinary feature sharing remain intact.
Recipe and bare-tool support must be truthful: dynamic scope may be implemented by a shared
composer, or a narrower supported route must clearly enforce its documented scope. A static
alias that silently drops validation or reintroduces an unrelated root fails the target.

No physical-semantic table entries or numerical stages are introduced: this change composes
build/check/test invocations and does not formulate, evaluate or solve a scientific model.
Well-posedness and physical checks remain with their existing owners and are not replaced by
Arrow structural validation.

## 4. Representative scenarios

The parent journey is [S01](design_review_efficiency-principles-codebase_2026-10-09.md#s01).

| ID | Stimulus and kind | Expected boundary |
|---|---|---|
| <a id="s01"></a>S01 | Quantity/modeling semantic unit selection; invocation composition | Actual pure closure builds without adding pse-relations test targets |
| <a id="s02"></a>S02 | Runtime/relations/columnar Arrow check; direct-consumer binding | Opt-in owner flags activate the actual consumed Arrow validation without unrelated target selection |
| <a id="s03"></a>S03 | Schema/codegen transitive consumption or operations test dev-dependencies; dependency composition | Closure inspection reaches the actual Arrow path and requires covering activation |
| <a id="s04"></a>S04 | Add a build/dev/optional Arrow consumer; ordinary dependency extension | Manifest/resolution changes make coverage explicit; missing propagation refuses before claiming a validated correctness invocation |
| <a id="s05"></a>S05 | Extra packages, target/features/default options and exact Nextest filter; invocation evolution | Requested roots/features are preserved; closure and filter stay distinct; unsupported interpretation refuses |
| <a id="s06"></a>S06 | Alternate pure, composite, native and workspace commands; mechanism interaction | Intended ordinary feature sharing and exact family versions remain; validation does not leak into default production features |
| <a id="s07"></a>S07 | Bare alias or affected preview/run; command substitution | The advertised command provides the same obligation and prints/selects its true roots |

New scientific physics, numerical library substitution and solver behavior are outside this
decision. The applicable simulator journey is local admission/contract testing without unrelated
infrastructure. No claim follows about the elapsed time of that journey.

## 5. Existing mechanisms and proposed execution

`scripts/select.py::unit` currently appends `-p pse-relations`, then intersects Nextest's
execution filter with the intended packages. `check-package`, affected selection and bare
`cargo c`/`cargo t` aliases similarly add the relation root. The filter controls execution;
it does not undo the unrelated Cargo test compilation. Manifest inspection confirms that
quantity and modeling have no Arrow or relation dependency in their shown normal/dev paths.

Actual consumers are heterogeneous. Runtime directly depends on relations; columnar directly
depends on Arrow but currently has no validation propagation feature. Schema and codegen
reach columnar transitively. Operations has schema/codegen in dev-dependencies. Those concrete
examples justify closure-aware activation rather than a direct-dependency or package-name test.
An existing whole-workspace metadata view alone is not proof of the selected closure.

The coordinator reports that pinned `cargo check --locked -p pse-quantity --lib --features
pse-relations/force-validate` and the corresponding pse-modeling command were rejected, while
the existing direct-dependency feature flag was accepted for runtime in an actual Cargo tree
observation. These are coordinator-provided interface observations, not reviewer-executed
tests. They show why a universal spelling is insufficient and why a wholesale feature-resolution
change is unnecessary. They do not establish all proposed propagation paths.

The target reuses Cargo resolution and manifest feature forwarding, retaining one shared
command policy rather than separate heuristics in each recipe. The composer must include all
explicit requested packages and relevant dependency contexts, preserve user arguments, and
verify coverage or refuse. Adding an opt-in feature changes its owner and upstream propagation
seams; adding a pure package should not require a central exemption entry.

## 6. Foundations and gates

These judgments assess the proposed target; current command behavior remains pending migration.

| Foundation | Target judgment and evidence |
|---|---|
| AP-01 | Satisfied: manifests own consumption/propagation; command composition owns invocation policy; no test-name classifier |
| AP-02 | Satisfied: actual closure, explicit opt-in validation and uncertainty refusal define substitutable command behavior |
| AP-03 | Satisfied: scope-aware recipes/aliases compose declared features without creating unrelated roots |
| AP-04 | Satisfied: selected compilation roots, semantic consumption, generated feature sharing and validated correctness execution remain distinct concepts governing selection |
| AP-05 | Satisfied: normal/build/selected dev paths and feature contexts expose the consequential structure |
| AP-06 | Satisfied: legitimate pure tests lose unrelated data-boundary compilation; selector logic has an isolated command/graph test boundary |
| AP-07 | Satisfied: selected work follows actual dependency demand; feature sharing is preserved rather than discarded to obtain locality |

| Gate | Judgment and reason |
|---|---|
| G1 | Pass: manifest/resolved graph remains authority; no independently maintained test-consumption list |
| G2 | Pass: package/test selection and validation scope are explicit; pure does not mean direct-dependency-free only |
| G3 | Pass: uncovered Arrow consumption refuses instead of executing a falsely validated correctness route |
| G4 | Pass: requested target/features stay visible; validation is explicit and does not become a production default |
| G5 | Pass: command composition preserves exact selection and existing execution/resource outcomes; unknown composition refuses before effects |
| G6 | Pass: ordinary feature sharing and relevant test validation remain explicit across route changes |
| G7 | Pass: two Cargo refusals are interface evidence only; no runtime or performance acceptance is claimed |
| G8 | Pass: Cargo's feature resolver and declared forwarding remain the mechanisms; no replacement resolver is proposed |
| G9 | Pass: all seven foundations are satisfied for S01–S07 |
| PS-G1 | Not applicable: no physical quantities, property envelopes or balance formulation change |
| PS-G2 | Not applicable: no variable roles, structural analysis or solver admission change |
| PS-G3 | Not applicable: no numerical formulation, derivative, solver-status or independent assessment change |

DP-03/DP-09/DP-18/DP-22–DP-24 support explicit validation and truthful evidence. H1/H5/H18–H20
support assurance at actual consumption boundaries; H25–H28 support removing unrelated build
work without a new classification framework. Arrow validation is not scientific qualification.

## 7. Findings

No blocking target-design findings or SHOULD deviations were identified. F04 remains an
unresolved current-implementation finding at [Plan 33 EFF02](../../plans/33-efficiency-principles-remediation.md#eff02).
The coverage and route requirements above explain what would violate the accepted target;
they do not create another implementation ledger.

## 8. Library and tool fit

The pinned Cargo configuration already owns feature unification; hakari supplies the existing
fallback generated feature wiring. Manifest forwarding is a fitting existing mechanism for
opt-in activation. The target does not replace either with a custom solver. Pure and dev-only
closure classification needs command-level composition, but not a second graph authority.
The existing DataFusion/Arrow capability route was consulted; no new external library API is
proposed. Local pinned Arrow manifest inspection confirms that validation features are distinct
from ordinary dependency presence. This review does not rely on unverified current-branch docs.

## 9. Alternatives and tradeoffs

Keeping the extra relation target preserves the build amplification. Turning validation on
by default or through hakari removes the explicit test-only boundary. Adding relations or Arrow
to pure crates merely for a flag worsens ownership. Classifying tests by name or a central
allowlist drifts when dependencies change. Static aliases alone cannot adapt to arbitrary
selection unless their supported scope is deliberately narrower and accurately enforced.

The selected design is the simplest viable alternative: existing manifest features plus one
scope-aware command policy, complete resolved consumption and refusal when uncovered. It adds
ordinary propagation wiring and selected-graph inspection but removes unrelated target builds.
Correctness requires inspecting enough context to establish coverage; no exhaustive per-test
inventory or new evidence framework is justified. A future Cargo mechanism that activates the
required features without extra roots or propagation could simplify the implementation while
preserving this contract.

## 10. Verification and evidence limits

**Proposed:** the target's execution/locality and assurance argument. **Interface-checked:**
the two refusals and runtime direct-dependency acceptance supplied by the coordinator, with
the reviewer independently inspecting the relevant source/manifests. Reviewer Cargo commands,
product tests, build probes and measurements are **not_run**.

EFF02 must distinguish argument wiring from actual resolution. Controls should cover S01–S07,
including pure quantity/modeling/math/native closures; direct, transitive, build and dev-only
consumers; an uncovered consumer; additional requested packages; user features/default and target
options; exact Nextest inventory; and feature alternation. Inspect the selected Cargo units and
actual validation-capable Arrow features, including applicable host contexts. An empty feature
with the correct name or a mocked argument list is insufficient acceptance.

Check that no relation test root appears solely for activation and no validation feature leaks
through defaults or regenerated hakari output. Retain the existing one-family check and its
conditions. Use metadata/unit inspection without clearing the shared target directory. The named
native-local status control and canonical positive controls remain separate EFF02 obligations;
this review qualifies neither. No unit/property model changes, so scientific conformance and
external parity suites were not run or claimed.

## 11. Rule impacts and disposition

| ID | Current rule and owner | Required target amendment | If retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Blueprint §24.1 and AGENTS.md mandate relation force-validation for every Rust test invocation | Require explicit full validation of actual Arrow-consuming correctness closures, including selected dev/build paths, with unknown/uncovered consumption refusing | Pure local selection retains unrelated relation test roots |
| <a id="rc02"></a>RC02 | Cargo aliases, workspace validation metadata, shared Rust guidance and relation/workspace declaration comments describe the universal feature spelling | Make documented/enforced routes scope-aware and preserve explicit opt-in semantics; migrate relevant recipes/selectors/affected/governance consumers together | Routes disagree about validation or claim guarantees they do not activate |

These implement the efficiency review's confirmed conditional RC01 through ADR-0170 and the
decision/design route, with a blueprint revision row. Accepted records are not rewritten by
this review. No type-universe, pin, native-effect or feature-sharing rule is waived. Current
adoption, implementation and F04 dispositions remain solely in Plan 33.

## 12. Decision

**Accept ADR-0170's proposed scoped Arrow-validation target.** Behavioral/semantic adequacy and
architectural fitness are independently acceptable for the invocation/assurance boundary.
All applicable gates pass; no SHOULD exception is needed. The current source and supplied Cargo
observations justify changing the universal mechanism while preserving actual-consumer assurance.

Adopt the scoped rule and migrate the command surfaces before claiming implementation. A
composer that omits selected dev/build paths, trusts only named owner features, ignores user
selection, classifies via test names, silently permits unknown consumers or injects unrelated
roots would fail this accepted design. This decision establishes no product runtime pass,
build-speed measurement, feature-matrix completion or wider scientific qualification.
