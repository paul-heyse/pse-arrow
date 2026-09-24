---
title: Rust computation W15–W20 execution
status: abandoned
date: 2026-09-23
adrs: [ADR-0076, ADR-0077, ADR-0078, ADR-0079, ADR-0080, ADR-0081]
phase: 1
evidence: Tested — 530 current isolated controls and scoped repair checks; complete W19-W20 acceptance remains open
---

# W15–W20 execution

**Historical execution packet:** [Plan 14](14-library-owned-process-simulator.md)
supersedes this sequence; remaining scope is not automatically carried forward.
The former checkpoint is [W19 repairs](13-w19-repair-checkpoint.md).
W15–W17 implementation and deletion work is complete. The first W19 campaign is
unsuccessful and incomplete; repaired boundaries have isolated/static evidence.
W20 remains unrun. The entries below retain the chronological implementation history.

This packet executes [Plan 13](13-rust-computation-architecture.md), following
the [stage contracts](13-stage-contracts.md) and the W00–W14 implementation
packets. Baseline: zero failures. The inherited dirty tree and staging area are
preserved; local starting digests live in
`build/plan13/w15-w20/starting-source.json`.

## Implementation sequence

1. **W15:** replace compiler native operation orchestration with one effectful
   compiler session, immutable products, typed incremental derivation and explicit
   relational handoffs. Migrate public Rust consumers and exact selected
   publication; preserve the existing Python inspection surface. Retain original
   typed failures, stable identities and last-reader resource ownership.
2. **W16:** finish normal dependency ceilings, feature attribution and generated
   batch mechanics. Reuse generated semantic declarations and shared adapters;
   preserve wire, order, null, floating-point and canonical identity contracts.
3. **W17:** reconcile inherited acceptance cases through explicit successors,
   register Q01–Q11, and make campaign coverage and continuation plan-specific.
   Require a qualified functional receipt before performance execution. Extend
   existing test and measurement owners with independent structural references
   and isolated build characterization.
4. **W18:** close every implementation/deletion row against actual source, unit,
   static and generation receipts, then seal and preflight Plan 13.
5. **W19:** run the complete local functional campaign, repair failures and rerun
   affected gates. No missing, interrupted or failed required case is qualified.
6. **W20:** measure the complete target matrix with one isolated cold build, five
   end-to-end repetitions and Criterion sampling. Record absolute results,
   dispersion, actual feature graphs, ownership/work bounds and independent
   G1–G7 verdicts. Cold-build dispersion remains unmeasured.

## Execution boundaries

Before W18, only isolated units, static checks, compilation and pure generation
are authorized by the implementation barrier. Integrated compiler, publication,
Python component/integration, solver and performance journeys run afterward.
W20 additionally requires a matching successful current W19 receipt. Historical
Plan 10/11 outcomes remain immutable.

The public Python surface remains publication, inspection, diagnostics and
streams. New Python compile/edit/solve APIs, new physics and new solver backends
are outside this scope. Qualification targets the current Linux environment;
IDAES parity, distribution builds and other platforms remain excluded.

## Verification

**Tested:** `just doctor`, current development environment, exit 0, zero failures
against baseline zero before implementation. This establishes environment
readiness only.

**Tested:** baseline zero for every command below. Counts overlap and are not a
combined campaign total. Local logs are under `build/plan13/w15-w20/`.
`semantic-cutover-checks.json` records current family, both Clippy modes, all-target
force-validation compilation, codec and error-taxonomy receipts.

| Command | Mode and result | Limit |
|---|---|---|
| `just unit-rust-computation` | Explicit force-validation; 37 passed, zero failed | Isolated algorithms, not full compiler journeys |
| `just unit-package pse-runtime 'test(compiler::foundation_unit::) or test(compiler_driver::native::tests::native_requirements_gate_actual_arguments_and_results)'` | Explicit force-validation; 12 passed, zero failed | Simulated lifecycle and finite argument/result boundaries |
| `just unit-package pse-compiler 'test(changes::durability_unit::)'` | Explicit force-validation; 6 passed, zero failed | Includes complete stored projection comparison; no Delta IO |
| `just setup-test-report build/plan13/w15-w20/setup-report-07` | 59 isolated setup tests, zero failed | Library-owned XML reporting; source/receipt/control tests only |
| `just check-native-contracts` | Locked all-target force-validation compilation, exit 0 | Includes authored source reconstruction and stored-update journeys, without execution |
| `just clippy-default` | Locked workspace/all-target default-feature check, exit 0 | Static check only |
| `just clippy-no-default` | Locked workspace/all-target no-default check, exit 0 (`clippy-no-default-04.log`) | Static check only |
| `just typecheck` | Pyrefly configured floor, zero diagnostics; two configured suppressions | Python static typing only |
| `just family-check` | One family universe and normal dependency ceilings, exit 0 | No runtime or build-time claim |
| `just unit-typed-boundaries` | Explicit force-validation; 5 passed, zero failed in the engine-owned test binary | Isolated codecs/SQL declaration checks only |
| `just unit-consolidation-governance` | Explicit force-validation; 2 passed, zero failed | Typed error taxonomy only |
| `just list-native-contracts` | 2,831 native identities enumerated without execution | Follow-up listing must include the newly authored source-release binary |

The strict Rust/Python/docs generation comparison runs with a copied temporary Git
index containing only intent-to-add for new generated outputs. The actual index is
unchanged. Full regeneration with integrated package fixtures remains behind W18.
The compiler retains the upstream `proc-macro-error2` future-incompatibility notice;
Clippy reports no project warnings in the successful named checks.

**Implemented:** `setup-test` retains its stdlib bootstrap route. Its terminal
reporting recipe uses `unittest-xml-reporting` from the locked test group, records the
loader's exact selected identities before execution, and compares them with actual
JUnit results. The pytest 9.1.1 subtest-report probe was rejected because its declared
146 cases did not match its 58 case records; that unsuccessful report is preserved.

**Proposed:** the full W18 seal, W19 functional acceptance and W20 measurements remain
open. Proposed ADRs retain their status; local execution authorization does not
establish formal decision acceptance.

## Outcome

Implementation is in progress. No W15–W20 completion or product qualification is
claimed by this packet.

**Implemented:** the public Rust `CompilerSession` owns exact document revisions,
atomic edits/renames/replacements, finite inspection demands and immutable completed
products. Stored source admission reuses `NativeReleaseReader`, verifies every
authoritative projected relation by complete generated values, and retains the
selected member vector. Direct inputs advertise no durable reconstruction; verified
stored sources advertise exact-release reconstruction. Prior products retain their
own compiler generation and input descriptors.

The compiler driver now invokes finite algorithms explicitly over owned checked
facts. Artifact plans are constructed after those values complete and from the same
binding scope. The old public `native::plan`, producer/program/operation wrappers,
edit wrapper and plan-valued stage transport are deleted. The independent Salsa
specialization database is deleted: specialization requests use the application's
existing coordinator, CPU admission, generation limits and last-reader lease.
Document and physical-inventory caches formerly attached to native contexts are
deleted; explicit driver owners retain those immutable values. Existing native
physical caches remain owned by the engine.

**Implemented:** P10 canonicalization uses the same coordinator and generation
allowance as specialization. Its pure request contains complete mathematical,
physical, unit and prerequisite values, including exact IEEE bits and ordered root
environments. Native provenance and buffer owners stay outside Salsa. Node insertion
refuses configured capacity before mutation; cancellation before/after a finite
noncooperative kernel retains its worker and generation until actual exit.

**Implemented:** selected outputs now use `PhaseRequest`, `RelationalDriver` and the
existing native executor. The separate output-combination execution path is removed.
Equal completed demands are read before the compiler stages run; source updates
invalidate their complete phase inputs. This is whole-demand reuse, not evidence of
fine-grained P3–P10 reuse. The executor separates immutable stage arguments from the
original exact durable source witnesses, sorts declared keys and checks complete
relational obligations. Complete generated values install atomically. Product handles
retain the resulting phase generation. The shared generated codec uses existing
budgeted Arrow builders, retains empty membership/order/multiplicity and reserves
scratch for one copied row.

**Implemented:** source-expression lowering and unit normalization now live in the
pure compiler. Their former runtime implementations are deleted. P3 submits exact
source text/grammar, exhaustive path bindings, projected declaration values,
physical unit inventory, package policy and node limits to the
same bounded Salsa generation. Native source evidence and Arrow encoding remain
outside. P5 containment similarly queries complete direct-parent values through the
existing graph projection/forest library code; rich cycle/missing-parent witnesses
survive unchanged. Old result handles retain their original generation owners.

**Tested:** `just unit-package pse-compiler 'test(lowering_computation_unit::) or
test(containment_computation_unit::)'`, explicit force-validation, seven passed and
zero failed against baseline zero (`semantic-wiring-units-03.log`). These isolated
controls cover equal-request reuse, changed bindings, unrelated declarations,
missing lookup recovery, continuous-domain removal, repeated operands, reparenting,
isolates, cycle witnesses, unit-scale edits, per-handle cancellation, shared-registry
allocation accounting and capacity refusal. Full source journeys remain W19.

**Implemented:** ordinary P5/P9 and mathematical-region output contracts no longer
produce `instance_tree`, `instance_reachability` or `scope_reachability`.
`ContainmentInspection@1` is selected only when an ancestor-pair relation is requested,
after the requested semantic boundary. Diagnostics alone cannot select it. The complete
three-table row count is admitted before pair allocation; scope provenance includes the
port read even when no ports exist. The old separate `Query::Layout` and its duplicated
parent inventory are deleted; complete production containment requests are the sole
incremental forest path. The full compiler fixture now explicitly requests inspection.

**Tested:** `just unit-package pse-runtime
'test(compiler_driver::native::model::integrated_performance_unit::) or
test(inspection_computation_unit::)'`, explicit force-validation, three passed and zero
failed against baseline zero (`inspection-units-01.log`). Selection, diagnostic
nonselection, early-boundary refusal and complete expansion capacity are isolated
controls; the updated compiler journey remains unexecuted.

**Implemented:** scalar configuration parsing, child binding and original-syntax
interpretation move to `pse-compiler::configuration`; the old runtime grammar is deleted.
The boundary projects enum declarations through the existing generated row types under
a reservation. The pure implementation uses Serde's exact object-field validation,
preserves its original JSON error and retains the compact-ID/hyphenated-UUID authoring
grammar. Tagged-value, finite-number and selectable-enum checks also apply to child
bindings. The older hash-only unexpanded specialization API and identity wrappers are
deleted; actual full `SpecializationRequest` equality remains authoritative.

**Tested:** `just unit-package pse-compiler 'test(configuration_computation_unit::)'`,
explicit force-validation, seven passed and zero failed against baseline zero
(`configuration-units-04.log`). These include exact large integers, signed zero, physical
unit compatibility, changed/deprecated dictionaries, malformed alternatives, duplicate
JSON fields and original numeric syntax. Serde capability discovery used its
[container attribute documentation](https://serde.rs/container-attrs.html); the pinned
implementation is exercised by these controls.

**Implemented:** `CompilerSession::update_release` connects qualified CDF candidate
normalization and complete endpoint comparison to the same atomic source/coordinator
install. Checkpoint publication occurs afterward; a checkpoint failure remains a
separate error on a successful admission receipt. The authored stored-update journey
checks driver coherence inside the checkpoint callback, stale-update refusal and old
product retention; it remains unexecuted before W18.

**Implemented, unqualified:** the independent structural reference fixture compares
matching cardinality with SciPy and complete DM partitions/block memberships with
Pyomo. Its matrices include stored-zero edges, isolates, duplicate occurrences,
rectangular partitions and overlapping row/column identifiers. It compares invariants
rather than arbitrary maximum-matching choices. Execution remains behind W18.

**Implemented:** the obsolete Plan 10 acceptance runner and its parallel receipt
parser are deleted. The active runner resolves explicit historical owner moves,
checks actual case identities and modes, authenticates continuation ancestors and
requires a complete current W19 receipt before W20. Historical source declarations
and unsuccessful receipts remain evidence; they retain no production compiler path.
Q01–Q11 now have explicit current functional mappings. The separately registered
Q10 measurement requirement retains the complete performance campaign. These are
unqualified mappings, not executed acceptance or proof that W17 is closed.

**Implemented:** six unused direct dependencies are removed: authoring→MathIR,
Python boundary→diagnostics, testkit→IDs, and generator→diagnostics/Arrow buffer/serde.
The exact metadata/feature capture is local evidence, not a second pin declaration.
DataFusion SQL retains real rule consumers. Delta, Arrow and graph transitive default
features remain enabled by the pinned consumers; no vendor patch or family upgrade
is used to pretend those features were removed.

**Implemented:** every native containment consumer (P5, prospective port paths and
P9 selected-instance expansion) now submits complete facts to the same application
coordinator. Tear grouping lives in the pure structural layer, uses petgraph's
unweighted feedback-arc heuristic and verifies a borrowed `EdgeFiltered` graph with
iterative `toposort`. The second owned graph and runtime's direct petgraph dependency
are deleted. Complete topology requests use the same generation/key/job bounds,
cancellation and last-reader leases as the other compiler queries; costs remain
output annotations and are not a minimum-cost claim.

**Tested:** `just unit-package pse-structural 'test(tear_computation_unit::)'`, explicit
force-validation, one passed and zero failed against baseline zero
(`tear-units-01.log`), including all 512 three-vertex directed masks with parallel
occurrences, self-loops and an isolate. `just unit-package pse-compiler
'test(topology_computation_unit::)'`, same mode/baseline, two passed and zero failed
(`topology-units-01.log`), covering actual reuse, relevant edits, full typed refusals,
per-handle cancellation and atomic capacity refusal. `just unit-package pse-compiler
'test(saturated_library_lru_recomputes_equal_values_and_preserves_escaped_forests)'`,
same mode/baseline, one passed and zero failed (`lru-units-01.log`). The last control
fills 130 distinct keys before triggering the library LRU: recomputation returns an
equal new allocation while the escaped old forest stays valid and key counts remain
unchanged.

**Implemented, unqualified:** the typed compiler/edit/solver journey now composes
actual numerical/structural queries, shared runtime preparation, numeric-only attempts,
mathematical edits, stale-update rejection, fresh-compiler equivalence, immutable
result labels and returned Arrow ownership. `just native-compiler-solver-test` uses
the existing pinned solver runner and is registered behind W18 and in W19. The source
template fixture additionally authors body/interface/membership/reparent/name edits,
failed lookup recovery, full generated-value/metadata comparison and old-product
retention. These journeys have not executed. `just lint-native-data-solver` passes
against the pinned interface (`lint-native-data-solver-06.log`, zero failures against
baseline zero); `just clippy-default` passes all workspace targets
(`clippy-default-29.log`, same baseline).

**Implemented, unmeasured:** the existing Criterion owner now separates cold/equal/
reparented forest queries, unrelated release updates, saturated LRU reconstruction,
complete topology conversion, tear analysis, incidence conversion and matching/DM/
blocks. Workloads name their cardinalities and include chains, diamonds, cyclic
parallel edges, self-loops, isolates, dense incidence and overlapping partition IDs.
The new `just bench-builds <output>` remains behind W18 and a qualified current W19
receipt. It captures exact dirty source bytes/modes in an isolated Git snapshot,
performs one cold engine-test build, controlled private/public edits, a verified
validation-feature flip and the full workspace test build, and records baseline
restoration builds separately. The real source and index are unchanged. Compiler
wrappers are disabled; dependency downloads and filesystem caches remain warm.
Cargo's supported JSON artifact protocol provides actual feature/rebuild attribution;
its [HTML timing report](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/timings.md)
remains the authority for unit-duration and critical-path inspection. Cold-build
dispersion remains unmeasured. Child RSS is labelled as the campaign maximum, not
per-stage or aggregate concurrent RSS.

**Tested:** `just setup-test-report build/plan13/w15-w20/setup-report-10`, isolated
stdlib controls, 63 passed and zero failed against baseline zero. New controls cover
dirty/deleted/untracked/ignored-tracked/symlink/mode preservation, unchanged original
Git index, incomplete/failed/duplicate Cargo completion records, actual unified feature
mode and barrier refusal before snapshot or compilation. `just architecture-manifest
--plan 13` passes (`manifest-09.log`). No compiler, solver or benchmark journey ran in
those controls.

**Implemented:** instance parameter/default/override selection, feature inheritance and
flowsheet dynamic interpretation now run as complete pure configuration queries in the
existing compiler generation. Native adapters project only the selected declarations and
referenced dictionaries, preserve exact declaration-to-source correspondence, and enforce
relational identity/domain membership. Both authored roots and selected method roots use
the same path. The old native override selector and semantic value-selection branches are
deleted. Conflicting typed child bindings for flowsheet `dynamic`, previously ignored,
now fail explicitly.

**Tested:** `CARGO_INCREMENTAL=0 just unit-package pse-compiler
'test(instance_configuration_computation_unit::)'`, explicit force-validation, four
passed and zero failed against baseline zero (`instance-configuration-units-01.log`).
These controls exercise actual Salsa events, independent default/override/inheritance
answers, failed lookup recovery, exact declaration support, malformed alternatives,
per-handle cancellation and atomic capacity refusal. `CARGO_INCREMENTAL=0 just
clippy-default` passes all workspace targets (`clippy-default-32.log`). A subsequent
public identity-rename journey is authored for W19; it checks rewritten references,
stale base/name refusal, clean compilation and retained prior products.

The default native listing enumerated 2,856 executable identities without running them
(`native-identities-02.json`), before the new configuration and rename controls. Every
then-registered Rust identity appears except the two explicitly solver-feature-only
owners. Solver listing compilation failed from disk exhaustion, not test execution
(`native-solver-identities-01.log`). Package-scoped Cargo cleanup removed only this
checkout's regenerable compiler/runtime artifacts; subsequent development checks disable
incremental artifacts explicitly to limit disk growth. This does not alter force-validation
or qualify the unlisted solver cases.

**Implemented:** `CompilerSession::prepare_case` now consumes complete P10 outputs
under the source revision lock, reuses their checked column owners, and projects
physical symbols through the existing registered loader. A pure preparation query
constructs scalar equality, one-sided and range residuals, binds an explicit objective
and parameter ABI, and registers the derived case through the same coordinator.
Source replacement removes derived cases; stale source stamps and replacement of
direct mathematical inputs refuse. Numerical/structural results and prepared solver
problems retain their original generation. No second compiler database is introduced.
Unexpanded indices, runtime-dependent equation guards and discrete solver variables
remain explicit unsupported errors, rather than silently changing the selected problem.

**Tested:** `CARGO_INCREMENTAL=0 just unit-package pse-compiler
'test(case_preparation_computation_unit::) | test(computation_unit::derived_case_admission)'`,
explicit force-validation, five passed and zero failed against baseline zero
(`case-preparation-units-04.log`). Independent affine/polynomial evaluations check
residual signs, objective values and parameter changes; matching checks retain isolates.
Salsa events establish equal-request reuse and changed-source recomputation. An initial
one-million-byte test allowance refused two complete physical requests; the reuse
fixture now explicitly admits four MiB. The failed receipt remains `case-preparation-units-02.log`.

**Interface-checked:** `CARGO_INCREMENTAL=0 just lint-native-data-solver`, pinned
solver interface plus force-validation, exit zero (`lint-native-data-solver-13.log`).
The authored public source/edit/Ipopt journey compiles, compares edited and clean
preparation, retains old solves and checks result labels and last-reader ownership.
It shares the real authored scalar fixture with compiler tests. It has not executed;
W19 remains responsible for actual source and solver behavior. The manifest registers
this journey and the isolated case controls (`manifest-12.log`).

**Implemented:** P3 expression provenance no longer constructs or executes a native
query for each referenced declaration, owner or unit. It reads the already admitted
complete source batches through the shared indexed adapter. The configuration-only
adapter file is deleted; configuration and provenance share exact generated row-token
and Arrow `RowConverter` ordering mechanics plus the pure `IndexedInventory`.
Key-only support lookup does not decode unrelated row payloads. Many-source occurrence
sets remain tied to the original immutable input roles.

**Tested:** `CARGO_INCREMENTAL=0 just unit-package pse-runtime
'test(compiler_driver::passes::indexed_rows::integrated_performance_unit::)'`, explicit
force-validation, one passed and zero failed against baseline zero
(`source-index-units-01.log`). The control covers duplicate source keys, key-only reads,
incremental append, exact typed order, cancellation, and fixed chunk/decode counts
under 1/10/1000 lookups. It executes no source compiler journey. Current manifest
successors point to the shared adapter (`manifest-13.log`); historical source/module
identities remain recorded separately.

**Tested:** `CARGO_INCREMENTAL=0 just clippy-default` and `CARGO_INCREMENTAL=0 just
clippy-no-default` pass for all workspace targets after the case and shared-index
changes (`clippy-default-44.log`, `clippy-no-default-08.log`). `just family-check`
passes (`family-check-08.log`). These are static checks against baseline zero.

The follow-up native identity listing exhausted disk during linking and executed no
tests (`native-identities-03.log`). Pinned Cargo package cleanup first removed 33.6 GiB
of test-package artifacts, then 143.6 GiB of artifacts for the exact current workspace
members. Third-party dependency builds, source, the real Git index and evidence logs
were preserved. `workspace-build-clean-01.json` records the exact member set and
command. The listing retry remains a compilation/enumeration operation, not acceptance.

### Corrections

An initial output-plan refactor escaped its private binding scope. The isolated
boundary test caught this; the plan-valued wrapper was removed and completed facts
now establish one explicit output scope. Stream field checking also exposed dropped
top-level relation metadata; the boundary restores declared schema metadata only
after exact field equality, using the existing owned-batch API.

Two benchmark dependencies initially appeared unused in local scans but were used
through a shared fixture outside the benchmark directory. All-target compilation
caught that error and both edges were restored. Macro-expanded diagnostic/serde
dependencies and Python's force-validation feature edge remain intact. Dependency
audit findings are retained and are not represented as a clean audit receipt.

A formatting command initially compared plain file digests with the starting
mode-qualified source digests and selected unrelated files. The 122 unrelated paths
were restored byte-for-byte after checking their index blobs against the starting
snapshot. Two task changes included in that restoration were reapplied. The actual
Git index remained untouched; `format-recovery.json` records the comparison.

The isolated generated-codec run exposed two fixture failures because a SQL-bearing
empty relation lacked its native validation context. The fixture now uses the
existing engine-owned binding, and its test file moves to `pse-engine`; the reverse
`pse-relations` development dependency on the engine was removed. Local SQL checks
are preserved. The failed receipt remains `typed-boundary-units-01.log`.


The compiler-only unit recipe probe was refused by Cargo because its selected
package does not own `pse-relations/force-validate`. That candidate recipe was
removed; `semantic-compiler-units-01.log` retains the failed capability probe.
The existing explicitly force-validating recipe remains authoritative. DataFusion
still enters that recipe through the pre-existing relation validation/test graph;
removing the reverse engine development edge does not establish a native-free build.

The first relocated configuration unit build lacked the quantity crate's existing
`fixtures` feature. That feature now belongs to the compiler's development dependency;
the production dependency remains unchanged. `configuration-units-01.log` records the
failed build. `just py-sync` refreshed the editable extension and actual API stubs
(`py-sync-01.log`), but subsequent Rust edits invalidated uv's editable source receipt:
`doctor-02.log` still reports one environment-sync failure against baseline zero.
Refresh again after the remaining implementation edits; this is not environment closure.

### Reachable bodies and source facets

**Implemented and Tested:** specialization now projects the reachable expression
graph through the existing `GraphView` traversal, including payload references and
ordered kernel inputs. Dense projection ordinals are separate from each occurrence's
source mapping; binding restores the actual source ordinals before provenance is
emitted. Whole expression families no longer enter specialization equality. Source
lowering also excludes relation-family offsets; the owned export operation translates
only mathematical references and retains source-local predicate/equation identities.
`CARGO_INCREMENTAL=0 just unit-package pse-compiler
'test(lowering_computation_unit::) | test(specialization_computation_unit::)'`, explicit
force-validation, passed nine tests with zero failures against baseline zero
(`body-reuse-units-01.log`). The event-backed shifted-ordinal control reuses the same
body while returning the new occurrence's exact mapping and preserving the old one.
`CARGO_INCREMENTAL=0 just unit-package pse-templates
'test(specialization_projection_unit::)'`, same mode/baseline, passed one test with
zero failures (`specialization-edges-units-02.log`), including repeated roots/inputs,
payload guards, signed zero, cancellation and missing-node/binding refusals. Initial
compile and fixture-capacity failures remain in their earlier logs.

**Interface-checked, unexecuted:** the public source/edit/solve fixture now pauses at
an actual Ipopt intermediate callback through a scoped tracing subscriber, completes
the source update while the old FFI workspace remains live, then resumes and checks
the old analytic solution. It uses the existing DataFusion task propagation owner;
no global subscriber, second solver or test callback API is introduced. The solver
exposes a debug iteration event with its existing numerical diagnostics. The optional
Ipopt feature owns the pinned tracing dependency; runtime tests own tracing-subscriber.
The DataFusion tracing skill's context contract and the upstream
[Layer composition API](https://docs.rs/tracing-subscriber/latest/src/tracing_subscriber/layer/mod.rs.html)
guided the scoped observer. `CARGO_INCREMENTAL=0 just lint-native-data-solver`
(`lint-native-data-solver-15.log`), `just clippy-default` (`clippy-default-48.log`),
and `just clippy-no-default` (`clippy-no-default-09.log`, both also
`CARGO_INCREMENTAL=0`) passed with zero project warnings. `just family-check`
(`family-check-09.log`) and `just architecture-manifest --plan 13`
(`manifest-14.log`) passed. No solver journey ran.

**Interface-checked:** the ordinary and pinned-solver test listings
(`native-identities-04.json`, `native-solver-identities-01.json`) compiled successfully
without execution. Their 2,879 distinct identities resolve all 286 then-declared
native cases (`native-reconciliation-01.json`). Subsequent projection controls require
a fresh final listing; this inventory is not functional coverage or a W17 completion.

**Implemented and Tested:** P3 retains complete local lowered definitions and installs
Body/Interface/Sources plus normalized instances through the existing coordinator after
all native output obligations pass. These are derived facets of the exact admitted
source and do not change its identity. A source replacement clears them. P7 reads the
actual facets, binds source-local nodes and restores native provenance correspondence;
its inventory no longer executes identity DataFusion scans to obtain generated rows.
`CompilerSession::instance` exposes a deliberately requested, source-qualified instance
query; the authored scalar source journey now exercises it before compilation.

`CARGO_INCREMENTAL=0 just unit-package pse-compiler
'test(definition_computation_unit::)'` passed two isolated controls with zero failures
against baseline zero (`derived-definitions-units-01.log`, explicit force-validation).
They check complete grammar admission, actual Salsa events for offset-only updates,
changed-vs-fresh equality, stale/direct-input refusal and old body retention.
`CARGO_INCREMENTAL=0 just unit-package pse-runtime
'test(compiler::definition_unit::)'` passed one control with zero failures, same
mode/baseline (`derived-definitions-native-units-03.log`). It checks invalid/stale
installation, caller-drop completion, no-op revision preservation, tombstoning and the
actual last-reader reservation. A failed intermediate compile remains in receipt 02.

**Implemented and Tested:** predicate four-valued logic, exact scalar/operator/unit
semantics and lexical free-index analysis now live in `pse-compiler::predicate`.
The runtime's recursive predicate walkers are deleted. Complete generated predicate
rows retain operand order/multiplicity; petgraph `DiGraphMap`, `toposort` and
`DfsPostOrder` own cycle admission and dependency order. A shared node is evaluated
once per requested predicate, with every visited node contributing native evidence.
Native configuration/path/domain lookups and source witnesses remain driver-owned.
The staging reservation covers both generated-row copies and graph/traversal scratch
before allocation. Compiler's direct pinned petgraph dependency uses the already
qualified family; runtime does not regain that dependency.

`CARGO_INCREMENTAL=0 just unit-package pse-compiler 'test(predicate::)'` passed
six isolated tests with zero failures against baseline zero, explicit force-validation
(`predicate-core-units-01.log`). These include a 10,000-node repeated-child graph,
all four-valued conjunction/disjunction/negation pairs, leaf errors, complete-source
cycle/missing/duplicate/mixed-source refusals, cancellation, exact integer division
and shared lexical binders. The two inherited arithmetic identities now point to
their real pure owner in the Plan 13 manifest; historical declarations remain intact.

### Consumer and representation audit

| Responsibility | Current owner and boundary |
|---|---|
| Document edits, rename and exact stored replacement | `compiler_driver::CompilerSession`; one source lock and owned coordinator update; complete source admission precedes derivation |
| Source lowering and parameter/default/child-value semantics | Pure `lowering` and `configuration` queries; native adapters project complete values and attach source evidence |
| Definition and instance inspection | Installed generated facets; leased Body/Interface/Sources/Instance queries retain the exact source generation |
| Dynamic instance expansion | Runtime bounded work queue composes actual configuration queries and native domain products; typed template identities and the active expansion chain reject recursion before child creation |
| Rule inference | Derived library dependency schedule; existing bounded DataFusion fixed-point executor retains complete support/conflict/negative dependencies |
| Predicate and graph semantics | Pure predicate, containment, topology and structural owners; native drivers bind data/evidence and materialize requested outputs |
| Mathematical realization/expansion/canonicalization | One typed MathIR region; shared structural specialization and canonicalization queries; original provenance outside body equality |
| Numerical/structural preparation and attempts | Source-derived complete case, exact numerical/structural queries, shared runtime preparation and fresh mutable solve attempts |
| Publication and Python | Selected complete artifact profiles and catalog protocol; existing Python inspection, diagnostics and streams; no new Python compile API is claimed |

A finite driver loop that constructs real occurrence outputs or attaches native row
witnesses is retained for that responsibility. The displaced producer/program/plan
wrappers, independent specialization database, hash-only specialization API, duplicate
layout query and identity-scan inventory paths have no production successor facade.
Whole-demand reuse and the specific pure query boundaries above are implemented;
this does not claim automatic fine-grained reuse of every driver substep.

### Current static and dependency evidence

**Tested:** with `CARGO_INCREMENTAL=0`, `just clippy-default`, `just clippy-no-default`
and `just lint-native-data-solver` pass against baseline zero, with zero project warnings
(`clippy-default-58.log`, `clippy-no-default-10.log`, `lint-native-data-solver-16.log`).
The last command compiles the Ipopt and force-validation consumers without running
solves. `just family-check` passes (`family-check-10.log`); `just architecture-manifest
--plan 13` passes (`manifest-15.log`). `just metadata-resolve` captures the exact current
normal edges and unified features in `metadata-resolve-04.json`.

The resolved Delta consumer enables DataFusion defaults and the kernel's Arrow,
Parquet and cloud storage features. Rustworkx enables petgraph defaults. Direct
feature flags cannot remove these transitive activations; the qualified Delta
pin/feature-update trigger remains the owner of any kernel/TLS change. Actual SQL
rule consumers and supported native APIs remain enabled.

**Implemented source-size observation:** against Git commit
`a46f358bdfc2ca27f9f240ab6c045b63141c3ee9`, generated native relation Rust decreases
from 474,305 to 417,154 lines. The new pure model declarations contain 126,966 lines,
so their combined total grows to 544,120 lines. Combined bytes decrease from 29,000,623
to 21,787,661 (`generated-size-01.json`). This is the complete Plan 13 tree delta,
including new contracts and formatting, not a mechanics-only or build-time gain.
Shared generator/codec ownership and normal dependency isolation are the architectural
changes; W20 must establish their actual build/runtime cost.

Blueprint revision 49 reconciles stale universal DataFusion and Plan 11 authority
with the existing proposed Plan 13 decisions. It uses the explicit design-edit scope
for this authorized hard pivot. Accepted ADRs remain unchanged and formal decision/design
PR acceptance remains open.

### Remaining qualification boundary

W00-W17 and all deletion rows are closed at the implementation boundary. The current
unit, listing, pure-generation and static evidence below supports W18 sealing. The
actual source, edit/rename, stored reconstruction/update, engineering, publication,
stream and live-FFI journeys remain unqualified until W19. W20 measurements and G1-G7
verdicts follow complete current functional acceptance.


**Tested, implementation boundary only:** W15/W16 are closed in the execution inventory.
`just codegen-rust-contracts-check`, `just codegen-python-check` and
`just codegen-docs-check` pass with zero failures against baseline zero
(`codegen-checks-04.json`), using the copied temporary index described above; the real
index is unchanged. `CARGO_INCREMENTAL=0 just unit-typed-boundaries --profile ci`
passes five tests with explicit force-validation and zero failures
(`typed-boundary-units-03.log`). `just py-sync` completes (`py-sync-04.log`) and
`just doctor` reports environment ready (`doctor-03.log`). Fresh ordinary and pinned
solver listings resolve all 290 native functional mappings across 2,888 distinct test
identities (`native-reconciliation-03.json`). These are compilation/enumeration facts,
not executed product acceptance. The initial reconciliation included the Criterion
case in a nextest inventory; its explicit mismatch remains in receipt 02. Criterion
coverage retains its separate performance owner.

### W17 fixture and gate reconciliation

**Interface-checked:** `just assessment-list --plan 13 --phase performance` lists
77 measurement gates in `performance-scope-01.json`; this command executes no
measurements. The supported owners cover the complete matrix as follows:

| Measurement | Executable owner and observations |
|---|---|
| Cold/private/public/feature/workspace builds | `bench-builds`; isolated source and target, actual Cargo artifacts/features, wall/CPU and retained native Cargo timing graphs |
| Admission, reconstruction and semantic edits | `native_consolidation/semantic.rs`; separate complete admission, fresh database, unchanged queries, body/interface/unrelated edits and negative lookup recovery; full fresh-result comparisons and retained extents |
| Repeated specializations | `native_consolidation/specialization.rs`; separate reachable request projection and cold/warm query plus real occurrence binding for 1/100/1000 instances; control where changed consumed values produce a different body and complete fresh-graph equality |
| Complete graph regions and structure | `native_consolidation/computation.rs`; chains, diamonds, cycles, parallel edges, isolates, dense incidence, projection, tear groups, matching/DM/blocks and saturated LRU reconstruction |
| Relational work and exact storage | `native_cache/{preparation,strata,delta,cdf}.rs`, `bench-rules` and `bench-recovery`; setup/execute phases, work/IO counts, exact-version/CDF/recovery controls and retained/peak resources |
| Numerical preparation and solves | `native_consolidation/numerical.rs`, `bench-solver` and four engineering recipes; preparation, scalar residual/derivative, batch shape and supported native solve controls |
| Ownership and observation | `native_consolidation/{ownership,observation}.rs`, cache pressure and engineering Python fixtures; copy/owner extent, actual reader progress, last-reader release, requested evidence and observation modes |
| Python and publication consumers | `test_engineering_inspection.py` and `engineering_inspection.rs`; subprocess cold import, exact reopen, streams/structured rows, two readers, retained arrays and process/pool peaks |

Criterion supplies repeated samples for its groups. End-to-end solver, rules, recovery
and engineering recipes run five repetitions in each named validation mode. Cold build
runs once and retains an explicit unmeasured dispersion. Reported extents and public
query counts are distinguished from allocator RSS and actual Salsa query executions.
No benchmark result or improvement is claimed before W20.

**Tested, failed receipt retained:** `development-units-01` completed all selected
implementation-unit groups without changing its captured executable source. Against
baseline zero, two rule query-binding controls and one native output fixture failed
because raw admission reached a registry-local SQL planner before engine installation.
All other selected groups passed, including 63 setup tests and 107 Python unit tests.
These receipts are not current acceptance and are not added to another campaign count.

**Implemented:** raw engine candidates and rule workspaces now admit through the actual
immutable engine function/configuration owner, using the existing native validation
adapter and reserved Arrow copying. A two-function-owner unit detects accidental use
of registry defaults; cancellation remains explicit. The standalone generated-output
fixture installs its native registry binding before construction. `just clippy-default`
passes with zero project warnings (`clippy-default-63.log`); `just family-check` passes
(`family-check-11.log`). A fresh isolated sweep follows these changes.

**Tested:** the fresh static preflight passes TOML, Python format/lint, import boundaries
and agent configuration. Its first type check retained one build-report inference error
(`typecheck-w17-01.log`); reading the validated Cargo artifacts before merging them
into the mixed result record fixes it (`typecheck-w17-02.log`, zero diagnostics against
baseline zero, two configured suppressions). The Rust format check identified exactly
two formatting differences in owned compiler source/tests; the targeted formatter
changes do not alter behavior.

`development-units-02` is intentionally interrupted and incomplete. It preserves its
actual passed and not-run identities. Its per-package build loop was replaced for the
fresh run by `just unit-libraries <explicit-filter> --profile ci`: the same declared
isolated native unit selectors now share one workspace feature graph. Separate typed
boundary, linked callback, setup and Python unit recipes retain their own modes and
reports. No compiler journey, storage journey or solver invocation is admitted by this
selected filter. `development-units-03` captures fresh source before this run.

**Tested, failed receipt retained:** `development-units-03` ran all six selected
unit groups on unchanged captured source. The original three admission failures passed.
The native selection had 346 passed and one failure: the new custom-function regression
fixture omitted its derived relation granularity, so registry construction correctly
refused it. The recipe, typed codec and linked-callback groups passed; setup had
63 passed. Python had 106 passed and one setup error because the benchmark dependency
addition changed Cargo.lock after the last editable-extension refresh. These are fixture
and environment repairs, not weakened semantic or validation controls.


**Tested, current implementation receipt:** `development-units-04` completes six
recipe-owned groups with zero failures against baseline zero: `unit-libraries`
(347), `unit-consolidation-tools` (two), `unit-typed-boundaries` (five),
`unit-native-callbacks-solver` (one), `setup-test-report` (63), and `py-unit` (107).
Native runs use explicit force-validation and the ci profile; the callback also
uses the pinned Ipopt feature/runner. These are 525 distinct reported controls in
this run, of which 460 binary/test/mode identities witness all 149 declared unit cases.
The source capture remains unchanged; XML and native/Python enumeration agree.
The original three failures and both later fixture/environment failures are repaired
and retained in their original receipts.

`build/plan13/development-checks.json` is exported from those actual results and passes
`validate_development`; its old W00-W06-only predecessor is retained in the run folder.
`codegen-checks-05.json` records zero failures for the three pure generation recipes
and byte-for-byte preservation of the real Git index. `just doctor` and `just py-sync`
pass (`doctor-04.log`, `py-sync-05.log`). `just fmt-rust-check` and `just typecheck`
pass (`fmt-rust-check-04.log`, `typecheck-w17-02.log`). `native-reconciliation-04.json`
resolves all 290 native functional mappings across 2,889 listed identities. These
receipts close W17/L16; they establish no product or performance acceptance.


**Tested, W18:** `just architecture-seal --plan 13` succeeds, zero failures against
baseline zero, with all implementation/deletion exits closed. The existing tooling
checks current source routes, passing development identities, feature modes, logs and
source digests and captures the full working source/lock/overlay. The status update
is sealed again before preflight. W19 is now authorized; W20 still requires complete
current W19 evidence. Formal ADR/design PR acceptance is not claimed.


**Tested, unsuccessful first W19 attempt:** `CARGO_INCREMENTAL=0 just assessment
build/plan13/w19-functional-01 --plan 13` retains unchanged source throughout its
executed gates. The force-validation ci-profile native gate reports 2,757 passed,
119 failed, zero skipped and zero not run against baseline zero. The separate
solver gate reports four passed and eight failed; compiler/solver reports one
passed and one failed. These overlapping results are not summed. Required static
and generation failures also remain in the receipt. The release enumeration was
intentionally interrupted before the remaining expensive gates to repair confirmed
shared causes; those gates remain interrupted or not_run, never passed. W20 is unrun.

**Implemented, awaiting renewed qualification:** native authoring installs its
standard SQL validation owner at the effectful document boundary. Pure schema
construction remains independent. SQL-bearing generated boundary controls move to
an engine-owned binary. Shared object-store streams use Futures `StreamExt::fuse`
and retain explicit permit ownership through body consumption; repeated EOF and
closed-gate controls cover nested wrappers. The descriptor output uses the existing
checked relation declaration before durable encoding. Standalone solver fixtures
retain bounded CPU admission. Rule scheduling preserves typed graph errors and
cycle witnesses; tests now observe derived settlement strata and schedule provenance.
Generator assertions follow semantic declarations in `pse-model` and shared native
codecs in `pse-relations`; obsolete per-relation builder assertions are removed.

The already specified ADR-0075 runtime tear result contracts are registered with
actual keys and foreign keys. Their `TearMethod` dictionary describes runtime
selection, while `TearHeuristic` names the existing pure compiler graph heuristic.
This closes a declaration omission exposed by Appendix B governance; it implements
no Pyomo selection backend and does not claim ADR-0075/R-32 execution acceptance.
The `compiled.rule_strata` snapshot class is corrected to Derived. All generated
Rust, Python, documentation and conformance outputs are regenerated through recipes.
The prior development receipt and W18 seal are historical until refreshed after
these repairs; required functional acceptance remains open.

**Tested, repair checkpoint:** the subsequent `development-units-07` run completes
530 isolated controls with zero failures against baseline zero: 348 selected library
units, two tool units, nine typed-boundary units, one linked callback, 63 setup tests
and 107 Python units. Native groups use the ci profile and explicit force-validation;
the linked callback uses the pinned Ipopt runner without invoking a solve. Captured
executable source is unchanged. The refreshed development export maps 461 distinct
binary/test/mode identities to all 149 required unit case IDs. Separate generator and
registry-admission binaries each pass 16 controls in the same native correctness mode.
All-target compilation, default-feature Clippy, family checks, strict generation and
conformance fixture comparison pass under the conditions recorded in the
[current resume checkpoint](13-w19-repair-checkpoint.md). These results supersede
receipt 04 for current development evidence; earlier results remain historical.

The maintainer requested the documentation pivot after this repair batch. Plans,
stage contracts and review status pointers are updated before refreshing the W18 seal.
W19 has not been restarted and W20 has not run. The checkpoint lists the remaining
functional journeys, complete campaign gates, measurement work and formal decision
acceptance without awarding any of them from isolated tests.
