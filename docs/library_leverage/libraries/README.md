# Library applicability assessments

**Account state: first assessment, 2026-10-27.** This collection is maintained by the
[library assessment skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/leverage-library-assessment/SKILL.md).
For each [codebase opportunity](../codebase/README.md) it records which library or native-system
capabilities could address it, under what conditions, and with what work left to the codebase.

Each library account keeps two kinds of content apart:
- **Reusable types.** These are repository-agnostic and use handles such as `arrow-ipc/…`. They
  stay useful after their current matches retire.
- **Current applicability.** This links each type to a `CBO`, together with the realization
  considered, a conclusion, the residual obligations and any unsettled premises.

A conclusion is one of *fits*, *fits with conditions*, *ruled out here* or *unsettled*. It is an
applicability judgment, not a recommendation. Comparative selection belongs to
[comparative reviews](../reviews/README.md).

Read alongside the [framework](../library-leverage-framework.md), the [base patterns](../patterns.md)
and the shared [additional patterns](../assessment-patterns.md) (`LOP-X001`–`LOP-X010`,
revision 2). Revision 2 added seven patterns found in this pass.

## Baseline and method

| Item | Value |
| --- | --- |
| Research date | 2026-10-27 |
| Opportunities consumed | [Codebase account](../codebase/README.md) published at `c2f8b485a` (source baseline `dbc4c9436`); `CBO-001`–`CBO-052` |
| Patterns consumed | Base edition 0.2; additional patterns revision 1 at the start, revision 2 published with this pass |
| Realizations | Each account names its realizations: installed pins from `Cargo.lock`, `uv.lock` and `vendor/`, and upgrade routes with release dates. On this host that also means kernel 7.0, systemd 255, CPython 3.14.8 and the pinned nightly. Unreleased upstream work is labelled as such. |
| Sources | Context7 first, then primary documentation, changelogs, tagged upstream source and tests, installed sources, and downloaded crate archives. |
| Method | Nine read-only research clusters: columnar/relational, SurrealDB, async lifecycle/observation, OS process/host/files, symbolic math/compilation, native solvers, statistics/sampling, Rust/Python boundary and schema, language and developer tooling. The coordinator integrated their drafts and spot-checked the decisive claims against installed or cloned source. |
| Evidence | Library support is *Interface-checked* as stated in each entry. Every application to pse-arrow is *Proposed*. Three probes are *Tested* within their stated conditions (below). No quantitative benefit is claimed. |

**Probes run (all three allotted):**
- [egobox-doe seed determinism](../../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md):
  every design kind was bit-identical across opt-level, `x86-64-v3` and thread count on one
  host. Bounds mapping is inexact, and a repeated `sample()` continues the RNG stream.
- [msgspec tagged decode](../../design_review/evidence/library-leverage-msgspec-tagged-decode-2026-10-27/README.md):
  a single tagged Struct does not require its tag; only a union of two or more does.
  Construction validates nothing, and `convert` accepts `nan` and `inf`.
- [Symbolica root refinement](../../design_review/evidence/library-leverage-symbolica-root-refinement-2026-10-27/README.md):
  `refine_root_interval` mishandles endpoint roots and intervals symmetric about zero, so the
  local endpoint screen is a correctness guard.

## Synthesis

**1. Much custom code is justified at the current pins.** These negative results are a main
output of the pass:
- **Arrow IPC preflight.** arrow-ipc 59.3.0 allocates advertised lengths before reading them, so
  the hand-written IPC preflight is a guard rather than duplication
  ([LOP-X009](../assessment-patterns.md#lop-x009), [arrow/ipc](arrow/ipc.md)).
- **Drop-to-cancel libraries.** Several mechanisms need work to stay owned until it exits, but the
  library candidates cancel by drop or let ready work win, so they were ruled out
  ([LOP-X005](../assessment-patterns.md#lop-x005)):
  - single flights: moka, quick_cache, async_singleflight;
  - the I/O gate: `LimitStore`;
  - native join waits: `run_until_cancelled`, `timeout_at`.
- **Python lock libraries.** filelock, fasteners and flufl.lock cannot preserve `flock`
  semantics, absolute deadlines or fail-closed behaviour.
- **Process supervision.** process-wrap cannot prove that a process group drained.
- **Identity framing.** The serde framer cannot reproduce the existing identity byte layout.
- **Symbolica refinement.** The cubic-refinement guard is required, as the probe showed.

**2. Recorded upstream triggers.** Several have changed state:
- **SUNDIALS.** The IDAS quadrature fix shipped in 7.8.0, so the vendored workaround's trigger
  has fired, although `sundials-sys` has not been re-released.
- **Uno.** v2.9.2 changed the QP path that the local feasibility seed patches.
- **Diffsol.** It is still infallible in `set_linearisation` at 0.18.0.
- **Symbolica.** 3.0.1 is the newest release. A resource budget exists only on an unreleased
  branch, and `main` has already rewritten the code that local cost mirrors copy.
- **SurrealDB.** 3.3.4 is the newest release. The 35k-line SDK delta is about 3.2k substantive
  lines.
- **faer.** 0.24.4 is the newest release.
- **Arrow.** Arrow 60 fixes several reader hazards, but no DataFusion release uses it yet.

**3. Conditional fits with installed or adjacent capabilities.** These are the most actionable
routes for comparative review:
- **Arrow compute kernels** for dense trajectory runs and selection
  ([arrow/compute](arrow/compute.md)).
- **DataFusion pruning** over recorded dense-output statistics ([datafusion](datafusion/README.md)).
- **Exact encoded sizes** from `StreamEncoder` ([arrow/ipc](arrow/ipc.md)).
- **One condition estimator.** FERAL's public Hager–Higham estimator could run over every factor
  type ([feral](feral/README.md)).
- **No-clobber publication** with `RENAME_NOREPLACE` or `persist_noclobber`
  ([rustix](rustix/README.md), [tempfile](tempfile/README.md)).
- **Kernel-owned containment and identity** through per-child cgroups (`cgroup.kill`, `populated`)
  and pidfs identity ([LOP-X008](../assessment-patterns.md#lop-x008),
  [linux-cgroup-v2](linux-cgroup-v2/README.md)).
- **systemd manager facts over D-Bus** ([systemd-manager-api](systemd-manager-api/README.md),
  [zbus-systemd](zbus-systemd/README.md)).
- **Seeded designs.** `egobox-doe` at its existing pin ([egobox-doe](egobox-doe/README.md)).
- **Rendered diagnostics.** miette source rendering at the admission driver
  ([miette](miette/README.md)).
- **Python surface.** pyclasses as the public Python types ([pyo3](pyo3/README.md)).
- **Registry data.** A postcard or ciborium blob, once the model has a serde contract
  ([postcard](postcard/README.md)).
- **Retry.** backon for conflict retries ([backon](backon/README.md)).
- **Parsing.** winnow structural parsing over the existing tokens ([winnow](winnow/README.md)).
- **Evidence index.** Parquet plus DataFusion for the development evidence index
  ([parquet-datafusion-index](parquet-datafusion-index/README.md)). Correction (2026-10-27): this
  route is conditional on the review's premise U3, whether the development evidence index must
  stay on SurrealDB ([comparative review](../reviews/README.md#unresolved-premises-and-their-branches)).
- **Telemetry capture.** Installed OTLP stubs, or switching the exporter to HTTP
  ([LOP-X010](../assessment-patterns.md#lop-x010)). Correction (2026-10-27): the HTTP switch is
  not available, because SurrealDB 3.3.4 exports OTLP only over tonic gRPC
  ([surrealdb/server-tooling](surrealdb/server-tooling.md),
  [comparative review](../reviews/developer-tooling.md#cbo-051)).

**4. A lower supplier layer may remove a patch.** The supplier's own generated gRPC client is
generic over a caller-built transport. It could host the SurrealDB lifecycle policy on public
seams, though it would move result and session handling into the repository
([LOP-X004](../assessment-patterns.md#lop-x004), [surrealdb/protocol](surrealdb/protocol.md)).

**5. Reproducibility depends on the whole stack.** Seeded sampling and correlated draws are
bit-stable only as far as their weakest layer: RNG policy, platform libm, SIMD/FMA kernels and
feature unification ([LOP-X006](../assessment-patterns.md#lop-x006)). The governing decision is
whether reproducibility means regenerating from a seed or replaying recorded points.

## Questions for comparative review

The first [comparative review](../reviews/README.md) (2026-10-27) answers these; its entry page
and group pages hold the current judgments. The questions are kept as the assessment posed them.

1. **CBO-009.** Keep the SDK delta and the mirrored layer, move to the direct protocol client, or
   contribute an upstream seam? This hinges on how much result and session handling would be
   re-implemented, and whether SurrealKit can work with an unpatched SDK.
2. **CBO-021 and CBO-001.** Contribute Symbolica hooks upstream (an evaluator abort setter,
   checkpoint polling), or keep mirrors re-derived at each release? Should a derive cover plain
   retained structs, with conservative overrides for tree and hash containers?
3. **CBO-004, CBO-019, CBO-020 and CBO-048.** For containment, choose between delegated child
   cgroups, a systemd scope per child, or keeping the `/proc` inventory. Which side owns the unit
   lifecycle? Which identity rule applies (start ticks or pidfs inode)?
4. **CBO-003.** Is resumable deterministic staging a required contract of the sealed-file store?
   That decides tempfile versus `RENAME_NOREPLACE`.
5. **CBO-028.** Move to SUNDIALS 7.8/7.9 and rebase Uno onto v2.9.2 now, accepting receipt-hash
   changes, or wait for crate releases?
6. **CBO-026.** Compose FERAL's estimator over faer and KLU, which keeps charge-before-allocate,
   or swap to FERAL LU, which gains typed singularity but loses prediction?
7. **CBO-033 to CBO-035.** Is the reproducibility contract "regenerate from seed" or "replay
   recorded points"? Should designs be authored in Rust or in Python (scipy)? Keep the rust-cv LM,
   whose criteria are part of fit identity, or move to basin?
8. **CBO-040 to CBO-043.** Is a serde contract on the registry model acceptable? Must Python
   re-check headers on documents produced in-process by Rust? Do relation rows need
   construction-time validation?
9. **CBO-013 to CBO-015.** Should observation keep a local lifecycle event that feeds the sinks
   directly, or relax "works without a subscriber" and adopt tracing as the backbone? Which
   admission waits need a stop checked first?
10. **CBO-016.** Is a subscription request class (LIVE SELECT) wanted under the bounded-request
    profile, or is in-process notification enough?
11. **CBO-050.** Does "SurrealDB stays canonical" cover development evidence?
12. **CBO-025, a product check rather than a leverage question.** Can the root isolator emit an
    interval symmetric about or straddling zero? The probe shows the library panics or hangs on
    those inputs, and the local screen does not cover them.

**Probe requests not run (beyond the allotment).** These are recorded in the cluster accounts
and could settle review questions:
- LIVE SELECT under the bounded profile, and conflict classification without explicit
  `BEGIN`/`COMMIT` (SurrealDB).
- Thread-pidfd readiness after TLS destructors (tokio, linux-pidfd).
- Delegated cgroup kill and `StartTransientUnit` on systemd 255 (OS).
- SUNDIALS 7.8 without the local fix, and the Uno v2.9.2 rebase (native).
- FBBT bounds from exported Symbolica instructions (math).
- `pse-schema` compile memory and the time of a first `registry()` call (boundary).
- The OTLP HTTP exporter for the capture sink (tooling).

Status update (2026-10-27), from the [comparative review](../reviews/README.md#probes): the
delegated-cgroup and `pse-schema` compile-cost questions were probed; LIVE SELECT under the
bounded profile is moot, because the bounded engine refuses it by design; the OTLP HTTP exporter
question was answered from server source (no HTTP exporter at 3.3.4) and its probe withdrawn, as
was the cargo-hack adapter probe recorded in [cargo-hack](cargo-hack/README.md). Question 12 is
answered statically: the isolator confines intervals to one closed half-line, and the live hazard
is strict containment across square-free factors ([symbolica](symbolica/README.md)).

## No library route found

Each item records the search scope in its library account or cluster notes:
- **CBO-014.** No carrier propagates context into OS threads or foreign callbacks; the remedy is
  one local context value.
- **CBO-011.** No library single flight keeps its producer alive past the last waiter and then
  joins it.
- **CBO-029.** No library provides NLP-oracle callback translation; Uno's conventions are
  configuration.
- **CBO-030.** No library provides a native session lifecycle guard; only the KINSOL session LRU
  fits `quick_cache`.
- **CBO-039.** No library emits the existing `SemanticFrame` byte layout.
- **CBO-042.** No library dedupes across contracts or names `Output` types; this is an open TODO
  in schemars itself.
- **CBO-047.** Python file-ledger primitives belong in one in-repo stdlib module.
- **CBO-017.** Consolidating the wait loops is local. CPython `signal.set_wakeup_fd` is an
  optional refinement.
- **Further gaps.** Cancelling Symbolica `derivative`, `replace`, `coefficient_list` or isolation
  (CBO-021). Exact semidefinite LDLᵀ with a witness (CBO-025). Propagation across BTF blocks
  (CBO-023). The cgroup limit walk (CBO-019). A deadline-bounded `flock` (CBO-003).
- **No consolidated route.** The following were assessed only as consolidation or premise
  corrections, with no consolidated library route: CBO-002 (combined cap, refusal and
  checkpoint), CBO-008, CBO-012, CBO-018 (unsettled pending the pidfd probe), CBO-031, CBO-032
  and CBO-036 (statistics policy).

## Coverage limits

- **Bodies not read.** Large script bodies were not read: `surreal_server.py` and the
  capability server. SurrealKit source and the C++ solver bridges were not read either.
- **Shallow library research.** Several new or peripheral libraries were assessed from docs or
  release metadata only; each account says so. Examples are chumsky, genco, DuckDB, basin,
  deepsize and facet.
- **No dependency resolution.** No candidate crate was resolved against the workspace with
  cargo, so dependency-family conflicts beyond those noted are unverified.
- **Fork revisions.** POUNCE fork revisions after the pin and HiGHS 1.15.1 were not assessed.
- **Peer review of the drafts.** The coordinator spot-checked the decisive claims, including the
  arrow-ipc allocation order, the SUNDIALS fix commit and the Diffsol trait. The cluster drafts
  were not independently re-reviewed line by line.

## Maintaining this collection

Refresh entries in place:
- Revisit a conclusion when its realization, the opportunity's premises or the library's version
  changes.
- Retain reusable types after their matches retire, and list such types in a "retained without
  current match" section here. In this first pass, every type was researched for a current
  opportunity.
- Remove obsolete applicability entries when their opportunities retire, carrying justified
  lessons into the type.
- Correct or merge misleading types deliberately.

## Opportunity coverage

| Opportunity | Library accounts and conclusions |
| --- | --- |
| [CBO-001](../codebase/cross-cutting-mechanisms.md#cbo-001) | [arrow/data](arrow/data.md): fits with conditions; [suitesparse/klu](suitesparse/klu.md): fits with conditions; [faer/sparse-factorization](faer/sparse-factorization.md): fits with conditions; [datafusion](datafusion/README.md): ruled out here; [petgraph-and-rustworkx-core](petgraph-and-rustworkx-core/README.md): premise corrected; [retained-size-crates](retained-size-crates/README.md): fits with conditions; [salsa](salsa/README.md): ruled out here; [feral](feral/README.md): fits; [sundials](sundials/README.md): fits with conditions |
| [CBO-002](../codebase/cross-cutting-mechanisms.md#cbo-002) | [arrow/ipc](arrow/ipc.md): fits with conditions; [bytes](bytes/README.md): ruled out here; [rust-std-io](rust-std-io/README.md): fits with conditions |
| [CBO-003](../codebase/cross-cutting-mechanisms.md#cbo-003) | [atomic-write-file](atomic-write-file/README.md): ruled out here; [cap-std](cap-std/README.md): unsettled; [rust-file-locks](rust-file-locks/README.md): ruled out here; [rustix](rustix/README.md): fits with conditions; [tempfile](tempfile/README.md): fits with conditions |
| [CBO-004](../codebase/cross-cutting-mechanisms.md#cbo-004) | [tokio/process](tokio/process.md): fits; [linux-pidfd/process-identity-and-drain](linux-pidfd/process-identity-and-drain.md): fits with conditions; [linux-cgroup-v2](linux-cgroup-v2/README.md): fits with conditions; [process-wrap](process-wrap/README.md): ruled out here; [rustix](rustix/README.md): fits; [zbus-systemd](zbus-systemd/README.md): unsettled |
| [CBO-005](../codebase/result-data-paths.md#cbo-005) | [arrow/ipc](arrow/ipc.md): fits with conditions; [arrow/data](arrow/data.md): no further route (local justified) |
| [CBO-006](../codebase/result-data-paths.md#cbo-006) | [arrow/compute](arrow/compute.md): fits with conditions |
| [CBO-007](../codebase/result-data-paths.md#cbo-007) | [arrow/compute](arrow/compute.md): fits with conditions; [datafusion](datafusion/README.md): fits with conditions |
| [CBO-008](../codebase/result-data-paths.md#cbo-008) | [arrow/json](arrow/json.md): unsettled |
| [CBO-009](../codebase/persistence-and-transport.md#cbo-009) | [surrealdb/rust-sdk](surrealdb/rust-sdk.md): ruled out here; [surrealdb/protocol](surrealdb/protocol.md): fits with conditions |
| [CBO-010](../codebase/persistence-and-transport.md#cbo-010) | [surrealdb/surrealql-server-constructs](surrealdb/surrealql-server-constructs.md): fits with conditions; [backon](backon/README.md): fits with conditions; [tryhard](tryhard/README.md): fits with conditions |
| [CBO-011](../codebase/persistence-and-transport.md#cbo-011) | [tokio/tokio](tokio/tokio.md): already adopted; [tokio/tokio-util](tokio/tokio-util.md): fits; [object-store/limits-and-caching](object-store/limits-and-caching.md): ruled out here; [object-store/request-admission](object-store/request-admission.md): ruled out here; [quick-cache/single-flight](quick-cache/single-flight.md): ruled out here; [datafusion](datafusion/README.md): ruled out here; [async-singleflight](async-singleflight/README.md): ruled out here; [futures-util](futures-util/README.md): already adopted; [moka](moka/README.md): ruled out here |
| [CBO-012](../codebase/persistence-and-transport.md#cbo-012) | [datafusion](datafusion/README.md): ruled out here |
| [CBO-013](../codebase/execution-lifecycle-and-observation.md#cbo-013) | [metrics](metrics/README.md): ruled out here; [opentelemetry](opentelemetry/README.md): ruled out here; [tracing](tracing/README.md): fits with conditions |
| [CBO-014](../codebase/execution-lifecycle-and-observation.md#cbo-014) | [tokio/tokio](tokio/tokio.md): ruled out here; [tracing](tracing/README.md): ruled out here |
| [CBO-015](../codebase/execution-lifecycle-and-observation.md#cbo-015) | [tokio/tokio](tokio/tokio.md): fits with conditions; [tokio/tokio-util](tokio/tokio-util.md): ruled out here; [tokio/tokio-util](tokio/tokio-util.md): fits with conditions |
| [CBO-016](../codebase/execution-lifecycle-and-observation.md#cbo-016) | [surrealdb/rust-sdk](surrealdb/rust-sdk.md): unsettled; [surrealdb/surrealql-server-constructs](surrealdb/surrealql-server-constructs.md): fits with conditions; [tokio/tokio](tokio/tokio.md): fits with conditions; [tokio/tokio-util](tokio/tokio-util.md): partly adopted |
| [CBO-017](../codebase/execution-lifecycle-and-observation.md#cbo-017) | [cpython-signal-wakeup-fd](cpython-signal-wakeup-fd/README.md): fits with conditions; [pyo3-async-runtimes](pyo3-async-runtimes/README.md): ruled out here; [pyo3](pyo3/README.md): fits; [signal-hook-registry](signal-hook-registry/README.md): ruled out here |
| [CBO-018](../codebase/execution-lifecycle-and-observation.md#cbo-018) | [tokio/tokio](tokio/tokio.md): fits with conditions; [linux-pidfd/thread-exit](linux-pidfd/thread-exit.md): unsettled |
| [CBO-019](../codebase/execution-lifecycle-and-observation.md#cbo-019) | [linux-pidfd/process-identity-and-drain](linux-pidfd/process-identity-and-drain.md): fits with conditions; [cgroups-rs](cgroups-rs/README.md): ruled out here; [linux-cgroup-v2](linux-cgroup-v2/README.md): ruled out here; [procfs](procfs/README.md): fits with conditions; [psutil](psutil/README.md): ruled out here |
| [CBO-020](../codebase/execution-lifecycle-and-observation.md#cbo-020) | [python-dbus-clients](python-dbus-clients/README.md): fits with conditions; [systemd-manager-api](systemd-manager-api/README.md): fits with conditions; [zbus-systemd](zbus-systemd/README.md): fits with conditions |
| [CBO-021](../codebase/mathematics-and-compilation.md#cbo-021) | [symbolica](symbolica/README.md): unsettled |
| [CBO-022](../codebase/mathematics-and-compilation.md#cbo-022) | [cacache](cacache/README.md): fits with conditions; [salsa](salsa/README.md): fits with conditions; [sccache](sccache/README.md): unsettled |
| [CBO-023](../codebase/mathematics-and-compilation.md#cbo-023) | [pounce/presolve-and-fbbt](pounce/presolve-and-fbbt.md): ruled out here; [suitesparse/structural-and-exact](suitesparse/structural-and-exact.md): fits with conditions; [petgraph-and-rustworkx-core](petgraph-and-rustworkx-core/README.md): ruled out here |
| [CBO-024](../codebase/mathematics-and-compilation.md#cbo-024) | [pounce/presolve-and-fbbt](pounce/presolve-and-fbbt.md): fits with conditions; [numerica](numerica/README.md): ruled out here; [symbolica](symbolica/README.md): fits with conditions |
| [CBO-025](../codebase/mathematics-and-compilation.md#cbo-025) | [suitesparse/structural-and-exact](suitesparse/structural-and-exact.md): ruled out here; [numerica](numerica/README.md): ruled out here; [rug-and-malachite](rug-and-malachite/README.md): ruled out here; [symbolica](symbolica/README.md): ruled out here |
| [CBO-026](../codebase/native-solver-integration.md#cbo-026) | [suitesparse/klu](suitesparse/klu.md): fits with conditions; [faer/sparse-factorization](faer/sparse-factorization.md): ruled out here; [feral](feral/README.md): fits with conditions |
| [CBO-027](../codebase/native-solver-integration.md#cbo-027) | [diffsol](diffsol/README.md): ruled out here |
| [CBO-028](../codebase/native-solver-integration.md#cbo-028) | [sundials](sundials/README.md): fits with conditions; [uno](uno/README.md): unsettled |
| [CBO-029](../codebase/native-solver-integration.md#cbo-029) | [pounce/solvers-and-sensitivity](pounce/solvers-and-sensitivity.md): no library route; [uno](uno/README.md): fits with conditions |
| [CBO-030](../codebase/native-solver-integration.md#cbo-030) | [quick-cache/weighted-session-cache](quick-cache/weighted-session-cache.md): fits with conditions (session LRU only) |
| [CBO-031](../codebase/native-solver-integration.md#cbo-031) | [pounce/solvers-and-sensitivity](pounce/solvers-and-sensitivity.md): ruled out here; [clarabel](clarabel/README.md): ruled out here |
| [CBO-032](../codebase/native-solver-integration.md#cbo-032) | [pounce/solvers-and-sensitivity](pounce/solvers-and-sensitivity.md): ruled out here |
| [CBO-033](../codebase/numerical-workflows.md#cbo-033) | [egobox-doe](egobox-doe/README.md): fits with conditions; [rand-family](rand-family/README.md): fits with conditions; [scipy-stats-qmc](scipy-stats-qmc/README.md): fits with conditions; [sobol-burley](sobol-burley/README.md): fits with conditions; [sobol-qmc](sobol-qmc/README.md): fits with conditions |
| [CBO-034](../codebase/numerical-workflows.md#cbo-034) | [faer/dense-decompositions](faer/dense-decompositions.md): fits with conditions; [rand-family](rand-family/README.md): fits with conditions; [sobol-burley](sobol-burley/README.md): fits with conditions; [statrs](statrs/README.md): ruled out here |
| [CBO-035](../codebase/numerical-workflows.md#cbo-035) | [faer/dense-decompositions](faer/dense-decompositions.md): fits with conditions; [argmin](argmin/README.md): ruled out here; [basin](basin/README.md): fits with conditions; [egobox-doe](egobox-doe/README.md): unsettled; [levenberg-marquardt](levenberg-marquardt/README.md): fits; [statrs](statrs/README.md): unsettled |
| [CBO-036](../codebase/numerical-workflows.md#cbo-036) | [statrs](statrs/README.md): fits with conditions |
| [CBO-037](../codebase/authoring-and-identity.md#cbo-037) | [chumsky](chumsky/README.md): fits with conditions; [derive-visitor](derive-visitor/README.md): fits with conditions; [parser-generators](parser-generators/README.md): ruled out here; [proptest-derive](proptest-derive/README.md): unsettled; [winnow](winnow/README.md): fits with conditions |
| [CBO-038](../codebase/authoring-and-identity.md#cbo-038) | [miette/source-rendering](miette/source-rendering.md): fits with conditions; [ariadne](ariadne/README.md): ruled out here; [codespan-reporting](codespan-reporting/README.md): ruled out here |
| [CBO-039](../codebase/authoring-and-identity.md#cbo-039) | [serde](serde/README.md): ruled out here |
| [CBO-040](../codebase/registry-generation-and-python-boundary.md#cbo-040) | [arrow/schema-serde-and-ipc](arrow/schema-serde-and-ipc.md): fits with conditions; [bincode](bincode/README.md): ruled out here; [ciborium](ciborium/README.md): fits with conditions; [postcard](postcard/README.md): fits with conditions; [rkyv](rkyv/README.md): ruled out here |
| [CBO-041](../codebase/registry-generation-and-python-boundary.md#cbo-041) | [cattrs](cattrs/README.md): subsumed by msgspec convert; [msgspec](msgspec/README.md): fits with conditions; [pythonize](pythonize/README.md): fits with conditions; [schemars](schemars/README.md): not applicable; [pyo3](pyo3/README.md): fits (entry added from the comparative review) |
| [CBO-042](../codebase/registry-generation-and-python-boundary.md#cbo-042) | [schemars](schemars/README.md): ruled out here; [serde](serde/README.md): ruled out here |
| [CBO-043](../codebase/registry-generation-and-python-boundary.md#cbo-043) | [cattrs](cattrs/README.md): fits with conditions; [datamodel-code-generator](datamodel-code-generator/README.md): unsettled; [msgspec](msgspec/README.md): fits with conditions |
| [CBO-044](../codebase/registry-generation-and-python-boundary.md#cbo-044) | [pyo3](pyo3/README.md): fits with conditions |
| [CBO-045](../codebase/registry-generation-and-python-boundary.md#cbo-045) | [miette/derive-and-forwarding](miette/derive-and-forwarding.md): fits with conditions; [strum](strum/README.md): already adopted |
| [CBO-046](../codebase/registry-generation-and-python-boundary.md#cbo-046) | [surrealdb/surrealql-server-constructs](surrealdb/surrealql-server-constructs.md): fits with conditions; [surrealdb/syn-and-sql](surrealdb/syn-and-sql.md): fits; [askama](askama/README.md): fits with conditions; [genco](genco/README.md): unsettled; [minijinja](minijinja/README.md): fits with conditions |
| [CBO-047](../codebase/developer-tooling.md#cbo-047) | [atomicwrites](atomicwrites/README.md): ruled out here; [python-file-locks](python-file-locks/README.md): ruled out here |
| [CBO-048](../codebase/developer-tooling.md#cbo-048) | [surrealdb/server-tooling](surrealdb/server-tooling.md): fits with conditions; [linux-cgroup-v2](linux-cgroup-v2/README.md): fits with conditions; [psutil](psutil/README.md): ruled out here; [python-dbus-clients](python-dbus-clients/README.md): fits with conditions; [systemd-manager-api](systemd-manager-api/README.md): fits with conditions |
| [CBO-049](../codebase/developer-tooling.md#cbo-049) | [cargo-hack](cargo-hack/README.md): premise corrected (already adopted); [cargo-nextest](cargo-nextest/README.md): fits with conditions |
| [CBO-050](../codebase/developer-tooling.md#cbo-050) | [duckdb](duckdb/README.md): unsettled; [parquet-datafusion-index](parquet-datafusion-index/README.md): fits with conditions; [sqlite](sqlite/README.md): unsettled |
| [CBO-051](../codebase/developer-tooling.md#cbo-051) | [grpcio](grpcio/README.md): fits with conditions; [h2](h2/README.md): fits with conditions; [opentelemetry-proto](opentelemetry-proto/README.md): fits with conditions; [python-frontmatter](python-frontmatter/README.md): ruled out here |
| [CBO-052](../codebase/developer-tooling.md#cbo-052) | [cargo-nextest](cargo-nextest/README.md): ruled out here; [cyclopts](cyclopts/README.md): fits with conditions; [fastmcp](fastmcp/README.md): ruled out here; [just](just/README.md): fits with conditions; [typer](typer/README.md): ruled out here |

## Library index

| Library account | Reusable types | Current matches |
| --- | --- | --- |
| [argmin](argmin/README.md) | `argmin/backend-generic-local-solvers` | CBO-035 |
| [ariadne](ariadne/README.md) | `ariadne/source-id-span-rendering` | CBO-038 |
| [arrow](arrow/README.md) | `arrow-ipc/untrusted-file-admission`, `arrow-ipc/caller-owned-push-decoding`, `arrow-ipc/encoded-size-without-body-copy`, `arrow-data/layout-description`, `arrow-data/memory-size-functions`, `arrow-data/validation-and-layout`, `arrow-compute/run-partitioning`, `arrow-compute/predicate-selection`, `arrow-compute/extrema`, `arrow-json/serde-rows-to-arrow`, `arrow-schema/serde-schema-types`, `arrow-ipc/deterministic-schema-and-batch-embedding`, `serde-arrow/serde-structs-as-arrow-tables` | CBO-001, CBO-002, CBO-005, CBO-006, CBO-007, CBO-008, CBO-040 |
| [askama](askama/README.md) | `askama/compile-time-checked-templates` | CBO-046 |
| [async-singleflight](async-singleflight/README.md) | `async-singleflight/leader-follower-dedup` | CBO-011 |
| [atomic-write-file](atomic-write-file/README.md) | `atomic-write-file/replace-with-durable-commit` | CBO-003 |
| [atomicwrites](atomicwrites/README.md) | `atomicwrites/durable-replace-or-link` | CBO-047 |
| [backon](backon/README.md) | `backon/retry-combinator` | CBO-010 |
| [basin](basin/README.md) | `basin/backend-generic-nonlinear-least-squares` | CBO-035 |
| [bincode](bincode/README.md) | `bincode/unmaintained-positional-binary` | CBO-040 |
| [bytes](bytes/README.md) | `bytes/bufmut-limit-writer` | CBO-002 |
| [cacache](cacache/README.md) | `cacache/content-addressed-disk-store` | CBO-022 |
| [cap-std](cap-std/README.md) | `cap-std/directory-confined-operations`, `cap-std/capability-tempfile-replace` | CBO-003 |
| [cargo-hack](cargo-hack/README.md) | `cargo-hack/feature-powerset-enumeration` | CBO-049 |
| [cargo-nextest](cargo-nextest/README.md) | `cargo-nextest/declarative-test-scheduling`, `cargo-nextest/layered-tool-configuration`, `cargo-nextest/setup-and-wrapper-scripts`, `cargo-nextest/list-archive-record` | CBO-049, CBO-052 |
| [cattrs](cattrs/README.md) | `cattrs/msgspec-preconf-passthrough` | CBO-041, CBO-043 |
| [cgroups-rs](cgroups-rs/README.md) | `cgroups-rs/cgroup-manager` | CBO-019 |
| [chumsky](chumsky/README.md) | `chumsky/token-input-with-recovery` | CBO-037 |
| [ciborium](ciborium/README.md) | `ciborium/self-describing-binary-serde` | CBO-040 |
| [clarabel](clarabel/README.md) | `clarabel/conic-standard-form`, `clarabel/accuracy-controls-and-resolve` | CBO-031 |
| [codespan-reporting](codespan-reporting/README.md) | `codespan-reporting/id-keyed-multi-file-rendering` | CBO-038 |
| [cpython-signal-wakeup-fd](cpython-signal-wakeup-fd/README.md) | `cpython/signal-wakeup-fd` | CBO-017 |
| [cyclopts](cyclopts/README.md) | `cyclopts/typed-cli-from-signatures` | CBO-052 |
| [datafusion](datafusion/README.md) | `datafusion/pruning-arbitrary-containers`, `datafusion/memory-reservation-and-batch-sizing`, `datafusion/default-cache`, `datafusion/nested-occurrence-plans` | CBO-001, CBO-007, CBO-011, CBO-012 |
| [datamodel-code-generator](datamodel-code-generator/README.md) | `datamodel-code-generator/jsonschema-to-msgspec-structs` | CBO-043 |
| [derive-visitor](derive-visitor/README.md) | `derive-visitor/derived-tree-traversal`, `local-alternative/span-insensitive-equality` | CBO-037 |
| [diffsol](diffsol/README.md) | `diffsol/fallible-newton-linear-solver`, `diffsol/checkpointed-adjoint` | CBO-027 |
| [duckdb](duckdb/README.md) | `duckdb/in-place-file-query-engine` | CBO-050 |
| [egobox-doe](egobox-doe/README.md) | `egobox-doe/seeded-space-filling-designs`, `egobox-moe/mixed-integer-design-mapping` | CBO-033, CBO-035 |
| [faer](faer/README.md) | `faer/sparse-lu-scratch-and-reuse`, `faer/runtime-simd-dispatch`, `faer/dense-covariance-square-root` | CBO-001, CBO-026, CBO-034, CBO-035 |
| [fastmcp](fastmcp/README.md) | `fastmcp/generate-cli-from-tool-schemas` | CBO-052 |
| [feral](feral/README.md) | `feral/hager-higham-estimator-over-any-factor`, `feral/unsymmetric-sparse-lu`, `feral/symmetric-ldlt-symbolic-estimates` | CBO-001, CBO-026 |
| [futures-util](futures-util/README.md) | `futures-util/shared-completion` | CBO-011 |
| [genco](genco/README.md) | `genco/language-aware-quasi-quoting` | CBO-046 |
| [grpcio](grpcio/README.md) | `grpcio/python-grpc-server-for-stub-services` | CBO-051 |
| [h2](h2/README.md) | `h2/sans-io-http2-protocol` | CBO-051 |
| [just](just/README.md) | `just/declarative-recipe-interface` | CBO-052 |
| [levenberg-marquardt](levenberg-marquardt/README.md) | `levenberg-marquardt/minpack-style-lm` | CBO-035 |
| [linux-cgroup-v2](linux-cgroup-v2/README.md) | `linux-cgroup-v2/containment-kill-and-drain`, `linux-cgroup-v2/limit-readback`, `linux-cgroup-v2/psi-triggers` | CBO-004, CBO-019, CBO-048 |
| [linux-pidfd](linux-pidfd/README.md) | `linux-pidfd/thread-exit-readiness`, `linux-pidfd/handle-based-process-identity`, `linux-pidfd/spawn-into-cgroup` | CBO-004, CBO-018, CBO-019 |
| [metrics](metrics/README.md) | `metrics/facade-recorder` | CBO-013 |
| [miette](miette/README.md) | `miette/derive-forwarding-and-overrides`, `miette/late-bound-source-labeled-rendering` | CBO-038, CBO-045 |
| [minijinja](minijinja/README.md) | `minijinja/runtime-text-templates-strict` | CBO-046 |
| [moka](moka/README.md) | `moka/coalesced-cache-initialization` | CBO-011 |
| [msgspec](msgspec/README.md) | `msgspec/tagged-struct-header`, `msgspec/typed-convert-and-attrs-interop` | CBO-041, CBO-043 |
| [numerica](numerica/README.md) | `numerica/exact-matrix-algebra`, `numerica/ball-enclosures`, `numerica/rational-to-binary64-conversion` | CBO-024, CBO-025 |
| [object-store](object-store/README.md) | `object-store/limit-store`, `object-store/request-concurrency-limit` | CBO-011 |
| [opentelemetry](opentelemetry/README.md) | `opentelemetry/observable-instruments-over-owned-state`, `opentelemetry/sync-instruments-and-provider-order` | CBO-013 |
| [opentelemetry-proto](opentelemetry-proto/README.md) | `opentelemetry-proto/otlp-message-decoding` | CBO-051 |
| [parquet-datafusion-index](parquet-datafusion-index/README.md) | `arrow-file-interchange/derived-index-as-files` | CBO-050 |
| [parser-generators](parser-generators/README.md) | `grammar-generators/declarative-grammar-with-printing` | CBO-037 |
| [petgraph-and-rustworkx-core](petgraph-and-rustworkx-core/README.md) | `petgraph/dag-transitive-reduction-closure`, `rustworkx-core/deterministic-dag-order-and-reachability`, `petgraph/matching`, `petgraph/storage-layout-sizing` | CBO-001, CBO-023 |
| [postcard](postcard/README.md) | `postcard/embedded-serde-blob` | CBO-040 |
| [pounce](pounce/README.md) | `pounce-presolve/structural-decomposition`, `pounce-nlp/fbbt-tape`, `pounce/engine-agnostic-parametric-sensitivity`, `pounce/convex-qp-and-cone-form`, `pounce/tnlp-and-c-abi` | CBO-023, CBO-024, CBO-029, CBO-031, CBO-032 |
| [process-wrap](process-wrap/README.md) | `process-wrap/process-group-command-wrappers` | CBO-004 |
| [procfs](procfs/README.md) | `procfs/typed-proc-readers`, `procfs/hardened-proc-open` | CBO-019 |
| [proptest-derive](proptest-derive/README.md) | `proptest-derive/derived-ast-strategies` | CBO-037 |
| [psutil](psutil/README.md) | `psutil/process-snapshot-and-wait` | CBO-019, CBO-048 |
| [pyo3](pyo3/README.md) | `pyo3/pyclass-as-public-type`, `pyo3/bulk-return-crossing`, `pyo3/interruptible-detached-wait`, `pyo3/per-type-python-handle-cache` | CBO-017, CBO-041, CBO-044 |
| [pyo3-async-runtimes](pyo3-async-runtimes/README.md) | `pyo3-async-runtimes/asyncio-cancellation-bridge` | CBO-017 |
| [python-dbus-clients](python-dbus-clients/README.md) | `python-dbus-clients/pure-python-blocking-client`, `python-dbus-clients/asyncio-client`, `python-dbus-clients/libsystemd-binding` | CBO-020, CBO-048 |
| [python-file-locks](python-file-locks/README.md) | `python-file-locks/flock-with-timeout-poll`, `python-file-locks/alternatives` | CBO-047 |
| [python-frontmatter](python-frontmatter/README.md) | `python-frontmatter/delimited-metadata-split` | CBO-051 |
| [pythonize](pythonize/README.md) | `pythonize/serde-to-python-objects` | CBO-041 |
| [quick-cache](quick-cache/README.md) | `quick-cache/placeholder-guard`, `quick-cache/weighted-native-handle-retention` | CBO-011, CBO-030 |
| [rand-family](rand-family/README.md) | `rand/portable-seeded-streams`, `rand-distr/float-transcendental-portability` | CBO-033, CBO-034 |
| [retained-size-crates](retained-size-crates/README.md) | `retained-size/derived-heap-measure` | CBO-001 |
| [rkyv](rkyv/README.md) | `rkyv/validated-zero-copy-archive` | CBO-040 |
| [rug-and-malachite](rug-and-malachite/README.md) |  | CBO-025 |
| [rust-file-locks](rust-file-locks/README.md) | `rust-file-locks/flock-whole-file` | CBO-003 |
| [rust-std-io](rust-std-io/README.md) | `rust-std-io/typed-error-through-io-channel` | CBO-002 |
| [rustix](rustix/README.md) | `rustix/noreplace-and-anonymous-publication`, `rustix/pidfd-process-handles`, `rustix/flock-and-ofd-locks` | CBO-003, CBO-004 |
| [salsa](salsa/README.md) | `salsa/query-identity-and-labels`, `salsa/memo-retention-observation`, `salsa/persistent-caching` | CBO-001, CBO-022 |
| [sccache](sccache/README.md) | `sccache/compiler-output-cache` | CBO-022 |
| [schemars](schemars/README.md) | `schemars/directional-contract-schemas`, `schemars/flatten-and-closed-object-policy` | CBO-041, CBO-042 |
| [scipy-stats-qmc](scipy-stats-qmc/README.md) | `scipy-qmc/design-engines` | CBO-033 |
| [serde](serde/README.md) | `serde/version-header-envelope`, `serde/selective-ordered-identity-framing` | CBO-039, CBO-042 |
| [signal-hook-registry](signal-hook-registry/README.md) | `signal-hook/chained-native-signal-action` | CBO-017 |
| [sobol-burley](sobol-burley/README.md) | `sobol-burley/seeded-owen-scrambled-sobol` | CBO-033, CBO-034 |
| [sobol-qmc](sobol-qmc/README.md) | `sobol-qmc/unscrambled-sobol-sequence` | CBO-033 |
| [sqlite](sqlite/README.md) | `sqlite/rebuildable-local-derived-index` | CBO-050 |
| [statrs](statrs/README.md) | `statrs/inverse-cdf-quantiles`, `statrs/multivariate-normal-sampling` | CBO-034, CBO-035, CBO-036 |
| [strum](strum/README.md) | `strum/closed-vocabulary-derives` | CBO-045 |
| [suitesparse](suitesparse/README.md) | `suitesparse/structural-permutations`, `suitesparse/spex-exact-ldl`, `klu/symbolic-estimates-and-measured-usage` | CBO-001, CBO-023, CBO-025, CBO-026 |
| [sundials](sundials/README.md) | `sundials/idas-quadrature-history-after-calcic`, `sundials/workspace-reporting`, `sundials/adjoint-checkpoint-introspection`, `sundials/release-fixes-7.2-to-7.9` | CBO-001, CBO-028 |
| [surrealdb](surrealdb/README.md) | `surrealdb-rust-sdk/grpc-request-lifecycle-controls`, `surrealdb-rust-sdk/grpc-live-subscription`, `surrealdb-protocol/generated-grpc-client`, `surrealdb-protocol/conformance-suite`, `surrealql/stored-function-transaction-guard`, `surrealql/single-statement-implicit-transaction`, `surrealql/define-param-constants`, `surrealql/changefeed-replay`, `surrealql/live-select`, `surrealql/request-time-ceiling`, `surrealdb-syn/parse-validate`, `surrealdb-sql/ast-render`, `surrealdb-server/readiness-probes`, `surrealdb-server/logical-export-import`, `surrealdb-server/telemetry-export-transport`, `surrealkit/schema-rollout` | CBO-009, CBO-010, CBO-016, CBO-046, CBO-048 |
| [symbolica](symbolica/README.md) | `symbolica/evaluator-cooperative-abort`, `symbolica/cooperative-resource-budget`, `symbolica/public-size-observables`, `symbolica/evaluator-instruction-interchange`, `symbolica/exact-root-isolation-and-refinement` | CBO-021, CBO-024, CBO-025 |
| [systemd-manager-api](systemd-manager-api/README.md) | `systemd-manager-api/transient-units-and-jobs` | CBO-020, CBO-048 |
| [tempfile](tempfile/README.md) | `tempfile/named-staging-noclobber-persist` | CBO-003 |
| [tokio](tokio/README.md) | `tokio/ordered-stop-race`, `tokio/change-notification`, `tokio/fd-readiness-await`, `tokio/task-local-context`, `tokio-util/cancellation-tree`, `tokio-util/task-supervision`, `tokio-process/async-child-with-pidfd-reaping` | CBO-004, CBO-011, CBO-014, CBO-015, CBO-016, CBO-018 |
| [tracing](tracing/README.md) | `tracing/span-carried-context`, `tracing/future-dispatch-propagation`, `tracing-subscriber/lifecycle-layer-fanout`, `tracing-opentelemetry/event-metrics-layer` | CBO-013, CBO-014 |
| [tryhard](tryhard/README.md) | `tryhard/retry-with-error-aware-backoff` | CBO-010 |
| [typer](typer/README.md) | `typer/click-based-typed-cli` | CBO-052 |
| [uno](uno/README.md) | `uno/highs-qp-subproblem-controls`, `uno/hessian-representation-for-qp`, `uno/c-api-conventions` | CBO-028, CBO-029 |
| [winnow](winnow/README.md) | `winnow/token-stream-combinators`, `winnow/located-text-lexing` | CBO-037 |
| [zbus-systemd](zbus-systemd/README.md) | `zbus-systemd/typed-systemd-manager-client` | CBO-004, CBO-020 |

