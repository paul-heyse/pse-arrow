---
title: Library-owned mathematics — Symbolica as the math IR and compiler, direct Ipopt/POUNCE solving, faer structure, no Pyomo
date: 2026-09-23
status: proposed
scope: >-
  The math pipeline as implemented at a46f358 plus the working tree (pse-mathir, pse-numerics,
  pse-backend-native, pse-ipopt-sys, pse-quantity, pse-structural, the math parts of
  pse-compiler, pse-runtime, pse-templates, pse-authoring and pse-codegen, the math relation
  families and their generated code, and the NL/Pyomo/kernel stubs), reviewed against the
  maintainer's library-first directive and docs/capability-maps/math_libraries.md
depth: deep
evidence: >-
  Implemented/Interface-checked for the repository (file:line below); Measured for the current
  evaluator and the Symbolica comparison (conditions in §9.1); Tested/Measured for oximo, POUNCE
  and the ipopt crate (scratch probes, §9.2); Documented (public docs) plus top-level Tested for
  Symbolica; Proposed for the target
decision: >-
  Revise the current math pipeline; Accept the library-owned target in §4 as the design direction
  (document stage, Proposed), with the maintainer's decisions U1–U5 of 2026-09-23 recorded in §10.1
---

# Design review — library-owned mathematics

Standard: [Data Model–Based Design Charter](../design_principles/DATA_MODEL_DESIGN_CHARTER.md)
(**DM-n**, **G1–G7**) and the
[Rust computation architecture guidelines](../design_principles/rust_computation_architecture_guidelines_graph_extended.md)
(**RCA §n**). Brief: [math libraries capability map](../../capability-maps/math_libraries.md).
Date: 2026-09-23.

**Governing objective (maintainer direction, recorded verbatim in intent).** Minimize, to the
greatest extent possible, the mathematics this repository implements itself. Dedicated libraries
own generic mathematics: expression representation, simplification, differentiation, evaluation,
code generation, sparse structure, linear algebra and solver interfaces. Pivot away from
implementing mathematics through Arrow, DataFusion, Delta and bespoke machinery. Pivot away from
Pyomo as a basis of the design; solve Rust-natively. The project is for personal use under the
maintainer's Symbolica hobbyist key; licensing and distribution do not constrain the design.
Where the blueprint, an ADR or RCA blocks this target, the blocking text is recorded as a
required change (§10.2), not treated as a constraint. Benchmarks inform the design; they do not
decide it. The main measures are how much generic math disappears and how local an ordinary
extension becomes (DM-56, DM-57, charter §E).

## 1. Decision and scope

**Decision: Revise the current math pipeline. Accept the target in §4 as the design direction —
Symbolica as the single mathematical IR from the typed template definition onward, evaluated per
specialization and bound per instance, solved through one adapter over Ipopt C and POUNCE, with
faer-owned sparse structure.** Four gates fail for the current code (G1, G2, G5, G7) and one is
unresolved (G6). On 2026-09-23 the maintainer decided the five questions the target depended on
(§10.1): real-algebra equivalence with explicit domain checks replaces the ordered, guarded
floating-point contract (F4); Symbolica starts at the typed template definition; one evaluator per
template specialization; Ipopt C and POUNCE behind one adapter; and personal use under the
maintainer's multi-core hobbyist Symbolica key as the operating assumption, so no licensing or
distribution mechanism is part of the design. With those
decisions every target gate passes at the document stage (§6); the claims stay Proposed until the
§9.3 qualification runs.

What the review establishes, in order of consequence:

1. **The pipeline cannot solve the models it claims to support (F1, G7).** Case preparation refuses
   every equation with a free index (`crates/pse-compiler/src/case/prepare.rs:156-159`: "case
   preparation requires expanded scalar equation indices") and no index-expansion pass (P12) or
   discretization pass (P11) exists. Kernel calls are validated but never lowered
   (`crates/pse-mathir/src/numerical/lower.rs:222-225`). Every inferred kernel conversion
   (molar↔mass) is refused, because the production symbol source returns `None` for every
   conversion binding (`crates/pse-mathir/src/infer/source.rs:54-61`, used by
   `passes/p10/production.rs:106`). There is no exact Hessian
   (`crates/pse-backend-native/src/driver/callbacks.rs:189-212`). The NL and Pyomo backends are
   9-line stubs. The solve operator is reached only from tests. A species balance written as
   `sum(c in components | …)` cannot reach a solver today.
2. **Most of the math code is representation plumbing and duplicated generic machinery, not
   mathematics (F2, F3, F5).** Mathematics crosses between passes as Arrow relations (3 full-family
   encodes and at least 4 decodes per compile), backed by about 41k generated lines in
   `pse-relations` math families, 9.7k lines of generated sink/source adapters and their
   generators. Two independent symbolic differentiators exist and disagree. A 1.8k-line
   DataFusion-shaped batch evaluator has no production consumer and compiles in quadratic time
   (25 s at 16,000 equations, §9.1).
3. **Symbolica can own the generic mathematics; the blocker is a contract, not a capability
   (F4).** Its documented normalization (sorting, merging like terms, cancelling) and its evaluators'
   documented "fast math" contradict blueprint §7.3/§7.4 and accepted ADR-0047, which require
   ordered, guarded floating-point semantics in the model graph. The one reason for that contract
   that survives scrutiny — never hide a domain failure — is met more directly by extracting
   domain obligations from the typed authored definition before any normalization and evaluating
   them beside the residuals. With that change, the ordered hash-consed `ExprGraph`, its canonical
   hashing, guarded folding, both differentiators, the numerical topology and the scalar
   interpreter all become deletable.
4. **The brief's route through oximo would discard Symbolica's derivatives (F9).** oximo 0.7 has no
   public way to accept external derivatives; on stable Rust its NLP path uses forward finite
   differences, a dense Jacobian and L-BFGS — measured 193× slower than exact sparse derivatives at
   n = 800 (§9.2). NLPs must go directly from the compiled problem to a TNLP-shaped adapter: the
   existing Ipopt C driver (gaining an exact Hessian) and POUNCE (pure-Rust Ipopt port, Tested with
   exact sparse Jacobians and Hessians), behind one adapter (U4). oximo keeps LP/MILP/QP/SOCP routing and ASCII
   NL/LP/MPS export.
5. **Compile per specialization, bind per instance (F7).** One Symbolica evaluator per template
   specialization body over formal slots, called per instance through binding tables, replaces the
   missing P12, keeps compile cost proportional to distinct templates rather than instances,
   bounds Symbolica's process-global symbol table, and allows cross-instance SIMD (documented
   `f64x4` JIT; not measured here). Measured on the
   same synthetic flowsheet (§9.1): building the one unit body — parse, derivatives, optimized
   evaluator and JIT — takes 1.9 ms regardless of system size, and a full residual-plus-Jacobian
   evaluation of 16,000 equations takes 115 µs (JIT) against 2,940 µs for the current production
   interpreter. A monolithic optimized Symbolica evaluator for 8,000 equations takes 4.9 s to build
   and JIT, and evaluates more slowly (108 µs) than the per-specialization JIT (68 µs).
6. **Pyomo leaves the production design (F10).** It is required by D12, §21, ADR-0075 and the
   phase exits, yet absent from the code. IDAES parity stays a test-only oracle in `.venv-parity`.

**Proposal.** Replace the bespoke math pipeline with library-owned stages (§4.1): Symbolica
(expressions, derivatives, evaluators, JIT), faer (sparse structure, Newton and diagnostic linear
algebra), petgraph / rust-igraph / rustworkx-core (structural analysis, already adopted), Ipopt C and
POUNCE (NLP), oximo with HiGHS/Clarabel (LP/MILP/QP/SOCP, model export), num-dual (opaque kernel
derivatives). Keep bespoke only what no library owns: physical typing, domain obligations,
specialization and instance binding, and the mapping of domain identities to library indices.

**Status.** The current pipeline is Implemented (with the unit-level evidence Plan 13 records) and
partly Tested; W19 functional acceptance failed and W20 has not run
([W19 checkpoint](../../plans/13-w19-repair-checkpoint.md)). The target is Proposed. Library
capabilities are Interface-checked (oximo, POUNCE, faer, `ipopt` crate: source read), Tested or
Measured where §9 names a probe, and Documented (public docs) for Symbolica, with its top-level
behaviour Tested through the public API only (see Method).

**Reviewer / author.** Design-review agent for the maintainer. **Affected revisions.** `main` at
`a46f358` plus the uncommitted working tree of 2026-09-23 (including the uncommitted
`pyproject.toml` dependency change).

**Observable outcome.** Every blueprint-scoped model that is mathematically well posed — including
indexed balances, kernel calls, unit conversions and exact-Hessian NLPs — compiles to one library-owned
symbolic form, is differentiated and evaluated by library code, and is solved in process without
Pyomo. Adding a function, law or kernel is one declaration plus one registration, not coordinated
edits across eight handwritten files and four generated trees (§5.1).

**Baseline.** Plan 13 moved P7–P10 into an in-memory `ExprGraph` and removed the scalar
DataFusion-`Expr` callback path (ADR-0078, proposed). The math still enters and leaves passes as
Arrow relations, numerical lowering is bespoke, solving is limited-memory Ipopt through a DataFusion
physical operator, and before this review nothing had measured compile, evaluation or solve time
for the current pipeline (Plan 13 W20 and Plan 11 I19 unrun). Historical measurements of the earlier,
DataFusion-centred compiler record heater/mixer publications of 398–448 s with 15.4–19.6 GB peak
compile reservations, and all five engineering workflows terminated at 900 s on 2026-09-23
([Plan 09 measurements](../../plans/09-cache-measurements.md);
[alignment review](design_review_rust-computation-architecture-alignment_2026-09-23.md) M9).

**Supported scope and non-goals.** In scope: the mathematical path from parsed template equations
to solver results, its generated relation families, the solver adapters and the Pyomo/NL/kernel
stubs; the units crate as it bears on math; libraries that would remove bespoke generic code
elsewhere (§8.3). Not in scope: the relational rule engine (`pse-rules`, which does not touch
mathematics), catalog/Delta publication mechanics beyond what math publishes, the Python package
beyond the Pyomo adapter, and IDAES parity content. Not a goal: a full CAS of our own, a generic
optimization-modelling language, or keeping any representation merely because it exists.

**Constraints and uncertainty.**
- *Operating assumption: personal use with a hobbyist key.* The design assumes the maintainer's
  multi-core Symbolica hobbyist key and personal use. Licensing and distribution are not design
  constraints (consistent with ADR-0066); if the project is ever distributed, the repository's
  existing deferral (register R-31) is where that question is reopened.
- *Symbolica evidence is public-API evidence.* No Symbolica claim below rests on its source
  code. Claims are **Documented** (docs.rs 3.0.0 rendered API, symbolica.io via Context7) or
  **Tested/Measured** through the documented public API.
- *No end-to-end compile or solve was run.* The W19 campaign environment was not rebuilt. Evaluation
  and construction costs were measured in isolation (§9.1); end-to-end costs remain hypotheses
  (DM-39).

### 1.1 Method and coverage

**Read (Interface-checked, by me unless noted).** Blueprint §1, D1–D14, §7 (all), §18 (all);
`crates/pse-mathir` (`lib.rs`, `node.rs`, `numerical.rs`, `numerical/derivative.rs`,
`numerical/lower.rs`, `infer.rs`, `infer/source.rs`, `contracts/operators.rs`),
`crates/pse-numerics` (`scalar.rs`, `expressions.rs`, `expressions/topology.rs`, `differentiate.rs`,
`stages.rs`, `bindings.rs`), `crates/pse-backend-native` (`driver.rs`, `driver/callbacks.rs`,
`driver/workspace.rs`, `problem.rs`, `native/output.rs`), `crates/pse-compiler/src/case/prepare.rs`,
`crates/pse-runtime/src/compiler_driver/session/case.rs`, `passes/p10/production.rs`,
`passes/p4/predicates.rs`, `passes/p9/parameters.rs`, `passes/parameter_indices.rs`,
`pyproject.toml`, `Cargo.toml`, ADR front matter for the records in §10.2, Symbolica `License.md`.

**Delegated breadth, verified at the cited grain.** Ten evidence agents inventoried the crates and
libraries. Their reports are leads; every `file:line` in §7 was read by me. Claims I did not
re-read are marked "(agent report)" and carry no finding on their own.

**Run.**
- `compare-probe` (scratch crate, path dependencies on `pse-numerics`, `pse-mathir`): the
  production lowering route (`NumericalBuilder` → `finish` → `EvaluationProgram::from_topology` →
  `ScalarWorkspace`) and the test/bench-only batch route (`EvaluationProgram::compile`) on a
  synthetic flowsheet of 10–2,000 units. Measured, §9.1.
- `compare-symbolica` (scratch crate, `symbolica =3.0.0`, default features, public API only,
  run with the maintainer's hobbyist key): the same
  expressions, monolithic and per-specialization evaluators, interpreted and JIT; and two
  observations of documented normalization behaviour. Measured and Tested, §9.1.
- Agent probes: oximo/oximo-io/oximo-pounce, POUNCE TNLP, the `ipopt` 0.6 crate, ASL acceptance of
  oximo NL files in the project solver image (§9.2); faer 0.24.4 sparse LU with pattern reuse,
  singular and KKT cases, thread determinism, and matching/Dulmage–Mendelsohn/BTF crates (§9.2);
  a second Symbolica probe with a different workload, unlicensed (§9.1); uom, rink-core, num-dual,
  symjit (agent report).
- `just unit-package pse-numerics 'package(pse-numerics)'` (debug, force-validate): 19 passed,
  0 failed, baseline zero (agent run; not re-run by me).

**Not inspected, or looked at and not settled.**
- Salsa reuse correctness of the math queries (`Canonicalize`, `PrepareCase`, numerical topology) was
  not attacked. The reuse claims there are asserted, not verified (G6 unresolved).
- P3–P9 instantiation logic was read only where it transports or computes mathematics.
- End-to-end compile and solve times on the current pipeline were not measured.
- Symbolica internals: evaluated only through its public API and documentation.
- **Licence conduct note.** Before the licence terms were noticed, three agents and I read parts of
  the Symbolica source. Every statement derived from that reading has been withdrawn from this
  review and from the agents' reports. The observable behaviours this review relies on were
  re-established through the public API (§9.1).

## 2. Authority and lifecycle map

The mathematics has one authored fact — the equation text — and many derived forms. Today several
derived forms are independently constructed, and two carry independent rule definitions.

| Concept or fact | Current representation and owner | Target representation and owner | Revision boundary | Update path |
|---|---|---|---|---|
| Authored equation text | `pse.expr_dsl` columns in `authored` relations (Delta release) | unchanged | model release | change set only (§7.7) |
| Parsed expression | P3 `normalized.*_expr_*` relations, encoded then decoded by P4, P6, P7, P9 (agent report, sites verified for P4 and case preparation) | typed authored AST in memory, per template; never persisted | derived per release | derived only |
| Physical typing | P10 over `ExprGraph` per region (`canonicalize.rs`, `pse-quantity` rules) | the same `pse-quantity` rules over the typed AST, once per specialization | specialization | derived only |
| Canonical mathematics | `compiled.math_*` relations + `CanonicalGraph` + ordered `pse-ids` subtree hashes | Symbolica atoms per specialization body; not an identity source | specialization | derived only |
| Math identity (reuse, publication) | ordered hash-consing of the graph (§7.4 step 1) | `pse-ids` framed hash of the typed definition + bindings + library versions + numerical policy | specialization | derived only |
| Derivative rules | **two engines**: `pse-mathir/src/numerical/derivative.rs` and `pse-numerics/src/differentiate.rs` | Symbolica `derivative` for built-ins; one registered derivative per non-built-in function; kernel bindings (num-dual, implicit) | library version + registration table | registration table only |
| Domain admissibility | implicit in ordered guarded evaluation (ADR-0047) | explicit obligation list per body, extracted before normalization | specialization | derived only |
| Numerical program | `NumericalTopology` + `scalar::Program` with DataFusion `Operator` opcodes; a separate batch `Expr` program | Symbolica `ExpressionEvaluator` (optionally JIT) per specialization | specialization + policy | derived only |
| Instance binding | none (no P12) | binding table: instance × formal slot → global variable / parameter ordinal | structure revision | derived relationally from model facts |
| Sparsity | held three times (topology, `jacobian_coordinates` rows, workspace COO) | one global pattern (faer `SymbolicSparseColMat` + value scatter maps), a COO view for Ipopt | structure revision | derived only |
| Structural analysis | computed per case (`session/case.rs:125-127`), read only by a test | computed from the derivative pattern; published as `compiled.incidence`, `blocks`, `dm_partition` | structure revision | derived only |
| Solver options | hard-coded in `driver.rs:224-244` | `solver_profiles` (blueprint §18.3) | case | profile edit |
| Operator capability | `lowering` lists in `contracts/operators.rs:156-172` ("potential", unenforced) and blueprint §7.2/§18.9 tables | generated from function registrations and adapter support, enforced at admission | library version | registration only |
| Results | one nested `runtime.solver_outcomes` row; `constraints` = Ipopt's `g` (`native/output.rs:64`) | `runs`, `solutions`, `duals`, natively recomputed `residuals`, `iterations` (§18.6) | run | append only |

**Deliberately opaque behaviour.** Two things stay outside the inspectable model, each behind a
contract. Kernel bodies (EOS roots, property functions) are Rust code with declared derivative
bindings (D9, ADR-0043). Symbolica's internals are a third-party engine; its contract is its
documented public API plus the qualification suite in §9.3.

**Identity behaviour.** Domain semantic IDs stay the only durable identities (D4). Symbolica
symbols are formal-slot names (`pse::s0 … pse::sK`) registered once at process start, plus
kernel-function symbols named with their kernel version. Neither is an identity. Atom bytes and
printed forms are never hashed for reuse (F11). Renaming a unit or reordering instances changes
binding tables, not specialization bodies.

## 3. Semantic contracts and invariants

| Contract or invariant | Representation (target) | Enforcement boundary | Failure behaviour | Verification |
|---|---|---|---|---|
| Physical typing is complete and runs on authored structure | typed AST; `pse-quantity` rules keyed on authored `Sub`/`Div`/`Neg` | specialization typing, before lowering to Symbolica | typed diagnostic with source span | existing `pse-quantity` inference tests re-hosted; point−point difference cases |
| Domain obligations | per body: `(guard, condition, kind, source span)` for every divisor ≠ 0, `log`/`log10` argument > 0, `sqrt` argument ≥ 0, fractional-power base > 0, kernel validity | extracted from the typed AST before normalization; evaluated as extra evaluator outputs at every callback | `evaluation.domain_violation` naming equation, obligation and span; the solver receives a recoverable evaluation error | negative tests where normalization would erase the failure (`x/x`, `exp(log x)`, `log(0)` inside and outside `if`) |
| Declared equivalence | real-arithmetic equivalence on the admissible domain, under Symbolica normalization vN and evaluator optimizations; tolerance reproducibility, not bitwise | numerical policy record entering compiled-artifact identity | a failed conformance comparison is a qualification failure | differential test against a test-only authored-order evaluator of the typed AST |
| Derivative contract | Symbolica built-ins; one registered derivative per non-built-in function; kernel `derivative_bindings` | function registration table, generated from the operator table | unavailable derivative → admission refusal, never a manufactured value | finite-difference checks per function; Ipopt `derivative_test=second-order` on fixtures |
| Nonsmooth policy | `abs`, `if` with variable guard, `min`/`max` require a declared policy | lowering | refusal without policy | existing refusal tests carried forward |
| Capability truthfulness | the accepted operator surface equals the registered surface | admission, before case preparation | `capability.unsupported_operator` with the opcode list | generated conformance test: every accepted opcode has a registration and an evaluator test |
| Determinism | slot symbols registered in fixed order at start; kernel symbols registered sorted | process start | none (construction) | two-process test: identical canonical strings and residuals within tolerance |
| Linear-solve failure classification | structural rank from the matching; finiteness and residual after each faer solve | before and after every Newton/block solve | typed `solve.singular_structure` or `solve.numerical_singularity`, attributed to the block | singular and near-singular fixtures on both faer paths (F15) |
| Solver status fidelity | Ipopt/POUNCE status mapped without collapsing acceptable, restoration failure or numeric errors | solver adapter | typed termination; recovered evaluation errors recorded as events, not terminal failure | fixture with a trial point outside `log`'s domain that still converges |

**Absence and uncertainty.** The target distinguishes: not yet evaluated (no value for this point);
domain violation (obligation failed; typed, attributed); non-finite result without a violated
obligation (typed `evaluation.nonfinite`, attributed to the equation); unavailable derivative
(admission refusal); unsupported operator (admission refusal); cancelled; and solver termination
classes. Today a recovered evaluation error and a terminal one collapse into
`EvaluationFailure` (F6).

**Equivalence requirements.** Byte equality: none claimed across processes for evaluator output.
Structural equality: canonical strings of specialization bodies within one Symbolica version.
Semantic equality: real-algebra equivalence on the admissible domain. Approximate agreement: the
§7.3 tolerance rule `abs(a−b) ≤ atol + rtol·max(|a|,|b|)` for residuals and derivatives.

## 4. Derivation and execution design

### 4.1 Target shape

```text
authored text + model facts (Arrow/Delta release)          ← authority, unchanged
   │ parse (pse-authoring, existing)
   ▼
typed authored AST per template          ← bespoke, domain: physical typing, obligations, spans
   │ specialize (bind finite sets, structural options, kernel selections)      [Salsa]
   ▼
SymbolicBody per specialization          ← Symbolica atoms over formal slots
   │ derivative / classify / evaluator build (+ JIT)                          [Salsa]
   ▼
CompiledBody per specialization          ← Symbolica ExpressionEvaluator, local sparsity,
   │                                        Hessian-of-Lagrangian evaluator, obligations
   │ bind instances (binding tables from model facts; DataFusion only where set-oriented)
   ▼
CompiledProblem                          ← binding tables + global pattern (faer) +
   │                                        structural analysis (petgraph / rust-igraph)
   ├──► NLP adapter (TNLP shape) ──► Ipopt C (pse-ipopt-sys, parity build) │ POUNCE (pure Rust)
   ├──► Newton / block initialization, diagnostics ──► faer sparse LU / SVD
   ├──► LP/MILP/QP/SOCP ──► oximo ──► HiGHS / Clarabel
   └──► export ──► oximo-io ASCII NL / LP / MPS (kernel-free models)
   ▼
results + compiled bundle tables ──► Arrow / Delta publication
```

### 4.2 Stage table

| Stage | Inputs and dependencies | Output contract | Effects and ownership | Invalidation |
|---|---|---|---|---|
| S1 Parse | authored text | untyped AST with spans | pure | text edit |
| S2 Specialization typing | AST, template bindings, quantity registry release | typed AST, obligations, specialization identity | pure; Salsa tracked | template, binding or registry change |
| S3 Symbolic lowering | typed AST, function registrations | `SymbolicBody` (atoms over slots), built through the atom API — never by printing and re-parsing text, because Symbolica's parser reads `x < y` as a product | pure; Symbolica process state is append-only symbol and function registration at start | S2 identity + Symbolica version |
| S4 Derivation and evaluator build | `SymbolicBody`, numerical policy | `CompiledBody`: evaluators (interpreted and optional JIT), local Jacobian and Hessian patterns, equation classes | pure; artifact owned by the compile session | S3 + policy + JIT setting |
| S5 Instance binding | release facts: instances, ports, domains | binding tables (instance × slot → global ordinal) | relational where set-oriented; typed Rust otherwise | structure revision |
| S6 Assembly and structure | binding tables, local patterns | global faer pattern + scatter maps; incidence, matching, DM, block order | pure | structure revision |
| S7 Execution | `CompiledProblem`, case values, profile | solver attempt state | effectful, cancellable, outside Salsa (RCA §7) | not reused |
| S8 Publication | results, compiled bundle tables | Delta members | effectful, explicit commit | run |

### 4.3 Computation contracts (RCA §9, compressed to the stages that carry findings)

| Item | S2–S4 (per specialization) | S5–S6 (per structure revision) | S7 (attempt) |
|---|---|---|---|
| Semantic output and equality | canonical strings of bodies and derivative atoms; identity from S2's framed hash | binding tables sorted by semantic ID; pattern equality by index sets | results with typed termination |
| Backend and representation | typed AST (Rust enums), Symbolica atoms, `ExpressionEvaluator`/JIT | Arrow tables for binding facts; faer `SymbolicSparseColMat`; petgraph `Graph` for incidence | Ipopt C or POUNCE; faer for Newton |
| Tracked boundary | Salsa query per specialization (RCA §4 DEFAULT) | Salsa query per structure revision | none |
| Observed dependencies | template definition, bindings, registry release, function registration table, Symbolica version, policy | instance membership including additions and removals, port bindings, domain members | case values, profile, warm start |
| Structural vs runtime inputs | a parameter used in a branch condition or exponent that normalization consumes is structural; other parameters remain symbolic slots | membership is structural | values are runtime |
| Cycles and termination | acyclic by construction (ASTs) | SCC on the matching-oriented graph (existing `pse-structural`) | solver iteration limits and cancellation |
| Exactness and reproducibility | exact symbolic derivatives; evaluator results tolerance-reproducible | exact | solver-dependent; recorded |
| Cardinality and cost | bodies ∝ distinct specializations, not instances; symbol table ∝ max slots per body | ∝ instances × slots | per callback ∝ instances × body size |
| Ownership, retention, publication | compile session owns evaluators; published bodies as canonical strings | tables published with the compiled bundle | attempt-owned; results published |
| Departures | none from RCA defaults | none | none |

### 4.4 Why per specialization, not one monolithic evaluator

- **Scale.** Compile work and memory grow with distinct template specializations (tens to hundreds
  in a flowsheet), not with instances or discretization cells (thousands to millions). A
  monolithic evaluator over every scalar equation must re-optimize the whole system after any
  structural edit. Measured (§9.1): the optimized monolithic evaluator took 7.8 ms, 296 ms and
  3,434 ms to build for 80, 800 and 8,000 equations (plus 1.4 s of JIT at 8,000); the
  per-specialization body took 1.9 ms in total, once.
- **Speed.** Per-specialization evaluation was faster than monolithic at 8,000 equations (68 µs vs
  108 µs, both JIT) despite the gather/scatter loop, and agreed with the reference to 8e-16
  against 5e-10 for the monolithic evaluator.
- **Symbol-table bound.** Symbolica's symbol registry is process-global (documented global state)
  and append-only. Per-instance scalar symbols would grow it without bound in a long-lived Python
  session. Formal slots bound it to the widest body.
- **Determinism.** §9.1 shows printed term order depends on which symbol a process created first,
  while `to_canonical_string()` does not. Registering the slot symbols once, in a fixed order, at
  start removes that dependence for bodies (Proposed; the §9.3 determinism test checks it).
- **Incrementality.** A value edit touches no body. Adding a unit adds binding rows. Editing a
  template recompiles one body. This is RCA §4's "interface separate from implementation,
  compiled structure from runtime values", and it replaces the missing P12 with binding tables.
- **Vectorization.** Instances of one specialization evaluate together; Symbolica's JIT documents
  `wide::f64x4` evaluators (Documented, not measured here).
- A monolithic evaluator remains a valid option for small systems when its measured cost is lower.

### 4.5 Relationship structures

Four graphs, kept distinct (DM-34): flowsheet connectivity (ports and arcs, model facts); the
equation–variable incidence bipartite graph (from local Jacobian patterns through binding tables,
not from free symbols, so guard-only and zero-derivative arguments do not create edges); the
matching-oriented dependency graph used for Dulmage–Mendelsohn partitions and block order (existing
`pse-structural`); and the tear/recycle graph over units. Communities or centrality are not used
for any of them.

### 4.6 Provider selection and limitations

| Provider | Selected for | Declared limits |
|---|---|---|
| Symbolica 3.0.0 | expressions, simplification, derivatives, evaluators, JIT, printing | normalization is real-algebra; no indexed-set semantics; no comparisons (`x < y` parses silently as a product), no built-in `tan`, `min`, `max`, `sign`, `step`, no derivative for `abs` — each needs a registration; process-global, append-only symbol and function registration (~0.9 kB per symbol); default features install a global allocator and cap tracing (F14); evaluation needs `&mut self`, so each thread uses a clone; `serde_json` round trip fails, `bincode` and canonical strings work |
| Ipopt 3.14.20 C (`pse-ipopt-sys` + existing driver) | NLP in process; parity-comparable iterates | C toolchain and container (ADR-0028) |
| POUNCE 0.12 (pounce-rs) | NLP in pure Rust (no C/Fortran toolchain or solver container) | pre-1.0; `!Send` application; FERAL linear solver, no MUMPS; vendor reports Ipopt faster on its large-scale suite |
| faer 0.24.4 | sparse pattern, Newton and block LU, diagnostics | numerically singular systems return `Ok` with NaN/inf; structural-singularity errors depend on the simplicial/supernodal path; applies its own column ordering; not a KKT solver (no inertia; Bunch–Kaufman falls back to unpivoted LDLᵀ on the simplicial path and returned NaN without error on a 30k KKT test); supernodal LU bits vary with thread count; value refill needs our own slot map (§9.2) |
| pounce-presolve 0.12 | maximum matching, Dulmage–Mendelsohn partition, block-triangular order | drops out-of-range incidence entries silently, so admission must check indices (§9.2) |
| petgraph 0.8.3, rust-igraph 0.7.0, rustworkx-core 0.18.1 | incidence, matching, DM, SCC, block order | unchanged from Plan 13 W10 |
| oximo 0.7.0 (+HiGHS, Clarabel) | LP/MILP/QP/SOCP routing; ASCII NL/LP/MPS export | no external derivatives; no conditional or external-function nodes; binary NL rejected by ASL; runtime construction only through `#[doc(hidden)]` builders; no cancellation hook |
| num-dual 0.15 | opaque kernel derivatives (ADR-0022) | only where a kernel exists |

### 4.7 Boundary contracts

Arrow and Delta remain the boundary: model facts enter as an exact release; binding tables may be
computed relationally; results and the compiled bundle leave as typed relations. Inside S2–S7 no
mathematics is encoded to Arrow. The compiled bundle publishes typed tables (variables, equations,
equation classes, incidence, blocks, DM partition, sparsity, scaling, source map) and each
specialization body as a canonical string for inspection. Expression-node relations are not
published.

**Coherent publication.** Unchanged: selected complete products at the §20 commit boundary.

## 5. Representative journeys

### 5.1 Ordinary extension: add one mathematical function

Measured by `grep` on `Tanh`, an existing unary function (§9.1): today it appears in 8 handwritten
production files — the opcode enum (`pse-quantity/src/enums.rs`), the operator table and
bindings (`pse-mathir/src/contracts/operators.rs`, `contracts/bindings.rs`), the production
derivative rule (`numerical/derivative.rs`), the batch derivative rule
(`pse-numerics/src/differentiate.rs`), the DataFusion UDF and scalar table
(`pse-numerics/src/bindings.rs`), the DSL function table (`pse-authoring/src/dsl/ast.rs`) and DSL
lowering (`pse-compiler/src/lowering/lower.rs`) — plus tests and four generated trees
(`pse-model` and `pse-relations` enums/contracts, the `pse-quantity` fixture, Python enums). Two of
those sites define the same derivative independently (F2).

Target: the DSL function name, one Symbolica registration (evaluation and derivative, or nothing
for a Symbolica built-in) and, if the function has a new quantity rule, one `pse-quantity` rule.
The opcode enum, derivative engines, numeric bindings and generated relation families disappear.
New semantic decisions: one. Coordinated edits: three at most.

A new **law** is a template (unchanged); a new **kernel** is a Rust body over num-dual plus one
registration that tells Symbolica how to evaluate and differentiate it. Neither touches numerics.

### 5.2 Meaningful change

- **Parameter value edit** (e.g. a heat duty). Today: a value change re-enters case preparation;
  the numerical topology is keyed on the whole request (agent report). Target: the value is a slot
  binding; no body, pattern or binding table is recomputed; the next attempt reads the new value.
- **Structural edit** (add a heater to a train). Target: one specialization already compiled; new
  binding rows; pattern and structural analysis recomputed for the revision (RCA §4, §7).
- **Template edit.** Target: one body recompiled; everything keyed on its identity invalidated.
- **Symbolica version change.** Target: every body identity changes (version is in the identity);
  bodies recompile; the qualification suite (§9.3) gates the pin move.

### 5.3 Boundary or alternate representation

- **Inspection.** A user asks for the residual of equation *e*: the bundle names its specialization
  and instance; the body's canonical string and the binding row reproduce it. Authored-order text
  remains available from the source span.
- **Export.** ASCII NL through oximo-io for kernel-free, conditional-free models, with `.row`/`.col`
  labels (Tested with the project's Ipopt 3.14.20/ASL, §9.2). Conditionals, opaque kernels and
  shared subexpressions are not carried; the exporter refuses them rather than degrading (DM-42).
- **IDAES parity.** Runs in `.venv-parity` against IDAES models; compares solutions and residuals by
  semantic-ID/IDAES-name maps. No production Pyomo.

### 5.4 Interruption or failure

- **Trial point outside a domain.** Ipopt's line search probes `log(x)` at `x < 0`. Today the
  callback stores the failure permanently (`driver/callbacks.rs:21-27`) and `pack` checks that flag
  before Ipopt's status (`native/output.rs:18-26`), so a converged solve is published as
  `EvaluationFailure` (F6). Target: the obligation fails, the callback returns false for that trial
  point, the adapter records a recoverable event, and the final termination comes from the solver.
- **Cancellation.** Unchanged mechanism: the intermediate callback checks the token (both Ipopt and
  POUNCE support stop-by-callback, Tested for POUNCE).
- **JIT failure.** A JIT compile error falls back to the interpreted evaluator only as a declared
  policy, recorded in the run.

## 6. Acceptance gates

| Gate | Current code | Evidence | Target (Proposed) | Required action |
|---|---|---|---|---|
| G1 Authority | **Fail** | two derivative engines with different rules for the same operators (F2); operator capability declared in the operator table and blueprint tables but not derived from lowering (F1) | **Pass (Proposed)** — one math IR: `ExprGraph`, `NumericalTopology`, both derivative engines and the relational transport are deleted, not kept beside Symbolica (U2); the capability matrix is generated from registrations | delete, don't wrap |
| G2 Semantic fidelity | **Fail** | termination meaning lost: a recovered evaluation error publishes a converged solve as `EvaluationFailure` (F6); published `constraints` are Ipopt's `g`, not residuals (§18.6) | **Pass (Proposed)** — the obligation contract (U1) makes every authored domain failure explicit and attributed; residuals recomputed natively | F4, F6 |
| G3 Validity | **Pass** | invalid inputs reach typed refusals: indexed equations, kernel calls, unsupported opcodes and kernel conversions are refused explicitly (`prepare.rs:156-159`, `lower.rs:222-225`, `infer/source.rs:54-61`) | **Pass (Proposed)** — admission-time refusal before case preparation | F1 |
| G4 Hidden behaviour | **Pass** | solve operator declares `{Read, Nondeterministic}` effects (agent report); no ambient reads found in the math path | **Pass (Proposed)** — symbol and function registration fixed at start, never inside tracked queries; default features disabled so no global allocator, tracing cap or stdout subscriber (F14) | F11, F14 |
| G5 Consistency and recovery | **Fail** | recovery from a failed trial point is not represented; the attempt's recorded failure outlives the recovery (F6) | **Pass (Proposed)** with F6's correction | F6 |
| G6 Transformation and reuse | **Unresolved** | Salsa reuse of math queries not attacked; the batch and production paths differ (F2) | **Pass (Proposed)** — the declared equivalence (U1: real algebra on the admissible domain, tolerance reproducibility) covers normalization and evaluator optimizations; reuse keyed on typed-definition identity plus library version and policy; verification in §9.3 | F4, F7; §9.3 differential, determinism and reuse-vs-clean tests |
| G7 Truthful capability | **Fail** | NL, Pyomo, exact Hessian, indexed models, kernel execution, kernel conversions and structural publication claimed by blueprint D12, §18.4, §18.9, §7.1, AGENTS.md and the operator table; absent or refused in code (F1) | **Pass (Proposed)** — generated capability matrix; Pyomo and NL claims withdrawn; oximo export limited to what it carries | F1, F10 |

## 7. Principle findings

Ranked by the review's severity order: correctness and authority, then semantic duplication and
extension difficulty, then measured cost.

| # | Finding | Principle IDs | Evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| F1 | **The pipeline cannot solve the model classes it claims.** Indexed equations, kernel calls and inferred kernel conversions are refused before any solver; there is no P11/P12, no exact Hessian, no NL or Pyomo backend, and the structural result is never published. | G7 · DM-43, DM-59, DM-04 | `pse-compiler/src/case/prepare.rs:156-159` refuses free indices; `pse-mathir/src/numerical/lower.rs:222-225` refuses KernelCall, Erf, SafeSqrt, SafeLog, PiecewiseLinear; `infer/source.rs:54-61` returns `None` for every conversion binding and production P10 uses it (`p10/production.rs:106`); `callbacks.rs:189-212` refuses `eval_h`; `pse-backend-nl`, `pse-backend-pyomo` are 9-line stubs; `session/case.rs:125-127` computes the structural result, only `tests/compiler_edit_solve/source_case.rs:137-145` reads it; `Solve::plan` has no non-test caller | A heater whose energy balance sums over phases, a flash with a cubic-EOS kernel, or any mass↔mole conversion cannot be solved. The operator table publishes `Native/Nl/Pyomo` routes for opcodes nothing lowers, so a reader of `reference.operator_specs` is misinformed. | The target stages S2–S6 (§4). Until then, narrow the claims: generate the capability matrix from what lowers, refuse at admission. | Source-to-solve tests for an indexed heater, a kernel-bearing flash, and a unit conversion; a generated test that every advertised route has an implementation |
| F2 | **Derivative rules are defined twice and disagree.** | G1 · DM-02, DM-25 | production `pse-mathir/src/numerical/derivative.rs` over slots (called from `numerical.rs:292`); batch `pse-numerics/src/differentiate.rs` over DataFusion `Expr` (called only from `expressions.rs:135`). `Abs` at 0: typed `UndefinedDerivative` vs a NaN literal; `x^p` with parameter `p = 0` at `x = 0`: 0 vs a non-finite failure (agent report, rules read) | The scalar-vs-batch agreement test builds both sides from the `Expr` engine, so the production engine is never cross-checked. Any change to one rule leaves the other stale. | Delete both; Symbolica derivatives plus one registration per non-built-in function. The batch path goes first (F5). | One derivative definition per function (grep); finite-difference conformance per registered function |
| F3 | **Mathematics is transported between passes as Arrow relations and partly computed by DataFusion.** | DM-56, DM-38, DM-26 · RCA §2 DEFAULT, RCA §5 | encodes at P3, the inferred-stage projection and P10; decodes at P4 (`passes/p4/predicates.rs:166-185`, `load_untyped`) and case preparation (`session/case.rs:214-226`), plus P6/P7/P9 (agent report); 41.2k generated lines in `pse-relations` compiled/inferred/normalized math and expression families (measured by `wc`), 9.7k in `pse-runtime/src/generated/mathir_{sink,source}.rs`, 1.0k of generators; P9 builds and executes one DataFusion plan per method × parameter × tuple (`passes/p9/parameters.rs:161-172` → `passes/parameter_indices.rs:235-236`) | Every math extension regenerates relation families; each compile pays full-graph encode/decode; per-tuple plan construction makes P9 scale with tuple count × planning cost (RCA §5: "Do not create one query execution per entity"). The historical heater compile (398–448 s, 15–19.6 GB) was dominated by DataFusion plan construction and publication. | Mathematics stays in memory from S1 to S7; publish only the compiled bundle (§4.7). Delete the math relation families other than bundle tables, their generated adapters and generators. Move P9's per-tuple projection into one set-oriented plan or typed Rust. | Count encodes/decodes per compile (0 expected before publication); generated line count; P9 plan-execution count independent of tuple count |
| F4 | **The ordered, guarded floating-point contract blocks every library CAS, and its surviving purpose has a direct replacement.** | DM-24, DM-40, DM-58 · G6 · RCA §2 MUST (identity wording) | blueprint §7.4 steps 1–3 ("Do not sort commutative children", "does not flatten, merge coefficients … or reassociate"); §7.3 (`x/x → 1` not unconditional); ADR-0047 (accepted: "Unconditional real-algebra rewrites and FMA: rejected by E6"); operator table `NO_REASSOCIATION` (`operators.rs:175-180`). Symbolica documents automatic normalization and "fast math" in evaluators ([numerical evaluation docs](https://symbolica.io/docs/numerical_evaluation)); observed through the public API (§9.1): `parse!("zz_a*3 + zz_b*5 + zz_a*zz_b + x0/x0 + exp(log(zz_a))")` becomes `1+4*zz_a+zz_a*zz_b+5*zz_b` — `x0/x0 → 1` and `exp(log(zz_a)) → zz_a` at construction | Under the current contract Symbolica can only be a backend behind a retained `ExprGraph` (§8, row D), keeping the graph, hashing, folding and their generated relations. Without an explicit replacement, adopting Symbolica would silently erase domain failures such as `x/x` at `x = 0`. | Supersede ADR-0047's principle with: real-algebra equivalence on the admissible domain; admissibility as explicit obligations extracted from the typed AST before normalization (§3); tolerance reproducibility (the §7.3 rule already disclaims bitwise equivalence). Amend §7.3/§7.4 and RCA §2's "ordered operands" to "canonical under the declared algebra policy; identity from the typed definition". | Obligation negative tests; differential test against an authored-order reference evaluator on every fixture |
| F5 | **Numerical evaluation is built on DataFusion vocabulary and a dead batch path.** | DM-38, DM-58, DM-39 · RCA §1 | scalar opcodes are DataFusion `Operator` values (`pse-numerics/src/scalar.rs:48`, `native_operator` `:69-85`); functions resolve by DataFusion `ScalarUDF` identity (`bindings.rs`); `EvaluationProgram::compile`, `NumericalOperation` and `ScalarMath` are called only by tests and `benches/benches/native_consolidation/numerical.rs:49,145` (grep); the batch route's build time grows quadratically: 0.14 s → 8.3 s → 24.8 s for 800 → 8,000 → 16,000 equations (§9.1) | 1.8k production lines (agent estimate) maintain a second numeric path nothing uses; the numeric layer changes with DataFusion's version; the scalar interpreter has no CSE, Horner or JIT. | Delete the batch path now (independent of Symbolica). Replace the interpreter with Symbolica evaluators (S4). | Crate compiles with no DataFusion dependency in the numeric path; evaluator throughput benchmark in CI-smoke |
| F6 | **A recovered evaluation error brands a converged solve as failed.** | G2, G5 · DM-30, DM-08 | `driver/callbacks.rs:21-27` stores `workspace.failure = Some(error)` on any failed callback and never clears it; `native/output.rs:18-26` tests `failure.is_some()` before Ipopt's status | An unbounded variable inside `log` or `sqrt` whose trial point leaves the domain, after which Ipopt cuts the step and converges: published `ipopt_status = 0`, `termination = EvaluationFailure`. Only the initial-point failure is tested. | Record callback failures as attempt events; derive termination from the solver status; treat a failure as terminal only when the solver stops because of it. | Fixture whose line search leaves `log`'s domain and converges |
| F7 | **There is no specialization-level compilation of mathematics for numerics.** | DM-18, DM-33 · RCA §2, §4 | no P12; numerical requests are built per case from the whole canonical graph (`case/prepare.rs:205-211`, agent report); Plan 13 W09 asks for "one compiled body per specialization with explicit formal slots" but the numeric path consumes whole graphs | When P12 is added in the current style, every instance and cell becomes scalar nodes in one program; compile cost and memory then grow with instances, and every structural edit rebuilds it. | Compile per specialization, bind per instance (§4.4). Measured: 1.9 ms per body regardless of instance count, against 3.4 s for a monolithic optimized evaluator at 8,000 equations (§9.1). | Build-time scaling test: bodies compiled ∝ distinct specializations; adding a unit compiles no body |
| F8 | **Solver integration is hard-coded and loses information.** | DM-19, DM-43, DM-47 | options fixed in `driver.rs:224-244` (`linear_solver mumps`, `nlp_scaling_method user-scaling`, `hessian_approximation limited-memory`, `acceptable_tol = tol`); `problem.rs` `HessianPolicy` has one variant; iteration capture drops regularization, `d_norm`, `alpha_du` (agent report); published `constraints` are Ipopt's `g` (`native/output.rs:64`) | IDAES's default profile (`gradient-based` scaling) cannot be selected; an exact Hessian cannot be requested; a run record cannot be reproduced on a host with HSL (§18.3). | One TNLP-shaped adapter reading `solver_profiles`; exact Hessian from S4; natively recomputed residuals. Keep `pse-ipopt-sys` and its `catch_unwind` driver: the `ipopt` 0.6 crate aborts the process on a callback panic (Tested, §9.2). Add POUNCE behind the same adapter. | Profile round-trip test; exact-Hessian solve of HS071 and a flash through both backends |
| F9 | **The brief's Symbolica → oximo → Ipopt route discards the derivatives Symbolica produces.** | DM-24, DM-43, DM-39 | oximo's derivative oracle is `pub(crate)` and `Solver::solve` takes only the model; stable oximo-pounce uses forward differences, a dense m×n Jacobian and L-BFGS (Interface-checked, agent probe); measured 2.30 / 24.3 / 590 ms vs 0.67 / 1.18 / 3.06 ms for exact sparse POUNCE at n = 50 / 200 / 800 (§9.2); oximo-expr has no conditional or external-function node | A flowsheet NLP routed through oximo would lose exact derivatives, conditionals and kernels, and converge slower (HS071: 155 iterations to "acceptable", no duals, vs 9). | NLP goes directly to the TNLP adapter (F8). oximo only for LP/MILP/QP/SOCP (e.g. an exact tear-selection MILP on HiGHS) and ASCII export behind a conformance test. | NL export conformance against the project's ASL solvers; LP/MILP result parity across HiGHS and Clarabel |
| F10 | **Pyomo is foundational in the design and absent from the code.** | G7 · DM-04, DM-43, DM-02 | D12 / ADR-0015 (accepted), §21.2–§21.4, §18.7 GDP row, §19.4 parmest, §24.1/§24.2 native/Pyomo conformance, §25 phase exits, ADR-0075 (proposed: "solver approaches are swapped in Pyomo, not in Rust"); `python/pse/adapters/pyomo/__init__.py` (docstring), `crates/pse-backend-pyomo/src/lib.rs` (stub); uncommitted `pyproject.toml` adds `pyomo>=6.10.1`, `pint`, `networkx`, `scipy` as core floors, contradicting its own "Not core" comment and §3.1 `==` pins | Scope, acceptance and tear selection depend on a component that does not exist and that the maintainer has now withdrawn; the core dependency would put Pyomo in every install. | Supersede D12; withdraw ADR-0075 and R-32; delete the stub crate and adapter; keep IDAES parity test-only; tear selection native (existing petgraph heuristic plus an exact MILP on HiGHS via oximo); revert the core dependency change. | Import-linter: no `pyomo` import outside `pse.parity`; `pip install pse-arrow` resolves without Pyomo |
| F11 | **Symbolica's symbol state is process-global, and term order depends on which symbol a process created first.** | G6 · DM-15, DM-31, DM-48 · RCA §3, §4 | observed (§9.1): the same expression printed as `1+4*zz_a+zz_a*zz_b+5*zz_b` or `1+5*zz_b+zz_b*zz_a+4*zz_a` depending only on which symbol was created first, while `to_canonical_string()` was identical in both processes; a second probe found 5 of 387 evaluator outputs differing by one ulp between such processes; documented: process-global, append-only symbol state | Term order — and therefore evaluator summation order — depends on what the process compiled first, so a hash of printed atoms or of evaluator output bits is unsound for reuse; per-instance symbols would grow the registry for the life of a Python session. | Register slot and kernel-function symbols once at start in a fixed order; use formal slots, not per-instance symbols; derive identity from typed definitions, never from atoms; record the Symbolica version in artifact identity. | Two-process determinism test (canonical strings equal, residuals within tolerance) |
| F12 | **Physical typing is domain code that no library owns, and it must run before normalization.** | DM-06, DM-07 · D5 | `pse-quantity/src/infer.rs` distinguishes `Add` from `Sub` on point quantities (agent report, lines 540-556); Symbolica has no `Sub`/`Div` atoms (documented normalization), so `T_a − T_b` becomes `T_a + (−1)·T_b` | Typing after lowering would reject valid differences or accept invalid sums. | Keep `pse-quantity` as the authority; re-host the P10 driver (`canonicalize.rs`, 2.3k lines) over the typed AST per specialization; do not adopt uom (compile-time only, no kernel consumer, no currency) or a runtime-units crate (none models kind, basis or reference state — agent probes). | Existing inference tests on the typed AST |
| F13 | **Generated code dominates the repository, and much of it encodes mathematics or row codecs a library would own.** | DM-52, DM-56 | 588k generated lines in `crates/*/src/generated` (measured); 957 `ArrowValue` and 450 `RelationRow` impls; the `pse-quantity` generated fixture (29.7k lines) is test-only (`fixtures` feature, `lib.rs:56,71`) | Every schema change regenerates tens of thousands of lines; compile time and review cost scale with them. | After F3, re-measure; spike `serde_arrow` for row codecs (agent proposal); emit the quantity fixture compactly. | Generated-line count per family before and after |
| F14 | **Symbolica's default features change process-wide behaviour of every crate linked with it.** | G4 · DM-28, DM-04, DM-50 | Tested (second probe, §9.2): the default `faster_alloc` feature makes mimalloc the global allocator — a host crate with its own `#[global_allocator]` fails to compile; the default `tracing_max_level_info` enables `tracing/release_max_level_info`, so `STATIC_MAX_LEVEL` is `INFO` in the whole release binary; the first Symbolica log call installs a global tracing subscriber writing to stdout, after which the host's `set_global_default` fails (documented as `GlobalSettings::initialize_tracing`); C++ export defaults compile with `-march=native -ffast-math` | Through Cargo feature unification, the solver's own `tracing::debug!` iteration record (`driver/callbacks.rs`, target `pse_backend_native::solver`) and DataFusion's debug instrumentation are compiled out of release builds; pse-py's tracing initialization loses to Symbolica's stdout subscriber; a native-CPU C++ evaluator would violate the repository invariant "Never `target-cpu=native`". | Depend on `symbolica` with `default-features = false` and an explicit feature list (`native_code_generation` for JIT, `bincode`, and either `integer-malachite` + `float-astro` — Tested pure-Rust, 103 dependencies — or the GMP pair); set `GLOBAL_SETTINGS.initialize_tracing = false` before first use; use JIT or interpreted evaluators, not C++ export with default flags. | Governance test: resolved `symbolica` features exclude `faster_alloc` and `tracing_max_level_info`; `STATIC_MAX_LEVEL` is `TRACE` in release; the host subscriber is installed after a Symbolica call |
| F15 | **faer reports success on singular linear systems, so Newton and block solves must classify failure themselves.** | G2 · DM-08, DM-40 | Tested (§9.2): numerically singular systems return `Ok` with NaN/inf solutions; one structurally singular case returns `Err(SymbolicSingular)` or `Ok` with ±inf depending on the simplicial or supernodal path; on a 30k KKT test the default symmetric-indefinite path returned NaN without an error; supernodal LU bits change with the thread count | A block-triangular initialization (§17) that trusts `Ok` would propagate NaNs into the next block and report a numerical failure far from its cause; a diagnostic's hashed result would change with the machine's core count. | Classify with structural rank from the matching (before factorization), then finiteness and residual checks after each solve; keep KKT factorization inside Ipopt/POUNCE; pin faer's `Par` for any durable or hashed result. | Singular and near-singular fixtures per path; thread-count determinism test |

**Applicability.** Groups 1 (authority), 2 (types, invariants), 5 (derivation and lowering), 6
(execution, failure), 7 (reuse), 8 (mechanism and cost), 9 (providers, capability) and 12
(leverage) carry the findings; the scope is a compiler, numeric backend and provider change.
Group 3 applies through F4/F11 (identity of derived math); group 10 through the source map and run
records (F8, §4.7); group 11 through generated code (F13) and conformance tests (§9.3). Group 4
applies only as DM-18 (F7): templates and bindings are unchanged. RCA §1, §2, §3, §4, §5, §7 and §9
bear on the scope; RCA §5.3/§5.4 (heuristic graph results, temporal graphs) do not, because the
structural analysis is exact and untimed; RCA §6's cycle rules are already met by `pse-structural`.

## 8. Alternatives and architectural leverage

### 8.1 Comparison

| Alternative | Semantic duplication and extension locality | Correctness and operational risk | Implementation and maintenance | Performance evidence | Verdict |
|---|---|---|---|---|---|
| A. Current baseline | two derivative engines; 8 handwritten sites + 4 generated trees per function; relational transport | F1–F8 | largest; bespoke CAS-like code plus generated families | interpreter 41–46 ns/output (§9.1); no CSE/JIT; batch build quadratic | Reject |
| B. Brief as written (Symbolica authoritative, oximo as the solver-model layer for NLP, faer everywhere, uom) | low for math; oximo re-derives derivatives | loses exact derivatives, conditionals and kernels on the NLP route (F9); uom duplicates `pse-quantity`; "Symbolica authoritative" conflicts with D1 unless it is derived | medium | FD+dense route 193× slower at n = 800 (§9.2) | Reject the oximo-NLP and uom parts; keep the rest |
| **C. Recommended target (§4)** | one IR; function = one registration; derivatives defined once | depends on F4's contract (decided, U1) and F11's controls | smallest bespoke surface: typing, obligations, binding, adapters | per-specialization JIT 20–26× faster than the current interpreter for 800–16,000 equations; 1.9 ms per body build (§9.1) | **Select** (decided) |
| D. Simpler viable alternative: keep the typed `ExprGraph` through P10, lower to Symbolica only at the numeric stage (no ADR-0047 change) | Symbolica replaces derivatives and evaluation only; graph, hashing, folding, relations remain | fewest authority changes; keeps ordered semantics | removes ~1–1.7k lines in `pse-mathir` plus the batch path; keeps ~6.9k | same evaluator gains as C | Not selected (U1, U2): it retains most of the bespoke math the directive targets |
| E. Symbolica-free stack (typed `ExprGraph` + echidna/num-dual AD + symjit or cranelift JIT, no Symbolica) | still owns simplification and canonicalization | echidna churns (0.8 → 0.15 in three months, agent report) | more bespoke than C | not measured | Not selected; the documented fallback if Symbolica ever has to be replaced |

**Why C over D.** D is the smallest change that keeps the blueprint's numerical contract, and three
evidence agents independently preferred it. It is rejected here on the governing objective: it
retains the ordered hash-consed graph, the guarded folder, canonical hashing, the relation families
and their generated code — the bespoke mathematics the maintainer wants gone — to protect a
contract whose defensible purpose (domain failures stay visible) the obligation mechanism
provides directly. The maintainer decided accordingly (U1, U2).

### 8.2 Abstractions justified by current needs

- **One TNLP-shaped adapter trait with two implementations** (Ipopt C with MUMPS for
  IDAES-comparable runs; POUNCE for pure-Rust in-process solving without the C toolchain or solver
  container). Two concrete backends with different
  jobs justify the seam; a solver plug-in registry does not exist and should not be built.
- **The function registration table**, generated from the operator table, is the one place a
  function's evaluation, derivative, quantity rule and capability are declared.
- **Binding tables** are data, not a framework.

### 8.3 Library leverage beyond the math core

| Bespoke mechanism | Library | Verdict |
|---|---|---|
| exact tear selection (ADR-0075 routed it to Pyomo) | oximo + HiGHS MILP; petgraph greedy feedback arc set stays as the heuristic | adopt with F10 |
| Newton and block-triangular initialization (§17), scaling Newton (§16.2), diagnostics (§15.4–§15.5) | faer sparse LU with symbolic reuse (numeric LU of a 100k-unknown chain in 3.2 ms, §9.2); failure classified by structural rank, finiteness and residual, never by `Ok` | adopt when §17 is built |
| Dulmage–Mendelsohn and block-triangular assembly (`pse-structural/src/incidence.rs:131-313`, ~150 lines of glue) and the `rust-igraph` matcher | pounce-presolve (DM + BTF, pure Rust, same family as the POUNCE backend) with an index-range guard; petgraph's matcher, already a dependency, was the fastest at 100k (29 ms vs 82 ms for rust-igraph) | adopt with S6; drop `rust-igraph` |
| opaque kernel derivatives | num-dual 0.15 (ADR-0022, already pinned in the blueprint, absent from `Cargo.toml`) | adopt with the first kernel |
| dynamics (§13.6) | diffsol 0.16 (its linear algebra is faer 0.24, agent report — §13.6's "separate stack" note is outdated) | later |
| Arrow row codecs (957 `ArrowValue`, 450 `RelationRow` generated impls) | `serde_arrow` with registry-supplied schemas (agent proposal) | spike after F3 |
| buffer reservation tracking (`pse-columnar/src/owned_buffer.rs`, 1.4k lines) | Arrow 59 `pool` feature + DataFusion `ArrowMemoryPool` (agent report) | spike |
| `closed_enum!` macro | strum (already in `Cargo.lock`) | small; do with F13 |
| rewriting engine | egglog | not needed: Symbolica owns algebra |
| symbolic-JIT without Symbolica | symjit (MIT; agent notes its default enables FMA fusion and reads `./symjit.toml`) | only under alternative E |

**What remains ordinary code.** The DSL front end (`pse-authoring/src/dsl`, 1.8k lines, of which
the parser is 601 lines of hand-written recursive descent — §7.7's statement that `winnow` parses
is outdated; `winnow` only tokenizes, `lexer.rs:6-8`), template
specialization, physical typing, obligation extraction, binding-table construction, the
TNLP adapter and its `catch_unwind` barriers, result mapping, and the ~150 lines of DM/block glue
extraction and back-substitution between blocks (faer has no KLU-style block solver). The DM/BTF
assembly itself moves to pounce-presolve; faer, rsparse and sprs have no matching, DM or BTF
routines, and SuiteSparse BTF works only through hand-declared C bindings (§9.2).

## 9. Verification and measurement plan

### 9.1 Measurements taken for this review (Measured)

Conditions: AMD Ryzen 9 9950X3D (32 threads, 188 GB), Linux 7.0, rustc 1.98.1, release profile
(`opt-level 3`, thin LTO, 16 codegen units), single-threaded evaluation. Other review agents were
running concurrently (including a multi-threaded faer benchmark), so absolute times carry noise of
unknown size; each figure is one run of the stated repetitions. Workload: a deterministic synthetic flowsheet chain; each unit has 8 variables and 8
residuals — three component balances, one energy balance with `F·(cp·(T−Tref) + a·T²)` terms,
three Antoine-form VLE relations `K·P − exp(A − B/(T + C))`, and a `sqrt` pressure drop — giving
28 Jacobian nonzeros per unit. "Full evaluation" = all residuals plus all Jacobian nonzeros at a new
point (alternating two points so cached values are invalidated). Residuals match a reference
tree-walk exactly (max |Δ| = 0). Sources:
`scratchpad/probes/compare-probe/src/{main,topo,workload}.rs` (session scratch, not committed).

**Current evaluator (production route and batch route).**

| Equations | Outputs per evaluation | Production route build (lower + differentiate / bind) | Production evaluation | ns per output | Batch route build | Batch one-row evaluation |
|---:|---:|---|---:|---:|---:|---:|
| 800 | 3,589 | 3.2 ms / 0.9 ms | 163.8 µs | 45.6 | 139 ms | 11.8 ms |
| 8,000 | 35,989 | 46.1 ms / 46.8 ms | 1,571.9 µs | 43.7 | 8.3 s | 204 ms |
| 16,000 | 71,989 | 90.9 ms / 136.7 ms | 2,939.8 µs | 40.8 | 24.8 s | 407 ms |

The production route's construction is roughly linear; its evaluation is an interpreter at about
41–46 ns per output. The batch route (no production consumer) builds in quadratic time and
evaluates a one-row batch 70–140× slower than the scalar interpreter.

**Symbolica 3.0.0 on the same expressions (public API only, hobbyist key).** Expressions were built
with `parse!`, differentiated with `AtomCore::derivative` for each structurally present variable,
and compiled with `AtomCore::evaluator_multiple(&outputs, &params).build()` (default optimization
settings: Horner schemes and common-subexpression elimination) then
`map_coeff(|c| c.re.to_f64())`; JIT via the documented `jit_compile::<f64>`. *Monolithic* = one
evaluator over every residual and Jacobian entry of the system. *Per specialization* = one
evaluator for the unit body over 13 formal slots (8 own variables, 5 upstream values), called once
per unit with Rust gather/scatter into global residual and Jacobian arrays. Sources:
`scratchpad/probes/compare-symbolica/src/{main,workload}.rs`.

| Equations | Monolithic build: parse + derive / optimize / JIT | Monolithic evaluation (interp / JIT) | Per-specialization build (once, all steps) | Per-specialization evaluation (interp / JIT) | Current production evaluation |
|---:|---|---|---:|---|---:|
| 80 | 0.8 + 0.5 / 7.8 / 1.7 ms | 1.04 / 0.26 µs | 1.9 ms | 2.03 / 0.72 µs | 17.4 µs |
| 800 | 4.1 + 4.8 / 296 / 25 ms | 10.3 / 3.4 µs | 1.9 ms | 18.2 / 8.0 µs | 163.8 µs |
| 8,000 | 44 + 48 / 3,434 / 1,425 ms | 152.7 / 107.6 µs | 1.9 ms | 158.0 / 67.8 µs | 1,571.9 µs |
| 16,000 | not run | — | 1.9 ms | 290.8 / 114.9 µs | 2,939.8 µs |

Accuracy: per-specialization residuals match the reference tree-walk to ≤ 7.8e-16 absolute;
monolithic to ≤ 4.7e-10 absolute on residuals of order 10⁵–10⁶ (optimization reorders
arithmetic, as documented); Jacobian entries match central finite differences to ≤ 6e-10
relative (finite-difference error dominates); JIT agrees with the interpreter to ≤ 5e-13 relative.
At 8,000 equations the monolithic JIT reported "SIMD (f64x4) request downgraded to scalar f64
due to the stack limit".

**Interpretation.** Per-specialization JIT evaluation is 20–26× faster than the current
production interpreter across 800–16,000 equations, and per-specialization interpreted
evaluation 9–10× faster; the body compiles once in 1.9 ms. These measure evaluation only, on a
synthetic workload, single-threaded, without domain-obligation outputs and without a solver; they
do not establish end-to-end solve time (DM-39).

**Observed Symbolica behaviour relevant to the contract (Tested, public API).**
- `parse!("zz_a*3 + zz_b*5 + zz_a*zz_b + x0/x0 + exp(log(zz_a))")` prints
  `1+4*zz_a+zz_a*zz_b+5*zz_b`: `x0/x0 → 1` and `exp(log(zz_a)) → zz_a` at construction (F4).
- Run in two processes that first create `zz_a` then `zz_b`, or the reverse, the same text prints
  `1+4*zz_a+zz_a*zz_b+5*zz_b` and `1+5*zz_b+zz_b*zz_a+4*zz_a` respectively; `to_canonical_string()`
  is identical in both (F11).

### 9.2 Library evidence from agent probes (Tested / Measured, conditions in the probe logs)

- **oximo 0.7.0.** ASCII NL files solve with the project's Ipopt 3.14.20/ASL 20241111 and match
  Pyomo+Ipopt to 12 printed decimals (FLASH3 T = 386.672373777599 K); `scaling_factor` suffixes are
  honoured. Binary NL is rejected by ASL ("bad line N", abort) because bound-type codes are written
  as 4-byte integers. No public SOL reader. Stable nonlinear route: finite differences, dense
  Jacobian, L-BFGS — 2.30 / 24.3 / 590 ms against 0.67 / 1.18 / 3.06 ms for exact sparse POUNCE at
  n = 50 / 200 / 800 (single run each, one pinned core). Status mapping merges acceptable with
  succeeded, and four numeric failures into one.
- **POUNCE 0.12.0 TNLP** with hand-coded exact sparse Jacobian and Hessian (a stand-in for
  Symbolica evaluators): FLASH3 and HS071 solved with Ipopt's iterates (9 iterations for HS071);
  `derivative_test=second-order` clean; user scaling, stop-by-callback and panic unwinding work;
  median 533 µs (FLASH3) and 693 µs (HS071) per solve including setup, vs 754 / 1,375 µs through the
  `ipopt` crate on system Ipopt 3.11.9 + MUMPS 5.6.
- **`ipopt` 0.6.0 crate.** A callback panic aborts the process (SIGABRT); `unsafe impl Send`
  without `P: Send`; `dbg!` output on every solve; needs a C++/CMake shim.
- **faer 0.24.4.** Pattern built once with `try_new_from_indices`; numeric values written in place
  through a precomputed slot map (0.25 ms vs 0.41 ms for the allocating public refill at 498k
  nonzeros); symbolic LU once, numeric LU per Newton iteration, converging on every problem.
  Sequential median numeric LU: 1-D chain 0.26 ms (10k) and 3.2 ms (100k); 2-D five-point 4.7 ms
  and 86 ms (symbolic 29 ms); a 3-D stress case 9.1 s against 1.07 s for AMD-ordered Cholesky
  (ordering fill). Numerically singular 2×2, stored-zero diagonal and rank-deficient Laplacian
  systems all returned `Ok` with non-finite solutions; a structurally singular case returned
  `Err(SymbolicSingular)` on the simplicial path and `Ok` with ±inf on the supernodal path (probe
  `out/singular.txt`). Supernodal LU results are bit-identical at a fixed thread count and differ
  across 1/4/8/16/32 threads; at 32 threads on the loaded host the 2-D LU took 3.2 s against 81 ms
  sequential.
- **Structural crates.** pounce-presolve 0.12 reproduced hand-computed DM partitions for a
  structurally singular 6×6 and a unique four-block order for a 7×7, invariant under relabelling
  (probe `out/structural.txt`, all PASS), and silently dropped an out-of-range entry. SuiteSparse
  BTF (via hand-declared `klu_sys` bindings) agreed. Perfect matchings at 100k on a hard random
  instance: petgraph 29 ms, rust-igraph 82 ms, pounce Hopcroft–Karp 156 ms (single runs, loaded
  host). faer, rsparse and sprs provide no matching, DM or BTF.
- **Second Symbolica probe** (different workload: 16 residuals per unit; unlicensed restricted
  mode; median of 7; load average 10–42). Monolithic evaluators: default optimization took 0.56 s /
  5.4 s / 36 s to build at 100 / 1,000 / 5,000 units and bought ~7–22% throughput over no
  optimization; JIT compile took 0.08 s / 4.0 s / 107 s and at 5,000 units was slower than the
  interpreter (560 vs 627 evaluations/s). A monolithic Lagrangian Hessian (115k nonzeros at 5,000
  units) was derived in 0.48 s and built in 12.7 s. Five of 387 outputs differed by one ulp between
  two processes that registered symbols in different orders. C++ export defaults compile with
  `-march=native -ffast-math`. These corroborate U3 and F11/F14.

### 9.3 Required verification for the target

| Claim or risk | Label now | Test, analysis or benchmark | Conditions and expected result |
|---|---|---|---|
| Domain obligations preserve every authored failure | Proposed | negative suite: `x/x`, `exp(log x)`, `log(0)`, `sqrt(−1)`, `0^−1`, each unconditional, inside a taken `if` branch and inside an untaken one | failure reported with span exactly when the authored expression fails on the taken path |
| Normalization preserves values on the admissible domain | Proposed | differential test: authored-order reference evaluator of the typed AST vs Symbolica evaluator (interpreted and JIT) on every fixture at 100 seeded points | §7.3 tolerance rule; no NaN on one side only |
| Derivatives are correct | Proposed | central-difference check of every Jacobian and Hessian entry per registered function and per fixture; Ipopt/POUNCE `derivative_test=second-order` | relative error < 1e-6 (Jacobian), 1e-5 (Hessian) |
| Determinism | Proposed | two processes registering unrelated symbols in opposite orders compile the same body | equal canonical strings; residuals within tolerance |
| Per-specialization reuse | Proposed | add a unit; edit a value; edit a template; compare with clean recompilation (RCA §9) | bodies recompiled = 0 / 0 / 1; results equal |
| Capability truthfulness | Proposed | generated test: every accepted opcode/function has a registration, an evaluator test and a declared solver route | no advertised route without an implementation |
| Solver status fidelity | Proposed | fixture: trial point leaves `log`'s domain, solve converges | termination from the solver; event recorded |
| End-to-end performance | Proposed (DM-39) | heater, flash and 1,000-cell distributed fixture: compile, bind, first evaluation, solve, publish; memory peak | recorded against this review's §9.1 baseline and the Plan 09 historical numbers |

**Cost accounting.** The material categories are compile (S2–S4 per specialization), binding and
assembly (S5–S6 per structure revision), evaluation per callback, and publication. The largest
historical cost — DataFusion plan construction and relational publication of intermediate math —
is removed by construction; the claim that the target is faster end to end is a hypothesis until
the §9.3 end-to-end row runs.

## 10. Exceptions, unresolved decisions and required authority changes

### 10.1 Decisions (made by the maintainer on 2026-09-23)

| # | Decision | Options considered | Decided |
|---|---|---|---|
| U1 | Numerical semantics of the model graph | (a) keep ordered guarded floating-point (ADR-0047) → alternative D; (b) real-algebra equivalence with explicit domain obligations (F4) | **(b)** |
| U2 | Where Symbolica starts | (a) after P10 (alternative D); (b) at the typed template definition | **(b)** |
| U3 | Evaluator granularity | (a) monolithic per case; (b) per specialization with binding tables | **(b)** — measured faster and cheaper to build (§9.1); a monolithic evaluator stays available as an optimization only if later measurement favours it for small systems |
| U4 | NLP backends | (a) Ipopt C only; (b) Ipopt C + POUNCE behind one adapter; (c) POUNCE only | **(b)**; IDAES-comparable iteration counts come from the Ipopt C/MUMPS build |
| U5 | Licensing and distribution | design for distribution, or for personal use | **Personal use.** The design assumes the maintainer's multi-core hobbyist Symbolica key; no licensing or distribution mechanism, policy or restriction is part of the target |

No SHOULD-level deviation needs an exception record: the target follows RCA's DEFAULT-level choices. U1(b)
changes contract text that RCA §2 states as a MUST (ordered operands in expression identity); the
target keeps that property for identity (the typed definition's operands are ordered) and needs
only RCA §2's wording amended so derived library forms are not held to it (§10.2).

### 10.2 Required authority changes (routes per AGENTS.md)

| Blocking text | Status | Target | Route |
|---|---|---|---|
| D6 "Expression nodes … are relations … Use native DataFusion `Expr` wherever it faithfully carries required symbolic meaning" | blueprint (ADR-0068 proposed) | math is in memory from parse to execution; relations only for the published bundle; no DataFusion `Expr` for mathematics | math ADR + design review; `design:` PR |
| §7.1 (graph stored in `compiled.math_*`, `argument_ordinal` authoritative), §7.2/§7.3 (43-opcode table as authority for evaluation order and lowering), §7.4 steps 1–3, §6.11 `rewrite_mode` | blueprint (ADR-0054 proposed; ADR-0047 accepted) | typed AST + Symbolica; operator table becomes the function registration table; obligations replace ordered semantics | same math ADR |
| ADR-0047 "Unconditional real-algebra rewrites and FMA: rejected by E6" | accepted | superseded by the obligation contract (F4) | supersede: ADR + design review |
| ADR-0037 / R-06: a rewrite engine "cannot be on a correctness path" | accepted | scope to egglog; Symbolica qualified by §9.3 determinism and differential tests | supersede in the math ADR |
| §6.15.4 "No second AST is introduced"; ADR-0078 "A bare expression DAG cannot represent that contract" | proposed | the typed AST and Symbolica body are the two layers; `ExprGraph` removed | amend ADR-0078 in its pending PR |
| D1 / ADR-0004: Rust structs holding model data are generated views | accepted | unchanged: Symbolica bodies are derived, not model authority | none (the target keeps D1) |
| D11 "evaluation programs … derived from relations"; §18.2 (reverse-mode sweeps, ordered regions, DataFusion batch) | blueprint (ADR-0047 accepted) | evaluators derived from compiled bodies; mechanism-neutral contract (§3) | math ADR |
| §18.3, ADR-0028, ADR-0038 (Ipopt C API only; `-sys` crate) | accepted | keep `pse-ipopt-sys`; add POUNCE behind the adapter; profiles drive options | backend ADR (short if Ipopt C stays) |
| D12 / ADR-0015, §21.2–§21.4, §18.7 GDP, §19.4, §19.7, §19.8, §24.1/§24.2 Pyomo conformance, §25 exits, AGENTS.md "generated Pyomo backend" | accepted / blueprint | no production Pyomo; IDAES parity test-only | backend ADR + design review (Python boundary contract) |
| ADR-0075, R-32 | proposed / open | withdrawn; native tear selection | withdraw in its PR; close R-32 |
| §3.2 crate map (`pse-numerics`, `pse-backend-pyomo`, `pse-backend-nl`) | blueprint (ADR-0076 proposed) | remove `pse-numerics` and `pse-backend-pyomo`; `pse-backend-nl` only when export is scheduled | ADR + design review (crate removal) |
| §3.3 "the KKT solves are not faer's"; faer and num-dual pinned in §3.1 but absent | blueprint | faer adopted for Newton/diagnostics; POUNCE may use its own linear algebra | `design:` PR |
| RCA §2 MUST "Expression identity must preserve … ordered operands" | review standard | identity preserves the typed definition's ordered operands; derived Symbolica forms are canonical under the declared algebra policy | amend RCA (governance change) |
| Plan 13 W09/W11 ("evolve `ExprGraph`", "existing guarded scalar instruction programs", "keep … NL and Pyomo") | active plan | successor plan carrying W19/W20, I18/I19 and Plan 10 obligations | new plan in `docs/plans/` |

## 11. Decision and implementation changes

**Decision: Revise the current pipeline; Accept the target as the design direction.** The current
math pipeline fails G1, G2, G5 and G7 and cannot solve indexed, kernel-bearing or unit-converting
models. The library-owned target, with the maintainer's decisions U1–U5, passes every gate at the
document stage (§6). It becomes repository authority when the §10.2 ADRs and `design:` PR land,
and its claims move from Proposed to Tested as the §9.3 suites pass.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 (now, no dependency on Symbolica) | Delete the batch `Expr` path, `differentiate.rs`, the `ScalarMath` entry and their tests | DM-02, DM-58 | `pse-numerics` has no DataFusion `SessionState` use; one derivative engine | grep gate on `EvaluationProgram::compile` |
| 1 | Fix F6 (recovered evaluation errors) | DM-30, DM-08 | converging fixture with a failed trial point publishes success | the fixture |
| 1 | Revert the uncommitted core `pyomo`/`pint`/`networkx`/`scipy` dependencies; delete `pse-backend-pyomo` and the adapter docstring package; narrow capability claims (F1, F10) | DM-43, DM-59 | install without Pyomo; generated capability matrix | import-linter rule |
| 2 | Math ADR recording U1–U3: obligation contract, typed AST, per-specialization bodies, identity scheme, supersessions in §10.2 | DM-24, DM-15, DM-40 | design review Accept or Accept-scoped | ADR lint |
| 2 | Backend ADR recording U4: TNLP adapter over Ipopt C and POUNCE, profiles, exact Hessian, no Pyomo, oximo scope | DM-19, DM-43, DM-28 | design review | ADR lint |
| 3 | Add `symbolica` with `default-features = false` and explicit features; disable its tracing initialization (F14) | DM-28 | governance feature test passes | the governance test |
| 3 | S2–S4: typed AST lowering to Symbolica, obligations, registrations, derivatives, evaluators (+JIT); delete `ExprGraph`, canonical hashing, folding, `NumericalTopology`, scalar interpreter | DM-56, DM-25 | §9.3 obligation, differential and derivative suites | the suites in CI |
| 3 | S5–S6: binding tables replacing P12; faer global pattern; structural analysis from local patterns; publish incidence/blocks/DM | DM-18, DM-43 | indexed heater and 1,000-cell fixtures solve; §9.3 reuse test | reuse-vs-clean test |
| 3 | Delete the relational math transport and its generated families, adapters and generators (F3) | DM-56, DM-38 | 0 encodes/decodes before publication | encode/decode counter test |
| 4 | Exact Hessian through the Ipopt driver; POUNCE backend; `solver_profiles`; native residual recomputation (F8) | DM-19, DM-47 | HS071, flash, heater through both backends | profile round-trip test |
| 4 | oximo for LP/MILP/QP/SOCP and ASCII export; exact tear-selection MILP (F9, F10) | DM-43, DM-42 | ASL conformance in the solver image | export conformance test |
| 5 | End-to-end measurement (§9.3) against §9.1 and the historical baseline | DM-39 | recorded receipts | benchmark smoke |

**Deletion ledger (Proposed sizes; handwritten production lines unless stated).**

| Removed | Approximate size | Replaced by |
|---|---:|---|
| batch `Expr` path in `pse-numerics` (stages, arena, differentiate, finite, vector, operation, compile) | 1.8k (+~1k tests) | nothing |
| `pse-numerics` scalar interpreter and bindings | ~0.8k | Symbolica evaluator wrapper (~0.15k) |
| `pse-mathir` numerical lowering, derivative, request | 1.1k | Symbolica lowering (part of S3) |
| `pse-mathir` graph, canonical, hash, fold, topo, view, most of payload/node, relation loader | ~2.9k | typed AST (existing parser output) |
| `pse-mathir` `canonicalize*` driver | 2.3k rewritten, not deleted | typing driver over the typed AST |
| relational math transport: `pse-runtime` mathir relation adapters and generators | ~1.4k + 9.7k generated | nothing |
| math relation families in `pse-relations`, `pse-model`, Python contracts | ~41k (+ projections) generated | compiled bundle tables |
| `pse-backend-pyomo`, Python adapter stub, core Pyomo dependencies | stubs | nothing |
| `pse-quantity` generated fixture emission | ~20k generated | compact fixture emission |
| New code | ~3–4.5k | typed-AST→Symbolica lowering, obligations, registrations, binding and assembly, TNLP adapter, POUNCE backend, bundle publication |

**Final check.** The claims above are labelled at the strength of the evidence behind them; the
supported scope of the current pipeline is narrower than its documentation (F1); and the target's
extension path — a function is one registration, a law is a template, a kernel is a num-dual body
plus a registration — is specified, with the decisions that make it valid named in §10.1.
