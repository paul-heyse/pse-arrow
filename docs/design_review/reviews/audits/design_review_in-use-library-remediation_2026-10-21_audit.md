# Recommendation audit: in-use-library remediation

This companion records the independent recommendation auditor's judgment, dated **2026-10-21**.
The [integrated principal review](../design_review_in-use-library-remediation_2026-10-21.md)
is the complete resulting design argument for planning. The original reviewer did not author
or approve this revised judgment. Dispositions below assess recommendations; they are not
implementation status, plan acceptance or rule confirmation.

| Basis | Exact scope |
|---|---|
| Input principal | `docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md` at **`e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`**; recoverable with `git show <revision>:<path>` |
| Product source | **`a791c533eca0f65b195895f071e60cd1e38e121c`**, unchanged from the original assessment at audit start; later evidence/document publication is not a product-source change |
| Diagnostic input | Publication **`0530dcaa1`**, source `a791c533e`; DA-001–DA-032 retain their stable identifiers |
| Criteria | Core 3.4, Efficient Architecture heuristics 1.0, Process Simulator 1.5, `pse-arrow` binding and the profile/blueprint functional target |
| Inherited focus | In-use pinned direct libraries, all features and resolved siblings; improve existing capabilities; minor clear wins only; no new third-party library and no remedy that regresses a preserved guarantee |
| Fixed premises | P1 canonical SurrealDB and 16-lane coordination; P2 caller-independent durable/multi-process work without a daemon; P3 tactical payloads subordinate to canonical state/commit/lifecycle; P4 integration hooks are legitimate, numerical hunks assessed numerically |
| Evidence | Independent source-led reassessment, bounded store/result and preparation research, and the coordinator's [selective codec probe and source evidence](../../evidence/in-use-library-remediation-2026-10-21/recommendation-audit/README.md) |
| Effects | Review/evidence publication only; no production source repair, build, solver or store-service run. No recommendation automatically starts implementation |

The original review's challenge section and all original dated test/measurement receipts remain
in the integrated principal as historical evidence. The new judgment does not promote those
receipts to fresh qualification. This is the same review engagement; product source and selected principle versions did
not change. Intervening workflow/binding additions are applied through the current audit skill
and publication route; they do not change the simulator target. The substantive changes below
correct or improve recommendations rather than respond to changed product circumstances.

## Result and combined direction

**Overall: Revise**, unchanged. All seven foundation verdicts and G9 remain violated in the
examined scope; G8 remains failed. Behavioural gates G1, G2, G3, G5, G7 and PS-G3 fail, while
G4, G6, PS-G1 and PS-G2 remain unresolved. F10 adds a source-derived covariance-validity
failure; it does not broaden the audit into full scientific qualification.

The original high-level direction is sound: keep physics and native mathematics with their
owners, move policy out of adapters/hosts, make canonical lifecycle meaning explicit, and remove
incidental work. Several concrete remedies needed correction to preserve precisely those
guarantees. The largest changes are the feasible Salsa retention policy, chain-correct regional
derivatives, full existing result-selection semantics, actual covariance content/PSD validity,
and separation of product contracts before toolkit extraction.

No new third-party dependency is proposed. No original R identifier is removed or reused.
Additional conditions remain with their original recommendations rather than creating a parallel
implementation registry. New finding **F10** and rule impact **RC18** extend the existing conventions.

## Complete R1–R22 accounting

| Recommendation | Disposition | Result in integrated principal |
|---|---|---|
| R1 typed store lifecycle | **Improve** | Retain generated literal/tagged types, verified immutable `READONLY` fields and transition backstops. Model closed-before-manifest and u64-compatible generation; distinguish physical ingestion phases from observational `AttemptState`/`StudyPointState` rather than retargeting them |
| R2 typed refusal codes | **Improve** | Retain code-prefixed `THROW`, native `is_thrown()` discrimination, one decoder, typed conflict exhaustion and downstream consumers. Preserve fresh-premise reread, exact acknowledged cancellation, operation stage and uncertain effects; class alone gives no replay permission; unknown codes fail closed |
| R3 keys and adjacency | **Retain** | Remains a candidate. Migrate key readers/paging before dropping indexes; test write-form assertion and RocksDB access plan; traversal ordering/paging and endpoint existence remain bounded conditions, not proof of race-safe lifetime |
| R4 record-batch pruning/locator retirement | **Correct or adjust** | Retain native pruning and removing per-row locator work. Correct conditional finite predicate and all four relation key shapes; preserve authoritative row coordinates, full surviving-batch admission, residual-before-limit, filter columns, statistics provenance and protected single decoder. Projected decode is a separate candidate |
| R5 resolved identities | **Improve** | Retain canonical identity-aware specialization/body framing and catalogued derivation frames. Preserve local positional sharing/physical/guard contracts; explicitly evolve currently VersionOnly expression metadata. Borrowed identity-aware lookup is the simplest candidate; `rpds` persistence is conditional on lifetime needs |
| R6 generated handles/membership | **Retain** | Existing `RELATION_ID`/`NAME` and registry artifact-profile mechanisms are suitable owners. Extend declarations for needed artifact kinds and remove hand lists; external unknown relations still need runtime refusal |
| R7 regional joint products | **Improve** | Retain demanded guarded regional fusion and Symbolica derivative/evaluator mechanisms. Preserve provider/branch/intermediate-slot chain terms using existing Taylor/reverse propagation or composed local jets/actions. Deterministic preflight is new required design; transient capacity/cancellation remains refusal, not route selection |
| R8 bounded retention/catalog firewall | **Correct or adjust** | Granular LRU already exists. Public Salsa has aggregate count/bytes, no resident-value count, and count-only LRU. Bound actual metadata/input/interned/value bytes, tune per-query capacities/live set, retain shared census until a sound callback migration, and distinguish default tuning from a general retention policy |
| R9 indexed preparation scans | **Correct or adjust** | Existing graph points dependency→consumer and omits non-expression rates/free inputs. Build complete dependency index and traverse the correct direction. Local lookup indexes can precede R5; numeric and guard dependencies remain distinct |
| R10 supervisor/cancellation tree | **Retain** | One run skeleton, child tokens and drain-safe native bridge remain appropriate. Keep independent durable worker cancellation ownership and explicit-strategy exception; F01's inherited-deadline acceptance remains open |
| R11 operation/preparation ownership | **Improve** | Share common requests/preparation and runtime composition. Host-only Shooting needs an explicit in-process superset and typed projection/refusal; adding it to serialized `OperationRequest` changes persisted meaning and needs version evolution. Retain paired outcomes, declared dispatch, deployment/admission owner and validated constructors |
| R12 parameter-valued dynamic/fit binding | **Retain** | Bounded addition completes existing study capability. Simulation needs parameter-binding preparation; declared/recorded starts and fresh consistent ICs stay mandatory. Other point dimensions and retained IDAS history remain leads |
| R13 start/annotation/view policy owners | **Retain** | Pure predicted-start eligibility/fallback, compiler-owned per-mode lowering and view selection remain sound. Deliberate per-mode differences and new-state scientific checks stay explicit; Salsa hosts rather than defines policy |
| R14 capability/seam compositions | **Retain** | Settings-dependent capability, map-specific fixed point/Picard route, original residual qualification, fresh sessions, authorized starts and adapter recovery profiles remain sound. Path preconditions stay with path owner; composition-only PETSc methods remain explicitly limited |
| R15 Uno mapping/binding | **Retain** | Correct small-step stall to Numerical, use named constants/static assertions, declare four numerical patch hunks and dense SQP scope. Preserve integration hooks and serial controls; replacing numerics without losing SQP-LBFGS remains remedy open |
| R16 native memory candidates | **Improve** | Retain FERAL a-priori pre-admission plus post-factor envelope. PETSc/KLU hooks cover allocator domains, not all foreign allocations; retain original allocation owner/size across free/realloc and process initialization, disclose bypasses, and do not replace full admission prematurely |
| R17 enum/identity boundary | **Improve** | Retain strict generated enum inputs, `INPUT_TYPE`, identity keyword/NewTypes, codec hooks and per-type `PyOnceLock`. Selective pinned probe resolves bytes-subclass hook uncertainty in favor of current representation; NewType gives static roles, not runtime role evidence |
| R18 physical Arrow boundary | **Correct or adjust** | Retain keyed Arrow relation inputs/outputs and extension-metadata acceptance. Omit blanket seconds→timedelta conversion: signed/precision-bearing physical coordinates must retain meaning; Duration/timedelta only where budget range/precision fit |
| R19 document catalogue/schema identity | **Improve** | Retain one catalogue, typed document returns and compiled schema identity. Use separate serialize/deserialize generator contexts and include direction in identity. Feature variation must be demonstrated on actual schema roots; non-JsonSchema Shooting enums do not establish it |
| R20 covariance provenance | **Improve** | Run-id resolution alone is insufficient: hydrate actual qualified covariance or compare complete ordered contents and validity/dependencies. External origin stays explicit; add mathematical PSD admission preserving singular valid covariance, using in-use numerical mechanisms (F10, RC18) |
| R21 tooling consumes owners | **Improve** | Retain Rust conformance/launch owners and generated bounded supervisor maintenance functions. Preserve privileged/version-specific historical restore/bootstrap/activation, page replay and frozen-placement gates; do not claim all maintenance is ordinary current-writer lifecycle |
| R22 toolkit placement/tracing | **Improve** | Separate product-owned detail/observation and canonical selection contracts before moving capture/export/analysis to tooling. Whole-module crate extraction would otherwise cause runtime↔tooling dependency. Crate remains conditional; tracing Layer stays a cost/semantic candidate |

## Why the material changes matter

### Store lifecycle and typed effects

The original R1 example required a manifest as soon as an attempt closed. The generator's
`close`, `cancel` and `recover` functions close ingestion before `manifest` attaches its
descriptor (`pse-codegen/src/codegen/surreal_execution.rs`). Refusing that legitimate gap would
break recovery and cancellation. `close_generation:int` would also narrow the existing UInt64
decimal domain. Observational attempt vocabulary describes queued/running/stale/superseded
science, not the physical ingestion phase; generate an explicit projection rather than merging
different meanings.

`CanonicalError`'s collapsed classification and retry-by-prose still justify R2. A classified
refusal does not establish that an interrupted mutation had no effects, nor that cancellation
was acknowledged by the server. The integrated recommendation keeps those distinctions and
fresh-premise rereads. R21 likewise keeps the special maintenance authority needed to install
definitions and activate restored historical state before an ordinary writer can exist.

### Result selection and engine visibility

Production `scalar_predicate` retains **all rows** when missing/minimum/maximum are absent;
the unconditional finite guard in the input prescription would discard null/NaN/infinity.
The review's 63/63 standalone receipt does not overrule that source contract. Selection keys
also differ: symbol/step, row/step, parameter, and observation/experiment. Sorting must occur
before row coordinates become authoritative, or preserve a mapping for recorded portable
selections (`result_projection.rs`, `connected_results.rs`).

The protected decoder currently checks the full relation schema and admits a complete
`FieldCheckedBatch` (`result_snapshot.rs`, `ProtectedResultDecode::decode`). Arrow projection
cannot simply bypass it. Selected first step: prune complete record batches, decode/admit all
required surviving columns, then apply output projection. A narrower trust boundary is a
separate candidate. `Inexact` pushdown retains engine filtering before limit fulfillment;
predicate columns remain available even when omitted from output. Statistics must come from
the admitted batch and bind to its digest/schema/interpretation; replay equality only checks
that metadata did not change.

An omitted native capability is now retained conditionally: DataFusion 55.1
`PruningStatistics::contained` can supply literal guarantees to reject absent ids if an exact
existing owner summary is cheap to reuse. `PrunableStatistics` returns unknown. This does not
justify recreating a membership index or unbounded owner sets. Parquet remains an alternative
with page/bloom mechanisms, rather than a categorical rejection from float-statistic labels.

### Regional mathematics, identities and retention

`SelectedFirstRegion` currently requires scalar-input-only regions; downstream provider/branch
slots have derivatives that must propagate (`pse-math/src/execution.rs`). Symbolically
differentiating only with respect to original inputs while treating those slots as parameters
loses chain terms. The integrated remedy preserves existing propagation unless a local
jet/action composition is established, with mixed-second-order acceptance cases.

Current derivative allowance is decremented inside construction, and `construction::additional`
is a transient runtime capacity request. Neither is a deterministic up-front route selector.
The proposal now names that missing preflight. Cancellation/capacity failure cannot silently
choose a different arithmetic artifact. Alias fusion remains an evaluator candidate, not
proof that derivative calls expand aliases; a cancelled truncated optimizer result is never
retained.

R5 keeps canonical local-coordinate roles rather than hashing every instance's global symbol
id into a shared body. Its expression metadata needs new grammar/role fields and explicit
version evolution. The in-use containers are realization options, not independent semantic
authorities. R9's local indexes do not need to wait for the whole representation redesign.

Salsa 0.28.4 already applies LRU to granular queries. `memory_usage` retains evicted memo counts
and exposes aggregate metrics, not value presence; LRU capacity is count-based. R8 now accounts
for real metadata/input/interned state plus values and uses finite live-root/backdating
allowances. The existing shared `RetainedInputs` census deduplicates Arc roots and generation
state; independent `heap_size` callbacks can double-count or omit those roots. They remain a
conditional migration. New defaults must fit the admitted workload under the enclosing budget;
the original ≈606 MB estimate is not resident memory or proof of a new policy.

Persistence was not selected, but P3 alone is not a reason to reject a discardable,
premise-validated derived cache subordinate to canonical state. Serialization, invalidation,
versioning and retention complexity still need a concrete benefit.

### Host, numerical-memory and toolkit boundaries

R11's common request/preparation semantics stay runtime-owned, while host-only operations use
an explicit superset. `OperationRequest` is serialized inside Version<7> `StudyOperation`;
its persisted meaning cannot grow invisibly. R19 similarly uses separate schemars 1.2.2
serialize/deserialize generator contexts chosen through `SchemaSettings` before construction,
and fingerprints actual compiled roots/directions rather than inferring schemas from debug-only
feature-gated enums.

The selective msgspec 0.21.1 probe covers direct/nested hook dispatch, hex round trips,
width rejection and NewType erasure. It resolves R17's replacement-class question; it is not
an extension/generated-code integration run. PyO3 0.29.2 Duration conversion rejects negative
timedeltas and exports microseconds, so R18 keeps Float64 physical time semantics and treats
budget conversion separately.

PETSc 3.24.0 allocator hooks intercept a named allocator domain; initialization can outlive
attempts, and current PETSc calls already serialize through a process owner. A later free or
realloc must charge the original allocation's owner. Thread-local current-owner lookup alone
does not preserve that lifetime, and MKL/BLAS bypass the hook. R16 keeps the full foreign
admission envelope until coverage and attribution are established.

Product workflows use `capture::DetailSession`, and canonical selection APIs use
`review_pack::PackResultSelection`. R22 therefore first returns those meanings to product
owners, then lets tooling depend on them. A crate move alone would create an inversion/cycle;
a module split is the simpler alternative where it earns the same locality.

## Additional finding and enabling mechanism

**F10 — covariance admission accepts an indefinite matrix.** `ParameterCovariance::admit`
(`pse-runtime/src/math/settings.rs`) checks finite values, extent, symmetry and nonnegative
diagonal, but not PSD. `propagate_checked` computes `JΣJᵀ`, rejects nonfinite output and copies
the supplied run id (`workflow/uncertainty.rs`). The matrix `[[1,2],[2,1]]` passes the stated
admission while `J=[1,-1]` yields variance `-2`. This is **Implemented/source-derived** evidence,
not an executed Rust regression test. It extends DA-010/R20; a genuine id with altered matrix
contents is also insufficient provenance.

R20 now hydrates or fully reconciles qualified contents and validity, and separately admits
external PSD covariance with honest upstream assumptions. Pinned **faer 0.24.4** exposes
`self_adjoint_eigenvalues(Side)` (`src/linalg/solvers.rs`), returning sorted eigenvalues or
`EvdError` without eigenvectors. This is an **Interface-checked** in-use enabler. Its square and
self-adjoint preconditions, declared triangle, scale-aware PSD tolerance, resource/parallelism
admission and failure withholding remain **Proposed** integration design. Singular valid PSD
matrices stay accepted; a positive-definite-only Cholesky requirement would be regressive.

**RC18** records the resulting §19.8 input/validity contract change. All existing RC01–RC17
retain identity, with corrected realization details. RC01 includes canonical framing/metadata
evolution; RC02 chain-preserving regional products and deterministic preflight; RC03 actual
retention accounting; RC04 all selection semantics; RC09 dependency direction before tooling
extraction; RC12 explicit persisted-set projection; RC14 faithful physical times; RC17 privileged
maintenance lifetime. Rule changes await operator confirmation in plan creation.

## Findings, clear wins, candidates and prose recommendations

This table accounts for recommendation-bearing findings outside or in addition to R1–R22.

| Original reference | Disposition | Result |
|---|---|---|
| F01 child-token immediate fix and deadline/drain controls | **Retain** | Original probe C stays historical; inherited deadline and native drain remain implementing acceptance |
| F02 mapping/constants | **Retain** | R15 owns correction; original synthetic mapping receipt does not qualify native execution |
| F03 typed exhaustion and serialization investigation | **Correct or adjust** | Keep typed exhaustion through R2. Commit-order sequence serialization is necessary; broad local mutation pacing has an unresolved avoidable effect. User-facing sequence draws have a durable cursor path, yet no caller commit-order guarantee; do not rely solely on internal batch-allocation comments |
| F04 retained surrogate pool join | **Retain** | Join actual retained worker handles after pool drop before releasing charge; scoped pool alternative retained |
| F04 eager global-pool pin | **Unresolved** | Establish a reached global-pool consumer first. An embedded process may already own initialization; per-deployment eager `build_global` cannot promise resource ownership and can itself add threads |
| F05 declared LM gtol/ftol/xtol | **Retain** | Remains a candidate: cosine tolerance differs from scaled stationarity; retain independent qualification/no newly unqualified fits or withheld covariance acceptance |
| F06 bind/remove disabled native cache adapters | **Improve** | Bind source-specific scan state with actual lifecycle generation, not a shared session merely because an ObjectStore Arc is stable. Establish a reached DataFusion file source; ProtectedFile IPC bypasses these caches. Remove only unused native-file adapters if chosen, retaining active preparation/analysis service |
| F07 assertion/index removal | **Retain** | R3 candidate and its reader/engine-plan prerequisites |
| F08 statrs pin reason | **Retain** | Correct stale reason; resolved nalgebra split alone is not a crossing-type defect. No family upgrade/new dependency proposed |
| F09 defaults/reuse | **Correct or adjust** | Original measured phenomenon retained, R8 mechanism corrected; acceptance includes finite churn/oversized live set, not only one reuse count |
| §6 child_token and Uno constants | **Retain** | Clear bounded fixes, same F01/F02 controls |
| §6 SUNDIALS error handlers | **Retain** | Clear first, then push; handlers chain and default stderr remains otherwise. Bounded owned text/FFI callback handling stays integration work |
| §6 Salsa input heap_size as automatic census replacement | **Correct or adjust** | Move to conditional candidate pending shared-root/generation accounting; retain callbacks as an available mechanism |
| §6 semantic_body LRU capacity | **Improve** | Cover the admitted live root with a finite allowance and prevent routine rotation; more retention has a memory tradeoff, not an unconditional free win |
| §6 dependency ENFORCED | **Retain** | Endpoint-existence backstop, retaining race-safe guards |
| §6 rayon spawn_handler | **Retain** | Only retained custom-pool drain; no claim that terminate alone joins |
| §6 per-type PyOnceLock | **Retain** | Macro emits distinct type cells; generic function static must not become one cell shared across T |

| Original candidate/prose reference | Disposition | Result |
|---|---|---|
| R1 literal-only first step | **Retain** | Simpler partial correction, explicitly weaker than state-combination constraints |
| Reference-delete rejection replacing guards | **Retain** | Rejection retained on original race receipt; typed record holders can still help typing |
| Asynchronous readiness events / synchronous fan-out | **Retain** | No asynchronous loss-prone replacement; synchronous contention tradeoff unqualified |
| DEFINE SEQUENCE drop-in | **Correct or adjust** | Still not selected; actual caller commit-order/u64 contract is decisive, not blanket batch semantics |
| SDK gRPC replacement | **Retain** | Not selected without equivalent bounded admission/deadline/replay/response behavior; no generic transport speed claim |
| R4 lowered-predicate-only partial step | **Improve** | Preserve all predicate modes and Inexact/limit semantics; still leaves locator writes |
| Array row ids/range scans | **Retain** | Secondary alternative retaining per-row writes, no selected benefit established |
| Parquet page/bloom/AccessPlan | **Correct or adjust** | Not selected for immediate locator remedy; preserve native opportunity, conditional IEEE/pruning/format transition rather than categorical impossibility |
| `PruningStatistics::contained` | **Improve** | Added opportunity: a cheap existing exact owner summary may enable absent-id pruning; no new membership index by default |
| Function-shaped Symbolica formals | **Retain** | Post-admission candidate; cross-process term order, derivatives and compiled export unresolved |
| Reserved authored `s_` prefix | **Retain** | Stopgap collision guard only; not chosen in place of resolved semantics |
| Whole-group Dualizer / merge | **Retain** | Not selected on original fixture operation counts; no universal claim about all workloads |
| Alias/direct-translation fusion | **Improve** | Keep candidate with host barriers, derivative-chain expansion and source attribution explicit; direct translation already default |
| Cancelled truncated evaluator retention | **Retain** | Must not retain interruption-dependent optimization as deterministic artifact |
| Per-instance tracked specialization | **Retain** | Larger measured-need candidate; root closure from original terms remains |
| Same-revision Salsa storage sharing | **Retain** | Candidate only within one revision lifetime; mutable head cannot cancel pinned durable lanes |
| Cross-lane handwritten keys | **Retain** | Still a G6 audit item; no stale reuse claimed without evidence |
| quick_cache pure-flight coalescing | **Retain** | Candidate for pure preparation, not replacement for detached native-drain completion |
| Salsa persistence exclusion | **Correct or adjust** | P3 does not forbid reconstructible subordinate caches; no benefit established to justify serde/version/invalidation/retention complexity |
| Explicit strategy supervisor inclusion | **Retain** | Maintainer decision; document existing exception rather than add durable functionality automatically |
| Shooting as study operation | **Improve** | Explicit decision/version evolution; host-only superset is separate from persisted admission |
| Non-parameter study values and IDAReInit/IDASensReInit | **Retain** | Leads; new history must be identical to fresh or a recorded dependency |
| Uno upstreaming / different IPM preset | **Retain** | Upstreaming process lead; IPM is major-method addition outside brief |
| PETSc alternate fixed point / NASM / pseudo-time | **Retain** | No demonstrated benefit for second fixed-point path; composition-only limits explicit |
| FERAL work_estimate | **Retain** | Candidate pre-admission, not exact runtime peak; post-factor enforcement stays |
| PETSc/KLU allocator hooks | **Improve** | Domain coverage, original owner and initialization lifetime are preconditions; bypasses retain full envelope |
| Arrow pool features / RecordBatch::claim | **Retain** | Candidate: real buffer accounting versus process-wide reservation-slot overhead, unmeasured |
| KINSOL neutral reset | **Retain** | Candidate only after fresh-session identity or recorded warm dependency |
| LIVE SELECT wake-ups | **Retain** | Candidate with polling backstop; verifier limitations retained |
| Symbolica hot_start | **Retain** | Candidate with previous scheme in identity, incompatible with direct translation |
| msgspec alternative identity value class | **Omit** | Pinned selective probe shows no replacement is needed for hooks; keep current bytes-subclass representation |
| Blanket seconds→timedelta | **Omit** | Narrowing signed/precision-bearing physical inputs is regressive; budget-specific fit remains candidate under R18 |
| UUID identity alternative | **Retain** | Not selected; a native codec mechanism alone does not justify changing hashed identity semantics |
| Unit/quantity resolver | **Retain** | Project-domain question remains open, not a claimed library absence |
| Schema-set feature policy | **Unresolved** | Actual compiled roots/directions must establish variation; runtime catalogue identity retained direction |
| Covariance stamp | **Retain** | Not selected; stamp is not content/statistical provenance |
| Tracing aggregation Layer | **Retain** | Candidate pending cost and propagation/attribution semantics; observation hub stays until settled |
| DA-023 architecture correction | **Retain** | Documentation owner follows adopted remedies; outside library solution focus |
| DA-024 native child/pse.canon.v2 purpose | **Retain** | Maintainer decision on bounded exploration purpose, not automatic deletion |
| DA-025 collector trigger | **Retain** | Maintainer decision; existing forget/reclaim/on-demand triggers can preserve daemonless P2 |
| Project-structure derive sizer | **Retain** | No selected in-use simplification established; no new dependency within this focus |

The refined prose does not create a mandate to adopt every available mechanism. Candidate
status retains the consequential question; it is not an implementation package or a finding
that every unexamined feature is missing.

## Independent evidence and limits

Decisive source was read for the material changed premises: store generator lifecycle and
maintenance, result predicate/key/protected decoder consumers, compiler graph/granular retention,
math region/input and construction admission, runtime cancellation/study persistence/policy,
boundary types/documents/time conversion, covariance propagation, allocator lifetime and toolkit
dependencies. Bounded researchers supplied store/result and preparation evidence; their
conclusions were leads and were reconciled with decisive source, not counted as independent
corroboration of the first review. Context7 oriented library research; exact pinned sources
decided version-specific claims. Negative conclusions are bounded to these owners and named
mechanisms, not an exhaustive absence claim over all library features.

The coordinator's only new execution was the small pinned codec probe, exit 0 and zero failures
on the recorded Linux/Python/msgspec conditions. F10 is a source-derived mathematical
counterexample. Original BT_PR operation counts, ThreeUnits estimates, horizon cancellation,
synthetic Uno mapping, memory-engine SurrealDB and standalone pruning probes retain original
dates and scopes. No RocksDB race, guarded-product performance, revised retention, native
allocator, full boundary or assembled scientific qualification was run for this audit.

Future adoption checks in §12 settle real remedy questions; publishing this audit does not
require implementing or measuring those remedies. G4/G6/PS-G1/PS-G2 remain unresolved, wider
unexamined areas remain unexamined, and quantitative latency/throughput/capacity benefits are
not inferred from structural work reduction.

## Planning handoff

No accepted implementation plan was revised or adopted by this audit. Future plan creation
remains the disposition owner. It should consume the integrated principal, present RC01–RC18
to the operator where relevant, and choose bounded scope rather than importing every candidate.
R8 default/live-set correction and catalog firewall are independent of R5; many R9 local indexes
can precede the final resolved representation. R2 does not require tagged lifecycle migration.
R14's map seam precedes removing the Python KINSOL route. R7 chain/preflight conditions and
R20 mathematical/provenance admission are preserved obligations, not future optional polish.

The next decisions are the desired resolved representation (RC01), bounded retention policy and
defaults (RC03), and covariance input/validity contract (RC18). Independent F01/F02 correctness
repairs need not wait for those larger choices. The review stays evidence until those decisions
and implementing scope are explicitly accepted.
