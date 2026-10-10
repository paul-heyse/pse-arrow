---
title: Testing architecture implementation conformance
date: 2026-10-05
tier: change
purpose: conformance
standard: core-3.3
profile: process-simulator-1.4
baseline: 67069879de0da7d65fa6dc505a8ff2ceea594b4c
evidence: Implemented
decision: Revise
---

# Testing architecture implementation conformance

**Revise the reviewed implementation for two bounded execution-evidence defects.** The completed-result and integrity replacements substantially realize the selected Plan 26 design. Their responsibilities are clearer, their independent controls address different failure risks, and the replaced production/test mechanisms are removed. However, one database control bypasses its declared resource scheduling, and Python provenance can identify an extension different from the one pytest imports. Those gaps prevent acceptance of the implementation's actual-effect and actual-native-identity guarantees.

This is an independent Change-tier, conformance-purpose review under Core/template 3.3 and Process Simulator 1.4, selected by `standard.toml`. The selected profile overrides the older profile version mentioned in the reviewer role. The earlier [testing-responsibility review](design_review_testing-responsibility_2026-10-05.md) accepts the target at Proposed design strength; that judgment does not establish implementation conformance.

## Boundary and evidence

The reviewed baseline is HEAD `67069879de0da7d65fa6dc505a8ff2ceea594b4c` plus the user-selected uncommitted implementation when this review began. Scope is [Plan 26](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/26-testing-architecture.md), [26a](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26a-completed-results-and-transport.md), [26b](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26b-test-composition-and-independent-evidence.md), [26c](https://github.com/paul-heyse/pse-arrow/blob/ad665a0222551196b1160e426f5242361215a6a0/docs/plans/26c-test-execution-and-qualification-evidence.md), and [ADR-0160](../../adr/0160-centralize-testing-responsibility.md), including relevant source and adjacent consumers.

Static inspection covered trajectory construction/completion/transport, supervised Simulation and Shooting encoding, Python Arrow export, invariant declaration/production/binding/mechanism controls, requested fixtures and isolated databases, nextest resource configuration, native execution, terminal composition, input capture, receipt reuse, measurement prerequisites, retirement consumers, and the affected architecture/qualification owners.

Claims about these source paths are **Implemented** or **Interface-checked**. No test, build, benchmark, or probe was executed by this reviewer. Plan 26's Outcome records historical composite observations; they are not fresh tests from this review. Newly reported coordinator checks and repairs belong to the plan's current disposition and verification record.

Excluded are Plan 25's full science/campaign qualification, unrelated ADR-0161 governance, other platforms/distributions, broad parity and performance campaigns, and numerical algorithms unchanged by this series. No comprehensive rerun is requested.

[Plan 26](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/26-testing-architecture.md) is the sole current finding-disposition owner.

## Responsibility and conformance argument

### Completed scientific meaning and transport

`ModelingTrajectory` now wraps a private clone-shared `TrajectorySnapshot`. The snapshot retains the native outcome, preparation, checks/reports, completion, endpoint coverage, final computation header, candidate assessment and ownership. Public accessors borrow its contents; `accepted()` derives permission from retained completion. A cloned handle shares the snapshot rather than deep-copying completed scientific projections.

`ModelingSimulation::finish` composes endpoint/check evidence and constructs the final Simulation header through `workflow::completion::simulation_header`. Successful supervised Simulation consumes that retained header and assessment. Shooting's private projection consumes its joined report, retained endpoint assessment, composed completion, final optimizer header and candidate assessment; it does not turn a dynamics header into a Shooting header by relabeling its kind.

`ModelingTrajectory::tables` serializes first materialization with a mutex, reserves map ownership, calls the existing whole-collection encoder and publishes only a successful complete checked map. Failed direct materialization leaves the cache empty and releases temporary ownership. Direct retry does not recomplete science. The surrounding `RunResult` retains its existing sticky encoding-error contract.

Checked Arrow export retains original checked storage, container ownership and separately admitted wrapper metadata on escaped buffers. Python's `TableStream::from_batch` uses that operation. The inspected controls explicitly distinguish shared storage, budget refusal/retry, concurrent first reads, outer sticky failure, empty/bufferless representations and final escaped-array ownership.

These changes conform to blueprint §19.2 and §21 within the reviewed transport boundary. Whole-map first access remains a deliberate capability/cost tradeoff. No measured memory or speed improvement follows.

### Integrity production and independent evidence

The integrity producer attaches typed native `GeneratedIntegrity(IntegrityBinding)` origin to its own outputs. Public declaration starts authored, and `RegistryBuilder::declare_invariant` resets copied declarations to `AuthoredQuery`. Registry consumers receive immutable origin access. Binding refers to existing relation versions and declaration/occurrence identities rather than copying their column definitions.

Executable generated rules select the current highest relation version, matching unversioned table resolution. Repeated derivation rebuilds producer-owned outputs while preserving authored predicates and historical migration declarations. Origin remains outside durable metadata and fingerprint projections; the inspected test changes origin independently and compares materialized metadata and fingerprint.

Three distinct verification responsibilities remain:

- `pse-rules` mechanism witnesses execute actual generated SQL against independently specified hostile inputs and expected offending keys.
- Catalog controls inspect real binding inputs, projected key types, singular/batched interfaces and exact required invariant identities.
- Typed invariant execution and candidate-admission controls exercise actual enforcement independently of field certificates.

The witnesses address singleton/scalar/composite keys, nullable/composite unique tuples, correlated/self/empty references, nested occurrence visibility and encoding variants, ordinal boundaries and an independently authored predicate with generic spelling/kind. Their expected values are not obtained by executing production evaluation.

This is a materially stronger arrangement than a valid/violating file pair per generated instance. Static inspection establishes the implementation and discriminating expectations, not fresh runtime passage.

### Effects, execution and qualification

Python setup is requested explicitly. Immutable deployment settings remain shared, while runtime facades and database effects are individually owned. `TestDatabase::create_with` cleans its acquired database when initialization fails. Owned disposable-input controls replace per-test Git attribution.

`validation_scope.comprehensive` composes one linked Rust graph, focused default-feature absence controls and one Python selection. Native enumeration belongs to the native wrapper; nested Python execution returns its process result to the enclosing terminal owner. One JUnit parser reconciles selection and terminal identities, retaining malformed/missing/interrupted evidence as failures or not-run results.

Version-5 input scopes project a conservative captured inventory. Unknown scopes retain all captured inputs. Reuse requires canonical invocation, scoped inputs/environment, original-report/artifact identity and applicable native bytes; reviewed transfer remains explicit and preserves original observation/rationale. Measurements consume exact named functional invocations or the explicitly declared linked covering invocation. Benchmark identities remain distinct from functional binaries, and smoke remains untimed.

The remaining defects are in binding these declared guarantees to actual execution: resource classification misses one owned database control, and Python binary capture uses filesystem candidates instead of actual import resolution.

### Replacement and authority

The reviewed production/test/tooling search found no live consumers of the retired `tables_for_kind`, package-surgery `reference_documents`, invariant fixture generator/discovery, `codegen_regeneration`, `phase0_placeholder`, upstream-only `math_composition`, or empty `pse-tests-structural` mechanisms. Searches covered crates, tests, scripts, xtask, Python, manifests and recipes, excluding derived generated contents where appropriate. This is scoped source evidence, not a claim about historical Git objects or excluded external corpora.

Complete emitted-tree comparison remains with xtask. Production structural analysis remains in `pse-structural`; retirement concerns the empty test shell. Scientific comparisons remain distinct from retired IDAES spelling compatibility.

ADR-0160 and its architecture amendments provide the selected governance route. ADR-0160 remains proposed; maintainer decision acceptance is separate from implementation review and product qualification. No accepted ADR is silently reinterpreted as newly tested.

## Revealing scenarios

| ID | Stimulus | Expected boundary | Reviewed result |
|---|---|---|---|
| S01 | Export an accepted/refused/partial trajectory through direct, supervised and Python paths | One completed permission/header; representation owners consume it | Implemented in the inspected construction and projection paths |
| S02 | Concurrent clone export under pressure, then release parents | One successful materialization; retry without recomputation; escaped ownership survives | Implemented; discriminating controls inspected |
| S03 | Add an existing-family relation or change current schema version | Producer derives current bindings; shared witnesses and catalog binding remain distinct | Implemented; version/origin controls inspected |
| S04 | Add a database support control | Actual-effect namespace inherits the store schedule | F01: reviewed baseline bypasses it |
| S05 | Execute Python with another compatible installation selected by import resolution | Evidence identifies the native module actually consumed, or rejects the selection | F02: reviewed baseline hashes checkout candidates |
| S06 | Reuse a claim after prose changes or measure a selected workload | Relevant inputs and exact prerequisite identity govern applicability | Implemented, subject to F02's native-attribution gap |

No new physical formulation is introduced. Transport retains authored quantity/unit identities and original coordinate conversions. Simulation endpoint/closure evidence and Shooting original-space checks govern permission; native termination alone does not authorize use. Degree-of-freedom/structural admission is unchanged and is not requalified here. No unit/property model gains a new numerical-fidelity claim from this review.

## Findings

### <a id="f01"></a>F01 — An owned database control bypasses the store resource schedule

**Principles/gates:** AP-05, DP-19; scenario S04.  
**Priority:** Medium.

**Implemented/source evidence:** In the reviewed baseline, `crates/pse-operations/src/testing.rs` places `failed_isolated_database_setup_removes_owned_database` under `testing::tests`. That control creates a database, initializes an admin connection, injects failure, removes the database and queries the server inventory. `.config/nextest.toml` assigns the store group through `store_tests`, `study_tests`, `tables_tests`, `migration_tests`, and `retirement::tests`; `testing::tests` does not match.

**Trigger and consequence:** Selecting the control with other PostgreSQL tests runs it outside the declared store group. The configured bound therefore does not cover all actual database effects. Additional controls in the same support namespace would inherit that bypass. This is a classification/enforcement defect, not evidence of an observed server-capacity failure.

**Correction:** Put the actual-effect controls in the existing store namespace, or otherwise make the existing effect selector cover them. Preserve pure support tests outside store scheduling. No second resource registry is needed.

**Closure evidence:** Static correspondence between the emitted test namespace and existing nextest selector, plus one focused linked-native execution of the setup-failure control. The coordinator's `tests` → `store_tests` rename was inspected during review and is a sound narrow remedy. Runtime confirmation and current disposition remain with Plan 26.

### <a id="f02"></a>F02 — Python provenance can identify a binary pytest never imports

**Principles/gates:** AP-02, AP-04, AP-05, DP-09, DP-22; G2/G6/G7; scenario S05.  
**Priority:** High for qualification identity.

**Implemented/source evidence:** The Python branch of `scripts/native_tests.py` passes every filesystem match from `ROOT/python/pse/_native*.so` to `native_provenance`, then launches `sys.executable -m pytest`. It does not resolve or verify the imported module's origin. The native recipe/environment preserves Python import controls. Pytest uses importlib mode without a checkout-path constraint. `python/pse/_build.py` imports the `_native` selected by the imported `pse` package; its compatibility check compares package version and registry fingerprint, which can agree between different source implementations.

**Trigger and consequence:** A compatible other checkout/installation selected through import resolution can execute while the report hashes this checkout's `.so` and source-input scope. The captured paths then describe candidate artifacts rather than the binary actually consumed. Changing the imported external extension while leaving the recorded local file unchanged can also evade native-byte revalidation for retained Python evidence. Correct terminal test results do not repair that attribution.

**Correction:** Resolve and verify the extension consumed by the exact interpreter, working directory and import environment used for pytest. For this checkout-scoped qualification contract, reject an origin outside the intended checkout; recording an external extension would also require truthful source-input ownership. Use the existing Python import boundary or a narrowly owned execution preflight rather than another product provenance framework. Hash the verified consumed extension and its applicable native closure.

**Closure evidence:** An isolated hostile-import control must select a different compatible origin and demonstrate refusal before a successful qualification claim. A matching-checkout control must demonstrate that provenance names the resolved imported extension. Preserve mandatory provenance, one terminal composer and nonzero-exit handling. A focused native Python journey establishes integration after the correction; no comprehensive rerun is required.

## Architectural foundations and gates

| Foundation | Judgment on reviewed baseline | Evidence/reason |
|---|---|---|
| AP-01 Separation of concerns | Satisfied | Scientific completion, transport, integrity production, independent evidence and terminal composition have identifiable owners |
| AP-02 Stable contracts | Violated | F02 does not fulfill the consumed actual-native-identity contract |
| AP-03 Composition | Satisfied | Existing loaders, completion, collection encoding and framework selection are reused |
| AP-04 Domain model and semantic authority | Violated in evidence binding | Completion and integrity models govern behavior; F02 fails to bind recorded execution identity to the consumed module |
| AP-05 Explicit structure and constraints | Violated | F01 bypasses resource scheduling; F02 leaves native identity unenforced |
| AP-06 Local reasoning/testability | Satisfied | Independent witnesses and disposable effects permit bounded tests without a new universal fixture/impact framework |

| Gate | Judgment | Boundary |
|---|---|---|
| G1 Authority | Pass | One completion/header, producer declaration and freshness owner in inspected scope |
| G2 Semantic fidelity | Fail | F02 can substitute candidate-file identity for consumed-native identity |
| G3 Validity | Pass for inspected production admission | Field certificates, relational enforcement and completed permission remain distinct; F01's scheduling gap is assessed under AP-05 |
| G4 Hidden behavior | Pass | Effects are requested and transport does not independently recomplete science |
| G5 Consistency/recovery | Pass for inspected completion/transport | Success-only cache, sticky outer failure and owned escaped storage remain explicit |
| G6 Transformation/reuse | Fail | F02 can leave an actually consumed external binary outside recorded native-byte revalidation |
| G7 Truthful capability claims | Fail | Actual Python binary provenance is not established by checkout globbing |
| G8 Library leverage | Pass | Framework selection, existing encoders, DataFusion query execution and xtask comparison remain owners |
| G9 Architectural fitness | Fail | AP-02/AP-04/AP-05 gaps cannot be averaged against strengths |
| PS-G1 Physical consistency | Pass for preserved transport meaning | Quantity/unit/coordinate projections remain under existing owners; no new model-fidelity claim |
| PS-G2 Well-posedness | Not applicable to changed algorithms | No structural-analysis or formulation algorithm changes in this boundary |
| PS-G3 Numerical integrity | Pass at inspected implementation level | Original checks and retained composed permission govern Simulation/Shooting projections; unexecuted broader numerical scope remains excluded |

## Alternatives, evidence limits and decision

The simplest viable correction keeps the current decomposition: effect namespaces use existing nextest scheduling, and the Python execution owner verifies import origin before publishing provenance. A new fixture language, exact-name test manifest, source-impact graph, per-relation trajectory dispatcher or provenance subsystem would add machinery without resolving a separate requirement.

Whole-map trajectory encoding remains justified by the existing collection contract and success-only snapshot publication. Reopen selective export only at ADR-0160's stated capability or measurement trigger. Independent mechanism witnesses plus catalog/admission controls preserve distinct oracles; deriving expected answers from the generator would weaken that arrangement.

Behavioral/semantic adequacy: **Revise** the bounded scheduling/provenance guarantees. No inspected defect was found in completed scientific permission, current-version integrity derivation, durable fingerprint preservation or replacement deletion.

Architectural fitness: **Revise** because the actual execution bindings do not yet honor their explicit resource and provenance contracts.

Overall decision: **Revise** the reviewed implementation. Plan 26 owns F01/F02 corrections, evidence and current status. Acceptance can follow a focused review of the remedies and their named controls. Historical Q1 observations retain their original scope and composite character; this review neither demands their wholesale repetition nor qualifies Plan 25's paused campaign.

## <a id="remedy-acceptance"></a>Bounded follow-up: F01/F02 remedies, 2026-10-05

**Accept the corrected implementation within this review's declared boundary.** This independent follow-up assesses the two remedies against the original findings. The original reviewed-baseline **Revise** judgment remains intact above; [Plan 26](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/26-testing-architecture.md) owns current dispositions and closure evidence.

The follow-up baseline is HEAD `67069879de0da7d65fa6dc505a8ff2ceea594b4c` plus the selected uncommitted implementation and the inspected F01/F02 corrections. The standard remains Core/template 3.3 and Process Simulator 1.4. No reviewer execution, edits or additional delegation occurred.

### F01: database scheduling

**Implemented/source-inspected:** The setup-failure control now belongs to `testing::store_tests` in `crates/pse-operations/src/testing.rs`. That namespace matches the existing `test(store_tests::)` selector for `pse-operations` in `.config/nextest.toml`, assigning the control to the declared store group.

This corrects the actual-effect classification at its existing namespace boundary. It adds no special-case selector, second resource inventory or production behavior.

**Tested, coordinator execution; terminal artifact inspected:** The focused command was:

```bash
just native-test --profile ci -p pse-operations \
  -E 'test(testing::store_tests::failed_isolated_database_setup_removes_owned_database)'
```

The zero-failure-baseline result is 1/1 passed. `target/nextest/ci/junit.xml` contains the renamed control with no failure/error element. Static correspondence between that identity and the existing selector establishes the scheduling correction; the executed control establishes retained setup-failure cleanup behavior.

### F02: consumed Python extension identity

**Implemented/source-inspected:** `scripts/native_tests.py::python_native_binary` resolves the imported `pse` package and loaded `pse._native` module in a subprocess using the same interpreter, environment and repository working directory as pytest. It requires the package and resolved native binary to belong to this checkout's `python/pse`, then passes only that resolved binary to native provenance. The filesystem glob is removed.

The wrapper passes the verified binary identity to pytest. `python/pse/tests/conftest.py::pytest_sessionstart` compares the already-loaded native module against that identity before collection, rejecting preflight/execution import divergence. The check follows the actual import boundary; native hashing and receipt composition remain wrapper-owned. No production API, numerical operation or test-selection policy is added.

The inspected `NativePythonImportIdentity` controls discriminate:

- A foreign package selected by real subprocess import resolution is refused before provenance or test dispatch.
- Matching import resolution selects one consumed binary and excludes a stale filesystem candidate.
- An actual pytest subprocess with mismatched loaded origin fails before its sentinel test executes.

Existing terminal-owner controls remain distinct and retain nonzero-exit, freshness and single-composition behavior.

**Tested, coordinator-reported:** `just unit-consolidation-tools` initially exposed one fixture-mock failure, which was repaired; the final invocation passed 52/52 against a zero failure baseline. This is composite evidence, not an initially clean result. The reviewer inspected the relevant control implementations but did not independently execute them.

**Tested, coordinator execution; retained artifacts inspected:** The affected linked Python selection ran through `just native-python build/plan26-q1-python-import` with the original 33-test filter and existing registry-unchanged inspection publication. The terminal artifact records exit code 0, status `passed`, 33 selected identities, 33 passed terminal results and no report errors. `native.json` names this checkout's `_native.abi3.so` as its single executable identity and retains 31 binary/library file identities.

This establishes integration of the repaired wrapper with the selected completion, trajectory, escaped-array, cancellation, publication, IPC and isolated-database journeys. It does not establish unselected Python behavior or a separate xdist execution campaign.

### Follow-up judgment and limits

F01's scheduling binding and F02's consumed-native-identity binding are corrected. AP-02, AP-04 and AP-05 are **satisfied** within the reviewed boundary; the earlier satisfied AP-01/AP-03/AP-06 judgments remain. G2, G6 and G7 now **pass** for the corrected evidence path, and G9 **passes** without averaging away the original defects. The remaining gate judgments retain their stated scope and evidence limits.

Behavioral/semantic adequacy: **Accept** the bounded implementation, supported by inspected mechanisms, focused coordinator execution and the retained terminal/provenance artifacts described above.

Architectural fitness: **Accept**. Both corrections fit the existing responsibility model and avoid additional selectors, provenance frameworks or product interfaces.

Overall: **Accept** the corrected Plan 26 implementation within the principal review's boundary. No material remaining issue was found in the remedies. ADR-0160 decision acceptance remains separate, historical Q1 observations retain their original composite scope, and Plan 25's broader campaign remains excluded. No performance benefit or comprehensive product qualification is claimed.
