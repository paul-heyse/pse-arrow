# Checked binary64 evaluator transfer

The maintained Symbolica **3.0.1** supplier exposes
[`evaluate::f64_transfer`](symbolica/src/evaluate/f64_transfer.rs) through its public
`evaluate` module. It transfers the existing `ExpressionEvaluator<f64>` execution
program, not symbolic preparation or a project-owned arithmetic interpreter. The
[complete archive delta](deltas/readme.md) remains the source-reconstruction owner.
The native delegation contract and activation status belong to Plan65k E03 and
proposed ADR-0246.

`ExpressionEvaluator<f64>::f64_transfer` borrows native builtin instructions and their actual
stack boundaries. `F64ProgramRef::extents` and `encoded_len` allocate nothing;
`write_into` requires an exactly caller-provisioned byte slice. Constants retain their
original binary64 bits, including signed zero and NaN payloads. Parameters and mutable
intermediates are absent from the encoding. Instruction and operand order, repeated
operands, integer exponents, exact stack extent, protected parameter/constant prefix,
result indices and result aliases are preserved. The versioned fixed-width
little-endian profile carries no serialized allocation allowance.

Both the borrowed native profile and inspected byte view expose allocation-free
`operation_count`. They share checked supplier counting rules: Add/Mul count their
original operand population minus one, integer Pow counts the original absolute
exponent minus one and a negative-power inversion, and Powf/builtins count one
function call. Accumulation overflow refuses. This computes original supplier work
without restoring an evaluator or introducing a consumer-side byte interpreter.

`F64ProgramView::inspect` checks framing/version, complete byte exhaustion, checked
integer conversions and requested layouts, populations, every index, protected
destinations, nonempty Add/Mul and the supported instruction profile without allocating.
This is passive structural inspection; definition-before-read validation remains
outstanding. `restore_with` calls the supplied native admission callback exactly once
with those derived extents, before requesting validation scratch or evaluator backing.
A denial retains the caller's original generic error value and source chain. Reservation
failures retain `TryReserveError` separately from program and admission refusals.

After admission, an initially empty definition bitset checks reads before each write
and checks every result. Parameters and constants are initially defined; temporary
destinations may be reused only under the original instruction order. The bitset is
released before final exact-reservation vectors are materialized. Fresh parameters
and intermediate stack slots contain NaN; constants keep their original bits. The
restored evaluator uses its existing instruction implementation. No Atom, coefficient
conversion, symbol registration, normalization, differentiation, hot-start tree,
optimization setting or warmed worker value is restored. Default preparation settings
are inert for this execution profile. Later symbolic preparation, complex evaluation
and vectorization eligibility are not supplied by numerical transfer.

The current wire version is **2**; old versions refuse explicitly. The builtin profile
supports Add, Mul, integer Pow, Powf and the existing unary
EXP/LOG/SIN/COS/SQRT/ABS/CONJ builtins. Builtins use fixed profile discriminants, not
serialized Symbols or tags. The convenience builtin-only entry refuses all external
state, including unused state. All control-flow instructions refuse. The f64 execution loop and its
operation counter ignore `ComplexPhase`; the scalar flags are preserved as metadata.
The supplier emits `PartialReal` only for Add/Mul, where inspection checks its prefix
count against the actual operand population.

`f64_transfer_with_heads` and `inspect_with_heads` additionally accept a trusted local
`F64KnownHead` table. A descriptor supplies a nonzero operation tag/version, fixed arity,
already registered Symbol, fixed ASCII export name and a closed function pointer. The
table is an application execution-profile premise: the caller establishes matching source
and destination semantics. It is never decoded from the program. The encoding carries
only the tag/version and ordered argument indices. Unknown tags/versions, wrong arity,
duplicate descriptors, tagged functions, hidden retained bodies and unqualified implementations
refuse before restoration. Known constants already materialized in the protected f64 prefix
transfer as their exact bits; their generators and fixed symbolic arguments are discarded.

Restoration admits descriptor-vector, fixed name, function-pointer box and private argument
buffer backing together with instructions and the main stack. It then installs only the
trusted destination table's fixed function pointers. It registers no symbol and invokes no
source, tag, derivative or implementation generator. These callbacks hold no producer control
or warmed values. Repeated outputs and source instruction order remain unchanged. The enclosing
mathematical owner still qualifies original guards, physical/source custody, demand and layout;
the table and numerical bytes grant no symbolic or scientific publication authority.

Integer Pow has an additional exact-source restriction: Numerica **3.0.1** native f64
`Real::pow(u64)` debug-asserts `e <= i32::MAX`, then casts to i32. Symbolica handles a
negative integer exponent by taking its unsigned absolute value and applying an
inverse. This profile therefore admits only `|e| <= i32::MAX`; neighboring values and
i64 MIN/MAX refuse during native export and encoded inspection, before admission.
The i64 encoding remains exact for accepted values. There is no Powf substitution or
arithmetic rewrite, no complete i64 power support claim, and no library absence claim.
The separate Powf path retains its existing supplier behavior.

Extents describe requested Rust vector backing, not allocator metadata, actual
allocator usable size, RSS or a whole execution bound. The definition scratch and
materialization phases do not overlap; their maximum is reported. Borrowed input bytes,
the evaluator value itself, outer Arcs/native templates, gather/scatter buffers and
native frames remain the consumer's accounting responsibility. A tiny valid encoding
can request a large stack: successful passive inspection never grants that allocation.
Every native scientific caller retains its own admission, context and permission checks.

**Tested, targeted, 2026-11-10:** `scripts/pse-env -- cargo test --manifest-path
vendor/symbolica/Cargo.toml --locked --no-default-features --features
integer-gmp,float-mpfr,native_code_generation --lib evaluate::f64_transfer::tests::`
passed 14/14 under the pinned toolchain and configured unbounded execution, zero failures
against baseline zero. The new independent 3–4–5/5–12–13 magnitude control covers repeated
outputs after producer retirement, exact backing admission refusal, foreign versions/arity
and definition-before-read. The first attempts exposed four private test-helper call sites
requiring the new profile arguments and one historical corruption test that wrote the now
valid version 2; both were corrected. This supplier result does not establish production
canonical receiving, compiled stable execution or whole-series qualification.

**Tested, targeted:** the pinned-toolchain, configured-unbounded command
`cargo test --manifest-path vendor/symbolica/Cargo.toml --locked --no-default-features
--features integer-gmp,float-mpfr,native_code_generation --lib
evaluate::f64_transfer::tests::` passed 12/12 with zero failures, ignored tests or
warnings. It checks original producer Value/First derivative streams, operand-order
sensitivity, every fixed builtin, exact bits/aliases and cold reset, malformed/truncated/
overlong encodings, integer-power boundaries, definition-before-read, denied growth,
unchanged admission error/source, and distinct failures at each reservation step.
The reservation fault selector is test-only; it returns an actual fallible reservation
error without provoking machine exhaustion.

The subsequent native-template prerequisite adds one overflow control and checks
operation-count equality with `ExpressionEvaluator::count_operations` in every
existing encoding fixture, including original Value/First derivative producers.
The same standalone command passed **13/13**, zero failures or warnings, with the
unchanged standalone Rug 1.29 lock. This is supplier evidence; the consuming native
template controls remain a separate obligation.

The first pure lib-test build encountered six existing tests requiring the native-code
generation feature, plus seven new Symbol constant-pattern errors and two unused
imports. The new errors/imports were corrected and the actual required feature closure
selected without changing or suppressing upstream tests. Its first functional run
passed 9/10; the deliberately executed i64 MIN power exposed the Numerica premise
above. The final 12-control run tests explicit refusal of that unsupported domain.
The standalone supplier lock uses Rug 1.29; the repository lock selects Rug 1.30.
Neither lockfile or source version is changed. **Interface-checked:** the configured
unbounded, cache-off `scripts/pse check pse-math --all-targets` passed against the
repository lock with zero errors, warnings or incomplete records and complete original
capture/settlement. Actual native compiled-stage fixture execution, live-child activation
and scientific qualification remain separate consumer obligations; the supplier probe
and consuming compile are not universal coverage.
