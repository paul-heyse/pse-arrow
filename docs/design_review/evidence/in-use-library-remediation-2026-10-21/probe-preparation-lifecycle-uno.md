# Probe report: preparation, lifecycle and Uno status (A–D)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Probe report A–D (Symbolica body granularity, Salsa demand and catalog reuse, horizon caller cancellation, Uno status latch). Executed in a throwaway worktree; diff and logs under probe-artifacts/worktree/.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

I ran all four probes in the throwaway worktree. **C** and **D** settle their questions cleanly. **A** and **B** each partly refute an assumption in inv-prep. The worktree and branch are removed and `/home/paul/pse-arrow` is clean.

**Headline results:**
- **Probe A:** one Dualizer over a whole dispatch group is far worse than today at derivative orders: up to 280× more second-order operations, and about 10 s to build. Symbolic derivatives per output, then one joint `evaluator_multiple`, cut operations by about 40–60% at every order.
- **Probe B:** with production default limits, any edit rebuilds the whole compiler workspace for the reference package. That defeats Salsa reuse before the `Catalog` firewall matters. Demanding a new `let` member also changes the dispatch group, not just `paths`.
- **Probe C:** the horizon ignores caller cancellation today. The one-line fix at `driver.rs:142` makes it stop.
- **Probe D:** the Uno latch masking (W2) is not a live defect. W1 (iterate 5 maps to `Infeasible`) is confirmed.

All builds used profile `test` (optimized + debuginfo), the pinned nightly, resource class `functional`, and a private `PSE_TEST_RESOURCE_STATE` in the scratchpad. `select.py` supplied the scoped force-validation. Runs were sequential.

---

## Probe A: body granularity (inv-prep Q1)

**What I built**
- `crates/pse-math/src/library/probe.rs`, function `measure_group`, plus three accessors on `PreparedBody` in `execution.rs`.
- A recording hook in `grouped.rs` at the per-output `BodyInput` site, and `AdmittedBody::probe_math`.
- Test `pr_jacobian_tests.rs::probe_a_body_granularity_pr_flash`. The fixture is `bt_pr_flash_nested` (`01a0eecf…2e96`, a BT_PR flash), prepared with `declared_execution` then `resolve_case(Second)`.
- The test uses a local runtime, because the canonical store profile at `PSE_SURREAL_STATE` refuses with "Missing payload incarnation; explicitly regenerate this profile".

**Command:** `scripts/pse-env --resource-class functional --native -- python3 scripts/select.py --execution-effects native-local --features solver-ipopt,solver-diffsol,solver-kinsol,solver-root-isolation unit pse-runtime 'test(probe_a_)|…' --no-capture`. It passed in 52.6 s; preparation took 9.6 s.

**Variants**
- **today:** `PreparedBody::compile` census per occurrence, `library_operations`, with outputs split by whether a fully substituted expression exists.
- **inlined:** each output as its own Symbolica evaluator over its own coordinates, with a Dualizer.
- **b group:** one `evaluator_multiple` over all substitutable outputs of an instance, union coordinates, Dualizer with per-parameter `zero_components`.
- **c merge:** per-output evaluators over the union parameters, combined with `merge(…, Some(1))`.
- **d symbolic** (an extra variant, Q1 option d): symbolic `derivative` per output over its own coordinates, then one joint `evaluator_multiple`.
- All variants use `Optimization::default()`: 1 core, 10 Horner iterations, 1 CPE round.

**Inventory:** 118 equation outputs in 19 instance groups, 229 admitted bodies, 167 coordinates. 82 outputs are substitutable; 36 are not (provider or branch), including all 16 single-output PR phase instances (`eq_liquid`, `dew_vapor`, …). Shared members, counted by `let`-binding name read by two or more outputs: 0 in every group. The two state blocks each have 10 members.

**`root.outlet`** (50 outputs, 40 substitutable, 17 distinct bodies, 75 union coordinates; `root.inlet` is within ±2 of every number):

| variant | ops V / 1st / 2nd | temps V / 1st / 2nd | Taylor width | build ms V / 1st / 2nd |
|---|---|---|---|---|
| today (substitutable)¹ | 145 / 339 / 901 | 178 / 380 / 486 | 1 / 9 / 24 | 1.4 / 1.9 / 2.7 |
| today (the 10 non-substitutable) | 5263 / 21977 / 77840 | — | 1 / 22 / 60 | 42.6 / 84 / 163 |
| inlined | 108 / 433 / 2165 | 80 / 186 / 407 | 1 / 7 / 28 | 0.1 / 0.2 / 0.6 |
| b group | 85 / 3078 / 252379 | 41 / 366 / 14723 | 1 / 76 / 2926 | 0.6 / 5.3 / 10060 |
| c merge | identical to b | identical to b | identical to b | 0.7 / 5.3 / 9834 |
| d symbolic | 85 / 142 / 371 | 41 / 57 / 100 | 40 / 124 / 262 outputs | 0.7 / 1.7 / 5.3 |

¹ For "today", the temps column is the census `frame_entries`, not Symbolica temporaries.

**Totals over 19 groups:** today (substitutable) 303 / 705 / 1843; inlined 223 / 901 / 4449; b group 177 / 6206 / 505003, with 19.2 s order-2 build; d symbolic 177 / 292 / 750.

**Interpretation, limited to this data** (*Measured*, this fixture and these settings):
- At value order, one group evaluator saves about 41% against today and about 21% against per-output inlining. That saving comes from cross-output CSE, not from shared members, because no member is read twice.
- C1's assumption (iii), that Taylor work should follow actual dependence through `zero_components`, is refuted here. Group Taylor work follows the 75-coordinate union: 9× today at first order and 280× at second.
- `merge` reproduces the joint evaluator's counts exactly, so it adds nothing over (b).
- (d) keeps both CSE and per-output sparsity, about 58–59% below today at first and second order.
- The non-substitutable outputs dominate today's cost (13854 / 57250 / 201392 total). Grouping cannot reach them, and most of the PR EoS sits in them.

**Caveats**
- Nested implicit-system bodies are outside this hook.
- (d) computes the full upper Hessian triangle, while today computes only consumed pairs, so (d) is if anything overcounted.
- Op counts only; nothing was evaluated, so there are no runtime numbers.

## Probe B: demand and catalog (inv-prep Q3, Q4)

**What I built:** `reuse_tests.rs::probe_b1_demand_diff_on_reference_flowsheet` and `::probe_b2_unreached_edit_reexecutes_projection`. Both use the reference `ThreeUnits` composition (mixer, CSTR, separator) on the steady route. No native libraries.

**Command:** `scripts/pse-env --resource-class functional -- python3 scripts/select.py unit pse-compiler probe_b --no-capture`, then `probe_b2` alone after refinements. Both passed (*Tested*).

**B1, `demand=[]` against `demand=[mixer[0{s}].in_total[first], mixer[0{s}].inlet_pressure[first]]`:**
- Five single demands of paths that are already symbols: only `paths` differs.
- Demanding the otherwise-undemanded `let` member `in_total`:
  - `symbols` gains one `Let` symbol (demand chain length 2).
  - `instances[ThreeUnits.mixer]` changes its group hash, and coordinates go from 42 to 44.
  - The mixer's `groups` key is replaced; its `DispatchBody` coordinates go from 42 to 44 and expressions from 20 to 21 (equations stay 7).
  - The private `source_occurrences` grows from 183 to 185 entries.
  - `equations`, `closures`, `ports`, `functions`, `annotations` and every other field are equal.

So Q3's expectation is only partly right. Demand of an existing symbol is pure bookkeeping. Demand of a new member changes the dispatch-group identity and body, so taking demand out of `Request` is not enough by itself.

**B2, one edit to a declaration the root does not reach:**

| variant | edit publish re-executes | re-prepare re-executes | generation | bodies reused by pointer |
|---|---|---|---|---|
| default limits (`query_values` 64; retained 16384 entries / 256 MiB) | every granular query (`parsed` and `typed` 2743 each, …) | `selected`, `specialized`, `projection`, 143 `semantic_body` | 1 → 2 | 0 / 143 |
| `query_values` 2²⁰, default retention | same | same | 1 → 2 | 0 / 143 |
| `query_values` 2²⁰ and retention unbounded | one of each granular query | `selected` and `projection` only; `specialized` backdates | stays 0 | 143 / 143 |

- The same results hold whether the unreached declaration sits in the root's package or in a separate one.
- `select(root)` is equal across the edit, and the final model and admitted modeling are equal in every variant.
- Retained usage for this package is about 54.5k entries and about 606 MB after publish, which is over the default retention limits. That is what forces the rebuild: generation 0 → 1 already happens at first prepare.
- A second prepare with no edit re-runs 79 `semantic_body` queries when `query_values` is 64 (LRU eviction, 143 > 64), and nothing when it is large.
- Prepare has about 60–155 ms of fixed cost with zero query executions.

**Interpretation** (*Measured*): C5's mechanism is confirmed once the retention limits are lifted (`projection` re-runs because it reads `Catalog` directly, and `specialized` backdates). Under production defaults, though, retention-driven rebuilds and LRU eviction defeat reuse first.

## Probe C: horizon caller cancellation (inv-runtime P1)

**What I built:** `horizon_tests.rs::probe_c_caller_cancel_after_admission_stops_horizon`. It is the antiwindup NMPC loop, 20 steps, started with `start_horizon(h, &caller)`. The test waits for the first progress events and then calls `caller.cancel()`. Two existing controls ran with it: `plan38b_replay_horizon…` and `horizon_proposal_queue_preserves…`.

**Command:** same `select.py` invocation as probe A, filter `test(probe_c_)` (*Tested*).

| | before fix (`driver.rs:142` `CancelSource::new()`) | after fix (`cancel.child_with_deadline(None)`) |
|---|---|---|
| events at cancel | 8 (cancelled at 82 ms) | 8 (cancelled at 39 ms) |
| result | `report` Ok, 20 of 20 steps, usable; test FAILED | `report` Err "math work cancelled", 1 step completed, not usable; test PASSED |
| join after cancel | +218 ms (ran to completion) | +0.5 ms |
| admission (pool bytes, CPU permits, job permits) | baseline (10.7M, 2, 4); during (87M, 1, 3); drained (153M, 2, 4) | baseline (10.7M, 2, 4); during (87M, 2, 3); drained (28.8M, 2, 4) |

**Interpretation:** today caller cancellation never reaches the running horizon; the fix makes it stop promptly, and both existing horizon controls still pass. CPU and job permits are back to baseline after the join in both runs.

**Caveats**
- Only the state after `wait()` is observed. This does not show that release happens only after the native join; that would need a gated native stage, as in `caller_lifetime_tests`.
- Pool bytes do not return to baseline in either run (retained artifacts).

## Probe D: Uno bridge latch (inv-native W1/W2)

**What I built**
- I lifted the post-native latch block at `uno.rs:597-604` unchanged into a helper, `probe_latch`, and production now calls it.
- Test `uno.rs::tests::probe_d_bridge_latch_and_native_status_mapping` builds a `SolveReport` from `termination(opt, iter)`, a fresh `CallbackState`, then runs `probe_latch(status)` and `CallbackState::finish`.

**Command:** `scripts/pse-env --resource-class functional --native=solver,uno -- python3 scripts/select.py --execution-effects native-local --features uno,ipopt unit pse-backend-native probe_d_bridge_latch --no-capture`. It passed (*Tested*, synthetic inputs).

**Native mapping:** `(0,1)` Success, `(0,2)` FeasibleOnly, `(0,3)` Infeasible, **`(0,5)` Infeasible (W1 confirmed)**, `(0,6)` Numerical, `(0,7)` Unbounded, `(1,·)` IterationLimit, `(2,·)` TimeLimit, `(3,·)` Evaluation, `(4,·)` Numerical, `(5,·)` Cancelled.

**Final category with the latch**, the same for every native pair tested: `(0,1)`, `(0,3)`, `(0,5)`, `(2,1)`, `(3,1)`, `(4,1)`, `(5,1)`:

| bridge status | no prior callback terminal | prior callback terminal (Evaluation) |
|---|---|---|
| OK | native category | Evaluation |
| CALLBACK_TERMINAL | Evaluation | Evaluation |
| CANCELLED | Cancelled | Evaluation |
| DEADLINE | TimeLimit | Evaluation |
| ABANDONED | Limit | Evaluation |
| MEMORY | ResourceExhausted | Evaluation |

When no native result exists, `bridge_termination` gives the same column as "no prior callback terminal".

**Interpretation:** W2 is not live. When `status ≠ OK` and no callback cause is latched, the latch block sets `state.terminal` from the bridge status, and `finish` (`callback.rs:695+`) overwrites the native category. A callback's own cause takes precedence. W1 stands.

**Caveat:** inputs are synthetic; I did not run a real Uno deadline. The existing `actual_operator_storage_refusal…` test already shows MEMORY → ResourceExhausted end to end.

## Environment issues to know about

These are not caused by the probes:
- The shared test-resource registry refuses with "requires explicit fenced migration recovery".
- The shared canonical store profile refuses with "Missing payload incarnation".
- Building `pse-backend-native` with `--features uno` alone gives a test binary that cannot load (`libmkl_intel_lp64.so.3: undefined symbol: mkl_blas_dgemm`); `uno,ipopt` works.
- The main checkout's `RUSTC_WRAPPER` inherited into the worktree is refused, so I unset it per command.
- A first full-native run in the worktree built several managed native generations into `~/.cache/pse-arrow/native`; they stay cached.

## Cleanup

The worktree `/home/paul/pse-arrow-wt/review-probes` is removed with `--force`, along with its `target`. Branch `review-probes` is deleted. `git -C /home/paul/pse-arrow status` is clean at `0530dcaa1`, and only the main worktree remains. Nothing was committed or pushed.

The full probe diff, the untracked probe module and all logs are in `/tmp/claude-1000/-home-paul-pse-arrow/15c6fe71-a184-4329-a417-2fc2e56d8c52/scratchpad/`:
- `probes.diff`
- `probe_math_probe.rs`
- `probe_a.log`
- `probe_b.final.log`
- `probe_b.run1.log`
- `probe_b/b1.log`
- `probe_ca.run1.log` (C before the fix, with A)
- `probe_ca.final.log` (C after the fix, A rerun, controls)
- `probe_d.final.log`