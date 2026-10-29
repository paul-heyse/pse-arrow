# Efficiency-oriented design principles in the test system

**Interface-checked · refreshed 2026-10-29 · source `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3` plus
the working tree.** This page holds the primary findings on preparation, build structure, artifact and
evidence lifetimes, scheduling and recovery, judged against AP-07/G9, the relevant DP refinements and
efficiency heuristics 1.0. Static evidence establishes repeated or unnecessary work; no elapsed time,
speedup or capacity was measured. Byte sizes and directory counts are local observations, not cost
measurements. Finding bodies keep their 2026-10-27 evidence (source `87f2eb905`) unless the refresh
section says otherwise.

The [integrated account](architecture-and-assurance.md) explains the interactions. The 2026-10-24
bounded execution follow-up, its corrections and its measured-configuration note are preserved at the
[prior publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/execution-efficiency.md#bounded-execution-follow-up--2026-10-24);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition.

## Refresh at `7bd341f0b` — 2026-10-29

The native route removes several Python-side costs: the native adapter computes the Arrow-leaf closure
once per Cargo invocation and persists no cache; native fixtures need no registry rewrite; native
providers reuse content-keyed generations (`crates/pse-host/src/native/providers.rs:325-392`). The
Python lifecycle machinery behind DTA-038/039/042/047/048/052 stays reachable through `py-unit`
(`justfile:816-822`) and `unit-native-package`/`unit-native-selected`. The native replacements
reproduce three earlier causes in new code.

| Finding | Outcome | Current residual and decisive evidence |
|---|---|---|
| [DTA-018](#dta-018) | Recurs in the native replacement | Every native test builds a fresh `SharedRuntime`, session factory and model binding and admits physical primitives (`reference_support/mod.rs:13-37`), even for narrow claims such as cone geometry (`native_conic.rs`). Every canonical native test runs `NativeCanonicalService::launch` (`native_service.rs:783-848`): manager connection, transient service start, listener wait, viewer provisioning, full schema installation (`initialize_database`) and `connect_ready`. The seven `worker` cases and the study, repeated-analysis and lifecycle journeys repeat that identical schema-ready preparation. The cost shapes test granularity: `native_repeated_analysis.rs` is a one-test binary of about 1.8k lines whose single journey (about 400 lines, `:1326-1737`) fuses project-source, workbench, export, correction and aggregation claims, so the first failure masks the rest and the project-source claims have no other native home. In `pse-operations`, every `native_service` test (eleven top-level plus the progression module), each publication control and each runtime consumer repeats the launch and full schema installation (about 1,100 DEFINE statements); the six progression controls take only `&CanonicalStore` and assert database-local semantics, yet each launches a fresh service, and the fixed `native_fixture`/`canonical` namespace prevents several databases sharing one service. Migration, restart, stale-handle and in-flight tests genuinely need their own lifecycle. |
| [DTA-046](#dta-046) | Escalated on the native route, under the chosen unbounded policy | `ManagedWorkerCase::new` selects the default profile from `.config/agent-capacity.toml` and divides the *whole-workstation* allocation among its own workers; the test asserts two workers above 64 GiB each (`worker.rs:100-113,1668-1670`). Local journeys each build a 4 GiB / 16-thread / all-CPU-jobs runtime (`reference_support/context.rs:24-45`). The native route runs up to 16 such processes (`test-threads = 16`) with no group ([DTA-015](evidence-routes-and-execution.md#dta-015)), so the joint-demand question is no longer bounded by the `scientific-owner` barrier. AGENTS.md and the environment guide deliberately select an uncapped local mode; the finding is not a call for caps but that tests which each claim the whole envelope are run concurrently with no evidence that their joint demand fits that policy. Oversubscription is suspected, not measured; separate checkpoint runs passed (58c). |
| [DTA-053](#dta-053) | Live, extended | `pse-runtime` now has 22 integration binaries, each linking the full runtime, DataFusion and solver closure, across about ten distinct `required-features` sets (`crates/pse-runtime/Cargo.toml:150-250`). Command-line opt-in features are not unified, so each separately invoked set rebuilds `pse-backend-native` and `pse-runtime`; one invocation with a superset would build every target once, but no recipe does so and the plan checkpoints used about eight separate selections. A focused `unit-package pse-runtime <filter>` still compiles the whole 327k-line crate with about 68k lines of tests. The tooling suite (126 modules) still has no focused route. |
| [DTA-052](#dta-052) | Live on Python; new native footprint | Python publication paths unchanged. `scripts/bootstrap-host` keeps every published `generation.*` with no retirement. Failed native private stores (whole service roots with credentials) accumulate with no reclaimer: no source references the `pse-canonical-native-` prefix; 39 such directories were observed in `/tmp`. Per-test disposal policy is [DTA-060](evidence-routes-and-execution.md#dta-060). |
| [DTA-030](#dta-030) | Overstated, unchanged; footprint grows | `reference_pid_exchange` also decodes the whole capsule per process (`tests/fixtures/scientific-inputs.rs:27-34`). |
| [DTA-033](#dta-033) | Superseded, residual | `py-unit` still runs `python_extension sync auto` (`justfile:817`). |
| [DTA-034](#dta-034) | Holds on Python; native counterpart bounded | The native adapter runs one `locate-project`, one `metadata --all-features` and one or two unit-graph passes per compiler invocation (`correctness.rs:265-319`). |
| [DTA-038](#dta-038), [DTA-039](#dta-039), [DTA-042](#dta-042), [DTA-047](#dta-047), [DTA-048](#dta-048) | Partial, live on Python | Unchanged sources; reachable through the routes above. Native fixtures retire DTA-038's premise on their path. |
| [DTA-043](#dta-043) | Holds on Python; premise retired on native | Native fixtures use a private server per test; no shared service. |
| [DTA-044](#dta-044) | Established (narrowed) | `no_canonical_allocation` still checks a nonexistent group (`python/pse/tests/conftest.py:98`). |
| DTA-029, 031, 032, 035, 036, 037, 040, 045 | Established, unchanged | No change to decisive sources. |

## <a id="dta-052"></a>DTA-052 — Prepared artifacts and test evidence are published without content reuse or a convergent lifecycle

**Principles:** AP-07, DP-10, DP-19, DP-20, H12, H14, H16, H19, H25, H26, H27; companion §6, §8.
**Status:** new, 2026-10-27; it absorbs the remaining lifetime parts of [DTA-042](#dta-042) and
[DTA-047](#dta-047), and explains the regression in [DTA-033](#dta-033).

**Claim at risk.** Immutable preparation is established once and reused while its premises hold, and
every artifact or resource the test system creates has a reachable path to retirement once nothing
references it (ADR-0231's "the owner also controls replacement and reclamation"; §24.1's Plan53 E
paragraphs).

**Structural cause.** The protection side of the lifecycle (publication, custody, pending/uncertain
records, borrowers) is complete; the convergence side either does not exist or is not wired to any
reachable step, and some publications key validity on identity that changes every time.

- **Per-invocation extension generations.** Every `just py-unit`/`py-test` runs
  `python_extension.run_stages("sync")`, whose prepare stage calls `extension_prepare` →
  `artifact_publication.publish` unconditionally under a fresh generation. Publication copies, fsyncs,
  hashes and imports the roughly 1.27 GB image. `stubs_prepare` compares a receipt containing the
  generation *path*, so it misses on every publish and regenerates stubs (a second Arrow resolve and
  `cargo run xtask python-stubs`). `artifact_publication.publish` deletes only on failure; no
  reclaimer exists anywhere in `scripts/`. Plan 56f F05 states "The owner also controls replacement
  and reclamation" and "Reclamation must respect attached owners", and records the package complete
  (CA06); its reclamation half is an unmet obligation of that packet, with Plan 56 as disposition
  owner. Locally, five distinct generations (about 5.9 GB, two of them F05's deliberate acceptance
  generations) were retained within about three and a half hours and none was retired.
  Identical-content republication follows from source, not from that observation.
  Evidence-executable generations follow the same lifecycle. The generation lifecycle also serves
  capability and agent attachment (ADR-0231), so that half is shared with the broad account's scope,
  which decides its identity; test-owned artifacts below are primary here.
- **Per-invocation scientific capsules.** `scripts/scientific_inputs.py::prepare_controller` writes
  `build/scientific-inputs/<time_ns>.json` per invocation; nothing removes them (thirty files, about
  115 MB locally).
- **Discovery and recovery not wired.** `test_resources.py::reclaim_reports` is the only reader of
  the `scratch-candidates`/`fixture-candidates` entries and `cleanup-cursor.json`, and has no
  production, CLI or recipe caller since `d4371a576` removed the global sweep. `resource_store.py::{_markers, _save}`
  still publish and fsync those entries for every passed fixture. `relocate_reports` (journaled
  custody conversion) is reachable only from tests, while worktree removal never calls it although
  report bytes are per checkout and authority is host-wide.
- **Deferred cleanup stops early.** In `scripts/test_run.py::execute_selected`, post-terminal
  cleanup runs `for resource in deferred: resources.reclaim(resource)` inside one `try`, so the
  first failure abandons the rest, and the loop is skipped entirely whenever any deadline exists.

**Consequence.** Each focused Python iteration pays a full image copy, hash and import plus stub
regeneration even when nothing changed. Unreferenced generations, capsules and passed resources
accumulate with no owner to converge them; every passed fixture writes discovery entries nobody reads;
a removed worktree can leave dangling authority for retained reports. Disk space is maintainer-managed,
so the issue is the missing lifecycle owner, not a disk guard.

**Strongest legitimacy argument.** Attached readers may hold older generations (ADR-0231/RC06),
so deletion must know who still borrows; ADR-0231 deliberately rejects inferring artifact freshness
from source trees and build records; removing the global sweep was correct; per-resource `reclaim`,
`list` and `resume` exist. These justify retaining *referenced* generations, constructing explicitly
and avoiding sweeps. They do not require republishing an identical image, keying stub currency on a
generation path, or never retiring unreferenced artifacts. Content-keyed publication at the
constructor is compatible with ADR-0231, because it is not a read-time scan.

## <a id="dta-053"></a>DTA-053 — The focused feedback loop is scoped by crate- and variant-wide build units rather than the responsibility under test

**Principles:** AP-06, AP-07, DP-10, DP-17, H2, H12, H13, H27, H28; companion §8.
**Status:** new, 2026-10-27. Structure established; magnitude suspected, not measured.

**Claim at risk.** `just unit-package` gives a focused loop sized to the responsibility under test:
"what developers and agents must instantiate to exercise a responsibility".

**Structural cause.**

- `pse-runtime` is one crate of about 324k lines (roughly 38% test code by heuristic parse) with
  inline `#[cfg(test)]` modules; `pse-backend-native` (about 139k) and `pse-math` (about 90k) are
  similar. Test selection happens at run time; the compile scope stays the crate. Under
  `[profile.dev]` (inherited by test) workspace crates build at O2 with incremental compilation, so
  one edited assertion pays the whole-crate front end, a large incremental session (about 0.5 GB per
  `pse-runtime` session locally) and a full test-binary link.
- Independent variant axes all land on that crate: force-validate on/off (ADR-0170), dev-dependency
  test features (`pse-operations` resolves to `[default,test-support]` with dev edges), canonical and
  native opt-ins, check versus build, rust-analyzer's separate target, and the extension profiles.
  Roughly ten whole-crate `pse-runtime` compilations exist across ordinary routes; a source edit
  stales all of them, and sccache does not restore incremental workspace units or cdylib links
  (`docs/dev/build-performance.md`). `unit-native-package` accepts any feature string, so variants
  are unbounded.
- About 60% of `pse-runtime` tests sit in files that also hold production code, so on any route that
  builds the lib (`test`, `check-package`, `py-sync`) a test-only edit dirties the lib unit and every
  dependent, including the extension relink.

**Consequence.** A Rust+Python iteration (`check-package` → `unit-package` → `py-sync` → correctness
extension) compiles `pse-runtime` at least four times and links a 1.27 GB cdylib once or twice.

**Strongest legitimacy argument.** ADR-0160 places crate tests in the owning crate, preserving
private access; adding crates needs an ADR; each variant is individually justified; AP-01 does not
require compile boundaries to follow responsibilities. The owners have decided the profile and unit
questions: `docs/dev/build-performance.md` says the existing O2 profiles serve the daily workflows,
and Plan 54c C04 retains workspace incremental compilation and features "unless a selected edit/build
sequence supports a better realization", with isolated profiling triggered by an actual recurring
binary-owned hotspot. This finding does not re-litigate those decisions. Its point is a gap in their
evidence: the retention decision rests on edit probes over `crates/pse-compiler/src/{workspace.rs,lib.rs}`
(`scripts/build_measurements.py`, a crate of about 39k lines), and the `pse-runtime` lib-test edit
sequence, the one most likely to meet the stated trigger, has not been sampled. Moving inline test
modules to sibling `#[cfg(test)] mod x;` files is available within current contracts for the third
mechanism.

**Tooling tests have no focused route either.** `scripts/setup_report.py` discovers all 181
`scripts/tests/test_*` modules (about 95k lines) in one serial `unittest` process, and
`scripts/affected.py` maps no `scripts/` edit to tooling tests. Hand-picking a module misses
consumers because of the cross-imports in [DTA-051](doubles-and-coupling.md#dta-051). For the
execution half, which doubled, the choice is a whole serial run or a guess. Confidence is moderate;
cost was not measured.

## Residuals of earlier findings

### <a id="dta-030"></a>DTA-030 — Shared reference preparation repeats a union closure for differently scoped consumers

**Refresh:** closure overstated. The reuse is a per-process `OnceLock` in
`pse-compiler/src/authored_transfer_tests.rs::authored_fixture_workspace`, but `cargo t` is nextest
(`.cargo/config.toml`), which runs each test in its own process. `docs/dev/validation-assessment.md`
itself states "nextest children do not share an `Arc`". Each authored-transfer test still rebuilds
and admits the bounded-reference closure; `process_composition_tests` still uses `fresh_workspace`
per test. Each scientific test process also decodes the whole capsule
(`tests/fixtures/scientific-inputs.rs::capsule`): about 4 MB of JSON encoding about 1.1 MB of raw
bytes as integer arrays, across nineteen families, before cloning the one it needs. The plan's
evidence shows the reuse is *correct* (refusal controls pass), not that the repeated preparation is gone.

### <a id="dta-033"></a>DTA-033 — Focused Python routes unconditionally invoke a broad extension-preparation pipeline

**Refresh:** D03's closure is superseded by the ADR-0231 direction, with residual cost. D03's skip
relied on freshness inference that ADR-0231 (proposed, revised under maintainer direction on
2026-10-27) deliberately rejects; `4a6d14469` delegates incremental decisions to Cargo/Maturin and
publishes on every explicit construction. This is not reopening that decision. The residual cost is
republishing an identical image and path-keyed stub currency; the test
`test_child_selection_carries_only_issued_image_and_stub_scope_stays_independent` checks stub reuse
only without an intervening republish. Primary home: [DTA-052](#dta-052).

### <a id="dta-039"></a>DTA-039 — Broad source capture and repeated artifact verification have different validity scopes

**Refresh:** partially resolved. Scope-before-read is delivered (`validation.sources` calls
`includes_input` before `source_stamp`). On the ordinary in-process route, `native_tests._rust_capture`
and `run_rust` → `prepared_rust_inventory` compute the rust-product source identity three times
(about 100 MB of tracked inputs) and hash every test binary and `ldd`-resolved library twice, with no
process or admission boundary between the second and third passes; `observer_rust` deliberately avoids
this. The validity bracket is the whole workspace for a one-package selection (H5), so another agent's
edit anywhere in crates, scripts or vendored sources during the build refuses the run as "changed
during preparation". The begin/end bracket is justified; the third pass and workspace-wide refusal
scope are not.

### <a id="dta-038"></a>DTA-038 — Fixture registration rewrites selection-bearing invocation state under shared admission

**Refresh:** partially resolved. Membership changes no longer rewrite the selection
(`resource_store.py::FileStore._publish_invocation`; `_invocation_object` publishes it once by content
hash). Each registration still serializes and digests the whole declaration and requirements about six
times (`_invocation_view`, three `_publish_invocation` calls, `test_resources.register`'s
`read_invocation` and `invocation_snapshot`), four inside the invocation lock that `register` takes
while holding the service-wide `surreal_server.py::context_admission`. The members map is rewritten
three times per registration, O(M²) bytes over an invocation of M fixtures, and each Rust fixture
action starts a Python bridge process (`pse-operations/src/testing.rs::resource_bridge_until`).

### <a id="dta-042"></a>DTA-042 — Coarse failure retention and cleanup joins amplify later lifecycle work

**Refresh:** partially resolved. E02's joined settlement is implemented (`testing.rs::settle` with
`tokio::join!` and `join_settlement`). The post-terminal deferred loop still aborts at the first error
and is skipped under any deadline; that residual is part of [DTA-052](#dta-052).

### <a id="dta-047"></a>DTA-047 — Current nondisposable reports still traverse generic reclamation machinery

**Refresh:** partially resolved. Custody, sealing and origin authentication are retained and
`_markers` admits only reports with scratch artifacts and verified provenance. The candidate list's
only production consumer was removed about four hours after it was built (`f7f0ca179`, then
`d4371a576`), leaving writes without a reader; see [DTA-052](#dta-052).

### <a id="dta-048"></a>DTA-048 — Checkpoint serialization repeats growing evidence maps and failure payloads

**Refresh:** partially resolved. The compact root and selected reads hold
(`assessment_store.py::{publish, selected}`). `validation.py::run_gates` still checkpoints about ten
times, each re-serializing and hashing every accumulated check record and the source cut through
`immutable()`, re-reading existing detail through `test_resources.read_report_file`, and rewriting
`failures.json` and `test-findings.json`. Work grows with gate count times accumulated detail.

### <a id="dta-044"></a>DTA-044 — The necessity of the Python canonical-owner barrier remains unestablished

**Refresh:** narrowed closure holds; `test_requirements.pytest_group` groups only `scientific-owner`
and declared exclusions. `python/pse/tests/conftest.py::no_canonical_allocation` asserts the absence
of a `canonical-owner` group that no longer exists, so that check cannot fail.

### <a id="dta-018"></a>DTA-018 — A canonical fixture reconstructs a complete integration lifecycle per test

**Refresh:** closure established as bounded (two local Python migrations; fresh database, schema and
drain controls retained).

### <a id="dta-029"></a>DTA-029 — Store-test selection is coupled to a feature-dependent production-library variant

**Refresh:** closure established. `canonical-tests` gates only test modules; hooks sit under
`test-support`, selected by dev dependency.

### <a id="dta-031"></a>DTA-031 — Capability-only Python tests acquire a durable fixture through a broad runtime dependency

**Refresh:** closure established (`test_native_workflow.py::test_native_capability_discovery_and_hard_cut`
uses an ephemeral runtime with `no_canonical_allocation`).

### <a id="dta-032"></a>DTA-032 — Overlapping testing/tooling routes prepare different feature closures

**Refresh:** closure established for its narrow scope. Remaining variant multiplicity is
[DTA-053](#dta-053).

### <a id="dta-034"></a>DTA-034 — The unprepared fallback repeats Rust resolution and enumeration

**Refresh:** closure established. `select.main` → `run_rust` → `ordinary_rust_capture` performs one
build, one retained listing and one run, reusing resolve metadata as `--cargo-metadata`.

### <a id="dta-035"></a>DTA-035 — Embedding shared reference documents widens compile-time change propagation

**Refresh:** closure established for migrated consumers; no `include_*` of `packages/reference`
remains. Live readers are a [DTA-027](doubles-and-coupling.md#dta-027) residual.

### <a id="dta-036"></a>DTA-036 — Package-based affected-test preview does not represent scientific fixture dependencies

**Refresh:** mechanism established. `scripts/scientific_inputs.py::affected` widens on unknown or new
members; consumer edges remain declared rather than observed ([DTA-027](doubles-and-coupling.md#dta-027)).

### <a id="dta-037"></a>DTA-037 — The grouped codegen comparisons repeat overlapping generation work

**Refresh:** closure established; the `codegen-check` group runs one aggregate plus independent checks.

### <a id="dta-040"></a>DTA-040 — Aggregate snapshot qualification has a wider invalidation window than an individual observation

**Refresh:** the corrected judgment holds; aggregate snapshot semantics and scoped reuse are unchanged
(`validation_receipts.py::classify`).

### <a id="dta-041"></a>DTA-041 — Disposability alone does not establish that fixture recovery is overengineered

Withdrawn 2026-10-23; never reused. This refresh again found concrete obligations for
pending/uncertain protection (discovery entries precede authority, so a crash leaves an entry that
must stay protective).

### <a id="dta-043"></a>DTA-043 — Per-run test exclusion does not by itself exclude another run sharing the service

**Refresh:** closure holds structurally (service-access discovery in `resource_store.py::_markers`;
`service_borrowers.py::service_resources` refuses unresolved associations).

### <a id="dta-045"></a>DTA-045 — A broad scientific-group regex serializes demonstrably pure controls

**Refresh:** closure holds for the three named pure cases. It was achieved by narrowing name
prefixes, which de-grouped real native modules; see [DTA-016](evidence-routes-and-execution.md#dta-016).

### <a id="dta-046"></a>DTA-046 — Per-runtime ceilings do not establish a safe aggregate concurrent-demand envelope

**Refresh:** closure holds within the retained `scientific-owner` barrier; joint simultaneous demand
remains an open question, not a finding.
