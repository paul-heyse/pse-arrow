> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R2 (library-research), Part B: obligations mapped onto the R2 libraries. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R2 Part B: mapping the obligations ledgers onto preparation, reuse and evaluation libraries

This is R2's Part B, a bounded supporting contribution with no verdict and no finding IDs. Its
inputs are W-A (`w-a-interactive-obligations.md`), W-B (`w-b-multicase-obligations.md`) and the
R2 Part A file. The baseline is `main` at `6c368c88a`, read statically. Ledger stage
references use the form `A1.9` (W-A journey 1, stage 9) and `B3.4b` (W-B journey 3, stage 4b).
Library claims are *Interface-checked*, as recorded in Part A. Everything marked **[I]** is
interpretation, and every claimed benefit is *Proposed*.

I re-read four source points for this mapping:

- `pse-runtime/src/math/preparation.rs:25-37` (`BasisKey`);
- `pse-modeling/src/specialize.rs:114-126` (`Bindings.demand`, documented as "Additional member paths requested by the caller");
- `pse-runtime/src/math/artifacts.rs` (DataFusion `Cache` keyed by `ContentHash`);
- `pse-runtime/src/workflow/integrated.rs:199-215` (`self.program.worker(...)` is built on every `integrate` call).

**Rules cited below.** Each label is used in the tables that follow.

- **§14.3-eff** (`mathematics-and-compilation.md`, table at line 34): "Symbolic initialization, native compilation and evaluation are effects outside tracked Salsa queries".
- **§3-salsa** (`workspace-and-dependencies.md:245`): "No I/O, native state or effects in tracked queries".
- **§14.4-basis**: "no compiler, protected read, attempt fence or mutable evaluator is retained as the basis … A previously qualified complete source receipt may be adopted only after a guarded store check verifies a fresh live pin".
- **ADR-0164-box** (proposed): "Persisted complete dependencies govern reuse across restart; Salsa is an optional local accelerator".
- **§14.4-rotation**: "There is no persistent Salsa store; a new process rebuilds from the admitted revision".
- **§19/§21/scope-table**: ephemeral execution "still requires canonical sources". The scope table (`scope-and-open-design.md`) says there is "no durable-to-ephemeral fallback".
- **pse-py docstring**: "modeling sources live only in the canonical store" (`pse-py/src/workflow/modeling.rs`).

---

## 1. Obligations in scope

Each kind is described under three headings:

- **Own** says whether a surveyed library could own the obligation wholesale, partly, or not at all.
- **Need** separates what the functional target requires from what a current rule imposes.
- **Consequence** gives what a mapping would remove and what it would weaken.

### K1. Reuse and invalidation of semantic preparation

This covers A1.9 selected admission, A1.10 basis and bodies, A1.12 views, A3.4 parsed fields, A3.8 Salsa early cutoff, A3.9 changed bodies, B1a.5 lane prepare, and W-B (c) rows for compiled programs, compiler workspace, structural admission and value-dependent products.

**Today there are seven reuse families beside Salsa.** `MathService.modeling_cache` is one DataFusion-backed weighted cache with families for ingress, parsed_fields, selected, numerics, body, encoded_body, frontier, basis, solver views, program and package (W-A (a)). Three further mechanisms sit beside it: the artifact cache keyed by `ContentHash`, `selected_flights`/`basis_flights`, and per-lineage Salsa `compiler_lanes`. That makes two in-process reuse authorities for pure products:

- Salsa records dependencies and backdates inside `pse-compiler`.
- The runtime caches use explicit keys (`BasisKey`, view keys, content hashes) whose equality is hand-specified.

**Own: largely Salsa 0.28.4, within one process.**

- Tracked queries keyed on interned identities can carry everything a pure function of admitted inputs produces:
  - parsed fields;
  - selected closure resolution;
  - specialization basis;
  - template bodies;
  - **compiled programs**, as `Arc<CompiledBody>` with `heap_size`;
  - pattern products and solver-view structure.
- Backdating gives the early cutoff that `PreparationCounts.shared` reports today, and does it by construction.
- Durability tiers separate context (HIGH), definitions (MEDIUM) and values (LOW).
- Salsa does **not** own byte budgets: its LRU is count-based and `heap_size` only reports. quick_cache 0.6.24 (`Weighter`, `get_value_or_guard`) or the existing DataFusion cache can stay the byte-bounded owner of large immutable products, keyed by Salsa-produced identities.

**Need.**

- *Functional:* "a value change must not rebuild structure" and "structural edit reuses what is unchanged".
- *Imposed:*
  - **§14.3-eff and §3-salsa** keep program construction outside Salsa, which is why the separate artifact cache exists.
  - **§14.4-basis and ADR-0164-box** make Salsa an accelerator whose products must be re-qualified against the store before use, which is why selected candidates are rechecked (K2).

  Neither rule follows from the scientific workload. Optimizing and dualizing a Symbolica program is a deterministic, pure function of the body request; its only effects are resource charging and cancellation. Whether the result is deterministic under `cores > 1` is unestablished (Part A §7).

**Consequence.**

- *Removes:*
  - the second reuse authority and its hand-written equality keys (`BasisKey` with `prehash`, view keys, the `selected` cache family);
  - per-family single-flight (Salsa blocks the same key across handles, skill B013);
  - the "what is recomputed on which edit" table of §14.4, which becomes emergent.
- *Weakens:*
  - edits wait for running queries, or cancel them and discard their work (B012/B022);
  - rotation (`rebuild`) drops compiled programs unless a byte-bounded cache holds them outside Salsa;
  - the fresh-workspace equality oracle (§14.4) would have to cover compiled programs.

### K2. Source freshness: store re-reads and rechecks on every solve and edit

This covers:

- A1.8 and A2.3: the root declaration is re-read under its own protect, read and release on every solve, bound solve and inspect.
- A1.9 and A2.3: the cached selected admission is **rechecked against the store** on every preparation (`qualify_selection_window`) under a fresh protection.
- A3.2: a structural edit hydrates the **full parent membership**, which grows with package size rather than edit size.
- A1.11 and A3.6: protection grant and release per call.
- W-A (c): rows for the root declaration, selected admission, revision protection and parent package.

**Own: wholesale by Salsa, in a single-writer in-process session.** In a live session the authored declaration fields are Salsa inputs.

- Reading the root declaration is a tracked read of an input. It needs no RPC and no protection, because nothing outside the session can mutate the inputs.
- A structural edit becomes setter calls on the changed fields only. Its cost then grows with edit size. The skill's B001/B002 controls and Plan 36's I-A1 granular design already assume exactly this.
- No library I surveyed can own *cross-process* freshness. Salsa is process-local, and its persistence is whole-database serde, not a shared store.

**Need.**

- *Functional:* the session must see its own edits, which is trivially true in-process.
- *Imposed:* the per-solve re-reads and rechecks come from **§14.4-basis** ("guarded store check verifies a fresh live pin"), **ADR-0164-box**, the **pse-py docstring** ("sources live only in the canonical store") and §14.3, under which database structural resolution supplies the selection.
- *Product need, from W-B (a):*
  - several Python runtimes may share one store (ADR-0181, proposed);
  - durable workers read sources that the creating process wrote (B1b.3–4).

  These consumers need the store as the *durable and cross-process* source of truth. They do not need a per-solve recheck inside a single-writer session that itself authored the revision. Retention and reclamation by another writer is a store concern (R1).

**Consequence.**

- *Removes:* the per-solve root read with its protections, the selected-candidate recheck, and full-parent hydration on a structural edit. The parsed-field cache (A3.4) becomes a Salsa memo.
- *Weakens:*
  - a second writer to the same lineage, in another process, is no longer seen until an explicit re-sync;
  - the store becomes a publication sink with **write-behind** semantics: an edit is durable only once published. Ephemeral sessions might never publish. That contradicts §19/§21/scope-table today ("still requires canonical sources").

### K3. Path demand in the specialization basis key

This covers A2.3 ("A path set not seen before misses the basis cache, so specialization re-runs for that demand set") and `BasisKey.bindings`, which embeds `specialize::Bindings.demand`.

**Own: partly, by Salsa.** With field-level tracking (B002) and demand held as its own input, specialization splits in two:

- the demand-independent basis, which is the science;
- a demand-dependent projection that makes extra member paths addressable.

A new path set then re-runs only the projection. Whether `specialize` can be factored this way is **[I]**: `demand` is documented as "Additional member paths requested by the caller", but I did not trace whether it can add equations.

**Need.**

- *Functional:* a value overlay names targets, and those targets must be addressable.
- *Imposed:* equality over the complete key is the project's choice. §14.4 lists "full structural bindings" in complete preparation retention. No library imposes it.

**Consequence.**

- *Removes:* re-specialization per distinct overlay path set.
- *Weakens:* nothing found, provided demand only extends addressability **[I]**.

### K4. Compiled-program retention beside Salsa

This covers W-B (c) ("compiled programs in `MathService.entries`, DataFusion `DefaultCache`, content-hash key, byte-weighted"), `artifacts.rs` ("callers cannot supply an independent cache key"), and A1.10/A3.9 body descriptions.

**Own: partly.**

- Salsa could own identity, dedupe and invalidation: a tracked fn on an interned `BodyKey`, of which `grouped.rs:31` already has one.
- A weighted cache keeps the byte bound.
- Symbolica can serialize evaluators and JIT IR (`serde`/`bincode`; `JITCompiledEvaluator::as_bytes`), but both features are **off** in the workspace. That would let a compiled program be a portable byte artifact (see K5).

**Need.**

- *Functional:* compile each distinct body once.
- *Imposed:* **§14.3-eff** places compilation outside Salsa, and the `no-jit;no-simd` evaluator ABI (§14.5) is a rule.

**Consequence.**

- *Removes:* the separate key namespace and epoch invalidation in `artifacts.rs`.
- *Weakens:* compile cost moves into query time, and cancellation then discards partial optimization.

### K5. Cross-process reuse: workers rebuild and recompile

This covers:

- B1b.4/8: a worker rebuilds `ModelingPackage` from the revision and recompiles in its own `MathService` and Salsa lanes, so nothing is shared with the creating process;
- B (c): the engine cache is disabled in `pse-worker` (`CacheBudget::disabled`);
- A1.10: body descriptions are published to the store, but under abi3 Python `local_python_configuration` returns `Err`, so store-received bodies are not reused.

**Own.**

- **Not Salsa.** Its persistence is whole-database serde (B025). It panics on a non-persisted input (B027), and restored interned ids are only valid inside that restored database. It cannot be a shared cross-process cache.
- **Partly Symbolica.** A portable compiled-program artifact can be produced with `bincode`/`serde` on `ExpressionEvaluator`, or as JIT compressed IR. Its identity must include the Symbolica and symjit versions, the optimization settings and the backend profile.
- The project still owns the store identity, the qualification (does it verify against the declared description?) and the producer trust.

**Need.**

- *Functional:* durable studies need separate worker processes. That rests on crash isolation and the claims and leases of W-B (a), under §19.3 "Durable adapter" and ADR-0177.
- *Imposed:*
  - **§14.4-rotation** (no persistent Salsa store) forces each worker to recompile.
  - The abi3 replay refusal (A1.2, `portable.rs`) prevents reuse of published descriptions.
  - `CacheBudget::disabled` is a worker configuration choice, not a rule.

  Recompilation per worker is a consequence of these rules plus process isolation, not of the science. It repeats once per worker, and once per lane whose `StudyCandidateLane.source` is empty.

**Consequence.**

- *Removes:* repeated optimization and dualization per worker and lane, for durable sweeps and 16-way case fan-out.
- *Weakens:* the evaluator ABI must be versioned and qualified across processes, and serialized programs become trusted input.

### K6. Value-lifetime reuse: evaluator scratch, factorizations, native state

This covers:

- A1.12 and A2.5: the view rebind;
- B3.4b: **transient fits build `self.program.worker(...)` and a new integrator on every oracle call** (`integrated.rs:211`);
- B3.4a: steady `CaseWorker`s are retained per experiment;
- B (c): a native session per independent study point, and KINSOL inner sessions per worker thread;
- B5.3: KLU analysis refreshed in recycles.

**Own: the libraries own the primitives; the project owns lifetimes.**

- Symbolica: evaluator scratch per thread, since evaluation is `&mut`.
- faer: a retained `SymbolicLu` with numeric refactorization.
- KLU: `klu_l_refactor` at a fixed pivot order, already used in `dynamics/linear.rs`.
- SUNDIALS/diffsol: reinitialization (`IDAReInit`, `SensReInit`), which R4 owns.
- num-dual: `implicit_derivative*`, for implicit-provider derivatives without retained factors.

**Need.**

- *Functional:* fitting with many observations and long trajectories with sensitivities (premises).
- *Imposed:* the per-call worker in transient fits is **not required by any rule I found**. "Only attempts own native state" (`rt/workflow/dynamics.rs`) permits retention within the fit's single attempt, as steady experiments already do **[I]**.
- The native session per independent point follows the session-retention policy (§17.6: fan-out successors get independent sessions). That policy is scientific and determinism-driven: no hidden warm start.

**Consequence.**

- *Removes:* evaluator-worker reconstruction, and (with IDAS/diffsol reinitialization) integrator memory and symbolic-factor setup per oracle call × experiment.
- *Weakens:* nothing scientific, if the worker's caches are reset per trial. DP-19 attempt ownership is preserved because the fit is one attempt **[I]**.

### K7. Held preparation scope

This covers W-A (a) (`HeldPreparationScope` exists, but the Python package path sets `held_preparation: None`) and B1a.5–6 (study lanes hold it, renewing `SOURCE_LEASE` every 10 s).

**Own: not a library obligation.**

- **[I]** Under the K1/K2 mapping, the held scope reduces to "the lane owns its Salsa database and products". Protection renewal then remains only for store premises that are actually consumed: retention of stored seeds or of published dependencies.

**Need.** It is imposed by §14.3 protection over off-transaction preparation. The functional need for Python interactive use is reuse across calls, which K1 provides in-process without leases.

### K8. Structure analysis and per-block execution

This covers:

- B5.1: a dedicated `pse-matching` thread;
- W-B (b): "BTF blocks, SCCs, multiple-shooting windows, fit experiments: none found" for parallelism;
- B5.3: KLU analysis refreshed per sweep;
- A4.9: diagnostics re-run preparation.

**Own: largely the libraries, as one pattern-lifetime product** (Part A A3).

- pounce-presolve matching, Dulmage–Mendelsohn and BTF (or `klu_l_analyze` block data);
- rustworkx `dag_algo::layers` for independent-block wavefronts;
- `longest_path`;
- colouring;
- faer and KLU symbolic factors.

These can be computed once per pattern and memoized as Salsa queries keyed by bound structure.

**Need.**

- *Functional:* "many small BTF blocks and some large SCCs" and structure-level parallelism, which the maintainer's requirements put in scope.
- *Imposed:* none found.
- Today's duplicate analyses deliberately cross-check each other (`incidence.rs`).

**Consequence.**

- *Removes:* re-analysis in callback oracles and diagnostics, and pattern refreshes where the pattern is unchanged.
- *Weakens:* the independent cross-validation of structure products, unless it is kept as a test oracle.

### Out of scope here

Validation re-runs (VAL in A1.6, A4.3, W-A (c)), result re-encoding (A4.1), leases, claims and heartbeats, and capacity semaphores belong to R1, R3 and R5. Salsa could memoize a validated batch, but the validation semantics belong to R3.

---

## 2. Refined allocations

**A1′, the Salsa-centred single authority with in-process authored inputs** (refines Part A A1).

One live Salsa database per lineage and session is the reuse and invalidation authority for every pure product. It covers declaration fields, selection, basis (with demand split off), bodies, compiled programs, pattern products and views. Program bytes go to a weighted cache under Salsa identities. The store is the durable sink and the cross-process authority. It is read at session open and written at explicit publication, and it is not re-qualified per solve.

- *Supported by:*
  - A1.8, A1.9, A2.3 and A3.2: per-solve and per-edit store re-reads that scale with package size;
  - A3.8: Salsa early cutoff already works;
  - W-A (a): "scientific kernels are in-process already … none of them reads the store except through the selection/protection wrapper";
  - B (c): seven reuse families beside Salsa.
- *Undermined by:*
  - §14.4-basis, ADR-0164-box and §19/§21 ephemeral-still-canonical; these are rules, not workload;
  - multi-runtime sharing of one lineage (ADR-0181) needs an explicit re-sync point;
  - edits blocking on running queries (Salsa's single writer);
  - write-behind durability, which changes interruption semantics before publication (premise: "interruption before publication").

**A2′, portable compiled programs as the cross-process reuse unit** (refines Part A A2).

Compiled programs, as interpreter bytecode or JIT IR, are serialized once and keyed by body identity plus a versioned ABI. Workers and later sessions load them instead of recompiling.

- *Supported by:* B1b.4/8 (each worker recompiles), A1.10 (descriptions are already published but unused under abi3), and the 16-case premise.
- *Undermined by:*
  - §14.4-rotation and §14.3-eff;
  - the `bincode`/`serde` and `native_code_generation` features are off;
  - symjit defaults to FMA and reads `symjit.toml`;
  - trust and verification of serialized programs is unowned.

**A3′, the pattern-lifetime structure plane** (unchanged from Part A A3; it composes into A1′ as tracked queries).

- *Supported by:* B5.1 and B5.3, W-B (b) (no structure-level parallelism, while `layers` and BTF block data are available), and A4.9.
- *Undermined by:* none found in the ledgers.

---

## 3. Repeated work the libraries would remove by construction

The following would go away without adding machinery:

1. **Re-reading the root declaration on every solve** (A1.8, A2.3). It becomes a tracked input read in A1′.
2. **Rechecking the selected admission against the store on every preparation** (A1.9, A2.3). Backdated memos in a single-writer session remove it. A rule must change first (K2).
3. **Hydrating the full parent on a structural edit** (A3.2). With per-field setters, cost grows with edit size.
4. **Re-specializing for a new overlay path set** (A2.3). Field-level tracking re-runs only the demand projection **[I: if demand only adds addressability]**.
5. **Recompiling per worker and per lane** (B1b.4/8). Serialized programs (A2′) remove it; Salsa cannot.
6. **Rebuilding the transient-fit evaluator worker and integrator per oracle call** (B3.4b). The fix is to retain the worker per experiment, as the steady path does, plus solver reinitialization (R4). This is retention within the existing attempt, not new machinery.
7. **Keeping two reuse authorities for pure products** (B (c): Salsa lanes plus the DataFusion `DefaultCache` families). They collapse to one dependency authority, with a byte-bounded store underneath.
8. **Re-deriving structure per consumer** (B5.1, A4.9). One memoized pattern product replaces it (A3′).

## 4. Open premises

| Premise | Changes | Settled by |
|---|---|---|
| Can §14.4-basis, ADR-0164-box and §19/§21 change, so that the store is a sink and cross-process authority rather than a per-solve qualifier? | A1′ as a whole | Maintainer decision. ADR-0164 is still *proposed*. |
| Does any product consumer need a second writer to one lineage while a session is live? | Whether in-process inputs need a re-sync protocol | W-B (a) product-actor list: ADR-0181 multi-runtime separation is per *caller*, and the ledger does not show two writers on one lineage. Needs confirmation from R1. |
| Is Symbolica optimization deterministic under `cores > 1`, and are serialized evaluators bit-identical after load? | Whether programs can live in Salsa (fresh-equality oracle) and be portable (A2′) | Pinned-source reading of `evaluate/tree.rs` parallel Horner, then a fixture (outside this brief) |
| Does `Bindings.demand` only extend addressability? | Item 4 of §3 | Trace `pse-modeling/src/specialize.rs` demand consumers |
| Does symjit honour an option that turns FASTMATH off, and can the `symjit.toml` lookup be isolated? | A2′ under the no-FMA invariant | Pinned symjit `config.rs` `set_option` keys |
| Does retaining a transient-fit worker within an attempt conflict with a stated contract? | Item 6 of §3 | `integrated.rs` and `dynamics.rs` contracts; §17.6 |
| Do `selected_flights`, `basis_flights` and `modeling_cache` dedupe package reconstruction across worker lanes? | The size of K5 inside one worker | W-B leaves this Interface-checked only; trace the hit path |
| How large are interruption semantics under write-behind publication? | A1′ durability | R1's durable-state mapping |

**Files written:** this file only. No network requests were made for Part B.
