# Efficiency-alignment codebase review: supporting evidence

The [principal review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md)
owns the combined architectural judgment, the finding identifiers (`F01`…), the rule impacts
(`RC01`…), the decision and the evidence limits. The documents here are bounded supporting
assessments and coordinator evidence. They are not implementation plans, competing verdicts or
finding-status ledgers. Recommendations remain *Proposed* until dispositioned through plan creation.

**Baseline.** `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11). Production
source was clean; the only dirty tracked file was a Plan 28e documentation edit. No production
file was changed by the review.

**Standard.** Core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and the
[pse-arrow binding](../../design_principles/binding/pse-arrow.md), selected by
[`standard.toml`](../../design_principles/standard.toml).

**Mode.** The maintainer selected a full re-assessment: earlier reviews and plan dispositions
were leads and contrary evidence, not conclusions. Scope covered the product (all crates, the
Python package, native and SurrealDB integration) and the build, test and development loop;
documentation tooling was excluded. Evidence was static source and contract inspection plus
light, read-only probes; no campaign, service, heavy native run or benchmark was started because
a Plan 28 campaign was waiting for exclusive host capacity.

## How the review was conducted

1. A read-only `code-mapper` traced three warm complete operations at function granularity
   ([operation map](operation-map.md)): value edit → re-solve from Python, durable study point
   N+1, and reopening a result subset into Python. Every inquiry consumed it.
2. Ten independent read-only inquiries ran in parallel with a common brief (standard, functional
   target, grounding rule, workload premises, report format) and inquiry-specific revealing
   questions and prior leads ([inquiry briefs](inquiry-briefs.md)):

   | Inquiry | Supporting assessment | Role |
   |---|---|---|
   | A | [Authoring, admission and incremental preparation](authoring-admission-and-incremental-preparation.md) | `design-reviewer` |
   | B | [Mathematical compilation, evaluation and properties](mathematical-compilation-and-evaluation.md) | `design-reviewer` |
   | C | [Single-solve native route](single-solve-native-route.md) | `design-reviewer` |
   | D | [Multi-case workflows and shared capacity](multi-case-workflows-and-capacity.md) | `design-reviewer` |
   | E | [Publication, persistence and results serving](publication-persistence-and-results.md) | `design-reviewer` |
   | F | [Relational and columnar layer](relational-and-columnar-layer.md) | `design-reviewer` |
   | G | [Build, test and development loop](build-test-and-development-loop.md) | `design-reviewer` |
   | H | [Assurance, identity and observation](assurance-identity-and-observation.md) (cross-cutting) | `design-reviewer` |
   | L1 | [Numerical, symbolic and scientific library utilization](numerical-library-utilization.md) | `library-research` |
   | L2 | [Data, persistence, boundary and infrastructure library utilization](data-and-infrastructure-library-utilization.md) | `library-research` |

   Library questions used the library skills, Context7, GitHub search of upstream repositories
   and pinned upstream source. The stale `docs/library-utilization.jsonl`, the library-catalog
   MCP and `docs/capability-maps/` were deliberately not used.
3. The coordinator read the decisive source for each consequential candidate (below), ran light
   probes, reconciled duplicates and disagreements, and commissioned a fresh principal
   `design-reviewer` that had not taken part in any inquiry. The principal independently re-read
   decisive evidence and wrote the principal review.

Local labels in the supporting assessments (`A-01`, `L2-03`, …) are investigation labels only.

## Coordinator source checks

Each check was performed by reading the cited source at the baseline. *Interface-checked*
unless stated.

| Check | Subject | Result |
|---|---|---|
| V1 | `pse-operations/src/canonical.rs::CanonicalStore::ensure_writes` → `WriteAdmission::read` | Confirmed: `config.json` read and decoded on every mutating call |
| V2 | Operation map claim "no index on settled/assigned" | Refuted: generated schema defines `study_candidates (study, settled, assigned, ordinal)` |
| V3/V12 | `pse-runtime/src/workflow/worker.rs::work_once_scoped` candidate handling | Confirmed: one `study_scope` RPC per candidate; `Wait(_)`/`Reconcile` → `continue`, so blocked candidates are revisited on each call |
| V4 | Two `prepare` calls per declared-case solve (`modeling/declared.rs::declared_execution`, `modeling/cases.rs::resolve_case`) | Confirmed |
| V5/V20 | DataFusion file-cache/IO machinery and default reservation | Confirmed no production caller of `bind_state`, `ListingTable`, `register_parquet`, `FileScanConfig`, `ParquetSource`; `resources.rs` uses `NativeCacheService::unbound_config()`; `Envelope::new` reserves upfront; Python `RuntimeSettings(cache=None)` → `CacheBudget::for_memory` (metadata ≤50 MiB, statistics ≤16 MiB). An explicitly constructed `CacheSettings` defaults those fields to zero |
| V6 | Retention guard | Confirmed: protected reads take `canonical_guards:'retention:'+problem` FOR UPDATE; edits/admissions bump its generation; `GUARDED_ATTEMPTS = 32` |
| V7 | `pse-backend-native/src/assembled.rs::AlgebraicOracle::trial`, `pse-math/src/binding.rs::values_selected` | Confirmed per-callback `BTreeMap` insertion and per-instance `BTreeSet`/lookup/`Vec` allocation |
| V8 | `pse-runtime/src/workflow/durable.rs::Tap::observe` | Confirmed: full channel blocks the native thread up to the heartbeat wait, then cancels the run; `Streamer::finish` reports "scientific run cancelled" |
| V9 | `pse-runtime/src/math.rs::MathService::workspace` | Confirmed: fresh `CompilerWorkspace` (new Salsa storage) per call; each publishes one revision and is dropped |
| V10 | Python native surface | Confirmed: `with_declarations(source)` complete replacement; no binding/overlay single-case solve entry; `bindings.rs::admit_overlay` used by study and initialization paths |
| V11 | `pse-buildinfo/identity.rs::LocalRuntimeObservation::capture_scoped`; `pse-py/src/workflow.rs::NativeRuntime::new` | Confirmed: each executable-mapped ELF hashed, then every loaded file re-verified (second pass); `observe_deployment` runs outside `py.detach` |
| V13 | Generated `fn::pse_execution_v1::writer` / `study_cancelled` | Confirmed: every writer-guarded execution mutation of a study point touches `study:<id>` |
| V14 | `pse-engine/src/session/factory.rs::EngineFactory` | Confirmed mechanism: `models` holds `Weak<ModelAssembly>`; the validation context lives in the assembly's `OnceLock`. Absence of a long-lived strong holder in production is the inquiry's search result; the rebuild frequency remains probe-pending |
| V15 | `justfile` `py-sync`, `py-sync-native` | Confirmed: editable extension built with `--features force-validate`, dev profile by default |
| V16 | `canonical_result_cells` | Confirmed five indexes; `output_results` is the only consumer (public API chain) |
| V17 | `pse-math/src/execution.rs::evaluate_stages` Domain/Applicability arms | Confirmed full-frame slot copy before each nested stage |
| V18/V19 | `pse-math/src/implicit_cubic.rs::CubicRoots`; `implicit_accelerators.rs::Cubic::admit`; `implicit.rs` factory | Confirmed: one `Mutex<ExpressionEvaluator>` inside an `Arc` cloned into every attempt's factory |
| V21 | `scripts/pse_env.py` main `finally` | Confirmed: every top-level `pse-env` exit runs `test_resources.reclaim_reports()` |

## Reconciliation

**Same defect, several inquiries.** The principal review merges these: per-callback semantic-map
binding (C-01, B-01, with B-03/H-04 adjacent); per-attempt evaluator clones (B-08, L1-03); cubic EOS
exact isolation and shared mutex (B-05, L1-04); dense root response (C-03, L1-01); faer global
parallelism (D-08, L1-02); factor-reuse gaps (C-08, L1-05); worker-executable re-hashing (H-02,
E-06); retention/study guard and local write pacing (L2-02, D-02, E-02, A-07); surrogate dependency
closure (L1-07, G-06); dead file-cache machinery (L2-01, F-04 in part); second `prepare` (A-04, H,
D-04); whole-runtime re-observation (H-01, H-03, A-03 trigger).

**Disagreements resolved.** The operation-map index claim is refuted (V2). F's statement that file
cache envelopes default to zero holds only for an explicitly constructed `CacheSettings` object;
the default runtime reserves them (V20). E-02 and D-02 are consistent: local read coordination was
narrowed by effect, while server-side guard domains remain problem- and study-wide (V6, V13).

**Remedy interactions passed to the principal.** The coordination-domain remedies (split guard
keys; register conflicts on the rare side; pace by written guard; carry the problem in the fence)
must form one design to keep phantom and write-skew protection under SurrealDB 3.3 (no shared lock;
`FOR UPDATE` on record identifiers only). The value-binding route (A-01) and study-scoped
preparation (D-04) share the binding/overlay mechanism. Validity carriage (H-01/H-02, A-03) changes
the documented meaning of "current admission". Publication shape (E-01) interacts with empty
relations (E-04), receipt reclamation (E-07), selected scalar verification (Plan 33 EI02) and a
possible DataFusion result provider (F-04). Inner-loop remedies (C-01/B-01, B-03, B-08, H-04) share
one owner and must preserve the fresh PS-10 original-case worker.

## Where inquiry candidates landed

| Principal finding | Inquiry candidates |
|---|---|
| [F01](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f01) | A-01 |
| [F02](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f02) | A-02, L2-08 |
| [F03](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f03) | A-04, A-06 |
| [F04](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f04) | A-03, H-05 |
| [F05](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f05) | H-01, H-02, H-03, E-06, G-05 |
| [F06](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f06) | D-04, A-07 |
| [F07](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f07) | A-05, L1-06 |
| [F08](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f08) | L2-02, D-02, E-02, E-03 (fence) |
| [F09](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f09) | D-01, D-09 |
| [F10](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f10) | D-03, D-06, D-08, B-10, L1-02, F-03 |
| [F11](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f11) | E-01, E-04 |
| [F12](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f12) | E-03, E-05 |
| [F13](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f13) | D-05, D-07, C-04 |
| [F14](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f14) | C-07, H-04 (events) |
| [F15](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f15) | E-07 |
| [F16](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f16) | C-01, B-01, B-02, B-03, B-04, H-04 |
| [F17](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f17) | B-07, B-08, L1-03 |
| [F18](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f18) | B-05, B-06, L1-04 |
| [F19](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f19) | C-02, C-06 |
| [F20](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f20) | C-03, L1-01 |
| [F21](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f21) | C-05 |
| [F22](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f22) | B-09 |
| [F23](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f23) | F-01, F-02, F-06, L2-03 |
| [F24](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f24) | F-04, L2-01, L2-05 |
| [F25](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f25) | G-01, G-08 |
| [F26](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f26) | G-02 |
| [F27](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f27) | G-03, G-06, L1-07 |
| [F28](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f28) | G-04 |
| [F29](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md#f29) | L2-07 |

Candidates not raised as findings remain opportunities or uncertainties in the principal review
(for example C-08/L1-05 factor reuse, L2-04 vendored SDK patch, F-05 single-slot physical cache,
G-07 build-script invalidation, L1 SIMD/JIT and FeOS notes). The principal review's §9.3 records
positions on prior reviews' findings.

## Probes executed

All read-only. Conditions: 2026-10-11, shared host with 32 logical CPUs (SHA-NI present), about
188 GiB RAM, warm page cache, other repository work possible.

- **M1 — SHA-256 pass time (*Measured*).** `scripts/pse-env --resource-class light -- python3 -I
  docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/sha256-pass-time.py
  target/debug/pse-worker python/pse/_native.abi3.so`: 0.295–0.387 s per pass over the 726,959,592-byte
  dev worker, 0.317–0.324 s over the 785,556,497-byte extension
  ([script](sha256-pass-time.py), [output](sha256-pass-time.txt)). Python `hashlib` (OpenSSL); the
  product uses Rust `sha2` 0.11 with its x86 SHA backend, not measured separately.
- **S1 — physical packages (static).** Only `packages/reference/physical` and
  `tests/fixtures/packages/physical-primitives` exist.
- **G-P1 — ledger cycle (*Measured*).** `python3 -I` parse plus sorted dump of the 1,710,027-byte
  `~/.local/state/pse-arrow/test-resources/allocations.json`, 20 iterations: 9.3 ms per cycle.
  Excludes validation, lock and write. Run outside `scripts/pse-env` deliberately, because its exit
  path rewrites this ledger.
- **G-P2 — bridge imports (*Measured*).** `.venv/bin/python -X importtime`: `scripts.test_resources`
  17.0 ms cumulative, `scripts.surreal_server` 12.6 ms cumulative, excluding interpreter start.
- **G-P3 — native generation verification (*Measured*).** `nice -n 19 find <newest solver
  generation> -type f -print0 | xargs -0 cat | sha256sum`: 0.71 s and 0.58 s over 1.2 GB in 2,305
  files.
- **Census (*Measured*, `ls`/`wc`).** 639 invocation files under
  `~/.local/state/pse-arrow/test-resources/invocations`; 1,190 entries under
  `~/.cache/pse-arrow/native/.operations`.

**Not run.** Probes that need builds, services, store fixtures, profiler privileges or a quiet host
remain proposals in the supporting assessments: callback-path profiles (B, C, H), validation-context
identity (F), SurrealDB `FOR UPDATE` reader semantics and discovery counters (D), result-store
queries (E), edit-path counts (A), strace observation counts (H), library timings (L1), cache
reservation and membership `EXPLAIN` (L2), build-script invalidation and CPU-quota effects (G).
Their absence leaves magnitudes unmeasured; it does not undermine diagnoses established from source.

## Coverage limits

Read shallowly or not at all: IBEX root isolation, PETSc/SCIP/HiGHS/Uno/Clarabel adapter internals,
`kkt/path/*` continuation, IDAS adjoint routes, dynamics trajectory and shooting encoders,
`pse-math` `derived.rs`, `factorable.rs`, `collocation.rs`, `surrogate.rs` and regime proof
transport, `pse-authoring` parser performance, `pse-quantity` inference, `pse-codegen` internals,
`pse-engine` session internals (`round`, `traversal`, `cache/*`, `operation/*`, `provider/*`),
`python/pse` beyond its boundary modules, `scripts/surreal_server.py` beyond ensure-primary and
readiness, `scripts/tests/` and `tests/`, teqp and the parity route. No end-to-end measurement was
taken.
