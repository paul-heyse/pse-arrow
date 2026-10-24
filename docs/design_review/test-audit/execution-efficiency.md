# Efficiency-oriented design principles in the test system

This is the current primary home of DTA-018 and DTA-029–DTA-048. The holistic follow-up
revalidated their composed consumers at source `05e6a2b8a`, 2026-10-23, against AP-07/G9,
relevant operational refinements and efficiency heuristics 1.0. Diagnoses are *Interface-checked*.
No tests, benchmarks, profiling, timing claims or fresh host-state inventories were produced.

The [integrated architectural account](architecture-and-assurance.md) explains interactions
among assurance scope, source dependencies, physical preparation, scheduling, artifact ownership
and recovery. The [collection README](README.md) owns the baseline, scope and dispositions.

## What the follow-up changes

Necessary scope comes from the consumed contract. A database's installed schema, a child process's
loaded artifact and a sealed evidence origin can establish distinct premises. Those checks cannot
be removed merely because something else was verified earlier. Conversely, carrying the whole
selection through every mutable membership publication, or constructing a durable fixture for a
capability-only assertion, is a concrete mismatch between responsibility and physical work.

Scheduling and budgets also interact: the Python serial group limits simultaneous receiver
demand, while a broad scientific regex demonstrably catches some pure controls. These observations
support different judgments. Neither universal serialization nor universal parallelization follows.

DTA-041's generic overengineering verdict is withdrawn. DTA-044 is unresolved. DTA-040 no longer
asserts false failure of historical observations, and DTA-047 preserves report-seal authentication.
Other findings below retain a narrower demonstrated cause where the previous scope exceeded it.
Historical timings/counts remain recoverable in Git with their original conditions; this page
does not freshly endorse them or present source inspection as a measured speedup.

## <a id="dta-018"></a>DTA-018 — A canonical fixture reconstructs a complete integration lifecycle per test

**Principles:** AP-06, AP-07, DP-10, DP-19; H2, H12; companion §6, §8.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `crates/pse-operations/src/testing.rs::{resource_bridge,canonical_fixture_with_profile}`, `python/pse/tests/conftest.py::canonical_substrate`, and `scripts/test_resources.py::{register,finalize}`.

Rust resource_bridge and canonical_fixture_with_profile, Python canonical_substrate,
test_resources registration/drain and canonical initialize/open compose an independent database,
schema admission, context and cleanup lifecycle per fixture. Registration starts one bridge;
registration plus drain supplies the pair. This is a substantial preparation boundary even when
the final assertion uses a small capability. DTA-031 identifies a concrete unnecessarily broad
consumer; the lifecycle is legitimate where fresh durable behavior is the claim.

**Corrected premises.** Schema installation, issuing-store identity and module validation are
per database. Service configuration can change between tests; mutable source/artifact paths
remain uncertain until captured. They are not all established invocation-wide invariants.
The earlier assertion that these checks merely repeat an already-secured premise was too broad.
Any smaller realization must preserve fresh-install controls, exact identity, independent
connections, registration before effects and actual drain; no cached template or shared mutable
database is established as equivalent by this audit.

## <a id="dta-029"></a>DTA-029 — Store-test selection is coupled to a feature-dependent production-library variant

**Principles:** AP-02, AP-05, AP-07, DP-10; H13; companion §5, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `crates/pse-operations/src/canonical_artifacts.rs::CANCEL_AFTER_ARTIFACT_APPEND_ACK`, `crates/pse-operations/src/canonical.rs::operation_failed`, and the operations/runtime Cargo manifests.

The canonical-tests feature deliberately prevents ordinary unit selection from needing a server.
It also enables library behavior: canonical_artifacts' post-ack cancellation suspension and
canonical operation_failed logging compile outside cfg(test). Cross-crate controls therefore
exercise a particular feature variant while test selection influences the production build closure.

The hook permits a real cancellation interleaving unavailable through a trivial fake. Feature
existence is not itself an invalid seam or proof of different unprobed scientific behavior.
Keep the selection/build-variant coupling and identify the exercised variant; do not presume
moving everything into another crate or removing deterministic controls preserves assurance.

## <a id="dta-030"></a>DTA-030 — Shared reference preparation repeats a union closure for differently scoped consumers

**Principles:** AP-01, AP-06, AP-07, DP-10; H5, H12, H13; companion §2, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `tests/fixtures/reference-source-fixture.rs`, `crates/pse-compiler/src/workspace/modeling.rs::publish_modeling_with_validation`, and `tests/support/plan14.rs`.

The shared reference-source fixture reparses its explicit union in fresh compiler workspaces,
whose publish_modeling_with_validation checks each candidate again. High-level plan14 source
preparation also loads a broad runtime package closure before individual selected cases. A
dependency/admission failure outside the case's intended assertion can prevent that assertion.
This couples focused feedback and reference dependency changes across consumers.

A smaller semantically sufficient closure was not established. The indexed-process composition
test genuinely exercises inventory, derivatives, connections and observation closure, not merely
arbitrary counts. Narrow output does not prove narrow necessary input. Preserve authentic source,
imports and case-specific independent assertions; quantify no preparation share. Manual closure
maintenance and embedded-source invalidation remain separately explained by DTA-027/035/036.

## <a id="dta-031"></a>DTA-031 — Capability-only Python tests acquire a durable fixture through a broad runtime dependency

**Principles:** AP-01, AP-06, AP-07; H2, H5; companion §2, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `python/pse/tests/test_native_workflow.py::{runtime,test_native_capability_discovery_and_hard_cut}` and `crates/pse-py/src/workflow.rs` local runtime construction.

`python/pse/tests/test_native_workflow.py::test_native_capability_discovery_and_hard_cut` reads
capabilities and checks the exported surface. Its runtime fixture depends on canonical_substrate,
so the case also creates/registers a durable fixture, its resource context and scheduling group.
Python has a local runtime construction using Runtime::from_shared_local. The asserted capability
responsibility itself does not consume the store, establishing unnecessary dependency activation
for this concrete consumer.

The previous Rust footprint was too broad. Runtime::from_shared selects Durable, and
modeling::canonical::admit_revision_authorized really records through store.edit. Such tests
exercise recording as constructed, even where another test could choose a different assurance
scope. Equivalent local coverage was not established for those paths. Changing parallelism or
sharing databases first would not remove the Python case's accidental preparation dependency.

## <a id="dta-032"></a>DTA-032 — Overlapping testing/tooling routes prepare different feature closures

**Principles:** AP-07, DP-10, DP-16; H12, H13; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/python_extension.py::{sync,develop_command}` and `Cargo.toml` / consumer Cargo feature declarations.

Dev-unit fixtures/test-support and validated versus unvalidated Arrow closures produce distinct
builds. In particular python_extension.sync invokes validated maturin followed by xtask without
the force-validation feature. Thus one correctness route reaches overlapping but different
dependency closures. Foundational edits can require preparing more than one variant.

This does not contradict ADR-0122's package-selection unification guarantee, and not every
workspace library is necessarily in every closure. Cargo retains variants; their existence is
not repeated full recompilation on every run. Interactive unvalidated builds and private-access
support have actual purposes. Any consolidation must retain force-validation for correctness,
the selected interactive contract and appropriate separation of test hooks from shipped builds.

## <a id="dta-033"></a>DTA-033 — Focused Python routes unconditionally invoke a broad extension-preparation pipeline

**Principles:** AP-07, DP-10, DP-16; H12, H14, H18; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/python_extension.py::sync`, `scripts/python_tests.py::main`, and `python/pse/tests/conftest.py::pytest_sessionstart`.

The py-unit/py-test recipes always invoke python_extension.sync: uv preparation, maturin develop
and stub generation. Separate downstream runner capture and pytest_sessionstart read full
extension digests; workers
also verify their loaded extension. Thus a narrow Python-only selection traverses this pipeline
even when its test body and compiled sources are unchanged.

Unconditional invocation is not proof of Rust recompilation: Cargo may report fresh artifacts,
uv may do little, and stubs write only changes. Maturin rewrite behavior, exact current extension
size and elapsed cost were not revalidated. Freshness, compiled validation features and protection
against concurrent replacement remain required; a stat-only or cached-identity shortcut has not
been established as equivalent.

## <a id="dta-034"></a>DTA-034 — The unprepared fallback repeats Rust resolution and enumeration

**Principles:** AP-07, DP-16; H4, H12, H19; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/select.py::main`, `scripts/test_run.py::run_rust`, and `scripts/native_tests.py::{ordinary_rust_capture,_rust_capture}`.

Without PSE_NATIVE_OPERATION, select.main resolves the closure and run_rust lists before normal
nextest execution, which can repeat metadata/build fingerprint and enumeration work. That fallback
contains a repeated preparation opportunity.

The earlier universal ordinary-managed-route claim was wrong: select skips this initial resolve
when the operation marker exists. observer_rust → ordinary_rust_capture → _rust_capture already
retains metadata and binary inventory for ordinary selections, not only special native campaigns.
Preserve validated graph/root correspondence and the exact pre-execution catalog. The fallback
frequency and elapsed impact are unmeasured, and the managed route must not be described as lacking
reuse that it already performs.

## <a id="dta-035"></a>DTA-035 — Embedding shared reference documents widens compile-time change propagation

**Principles:** AP-01, AP-07, DP-10; H13; companion §2, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `tests/fixtures/reference-source-fixture.rs` and its compiler/runtime/conformance test consumers.

The include_str inputs of tests/fixtures/reference-source-fixture.rs make reference science
documents compiler inputs to its consumer test binaries. A data/declaration-only edit therefore
invalidates those compiled consumers in addition to changing the scientific case. Multiple
test consumers share that coupling, while affected-test preview does not infer these edges.

Embedding also preserves exact bytes, missing-file errors and executable provenance. Runtime
loading would require equivalent source identity and freshness; it is not automatically superior.
Prior counts, binary sizes and rebuild timing were not refreshed. The qualitative input coupling
is established without claiming every consumer rebuild is avoidable.

## <a id="dta-036"></a>DTA-036 — Package-based affected-test preview does not represent scientific fixture dependencies

**Principles:** AP-01, AP-05, AP-07, DP-10; H13; companion §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/affected.py::{owner,closure,main}` at the fixed audited source revision.

affected.owner maps paths to Cargo package roots and closure follows manifest reverse edges,
including dev dependencies. Shared fixture/reference paths outside those roots are unowned;
test-only edits also propagate through package dependencies more broadly than actual test reach.
A reference-only change receives no Rust selection even though embedded or runtime-loaded
scientific inputs have test consumers. This makes the developer combine different dependency
models manually when selecting checks for a scientific change.

The tool explicitly reports dynamic exclusions and calls itself a dependency preview, not an
exhaustive behavior guarantee. This is a bounded capability/change-locality shortfall in the whole
feedback workflow, not a false report of complete coverage. Depfiles alone would not settle
arbitrary runtime-loaded inputs either.

## <a id="dta-037"></a>DTA-037 — The grouped codegen comparisons repeat overlapping generation work

**Principles:** AP-07, DP-10, DP-16; H6, H12; companion §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `xtask/src/codegen.rs::append_physical` and generation gate selection in `scripts/validation_scope.py`.

The validation generation group selects relations, physical and rust-contracts comparisons.
In xtask/codegen, Rust relation generation already includes physical projection, rust-contracts
narrows Rust roots, and physical calls append_physical separately. The assembled group therefore
performs overlapping comparison work.

Each gate remains useful independently, and regeneration is necessary to establish drift.
Bootstrap's mutating phases serve another purpose. The concern is composition of those gates,
not eliminating fresh generation checks or their separately attributable failures. No timing
or speedup is claimed.

## <a id="dta-038"></a>DTA-038 — Fixture registration rewrites selection-bearing invocation state under shared admission

**Principles:** AP-07, DP-10, DP-19, DP-20; H8, H15, H23; companion §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/resource_store.py::FileStore.{register,invocation_edit}`, `scripts/test_resources.py::register`, and `scripts/surreal_server.py::context_admission`.

FileStore.register publishes whole invocation state for pending and uncertain transitions;
invocation_edit publishes completion. The state includes the immutable selected declaration,
so each membership update repeatedly serializes unrelated selection data. test_resources.register
holds context_admission and invocation/identity/resource exclusion across those publications.
Registration work consequently grows with selection size as well as resource count, and users
of the same service share the admission serialization boundary.

Crash-safe pending publication, exact identity, terminal freeze and lifecycle visibility of
accepted borrowers are concrete obligations. They do not establish the necessity of repeatedly
rewriting an immutable selection, nor does this finding establish that locks or durable transitions
can simply disappear. Queue timeout frequency and runtime share remain unmeasured.

## <a id="dta-039"></a>DTA-039 — Broad source capture and repeated artifact verification have different validity scopes

**Principles:** AP-07, DP-09, DP-10; H5, H18, H19; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/native_tests.py::{native_provenance,rust_source_identity,prepared_rust_inventory}` and `scripts/validation.py::sources`.

native_provenance, fixture bindings and prepared_rust_inventory repeatedly verify full-byte
artifacts/source. Parent preparation, admission waits, separately launched children and retained
reuse do cross mutable-path replacement windows, so these are not all the same already-secured
premise. Hardlinks or file metadata alone do not freeze bytes.

A sharper source-visible excess is rust_source_identity: it obtains validation.sources over the
broad source inventory before projecting rust-product. This does unnecessary pre-projection work
for that narrower identity. Repetition elsewhere is a scoped opportunity requiring ownership and
immutability analysis, not an established claim that most checks are redundant. Generation
publication and verify_generation already carry/reuse some verified premises within their owner.
No hashing time or equivalent shortcut was established.

## <a id="dta-040"></a>DTA-040 — Aggregate snapshot qualification has a wider invalidation window than an individual observation

**Principles:** AP-04, AP-05, DP-09, DP-22; companion §6, §8.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/validation.py::run_gates` and `scripts/validation_receipts.py::{classify,reuse}`.

validation.run_gates compares relevant inputs with the assessment's single starting snapshot
after the gates. A later edit can make an earlier observation inapplicable to that aggregate
snapshot claim. A later edit does not erase the historical execution, but its exact input
attribution requires evidence beyond the aggregate snapshots. `scripts/validation_receipts.py::classify`
also verifies immediately after each gate, not solely at the assessment end.

The former false-failure diagnosis and implication that all work must be rerun are withdrawn.
This is conservative qualification semantics; validation_receipts already supports scoped reuse
and transfer. Whether the aggregate window is disproportionately broad for a particular intended
claim remains unresolved. A different result boundary must state what final-source assurance it
preserves, rather than silently treating historical success as current qualification.

## <a id="dta-041"></a>DTA-041 — Disposability alone does not establish that fixture recovery is overengineered

**Principles:** AP-07, DP-19, DP-20; H25; companion §6, §8.

**Follow-up disposition:** Withdrawn, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/resource_store.py::FileStore.{register,_settle_members,ensure}`.

Withdraw the established overengineering verdict. Disposable passed databases coexist with
failed/unknown retention, durable association, late effects and cross-process server borrowers.
FileStore.register and _settle_members explicitly handle pending/uncertain recovery; ensure checks
a durable legacy-writer fence. UUID naming does not remove duplicate-registration or stale-writer
exposure. One current worktree does not establish that the compatibility obligations have vanished.

The earlier statement that nobody needs this state after a crash contradicted the retained
diagnostic/safety contract. No simpler realization satisfying that complete contract was established.
Concrete representation amplification remains in DTA-038 and failure/lifecycle coupling in
DTA-042; neither requires dismissing recovery machinery wholesale. This identity is retained for
lineage, not recorded as an implementation repair or permission to delete uncertain resources.

## <a id="dta-042"></a>DTA-042 — Coarse failure retention and cleanup joins amplify later lifecycle work

**Principles:** AP-07, DP-19, DP-20; H10, H24; companion §6, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/test_run.py::execute_selected`, `scripts/test_resources.py::{finalize,reclaim_reports,resource_status}`, and `scripts/surreal_server.py::require_borrowers_drained`.

An incomplete/untrusted enclosing report keeps invocation members pinned, even when individual
child outcomes look successful. execute_selected's deferred cleanup loop stops after its first
exception. Bounded report sweeps coexist with an active-registry read on resource_status and
require_borrowers_drained. An infrastructure failure can thus enlarge retained state and later
service lifecycle inspection beyond the test body that triggered it.

Conservative retention is purposeful when outcome association is unauthenticated. Explicit
abandonment/release exists; automatic disposal of unknown state is not a required convergence
inference. The earlier effectively-monotonic claim and memory-pressure attribution from counts
alone are removed. Keep the demonstrated failure amplification and scan coupling, preserving
owner/liveness/drain/reference safety and the distinction between retention and live memory demand.

## <a id="dta-043"></a>DTA-043 — Per-run test exclusion does not by itself exclude another run sharing the service

**Principles:** AP-05, AP-07, DP-20; H21, H23; companion §6, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `.config/agent-capacity.toml`, `.config/nextest.toml`, and `scripts/surreal_server.py::execution_resources`.

Host resource classes, nextest groups and service limits have different scopes. In the selected
unbounded build mode, two runs can share one server while each independently grants a test
exclusive nextest width. That per-run property does not establish service-wide quiet for a
test whose actual assertion requires it.

The server has real runtime/session/memory limits, and bounded host classes can coordinate
exclusivity. It is false to say no owner coordinates any capacity. Specific cross-run interference
and failure attribution remain unobserved. Preserve the maintainer's unbounded mode, explicit
overload outcomes and lifecycle fencing; align the asserted requirement with its real exclusion
scope rather than imposing a global serial policy.

## <a id="dta-044"></a>DTA-044 — The necessity of the Python canonical-owner barrier remains unestablished

**Principles:** AP-07, DP-20; H22; companion §6, §8.

**Follow-up disposition:** Unresolved, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `conftest.py::pytest_collection_modifyitems`, `scripts/native_tests.py::python_command`, and `python/pse/tests/conftest.py::canonical_substrate`.

Root conftest groups every canonical_substrate fixture closure, and ordinary python_command
uses loadgroup, so canonical cases are serialized within that run. Distinct databases establish
data isolation, not independence of receiver capacity, service timing or resource-heavy bodies.
The barrier also mitigates simultaneous demand from the large per-runtime ceilings in DTA-046.

The previous confirmed unnecessary-serialization verdict was not supported by absence of a
recorded motivation alone. No specific safely independent canonical body whose barrier is
dispensable was established in this pass. Keep the observation and dependency question visible;
do not remove the group or claim a parallel speedup on this evidence.

## <a id="dta-045"></a>DTA-045 — A broad scientific-group regex serializes demonstrably pure controls

**Principles:** AP-01, AP-07, DP-20; H22; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `.config/nextest.toml` scientific-group rules and pure controls in `crates/pse-backend-native/src/{kinsol,ipopt}.rs`.

The nextest module regex assigns both scientific slots to pure KINSOL profile-identity controls
and Ipopt status_scope_and_finite_infinity_are_not_conflated. These local parsing/mapping/bounds
assertions acquire no solver pool; module location gives them an unnecessary scheduling barrier.

This does not establish independence of all solver unit tests. Other cases own real native state,
memory or threads; Clarabel controls explicitly select threaded MKL settings. Preserve those
obligations and memory-heavy journey isolation. The actionable scope is the concrete pure controls
and requirement assignment, not blanket parallelization of hundreds of native tests.

## <a id="dta-046"></a>DTA-046 — Per-runtime ceilings do not establish a safe aggregate concurrent-demand envelope

**Principles:** AP-07, DP-20; H17, H21; companion §6, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `.config/agent-capacity.toml`, `scripts/surreal_server.py::execution_resources`, and `python/pse/tests/test_native_workflow.py::runtime`.

Independent Python runtime owners can each configure a 64-GiB pool; a ceiling is not an aggregate
reservation. Their allowed simultaneous demand must be understood with observer/receiver/context
lifetimes, rather than multiplying one body in isolation. A potential mismatch remains for
simultaneously active consumers, not an observed OOM or unavoidable failure.

Correct the earlier arithmetic: exclusive-observer has a 144-GiB observer/receiver/control role
envelope (64/72/8 GiB), not a 64-GiB complete service allocation. Functional-store and host-light
costs are separate. canonical-owner prevents concurrent Python canonical fixtures
within an ordinary run, and many receiver descriptors do not start heavy work. No source proof
established that all ceilings become live together or cannot fail explicitly before host failure.
Assess scheduling and budgets jointly while preserving the selected operating mode.

## <a id="dta-047"></a>DTA-047 — Current nondisposable reports still traverse generic reclamation machinery

**Principles:** AP-01, AP-07, DP-16, DP-19; H26, H28; companion §8.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/test_resources.py::{reference_report,eligible}`, `scripts/validation.py::artifact_roles`, and `scripts/validation_receipts.py::reuse`.

Current validation producers declare no scratch role, so normal report-reclaim eligibility
cannot delete their protected receipt/provenance/evidence artifacts. Generic reclaim inspection
therefore operates on reports that are not disposable by that route, creating a bounded lifecycle
scope question.

Registration and sealing also authenticate reuse: reference_report requires a registered sealed
origin, verifies its receipt digest and manifest, and validation_receipts.reuse consumes it.
The earlier claim that protection guards nothing was false; dropping registration/sealing would
discard a consumed guarantee independently of cleanup. Keep the narrower reclaim-path concern,
with its material cost unresolved, and preserve original evidence custody and immutable references.

## <a id="dta-048"></a>DTA-048 — Checkpoint serialization repeats growing evidence maps and failure payloads

**Principles:** AP-07, DP-10, DP-16; H16, H26; companion §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

**Source anchors:** `scripts/validation.py::checkpoint`, `scripts/native_tests.py::prepared_rust_inventory`, and `scripts/validation_scope.py::input_identity`.

validation.checkpoint rewrites the accumulated receipt; per-gate input identities and copied
failure detail increase that payload as a run proceeds. These repeated full serializations create
qualitative amplification with gate count and retained detail. The exact current byte counts and
timing were not refreshed.

Native Cargo metadata and unit graphs are not merely opaque equality strings: prepared_rust_inventory
parses package IDs, roots, suites and executable metadata and validates their correspondence.
The old equality-only claim is removed. Any narrower representation must preserve those checks,
crash-reviewable checkpoints, changed-input diagnosis and original-detail access through just result.
