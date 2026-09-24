---
title: M09–M10 structural analysis and incremental compiler execution
status: done
date: 2026-09-24
plan: 14-library-owned-process-simulator
adrs: [ADR-0082, ADR-0083]
phase: 1
---

# M09–M10 execution

The approved target uses Salsa for the semantic compiler because dependency tracking,
absence dependencies and equal-result backdating fit this boundary. Native evaluator
construction, mutable provider workers and cache population remain runtime effects.
Plan 13 scope is not inherited.

**Current-state pointer:** this packet's completion receipts are historical. M10–M14
has since implemented the native solver lifecycle; see the
[current packet](14-m10-m14-execution.md) and [inventory](14-execution-inventory.md).
M15 is the next package; full qualification remains M22.

## Dependency order and acceptance

1. **M09.1:** Extract immutable `CasePlan` from numeric assembly. Preserve physical
   admission, aliases, all-branch support and scatter ordering. Coefficient extraction
   consumes the plan and only fixed/parameter assumptions. Graph-only preparation
   constructs zero numeric evaluators.
2. **M09.2–3:** Adapt pounce-presolve 0.12.0 equality incidence, Hopcroft–Karp,
   Dulmage–Mendelsohn, square components, BTF and structural coupling. Validate complete
   selected-case inventories and every library boundary. Keep isolates and contribution
   provenance. Exact finite equal bounds define equality; no sampled Jacobian values.
   A square equality block does not establish independent optimization structure.
   Bound recursive matching to 100,000 rows on a 32 MiB stack. Test exhaustive tiny
   incidence, alternate matchings, invalid inputs, partial scope and long chains.
3. **M10.1–2:** Private Salsa workspace with atomically published admitted input batches,
   tracked presence/absence lookups, actual physical declarations and immutable provider
   descriptors. Queries produce typed bodies, case plans, structural analysis,
   coefficients and compiler-owned artifact requests. No factories, mutable evaluators,
   Salsa IDs or session identities escape as semantic authority. Test clean/reused
   equivalence, unrelated changes, missing/empty membership, whitespace and backdating.
4. **M10.3:** Artifact identity includes semantic body, ordered outputs/coordinates,
   derivative order, numerical interpretation, optimization and evaluator limits,
   source/build identity and actual feature contract. Delete `BodyStore` and its
   caller-supplied physical-hash admission route; migrate all consumers.
5. **M10.4:** Shared runtime artifact retention uses DataFusion `DefaultCache`, registered
   with `NativeCacheService`. Typed single flights remain owned through native exit;
   one waiter cancelling cannot cancel another. Last-waiter departure requests cancel.
   Epoch invalidation prevents late publication; overflow is bounded; errors are not
   negative cached. Allocation leases follow retained readers after eviction.
6. **M10.5–6:** Finite shared-pool allowances for query generations, compilation, foreign
   memory, thread stacks, artifacts and workers. Salsa LRU maintenance happens between
   leases; generation reconstruction bounds metadata. Shared CPU admission reserves
   all requested Symbolica cores once. Worker-local construction/destruction and join
   precede lease release, including caller cancellation. Test cancellation races,
   retained owners, namespace bounds and multi-workspace contention.
7. Delete replaced routes, run targeted force-validation unit packages and `just check`,
   then update plans, inventory, foundation contract and current-state documents.
   M22 retains full integration, solver, Python, performance and release qualification.

## Verification

**Tested, failure baseline zero:** pinned Rust 1.98.1, Nextest default profile and
explicit `pse-relations/force-validate`, with the locally provisioned Symbolica license:

```sh
source .envrc.local
just unit-package pse-math 'package(pse-math) | package(pse-kernels) | package(pse-backend-native) | package(pse-compiler) | test(incidence::tests::) | test(cache_service::flight::tests::) | test(math::tests::)' -p pse-kernels -p pse-backend-native -p pse-compiler -p pse-structural -p pse-engine -p pse-runtime
```

Result: **63 passed, 0 failed; 203 tests outside the reviewed selection**. This retains
M06–M08 numerical/provider controls and adds semantic reuse, structural, cache and
worker-lifetime tests. The tiny independent oracle covers 74,963 bipartite graphs and
259,537 maximum matching witnesses. The 100,000-row augmenting-chain unit passes on
the 32 MiB stack with the workspace's optimized dev dependency profile. The isolated
exact-version pounce-presolve release-backed probe also passes that chain on 32 MiB;
this is not a full release-profile or native-solver qualification.

**Interface-checked:** `just check` compiles all workspace targets. Production source
search finds no `BodyStore`, `compiled_cache` or `prepare_symbol` route. Cargo retains
the existing transitive `proc-macro-error2` future-compatibility notice; M22 owns full
policy/static qualification. No integration, solver, Python, performance campaign or
release distribution is claimed by these units.

## Outcome

**Implemented:** M09–M10 and their replacement deletions are complete. `CasePlan`
separates graph/coefficients/support from evaluator construction. pounce-presolve owns
matching/DM/components/BTF and coupling; rustworkx supplies deterministic topological
ordering. Salsa owns typed semantic dependencies, negative lookups, stable products,
coefficient assumptions and compiler-issued artifact requests. One runtime component
uses DataFusion retention, typed completion-owned flights, shared CPU/pool admission,
epoch fences and allocation owners retained through native exit and escaped clones.

**Mistakes corrected:** the original completion guard requested cancellation after a
successful job, causing two artifact tests to fail. It now disarms only after completion.
A cancelled Salsa token was lost during generation rotation; replacement storage now
inherits pending cancellation. Registry changes also re-admit unit gathers rather than
retaining numeric conversion coefficients from an earlier registry. Retained artifacts
release unused numeric build capacity after compilation while keeping their explicit
foreign allocation allowance.
Prepared bodies retain their preparation allowance even when a worker returns a body
handle that outlives the case; allocation ownership does not affect semantic equality.

**Deliberate choices:** preparation is serialized within each workspace; independent
workspaces share deployment admission. No database clones escape, so there are no
retiring Salsa snapshot generations. Prepared results have separate pool allowances.
Whole-registry physical identity is deliberately conservative; unrelated definitions
and providers still backdate. Foreign allocator/TLS memory remains an admission estimate,
not an enforced process RSS ceiling. Unknown pinned/foreign extents are reported as
unknown. JIT, SIMD, persistence, additional graph consumers and solver execution remain
outside this package. Existing package/rule graph projections retain their real consumers.

The [scoped review](../design_review/reviews/design_review_incremental-math-structure_2026-09-24.md)
records gate results and limitations. ADR-0082/0083 remain proposed; no accepted ADR or
blueprint was amended. The later [M10–M14 packet](14-m10-m14-execution.md) supplies
native solver execution; M15 is next, with public workflows still assigned to M16.
M19–M21 complete cleanup/acceptance inventory, and M22 performs full qualification.
