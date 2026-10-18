# Post-Plan44 preparation, reuse and accounting inquiry

Supporting inquiry for [the principal review](../../reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The principal owns the integrated judgment and follow-up; this file records a bounded source/library assessment, not a competing disposition ledger.

Supporting focused advice for the coordinator's post-Plan44 design review. This document does not issue an enclosing acceptance verdict or create a disposition ledger.

Examined baseline `91fd915134` in `/home/paul/pse-arrow`, preserving the six existing dirty/untracked paths: `.envrc`, `.python-version`, `docs/dev/agent-environment.md`, `python/pse/tests/test_native_workflow.py`, `scripts/pse_env.py` and `scripts/setup-env`. Investigation was read-only: no repository files changed, builds, tests, services or further agents started. Applied Core 3.4, heuristics 1.0 and Process Simulator 1.5; loaded the shared worker/design-reviewer contracts, design-review/process-simulator skills, and Salsa/math library skills. Publication of this inquiry is the coordinator's responsibility.

The strongest remaining opportunities concern **what gets prepared and represented**, followed by **how retained preparation is accounted and invalidated**. Plan44's active-view and immutable instruction reuse fixes are present; a further program should build on them.

## Already repaired and worth preserving

- Active study lanes retain one compatible current immutable view before consulting optional shared lookup retention. `bound_lookup` still admits current states/domain tightenings; incompatible keys discard the lane slot; changed attribution is reapplied. See [views.rs](../../../../crates/pse-runtime/src/workflow/modeling/views.rs) at `bound_case_with_lookup_reservation` (approximately lines 96–174) and [study_operations.rs](../../../../crates/pse-runtime/src/workflow/study_operations.rs) at `StudyPreparationLane` (1194–1280). Repository-relative source owners are `crates/pse-runtime/src/workflow/modeling/views.rs` and `crates/pse-runtime/src/workflow/study_operations.rs`.
- `MathService::artifact` and `scalar_products` use completion-owned flights, actual compiler-issued keys, generation fences and product-held allocation owners. Clearing retention does not cancel escaped products. See [artifacts.rs](../../../../crates/pse-runtime/src/math/artifacts.rs), through approximately line 240.
- Implicit residual/hint/assessment instruction requests now use those artifact owners while configuration, provider workers and cancellation remain fresh. The C02 outcome is consistent with [math/modeling.rs](../../../../crates/pse-runtime/src/math/modeling.rs)'s request/build/assemble route.
- Compact scientific cases now explicitly select local runtime construction ([compact.rs](../../../../benches/benches/native_process/compact.rs), approximately 28–41). Reintroducing durable fixture construction would regress the demonstrated N01 correction.
- `ConstructionEstimate::{Known,Opaque}` already distinguishes growable known construction from prepaid opaque capacity ([jobs.rs](../../../../crates/pse-runtime/src/math/jobs.rs), approximately 373–457). A replacement should extend this owner rather than introduce another admission system.

[Plan44c](../../../../docs/plans/44c-active-preparation-and-study-reuse.md), [Plan44e](../../../../docs/plans/44e-short-run-investigation-and-optimization.md) and [Plan44f](../../../../docs/plans/44f-integrated-acceptance-and-follow-up.md) record selected qualification and once-only observations, including fresh reference admission around 8.664 seconds and property-value work around 10.204 seconds. Those records do **not** establish exclusive loader/Serde CPU attribution, comparative speedup, large-model feasibility or sustained throughput. None was newly executed here. The source plans reside in `docs/plans/`; the coordinator should adapt these source links to the publication location.

## P01 — Local model admission still performs a typed-to-transport-to-typed round trip

**Implemented path, source-backed diagnosis:** `canonical::persist` takes typed declarations, builds a generated Arrow declaration batch, finishes and drops that batch, constructs a `SourceIndex`, serializes each declaration and spans into JSON `ObjectEdit` payloads, and hashes those payloads ([canonical.rs](../../../../crates/pse-runtime/src/workflow/modeling/canonical.rs), approximately 535–610). `admit_revision` invokes `AdmittedModelRevision::apply` **before** branching between ephemeral and durable execution ([canonical.rs](../../../../crates/pse-runtime/src/workflow/modeling/canonical.rs), approximately 793–828). `LocalSourceObject::from_edit` then decodes the declaration payload back into `SourcePayload` and obtains parsed fields ([local.rs](../../../../crates/pse-runtime/src/workflow/modeling/canonical/local.rs), approximately 49–113).

Thus local execution avoids the database but still pays the canonical transport realization. This substantiates a representation diagnosis that the previous partial CPU capture could not establish. It does not establish how much of the fresh-case wall duration it explains.

**Proposed target:** the existing local source owner should consume admitted typed source objects and typed deltas directly, retaining their immutable declaration, span/parser and allocation owners. Canonical serialization belongs at the actual recording/export/import boundary. Consumers should receive the same exact local lookup, version, absence/membership and interpretation contracts without interpreting serialized payloads themselves.

The simplest correction can preserve the existing canonical byte/hash identity while eliminating immediate decode and duplicate typed reconstruction. A more ambitious correction can defer encoding until a real boundary; that requires an explicit identity decision if local revision identity currently depends on those bytes. Neither requires moving scientific authority to a database or adding a second editable model.

**Preservation/adverse case:** direct typed ingress must still execute the same generated/native predicates under the actual receiving context. A source loaded under another registry, validation owner or invocation cannot inherit current eligibility merely through an `Arc` or matching digest. Imported bytes remain untrusted until admitted. Source ordering, spans, absent-name dependencies, exact durable retry identity and acknowledged-effect cancellation behavior must survive.

**Rules:** removing the transport round trip while preserving identities changes no fundamental rule. Changing canonical object preimages requires an explicit impact against ADR-0089 and the source-identity contracts in blueprint §14.4/§20. Keeping those rules limits the first correction to sharing typed products alongside unchanged encoding.

## P02 — Small source edits still copy metadata for the whole local source inventory

**Implemented path:** `AdmittedModelRevision::apply` calculates `old_bytes` over all object/inventory keys, reserves a multiple of that total, then clones both complete `BTreeMap`s before examining whether individual edits are unchanged ([local.rs](../../../../crates/pse-runtime/src/workflow/modeling/canonical/local.rs), approximately 211–267). Unchanged payload owners and unchanged bucket vectors are shared, which is good; the map nodes and keys still copy. `canonical::persist` also constructs the full candidate edit population before filtering unchanged objects ([canonical.rs](../../../../crates/pse-runtime/src/workflow/modeling/canonical.rs), approximately 738–788).

A single changed declaration therefore has an operation component proportional to the whole admitted source inventory. Retaining several revisions compounds the copied metadata even when most actual objects remain shared. This is a concrete AP-07/H12/H15/H17 target for edit/re-solve; no latency estimate is needed to establish the copied work.

**Proposed target:** local revision construction consumes a typed changed-object set and updates structurally shared ordered indexes. Existing exact Scope/Kind/Name/reverse-References buckets remain the semantic access contract. Unchanged versions, parser owners and bucket owners remain shared. A whole inventory/export remains an explicit full operation.

**Library alternative:** `rpds = 1.2.1` is already pinned and used by `pse-modeling` phase maps ([phases.rs](../../../../crates/pse-modeling/src/check/phases.rs), also line 512). Exact installed source exposes `RedBlackTreeMapSync`, constant-time structural-sharing clone and `insert_mut`/`remove_mut` (`/home/paul/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rpds-1.2.1/src/map/red_black_tree_map/mod.rs`, approximately 90–111,838–878). Context7 did not resolve this Rust library correctly: searches for `rpds` and `Rust persistent data structures rpds` returned unrelated AWS/JavaScript libraries. Those results were not used. The capability claims are from exact local source. This is a credible existing-library replacement for map snapshot copying, not a recommendation for a custom persistent tree.

**Preservation/adverse case:** bulk imports may favor a transient builder; replacing every map indiscriminately could hurt cold construction and iteration. Deletion, rename, references changing without payload changes, ambiguity and negative lookup must update the same authoritative indexes. Persistent node charges need an ownership contract; naïvely charging each reachable snapshot recreates the present inflation.

**Rules:** no fundamental rule change; blueprint §14.4 explicitly acknowledges that map containers can still copy once per edit. Its implementation description would change when the correction lands.

## P03 — Candidate construction allowances accumulate as retained Salsa bytes and eventually cause broad invalidation

**Implemented path:** `granular::Driver::preflight` adds each phase's allowance to `State.retained_preflight` unconditionally ([granular.rs](../../../../crates/pse-compiler/src/workspace/modeling/granular.rs), approximately 295–318). Calls precede the equality/query-hit checks in parse, inventory, signature, members, finish and expression staging. `State::known_bytes` adds that cumulative allowance to known retained bytes ([granular.rs](../../../../crates/pse-compiler/src/workspace/modeling/granular.rs), approximately 200–233). Retention excess rotates the entire Salsa generation ([workspace.rs](../../../../crates/pse-compiler/src/workspace.rs), approximately 1536–1552; [modeling.rs](../../../../crates/pse-compiler/src/workspace/modeling.rs), approximately 474–480).

An entirely unchanged revision correctly takes an early reuse return ([modeling.rs](../../../../crates/pse-compiler/src/workspace/modeling.rs), approximately 440–451). The problem concerns repeated small accepted edits: conservative candidate allowances keep accumulating even where the associated inputs compare equal or phase results reuse persistent structure. Eventually the implementation discards unrelated useful memos because a construction-history upper bound crosses the retained limit.

This is **not an identified under-accounting bug**. It is conservative accounting with a reuse cost and a lifetime mismatch: candidate peak admission and still-retained memory are different quantities.

**Proposed target:** keep candidate construction admission separate from retained product ownership. Count newly retained conservative node/container deltas at the owners that create them; preserve shared-node ownership across phase products. Let actual Salsa metadata/heap callbacks cover the storage they can report. Keep a separately justified bound for unreported foreign state. LRU and generation rotation remain safe fallbacks, rather than the normal consequence of accumulating allowances for cache hits.

Salsa 0.28.4 already supplies equal-result backdating, query LRU, immediate `trigger_lru_eviction`, heap reporting and generation-safe handles. Its memory report does not discover arbitrary hidden heaps. The skill's exact 0.28.4 `content/capabilities/salsa.memory.md` and `salsa.backdating.md` briefs and source support these distinctions. Context7 resolved `/salsa-rs/salsa` and returned current upstream tuning documentation for LRU/set_lru_capacity/cancellation. That result is consistent but was not treated as exact-release proof: its examples are from `master`, and version-specific decisions used the skill's 0.28.4 source/probes and the consumer pin.

**Preservation/adverse case:** equal source inputs do not imply equal contextual admission, and a new numerical state still needs original checks. Persistent nodes retained through older memos really do occupy memory. The correction cannot simply reset the cumulative counter after each request or lower the workspace bound from RSS.

**Rules:** no semantic rule change is needed. Any change to the declared reserved workspace model/defaults affects blueprint §14.3.2 and the corresponding allocation contract. Retaining those bounds still permits correcting retention bookkeeping and reducing unnecessary rotations.

## P04 — Complete physical inventory identity remains a broad invalidation boundary

`physical_identity` hashes all units, quantity kinds, bases, reference states and physical prerequisites ([physical_identity.rs](../../../../crates/pse-compiler/src/physical_identity.rs)). Compiler lane matching compares complete `CompilerContext`; queries read its full quantity/precondition inputs; view keys frame the physical context hash ([solve.rs](../../../../crates/pse-compiler/src/workspace/modeling/executable/solve.rs), approximately 221–269). Consequently, an unrelated admitted physical declaration can invalidate preparation for a model that does not consume it.

Blueprint §14.4 explicitly documents this as conservative current behavior. It is correct invalidation, but it leaves a credible scaling improvement for extensible property/unit catalogs.

**Proposed target:** physical admission produces the consumed operation/type/conversion dependencies needed by each prepared product, including relevant negative lookup and membership premises. Compiler/body/view reuse consumes that dependency closure. Complete physical inventory identity remains suitable for full-context interchange and admission; it need not be the identity of every mathematical product.

**Preservation/adverse case:** a newly added declaration may change resolution ambiguity or operation selection even when no previous positive reference names it. A finer key that tracks only used IDs is unsound. The dependency owner must capture absence, resolution scope, provider interpretation and conversion/reference-state semantics.

**Rules:** adopting this changes the conservative physical-inventory rule documented in blueprint §14.4 and the relevant versioned identity frames. If retained, preparation must continue invalidating after any physical inventory change. No claim that current results are stale follows.

## P05 — Scalar optimization reuse still leaves repeated program-description construction

**Implemented path:** `PreparedSupport::prepare_scalars` traverses selected stages, forms arithmetic regions, collects reads/parameters and builds exact scalar products ([execution.rs](../../../../crates/pse-math/src/execution.rs), approximately 761–865). Compilation traverses and forms those regions again ([execution.rs](../../../../crates/pse-math/src/execution.rs)). Scalar preparation, bounded conversion and final program census repeatedly count operations and export instructions ([admission.rs](../../../../crates/pse-math/src/library/admission.rs), approximately 332–350,440–605; [library.rs](../../../../crates/pse-math/src/library.rs), approximately 327–340; [execution.rs](../../../../crates/pse-math/src/execution.rs), approximately 4430–4460).

Exact Symbolica 3.0.1 `export_instructions` allocates exported instruction/constants/call descriptions; it is not a borrowed zero-cost view (`symbolica-3.0.1/src/evaluate/instruction.rs:219–287`). The current compiled constructor correctly borrows the caller's final export instead of making another inspection clone, but preceding exports remain.

**Proposed target:** prepared arithmetic regions retain immutable expression/parameter/source-check descriptors and source-qualified census facts. Product construction carries one inspected export through admission, storage census, reset eligibility and optional compilation. Per-order Taylor products remain distinct when their layout differs. Changed policy still performs its own admission against retained facts.

This is a smaller correction than whole-body fusion and fits existing compiler/math ownership. It removes repeated description materialization without inventing arithmetic instructions or another cache authority.

**Qualification:** the number and cost of these exports are source-backed; their material elapsed contribution is unmeasured. Scalar-family sharing across different coordinate selections is another candidate because the outer scalar key includes coordinates while exact inner keys use expressions/parameters/options ([workspace.rs](../../../../crates/pse-compiler/src/workspace.rs), approximately 541–572; [admission.rs](../../../../crates/pse-math/src/library/admission.rs), approximately 372–380). It needs an exact-support comparison before changing that key; it is not established here as a defect.

## P06 — Larger numerical preparation should compare library-owned program composition and JIT, beyond adjacent sibling CSE

Current `arithmetic_region` combines consecutive **independent** arithmetic blocks and stops when a later block consumes an earlier output ([execution.rs](../../../../crates/pse-math/src/execution.rs), approximately 4140–4230). Provider, domain and branch stages remain barriers. This preserves original materialization/failure semantics, but dependent straight-line regions cannot benefit from one library program merely because sibling CSE landed.

Current compiled evaluation also exports and invokes a compiler/linker for every eligible region ([native_compiled.rs](../../../../crates/pse-math/src/native_compiled.rs), approximately 470–584). The implementation has real benefits: explicit generic x86-64 flags, disabled contraction/unsafe math, bounded source/output populations, owned child cancellation/drain and private loaded worker scratch. These must be compared with alternatives, not discarded as incidental complexity.

Two exact-release alternatives deserve a larger design inquiry:

1. Symbolica 3.0.1 `FunctionMap`, aliases and `FunctionRegistrationOptions` with explicit inlining support library-owned composed programs (`symbolica-3.0.1/src/evaluate/function_map.rs:30–156`). Current bounded Taylor conversion, compiled export and reset eligibility deliberately reject relevant sub-evaluator/external state. Exploiting composition therefore changes the consumed preparation contract; it is not a wrapper substitution.
2. Symbolica 3.0.1 exposes `jit_compile` and `JITCompilationSettings` (`evaluate/backend.rs:418–585`). The existing `pse-math/compiled-evaluator` feature already enables `symbolica/native_code_generation`, so the JIT API is available in that optional profile. Locked SymJIT is 2.26.0. Its options include disabling `fastmath`, SIMD and threads; its FMA fusion is gated by `fastmath` (`symjit/config.rs:604,781–845`; `symjit/mir.rs:2375+`). The present floating-point policy is therefore a qualification question, not sufficient reason to exclude JIT without comparison.

**Exact release and documentation boundary:** the math skill pins Symbolica/Numerica/Graphica 3.0.0; the consumer pins Symbolica 3.0.1 (`Cargo.toml:228`), Salsa 0.28.4 and faer 0.24.4. API-specific claims above were checked in the installed exact 3.0.1 source under `/home/paul/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/symbolica-3.0.1/`; the skill's 3.0.0 contracts were not silently transferred. Context7 resolved `/symbolica-dev/symbolica` (also offered `/websites/symbolica_io`) and returned current upstream evaluator clone/mutable-evaluation, optimization and CSE documentation. A JIT-specific query returned more general optimization/evaluator examples rather than a settling JIT contract. Those current `main` results were discovery context, not exact3.0.1 API proof. Exact installed Symbolica and locked SymJIT sources supplied the stronger evidence. No JIT compile or execution probe was run.

**Proposed target:** the math integration owner consumes a demanded, source-mapped arithmetic program and supplies library-owned interpreted or native executable templates. Scientific control, provider effects, domains, ordered original failures and independent result verification remain host-owned. Private mutable evaluator/provider scratch stays attempt/session-owned.

**Adverse cases:** fusing a dependent region may evaluate past an original failed intermediate, erase attribution, or remove a nonfinite intermediate through algebraic cancellation. Literal substitution can explode expression size. Registered functions may introduce hidden state or exceed admitted derivative order. JIT construction may lack cancellation or a sound heap/code bound; cloning may duplicate code or mutable state. These are deciding premises, not reasons to prescribe JIT now.

**Rules:** changing the C++ realization or eligible instruction family affects proposed ADR-0191, blueprint §14.1/§14.3, evaluator ABI identities and source-qualified export/reset/admission assumptions. Keeping those rules narrows the correction to reducing metadata work and improving region/module sharing within the current C++ route. Scientific accuracy and original assessment need not change.

## P07 — Historical/branch preparation has a fresh-workspace cost distinct from active study reuse

When a requested immutable revision differs from the current compiler head, `with_modeling_compiler` releases live authority, creates a separately charged workspace and publishes the admitted historical revision into it ([modeling.rs](../../../../crates/pse-runtime/src/math/modeling.rs), approximately 1547–1579). An older source stamp similarly forks (1582+). Each workspace prepays the policy's 4 GiB generation capacity ([math.rs](../../../../crates/pse-runtime/src/math.rs), approximately 796–813), while source staging separately reserves up to half that capacity ([modeling.rs](../../../../crates/pse-runtime/src/math/modeling.rs), approximately 1073–1082).

Correctness benefits are clear: historical reads cannot rewind the live head; no cloned Salsa handle masquerades as a snapshot; no recursive acquisition of the same writer. A complete basis cache hit can bypass much of this work. On misses—particularly changing bindings, selections or pressures—repeated historical work can still reconstruct private semantic generations.

**Proposed target:** compare bounded retention of exact historical preparation lanes with fresh private workspaces, keyed by immutable revision plus complete consumed context. Keep a sole writer for a live lineage and exclusive mutation for each retained historical lane. Consumers receive immutable products; no Salsa handles escape. Retention remains optional and product-owned.

This is a proposed remedy, not evidence that every historical request presently rebuilds or that retaining more lanes is always better. History-heavy use, branch skew and ordinary active studies are different workloads. Extra retained lanes can themselves worsen admission; retaining a compact immutable phase product may be better.

## Memory and accounting conclusions

- The 8 GiB optional artifact/modeling limits, 4 GiB workspace reservation, 8 GiB worker capacity, 64 MiB foreign allowance and 16 MiB inner-session allowance describe different populations. Their sum is not RSS.
- Both artifact and modeling caches hold reachable-product weights; shared payload references can appear in more than one cache weight while unique pool leases remain attached to owners. Cache weight is therefore not unique physical allocation.
- Lane count does not universally bound live view count: incompatible transitions, Full retained results and escaped aliases can legitimately keep earlier products alive.
- Current `bound_lookup` still clones/adopts variable metadata and rehashes stable model/profile components for each point ([solve.rs](../../../../crates/pse-compiler/src/workspace/modeling/executable/solve.rs), approximately 221–269,619–665). A precomputed immutable base identity plus current admitted state delta is a further local optimization; checks on changed bounds/fixed state must remain.
- No sound smaller opaque/foreign bound follows from Plan44's RSS observations. Prefer removing representations, sharing actual owners and source-qualifying known construction before reducing an allowance.

## A genuinely larger follow-up program

The useful scope is an incremental **source-to-executable preparation architecture**, with independently reviewable responsibilities:

- Typed local source revision/delta ownership; serialization at actual transport boundaries.
- Persistent source indexes and dependency-shaped phase inputs, including absence/deletion.
- Retained accounting aligned with actual immutable products and shared nodes.
- Explicit consumed physical dependencies.
- Prepared arithmetic regions and one-pass library description inspection.
- Qualified library composition/JIT versus the existing C++ route.
- Historical preparation reuse and active study/session/result lifetimes considered together.

This extends beyond “repeat the short campaign” or “wait for KLU symbolic cost to grow.” P01–P03 already have concrete source causes. P04/P06/P07 need design choices and selective settling evidence. They should not become a mandatory new registry, scheduler, universal cost model or wholesale rewrite. These are proposed follow-up responsibilities, not scheduled work or an implementation plan.

The coordinator could schedule these small probes where they decide remedies:

1. **Local ingress:** generated typed rows and the real reference closure, separating load, physical admission, model admission and repeat use of the same owned sources. Count encoding/decoding and allocation populations; retain original semantic checks.
2. **Edit/invalidation:** repeated one-expression, value and span edits with unrelated declarations held fixed. Observe query executions/backdating, retained totals and generation rotations; include failed/cancelled candidate restoration.
3. **Physical dependency:** add an unrelated unit/type versus changing an actually consumed conversion/reference state. Establish the conservative current invalidation and the required future negative-lookup premises.
4. **Program preparation:** compare one dependent arithmetic chain and one provider/guard-bearing body across interpreted, current C++ and conservative JIT candidates. Separate preparation from repeated evaluation; test nonfinite intermediate attribution and cancellation/drain.
5. **Historical reuse:** alternate current and one historical revision under changed selections and optional-retention pressure, with retained old outputs and simultaneous readers. Record workspace construction/reservation counts, not just RSS.

The negative searches here covered the named preparation/source/math/study owners and adjacent consumers, not every model, provider or process-global allocation. No new runtime performance or whole-scientific qualification claim is made. The broad source-admission timing attribution, actual edit-scaling magnitude, JIT cancellation/heap/private-worker guarantees, composed-function derivative eligibility and complete historical-workload economics remain unmeasured or unresolved where noted.

## Memory use

Prior registry guidance in `/home/paul/.codex/memories/MEMORY.md:909–916` supplied only the preparation decomposition as a lead; decisive findings above were checked against current source. Associated prior rollout ID: `01a10e4e-b043-7292-82a5-796d9844f8ca`. No memory files were changed.
