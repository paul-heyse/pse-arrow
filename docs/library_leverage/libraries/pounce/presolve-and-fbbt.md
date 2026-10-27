# POUNCE presolve (structural decomposition)

Realizations considered: installed `pounce-presolve` 0.12.0 at fork revision
`7f5e2e9d78555bd53f780c1af186c8fd22c922f5` (`paul-heyse/pounce`, commit date 2026-10-18).
Upstream `jkitchin/pounce` has released v0.13.0 (2026-10-22). Neither shows any change to the
matching, Dulmage–Mendelsohn (DM) or block-triangular form (BTF) sources after 2026-05-27:
- the path history for `crates/pounce-presolve/src/matching.rs` ends with `21f38fe7d`, 2026-05-27,
  on both repositories;
- the `v0.12.0...v0.13.0` compare file list shows no `matching.rs`, `dulmage_mendelsohn.rs` or
  `btf.rs`. That list can be truncated.

Research date 2026-10-27. Sources:
- `~/.cargo/git/checkouts/pounce-8316ac8e070882e5/7f5e2e9/crates/pounce-presolve/src/{matching.rs,
  dulmage_mendelsohn.rs, components.rs, btf.rs, incidence.rs}`;
- `crates/pounce-nlp/src/expression_provider.rs`;
- `gh api repos/{jkitchin,paul-heyse}/pounce/commits?path=…`.

## Types

### pounce-presolve/structural-decomposition
- **Need it serves:** maximum matching, the coarse DM partition and the BTF of an
  equality-row × variable incidence.
- **Recognizable code shapes:** building an `EqualityIncidence` (`n_vars`, `eq_row_inner_idx`,
  `adj_ptr`, `vars`), then calling `hopcroft_karp`, `DulmageMendelsohnPartition::from_matching`,
  `SquareComponents::of_square_part` and `BlockTriangularForm::of_component`.
- **Mechanisms and APIs:**
  - `EqualityIncidence::from_probe(&ProbeView)`, or direct construction (the fields are public).
  - `matching::hopcroft_karp(&inc) -> BipartiteMatching { row_to_var, var_to_row, size }`.
  - `DulmageMendelsohnPartition { row_part, col_part, over/square/under_{rows,cols} }`.
  - `SquareComponents`.
  - `BlockTriangularForm { blocks: Vec<BlockTriangularBlock { eq_rows, cols }> }`, with the blocks
    in elimination order.
- **Obligations discharged:**
  - the O(E·√V) matching;
  - alternating-BFS DM coarse parts;
  - connected square components;
  - Tarjan SCC over the matched dependency graph. It is iterative, with an explicit
    `call_stack` in `btf.rs`.
- **Residual caller obligations:**
  - Mapping domain identities to dense indices, and applying any domain tie-break order.
  - Inter-block DAG edges and transitive dependency closure. The BTF result carries only the
    ordered blocks; it has no block-dependency graph.
  - Resource limits and cancellation: there are no hooks.
  - Stack safety for the matching.
- **Semantic conditions:**
  - `hopcroft_karp`'s `dfs` is *recursive*, with depth up to the augmenting-path length
    (O(rows)). Large or chain-like incidences need a large stack.
  - DM uses BFS (iterative) and BTF uses an iterative Tarjan.
  - Block rows and columns are sorted ascending "for determinism". The block order follows the
    traversal over dense indices, so determinism is relative to the caller's index assignment.
  - Sequential and allocation-only. There is no failure type except panics on invariant breaks
    (`debug_assert`).
- **Version / build conditions:** the same at the installed fork revision and at upstream v0.13.0.
- **Limits and negative results:**
  - no iterative or work-limited matcher;
  - no decomposition object that carries closure or block edges;
  - no cancellation.
- **Evidence:** Interface-checked (source).

### pounce-nlp/fbbt-tape (operator vocabulary)
- **Need it serves:** an expression tape for interval forward and reverse propagation
  (feasibility-based bound tightening, FBBT).
- **Recognizable code shapes:** lowering an expression DAG into `Vec<FbbtOp>` with
  operand-before-use indices.
- **Mechanisms and APIs:**
  - `FbbtTape { ops }`.
  - `FbbtOp::{Const(f64), Var, Add, Sub, Mul, Div, PowInt(u32), Neg, Sqrt, Exp, Ln, Abs, Sin, Cos,
    Opaque}`, all binary or unary.
  - `first_invalid_slot()`, `operand_indices()`.
  - The `ExpressionProvider` trait.
- **Obligations discharged:** sound forward and reverse interval rules for the listed operators.
  `Opaque` degrades to `ENTIRE`.
- **Residual caller obligations:**
  - translating n-ary sums and products;
  - negative and real powers;
  - external calls;
  - outward-rounded constants (`Const` is `f64`).
- **Semantic conditions:** a fixed, closed operator set declared by the library, so it is the
  natural single declaration of the FBBT vocabulary.
- **Version / build conditions:** installed revision.
- **Limits and negative results:** no generic expression import (for example from a Symbolica
  instruction stream).
- **Evidence:** Interface-checked.

## Current applicability

### [CBO-023](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-023) — Structural decomposition and closure owned in several places
- **Types:** pounce-presolve/structural-decomposition.
- **Realization:** installed fork revision. Upstream v0.13.0 adds nothing here.
- **Conclusion:** ruled out as a source of the missing pieces (an iterative matcher and closure).
  It fits as the one shared decomposition kernel if the local guard (thread stack, row limit,
  cancellation between stages) is owned once and reused by every call site.
- **What it would discharge / what remains local:**
  - *Discharged:* the algorithms themselves, which are already used at both inspected sites.
  - *Remains local:*
    - the recursion guard;
    - the row and resource limits;
    - SemanticId ordering;
    - inter-block parameter propagation (the `BTreeSet` loop in implicit reconstruction).
- **Prerequisites, configuration or extension:** either an iterative `dfs` in `matching.rs` or a
  work-limited matcher from SuiteSparse. That would be an upstream or fork change, or see
  `suitesparse.md`.
- **Semantic gaps against the CBO's preservation constraints:**
  - Determinism depends on the caller's dense index order.
  - No cancellation between augmentations.
- **Unsettled premises and what would settle them:**
  - Whether implicit reconstruction already runs on a qualified stack. Runtime math policy
    requires `stack_bytes >= MATCHING_STACK` (`pse-runtime/src/math.rs`), so the recursion half
    of the CBO may already be covered there, while the 100 000-row cap is not applied. Tracing
    the thread that calls `prepare_contract_inner` would settle this.
- **Evidence:** Interface-checked; Proposed.

### [CBO-024](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-024) — FBBT operator set declared once
- **Types:** pounce-nlp/fbbt-tape.
- **Realization:** installed fork revision.
- **Conclusion:** fits with conditions. The library already declares the FBBT vocabulary, so the
  local lowering can treat `FbbtOp` as the authority for that consumer. Other consumers'
  vocabularies (curvature, class evidence) remain separate.
- **What it would discharge / what remains local:**
  - *Discharged:* operator semantics for FBBT.
  - *Remains local:* the mapping from source operators to `FbbtOp`, and the `Opaque` policy.
- **Prerequisites, configuration or extension:** none.
- **Semantic gaps against the CBO's preservation constraints:** `f64` constants (exact
  rationals must be outward-rounded locally).
- **Unsettled premises and what would settle them:** none material.
- **Evidence:** Interface-checked.
