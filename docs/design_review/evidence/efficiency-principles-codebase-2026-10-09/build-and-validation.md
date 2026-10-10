# Build and validation efficiency: supporting assessment

This is a bounded independent supporting assessment for the coordinator's comprehensive
efficiency review. It does not create a second overall simulator verdict or finding
disposition ledger. The [principal review](../../reviews/design_review_efficiency-principles-codebase_2026-10-09.md) owns the combined judgment;
[Plan 28](../../../plans/28-surrealdb-unified-substrate.md) owns existing efficiency finding
dispositions. BF01–BF04 below are supporting identifiers for reconciliation into that review.

The developer lifecycle has useful separation and several corrected inefficiencies, but narrow
test execution still acquires unrelated build or runtime dependencies, and qualification reuse
does not capture all consumed inputs. The bounded architectural judgment is **Revise**. This
does not establish a scientific defect in computed simulator results.

## Scope, baseline and evidence

The inspected baseline is clean main HEAD `4c24721e691187e1a5b28398b29722fbde671da8`, which
superseded the assignment's original baseline during startup. This assessment applies
[Core 3.4](../../design_principles/core/design-principles.md),
[Efficient Architecture Heuristics 1.0](../../design_principles/core/efficient-architecture-heuristics.md),
[Process Simulator 1.5](../../design_principles/profiles/process-simulator/principles.md), the
[template](../../design_principles/core/design-review-template.md) and the
[pse-arrow binding](../../design_principles/binding/pse-arrow.md).

The target is efficient local development of a physically meaningful, local-first extensible
simulator. A developer changing a physical definition, compiler operation, native adapter or
result boundary should exercise that responsibility with its actual dependencies. Native
linking, numerical execution, canonical persistence, test disposition and retained qualification
have distinct purposes and lifetimes. A native adapter test can need an actual linked library
without needing a canonical server or worker process.

Coverage includes workspace dependency/features/profiles, compiler environment and cache
composition, native installation admission, generation ownership, Rust/Python test selection,
fixture lifecycle, validation orchestration and retained applicability, producer/deployment
association, and generated publication/equivalence. Adjacent production contracts were inspected
where they explain test dependencies. This supporting scope does not qualify solver accuracy,
physical consistency, structural well-posedness, performance campaigns, other platforms or
distribution artifacts.

No builds, product tests, feature-matrix continuation, benchmarks or formatters were run for
this assessment. Source inspections establish the described implemented paths. An initial
synthetic projection probe used `python3 -B` outside the pinned checkout wrapper; its output is
diagnostic support only, not a pinned **Tested** claim. The coordinator retained a reproducible
[probe source](input-scope-probe.py) with positive controls. Its first pinned-wrapper invocation
was admission-refused with exit 125 because host capacity was unavailable. After `just ready`
passed, the normal pinned-wrapper retry succeeded with Python 3.14.7 and exit 0. Its
[retained output](input-scope-probe.json) establishes projection behavior only. No admission
bypass or actual retained-reuse experiment occurred.

## Responsibilities and preservation constraints

| Responsibility | Current owner and useful boundary |
| --- | --- |
| Dependency versions and feature graph | [Workspace manifest](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/Cargo.toml), [Cargo configuration](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/.cargo/config.toml), [hakari](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/.config/hakari.toml) |
| Compiler environment and cache composition | [build_environment.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/build_environment.py) |
| Native installation admission and generations | [native_cache.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_cache.py), [native_operation.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_operation.py) |
| Test selection | nextest/pytest, adapted by [select.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/select.py) and [native_tests.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/native_tests.py) |
| Actual execution placement | [host_admission.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/host_admission.py), [surreal_server.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/surreal_server.py), [test_run.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/test_run.py) |
| Terminal test outcome | Existing composer in [validation.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation.py) and [validation_receipts.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/validation_receipts.py) |
| Disposable resource lifetime | [test_resources.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/test_resources.py), independent of fixture Drop |
| Producer/deployment association | [producer_deployment.py](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/scripts/producer_deployment.py), Rust capture and [buildinfo identity](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-buildinfo/identity.rs) |
| Generated publication/equivalence | [pse-codegen](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-codegen/Cargo.toml), [xtask generation](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/xtask/src/codegen.rs), specific [recipes](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/justfile) |

The main remaining problems occur when these responsibilities compose, rather than from a
lack of named modules.

### Strengths and corrected historical mechanisms

Pure semantic roots no longer directly consume the workspace hack. The inspected
[model](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-model/Cargo.toml),
[quantity](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-quantity/Cargo.toml) and
[compiler](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-compiler/Cargo.toml) manifests preserve the semantic/data
boundary, reflected by hakari's `final-excludes`. The former EF04 diagnosis against the model's
normal dependency graph should not be repeated unchanged. BF04 concerns a separate test-build
selection mechanism.

[pse-buildinfo/build.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-buildinfo/build.rs) now embeds compiler
release/profile rather than watching and embedding the complete outer source tree.
`BuildInfo::captured` leaves unembedded outer fields empty. Deployment independently observes
actual artifacts and verifies receipts. This substantially corrects the former PE06 physical
invalidation mechanism. It does not close the plan's actual three-role or assembled
qualification. [Plan 28h](../../../plans/28h-native-setup-and-artifact-identity.md) remains the
scoped setup/artifact owner.

`native_cache.prepare` now separates construction from publication, hashes outside publication
coordination, publishes immutable generations and remembers admission within an authentic
operation. `native_operation.drained` retains ownership while descendants survive. This
substantially corrects the old PE05 complete-verification-under-exclusive-preparation mechanism.
A fresh operation still verifies installation bytes, consistent with its stated trust boundary.

`build_environment.configure` preserves workspace incremental compilation, separate checkout
target directories, optional compiler caching and actual compilation inside the caller's
supervised lifetime. The inspected evidence does not justify changing numerical compiler
guarantees, globally changing dependency optimization or replacing the selected linker.
[Build reuse guidance](../../../dev/build-performance.md) correctly distinguishes cache canaries,
build observations and product qualification.

`xtask::codegen::write_tree` skips byte-identical files and removes stale owned outputs. The old
no-op rewrite finding is corrected in this implementation. `compare_tree` retains bidirectional
generated equivalence rather than weakening it to expected-file presence. Registry authority,
bootstrap-safe generation and complete equivalence should survive any build-locality correction.

Python's [canonical_substrate fixture](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/python/pse/tests/conftest.py) is explicitly
requested, not autouse. Its UUID database isolation and post-terminal cleanup model are useful.
A database test's full-schema setup is not itself evidence of slow production queries.

[dependency_ceilings.rs](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/xtask/src/dependency_ceilings.rs) deliberately excludes the
workspace-hack edge. Its reported normal closure is therefore a semantic dependency check, not
a complete cold-build closure measurement. Generator/data-boundary roots still inherit
substantial hack dependencies. That tradeoff warrants an actual build scenario if consequential;
this assessment does not invent a quantified defect from the crate names alone.

## Findings and corrective behavior

### BF01 — Narrow native unit execution acquires a complete worker/store observer journey

**Implemented diagnosis.** `test_run.run_rust` routes any invocation with
`PSE_NATIVE_OPERATION`, without retained binary metadata or its child sentinel, to
`observer_rust`. The condition does not ask whether selected tests use canonical persistence.

`observer_rust` calls `native_tests.ordinary_rust_capture`; `_rust_capture` unconditionally calls
`worker_binary` before building and enumerating the selected test binaries. `worker_binary`
builds `xtask::pse-worker` with
`native-solvers,canonical-tests,pse-relations/force-validate`. The path then always calls
`surreal_server.observer` with `exclusive-observer`, requiring canonical service configuration
and the observer lifecycle.

A revealing legitimate selection is:

```text
just unit-native-capability-package pse-backend-native ipopt solver status_scope_and_finite_infinity_are_not_conflated
```

The inspected [Ipopt status test](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/src/ipopt.rs) checks
termination classification, infinity conversion and integer bounds. It constructs no runtime,
database or worker. Nevertheless, the generic selected-unit route builds the full worker and
enters a store observer.

There is a possible fresh-environment failure in addition to unnecessary work. The recipe
requests only `solver`, but the unconditional worker build enables root isolation, Uno, PETSc
and other native features. [The backend build script](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-backend-native/build.rs)
requires `PSE_ROOT_ISOLATION_DIR` for root isolation. The narrow setup request does not establish
that input. This failure path is source-supported and was not executed; an inherited broad
native environment can conceal it.

**Consequence and criteria.** An ordinary adapter/status change requires unrelated native build
inputs and canonical infrastructure. It can compete for the observer lane despite not using
the service. AP-01, AP-06 and AP-07 are violated in this scenario; H5, H21, H23 and H25 explain
the unnecessary dependencies. Native linkage and canonical execution effects are independent
variation axes.

**Proposed correction.** Make the execution role an explicit input to the existing composition
root. Pure/native-local selection retains ordinary process admission, authentic native setup,
exact nextest selection, terminal reconciliation and required native drain. Canonical selections
retain worker capture, service/receiver admission and the observer route. The simplest viable
alternative is a narrow existing-runner route selected by the recipe's declared effects. No new
test-name manifest, source classifier, scheduler or independent test registry is needed. Unknown
effect scope can remain conservative.

Do not infer pure execution from `unit`: unit-labelled tests can legitimately exercise canonical
contracts. A test requesting canonical resources through a local-only route must fail clearly
or request the proper execution role. Merely replacing the marker or moving the unconditional
worker build into another helper would not correct the dependency.

**Verification.** Exercise the inspected status test with only its declared native capability
and without a configured canonical service; establish that no full-worker build or observer
launch occurs. Separately retain positive canonical/managed journeys proving worker association,
resource placement, truthful terminal outcome and descendant drain.

**Rule impacts.** The desired behavior agrees with blueprint §24.2's requested-effects boundary
and the targeted recipe's stated scope. No new waiver of resource supervision, native drain or
canonical fixture obligations is proposed. The current runner's implicit route changes; if an
owning plan treats that route as a universal native-test requirement, it must be corrected with
the mechanism rather than silently preserved as an exception.

### BF02 — Qualification applicability omits consumed orchestration and environment inputs

**Implemented diagnosis.** `validation_scope.RUST_INPUTS`, inherited by the Python product
scope, omits `scripts/test_run.py`, `scripts/test_resources.py`, `scripts/host_admission.py`,
`scripts/surreal_server.py` and `benches/`. The first four directly govern execution, fixture
identity, placement, service admission, terminal ownership or cleanup. `benches/` contains Rust
benchmark consumers but is absent from the product projection.

`PRODUCT_ENVIRONMENT` also omits supported native inputs including `UNO_DIR`, `PETSC_DIR`,
`SUITESPARSE_INCLUDE_DIR`, `CC`, `CXX` and `CMAKE_TOOLCHAIN_FILE`. Their consumption is explicit
in `native_operation.CAPABILITY_PATHS`, `native_operation.environment` and
`native_cache.INPUT_ENV`.

The complete source snapshot contains the scripts, but `validation_receipts._reuse_guarded`
compares only `input_identity(...)`. The assessment's final relevant-drift check uses that
projection too. A full outer snapshot does not repair the applicability gap.

**Tested projection observation.** The coordinator ran the [retained probe](input-scope-probe.py)
through `scripts/pse-env --resource-class light -- .venv/bin/python -B` on this file, using
Python 3.14.7. Synthetic before/after values passed to the actual `input_identity` left both
product identities unchanged for every listed omitted path and environment name. Changed
crate source, `native_tests.py` and `IPOPT_DIR` positive controls changed both identities.
The [output](input-scope-probe.json) uses `false` for undetected changes and `true` for
detected changes. This tests the projection, not actual retained qualification reuse.

**Native revalidation limit.** `validation_receipts.verify_native` rehashes paths stored in the
old capture and checks its recorded thread budget. It does not resolve a newly selected active
native prefix and compare that closure with the old one. Keeping the old captured libraries
unchanged while selecting a different `UNO_DIR` or `PETSC_DIR` does not establish the old
capture's applicability to the new configuration.

**Consequence and criteria.** An earlier successful observation can be labelled unchanged-input
reuse despite changed execution/cleanup behavior or supported native configuration. This is a
validity and evidence-applicability defect, not proof that an earlier scientific result was
numerically wrong. AP-04/AP-05, DP-09/DP-22 and G3/G6/G7 apply.

**Proposed correction.** Restore a conservative consumed-input boundary and bump its
interpretation version. Including all execution scripts is the simplest initial correction;
a narrower explicit closure is acceptable only if complete. Native qualification must include
effective supported native/tool inputs, or an owner-established complete configuration identity.
Preserve old receipts' historical scope rather than relabelling them under the expanded
definition. No global change-impact engine is necessary. Keeping unrelated documentation
outside a product identity remains reasonable.

**Verification.** Positive controls must detect changed crate and `native_tests.py` inputs;
controls must detect each currently omitted consumed script/native selector. An actual successful
retained report should refuse unchanged-input reuse after such a change. Reviewed transfer
remains explicitly historical with its applicability rationale. Old-library hash checks must
not substitute for establishing the current selected configuration.

**Rule impacts.** Conservative consumed-input coverage is already the intended blueprint §24.2
contract. Correct the implemented scope/version and its consumers; no weakening of exact source,
native-byte or receipt association is proposed. Expanded interpretation must not retroactively
promote old receipts. This finding does not demand automatic requalification of prior scientific
results merely because development inputs changed.

### BF03 — Active coordination repeatedly processes completed resource history

**Implemented diagnosis.** `test_resources.resource_metadata` uses `host_admission.metadata`,
which takes one exclusive filesystem lock, parses the complete JSON ledger and rewrites it on
context exit. This applies even to `resource_status` reads. Every use validates accumulated
records. `register`, `finalize`, `register_report`, `report_resource` and `reclaim_reports` scan
the owner map. Reclamation marks entries `removed` or `compacted`; no retirement from the active
ledger was found in the inspected `scripts/` production paths. `reclaim_reports` limits selected
cleanup candidates but discovers and sorts them from the complete owner history.

Similarly, `native_operation.Operation.__exit__` intentionally retains records;
`generation_in_use` scans `.operations/*.json` when retired generations are considered for
collection. This native scan does not occur on every ordinary current-generation lookup.

A read-only filesystem observation during this turn found the following in the selected default
state/cache locations. These counts are diagnostic snapshots, not timing or capacity results:

| Structure | Observed state |
| --- | --- |
| Test-resource ledger | 226 records; 93 already removed/compacted; 211,549 bytes |
| Native `.operations` | 830 JSON files; 50,863,647 bytes |

The absence search covered production paths under `scripts/`; it does not establish that no
external operator tool can prune these structures.

**Consequence and criteria.** Registration/status/cleanup coordination grows with completed
historical runs as well as live participants. Native generation retirement reads historical
admission payloads whose full contents are unnecessary after authentic drain. This is structural
work amplification under repeated development and qualification; AP-07 and H9/H23/H26 apply.
No share of current command latency is attributed to it.

**Proposed correction.** Separate the bounded active ownership set from retained final receipts.
After authentic final drain and terminal disposition, retain the minimal immutable
identity/outcome/reclamation record through exact lookup or cold archival storage; stop rewriting
and scanning its complete historical payload during ordinary active operations. The simplest
viable direction is bounded active filesystem metadata plus exact retained receipts. A new
database service is not justified merely by this diagnosis.

Preserve failed/incomplete/unknown pins, exact reference acquisition, no-recreation semantics,
crash-resumable cleanup and PID/unit-generation protection. Pending handoffs and unverifiable
native owners remain active. Never retire by age or parent exit alone. Native generation-use
admission payloads may be compacted only after the same actual-drain proof that permits their
retirement; a missing file cannot become evidence that work drained.

**Verification.** Holding active participants constant while adding completed archived records
should not enlarge ordinary active metadata reads/writes or generation-use scans. Exact
historical queries, repeated cleanup, reference races, unknown owners and surviving native
descendants retain their current safety behavior.

**Rule impacts.** [Plan 30c](../../../plans/30c-test-ownership-and-evidence-retention.md) already
requires minimal retained outcome/provenance/reclamation receipts and preserves failed/unknown
materials. A different hot/cold representation changes no required retention or drain policy.
No age-based sweep, stale-owner stealing or automatic deletion of failure evidence is proposed.

### BF04 — Pure semantic tests build an unrelated Arrow/DataFusion test target

**Implemented diagnosis.** `select.unit` always adds
`-p pse-relations --lib --features pse-relations/force-validate`. It then intersects the execution
filter with the user's packages, excluding relation tests unless explicitly requested. Thus
`just unit-package pse-compiler ...` selects an unrelated relation test target for compilation
even though its tests will not run.

[pse-relations](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/crates/pse-relations/Cargo.toml) has full `datafusion` as a
dev-dependency, plus optimizer/functions/physical-expression and workspace-hack normal
dependencies. The inspected compiler/model/quantity manifests have no Arrow test dependency.
Corrected semantic normal-dependency ceilings do not remove this test-build dependency.

**Consequence and criteria.** The recommended local test route restores substantial unrelated
cold compilation/linking dependencies for pure semantic work. Fixing BF01 does not fix this,
and fixing this does not remove BF01's worker/store widening. AP-06/AP-07 and H5/H20/H28 apply.
The extra selected target is explicit in source; no quantitative build penalty was measured.

**Proposed correction.** A pure-owner test route builds its selected actual dependency closure.
Force-validation remains required for every selected test that consumes Arrow. If the pinned
Cargo configuration can activate the required feature without selecting an unrelated relation
test target, preserve the current assurance policy mechanically. Cargo behavior was not probed
because builds/checks were prohibited. If it cannot, change the every-invocation mandate into
an assurance requirement on actual Arrow-consuming test closures.

This preserves array validation, physical admission, independent scientific checks, explicit
test oracles and single-family guarantees. A genuinely Arrow-free test closure has no array
boundary to validate. Do not exempt a supposedly pure test whose actual dev-dependency closure
does consume Arrow; selection must reflect the real consumer boundary.

**Verification.** A cold pure compiler/quantity test selection should produce no unrelated
relation test artifact. Actual Arrow-consuming selections must still demonstrate enabled
force-validation. Alternating pure/composite/native selections must retain intended workspace
feature sharing and the one type universe. Source changes in related consumers must still
invalidate their actual tests. The exact Cargo mechanism remains a design/implementation
question, not an established alternative API claim.

**BF-RC01 — Assurance scope for pure test closures.** The semantic correction may require
changing [AGENTS.md](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/AGENTS.md)'s every-test feature mandate, Cargo's `c`/`t` aliases,
workspace validation metadata, the `pse-relations/force-validate` declaration comment and
blueprint §24.1. The proposed guarantee is full validation of consumed Arrow boundaries without
building an unrelated target solely to activate a feature. If that rule is retained and no
narrower Cargo mechanism exists, retain the extra compilation as an explicit assurance-policy
tradeoff; do not claim pure test-build isolation. No rule change is applied by this review.

## Existing command evidence and its limits

The feature matrix remains explicitly cancelled. The retained
assessment summary (local-only `build/assessment/20261009T222854.275137Z-features-powerset-2890517-689811/summary.md`)
records 1,539.9 seconds for unsuccessful `features-combinations` and 7.5 seconds for interrupted
`features-no-default`, against a zero-failure baseline. The
combinations log (local-only `build/assessment/20261009T222854.275137Z-features-powerset-2890517-689811/features-combinations.log`)
reached the started `207/313` entry and ended with signal 15. The recipe uses
`cargo hack check --feature-powerset --depth 2`: bounded compilation enumeration, not exhaustive
feature coverage or product test runtime. Duration includes command/setup/check work and does
not measure solve latency. No restart or continuation occurred in this assessment.

That group report has `input_coverage: false`; `_reuse_guarded` correctly refuses it. It is not
an example of an affected reusable receipt under BF02. Historical logs keep their original
source and conditions and do not qualify the current tree merely because their files remain.

The pinned synthetic projection probe is **Tested** for the omitted-input behavior only.
Its first invocation was admission-refused 125; the later normal retry passed. An actual
successful-receipt reuse scenario is **not_run**. All proposed corrections
remain **Proposed**; none has new functional or performance qualification from this assessment.

## Scoped architectural and behavioral judgments

| Foundation | Judgment for this supporting scope |
| --- | --- |
| AP-01 Separation | Violated at native-link versus canonical execution routing; useful owners elsewhere |
| AP-02 Contracts | Useful native/terminal/resource contracts; their composition needs explicit effect scope |
| AP-03 Composition | Violated by unconditional worker/store composition of narrow native selections |
| AP-04 Domain/authority | Qualification applicability is incomplete; outcome and drain authorities are usefully separated |
| AP-05 Explicit structure | Violated by incomplete consumed-input declarations |
| AP-06 Local reasoning/testability | Violated by BF01 and BF04 |
| AP-07 Execution fit | Violated by unnecessary test dependencies and historical coordination amplification |

G3/G6/G7 are adversely affected by BF02's applicability claim; G4 by BF01's implicit
infrastructure effects; G9 fails independently. Scientific PS-G1–PS-G3 were not qualified by
this tooling assessment. The inspected native installation and fixture lifecycle protections
are preservation constraints, not evidence of whole-simulator recovery or scientific acceptance.

BF01 and BF04 are distinct because native execution effects and forced test-build roots require
different corrections. BF02 is a validity/applicability correction independent of their speed.
BF03 changes the physical lifetime of coordination metadata without weakening its retention
meaning. These remedies compose: a narrower execution route still receives complete relevant
qualification inputs and exact resource outcomes, while archived completed ownership stops
inflating live coordination.

No library substitution is required by these findings. Existing Cargo/nextest/pytest selection,
filesystem exclusion and native ownership can serve the corrected boundaries. Their behavior
must remain observable without creating another policy authority or exact-name acceptance
inventory. Preserve native library-owned iteration, original-space checks, truthful outcomes,
complete relevant dependencies, Arrow validation where consumed, and ownership until actual
native/reader/cgroup drain.

The next consequential choices are the local-versus-canonical execution-role contract and the
assurance scope for genuinely Arrow-free tests. BF02's conservative applicability correction
does not require either choice. Plan 28 retains existing disposition ownership; the principal
review reconciles BF01–BF04 and their identifiers before any work is scheduled. Current
installation/attestation corrections deserve preservation, while their integrated qualification
remains separate from these source-based findings.
