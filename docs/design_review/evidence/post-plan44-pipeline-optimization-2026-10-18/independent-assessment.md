# Independent post-Plan44 design assessment

Supporting independent assessment for [the principal review](../../reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The principal owns the reconciled judgment and future disposition route.

This is the independent reviewer's integrated assessment for the coordinator's principal review, intended for `docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md`. It is not a second disposition ledger. The coordinator should preserve material disagreements and the evidence limits below when publishing.

## Contract and decision

Design tier, target purpose; bounded end-to-end preparation, numerical execution and durable response/analysis performance. Baseline independently verified: `91fd915134f99edeb8ef00368096f84761960d95`. Applied core 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator 1.5 and the pse-arrow binding. Functional outcomes are interactive editing/re-solving, studies, recycles, dynamics from shared definitions, and extensible units/properties/solvers. Existing ADRs and policies describe the current implementation; they do not disqualify a technically better target.

**Overall: Revise the current physical execution design.** AP-07/G9 fail on concrete repeated reconstruction and growing scans in ordinary admitted-source and result-analysis operations. Correct numerical outputs do not resolve those findings. There is no independently established new scientific correctness defect. Equally, this assessment does not certify all numerical models, derivative families, recovery transitions or backends. Proposed native/JIT changes remain unqualified alternatives until their complete consumed contracts are tested.

The right target is a sequence of independently useful corrections, not a new universal optimizer, registry, cache or scheduler. Carry sufficient demand into each existing owner; retain reusable preparation only for its real validity; compose protected scientific evidence without repeating discovery; preserve checks at genuine trust and numerical-state transitions.

## Evidence and independence

I independently inspected the decisive source in `workflow/modeling/canonical.rs`, its `local.rs`, compiler `workspace/modeling/granular.rs`, workspace retention, `workflow/connected_results.rs`, `result_snapshot.rs`, `result_files.rs`, `analyses.rs`, `artifacts.rs`, `surreal_analyses.rs`, `math/modeling.rs`, `pse-math/execution.rs`, `native_compiled.rs`, `workflow/integrated.rs` and the native dynamics integrator. The three inquiry documents supplied leads and wider library/Plan44 interpretation; their complete library claims were not independently rerun here. I read the review skills and the exact standard/binding/profile obligations. No repository edit, build, service operation or test was performed by this reviewer.

I also read the coordinator's retained local-pipeline JSONL and membership prototype source/results. Those executions belong to the coordinator, not to this reviewer. Five usable analytic local runs and membership equivalence are scoped behavior evidence; neither is a measured implementation speedup. The 8-state constant-rate case tests local composition/reuse, not sparse stiffness or realistic convergence cost.

## Responsibility and semantic model

The existing decomposition is largely sound: typed authored declarations govern model meaning; source admission owns revision closure; compiler lanes and mathematical products own immutable preparation; native workers own mutable numerical state; result selections own a historical terminal attempt; scientific artifacts own evidence independently of producer retirement; analyses own occurrences and graph meaning. Those distinctions should survive physical fusion and reuse.

The important variation axes are source edits versus value bindings, current versus historical revision, selected versus full products, sparse/fill/stiffness shape, number of derivative coordinates, publication ownership, and cold construction versus repeated use. A single variable called “problem size” or “cold/warm” cannot characterize them. A new unit or property should primarily extend scientific declarations/providers; a different linear solver should remain a native integration change. A narrower result reader should not force scientific consumers to interpret file ownership or raw index mechanics.

## Independently confirmed current optimization findings

### IR01 — Local admission reconstructs a transport representation

`canonical.rs::persist` clones typed declarations into an Arrow builder, finishes and drops that batch, constructs another source index, and serializes declaration payloads into `ObjectEdit`. `admit_revision` calls `AdmittedModelRevision::apply` before checking durability. `local.rs::LocalSourceObject::from_edit` immediately decodes the serialized declaration and reparses fields. Ephemeral operation therefore pays the canonical transport round trip too.

Correction: local admission should consume checked typed declarations/deltas and construct the local authoritative revision directly; serialize only where durable/export effects require that representation. This is not permission to treat an arbitrary typed row as semantically valid. Registry cross-field validation, complete reference/negative-lookup closure, cancellation, version identity and atomic publication still require an enforcement owner. Preserving the existing durable version/hash definition may require computing canonical identity bytes even when a local consumer no longer needs to deserialize them; eliminating decoding is independently useful.

### IR02 — Small source edits copy whole indexes and accumulate candidate allowances as retained memory

`local.rs::AdmittedModelRevision::apply` reserves for and clones the previous complete object and inventory maps before filtering unchanged edits. `granular.rs::Driver::preflight` unconditionally increments `retained_preflight`; `State::known_bytes` includes that cumulative field, and workspace retention excess rebuilds the whole generation. The entirely unchanged revision early return already exists, so this is not a finding that every repeated preparation rebuilds.

These are related edit-path costs but distinct remedies: structurally shared authoritative indexes, and candidate-versus-retained lifetime accounting. A persistent map is credible because rpds is already available, but cold bulk building, deletion/rename/ambiguity and shared-node charging must be compared. Do not reset the counter opportunistically, infer a safe bound from RSS, or omit old products still retained by escaped handles. A repeated small-edit ladder with unrelated declarations, failed candidates and retained old outputs is the discriminating test.

### IR03 — Selected result intent arrives after full inventory discovery

`output_results` invokes `results`; `result_selection` eagerly captures a relation/range snapshot, querying block and descriptor pages. Consumption subsequently walks block metadata again, or obtains output pages for a selected reader. Full-reader duplication is independently removable by consuming the frozen inventory. Narrow or absent outputs still pay broad initial metadata discovery.

**Material corrective constraint:** public `snapshot()` synchronously returns the frozen relation-and-row-range inventory, and native query plans share it. It does not promise the output reader's predicate. A proposal that simply narrows the existing snapshot would silently change meaning. Avoid claiming the whole selected-reader correction is merely internal. Choose an explicit selected-snapshot/constructor contract or deliberate API migration; alternatively keep broad snapshot behavior and accept that cost. Protection of unread future files, original row coordinates, index-versus-IPC checks and late errors remain mandatory. Different dense-time and scalar-value predicates must not be conflated.

### IR04 — Scientific promotion and analysis repeat selection, conversion and evidence qualification

`select_result_analysis` admits an exact result selection, reads sources, decodes dispatch context and computation headers. `result_analysis_body` calls public promotion, which repeats selection/source discovery and reads/encodes the scientific relations. Analysis then opens the copied artifact, decodes/admit its members and rebuilds evidence. Source independently confirms repeated work; Plan44's inclusive activation interval alone does not identify which part dominates.

Correction: an internal composition accepts the existing protected selection and produces validated scientific evidence together with final artifact coordinates as promotion copies data. Retain the independently callable public operation. Established validity may be reused only while byte identity, schema/interpretation, contextual admission and immutability premises remain valid. New file ownership, final coordinates, activation and lost-ACK recovery are genuine new transitions, not redundant checks to skip. Graph activation waits for the exact ownership handoff. A shared sealed-file design is a larger alternative, because current artifact block-zero and owner/selection contracts do not support it by adding a borrower alone.

### IR05 — Analysis qualification scans samples and run headers per response row

`ResponseEvidence::transient_recorded` creates filtered computation/context/endpoint vectors for each response, then calls `samples.iter().any` with run, sample, time, output and unit equality. This is avoidable repeated work as output samples and response directions grow. Build operation-local membership and run/step qualification groups once; preserve all conflicting-header/context checks and per-row quantitative locators. A new persistent evidence cache or a DataFusion conversion is not necessary for this bounded typed operation.

The coordinator's standalone prototype reduces the 80-output-symbol × 25-time × 4-direction case from 8,004,000 sample visits to an index built from 2,000 rows plus 8,000 membership lookups. It tests only membership, not the full graph, header predicates, derivative permission, artifact lifecycle or end-to-end latency. The initial JSON field `outputs` named the sample-time count; the coordinator renamed it `sample_times` and recompiled/reran the prototype before publication. Signed zero must remain equal and every NaN must remain unequal; an unqualified bitwise float key is wrong. Additional production tests must preserve duplicate/conflicting metadata, withheld claims, partial endpoints, units, missing samples and original graph identity.

### IR06 — Repeated decoding preflight and graph-page checks have narrow work-removal routes

`decode_result_file` preflights immutable bytes, then `read_file_batch` invokes `open_file_reader`, which preflights them again. One opener enforcing the single-batch count can remove this exact duplicate without bypassing bounded hostile-input checks, Arrow decoding or row/schema validation.

Separately, `surreal_analyses.rs::append` loops over edges and repeatedly queries shared endpoint/analysis-input/artifact-block facts within a bounded page. Resolve distinct facts once per page inside the existing transaction, then check every edge and its immutable replay. The server remains an enforcement boundary even when the client validated an earlier graph. Bulk insertion, new indexes or lock removal need actual pinned-engine semantics and query access evidence; “one RPC” is not a work count. These are independent narrow corrections, not explanations for all publication latency.

## Expanded opportunities requiring qualification rather than assumed adoption

The preparation inquiry's finer physical-dependency closure, arithmetic-region metadata/export reuse, conservative library JIT/composed programs and bounded historical preparation are credible. Current physical whole-inventory identity is conservative, not stale-result evidence. Used-ID keys alone omit negative lookup and ambiguity. Current adjacent arithmetic fusion deliberately stops at dependencies/providers/guards; larger fusion must preserve ordered nonfinite/domain failures and attribution, not merely final scalar equality.

The C++ route currently buys bounded outputs, explicit conservative CPU/math flags, owned child cancellation/drain and private scratch. Symbolica JIT eligibility is not the same as an equivalent lifecycle. Exact Symbolica 3.0.1 must be used rather than transferring all 3.0.0 skill claims. Compare conservative JIT options, cancellation during construction, code/heap ownership, clone behavior and derivative products. An upstream API extension may remove machinery more effectively than a larger host wrapper. A JIT switch is not a prerequisite for removing repeated exports under the current route.

Native selected-adjoint parameter coordinates are a concrete source-backed opportunity: non-Forward requests clear the forward selection; integration retains complete parameter axes and shooting later projects gradients. Introduce a distinct consumed-adjoint-coordinate contract, keep the state adjoint and full physical values, and account for direct, initial-condition, tied/scaled and scheduled/anchored contributions. Truncating physical values or reusing a forward-axis field would be unsound. State-adjoint cost may dominate, so the elapsed benefit is unmeasured.

Cross-application KLU symbolic retention remains a real work-repetition lead, but tiny Plan44 symbolic intervals do not justify a maintained replacement on their own. Compare stock behavior, an upstream/narrow SUNLinearSolver integration and eligible Krylov/preconditioning under meaningful sparse fill. Keep KINSOL's iteration/globalization and fresh numerical validity. Do not suppress library initialization to make counters smaller. Checkpoint replay/retention and method/linear-profile selection need independent workload dimensions; a longer run of the same trivial model does not expose them.

Do not re-propose completed selected-forward work, fresh fitting final validation, combined IDAS quadratures, zero-weight adjoint jump skipping or HiGHS compatible numeric updates. Do not infer a universal solver winner from step/setup counters with different meanings.

## Foundation and gate judgment

These judgments concern inspected current performance responsibilities, not complete numerical-catalog qualification. AP-01 through AP-06 are **satisfied within this boundary**: the source exposes coherent scientific and lifecycle owners, explicit historical selections/outcomes and local ephemeral execution. No independently established authority or scientific-model adequacy violation follows merely from the repeated representations. The proposed corrections must retain these strengths. AP-07 is **violated**: IR01/02/03/04/05 demonstrate avoidable reconstruction, broad work before selection and growing repeated searches in supported editing, studies and result analysis. G9 consequently **fails**.

G1/G2/G3/G4/G5/G6/G7 are **pass for the inspected existing contracts**, with no new failure established: distinct revisions/attempts/artifacts, explicit validation/effects/recovery, exact numerical settings and honest incomplete diagnostics remain. This pass does not cover uninspected provider physics, all crash/recovery interleavings or every library routine; proposed replacements do not inherit it. G8 **passes at the reviewed capability boundary**: native libraries own the numerical machinery, and no clear established-library substitution has yet been qualified and then unjustifiably rejected. Potential JIT and symbolic-retention alternatives do not by themselves make the current bindings G8 failures.

PS-G1 **passes only for the preserved physical-coordinate contracts examined** (time/output/unit identity and source-qualified evidence). Whole property-envelope/conservation qualification was not repeated. PS-G2 is **not applicable to the structural-algorithm assessment** because no formulation, degree-of-freedom or decomposition algorithm is changed or independently audited here; it remains a required preservation check for a later composed-program implementation. PS-G3 **passes for the inspected existing outcome/accuracy distinction**, backed by source contracts and the coordinator's limited analytic control; it is **unresolved for the proposed numerical alternatives** until derivative, failure-order, accuracy and lifecycle qualification. The overall Revise judgment does not rely on turning unexamined breadth into a defect.

## Follow-up program and ordering

Prioritize consequences rather than simply taking the cheapest patch first. Broad local preparation and durable analysis can dominate user-visible small simulations; transient membership has demonstrated growth; sparse solver, JIT and adjoint work need shape-sensitive evidence. Independent IPC duplicate removal is useful but should not be advertised as the multi-second solution.

First specify consumed typed-source and selected-reader meaning and the reusable scientific-evidence/ownership handoff. Those contracts enable larger work removal; they need not block independent map/accounting, membership, export-description or duplicate-preflight corrections. Native investigations proceed independently under unchanged science policies. No global cache/service is a prerequisite.

The substantial follow-up scope should include: repeated source edit/invalidation with actual retained products; cold source publication versus current-runtime reuse; same-run full/narrow/absent result selection with construction/first-batch/EOF query counts; promotion/graph stage work with ACK/cancellation cases; sparse fill/stiffness/coupling contrasts; selected forward/adjoint/quad dimension contrasts; schedule/checkpoint observation patterns; conservative program-composition/JIT qualification; history/branch concurrency and pressure. Select the smallest model that exposes each mechanism and retain independent physical/original-state assertions. Longer horizons follow only a demonstrated question that short shape contrasts cannot settle.

Use existing operation/phase diagnostics first. Add precise encode/decode/export, selected-coordinate and query-owner counters only where their absence blocks a decision. Counts are not time, server wall is not CPU, inclusive phases cannot be summed as disjoint work, and incomplete raw streams cannot establish exclusive attribution. Once-only observations are useful controls but not causal speedup or stable rank estimates.

## Rule impacts and exclusions

Concrete current rules potentially affected are the synchronous relation/range snapshot meaning; canonical source/version identity if deliberately changed; whole physical-inventory preparation identity; evaluator ABI/export/reset eligibility and C++ realization; native symbolic-preparation ownership; internal/public adjoint coordinate contracts; packed-batch artifact/file ownership if shared files are adopted; and any numerical default deliberately changed after qualification. Distinguish a blueprint implementation-description update from a change of scientific or public contract. Ordinary internal work removal need not invent a rule decision. No default tolerance change is recommended.

Keep current public snapshot meaning if a selected-snapshot decision is not adopted; still remove full-reader rediscovery. Keep existing file ownership if shared files are not adopted; still compose discovery/validation during the required copy. Keep C++ if JIT is not qualified; still retain prepared descriptions and eliminate repeated exports. Keep complete physical identity if a sound negative-dependency closure is unavailable. These alternatives make the program useful without forcing every investigation to choose a new mechanism.

No production remedy, library adoption, changed default, architecture edit or old-plan disposition is authorized by this assessment. The principal review records proposed work; a future plan owns adoption, rule decisions, implementation and qualification.


## Draft assessment addendum

# Independent draft assessment addendum

Reviewed the principal draft at `docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md`, including findings, follow-up program, rule impacts and scoped judgments. No repository edits, tests, builds or service operations were performed.

## Material correction

F08 currently says production integration must reject partial/event trajectories. That is broader than the inspected existing contract. The comment immediately above `ResponseEvidence::transient_recorded` in `crates/pse-runtime/src/workflow/analyses.rs` states that partial/event trajectories retain the weaker meaning of recorded sensitivity availability rather than a scalar local certificate. The predicate checks completed time/sample coverage and recorded context/endpoint membership.

The correction must preserve valid recorded transient evidence within its completed coverage, while withholding unsupported derivative/certificate claims and rejecting out-of-coverage or conflicting evidence. It must not introduce a blanket rejection of partial/event trajectories. Suggested replacement: “retain the weaker partial/event recording meaning and reject claims outside completed sample/endpoint coverage; unavailable scalar certificates remain unavailable.”

## Other material conclusions

No additional material unsupported claim, incompatible correction or omitted central scope was identified. The draft preserves the independently raised snapshot-contract, artifact ownership/ACK, exact float equality, full-value/state-adjoint and JIT lifecycle constraints. It separates existing source causes from unqualified mechanisms and substantial follow-up investigations. The corrected prototype naming and collection limits are described honestly. The scoped architectural Revise judgment remains supported independently of the unresolved native/JIT alternatives.

One implementation-planning caution remains: duplicate/conflicting evidence checks must be preserved together with an indexed membership kernel. The prototype establishes membership equality and reduced work counts only; it cannot itself authorize any new scientific permission. The draft already makes that limit explicit.
