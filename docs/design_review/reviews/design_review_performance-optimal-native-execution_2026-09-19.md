---
title: Performance-optimal native execution — where the current design repeats, copies and rebuilds work, and what the optimal design changes
date: 2026-09-19
status: proposed
scope: Active codebase — validation, engine preparation/execution, compiler stage pipeline, rules engine, numerical program and Ipopt driver, Delta/cache storage, memory accounting, identity/canonicalization, Python boundary
evidence: Implemented (code paths cited by file:line); Measured only where a named run or log is cited; Interface-checked for pinned-library contracts; Proposed for every recommendation
---

# Design review: performance-optimal native execution

## 1. Decision and scope

**Decision: Revise.** The codebase is correct in the places this review attacked, and it is
built on the right libraries. It is slow because the same facts are re-established at every
internal boundary, because DataFusion's session, catalog and plan machinery is used as a
disposable value object instead of a long-lived engine, and because the numerical evaluation
program that blueprint §18.2 promises is not the program that runs. Two MUST-level gaps stand
regardless of the rest: no end-to-end measurement exists for any performance claim (DM-39),
and two capability claims in the blueprint are not implemented as stated (G7). The
recommended target keeps every semantic guarantee and removes most of the repeated work; it
requires amending five documented policies, listed in §10, which the maintainer decides on
when this review becomes a plan.

**Proposal under review:** the implemented system at `main` `a46f358` plus the working tree
(Plan 10 N00–N17 complete, N18 open), viewed against the question "what is the optimal
performance design that still meets the functional target and the charter?" The user asked
for holistic coverage with specific attention to `SessionContext` use. Existing rules and
policies were treated as inputs to weigh, not constraints on the analysis.

**Reviewer:** Claude (Fable 5.1) with five read-only scouting subagents whose leads were
verified by reading the cited code first-hand before inclusion. Nothing below cites a line
the reviewer did not read.

**Affected revisions:** blueprint revision 37+ (§3.3.3, §5.4, §14.3, §14.3.1, §14.3.2,
§14.4, §18.2, §18.8, §24.3); ADR-0046, ADR-0047, ADR-0052, ADR-0055, ADR-0067, ADR-0068,
ADR-0070, ADR-0073; register R-01, R-22.

**Observable outcome sought:** a heater flowsheet compiles source→P10 in seconds, not 15+
minutes; a solver callback costs microseconds per node, not a lock and an allocation per
node; a publication opens each member once; unchanged inputs are never re-validated.

**Baseline (Measured, `build/assessment/2026-09-19-local-full/test.log`, 32-thread Ryzen
9950X3D, 197 GB RAM, `hardware.json`):**

| Observation | Value |
|---|---|
| `native_engineering_workflows heater_fctp/ftpx_source_to_canonical_graph` | timed out at 900 s each |
| `conservation_expansion` (three P3–P8 cases) | timed out at 360 s each |
| Full suite | 1052/1085 run, 1020 passed, 27 failed, 5 timed out, 1463 s wall |
| Passing-test latency | median 0.058 s, p95 7.0 s, max 98.9 s (`latency-summary.json`) |
| Heater reached P10 (earlier CI note, `.config/nextest.toml`) | 293 s; template journey 395 s; two engine tests 148 s and 210 s |

These runs use one worker thread and one partition per workflow
(`tests/support/workflow_runtime.rs:30-34`), a 32 GiB pool, and Arrow `force_validate`.
The numbers bound the problem; they do not attribute it. §9 says what to measure.

One attribution probe was run for this review (Measured, this session, same machine,
`cargo nextest run -p pse-tests-engine --locked --features pse-relations/force-validate
-E 'test(material_configuration_keeps_generated_members_and_native_support_together)'
--no-capture` with `PSE_TEST_PHASE_TIMINGS=1`, dev profile, nextest run
`f3491609-fd39-4b5e-a2f8-81c9393bcbd2`):

| Phase | Wall time |
|---|---|
| Compose the plan graph for source projection + P3 only, no data execution (`native_pipeline phase=compose through=P3`) | 10.08 s |
| Whole test until its pre-existing failure during execution (`AmbiguousReference witness_0.scope_id`, the same failure as in the 2026-09-19 assessment) | 54.5 s |

Ten seconds to *build* a plan for two stages with zero rows moved is direct evidence for
P03 and P04 below: that time is spent in session-state rebuilds, plan traversals and
requirement lowering, none of which depends on the data.

**Supported scope and non-goals:** the review covers the Rust workspace and the Python
boundary as they execute today. It does not review the authoring DSL parser, the physical
correlation formulas, Pyomo/NL backends (stubs), or generated code contents. It proposes no
new library, crate, or DSL. It does not relax any validation at an ingress boundary.

**Constraints and uncertainty:** the pins are DataFusion 55.1.0, Arrow 59.3.0, object_store
0.13.2, vendored delta-rs `58f07cd`. Two library contracts matter and were checked against the
skill's pinned rustdoc: `SessionContext::state()` returns a fresh clone that is "not shared
with the current session state", and `SessionStateBuilder::new_from_existing` clones "all
other fields" of the existing state. No profiler run exists; the cost attribution below is
structural (what code executes how many times), labelled Implemented, not Measured.

### Method and coverage

Read first-hand (complete files unless noted): `pse-relations` `columnar.rs`,
`validate/{mod,prepared,field,predicates,obligations,occurrences,row_checks}.rs`; `pse-engine`
`session/{engine_session,preparation,admission,admission/derive,traversal,reuse,cache,
cache/admission,cache/logical,freshness,input,roles,output,execution,config,policy,contract,
contract/protection,scalar,scalar/preserve_field,observation,materialized,round,dependencies,
operation,operation/completion,factory,resources,assurance,trace,native,capture,query_schema,
physical_fields,facts,checks,mod}.rs`, `provider/binding.rs`, `cache_service/{mod,inspection,
policy}.rs`; `pse-compiler` `native.rs`, `native/{model,value,layout,execution,source}.rs`,
`passes/{mod,bundle,argument}.rs`, and the cited ranges of `native_outputs.rs`,
`native_construction.rs`, `native_rows.rs`, `p3/config.rs`, `p3/config/instances.rs`,
`quantity_relations/inventory.rs`, `p7/paths.rs`, `p7/expressions.rs`,
`p4/predicates/scalar.rs`; `pse-rules` `strata/{mod,native,rounds,relational,admission}.rs`
(cited ranges), `invariants.rs`, `invariants/program.rs` (cited ranges); `pse-numerics`
`stages.rs`, `expressions.rs`, `finite.rs`; `pse-backend-native` `driver/{workspace,
callbacks}.rs`; `pse-ids` `owned_buffer.rs`, `resource.rs`; `pse-schema` `arrow.rs`
(cited ranges), `builder.rs:199-243`; `pse-catalog` `artifact.rs`, `delta/{publish,
admission,contract,attempt,publication,provider,dml/execution}.rs` (cited ranges),
`cache_service/{snapshot,settings,policy}.rs` (cited ranges); `pse-py` `inspection/{stream,
runtime}.rs`; `pse-runtime` `settings.rs`, `env.rs`, `budget.rs`; `tests/support/
{workflow_runtime,native_pipeline}.rs`, `tests/engine/tests/native_engineering_workflows.rs`;
`benches/benches/native_consolidation.rs`; `.config/nextest.toml`; the charter, the
template, blueprint §3.3.1, §3.3.3, §5.4, §14.3–§14.5, §18, §20, §24.3; ADR-0019, 0041, 0042,
0046, 0047, 0052, 0055, 0067, 0068, 0070, 0073; register rows R-01, R-22, R-24, R-31; the
2026-09-19 work-reuse review; the assessment logs under `build/assessment/2026-09-19-local-full`.

Scouted by subagents and **verified only at the cited lines**: the remainder of
`pse-compiler/src/passes/**`, `pse-rules/src/strata/native_input/**`, `plan/trace/**`,
`pse-mathir/src/*`, `pse-quantity/src/{infer,registry,index}.rs`, `pse-ids/src/canon/*`,
`native_value.rs`, `python/pse/*.py`. Findings that rest only on scout leads are marked
"(scout lead, cited lines verified)" in §7; leads that were not verified are listed in §9 as
open checks, not findings.

**Not inspected:** `pse-authoring/**` (except its entry into `p1::source_batches`),
`pse-schema/src/catalog/**` and `codegen/**`, `pse-quantity/src/generated/**`,
`pse-relations/src/generated/**` (117,999-line `contracts.rs`), `pse-catalog/tests/**`,
`pse-backend-nl`, `pse-backend-pyomo`, `xtask/**`. Silence about them is not clearance.

**Guarantees attacked:** "checked once" (attacked and found violated internally; ingress
boundaries hold); "reuse keyed to semantic dependencies" (attacked; no unsound reuse found);
"native evaluation program" (attacked; not implemented as specified); "zero-copy" (attacked;
wrapping is copy-free but not cost-free). **Asserted, not attacked:** commit atomicity under
concurrent writers, cancellation across the Ipopt boundary, retention/vacuum safety.

## 2. Authority and lifecycle map

The semantic authority model is sound and is not what this review revises. The map below
records where each fact is *established* versus where the code *re-establishes* it, because
that gap is the performance defect.

| Fact | Authority (established once) | Where it is re-established today | Consequence |
|---|---|---|---|
| Local field/value validity of a relation | `FieldCheckedBatch` minted by generated builders (`columnar.rs:515-543`) or by raw admission (`columnar.rs:122-136`) | `Layout::member` (`native/layout.rs:142`), `RelationPlan::capture` (`session/input.rs:96`), `checked_relation` (`preparation.rs:1080`), `merge_generated` (`p3/config.rs:235`), `execute_relation` (`rules/strata/rounds.rs:458`), `SourceProjection::run` (`native/source.rs:45`), Delta `admit` (`delta/admission.rs:108-135`) | Every stage boundary pays full validation of already-checked values |
| Semantic engine assembly (functions, rules, policies, settings) | `EngineFactory` (`factory.rs:112-187`) | `bound_state()` (`engine_session.rs:540-565`) at 23 production sites; `NativeExecutionContext::bind` (`execution.rs:54-101`); `execution_state` (`execution.rs:145-184`); `Witness::capture/matches` (`reuse.rs:25-56, 99-140`) | A `SessionState` is rebuilt several times per prepared plan and per cache node |
| Resolved Arrow schema of a declaration | Registry contract arena (`builder.rs:209`) | `relation_schema` rebuilt with two JSON serializations per call (`pse-schema/src/arrow.rs:62-89`) at validation (`validate/mod.rs:704`), preparation (`prepared.rs:62`), concat (`columnar.rs:153`), output (`preparation.rs:1057`), layout (`native/layout.rs:40`) | Declarations are re-serialized on every admission |
| Policy requirements (invariants) over a source set | Registry invariant declarations | Re-lowered by `RegistryRequirementPlanner::plan` (`invariants.rs:24-43` → `program::compile`) inside `plan_execution` (`preparation.rs:616-620` → `policy.rs:300-320`) on every execution of every prepared plan | The same invariant queries are re-bound and re-executed per output |
| Stage output relation | Algorithm result packed once (`native/layout.rs:144-193`) | Unpacked per consumer through Projection→Unnest→Unnest→Projection (`native/layout.rs:64-109`), re-captured and re-admitted (`native/execution.rs:112-148`) | Every consumer copies and re-validates every input relation |
| Numerical program | `EvaluationProgram::new` (`expressions.rs:126-181`) | Per callback: new `MemoryConsumer` (`expressions.rs:252`), global inventory sweep (`owned_buffer.rs:156-157`), per-node `RecordBatch` (`stages.rs:145-150`) | Structure-only work is repeated per solver iteration |
| Delta snapshot at a pinned version | Delta log | Uncached `get_latest_version` before every cache lookup (`cache_service/snapshot.rs:235-239`); key includes `observed_latest` (`:250-255`) | A cached pinned version is invalidated by any later commit; every open lists the log |
| Publication commit outcome | Delta commit result | `reconcile` on the success path (`delta/publish.rs:308-315`) and read-back `admit` of members just written (`publish.rs:240`); `inspect` re-reads every commit file (`delta/attempt.rs:160-186`) | Every successful publish re-reads what it wrote |

**Deliberately opaque behavior:** domain algorithms (P3–P10, source projection), numerical
kernels, Ipopt. Their contracts are typed (`AlgorithmSpec`, `Operation`), which is right. The
defect is not opacity; it is that the transport *around* each opaque box repeats work.

**Identity behavior:** unchanged by this review. Semantic IDs, row tokens and content hashes
keep their contracts; the review only removes recomputation of identities that are already
carried by checked owners.

## 3. Semantic contracts and invariants

Only the contracts whose enforcement placement drives cost are listed.

| Contract | Representation | Enforcement boundary today | Failure behavior | Evidence |
|---|---|---|---|---|
| Local field validity | `FieldCheckedBatch` capability (`columnar.rs:46-53`) | Everywhere a batch crosses a function boundary (§2 row 1) | `RelationError::Validation` | Implemented; `just test` runs it under `force_validate` |
| Relational obligations (keys, FKs, ordinals, quantities, source spans) | `ObligationTemplates::bind` (`obligations.rs:381-414`) | Publication admission (`delta/admission.rs:36-55`) and policy requirements per execution (`policy.rs:300-320`) | Zero-row requirement or `Admission` error | Implemented; the local-values check (`obligations.rs:447-472`) is *also* persisted as a Delta CHECK (`delta/contract.rs:71-106`) and evaluated by delta-rs on write, then evaluated again by `admit` after commit |
| Semantic settings equality for reuse | `Witness` (`reuse.rs`), `SessionSemantics` (`engine_session.rs:175-191`) | Every `rebind` (`cache/admission.rs:50-100`) and every cache proof (`cache/admission.rs:133-188`) | Cache proof dropped | Implemented; correct but each comparison rebuilds the state it compares |
| Finite intermediate values | Volatile UDF around every `Float64` stage (`stages.rs:326-328`, `finite.rs:91`) | Per node per evaluation | `DataFusionError::Execution` | Implemented; declared as contractual (`finite.rs:89-90`) |
| Cooperative cancellation | `CancellationToken::checkpoint` (`resource.rs:99-105`) → tokio-util tree node (mutex, not atomic) | Per stage per evaluation (`stages.rs:134`) plus per finite UDF call (`finite.rs:60-63`) | `CanonError::Cancelled` | Implemented |
| Accounted allocation | Per-buffer `Bytes::from_owner` lease plus a process-global weak inventory (`owned_buffer.rs:107-116, 147-229`) | Every export, every cache read, every numerical evaluation | `ReserveError::Exhausted` | Implemented; blueprint §14.3 specifies this mechanism |

**Absence and uncertainty:** unchanged; not a performance concern.

**Equivalence requirements:** the review changes none. Where it recommends removing a
re-validation, the replacement is the *same* capability type carried through the boundary,
not a weaker check.

## 4. Derivation and execution design

The path a heater workflow actually takes (`tests/engine/tests/native_engineering_workflows.rs:105-146`
→ `tests/support/native_pipeline.rs:67-111` → `pse_compiler::native::model::from_documents_using`):

| Stage | What runs | Repeated work found (cited in §7) |
|---|---|---|
| Source binding | `candidate_checked` → `with_checked_workspace` (`roles.rs:82-114`); `relation_plan` scan | `admit_external` copies + validates raw ingress once — correct |
| Compose (per algorithm, `native.rs:50-199`) | `requirements` (plan_workspace + `RegistryRequirementPlanner`), `Operation::plan`, `cache_plan` ×2, `unpack_plans` ×2 | `cache_plan` calls `derive_plan_fields` twice and `bound_state` once each time (`cache.rs:57-65`); `seal` captures a `Witness` (another `bound_state`) per cache node; requirements re-lowered per algorithm |
| Artifact | `ArtifactPlan::new` derives every output plan again (`artifact.rs:59-65`) then `declare_relation_output` per output | One more full-graph admission over the accumulated plan set |
| Execute outputs | `execute_group` prepares and collects each output **sequentially** (`preparation.rs:475-491`, `artifact.rs:365-374`) | Each output: full `prepare_scoped` (≈10 traversals, §7 P04), `plan_execution` with `prepare_requirements` (another full preparation), physical planning |
| Algorithm body (`native/execution.rs:93-224`) | Captures every non-tuple child (stream → export → `admit_owned` per batch → `concat_reserved`), unpacks tuple members via `Layout::member` (full `admit`), builds a `candidate_roles` session (`retained` per role), parses documents, **loads `PhysicalInventory`** (`:193-201`), runs the algorithm, packs outputs | Inputs re-validated; physical catalogue re-scanned per algorithm; results packed into a one-row `LargeList<Struct>` per relation |
| Rules (inside P3–P10 native construction) | `iterate` → per stratum `Program::prepare` (reusable plans, good) → per round `execute` (`rounds.rs:200-297`) | Per round: window over the full accumulated assertion set, aggregate+self-join for facts, anti-joins of full-vs-full, `into_checked_relation` of every head, `replace` of every input |
| Numerical | `EvaluationProgram::evaluate` per Ipopt callback | Per node: `RecordBatch` construction, trait-object evaluate, volatile finite UDF with `to_array`, mutex cancellation check; per evaluation: new consumer, global inventory sweep, buffer rewrap |
| Publication | `commit_inner` (`publish.rs:200-298`) | Control table loaded twice, three reads of one control row, members opened sequentially with a `SessionState` rebuild each, `admit` re-scans members, `reconcile` on success |

**Relationship structures:** ownership (Arc owners), dataflow (plan DAG) and provenance
(support edges) are distinct and typed. No conflation found.

**Provider selection and limitations:** one DataFusion assembly per process, correct. The
limitation is that the assembly is *forked*, not *shared*: `EngineSession` owns a
`SessionContext` (`engine_session.rs:34`) but every preparation asks it for a fresh state.

**Boundary contracts:** Python receives Arrow C streams without copy (`pse-py/src/inspection/stream.rs:39-59`);
Ipopt receives raw slices without copy (`callbacks.rs:35-55`). Both are right. The Python
batch pump holds the GIL while pulling (`stream.rs:19-26`, no `detach`), a minor defect.

**Coherent publication:** unchanged and not reviewed for correctness beyond §7 P08's
observation that the success path re-reads.

## 5. Representative journeys

### Ordinary extension: one more registry invariant

Today: declare it in the registry; `program::compile` (`invariants/program.rs:32-107`)
rescans `registry.invariants()` and re-binds every selected query through
`bind_declared_queries` (`:96`) **every time any prepared plan is executed**, because
`prepare_requirements` runs inside `plan_execution` (`preparation.rs:616-620`). The new
invariant's cost is therefore paid once per output per stage, not once per compile. The
combined findings plan ends in `DISTINCT` + a full `Sort` with `fetch: None`
(`program.rs:178-195`) although the consumer only tests non-emptiness
(`contract.rs:305-319`). Target: compile the invariant program once per (registry, source
set) and cache the physical plan; require existence, not the sorted set.

### Meaningful change: one source relation changes

ADR-0068 deleted the stage memo ("Delete … stage memo", ADR-0068 Consequences), and no
replacement per-stage reuse exists in the compose path (`native/model.rs:120-183` builds the
whole graph every time). `ArtifactPlan::prepare_reuse` (`artifact.rs:260-351`) can reuse a
*published* output after a dependency-difference query, but the compile itself recomputes
every stage. DM-33's warning sign ("everything recompiles for every value change") is the
current state. The register still carries R-22 and R-01 as the triggers for finer reuse,
but the whole-stage granularity they assume no longer exists in code. Target: reinstate
stage-level reuse keyed to checked input owners (the `Witness` mechanism already exists for
plans; extend it to algorithm inputs).

### Boundary: a computed relation reaches a Python consumer

`open_publication` → `TableStream` → `PyRecordBatchReader` capsule. Zero-copy, correct. The
process assembles a second `Registry` (`pse-py/src/inspection/runtime.rs:48-50`) instead of
`pse_schema::registry()`, and `structure_rows` (`python/pse/codec/__init__.py:117-128`,
scout lead) documents a per-row `to_pylist` contract. Neither changes meaning.

### Interruption: cancellation during a solve

`Workspace::evaluate` checks the token once (`workspace.rs:123`), then `Stages::evaluate`
checks it per node (`stages.rs:134`) and the finite UDF checks it again per `Float64` node
(`finite.rs:60-63`). Each check locks a tokio-util mutex (`resource.rs:69-71`). Semantically
fine; a per-callback check would preserve the contract (the intermediate callback already
returns `false` to Ipopt) at a fraction of the cost.

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Pass (inspected scope) | Registry contract arena, checked owners, Delta log and `Witness` remain the single authorities; no second editable definition found | None |
| G2 — Semantic fidelity | Pass (inspected scope) | Re-validation over-applies checks; it does not drop meaning. Nested-field metadata is carried explicitly (`preserve_field.rs`) | None from this review |
| G3 — Validity | Pass (inspected scope) | Ingress admission (`admit_external`, Delta CHECK, obligations) rejects invalid states; the defect is redundancy, not a gap | None |
| G4 — Hidden behavior | Pass, with one note | `attach_native` mutates a process-global inventory inside a function callers treat as a pure export (`owned_buffer.rs:156-157, 218`); it does not change observable results. `SessionContext::state()` resets `query_execution_start_time` on every `bound_state()` (skill contract note 1), harmless because volatility is detected separately (`freshness.rs`) | Document the inventory as a declared effect or remove it (§11) |
| G5 — Consistency and recovery | Pass (bounded existing evidence) | `publication_each_object::every_actual_delta_write_boundary_preserves_a_complete_publication` passed in the 2026-09-19 run (51.7 s) | None |
| G6 — Transformation and reuse | Pass (inspected scope) | Cache proofs compare actual owners and semantic settings (`reuse.rs:99-140`); snapshot keys include the observed head (`snapshot.rs:250-255`); no stale hit constructed | None; reuse is *too conservative*, not unsound |
| G7 — Truthful capability claims | **Fail** | Blueprint §18.2 claims an instruction program with opcodes, slots and owned workspaces; the implementation evaluates DataFusion `PhysicalExpr` objects over one-row batches per node (`stages.rs:133-177`). Blueprint §5.4 claims "Arrow IPC files … (hot path, zero-copy mmap)"; the hot path rewraps every buffer through `Bytes::from_owner` and a global inventory on every export and cache read (`owned_buffer.rs:203-223`, `cache.rs:771-781`). Both claims are Proposed presented as Implemented | Relabel or implement (§11 rows 1 and 6) |

DM-39 (MUST) is additionally violated across the codebase: the only benchmarks measure
isolated kernels (`benches/benches/native_consolidation.rs`: local validation of one
column, row tokens, literal encoding, error trees, codegen), while every performance claim
in ADR-0055, ADR-0067 and the Plan 10 acceptance rows is end-to-end. That gap alone makes
the decision Revise.

## 7. Principle findings

Findings are grouped by cause. Severity order: correctness/capability (P01), then
semantic-authority-of-work (P02–P05), then measured/structural cost (P06–P12). Every
row's consequence is a concrete situation the cited code produces.

| # | Finding | Principle IDs | Concrete evidence | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| **P01** | **The numerical evaluation program is a per-callback DataFusion expression interpreter, not the instruction program §18.2 specifies.** | DM-38 **Violated**; DM-37 **Violated**; DM-59 **Violated** (claim vs implementation); DM-39 **Violated** | `stages.rs:133-177`: for every node, build a `RecordBatch` (`:145-150`), call `Arc<dyn PhysicalExpr>::evaluate` (`:173-176`), `into_array`; every `Float64` node is wrapped in a `Volatile` UDF (`:326-328`, `finite.rs:91`) whose body locks the cancellation mutex (`finite.rs:60-63`, `resource.rs:69-71`) and materializes `to_array` (`:69`); `stages.rs:134` locks it again per node. Per evaluation: `MemoryConsumer::new(...)` with a `String` (`expressions.rs:252`), `retained_buffer_bytes` walk (`:268`), `attach_reservation` → `attach_native` which locks a process-global mutex and sweeps the whole inventory (`owned_buffer.rs:156-157`) then rebuilds every output array (`:203-223`). `Workspace::evaluate` allocates one `Float64Array` per decision variable per evaluation (`workspace.rs:136-141`). Every callback evaluates the *whole* program including all Jacobian stages (`stages.rs:133`; `callbacks.rs:73-81` reads one scalar after `w.evaluate`). Derivatives are symbolic jets computed at compile time (`expressions.rs:126-141`), so the runtime shape is "N tiny DataFusion evaluations per iteration" | A square problem with 10⁴ nodes and 200 Ipopt iterations performs ≈2×10⁶ batch constructions, ≈4×10⁶ mutex acquisitions and 200 global inventory sweeps to compute what a compiled tape does with 10⁴ multiply-adds per iteration. The bits-memo (`workspace.rs:127-135`) merges the four callbacks per point but cannot remove per-node overhead | Lower the arena to a **flat tape**: `Vec<Instr{opcode, out, in[..]}>` over an `f64` slot vector owned by the workspace; region masks as branch instructions; finite check as an inline compare that records the failing node; cancellation checked once per callback (Ipopt already receives `false`); selective evaluation of the objective/constraint/Jacobian output sets; keep symbolic derivatives but emit them into the same tape. `PhysicalExpr` remains the *batch* evaluator for DataFusion-side use. This implements §18.2 as written | Microbenchmark: evaluations/s for N=10², 10³, 10⁴ nodes, before/after; differential test tape-vs-`PhysicalExpr` on the existing `pivot_unit` fixtures and `native_solve` cases (bitwise equality per DM-40 ordered-operation policy) |
| **P02** | **Already-checked values are re-validated at every internal boundary.** The `FieldCheckedBatch` capability exists precisely to carry evidence, and the code discards it and re-mints it. | DM-26 **Violated**; DM-07 **Unresolved** (validation boundary undeclared, so it is "everywhere"); DM-56 | Re-admission sites over data that was already a `FieldCheckedBatch` or a completed native result: `native/layout.rs:142` (`Layout::member` → `admit`) for every tuple member of every algorithm input; `native/layout.rs:166` (findings/derivations batches) plus `concat_reserved` (`:169-175`); `session/input.rs:91-99` (per-batch `admit_owned` then `concat_reserved` on capture); `preparation.rs:1075-1093` (`checked_relation`: `admit_owned` per batch then concat); `p3/config.rs:210-241` (`merge_generated` unions the whole accumulated relation with one generated row, executes it, and re-admits everything — per generated row); `native/source.rs:40-46` (registry reflection constants re-admitted on every source run); `rules/strata/rounds.rs:456-458` (`into_checked_relation` per head per round); `native_outputs.rs:162-180` (`rows()` flushes, re-admits and decodes the accumulated relation per call — scout lead, verified). Each `admit` costs `validate_schema` (rebuilds the schema with two JSON serializations, `arrow.rs:62-89`, and re-validates every field's extension metadata, `validate/field.rs:480-643`), `validate_full` per column (`prepared.rs:213`), a linear cache scan that clones every field for comparison (`prepared.rs:95-97`, `279-290`), and the occurrence walker's per-row `format!` path strings even when nothing fails (`occurrences.rs:610-616`, `750-753`, `774`) | For a relation with R rows and C columns crossing S stage boundaries, validation work is O(S·R·C) with a string allocation per row per column per crossing; the heater compile pays it for every one of dozens of relations at every one of nine stages. `checked_literal` (`output.rs:939-957`) runs the full path per scalar literal at 30+ compiler call sites | Declare the validation boundary: **ingress** (`admit_external`, Delta read, Python/authoring) and **new-value producers** (a native plan output is admitted once by `declare_relation_output` semantics or one `admit_owned`). Everything else transports `FieldCheckedBatch` by `Arc` and uses `check_declaration` (pointer/handle equality) only. Make `Layout::pack/member` carry the checked members alongside the tuple. Cache `relation_schema` per contract handle. Replace per-row `Location` strings with lazy paths built only on failure. `checked_literal` → validate the scalar directly without a batch | Counter test: number of `PreparedLocalContract::evaluate` calls per heater compile (target: one per produced relation); existing negative admission tests unchanged; `evaluate-65536` bench before/after |
| **P03** | **`SessionContext`/`SessionState` are used as disposable value objects; the long-lived session facilities DataFusion is built around are bypassed.** (User's specific concern; confirmed.) | DM-26 **Violated**; DM-38 **Violated**; DM-19 (declared bindings exist but are re-derived) | `EngineSession` owns a `SessionContext` (`engine_session.rs:34`) but `bound_state()` (`:540-565`) builds a *new* `SessionState` on every call: `context.state()` clone (`:544`), `EffectiveSettings::resolve` (config clone + builder rebuild, `config.rs:185-210`), `effective_policy()` recomposed from all policies × all bindings (`policy.rs:206-231`), catalog list forked (`binding.rs:429-443`), `cache_service::inspection::bind` (another builder rebuild). 23 production call sites, including inside `Witness::capture` and `Witness::matches` (`reuse.rs:48, 132`), which `rebind` calls **twice per cache node** on every `prepare`/`derive_plan_fields` (`cache/admission.rs:66, 83`). `scalar_function()` clones the entire state to look up one UDF (`engine_session.rs:322-332`) and is called per prepare (`preparation.rs:312, 388`). `NativeExecutionContext::bind` and `execution_state` rebuild the state again (`execution.rs:98-100, 181-183`). `EngineSession::clone()` deep-copies `settings: BTreeMap<String, Option<String>>` (~250 entries) and `function_names` (~600 strings) (`engine_session.rs:44-46`) on every fork: per prepare (`preparation.rs:273`), per workspace/role binding (`roles.rs:82, 116, 155`), per `with_purpose`/`with_policies`. Per SQL binding, a private `MemoryCatalogProviderList` is built (`native.rs:104-152`). Facilities unused: `state_ref()` (shared lock, no clone), `register_table` + `table()` once per snapshot, `register_udf`, a reusable `TaskContext`, prepared/cached physical plans; the deployment `CacheManager` is built with **all limits 0** (`resources.rs:57` → `cache_service/mod.rs:214-219`) and the production budget leaves statistics, listing and the Parquet predicate cache at 0 and `concurrent_loads = 1` (`cache_service/policy.rs:55-63`, `config.rs:62-66`). What *is* used well: `CacheFactory` seam, `ExtensionPlanner`/`UserDefinedLogicalNode`, shared `RuntimeEnv`/pool, `datafusion-tracing` | Per prepared output the engine performs on the order of 4–8 `SessionState` rebuilds and 2·(cache nodes) `Witness` comparisons, each of which re-reads ≈250 config entries and re-partitions them (`config.rs:217-221`). With ~2 cache nodes per algorithm and 9 algorithms, a P10 output pays ~36 state rebuilds before its optimizer runs; multiply by the number of outputs in `execute_group`. None of this work depends on the plan | One **immutable `EngineAssembly`** per (factory generation × bindings generation): build the `SessionState` once with tables registered in the snapshot catalog, effective policy composed once, semantic settings materialized once; hand out `Arc<SessionState>` and one `TaskContext` per execution. `Witness` becomes a pointer comparison of the assembly. Forks (`with_checked_workspace`, roles) add providers to a copy-on-write catalog layer without touching settings or functions. Use `state_ref()` where a clone is not needed. Enable the native caches with real limits and a listing TTL. Keep the `CacheFactory` and extension planners exactly as they are | Assertion: `SessionStateBuilder::build` count per compile (tracing counter) ≤ generations; `Witness::matches` no longer calls `bound_state`; existing `scoped_reuse_unit` tests unchanged |
| **P04** | **Each prepared plan is traversed ~10 times and requirements are re-lowered per execution.** | DM-26 **Violated**; DM-38 | `prepare_scoped` (`preparation.rs:183-285`): `rebind` (rewrite traversal), `restore_semantic_fields` (derive + admit traversals), `plans_require_fresh` (`:333`), `admit_effects` (`:335`), `stage` (`cache/logical.rs:306-389`: visit + protect + rewrite), analyzer, `expand` (`:249`), `plans_require_fresh` again (`:251-253`), `admit_effects` again (`:254`), optimizer, `finalize_native_fields` (`:296-316`: restore + `materialize_nested_fields` + `expand`). Traversals clone plan nodes (`traversal.rs:130, 171, 179`) and `Scope` (with `Vec<Expr>` outer refs) per child (`:212, 248`); `admit_expression` calls `to_field` for every sub-expression (`admission.rs:747`, plus `:730-731`), quadratic in expression depth. `plan_execution` re-lowers policy requirements on every execution (`preparation.rs:616-620` → `policy.rs:300-320` → `RegistryRequirementPlanner` → `program::compile`), i.e. a second full `prepare_scoped`. `native.rs:78-84, 154-160` runs the planner twice per algorithm and `unpack_plans` twice (`:142, 187`) | The traversal count multiplies the expression volume of the accumulated compile graph; combined with P03 it is the structural reason a P10 compose is minutes before any data moves | Fuse admission into one pass per plan revision (derive → admit → freshness → effects in one postorder), memoize `to_field` per node, and compute requirements once per (assembly, source set) and attach the prepared requirement plan to the `PreparedComputation` | Traversal counter per prepare (target ≤ 3: admit, analyze/optimize, finalize); requirement compile count per compile = number of distinct source sets |
| **P05** | **The stage pipeline is encoded as a UNNEST-of-tuple plan graph, which forces materialize→pack→unpack→re-admit at every stage with no relational benefit.** | DM-38 **Violated**; DM-36 **Violated**; DM-58 **Violated** (machinery without leverage); DM-18 | Every algorithm result is packed into one row of `LargeList<Struct>` per relation (`native/layout.rs:195-213`); consumers get Projection→`unnest`→`unnest`→Projection (`:64-109`) over a `Cache`d `ExecutionContract`(`Operation`) (`native.rs:136-141, 176-179`); `Cache`, `ExecutionContract` and `Operation` all report `prevent_predicate_push_down_columns` = all columns (`cache.rs:331-338`, `contract.rs:135-142`, `operation.rs:269-275`) and `UnknownPartitioning(1)` (`cache.rs:837-842`, `operation.rs:370-375`); the algorithm body captures each child by fully draining it (`native/execution.rs:112-148`) and re-admits (P02). `execute_group` runs outputs sequentially (`preparation.rs:481-489`). The compose loop accumulates *every* stage output as an artifact output (`native/model.rs:131-135, 156`), so each is prepared and executed separately | DataFusion's optimizer sees only trivial Projection/Unnest wrappers between opaque leaves; it cannot push, prune, or parallelize across a stage; the plan machinery adds P03/P04 costs and the unnest copies without buying any relational optimization. The 900 s heater timeout is the observable consequence | **Typed stage handoff.** Stages exchange `Arc<FieldCheckedBatch>` bundles directly (the `AlgorithmOutput` type already is that). DataFusion is used *inside* a stage for relational work over tables registered once in the assembly (P03), and *around* the artifact for publication. Keep `Operation`/`ExecutionContract` for genuinely relational operations (invariants, source projection queries, Delta writes), not as the transport for opaque algorithm results. Execute independent outputs/stages concurrently under the shared pool | End-to-end: heater source→P10 wall time, cold and warm, with per-stage spans (already emitted at `native/execution.rs:85-92`); differential equality of all P10 relations against the current path on the existing fixtures |
| **P06** | **Compiler internals issue per-scalar DataFusion queries and keep quadratic in-memory scans.** (scout leads, cited lines verified) | DM-38 **Violated**; DM-36 | `p3/config.rs:139-167`: every configuration lookup forks a session, builds a plan, prepares and executes it (`native::rows` → `native_rows.rs:121-134`: prepare + execute + `checked_relation`); callers: `instances.rs:192-194, 210-212` (per instance), `:360-390` (two queries per child parameter binding). `PhysicalInventory::load` re-scans, sorts and re-admits every physical relation on **every algorithm invocation** (`quantity_relations/inventory.rs:49-89`, called at `native/execution.rs:193-201`). `Plans::require` collects every violating row to test emptiness (`native_construction.rs:257-278`, no `LIMIT`); `retain_with_count` forks a session per fixpoint step (`:287-315`). `OutputRows::retain` concatenates the accumulated relation on every append (`native_outputs.rs:261-297`, O(n²)); `append_checked` walks rows with a closure (`:199-204`). `p7/paths.rs:27-28` clones the inventory and rebuilds 13 indices per expression; `p4/predicates/scalar.rs:115-120` rebuilds a reverse node map per scalar evaluation | For a template with I instances and B bindings, P3 alone issues ≈6·I + 2·B full DataFusion round trips (each with P03/P04 overhead); `merge_generated` makes instance configuration O(I²) in rows; the physical catalogue (units, quantity types, elements) is decoded from Arrow nine times per compile | Load configuration relations once into indexed Rust structures (or one join plan per relation) at P3 entry; load `PhysicalInventory` once per compile and share it (the `AlgorithmContext.physical` field already exists for sharing); `require` → `LIMIT 1` existence (the `reject` helper at `native_rows.rs` already does this); accumulate outputs in builders and concat once at `finish`; replace linear inventory scans with `HashMap` indexes built once per pass | Query count per P3 configure (tracing counter) ≤ number of configuration relations; heater P3 wall time |
| **P07** | **The rules engine re-derives full relations every round, re-admits every head, and recompiles invariant programs per call.** (round loop verified; scout leads for trace overhead) | DM-38 **Violated**; DM-26; DM-33 | Per round (`rounds.rs:200-297`): `next_facts`/`next_assertions` map clones (`:220-221`); a fresh empty admitted batch per head (`:222-226`); for each head, `merged` = window `row_number() OVER (PARTITION BY assertion_id)` over the **full** accumulated assertion set (`relational.rs:180-198`), `facts` = aggregate + self-join over the full set (`:261-325`), `added`/`delta` = anti-joins of full-vs-full (`:218-224`); each result fully collected and `into_checked_relation`'d (`:456-458`); every input replaced in three loops (`:276-290`). `execute_empty` collects all batches to test emptiness (`:442-447`). Invariant programs are recompiled on every planner call (`invariants.rs:24-43` → `program.rs:32-107`), combined by a left-deep union fold (`:171-177`) under `DISTINCT` + full `Sort` (`:178-195`). Reusable round plans (`prepare_reusable`) are correctly used for the hot loop (`:418-421`) | Round cost is O(|accumulated|) not O(|delta|) on the head side, so a fixpoint with k rounds costs O(k·|A|·log|A|) in sorts plus k full re-validations of every head relation; every prepared plan execution re-lowers the invariant program | Head side: dedup and difference against an index of the accumulated set (hash anti-join with the small delta as probe; DataFusion does this when the build side is the accumulated set), emit only `added` rows, append to the accumulated owner without re-admission; `execute_empty` → `LIMIT 1`; invariant program compiled once per (registry, source set) and cached; findings root = union-all + existence, sort only for the diagnostic report | Rows scanned per round (metrics on the reusable plans) proportional to delta; invariant compile count per compile |
| **P08** | **Delta and cache paths re-read what they already hold.** (scout leads, cited lines verified) | DM-26 **Violated**; DM-32 (conservative to the point of no reuse) | `open_snapshot` performs an uncached `get_latest_version` **before** the cache lookup (`snapshot.rs:227-239`) and keys entries by `observed_latest` (`:250-255`), so any later commit misses every pinned version; seeding applies only when `version == observed_latest` (`:267-271`); a second listing after each fill (`:281-286`). Production policy leaves CRC replay and checksum seeding off (`catalog/cache_service/policy.rs:34-41`). `bind_members` opens members sequentially (`publication.rs:224-238`), each building a provider (`provider.rs:117-135`) after a `SessionState` rebuild + `CacheManager::try_new` (`engine/cache_service/mod.rs:268-302`). `commit_inner` loads the control table (`publish.rs:223`), reconciles (`:227`), admits the parent (`:231`), verifies inputs (`:236`), binds members (`:237`), then **re-admits the members it is about to publish** (`:240` → `admission.rs:102-137`, whose local-values obligation duplicates the Delta CHECK persisted at `contract.rs:102-106`); on success it `reconcile`s again (`:308-315`) despite the comment at `:295-296`. `MemberAttempt::inspect` reads and JSON-parses every commit file from base to latest (`attempt.rs:160-186`). Statistics-less DELETE counts the table twice (`dml/execution.rs:159-168`) | A publication of M members performs ≥M sequential opens with state rebuilds, M local-value scans that Delta already enforced, and 3–4 reads of a one-row control table; a workflow that reads a pinned version after each commit never hits the snapshot cache | Cache snapshots by (store, version) with the listing done only when `version` is `None`; open members concurrently with one shared `Arc<SessionState>`; trust the validating write route for local values and keep publication admission for cross-relation obligations only (the `contract.rs:39-40` intent); reconcile only on ambiguous outcomes; read receipts from the commit metadata of the version just written rather than re-listing the range; enable CRC replay/checkpoint seeding by default | Object-store request counts per publish (the `ObservedStores` wrapper already instruments); cache hit ratio on repeated pinned opens; `publication_each_object` and the concurrent-writer tests unchanged |
| **P09** | **The buffer-lease machinery costs a lock, a sweep and a rewrap per batch, and is applied repeatedly to the same buffers.** | DM-37 **Violated** (zero-copy claim ignores wrapping cost); DM-39; DM-28 (undeclared global effect) | `attach_native` (`owned_buffer.rs:147-229`): walks every buffer (`:153-154`), locks a process-global `Mutex<BTreeMap>` and sweeps it (`:156-157`), rebuilds every array (`:203-223`), one `Bytes` owner per buffer (`:214-217`). Called by `OwnedRecordBatch::export` per streamed batch (`preparation.rs:916-920`), by `retain_allocations` at cache fill (`cache.rs:683`) **and again on every cache read** together with `retain_owner` (`cache.rs:771-781`), which locks the global mutex once **per buffer** (`owned_buffer.rs:320`). `RetainedBuffers::additional` re-sums its entire map per batch (`:283-285`), quadratic across a stream. `copy_batch` walks buffers three times (`:486-489, 504-506, 517`) | For a stream of B batches each with K buffers, export costs O(B·(K + |inventory|)) under a global lock; a cached producer read by C consumers rewraps B·K buffers C more times; nothing here is proportional to useful work | Account at the granularity DataFusion already uses: one `MemoryReservation` per completed result or per operator, sized once from `get_array_memory_size`, released with the owning `Arc<Vec<RecordBatch>>`; drop the per-buffer wrapper and the global inventory. Shared slices are then counted per owner, which is the DM-31 conservative direction. Keep `copy_batch` for genuine ingress isolation | Pool-accounting tests (`every_nested_dictionary_and_validity_buffer_keeps_the_reservation` and siblings) rewritten at result granularity; export throughput bench (batches/s) before/after |
| **P10** | **Concurrency is configured to one everywhere that matters.** | DM-35 **Unresolved** (no declared parallel contract); DM-38 | Workflows run with `pool_threads = 1`, `target_partitions = 1` (`tests/support/workflow_runtime.rs:30-34`); nextest serializes engine tests (`.config/nextest.toml` `serial`/`engineering`); `execute_group` is sequential (`preparation.rs:481-489`); members open sequentially (P08); `concurrent_loads = NonZeroUsize::MIN` in both cache budgets (`cache_service/policy.rs:44, 62`); every custom `ExecutionPlan` is `UnknownPartitioning(1)` (P05); the finite guard makes every `Float64` stage volatile (P01), which also blocks any DataFusion-side simplification. Blueprint §18.8 says defaults give the pool to DataFusion outside a solve | A 32-thread workstation runs a compile on one thread; independent outputs, members and stages wait on each other; nothing in the design prevents parallelism — the configuration and the sequential `await` loops do | Default `target_partitions` to the pool size outside solves (as §18.8 already states); `join_all` bounded by the pool for outputs and members; keep deterministic single-partition mode as an explicit test policy, not the workflow default | The same workflow at 1 vs 16 partitions; equality of results (order-insensitive where declared) |
| **P11** | **Identity and MathIR paths repeat traversals and registry scans.** (scout leads, cited lines verified for `owned_buffer`; MathIR/quantity lines are scout-verified only) | DM-38; DM-36 | `pse-mathir/src/canonicalize.rs:121-122` runs a postorder over all nodes whose result is discarded; `literal_context.rs:31-46` scans every registry quantity type and runs a full inference per candidate per additive slot; `pse-quantity/src/registry.rs:225-228` `resolve_key` and `:239` `operations_for` are linear scans invoked per inferred node; `index.rs:116` performs a linear `find` on a `BTreeSet`; `pse-ids/src/canon/stages.rs:20, 59-62, 81` and `api.rs:97, 116-117` materialize the batch four times before hashing although `frame.rs:58-62` implements `FrameSink for blake3::Hasher`; `relations/load.rs:312-318` re-canonicalizes to validate a stored artifact | Compile-time costs linear in registry size per expression node and per literal; canonicalization copies scale with relation size. Lower in severity because canonicalization runs once per publication and MathIR once per compile | Index registry lookups by key; memoize `resolve_key`; stream the canonical frame into the hasher; validate stored canonical rows by hash, not by recomputation | Canonicalization bench (`canonicalization.rs` exists) before/after; MathIR canonicalize time on the heater equation set |
| **P12** | **Python boundary: sound design, three avoidable costs.** (verified `stream.rs`, `runtime.rs`; python leads scout-verified) | DM-37 | `TableStream` pumps batches with `block_on` **while the caller holds the GIL** (`pse-py/src/inspection/stream.rs:19-26`, no `py.detach`); `runtime.rs:48-50` assembles a second `Registry` instead of the process `OnceLock`; `python/pse/codec/__init__.py:117-128` structures rows from `to_pylist()` per row; `governance.py:143-154` walks and re-validates every generated module at `import pse` | Consumers that overlap Python work with native reads serialize on the GIL; import time grows with the contract surface | `detach` around the pump; use `pse_schema::registry()`; keep `structure_rows` as an explicit convenience, not the documented path; move governance checks to `just quality` | Import-time measurement; a two-thread reader test that overlaps |

**Additional verdicts on the inspected scope.** DM-02, DM-11–15, DM-23 **Satisfied** (one
authority per fact; identity independent of layout). DM-20 and DM-28 **Satisfied** except for
the inventory note in G4. DM-24 **Satisfied**: rewrites (`normalize_casts`, `materialize_nested_fields`,
`SemanticFields`) declare what they preserve and are tested. DM-32 **Satisfied** where reuse
exists (`Witness`, snapshot keys); DM-33 **Violated** for the compile as a whole (§5). DM-40
**Satisfied** (ordered arithmetic, no `target-cpu=native`, `panic=unwind`). DM-44/DM-43
**Satisfied** for the extension surfaces read. DM-52 **Satisfied** (generated builders/views).
DM-53/DM-54 **Satisfied** for admission and publication; **Unresolved** for performance
regressions, which nothing detects (§9). DM-56/57 **Satisfied** for the semantic model;
**Violated** in P05 where the plan-transport machinery re-expresses one meaning (a stage
output) in three physical forms (checked batch, packed tuple, unnest plan).

**Applicability:** groups 5–8 (compilation, execution, reuse, performance) carry the review
and were applied everywhere. Groups 1–3 were applied to confirm that removing repeated work
does not move authority; they yielded no findings. Group 9 applied to the Python and Ipopt
boundaries (P01, P12). Group 10 applied only to observation cost (Contract vs Diagnostic
policy; no finding beyond the prior review's F09). Group 11 applied through DM-53/54 above.
Group 4 (declarative composition) was not applied: the review proposes no change to what is
declared. DM-51 (migration) does not apply: no stored-format change is proposed.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risk | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current baseline**: every stage output is a native plan; every boundary re-admits; sessions rebuilt per preparation; per-buffer leases; DataFusion-expression numerics | One meaning of "stage output" in three forms; validation logic in one place but invoked everywhere | Correct in the attacked scope | Already paid; each new relation or stage inherits the whole cost stack | Measured: heater 900 s timeout; conservation 360 s timeouts; median test 0.058 s | Rejected as target: the costs are structural, not tunable |
| **Prior review's proposal (2026-09-19)**: keep the native-plan transport, carry checked evidence through tuples, retain intrinsic plan facts, freeze bound assemblies, consume completed requirements | Removes the internal re-admission and part of the state rebuilding; keeps the pack/unpack and per-output preparation | Low: all changes inside existing modules | Moderate; touches relations, engine, compiler, rules | Structural work removal; no speedup measured | Necessary but insufficient: it leaves P05 (the transport itself), P01 (numerics), P09 (leases) and P10 (concurrency) in place, and P04's traversal count survives because the plan graph still exists |
| **Optimal design (recommended)**: (a) one immutable `EngineAssembly` per generation with tables registered once and `Arc<SessionState>` shared (P03); (b) validation at ingress and at new-value producers only, `FieldCheckedBatch` transported by `Arc` (P02); (c) **typed stage handoff** — algorithms consume and produce checked-batch bundles directly; DataFusion plans are used for relational operations inside stages and for publication, not as the inter-stage transport (P05); (d) invariant programs compiled once per (assembly, source set) with existence checks (P04, P07); (e) a compiled `f64` tape for the numerical program (P01); (f) result-granular memory accounting without a global inventory (P09); (g) snapshot cache keyed by (store, version), concurrent member opens, no post-commit read-back except on ambiguity (P08); (h) pool-sized partitions and bounded `join_all` for independent work (P10) | One form per meaning; a new stage is one `Algorithm` impl plus its declared ports (unchanged from today); a new invariant is one declaration compiled once | Moderate: (c) changes the dependency capture for reuse (the `Witness`/dependency machinery must key on checked input owners rather than plan nodes); (f) changes what "accounted" means for shared slices (conservative); (b) needs a compile-time-enforced distinction between raw and checked inputs, which the type already provides | Substantial but subtractive: most of `native/layout.rs`, `native/value.rs`, the tuple unpack path, the per-buffer inventory, and the repeated admission call sites are deleted; the tape is new code in `pse-numerics` | Proposed; §9 names the measurements that must accompany it | **Selected**. It is the smallest design that preserves every §2/§3 contract and removes every structural repetition in §7 |
| **Simpler viable alternative**: keep everything, raise thread counts, enable the caches, add `LIMIT 1`s, memoize `bound_state` per session | No change to duplication | None | Days | Would remove some constant factors; leaves O(S·R·C) validation, O(I²) configuration, per-node numerics | Rejected as the target; acceptable as the first slice of §11 if it is measured |

**Abstractions justified by current needs:** the registry/contract arena, `FieldCheckedBatch`,
`Operation`/`ExtensionPlanner`, `CacheFactory`, the Delta publication control table, the
`Witness` reuse proof, `datafusion-tracing` instrumentation. All stay.

**Machinery whose leverage is not demonstrated (DM-58):** the one-row `LargeList<Struct>` tuple
transport and its unpack plans (`native/layout.rs`, `native/value.rs`); the process-global
buffer inventory (`owned_buffer.rs:107-116`); per-buffer `Bytes::from_owner` leases; the
volatile finite UDF as the finiteness mechanism; `PhysicalExpr` as the solver-callback
evaluator. Each exists to satisfy a stated principle (native-only execution, exact accounting,
guarded numerics) and each can satisfy the same principle with a cheaper mechanism.

**What remains ordinary code:** P3–P10 algorithms, MathIR canonicalization, quantity
inference, the Ipopt driver, Delta commit protocol. The review recommends no new DSL.

## 9. Verification and measurement plan

| Claim or risk | Evidence label today | Test / analysis / benchmark | Conditions and expected result | Current result or gap |
|---|---|---|---|---|
| Heater source→P10 completes | Measured (timeout) | `native_engineering_workflows heater_*` with `PSE_TEST_PHASE_TIMINGS=1` and the `pse.algorithm` spans (`native/execution.rs:85-92`) | Cold and warm, 1 and 16 partitions, `force_validate` on and off; per-stage `input_seconds`/`algorithm_seconds`/`packing_seconds` attributed | 900 s timeout; the only attribution is this review's probe: composing source+P3 with no data execution took 10.08 s (§1). The `pse.algorithm` spans did not print because the test subscriber has no span-close events; enable `with_span_events(FmtSpan::CLOSE)` in the fixture. **First measurement to run** |
| Validation happens once per produced relation | Implemented (violated) | Counter on `PreparedLocalContract::evaluate` and `FieldCheckedBatch::admit*` per compile | ≤ produced relations + ingress batches | Not instrumented |
| Session state is built once per generation | Implemented (violated) | Counter on `SessionStateBuilder::build` per compile | ≤ generations + executions | Not instrumented |
| Numerical callback cost | Implemented (violated) | Criterion: evaluations/s at N ∈ {10², 10³, 10⁴} nodes, before/after tape; differential bitwise equality tape vs `PhysicalExpr` on `native_solve` fixtures | Tape ≥ 100× fewer allocations per evaluation; identical residual/Jacobian bits under the ordered policy | `benches/native_consolidation/numerical.rs` exists; its scope must be checked before reuse |
| Publication opens each member once | Implemented (violated) | Object-store request count per publish via `ObservedStores`; snapshot cache hit ratio | Requests ∝ members; second pinned open hits | Not instrumented |
| Reuse invalidation remains sound after P02/P03 | Tested (existing) | `scoped_reuse_unit`, `incremental_equals_clean_p0_p3`, `changed_species_commit_invalidates_p3_and_matches_a_clean_complete_run` | Unchanged pass | Pass in the 2026-09-19 run for the first; the third took 209.9 s (nextest note) |
| No DM-24 regression from removing internal re-admission | Tested (existing negative tests) | Raw-ingress rejection tests, `changed_projection_cannot_reuse_parent_admission` (`columnar.rs:598-611`), hostile metadata tests | Unchanged pass | Pass |
| Memory accounting at result granularity is conservative | Proposed | Rewrite `owned_buffer` tests at result granularity; assert pool ≥ RSS-attributable Arrow bytes on the heater compile | Accounted ≥ actual for owned results | Not started |
| Performance regression control (DM-60) | Missing | A `bench-smoke`-style gate with *ratio* thresholds against a stored baseline for: validation calls, state builds, traversals, evaluations/s, heater wall time | Fails on ×2 regressions | `just bench-smoke` runs benches once with no gate |

**Open checks from scout leads not independently verified** (record, do not treat as
findings): `pse-quantity` `IndexSet::get` linear scan; `pse-mathir` five traversals per
canonicalize; `python/pse/_transfer.py` O(columns²) schema rebuild; `delta/layout.rs:294-302`
double cast per read batch; `cache_service/resident.rs:338-357` per-read listing.

**Cost accounting (material categories):** construction (P02, P06), preparation (P03, P04,
P07-invariants), transfer (P05, P09), execution (P01, P07, P10), storage (P08), inspection
(Diagnostic policy; prior review F09). Memory: the pool is sized at 32 GiB for tests; RSS is
not measured per stage.

## 10. Exceptions and unresolved decisions

No SHOULD exception is requested by this review. The following are the documented policies
the selected design would change; each is a decision for the maintainer when this review is
converted into a plan. None of them is a D1–D14 decision; the closest are D10 (Arrow/DataFusion
default) and D11 (native numerics own execution layouts), both of which the target *satisfies
better* than the current code.

| Policy text today | Where | What the optimal design needs | Level |
|---|---|---|---|
| "There is no stage executor: DataFusion plans, schedules and executes every dependency" (`native.rs:5`); "native DataFusion plans, expressions, providers and execution implement all product data operations" | blueprint §3.3.3, §14.3.1 ("Native plan sharing is not an execution-once guarantee. … Materialize at named stage artifacts, shared expensive work, fixed-point state, explicit checkpoints and specialized algorithm boundaries"), ADR-0067, ADR-0068 | Amend to: native plans implement *relational* operations; opaque finite algorithms exchange checked Arrow bundles directly through typed stage contracts; the artifact/publication boundary remains a native plan. §14.3.1's own materialization rule already permits this; the sentence that forbids a stage executor does not | Blueprint amendment + short ADR superseding the "no stage executor" clause of ADR-0067; design review required (changes a D10-adjacent SHOULD) |
| "Complete residual obligations admit the whole output bundle"; "untrusted restore repeats substantive admission"; "P2 consumes unpublished schema-admitted candidate batches" | blueprint §14.3.1, §5.4; ADR-0052 | Declare the trust boundary explicitly: `FieldCheckedBatch` minted inside the process is trusted evidence; re-admission happens at ingress, reopen and new-value producers only | Blueprint wording; no ADR change (ADR-0052 already distinguishes trusted immutable validation from untrusted restore) |
| "The safe `pse-ids` adapter uses `Bytes::from_owner` to retain an Arrow buffer and lease, recursively covering data/validity/dictionary/child buffers … No unsafe owner construction or pointer-deduplication ledger is used" | blueprint §14.3; ADR-0055 | Account at result/operator granularity; delete the per-buffer wrapper and the global weak inventory (which *is* a pointer ledger). ADR-0046's guarantee (accounted consumers, not RSS) is unchanged | ADR-0055 supersession + blueprint §14.3 amendment; design review required (changes the accounted-allocation contract) |
| §18.2 evaluation program (instructions, slots, owned workspaces, reverse sweeps) and §18.9 matrix | blueprint §18.2; ADR-0047 | No wording change: **implement it**. Relabel the section's evidence from Implemented to Proposed until the tape lands | None beyond the plan's Verification section |
| "Arrow IPC files for compiled and runtime relations (hot path, zero-copy mmap)" | blueprint §5.4 (last line) | Delete or relabel: the hot path is Delta/Arrow streams with owned leases; IPC mmap is not the implemented transport | Blueprint wording (stale under ADR-0068) |
| `for_memory` cache budgets: statistics 0, listing 0, predicate cache 0, `concurrent_loads` 1, CRC replay 0; deployment `CacheManager` unbound | `cache_service/policy.rs:55-63`, `catalog/cache_service/policy.rs:34-41`, `resources.rs:57` | Deployment defaults with real limits and a listing TTL; a measured matrix, per ADR-0070's own Confirmation | Configuration; register row under ADR-0070 |
| Test/workflow runtime: one worker, one partition, serial groups | `tests/support/workflow_runtime.rs`, `.config/nextest.toml` | Deterministic single-partition as an explicit fixture policy; workflows at pool size | Test policy; no ADR |
| Register R-22/R-01: finer memoization only after "whole-stage recomputation dominates" | `docs/adr/register.md` | The premise no longer exists (no stage memo since ADR-0068); rewrite the trigger around the stage-handoff design | Register edit under ADR-0068 |
| `force_validate` on every test invocation | AGENTS.md invariant; justfile `validate` | Keep for correctness runs; performance runs must also be recorded *without* it, otherwise every Arrow construction carries validation cost the product never pays | Measurement policy in §24.3 |

**Owner:** the maintainer through the follow-up plan. **Revisit trigger for every row:** the
measurements in §9 exist and the ratio gate is in CI.

## 11. Decision and implementation changes

**Decision: Revise.** Not because the semantics are wrong — they hold everywhere this review
pushed — but because DM-39 and G7 fail on the record: performance is claimed end-to-end and
measured nowhere, and two blueprint capabilities describe code that does not exist. The
selected target (§8 row 3) removes the structural repetition without touching a single
authority, identity or validity contract. Order the work so every slice is measured against
the previous one.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 — measure first | Land the §9 instrumentation (validation/state-build/traversal counters, per-stage spans already present, evaluations/s bench, object-store request counts) and record the heater/mixer baselines cold/warm with and without `force_validate` | DM-39, DM-59, DM-60 | Baseline table committed under `docs/design_review/evidence/` with commands and conditions | Ratio gate in `just bench-smoke` |
| 2 — numerical tape | Implement §18.2 as written in `pse-numerics`/`pse-backend-native`: tape over `f64` slots, branch regions, inline finite check, per-callback cancellation, selective outputs; keep `PhysicalExpr` for batch/DataFusion evaluation | DM-38, DM-37, DM-40, G7 | Differential bitwise equality on existing solve fixtures; evaluations/s ≥ 100× | Existing `guarded_branch_execution`/`numerical_policy_conformance` fixtures; new tape-vs-expr differential test |
| 3 — one assembly per generation | `EngineAssembly` with shared `Arc<SessionState>`, tables registered once, policy/settings materialized once, `Witness` = owner pointer comparison, `TaskContext` reuse, caches enabled with limits | DM-26, DM-19, DM-32 | State-build counter ≤ generations; `scoped_reuse_unit` unchanged | Policy-change and provider-change invalidation tests (exist) |
| 4 — validation boundary | Declare ingress + new-value producers; delete internal re-admission at the eleven sites in P02; carry checked members with tuples until step 5 removes tuples; cache `relation_schema`; lazy failure paths | DM-07, DM-26, DM-56 | Validation counter ≤ produced relations; negative admission tests unchanged | Raw-ingress rejection tests; hostile metadata tests |
| 5 — typed stage handoff | Algorithms exchange checked bundles; plans only for relational work and publication; delete `layout.rs` pack/unpack, `value.rs` tuples; concurrent outputs; stage reuse keyed to checked input owners | DM-38, DM-36, DM-58, DM-33 | Heater P10 wall time (cold/warm) and per-stage spans; all P10 relations equal to the current path on fixtures | `incremental_equals_clean_*`; changed-source invalidation tests |
| 6 — compiler and rules internals | P3 configuration lookups → indexed structures; `PhysicalInventory` once per compile; `require`/`execute_empty` → existence; output builders concat once; rules head side delta-only; invariant program compiled once | DM-38, DM-36, DM-33 | Query count per P3; rows scanned per round ∝ delta; invariant compile count | Conservation and template fixtures unchanged |
| 7 — storage and accounting | Snapshot cache by (store, version); concurrent member opens; no post-commit read-back except ambiguity; result-granular accounting, delete the global inventory | DM-26, DM-32, DM-37, DM-30 | Request counts per publish; accounting conservative on the heater compile | `publication_each_object`, concurrent-writer and uncertain-commit tests unchanged |
| 8 — close the record | Blueprint/ADR amendments from §10; relabel §18.2 and §5.4 evidence; rewrite R-22/R-01; second full measured campaign | DM-59, DM-60 | Baseline vs final table; zero failures against zero baseline with commands/modes named | The §9 gate stays in CI |

**Final check:** the design's claims match the evidence only after step 1; the scope matches
the implemented guarantees once §10's relabelings land; later extensions (a new stage, a new
invariant, a new relation) then add one declaration and one implementation, and pay
validation once.
