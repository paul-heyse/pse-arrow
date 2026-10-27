# Additional library-leverage patterns

**Assessment output · revision 1, 2026-10-27.** These are additional framings found during
assessment. They sit alongside the [base patterns](patterns.md) (`LOP-01`–`LOP-12`, edition 0.2),
and every stage uses them on the same footing as the base set. A pattern match is only a
recognition aid. It is not evidence of a defect, a library fit or an obligation to adopt.
Promotion into the base framework is a separate manual operator decision. If one is promoted,
it keeps its `LOP-X` identity and records the base ID it maps to.

| Pattern | Recognizable opportunity | Origin |
| --- | --- | --- |
| [LOP-X001](#lop-x001) | Prepaid resource accounting mirrors library internals | First codebase audit, 2026-10-27 |
| [LOP-X002](#lop-x002) | Ambient execution context re-established at every boundary | First codebase audit, 2026-10-27 |
| [LOP-X003](#lop-x003) | Generation pipeline becomes the maintained machinery | First codebase audit, 2026-10-27 |

<a id="lop-x001"></a>

## LOP-X001 — Prepaid resource accounting mirrors library internals

**Articulation.** A codebase must admit memory, work or time before a library operation runs.
The library neither reports a prediction nor accepts a budget, so the caller writes its own cost
model of the library's internals. It then charges that estimate, wraps each call in
charge-and-checkpoint code, or polls a cancellation flag between library steps. The guarantee
is real, but the mechanism duplicates private library behaviour and drifts when the library
changes.

**Recognize.**
- Functions named `*_bound`, `*_extent`, `allocation_bound` or `retained_bytes`.
- Multipliers with no stated derivation (`rows * 64`, `fields * 512 + 2048`).
- Comments that cite a library version ("from `Dualizer::map_instruction` in X 3.0.1",
  "pinned graph crate node layout is eight usize fields").
- `size_of` arithmetic over another crate's types.
- Thread-local abort bindings that work around a missing setter.
- The same estimate written once per library: graph, sparse factor, evaluator, columnar batch.

**Investigate.**
- Does the library expose a symbolic-phase estimate (fill, `nnz(L+U)`, workspace size), a
  budget or limit setting, an abort or progress hook, or a memory-size function?
- Would an upstream change make the mirror unnecessary?
- Can a derive or generated accounting layer produce retained sizes from the type declarations?
- Which part is a true prediction and which is a post-hoc measurement wearing a prediction's
  name?
- Are the estimates conservative in the direction the contract needs?

**Counterconditions.**
- Prepay semantics need a prediction made before allocation. Post-hoc measurement, a counting
  allocator or a library-side limit that fires late does not discharge them.
- A deliberately conservative model can be the correct contract even when it is inexact.
- Library hooks may not exist. An upstream contribution is then a legitimate option, but it
  is not a library capability.

**What the base patterns missed.**
- LOP-03 assumes the library can discharge the obligation; here it cannot report what the
  caller must prepay.
- LOP-08 covers lifecycle coordination, but not cost prediction.
- LOP-02 covers repeated declarations, but not mirrors of a dependency's private layout.
- LOP-10 and LOP-11 each describe part of the version coupling.

**Prompting observations (2026-10-27, static reading).** Three independent subsystem passes
found the same shape:
- Hand-derived result extents and Arrow-sized multipliers in `pse-runtime`.
- Symbolica 3.0.1 cost formulas and a thread-local abort binding in `pse-math`.
- Pinned petgraph node-layout arithmetic in `pse-structural` and `pse-compiler`.
- FERAL fill and KLU workspace formulas in `pse-backend-native`.

See [CBO-001](codebase/cross-cutting-mechanisms.md#cbo-001) and
[CBO-021](codebase/mathematics-and-compilation.md#cbo-021).

<a id="lop-x002"></a>

## LOP-X002 — Ambient execution context re-established at every boundary

**Articulation.** An operation's identity, attribution, telemetry and cancellation scope live in
several ambient carriers: task-locals, thread-locals, a tracing dispatcher, the current span,
a runtime handle and a recorder. Every boundary that moves work to another thread, task or
foreign callback must capture and re-enter each carrier in the right order. The carriers form
one concept, the current operation context, kept in several places. A missed carrier silently
loses attribution on one path.

**Recognize.**
- A block of `X::capture()` / `Y::current()` lines before `spawn`, followed by matching
  `let _guard = x.enter()` lines inside the closure.
- The same block repeated at each spawn site.
- Fallback code that consults a task-local and then a thread-local.
- Wrapper `inherit` helpers per subsystem.
- Bugs or tests about attribution lost on one path.

**Investigate.**
- Which carrier is authoritative, and which exist only because of the spawn mechanism?
- Can the carriers collapse into one propagated handle? Options include a single context value
  with one `enter`, span extensions, an instrumented spawn helper or an executor feature.
- Which boundaries are foreign (C callbacks, OS threads) and cannot inherit automatically?

**Counterconditions.**
- Carriers can serve distinct lifetimes (task, thread or process) or different consumers that
  must survive without each other, such as a recorder that must work without a tracing
  subscriber.
- Collapsing them can couple an observation sink to the execution model.
- A small local helper may be the right answer, with no library involved.

**What the base patterns missed.**
- LOP-02 addresses forms of one declared meaning, not runtime carriers.
- LOP-09 addresses fragmented responsibilities across consumers, not one consumer re-establishing
  context at each move.
- LOP-08 addresses resource lifecycles, not attribution.

**Prompting observation (2026-10-27).** A five-way capture and enter at the native job spawn in
`pse-runtime`, repeated in staged solves, shooting window teams and the Python pull thread. See
[CBO-014](codebase/execution-lifecycle-and-observation.md#cbo-014).

<a id="lop-x003"></a>

## LOP-X003 — Generation pipeline becomes the maintained machinery

**Articulation.** A codebase derives many forms from one authoritative declaration, which is
the remedy that LOP-02 recommends. The generators, however, become the burden:
- Each source/target pair has its own emitter.
- Generated artifacts feed further generators.
- Output formats are chosen around build limits rather than consumer needs.
- Runtime layers translate between the generated forms.

The authority is sound. The cost lives in the translator tower or in data emitted as code.

**Recognize.**
- A generated artifact that is the input of another generator. Examples include
  schema → JSON Schema → a language model emitter, or Rust types → stubs → wrappers.
- Hand-written emitters that map one schema language to another.
- Generated source tens of thousands of lines long whose header explains a compiler-memory
  workaround.
- The same products emitted in several encodings, such as source, JSONL and IPC.
- Runtime code that transcodes between two generated model systems.
- Template placeholders replaced by string substitution.

**Investigate.**
- Could one interchange representation be consumed directly at runtime, such as an embedded
  data blob, a shared schema or a native interchange format?
- Does an established generator or runtime library cover a pair, and does it meet the
  determinism needs?
- Does every target need to exist, or can a consumer use an upstream form?
- Which hops give independent assurance, and which only translate?

**Counterconditions.**
- Generated code gives type checking, reviewable diffs, deterministic regeneration checks and
  zero startup parsing.
- Codegen is not disfavoured in itself, so judge each hop on its merits.
- An independent emitter can intentionally encode a stricter contract than a general tool.
- Each hop can serve a different consumer.

**What the base patterns missed.**
- LOP-02 names generation as a remedy and does not consider generation as the upkeep.
- LOP-06 sees runtime translations but not the generator chain that creates them.
- LOP-07 and LOP-11 touch data-as-code only indirectly.

**Prompting observations (2026-10-27).**
- The registry baked as about 87k lines of Rust plus JSONL plus IPC.
- A custom JSON Schema-to-msgspec emitter beside a separate attrs row emitter.
- Document wire envelopes adjusted by schema surgery.

See [CBO-040](codebase/registry-generation-and-python-boundary.md#cbo-040) to
[CBO-043](codebase/registry-generation-and-python-boundary.md#cbo-043).

## Framings considered and left to the base set

The first audit also weighed the following shapes. Each is adequately framed by existing
patterns, so none was added:
- Polling standing in for an absent change notification. LOP-08 already names polling, and
  LOP-12 covers the missing notification capability.
- A lifecycle manager rebuilding an OS manager's state, such as re-deriving systemd state from
  `/proc` and `systemctl show`. This is LOP-03 with the OS manager as the adopted mechanism,
  plus LOP-08.
- A parser/printer or encoder/decoder inverse pair synchronized only by a round-trip test.
  This is LOP-02. Ask whether a bidirectional description exists, and whether the printer
  deliberately normalizes.
- A documented revisit trigger on a local patch. This is LOP-11. Research the recorded trigger
  first.
