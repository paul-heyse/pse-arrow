# Packed evaluator domains and batch adapters

Reusable type: `symbolica/packed-evaluator-boundaries`. Research date 2026-11-09.
**Interface-checked:** inspected Symbolica 3.0.1 source variant and Numerica 3.0.1 source.
No compile probe or runtime experiment. Claims below are scoped to those source implementations;
API exposure depends on features and numeric domains. Contextual source identity is retained in
the linked account's coverage, rather than inferred solely from a version number.

## Domain and backend eligibility are separate

Locked Numerica implements packed Real/FloatLike for f64x2/x4/x8. Maintained Symbolica
`EvaluationDomain` supports **x4 and x8**, not x2 in the inspected domain file. Its JIT and
compiled-number implementations here support **x4**, not an arbitrary domain width.
A valid interpreted coefficient type is therefore not automatically a compiled backend.

Domain resolution prefers a packed external callback, but otherwise the x4/x8 implementation
wraps an f64 callback and invokes it independently for each lane. This preserves access to scalar
function implementations; it does not make the callback itself SIMD arithmetic and may erase
expected speed gains. Registering a native packed callback is a distinct opportunity.

JIT conversion of an f64x4 coefficient explicitly rejects a vector whose lanes are unequal:
`JITCompiledNumber::to_complex_f64` expects a scalar constant replicated across lanes. The
supported numerical model is shared code/constants with packed varying inputs, not arbitrary
lane-dependent coefficient programs. `map_coeff` also remaps external functions and resolves
registered constants for the target type; unresolved or inappropriate constants can panic
through its current unwrap path. Coefficient mapping is not a fallible blanket eligibility check.

Packed interpreter integer Pow dispatches `T::pow`; Numerica SIMD currently implements that
through `powf_simd` with a debug exponent bound. Integer powers, negative bases, subnormals,
signed zero and overflow therefore need their own numerical-profile qualification. They must
not inherit scalar repeated-squaring claims merely from the Pow instruction name.

Interpreter IfElse/Join tests `is_fully_zero` on the whole numeric value, not a lane mask. Do
not use one packed branch program as evidence of lazy independent branch selection for mixed
lanes. An initially provider-free, branch-free region with one demand/layout is the cleanest
investigation case; a later branch-aware route needs explicit semantics rather than an eager
both-branches workaround that can trigger unselected-domain failures.

## Supplier batch adapters have materially different current behavior

The compiled C++ real-SIMD adapter in `backend.rs` checks dimensions through its loaded module,
packs groups of four scalar rows, evaluates packed groups and unpacks using
`output_buffer[(row/4)*n_out+column][row%4]`. It handles multiple outputs and row tails in this
source. Padding uses zero values in inactive tail lanes; a callback can still execute those
lanes, so scalar purity and domain-sensitive effects matter even when padded outputs are discarded.

The real JIT x4 `BatchEvaluator<f64>` adapter uses a different unpack loop:
`out.chunks_mut(4*n_out).zip(&output_buffer)`, copying from one four-lane vector with a slice of
the destination chunk's length. For a full group with two outputs the destination length is eight
but the source has four lanes. Source inspection therefore exposes an out-of-bounds slicing path;
this is **not runtime-tested** and no supplier or product repair was attempted. Zero batch sizes
and zero-parameter layouts also deserve direct verification: this JIT path derives counts using
division by batch_size and chunks by n_params, unlike the more explicit C++ adapter.

Consequently the current scalar-buffer JIT adapter cannot simply be credited with a qualified
multi-output batch capability. Packed JIT `evaluate` and the separately implemented compiled C++
packing path remain distinct candidates. A targeted adapter probe or source correction would be
appropriate in a later selected implementation; there is no need to launch a broad benchmark
campaign to establish this source-level concern.


## Source and interpretation

Inspected library-relative sources: Symbolica `src/evaluate/{domain,evaluator,backend}.rs`;
Numerica `src/domains/float/{simd,native}.rs`. The native backend is feature-gated, and its
methods are not established by a default arithmetic-only API export.
[Versioned source navigation](https://docs.rs/crate/symbolica/3.0.1/source/src/evaluate/).
Observed slicing and branch behavior is source evidence, not a tested failure or a generic
guarantee across releases. Public trait implementation alone does not establish complete
operation coverage, lane-wise control flow or numerical-profile equivalence.
