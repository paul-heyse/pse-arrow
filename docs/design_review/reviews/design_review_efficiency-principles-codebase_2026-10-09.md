---
title: Efficiency principles across the simulator codebase
date: 2026-10-09
tier: design
purpose: target
status: review
standard: Core 3.4
profile: Process Simulator 1.5
baseline: 4c24721e691187e1a5b28398b29722fbde671da8
---

# Efficiency principles across the simulator codebase

## 1. Principal assessment, scope and decision

**Decision: Revise.** The current architecture has a credible semantic and numerical foundation, but its physical realization still contains material work amplification across local development, selected preparation, sparse block initialization, fitting, result export and retirement. Qualification applicability also omits consumed inputs, which is a correctness defect in evidence reuse rather than merely an efficiency opportunity.

The best corrective direction is to retain the separation among authored physics, immutable mathematical preparation, private native execution and coherent scientific publication, then remove the unnecessary work inside those boundaries. The findings do not support moving numerical execution into the persistence engine, replacing established solvers, or introducing a new general cache, scheduler, dependency engine or accounting framework.

The target is the best-in-class simulator described by the Process Simulator profile and blueprint §0/§0.2: reusable physically typed models serving interactive edit/re-solve, studies, recycles, square simulation, optimization, dynamics, estimation and extension. Current mechanism choices, accepted decisions and policies are subjects of this target review. They are not acceptance criteria merely because they already exist.

The assessment balances two independent user experiences:

- A developer should compile and exercise a changed responsibility with its actual dependencies.
- An engineer should prepare, execute and inspect legitimate models without incidental work growing faster than the scientific operation requires.

These experiences share preparation, dependency and lifecycle concerns, but they are different latency domains. Feature enumeration is not solve latency; key hashing is not complete preparation; a small query result does not establish small examined work.

The maintainer's adoption criterion favors correct approaches expected to improve performance even when present timing is marginal, inconclusive or slightly adverse; incorrect behavior or substantial performance regression is grounds for non-inclusion. Concurrent repository workloads can add measurement noise. The findings therefore do not require a large measured gain before correction, while quantitative benefit claims still require appropriate measurements.

| Field | Assessment |
|---|---|
| Boundary | Repository-wide architectural review of consequential build/test, authoring/preparation, numerical execution, publication, serving and retention paths, including Rust/Python and canonical-store neighbors |
| Tier and purpose | DESIGN / TARGET |
| Standard | Core/template 3.4, Efficiency Heuristics 1.0, Process Simulator principles/review additions 1.5, selected pse-arrow binding |
| Reviewer | Fresh principal reviewer, independently examining decisive contracts and source and reconciling three bounded supporting assessments |
| Baseline | `4c24721e691187e1a5b28398b29722fbde671da8`; concurrent graph/hash implementation was committed during inquiry startup and is included |
| Architectural fitness | Fails G9; concrete violations of applicable foundations |
| Behavioral/semantic adequacy | Revise for qualification-applicability defects; whole-simulator scientific acceptance is not established |
| Disposition | Existing [Plan 28](../../plans/28-surrealdb-unified-substrate.md) remains the coordination/disposition owner if these findings are adopted; suggested work owners below are prospective |
| Exclusions | No remedy implementation, plan scheduling, full qualification, other-platform/distribution assessment or new performance campaign |

The workloads considered include increasing expression/declaration populations, many small sparse blocks, many study cases, observations and fitting parameters, longer trajectories and sensitivity output, concurrent native work, interruption, and repeated development/analysis history. Selected sixteen-case study journeys and finite job-population controls provide concrete concurrency scenarios, not proof of throughput. No new capacity SLA, universal model-size promise or numerical cost model is invented.

The principal argument and identifiers below own the combined judgment. Supporting documents provide bounded evidence, not competing decisions or disposition ledgers:

- [Build and validation](../evidence/efficiency-principles-codebase-2026-10-09/build-and-validation.md)
- [Preparation and mathematics](../evidence/efficiency-principles-codebase-2026-10-09/preparation-and-mathematics.md)
- [Execution and results](../evidence/efficiency-principles-codebase-2026-10-09/execution-and-results.md)
- [Coordinator evidence and probe conditions](../evidence/efficiency-principles-codebase-2026-10-09/README.md)

## 2. What the architecture gets right

The system's important responsibilities are recognizable in executing code, rather than only in output vocabulary.

| Responsibility | Meaning and decisions owned | Consumed contract and effects | Relevant local reasoning boundary |
|---|---|---|---|
| Authoring and physical checking | Lexical meaning, declared quantities, operation prerequisites and original domains | Immutable checked definitions; typed refusal | `pse-authoring`, `pse-modeling`, `pse-quantity`; no native solve required to test an occurrence or physical inference |
| Selected compilation | Complete selected dependencies, specialization, structure and demanded mathematical products | Pure preparation through Salsa; no publication authority in tracked mathematics | `pse-compiler`, `pse-math`; structural/value inputs remain distinct |
| Numerical policy and structure | Physical tolerances, normalization, analysis intent, incidence, matching and conditional boundaries | Admitted class/capability and explicit numerical context | `pse-structural`, `pse-math::numerics`, native routing |
| Native execution | Attempt-private workspaces, callbacks, solver lifecycle and actual resource ownership | Library-owned numerical iteration; typed outcomes; cancellation and drain | `pse-backend-native`, runtime jobs/staged sessions |
| Workflow composition | Current case attribution, selected protection, attempts, studies, continuation and scientific completion | Explicit current effects around retained immutable preparation | `pse-runtime` |
| Publication and inspection | Exact manifest membership, bounded transport, protection and retirement | Canonical operations and checked Arrow boundaries | `pse-operations`, runtime readers/exporters, Python streams |
| Development and qualification | Selection, native installation, placement, terminal evidence and applicability | Authentic artifact association and retained observations | Existing scripts and recipes; defects occur when these capabilities compose too broadly |

Several historical diagnoses no longer describe this baseline:

- `ModelingPackage::prepare_selected` checks a retained complete basis before creating a compiler workspace. `BasisKey` compares complete request/dependency/root/instance/binding/limit fields; its process-local prehash is a bucket hint, not equality authority. `PreparedCase::rebind` retains plans and structure while refreshing affected value products. See [preparation](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/modeling/preparation.rs), [basis identity](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/preparation.rs) and [rebinding](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling/executable/solve.rs).
- J3 indexed descriptions, B8/N13 retained supplier topology, N12 policy lookup and the current hashing changes are implemented. They are preservation constraints, not fresh unresolved findings.
- Jobs acquire entry admission before spawning. Staged native teams retain actual stack ownership; cancellation keeps charges through native join/destruction. Studies use bounded concurrent lanes, while continuation preserves its genuine predecessor dependency. See [jobs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/jobs.rs), [staged sessions](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/math/staged.rs) and [study execution](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/study_execution.rs).
- Native installation separates construction, immutable generation publication and operation admission. Build information no longer embeds the whole outer source tree. Generation skips byte-identical outputs while comparison remains bidirectional. See [native cache](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_cache.py), [build information](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-buildinfo/build.rs) and [generation](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/xtask/src/codegen.rs).
- Registry construction is retained through `OnceLock`; resolved contracts, schemas and contexts share ownership. Same-owner columnar readmission reuses established admission. There is no supported diagnosis of global registry/schema reconstruction on every lookup.
- Native termination, candidate availability, original numerical assessment, physical closure and final usability remain distinct. `attach_nlp` independently evaluates original constraints and records validation failure rather than promoting a native return code into assurance. See [quality](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/quality.rs).
- Connected readers protect exact manifests, validate selected membership and retain allocation ownership for escaped arrays. Source/run reclamation already has explicit ownership and bounded cleanup.

These are substantive strengths. A correction that removes them in pursuit of fewer checks, fewer copies or more concurrency would not satisfy this review.

## 3. Physical meaning, authority and well-posedness

The inspected model captures more than dimensions. Quantities distinguish origin-sensitive points and differences, basis, datum, subject, indices and allowed operations. Formulation preserves original domain obligations before symbolic simplification. Definitions, case values, policies, starts, attempts and results have separate lifecycles under blueprint §5, §8–§10 and §14–§20.

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority and operation |
|---|---|---|---|---|---|
| Variable, parameter and physical output | Registered complete quantity and canonical representation | Declared mass/molar/other basis where applicable | Datum, point/difference and subject retained | Declared bounds and mathematical obligations | Quantity/model declarations; checked construction, specialization and value binding |
| Temperature and pressure coordinates | Admitted units and explicit conversion | Physical coordinate meaning retained | Temperature origins and gauge/absolute pressure are distinct | Finite bounds and declared model applicability | Blueprint §8; quantity-owned conversion and operation inference |
| Enthalpy, entropy and energy contributions | Physical units and signed contributions | Declared material basis | Explicit datum and heat/work convention | Package/model validity and closure conditions | Authored definitions and generic balance contributions, blueprint §9–§10 |
| Property function or implicit closure | Typed input/output contracts | Component, phase, material/state bindings | Declared method/branch and parameter provenance | Hard domains, applicability and supported derivative order | Checked physical/function contracts and current provider admission |
| Error allowance and normalization | Positive magnitude in admitted error/difference units | Inherits target meaning | Affine origin shifts must not become error magnitudes | Explicit numerical-policy prerequisites | `pse-math::numerics`; F08 identifies a missing prerequisite-context input |
| Dynamic state, rate, sample and endpoint | Physical coordinate/time contracts | Declared state and rate roles | Time origin, actual endpoint and event convention | Supported ODE/index-1 profile, admitted events/schedules | Dynamic formulation, integration profile and original endpoint/closure assessment |
| Result scalar or relation | Physical metadata against original semantic identities | Preserved from the completed computation | Original interpretation and outcome retained | Complete or explicitly partial scope | Scientific completion, checked transport and exact manifest admission |

This table reconstructs inspected contracts; it is not a claim that every scientific package and provider chain was independently validated in this review.

**Well-posedness statement.** Fixed specifications, free unknowns, guesses, decisions and observations have explicit roles. Complete selected structure reaches conservative incidence and class-aware structural admission before native solving. Matching, DM and BTF are distinct from topology and numerical rank. Root intent requires original square equality coverage; NLP admission allows genuine optimization degrees of freedom while rejecting offending equality structure. Partial structural scope is refused. Diagnostics retain semantic equation/variable identities. Conditional blocks preserve original rows, solved columns and predecessor inputs.

The decisive source is [native structural admission](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/structural.rs), with compiler/structural owners described in blueprint §15 and §17. Structural matching does not establish numerical regularity. A singular numerical point remains a numerical outcome. F07 challenges the physical representation of valid blocks, not the necessity of structural analysis.

## 4. Representative journeys and change locality

| Scenario | Stimulus and kind | Expected boundary | Assessment |
|---|---|---|---|
| S01 — Local developer iteration | Change a physical rule, compiler operation or native status adapter; implementation/policy change | Build/test actual consumer closure; native linkage and canonical effects selected independently | F01/F04 violate locality; F02 affects the applicability of resulting evidence |
| S02 — Edit and re-solve | Change values, then structure or lexical membership; binding/domain change | Values retain compatible structure; structural/absent-name changes invalidate affected meaning | Retained basis/rebinding are strengths; F05/F06 amplify preparation that still runs |
| S03 — Sparse initialization and recycle | Increase small blocks, introduce predecessor chains or non-convergence; instance/composition change | Compact original-ID block views, explicit strategy, finite attempts and rollback | F07 amplifies coordinates and edge searches; existing transactional stages and typed outcomes must survive |
| S04 — Many cases and fitting observations | Increase cases, observations and shared parameters; workflow composition | Shared immutable preparation, bounded private workspaces, direct sparse contribution refill | Concurrent lanes are present; F10 repeatedly rediscovers prepared relationships |
| S05 — Dynamics and useful result demand | Longer trajectory/sensitivities; request a header, names or canonical publication | Independent numerical qualification, then requested/bounded transport | F09 materializes unrelated complete Arrow output first |
| S06 — Extension and substitution | Add a property/unit using existing concepts, or replace a solver/store mechanism | Semantic addition and specialized behavior stay with their owners; capability differences explicit | Existing library/semantic separation is useful; F08 exposes missing context at a consumer boundary |
| S07 — History, interruption and retirement | Repeat development and derived analyses; cancel or retire | Live coordination follows actual participants; payload cleanup preserves immutable lineage and fences | F03/F11 retain history in inappropriate physical lifetimes |

Out-of-envelope properties, ill-posed models, failed initialization, event termination and interrupted publication were considered through their contract owners. No proposed efficiency correction may silently extrapolate, treat a guess as a specification, publish a nonqualified candidate as a solution, or hide missing trajectory observations.

## 5. Findings and corrective contracts

The diagnoses below are **Implemented** source observations unless a stronger or narrower evidence label is stated. All remedies are **Proposed**. Structural work counts explain the mechanism; they are not latency, RSS or speedup measurements.

| Finding | Cause | Main scenario |
|---|---|---|
| [F01](#f01) | Native-local selected tests unconditionally acquire worker/store execution | S01 |
| [F02](#f02) | Qualification applicability omits consumed source/environment inputs | S01/S07 |
| [F03](#f03) | Active coordination processes completed resource history | S07 |
| [F04](#f04) | Narrow semantic checks/tests select unrelated relation targets | S01 |
| [F05](#f05) | Occurrence preparation renders and hashes subtree text repeatedly | S02 |
| [F06](#f06) | Selected closure repeatedly scans and copies its checked universe | S02 |
| [F07](#f07) | Each conditional block materializes original coordinate metadata | S03 |
| [F08](#f08) | Numerical difference inference lacks required physical facts | S02/S06 |
| [F09](#f09) | Selective/bounded result transport first builds the complete Arrow map | S05 |
| [F10](#f10) | Fitting repeatedly searches prepared observation/response relationships | S04 |
| [F11](#f11) | Retired analyses retain inaccessible derived graph payloads | S07 |

### <a id="f01"></a>F01 — Native-local tests acquire an unrelated worker/store journey

`test_run.run_rust` routes a native-operation invocation without retained-binary metadata or its child sentinel into `observer_rust`. Capture unconditionally builds the full worker; execution then enters the canonical observer. Neither decision consumes an explicit local-versus-canonical effect requirement. See [runner](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/test_run.py) and [capture](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_tests.py).

The revealing legitimate selection is the Ipopt status/infinity classification test:

```text
just unit-native-capability-package pse-backend-native ipopt solver status_scope_and_finite_infinity_are_not_conflated
```

The [test](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/ipopt.rs) needs no runtime, database or worker. Its route nevertheless builds unrelated native features and requires canonical observer configuration. A fresh narrow native environment can also lack root-isolation inputs demanded by the broad worker build; that failure path is source-supported, not executed here.

This violates AP-01/AP-03/AP-06/AP-07 and G9. Native linkage, canonical execution and terminal reconciliation are independent responsibilities.

**Correction.** Make execution effect scope an explicit input to the existing composition root. A native-local selection retains authentic setup, admission, exact selection, terminal reconciliation and actual drain; canonical selections retain worker association and observer/service admission. Unknown scope may remain conservative. No source classifier or exact-test-name registry is required.

A `unit` label alone is insufficient: a unit test may legitimately consume canonical resources. A local-only invocation attempting canonical effects must refuse clearly or request the appropriate role.

**Verification.** Run the inspected status test with only its required native capability and no configured canonical service; establish absence of the full-worker/observer route. Retain positive canonical/managed controls for placement, receiver association, terminal evidence and descendant drain.

### <a id="f02"></a>F02 — Qualification applicability omits consumed inputs

`validation_scope.RUST_INPUTS`, inherited by Python product scope, omits `test_run.py`, `test_resources.py`, `host_admission.py`, `surreal_server.py` and `benches/`. Supported native/tool selectors including `UNO_DIR`, `PETSC_DIR`, `SUITESPARSE_INCLUDE_DIR`, `CC`, `CXX` and `CMAKE_TOOLCHAIN_FILE` are absent from its environment projection.

The omitted scripts govern execution, fixture identity, placement, service admission or cleanup. Native configuration consumes the omitted selectors. `_reuse_guarded` compares the projected `input_identity`; the complete outer snapshot cannot repair that comparison. `verify_native` rehashes paths from the old capture, not a newly selected configuration's closure. See [scope](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation_scope.py), [reuse](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation_receipts.py), [native configuration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_operation.py) and [native cache inputs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_cache.py).

**Tested projection behavior:** the retained pinned Python 3.14.7 probe changed synthetic file digests/environment values passed to the actual function. Both product scopes ignored the five tested omitted paths and six selectors; crate source, `native_tests.py` and `IPOPT_DIR` positive controls changed identity. This tests projection only. Actual successful-receipt reuse was not exercised.

Consequently, a prior successful observation can satisfy the unchanged-input comparison after relevant orchestration/configuration changes. This violates AP-04/AP-05, DP-09/DP-22 and G3/G6/G7. It does not prove that a scientific result was numerically wrong.

**Correction.** Restore conservative consumed-input coverage, include effective supported native/tool configuration and bump the scope interpretation version. Including execution scripts broadly is a viable first correction; a narrower complete closure is also valid. Preserve historical receipts under their original interpretation. Do not relabel them under the expanded definition.

**Verification.** Retain positive controls, detect each omitted consumed input, and demonstrate refusal of unchanged-input reuse on an actual successful report after a relevant change. Reviewed transfer remains explicitly historical. Old captured-library hashes cannot substitute for current configuration association.

### <a id="f03"></a>F03 — Active coordination processes completed resource history

`test_resources.resource_metadata` uses a shared exclusive metadata operation that parses and rewrites the complete ledger, including status reads. Its callers repeatedly validate and scan owner history. Cleanup marks entries removed/compacted but leaves them in the hot ledger. Native generation retirement similarly scans retained `.operations/*.json`; that native scan occurs during retirement consideration, not every current-generation lookup. See [resource ownership](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/test_resources.py), [metadata](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/host_admission.py) and [native operation records](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_operation.py).

The supporting filesystem snapshot found 226 resource records, 93 already removed/compacted, and 830 native operation JSON files. These are diagnostic counts, not a latency attribution.

As completed history grows while active participants stay constant, ordinary coordination and retired-generation discovery inspect historical payloads unrelated to current ownership. This is AP-07/G9 work/lifecycle amplification under H9/H23/H26.

**Correction.** Separate the active ownership set from retained final receipts. Only after authentic terminal disposition and actual drain may completed ownership leave hot coordination. Keep minimal immutable identity/outcome/reclamation records available by exact lookup or cold archival storage. A new database is not required.

Preserve failed/incomplete/unknown pins, pending handoffs, protected references, no-recreation semantics and PID/unit-generation protection. Parent exit or age is not drain proof. Missing archived metadata must not become evidence of safe reclamation.

**Verification.** Hold active participants constant while adding completed history; active metadata work should remain independent of archived payload population. Exercise exact historical lookup, repeated cleanup, reference races, unknown owners and surviving descendants.

### <a id="f04"></a>F04 — Narrow semantic checks/tests build unrelated relation targets

`select.unit` always selects `pse-relations --lib`, then excludes its tests from execution unless requested. `check-package` also selects `pse-relations --all-targets`; narrow native checks use related widening. Thus an Arrow-free compiler/quantity responsibility can acquire a relation test/check target with DataFusion dev-dependencies. See [selection](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/select.py), [recipes](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/justfile) and [relation dependencies](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-relations/Cargo.toml).

The corrected semantic normal dependency graph does not remove this selected test-build root. This violates AP-06/AP-07 and G9. No cold-build penalty was measured.

**Correction.** Build the selected responsibility's actual dependency closure. Preserve force-validation of every consumed Arrow boundary. First determine whether the pinned Cargo configuration can activate that feature without selecting unrelated relation targets. If it cannot, change the every-invocation mandate into an assurance requirement on actual Arrow-consuming closures.

An allegedly pure test whose dev-dependencies consume Arrow is not exempt. The correction follows actual consumers, not labels.

**Verification.** A cold pure semantic selection should produce no unrelated relation test artifact; actual Arrow-consuming selections must demonstrate enabled validation. Alternating pure/composite/native selections must preserve intended feature sharing and the single type universe. The exact Cargo mechanism remains unresolved; no alternative invocation was tested here.

F01 and F04 require distinct corrections: one widens runtime effects/native features; the other widens selected build targets.

### <a id="f05"></a>F05 — Occurrence preparation repeatedly renders and hashes subtree text

`ExpressionOccurrence::of` renders an expression twice. `in_body` invokes it for every node, then overwrites each subtree hash with the root hash. Checking and binding consume these maps; typed function lowering constructs further occurrence products. See [admission](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/expression/admission.rs), [binding](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/expression/occurrences.rs) and [typed lowering](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/typed_math.rs).

Nested bodies retain text proportional to the sum of subtree lengths and compute discarded hashes. Depth limits bound the amplification but do not justify it. This violates AP-07/G9; H14/H16/H26 identify the unnecessary auxiliary work.

**Correction.** The immediate optimization can preserve existing key/wire meaning: render once, compute the root identity once, construct body-relative occurrences directly, and avoid syntax allocation for hash-only demand. A fuller compact representation may retain syntax and occurrence structure once per immutable body.

Do not confuse `ExpressionOccurrence.syntax: String`, part of a physical occurrence key, with `CheckedExpression.syntax`, the AST used for dependency discovery. Preserve lexical/binder distinctions, synthetic empty spans, physical admissions and positive/absent/membership consequences. A broader representation change needs explicit identity/wire evolution.

**Verification.** Compare addresses, admissions, dependencies and diagnostics for identical subexpressions, nested binders, shadowing, partials and empty spans. Deep versus balanced bodies can distinguish retained-text growth without native execution.

### <a id="f06"></a>F06 — Selected closure repeatedly scans and copies its checked universe

`CheckedPackage::select` scans the complete occurrence map twice per reached declaration, scans declarations to discover datasets supplying reached tables/kinds, then clones the complete package and removes unselected content. Its occurrence map is already ordered by declaration/role/position. The Salsa selected query consumes this operation. See [selection](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-modeling/src/check.rs) and [compiler consumer](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling.rs).

Reached declarations multiply examined occurrences even when complete selection is legitimate; sparse selection also copies material immediately discarded. This concerns the operation's admitted checked universe, not a claim that current canonical loading fetches the whole repository. Salsa backdating can stop downstream work after equality, but producing that equal result still pays these scans.

This violates AP-07/G9 under H5/H9/H13/H16.

**Correction.** Retrieve declaration ranges/grouped occurrences, prepare reverse dataset ownership where justified, and construct selected-only products. Preserve inherited members, refinements, imports, lexical visibility, engineering markers, provenance, supplier identity and test taint. A positive-only dependency graph would be an unsound replacement.

**Verification.** Independently grow reached declarations and unrelated expressions; compare exact selections/refusals for imports, refinements, shadowing, additions, deletions and missing names. Full typed-checker incrementality is a separate decision.

### <a id="f07"></a>F07 — Every conditional block materializes original coordinate metadata

`CasePlan::conditional` clones every variable, marks unselected ones fixed, copies all parameters and scans/clones instances before preparing another structure. `conditional_blocks` retains one such plan per block. Schedule construction also scans the complete edge set for each resulting block. The allocation bound explicitly accounts for repeated source populations. See [conditional plans](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/assembly.rs), [retained block consumer](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace.rs) and [schedule](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-structural/src/initialization.rs).

For B small blocks and V source coordinates, coordinate preparation/retention grows with B×V rather than actual local block membership. Independent scalar blocks are a legitimate sparse growth case. Runtime initialization, automatic block composition and PETSc block preparation consume the mechanism. Protective refusal is truthful but does not establish a credible target-scale route.

This violates AP-07/G9 under H2/H5/H15–H19. No exhaustion threshold or runtime improvement was measured.

**Correction.** Share one immutable original coordinate universe and retain compact block selections/overlays with original-ID mappings. Preserve the meaning that unselected variables are fixed, complete original binding validation, each block's actual dependencies, transactional stages and original post-solve assessment. Index predecessor discovery by consuming rows/blocks.

A physically local structure containing solved coordinates plus the complete arithmetic/guard/provider closure is another viable direction, but migrates more consumer assumptions. Sharing the original universe is the safer initial design.

A Jacobian-only closure is insufficient: guards and provider/physical obligations may depend on coordinates absent from numerical incidence. No solver algorithm or matching implementation needs replacement.

**Verification.** Increase independent scalar blocks and a genuine predecessor chain. Compare boundaries, support and outcomes while inspecting retained coordinate populations. Missing inputs, domain refusal, cancellation, stage rollback and original assessment must remain unchanged.

### <a id="f08"></a>F08 — Numerical difference inference lacks required physical facts

`engineering_error_quantity` derives point self-subtraction using `NoInvariantFacts`; numerical resolution receives the registry without actual prerequisite context. Conditional-boundary difference preparation has an analogous path. The registered `LogFugacityCoefficient` subtraction requires operand-contract facts, which the physical precondition owner can check. See [numerical inference](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-math/src/numerics.rs), [boundary preparation](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/src/workspace/modeling/executable.rs), [declaration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/packages/reference/physical/materials/physical.yaml) and [precondition semantics](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-quantity/src/preconditions.rs).

Lawful preparation needing these facts can therefore refuse even when the loaded physical inventory supplies them. This is a consumed-contract gap under AP-02/AP-04/G9. It is not evidence of invalid subtraction or bad physical results escaping admission.

This existing limitation already has [Plan 28f's investigation owner](../../plans/28f-shared-numerical-preparation.md#pc-saft-numerical-fact-investigation-boundary). The PC-SAFT harness recorded cold/warm/value/structural typed refusals; it did not uniquely identify the first failing caller. This review does not reattribute that first failure.

**Correction.** Supply current immutable prerequisite context to fresh numerical difference inference and affected boundary preparation, checking actual operands/operations and invalidating affected products when context changes. Do not substitute a scalar, invent allowances or treat an invariant ID as proof. Not every no-facts use is wrong: qualified receipt replay is a different operation.

**Verification.** Test present, absent and changed operand prerequisites, retain exact existing allowances, trace the first actual failure and rerun the unchanged corrected PC-SAFT fixture. Until then, no PC-SAFT prepared-product or performance claim follows.

### <a id="f09"></a>F09 — Selective result transport first builds the complete Arrow map

`RunResult::table` calls `tables()`, whose cache encodes every result relation. Durable publication calls the same operation before writing its first result block. Trajectory tables likewise expand all samples, outputs and available sensitivities; Python name discovery invokes full encoding. See [result cache](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/results.rs), [publication](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/durable.rs), [trajectory transport](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/modeling/trajectory.rs) and [Python boundary](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-py/src/workflow.rs).

A large completed trajectory/fitting result retains its native report plus a complete Arrow map even for a small-table request, name discovery or nominally bounded canonical transport. Unrelated relations can cause a useful small request to refuse. Blueprint §19.2 explicitly acknowledges the complete-map choice; honesty alone does not establish feasibility for the target's longer trajectories and studies.

This violates AP-07/G9 under H5/H6/H8/H10/H16.

**Correction.** Export requested relations from immutable completion and iterate chunks for canonical/streaming consumers. Discover names from the completion/request inventory. A retained single-batch API can materialize its requested relation without unrelated encoding. Existing builders, checked batches, block writers and manifest sealing are useful primitives.

Preserve scientific completion authority, exact identities/units, partial-result interpretation, escaped-buffer ownership and complete-manifest activation. Decide explicitly how the current sticky whole-map error contract evolves; silently changing it to per-relation errors changes observable behavior. This remedy removes unnecessary Arrow materialization, not inherent native-report retention.

**Verification.** Exercise a substantial completed trajectory with header-only access, name discovery and publication under a budget sufficient for native reports plus bounded transport but insufficient for the complete Arrow map. Confirm exact rows/manifests, scientific checks, interrupted-publication recovery and escaped-array validity. Measure peak allocation/latency only on matched workloads.

### <a id="f10"></a>F10 — Fitting repeatedly searches prepared response relationships

The fitting oracle repeatedly filters global observations by experiment. In transient forward-response refill, each included observation scans the experiment's complete response-term vector and then searches parameter bindings. Layout construction creates a response term per included observation/free-binding pair. See [oracle](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting/oracle.rs) and [layout](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/fitting/sparse.rs).

With m included observations and b eligible bindings, matching examines m²b response terms to emit mb contributions. That is a source-derived count of the inspected loop, not a timing model. Final response-rank assessment can switch a gradient fit into response mode, so the path is relevant beyond an explicitly response-mode fit.

This violates AP-07/G9 under H5/H9/H12/H15.

**Correction.** Prepare per-experiment observation lists and direct response ranges/entries, including local binding indexes, under the existing mapping owner. Refill then follows actual sparse contributions. No new scientific declaration, cache or dependency is necessary.

Preserve source order, repeated observations, exclusions/missing data, shared-parameter addends, sample/output identity, derivative upgrade allowances and independent final rank/candidate checks. Removing fresh scientific checks is not the remedy.

**Verification.** Compare predictions, sparse addends, gradients and rank diagnostics across steady/transient mixtures, repeated/excluded measurements and shared parameters, including the final gradient-to-response upgrade. Vary observations independently of bindings to distinguish relationship matching from required integration work.

### <a id="f11"></a>F11 — Retired analyses retain inaccessible derived graph payloads

`forget_analysis_results` withdraws source roots and writes retirement state. Analysis availability/begin then fence ordinary access/re-admission. No corresponding production collection of analysis node/edge payloads was found in the inspected generators, operations, runtime, scripts and Python consumers. Source/run reclamation does not collect those rows. See [retirement API](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-runtime/src/workflow/retention.rs), [retirement declaration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-codegen/src/codegen/surreal_retention.rs), [analysis lifecycle](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-codegen/src/codegen/surreal_analyses.rs) and [bounded graph admission](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-operations/src/canonical_analyses.rs).

Each graph is bounded, but repeated distinct analyses accumulate inaccessible payloads after retirement. Failed staged analyses can also leave partial graphs. This is AP-07/G9 lifecycle amplification; a per-analysis limit does not bound lifetime history. The bounded absence search does not exclude an unknown external administrative procedure.

**Correction.** Provide bounded resumable collection of derived edges/nodes, preserving header, method/configuration, input lineage, retirement fence and required digest/count receipts. Fence late staging/replay and retain interruption-safe progress. If permanent retired payload is intentional history, it needs an explicit useful audit/reopening purpose; currently ordinary access is fenced.

This does not claim the API promises immediate physical deletion. It separates useful immutable history from unbounded inaccessible derived storage.

**Verification.** Collect complete and partially staged retired analyses; interrupt/resume cleanup and race staging/activation against retirement. Preserve lineage and prohibit reopening, while source/run retention obligations remain governed by their owners.

## 6. Numerical stages and architectural gates

### Numerical stage coverage

| Stage | Formulation and guards | Derivatives | Scaling | Class/capability | Outcome and checks | Execution assessment |
|---|---|---|---|---|---|---|
| Authored checking and specialization | Original obligations, complete physical operation/context; guards precede algebra | Typed function/provider capability retained | Physical representation and nominal/error distinctions | No solver selected by syntax alone | Typed admission/refusal | F05/F06; F08 concerns fresh numerical-context consumption |
| Mathematical support and artifacts | Immutable admitted bodies and demanded support | Library symbolic/provider derivatives; required order admitted separately | Resolved policy and normalization | Representation/capability readiness explicit | Unsupported demand refuses | Retained complete basis and supplier topology are strengths |
| Structural/conditional preparation | Complete incidence, original rows/roles, explicit predecessors | Conservative support is distinct from numerical derivative values | Block views consume original policy | Root/NLP/native-feasibility admission differs | Named structural deficiency; no numerical-rank claim | F07; preserve complete guard/provider closure |
| Steady native solve | Admitted formulation and original domains | Required Jacobian/Hessian chain; no permission for silent approximation | Declared normalized solve and physical acceptance | Class-specific adapters; library iteration/factors | Typed termination plus fresh original-space observation and physical closure | Admission/drain ownership is strong; no new solver-core defect established |
| Initialization/recycle | Immutable overlays, declared strategy and finite termination | Same admitted mathematical capability | Resolved block/recycle policy | Shared admitted solve routes | Commit qualified coordinates; failed stages do not mutate specification | F07 affects preparation; dependency order itself is necessary |
| Dynamics and events | ODE/index-1 profile, consistent initialization, explicit event/schedule/endpoint | Forward/adjoint/second-order support varies by admitted route | State/residual tolerances retain physical interpretation | Diffsol/IDAS route limits explicit | Actual endpoint and partial trajectory remain distinguishable | F09 affects transport; universal derivative/integrator qualification not re-established |
| Fitting and response assessment | Same model plus experiments/observations and declared statistical interpretation | Responses or adjoints; upgrade and final rank demands explicit | Observation/parameter scaling and rank policy | Shared NLP/numerical owners | Fresh candidate and local response/rank validity | F10 removes matching work; F09 affects output |
| Scientific completion and transport | Original numerical/physical assessment precedes usable result | Derived quantities retain validity conditions | Physical tolerances/units remain observable | Transport does not select solver behavior | Complete/partial/failed outcomes and manifests separate | F09/F11; no transport optimization may manufacture scientific assurance |

### Foundation judgments

| Foundation | Verdict | Decisive reason |
|---|---|---|
| AP-01 — Separation | **Violated** | F01 couples native-local testing to canonical worker/store execution; useful separation elsewhere does not offset it |
| AP-02 — Stable contracts | **Violated** | F08 fresh inference lacks a consumed prerequisite context; F01 needs explicit execution effect scope |
| AP-03 — Composition | **Violated** | F01 unconditionally composes an unrelated complete journey into a narrow selection |
| AP-04 — Domain model and authority | **Violated** | Strong physical model, but qualification applicability omits consequential dependencies and numerical inference does not consume required context: F02/F08 |
| AP-05 — Explicit structure/constraints | **Violated** | F02's declared applicability boundary is incomplete; current native configuration is not established by old-path hashing |
| AP-06 — Local reasoning/testability | **Violated** | F01/F04 require unrelated build/infrastructure dependencies for legitimate local responsibilities |
| AP-07 — Execution fit | **Violated** | F03–F07/F09–F11 demonstrate avoidable scan, copy, retained-state or lifecycle amplification under credible target workloads |

### Independent gate judgments

These are design/source judgments at the stated scope, not a new product qualification receipt.

| Gate | Judgment | Evidence and limits |
|---|---|---|
| G1 — Authority | **Pass in inspected contracts** | Authored facts, policies, derived products, attempts and results have explicit owners; F02 is an incomplete applicability definition, not demonstrated competing mutable scientific authorities |
| G2 — Semantic fidelity | **Pass in inspected crossings** | Physical distinctions, original identities and typed absence/outcomes are retained; no new silent scientific-loss trigger established |
| G3 — Validity | **Fail for qualification applicability** | F02 permits a relevant changed condition to remain indistinguishable under an unchanged-input guard; scientific invalid-state admission was not separately disproved |
| G4 — Hidden behavior | **Pass for inspected pure preparation** | Tracked mathematics is separated from current publication/host effects; F01 is an execution-composition defect, not evidence that pure mathematical inspection performs I/O |
| G5 — Consistency/recovery | **Pass at inspected design level** | Exact protected manifests, terminal admission, drain and tombstones distinguish partial work; arbitrary concurrent schedules and assembled deployment qualification remain outside this pass |
| G6 — Transformation/reuse | **Fail for qualification reuse** | F02 incomplete dependencies; mathematical complete-basis equality/rebinding remain strengths |
| G7 — Truthful capability claims | **Fail for unchanged-input applicability claim** | F02's identity can omit consumed changes; honest unsupported numerical refusals and cancelled reports remain correctly distinct |
| G8 — Library leverage | **Pass for inspected mechanisms** | Established mathematics, matching, graphs, factors and solvers own generic algorithms; findings do not establish a clearly fitting library controller being reimplemented |
| G9 — Architectural fitness | **Fail** | Applicable foundation violations above; neither correct current outputs nor proposed remedies offset them |
| PS-G1 — Physical consistency | **Unresolved for whole-simulator acceptance** | Inspected physical contracts and closure separation are credible; all property envelopes, reaction balances and scientific references were not requalified. F08 refuses rather than admits invalid physics |
| PS-G2 — Well-posedness | **Pass for inspected admission design** | Explicit roles, complete scope, class-aware matching refusal and semantic IDs precede solving; numerical rank remains separate |
| PS-G3 — Numerical integrity | **Unresolved for whole-simulator acceptance** | Fresh original-space assessment and typed outcomes are present, but every derivative/provider/dynamic route and current assembled composition was not independently qualified |

The unresolved scientific judgments describe an acceptance-evidence limit, not additional defect findings. No new PS-G1/PS-G3 failing result trigger was established. The overall decision already requires revision for demonstrated architectural and applicability failures.

## 7. Library fit, alternatives and remedy compatibility

The existing library composition remains the strongest starting point. Symbolica/Numerica owns mathematical programs and derivatives; native solvers own fitting iteration/globalization/factors; structural libraries own matching and graph algorithms; Salsa owns downstream incremental preparation; Arrow owns boundary arrays; canonical operations own storage visibility and retention. Integration owners absorb capability, lifecycle and version differences.

Most corrections are operation-shaped indexing, views and export contracts inside existing owners. Replacing a library/container alone would leave repeated scans, coordinate copying or complete-map materialization intact.

| Alternative | Benefit | Cost/limit | Judgment |
|---|---|---|---|
| Retain current paths and raise budgets/concurrency | Least immediate change | Preserves work amplification and lifetime growth; no guarantee ordinary work becomes feasible | Insufficient closure for the findings |
| Narrow existing execution/build selection | Removes unrelated dependency/effect requirements | Must preserve actual Arrow validation, native provenance and drain | Preferred for F01/F04 |
| Indexed selected products and shared conditional views | Follows actual declaration/block demand under one semantic owner | Requires complete dependencies and original-ID mappings | Preferred for F05–F07/F10 |
| Relation/chunk export from immutable completion | Removes unrelated Arrow materialization | Explicit error-contract evolution and publication completeness required | Preferred for F09 |
| Active/cold ownership and bounded analysis collection | Separates live coordination/payload from useful history | Requires exact lookup, fences and crash/drain argument | Preferred direction for F03/F11 |
| Full typed incremental checker | Could reduce structural edit work | Must track lexical membership, absence, physical facts, documents, provenance and failure atomicity | Further design decision; not prerequisite to F05/F06 |
| Broader graph/store or numerical placement pivot | Could expose useful native set/traversal capabilities for a demonstrated workload | Does not inherently fix these application loops or transport lifetimes; introduces migration/qualification obligations | Not supported as the remedy by this review |

The remedies compose without creating another policy authority. F01/F04 narrow dependencies; F02 makes evidence for those actual operations sound. F05/F06 reduce source preparation; F07 shares original structure while retaining complete semantics. F08 supplies a required context and is not interchangeable with these optimizations. F10 indexes immutable relationships without weakening final numerical assessment. F09 reduces transport representation; F11 later reclaims derived storage without erasing lineage.

Consequence-based priority differs from prerequisite order. For example, compact conditional views may have greater runtime-scale consequences than a source-key cleanup, while physical occurrence representation or complete context contracts may be prerequisites for a broader implementation. Relation-aware export needs its failure/publication contract settled before streaming replaces the complete map. Hot-history separation cannot precede authentic drain proof.

No remedy is justified merely by fewer source lines, a new library name, a new graph representation or a larger number of workers.

## 8. Opportunities and uncertainties that are not findings

The following were deliberately kept outside the finding list:

- **Checked gather forecasting.** Selected `take_reserved` can allocate counts over the complete source and forecast whole-source decode extent for a subset. The historical quadratic multiplicity bug is repaired. Bounded blocks, aliases and ownership matter; no material ordinary exhaustion trigger was established here. Preserve duplicate multiplicity, nested/variable-width aliases, checked bounds/nulls and preallocation admission before narrowing the forecast.
- **Scalar-cell verification.** Complete scalar metadata is rebuilt/sorted after block admission. The earlier claim of ordinary repeated decoding once per sixty-four matching scalar cells was retracted: exact entity/field/partition keys ordinarily provide uniqueness. Selected verification remains an opportunity, not that paging defect.
- **Analysis size.** Initial connected analysis admits 4,096 nodes and 8,192 edges, and some analysis paths construct the complete bounded graph before root selection. Honest limits do not provide a larger-scale route, but whether the limit obstructs ordinary required analysis needs a concrete workload premise.
- **Upstream full checking.** Changed publication still runs the complete checker before downstream Salsa reuse. Existing bounded inquiry measurements are useful leads; they do not establish that a full incremental-checker redesign is presently the best correction.
- **Recipe persistence and hashing.** Existing inquiries retain current mechanisms and expose prerequisites; they do not authorize a production cutover. The historical 0.535 µs key traversal and 150.857 ms complete warm preparation measure different operations and establish neither a speedup nor an internal latency share.
- **Build tuning.** No evidence here justifies changing floating-point compiler guarantees, global dependency optimization, linker choice or toolchain policy. Work removal and actual consumer locality should be assessed before broad tuning.

These uncertainties may influence later remedy selection or prioritization. They do not weaken the concrete findings, and they are not an invented implementation backlog.

## 9. Verification, conformance and evidence limits

This review used source, contracts and retained evidence first. The principal reviewer ran no builds, tests, probes or formatters. No remedy was implemented.

The coordinator executed one bounded input-projection probe:

```bash
scripts/pse-env --resource-class light -- .venv/bin/python -B docs/design_review/evidence/efficiency-principles-codebase-2026-10-09/input-scope-probe.py
```

**Tested:** exit 0 on pinned Python 3.14.7, scope version 2. The expected baseline was detection of every changed consumed input; the omissions reproduced while positive controls were detected. [Source](../evidence/efficiency-principles-codebase-2026-10-09/input-scope-probe.py) and [output](../evidence/efficiency-principles-codebase-2026-10-09/input-scope-probe.json) are retained. This establishes projection behavior only, not an actual successful-report reuse or scientific error.

The first wrapped attempt exited 125 before execution because host admission lacked capacity. `just ready` then passed skills synchronization and doctor against the zero-failure baseline; an ordinary retry succeeded. No admission bypass occurred. The earlier unwrapped diagnostic is superseded by the pinned probe.

The user-cancelled feature matrix remains stopped. Its local-only receipt is:

```text
build/assessment/20261009T222854.275137Z-features-powerset-2890517-689811/summary.md
```

The combination command was interrupted after 1,539.9 seconds; its log reached the started `207/313` entry. The following no-default command was interrupted after 7.5 seconds. These are unsuccessful/interrupted observations against the zero-failure baseline, not feature coverage or product acceptance. The report's `input_coverage: false` correctly prevents unchanged-input reuse; it is not evidence of a stale pass being reused.

| Scientific/conformance scope | Status used by this review |
|---|---|
| Generic physical/quantity and structural mechanisms | Implemented paths and existing named controls inspected; no new product execution |
| PC-SAFT preparation fixture | Existing cold/warm/value/structural typed refusals; not accepted preparation/performance evidence |
| Other unit/property models and thermodynamic references | Existing conformance/parity retain their original fixture, source and tolerance conditions; not requalified here |
| Dynamics, fitting, recycle and result publication | Source/design coverage plus retained scoped evidence; no fresh whole-route campaign |
| Plan 30 runtime/environment | Its completed scoped outcome is distinct from broader Plan 28 acceptance |
| Plan 28 E3/E4/E5 | Assembled qualification/measurement/acceptance remain with 28e; this review does not restart or close them |

Blueprint §24.2's earlier qualification basis and current plan evidence must not transfer automatically to changed products. Source establishes the described structural mechanisms. Quantitative latency, RSS, throughput and capacity improvements remain unmeasured. Publication checks, if run by the coordinator, establish documentation mechanics only.

## 10. Rule impacts and follow-up ownership

The following impacts are recommendations for operator consideration. This review applies none of them.

| ID | Current rule or contract | Proposed impact | Dependent finding; if retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Every Rust test invocation activates relation force-validation; AGENTS.md, Cargo aliases/validation metadata and blueprint §24.1 | First seek activation without unrelated selected targets. If impossible, require full validation on actual Arrow-consuming closures instead | F04; if unchanged and no narrow mechanism exists, retain the extra build as an explicit assurance tradeoff and do not claim pure build isolation |
| <a id="rc02"></a>RC02 | Complete-map encoding and sticky shared error behavior, blueprint §19.2 and `RunResult::tables` | Permit relation-aware/chunked transport; explicitly decide whole-map versus per-relation error semantics | F09; retaining complete-map transport retains the small-demand/large-materialization limitation |
| <a id="rc03"></a>RC03 | Analysis retirement withdraws retention while lineage remains immutable, blueprint §20.4 and current analysis operations | Make derived payload collection explicit and bounded while retaining receipts/fences | F11; if permanent payload is retained, state its useful retention purpose and accept continued lifetime accumulation rather than claiming reclamation |
| <a id="rc04"></a>RC04 | Physical occurrence keys retain subtree syntax under existing occurrence identity/wire meanings | Immediate duplicate-work removal needs no semantic change. A fuller body-relative compact representation requires explicit versioned meaning/migration | F05; preserving the current representation still permits the narrow optimization but retains subtree-text storage |
| <a id="rc05"></a>RC05 | Implemented input-scope version 2 and its projected consumed-input definition | Expand conservative coverage and advance interpretation version; preserve old receipts as historical | F02; this corrects implementation to the intended complete-input contract. Keeping incomplete scope cannot support the unchanged-input claim |

No weakening of source/native-byte association, physical prerequisites, original-state verification, manifest completeness or drain ownership is proposed.

F03's hot/cold representation change fits Plan 30c's existing minimal retained outcome/provenance/reclamation contract. Archiving must preserve exact historical identity and no-recreation/cleanup semantics; it cannot silently reinterpret unknown old records as drained or successful. F01 similarly fits blueprint §24's requested-effects intent. These are implementation-contract corrections, not new policy waivers.

If adopted, current status should live in the existing coordinator rather than this document:

| Findings | Proposed work owner, subject to adoption |
|---|---|
| F01/F02/F04 | Tooling/qualification owners, coordinated through Plan 28h/28e and relevant Plan 30c/30d contracts |
| F03 | Test/native resource-lifecycle owners; Plan 30c/30d with 28h integration where relevant |
| F05/F06 | Modeling/compiler preparation owners; Plan 28b/28j/28f as applicable |
| F07/F08/F10 | Numerical preparation/workflow owners in Plan 28f; F08 already has its named investigation boundary |
| F09/F11 | Result/analysis/lifecycle owners; Plan 28g with canonical execution/serving integration |
| Assembled acceptance and quantitative claims | Plan 28e after authorized correction and affected functional evidence |

These are routing suggestions, not schedules or authorizations.

### Adoption handoff — 2026-10-09

After this review, the maintainer selected a standalone remediation workstream rather than
the proposed Plan 28 distribution. [Plan 33](../../plans/33-efficiency-principles-remediation.md)
is the sole current disposition, implementation and affected-qualification owner for F01–F11
and the six opportunities in §8. Existing Plan 28 scientific/campaign obligations remain
with their owners; they do not gate Plan 33's completion.

The maintainer confirmed RC01–RC05 and then superseded this review's preservation proposals:
retired analyses must ultimately be removed entirely, including lineage, headers and markers;
disposable generated internal state can be rebuilt without legacy readers or migrations;
obsolete internal history need not be archived. Current protection, reference and drain
obligations remain. Tests use current contracts, applicable analytical expectations and
independent/pinned external IDAES/Pyomo references. Invalid internal historical outputs are
not correctness baselines. First-principles thermodynamic validation is a future direction,
outside the current remediation scope. The original diagnoses and evidence conditions above
are unchanged; the plan records the selected remedies and adoption routes.

## 11. Decision and next consequential choices

**Behavioral/semantic adequacy: Revise.** Qualification applicability has a concrete complete-dependency failure. Whole-simulator physical/numerical acceptance remains unresolved at this review's evidence level; no new escaping physical inconsistency or misclassified numerical solution was demonstrated.

**Architectural fitness: Revise.** Legitimate local development and simulator growth scenarios violate applicable foundations through unnecessary dependencies, scans, coordinate copies, result materialization and retained-history work.

**Overall: Revise the current physical paths while retaining the semantic/numerical foundation.** The strongest evidence is decisive source tracing, supported by the pinned projection probe. It establishes the diagnoses, not implemented remedies or quantitative gains.

| Consequence-based priority | Change | Acceptance obligation |
|---|---|---|
| Evidence soundness | F02 complete qualification applicability | Complete relevant-input controls and an actual retained-reuse refusal; historical interpretations preserved |
| Supported capability and runtime-scale feasibility | F08 physical facts; F07 compact blocks; F09 bounded/selective transport | Lawful facts admitted, missing facts refused; actual local block scope; exact result/publication behavior under bounded transport |
| Local build/test turnaround | F01/F04 actual execution/build dependencies | Local-only and Arrow-free responsibilities avoid unrelated journeys while real consumers retain supervision/validation |
| Repeated preparation/evaluation | F05/F06/F10 direct occurrence/selection/response work | Exact semantic products and scientific checks preserved; removed scans/copies not merely moved |
| Repeated-use lifetime | F03/F11 active history and derived payload reclamation | Actual drain, exact immutable receipts, retirement fences and interruption-safe cleanup |

The next design choices are the explicit native-local/canonical execution role, the narrow force-validation mechanism, compact conditional-view ownership, and relation-aware export/error behavior. Conservative applicability correction need not wait for them. Any fuller occurrence representation, incremental checker or payload-retirement protocol requires its own complete meaning and failure argument before implementation.

The review's intended artifact is `docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md`. Findings F01–F11 and impacts RC01–RC05 remain recommendations until dispositioned through the existing planning route.
