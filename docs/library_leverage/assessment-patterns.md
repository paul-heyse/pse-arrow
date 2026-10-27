# Additional library-leverage patterns

**Assessment output · revision 4, 2026-10-27.** Revision 1 came from the first codebase audit;
revision 2 added `LOP-X004`–`LOP-X010` and refined `LOP-X001`–`LOP-X003` from the first library
applicability assessment; revision 3 adds `LOP-X011`–`LOP-X014` and refines `LOP-X001`–`LOP-X010`
from the first [comparative review](reviews/README.md). Reassessment revision 4 corrects
layout-guard independence, nested projection coverage, decoder allocation and numerical
value-semantics guidance below; it adds no IDs and does not refresh untouched applications.
These are additional framings found during
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
| [LOP-X004](#lop-x004) | A supplier's high-level client is patched although it publishes the lower layer | First library assessment, 2026-10-27 |
| [LOP-X005](#lop-x005) | Completion-owned work meets drop-to-cancel library combinators | First library assessment, 2026-10-27 |
| [LOP-X006](#lop-x006) | A reproducibility guarantee spans a composed library stack | First library assessment, 2026-10-27 |
| [LOP-X007](#lop-x007) | A library derive owns one projection of a multi-projection declaration | First library assessment, 2026-10-27 |
| [LOP-X008](#lop-x008) | A property is proved by enumerating a recyclable namespace instead of holding an owned handle | First library assessment, 2026-10-27 |
| [LOP-X009](#lop-x009) | A local guard compensates for a library's unsafe handling of untrusted input | First library assessment, 2026-10-27 |
| [LOP-X010](#lop-x010) | A harness hand-builds one side of a protocol whose counterparty could change | First library assessment, 2026-10-27 |
| [LOP-X011](#lop-x011) | A publication contract keeps interrupted work products | First comparative review, 2026-10-27 |
| [LOP-X012](#lop-x012) | A retry's classification requires settling an uncertain effect | First comparative review, 2026-10-27 |
| [LOP-X013](#lop-x013) | A library kernel's value semantics differ from the domain's | First comparative review, 2026-10-27 |
| [LOP-X014](#lop-x014) | Library-held process-global retention outlives caller ownership | First comparative review, 2026-10-27 |

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
- Pinned petgraph node-layout arithmetic in `pse-structural` and `pse-compiler`. The library
  assessment corrected this: petgraph storage is sized with `size_of` over its public types,
  and the real hand mirrors are std `BTreeMap` node geometry and per-node workspace multipliers.
- FERAL fill and KLU workspace formulas in `pse-backend-native`.

See [CBO-001](codebase/cross-cutting-mechanisms.md#cbo-001) and
[CBO-021](codebase/mathematics-and-compilation.md#cbo-021).

**Revision 2 refinements (library assessment, 2026-10-27).**
- **Kinds of library size information.** Comparing the mirrors with Arrow, faer, FERAL, KLU,
  SUNDIALS, Salsa and Symbolica showed that "library size information" covers distinct things.
  Name which one a candidate offers:
  - a symbolic-phase prediction (FERAL LDLᵀ work estimates);
  - a conservative bound the library publishes;
  - a checkpoint the library polls during work (budget or abort carriers);
  - a cheap pre-materialization step that fixes the output size exactly (a selection count before
    filtering, or an encoder returning zero-copy buffers whose lengths sum to the encoded size);
  - reuse-time exactness, where a refresh allocates nothing so the first factorization can be
    bounded and later ones charged exactly;
  - post-hoc measurement.

  Only the first four serve a prepay contract directly.
- **Check before relying on a reporter.**
  - Re-express a mirror over the library's public size observables before deleting it.
  - Check whether a reporter is deprecated (SUNDIALS workspace queries were deprecated in 7.3
    and removed in 8.0).
  - Check whether an allocator hook is process-global.
  - Check upstream development branches for an existing budget or abort carrier, and record it
    as unreleased.
- **Interruption semantics.** Ask whether interruption returns an error or a truncated but still
  valid result. A library that silently returns a less-optimized result on abort needs the
  caller's own "observed abort means cancelled" rule.
- **Countercondition.** An RSS or wall-time budget is non-deterministic, so it can only be a
  backstop beside a deterministic prepaid refusal.

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Charge granularity.** Ask whether a charge is sized from the enclosing container (file,
  document, batch) when the library would expose the exact part extent to a caller that owned the
  decode (block spans in a verified footer, slice lengths). A container-sized charge is
  conservative but wastes shared capacity in proportion to the number of parts. The usual remedy
  is a lower-level caller-owned API, not a new estimate.
- **Stale but conservative derivations.** A mirror that over-admits after a library change gives
  no signal. Record each residual bound's derivation and pinned version beside the number, in one
  owner per library quantity, so that a version move revisits it.
- **Reconciled versus unreconciled mirrors.** A prediction checked after the call against the
  built product, through an exact public measure with a typed refusal, turns drift into a loud
  failure; its remaining exposure is the allocation made inside the call. An unreconciled mirror
  drifts silently. Adding reconciliation is often the cheapest improvement. A library-side limit
  that the library checks with its own exact count *before* allocating does discharge prepayment
  when the caller reserves the limit first and releases the excess afterwards. Splitting a grant to
  the measured native extent after allocation is the correct use of post-hoc measurement; it never
  replaces the prepayment.
- **Owner-side remedies when the supplier is already carried locally.** If the project already
  vendors or forks the supplier with admission or accounting integrations, the owner-side remedy
  for a caller mirror is a predictor in the supplier beside the allocating code, with a
  supplier-side test. Countercondition: in a large native delta that is periodically rebased, each
  added predictor grows the rebase burden, and a caller-side formula re-verified on a
  version-coupled checklist can be the better owner.
- **Structure-blind complete bounds are not mirrors.** A dense n² charge for a sparse factor
  mirrors nothing; it is the honest complete bound when no structural bound exists, and its cost
  is refusing valid large sparse work. Investigate structure-derived complete bounds (dense per
  block over a block-triangular decomposition, column counts of AᵀA for partial-pivoting LU) and
  check they stay complete once the supplier's growth and padding policy is included.
- *Prompting observations (Interface-checked):* an IPC decode reserving four times the whole file
  per block; a writer grant derived against an older Arrow writer's body copy; a maintainer-owned
  solver fork that already carries a conservative bound for its symmetric factor while the
  unsymmetric LU bound is still mirrored by the caller. See the
  [comparative review](reviews/result-data-and-prepaid-accounting.md#cbo-001).

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

**Revision 2 refinements (library assessment, 2026-10-27).**
- **Request-message carriers.** Context is also captured into request messages handed to
  long-lived worker threads, not only at spawn. A spawn helper alone does not cover those.
- **Library-held thread-locals.** Count library-specific thread-local bindings as carriers too,
  such as an abort callback slot.
- **Subscriber dependence.** Ask which carriers are mandatory when no subscriber is installed.
  Tracing span extensions exist only when a subscriber enables the span, so they cannot carry
  context that must survive without one.

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Attribution versus authority carriers.** *Attribution carriers* (observation operation,
  recorder, tracing dispatcher and span, telemetry parent) only explain work; losing one loses
  explanation, and propagating one by default is safe. *Authority carriers* (connection or
  continuation activity, admission scopes, an enclosing deadline, an I/O lease chain, construction
  or compute scopes, library abort bindings) grant admission or extend a lifetime; propagating one
  silently changes behaviour, and some are deliberately confined to sanctioned children.
  Investigate which kind each carrier is before collapsing them. The remedy is usually one
  attribution context value, with authority kept explicit.
- **Relationship matters.** Moving work to a *shared producer* must link rather than parent the
  context, or the first waiter is credited with shared work. A useful context value offers named
  derivations (continue, nested child, shared producer) rather than only capture and enter.
- **Recognize** divergent carrier subsets at sibling spawn sites. See the
  [comparative review](reviews/execution-lifecycle-and-observation.md#cbo-014).

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

**Revision 2 refinements (library assessment, 2026-10-27).**
- **Recognize.** Watch for post-generation textual rewriting of emitted code, such as
  `.replace(...)` over already-generated source. It is riskier than placeholder substitution, and
  no template engine addresses it.
- **Investigate: serialization contract.** Do the emitted data types already carry a
  serialization contract? A data-blob route's main cost may be giving them one, including
  deterministic ordering of hash-based members and a self-describing format for dynamic values.
- **Investigate: structure versus data.** Separate the compiler cost of emitted *structure* from
  cheaply embedded *data* (`include_bytes!` / `include_str!`).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Recognize bootstrap coupling:** a generator whose crate depends on the crate that compiles
  the generator's own output, where that output *constructs* hand-written types. A model change
  then breaks the generator's build before it can regenerate. (*Tested* in the
  [registry build-coupling probe](../design_review/evidence/library-leverage-registry-build-coupling-2026-10-27/README.md).)
- **Recognize:** several formats in one generated product may each carry a *different*
  sub-product, split by type capability. Count encodings per sub-product before treating them as
  one product encoded three times.
- **Investigate** runtime-decoded data versus a crate split. Workspace-wide feature unification
  can defeat feature gates.
- **Countercondition:** named literals give compile-time staleness detection that a positional
  binary format loses. A layout guard must derive the expected description from current
  compiled types independently of the committed blob; a constant generated alongside it can
  be stale with it. Shape compatibility, payload integrity and independent semantic equivalence
  are distinct guarantees. See the
  [comparative review](reviews/declarations-registry-and-python-boundary.md#cbo-040).

<a id="lop-x004"></a>

## LOP-X004 — A supplier's high-level client is patched although it publishes the lower layer

**Articulation.** A codebase forks or patches a library's convenience client to add policy the
client does not expose: deadlines, admission, budgets, connection ownership or tracing. The same
supplier publishes the layer underneath as a public crate: a protocol or wire crate, a generated
RPC client, a codec, or a value-conversion crate. That layer accepts a caller-built transport,
so the policy could live on public seams without patching the supplier.

**Recognize.**
- A vendored or patched client whose substantive changes sit between its engine trait and its
  transport, threaded through internal context types.
- A "pinned wire boundary" extracted from the supplier's protocol crate.
- Rebase notes that move with every supplier release.
- Test peers that impersonate the server at the wire level.

**Investigate.**
- Is the lower layer published separately, and is its generated client generic over a transport
  service?
- Which client semantics would have to be re-implemented: result assembly, session state, typed
  decode, error mapping?
- Is the lower layer's stability tied to the server version?
- How large is the patch once whitespace and reflow are excluded? Measure a vendored patch with a
  whitespace-insensitive diff against the reconstructed upstream before citing its size. A
  byte-preserving patch can overstate the burden by an order of magnitude.

**Counterconditions.**
- Re-implementing the client's semantics can cost more than the patch.
- Companion tools built against the high-level client may still need it.
- The lower layer may carry weaker compatibility promises.
- An upstream seam contribution can be better, but it is not a current capability.

**What the base patterns missed.**
- LOP-10 looks sideways, at hooks of the same layer or a different mechanism, rather than down
  the supplier's own stack.
- LOP-11 treats the patch as historical.
- LOP-03 assumes some adopted layer already discharges the obligation.

**Prompting observation (2026-10-27, Interface-checked).** A vendored database SDK delta adds a
bounded request lifecycle beneath the SDK's engine trait, while the supplier's installed protocol
crate exposes a generated gRPC client generic over its transport plus value conversions. See the
[SurrealDB assessment](libraries/surrealdb/README.md) and
[CBO-009](codebase/persistence-and-transport.md#cbo-009).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Read the supplier's own stability statements for each layer.** A wire protocol written for
  several client implementations (normative message comments, a capability handshake, a
  conformance suite) is a better seam than the client's internal engine SPI, even when the SPI
  preserves the high-level API. Crate docs saying "internal", "no SemVer" or "may change in a patch
  release" mark the latter.
- **Check whether the patched behaviour is profile-gated.** Companion tools compiled against the
  patched client may run the unpatched profile; their dependence on the patch is then incidental.
- **Measure how narrow the high-level API usage is** (result shapes, builders, streaming). A
  narrow façade makes re-owning it cheaper than keeping a patch. Pin the supplier's observable
  result rules first: the [direct-client probe](../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)
  needed exactly four of them for parity.
- **Ask whose policy the patch encodes.** If it is the codebase's policy (deadlines, admission,
  replay rules), an upstream seam asks the supplier to own the codebase's policy.
- **Countercondition:** a hybrid implementing the supplier's internal SPI under its high-level
  client shrinks the patch but binds the codebase to a layer the supplier disclaims.
- **Lifecycle tiers split across the patch and the codebase** often mirror each other through
  opaque owner handles; moving the lower tier into the codebase can merge them, a benefit that
  patch-size metrics miss. See the [comparative review](reviews/canonical-store-transport.md#cbo-009).

<a id="lop-x005"></a>

## LOP-X005 — Completion-owned work meets drop-to-cancel library combinators

**Articulation.** The codebase must keep owning work after a waiter stops. A stop is a
*request*, and resources stay held until the work actually exits. Most async library
combinators express cancellation by dropping the future: race, timeout, run-until-cancelled,
cache initializers and single-flight leaders. Several also let ready work win over a ready stop.
Applied naively, they release work early or let a ready result beat a stop.

**Recognize.**
- `select!` arms that signal a stop and then `.await` the same retained future.
- Spawned producers feeding a oneshot or shared future, producers that must survive their last
  waiter, and waits on join handles after cancellation.
- Comments such as "retain ownership through join" or "a ready stop wins".

**Investigate.**
- For each candidate:
  - Does it drop or retain the work on stop?
  - Which side wins under simultaneous readiness?
  - Does the producer run in the caller's task or in an owned task?
- Can the library be applied only to the abandon-safe portion, such as admission waits or
  idempotent requests?

**Counterconditions.**
- Many waits are genuinely abandon-safe; a drop-to-cancel combinator is then correct and simpler.
- Completion ownership can be over-applied where an operation is idempotent.

**What the base patterns missed.** LOP-08 frames lifecycle delegation, and LOP-03's
counterconditions mention differing semantics generically. Neither names this split. Without it,
such opportunities read as "use the library's cancellation combinator or cache", which is wrong
at their core sites.

**Prompting observations (2026-10-27, static reading).**
- A native join wait that signals a stop and then keeps awaiting the thread.
- Completion-owned single flights.
- Library combinators read at source, which drop the losing side or bias toward ready work:
  - tokio-util `run_until_cancelled`;
  - tokio `Timeout`;
  - moka's waiter guard;
  - async_singleflight's leader-dropped path.

See the [tokio assessment](libraries/tokio/README.md),
[CBO-011](codebase/persistence-and-transport.md#cbo-011) and
[CBO-015](codebase/execution-lifecycle-and-observation.md#cbo-015).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Three wait shapes.** Stop-aware waits are (1) *wake-source loops*, where a stop is another
  wake and the loop re-checks its predicate; (2) *abandon-safe races*, where the losing work can be
  dropped but a ready stop must win (`select! { biased; stop … }`); (3) *signal-and-drain*, where
  the work is retained until it exits. Classify each site before choosing a combinator. Only (2)
  can take a library race, and only with stop-first bias.
- **Typed stop reason.** A deadline can join cancellation as a typed stop *reason* (cancelled or
  deadline) without folding it into the token. That keeps outcomes distinct and needs no timer
  task. See the [comparative review](reviews/execution-lifecycle-and-observation.md#cbo-015).

<a id="lop-x006"></a>

## LOP-X006 — A reproducibility guarantee spans a composed library stack

**Articulation.** A codebase promises deterministic or portable results: seed-determined,
platform-independent, or bit-identical on replay. The values are produced by a stack of
libraries:
- an RNG stream;
- a distribution or transform algorithm;
- a transcendental math source;
- linear-algebra kernels with runtime SIMD dispatch;
- parallel reductions;
- version value-stability policies;
- feature unification that selects among these.

Each layer has its own, usually weaker, determinism contract. The effective guarantee is the
weakest one, and it can change with no code change: a feature unified on by an unrelated crate,
or a minor version bump.

**Recognize.**
- "Reproducible", "seeded" or "replay" claims over library-generated values.
- Seeded RNG constructors combined with library distributions.
- Dense factorizations inside replayed computations.
- `powf`, `exp` or `ln` inside algorithms that make discrete choices.
- Comments asserting FMA-free or bit-reproducible arithmetic next to calls into SIMD libraries.
- Several major lines of one RNG family in a lockfile.
- Switches between the standard library's math and a `libm` feature.

**Investigate.**
- Which layer actually fixes each bit, and what does each library promise across patch, minor,
  platform and CPU features?
- Does feature unification select the platform math library?
- Do downstream discrete decisions amplify last-ulp differences?
- Can the guarantee be narrowed instead? For example, record realized values and key identity
  on them rather than regenerating.
- Can it be strengthened cheaply, with scalar kernels, explicit `libm` calls or exact integer
  constructions?

**Counterconditions.**
- If realized values are recorded and identity keys on them, regeneration portability may be
  unnecessary.
- Tolerance-based contracts need no bit identity.
- A same-host guarantee may be the honest contract; do not demand portability no consumer uses.

**What the base patterns missed.**
- LOP-03 treats a library as one unit.
- LOP-06 covers precision across translations, not determinism across a composed computation.
- LOP-11 sees pins, not value-stability policies.

**Prompting observations (2026-10-27).**
- A design-of-experiments library's optimized sampling uses `powf` in its acceptance decisions.
- A dense linear-algebra crate dispatches SIMD at runtime with fused multiply-add.
- A numeric-traits `std` feature overrides `libm` for distribution crates.
- A probe showed the non-transcendental design paths bit-stable across opt-level, CPU flags and
  thread count on one host
  ([evidence](../design_review/evidence/library-leverage-egobox-doe-determinism-2026-10-27/README.md)).

See [CBO-033](codebase/numerical-workflows.md#cbo-033) and
[CBO-034](codebase/numerical-workflows.md#cbo-034).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Split the guarantee by role.** Randomness whose realized values are an externally visible
  input (a design, a sample set) can be recorded and keyed by value. Randomness internal to a
  library algorithm (infill starts, optimizer restarts) is impractical to record and is usually
  keyed by seed plus same-host determinism. One codebase can legitimately use both contracts.
- **Downstream ceiling.** A bit-portable stage feeding a stage that varies by CPU class (SIMD or
  FMA dense kernels, platform-libm statistics) yields results that vary by CPU class. Portable
  regeneration of inputs then buys shared input identity across hosts, not identical results.
- **Recognize** scalar "FMA-free, bit-reproducible" loops whose operands come from
  runtime-dispatched kernels, and admission decisions (PSD refusal, rank or sign tests) taken on
  such kernels' outputs.
- **Identity coverage of local patches.** When replay or receipt identity keys on a linked-build
  identity, check that it covers every layer that fixes bits, including local patches to a
  vendored supplier at an *unchanged* package version. Recognize a default "pinned crate fully
  determines the build" hook, vendored notes saying "package versions stay unchanged", and
  fingerprints that frame version macros but not patch content.
- **Countercondition:** pure-software math crates whose architecture overrides replace only
  correctly rounded operations (`sqrt`, `fma`) stay portable with those overrides. See the
  comparative reviews of [numerical workflows](reviews/numerical-workflows.md#cbo-033) and
  [native solver integration](reviews/native-solver-integration.md#cbo-028).

<a id="lop-x007"></a>

## LOP-X007 — A library derive owns one projection of a multi-projection declaration

**Articulation.** One declaration, a type or a variant, must yield several projections: wire
encoding, schema, identity framing, diagnostic code and facts, Python surface. A library derive
covers one projection well. Its attributes, however, are shared across every consumer of that
trait, or its values are untyped. The other projections are then hand-maintained beside it, or
the library forces a second spelling of a fact the codebase already holds in typed form.

**Recognize.**
- Hand trait impls next to a library derive on the same types.
- String literals in library attributes duplicating a typed enum.
- Macros that implement a library trait by hand to keep a typed half in sync.
- Post-passes that rename library output per projection.

**Investigate.**
- Does the library offer a separate attribute namespace, shadow or remote types, or
  per-projection settings?
- Would using the derive for one projection create a second declaration of a shared fact?
- Would a small local derive that emits both the library trait and the local trait be the single
  authority?
- Which projections intentionally differ?

**Counterconditions.**
- Deliberately divergent projections should not be forced through one set of attributes.
- A local proc-macro is real maintenance.
- A library may later add typed or dynamic hooks.

**What the base patterns missed.**
- LOP-02 assumes a derive from the authority removes the parallel forms; here the library derive
  cannot be the authority for every projection.
- LOP-10 frames this as fighting the mechanism, yet the mechanism works as designed for its own
  projection.
- LOP-X003 concerns generator towers, not derive scope.

**Prompting observations (2026-10-27, static reading).**
- A diagnostics derive whose code attribute is a static string while the codebase holds a typed
  code enum.
- A serialization derive shared by wire encoding and identity framing.
- Per-contract schema generation without cross-contract naming.

See [CBO-039](codebase/authoring-and-identity.md#cbo-039),
[CBO-042](codebase/registry-generation-and-python-boundary.md#cbo-042) and
[CBO-045](codebase/registry-generation-and-python-boundary.md#cbo-045).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Which side is the authority?** A local projection that implements the library trait *from*
  the typed fact may be the single authority, and the library-derive sites the drifted second
  declaration. Then migrate the derive sites onto the local projection rather than the reverse.
- **Forcing a per-field decision.** When a projection must make a deliberate decision per field,
  as identity framing does, exhaustive destructuring is a zero-dependency alternative to a derive
  whose default includes every field. Cover nested records whose fields are manually selected;
  an exhaustive outer pattern does not force a decision about a new nested field. See the comparative review of
  [CBO-045 and CBO-039](reviews/declarations-registry-and-python-boundary.md#cbo-045).

<a id="lop-x008"></a>

## LOP-X008 — A property is proved by enumerating a recyclable namespace instead of holding an owned handle

**Articulation.** Code establishes membership, liveness, identity or completion by scanning or
re-reading a global, recyclable namespace and guarding the races by hand. Examples are "all
descendants exited", "this is still the same process" and "the unit has stopped". The hand
guards are read-before and read-after, repeated inventory, start-time comparison and polling.
The platform offers a handle or container that owns the property:
- a process file descriptor or its inode identity;
- a control group with a populated event and an atomic kill;
- a manager job with a completion signal;
- an anonymous temporary inode that has no name until it is published.

**Recognize.**
- Process-directory scans filtered by group or parent.
- Identity tuples such as (boot id, pid, start time).
- "Re-inventory after kill" loops.
- Leaders kept unreaped to reserve an ID.
- Polling a status command until a state string changes.
- Orphan-cleanup scans for staging names.
- Comments about PID reuse or "unproved drain".

**Investigate.**
- Which kernel or manager object owns the property, and at what version on the actual host?
- Can the subject be placed into that object at creation, or only afterwards, leaving a window?
- Does the event cover every exit path: crash, re-parenting, self-migration?
- What deployment prerequisite does it impose (delegation, bus access), and what remains when the
  prerequisite is absent?

**Counterconditions.**
- The handle may be unavailable in some execution contexts, such as no delegation, an older
  kernel, or tests outside a manager. A fallback then keeps both mechanisms.
- The handle's guarantee can differ subtly: containment does not stop deliberate
  self-migration, and inode identities are per boot.
- Records checked after the subject is gone may still need namespace-derived facts.

**What the base patterns missed.**
- LOP-01 points at libraries, but the decisive capability is usually a kernel or manager object
  with no library in between. Libraries in this space often repeat the enumeration with weaker
  guarantees.
- LOP-08 frames lifecycle ownership, while the change here is the *evidence object* for a
  lifecycle fact.

**Shared-language note.** For host-local projects, the host's kernel and service-manager versions
are realizations in the same sense as library pins. Name them in applicability records.

**Prompting observations (2026-10-27, static reading).**
- Process-group inventories with pidfd re-checks.
- Start-tick identity parsed at many sites in two languages.
- Drain observation that re-reads cgroup events.
- Unit-state polling through status output.
- Deterministic staging names that need recovery logic.

See [CBO-003](codebase/cross-cutting-mechanisms.md#cbo-003),
[CBO-004](codebase/cross-cutting-mechanisms.md#cbo-004) and
[CBO-019](codebase/execution-lifecycle-and-observation.md#cbo-019).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Countercondition: the evidence object must sit inside the owner that holds the subject's
  resource charge.** A handle created under a different manager or ledger (a transient unit per
  child) can satisfy containment while moving the subject outside its accounting boundary. Prefer
  an owned sub-object of the container that is already charged.
- **Countercondition: a persisted record checked later by another process is not a handle.** If
  the recyclable key carries a qualifier that makes reuse non-credible on the actual host (start
  ticks, bound to boot and namespace), migrating persisted records to a handle-derived ID adds
  format churn without a guarantee gain. The handle belongs where the process is acted on.
- **Countercondition:** objects created by a manager may be read freely but must be acted on
  through the manager or through an owned, delegated sub-object. Delegation must be signalled
  explicitly: on this host's systemd the user manager sets no `user.delegate` xattr, and directory
  ownership lets an undelegated scope create leaves anyway
  ([delegated cgroup probe](../design_review/evidence/library-leverage-delegated-cgroup-containment-2026-10-27/README.md)).
- **Investigate the coarser level first.** A container handle at the unit level may already prove
  the property, leaving only an in-process proof to improve.
- **Look for consumers that call the effect owner's tool only to read facts** they could read
  directly. Hand the evidence handles off with the effect's result and let the consumer observe
  in-process; authority to act stays with the owner. See the
  [comparative review](reviews/host-processes-and-durable-files.md#cbo-020).

<a id="lop-x009"></a>

## LOP-X009 — A local guard compensates for a library's unsafe handling of untrusted input

**Articulation.** A local preflight or wrapper exists because a library, given hostile or corrupt
input, panics, aborts or allocates whatever sizes the input advertises. The guard looks like
duplicated validation, but at the pinned version the library does not discharge it. Upstream
fixes then arrive piecemeal across maintenance lines, majors and unreleased branches. A guard can
be correct now and become obsolete one check at a time.

**Recognize.**
- Comments such as "before the library allocates" or "some constructors assert".
- Tests named after corrupt lengths.
- Manual verifier calls ahead of a reader.
- Unwind-catching around decoders.
- Type allow-lists placed in front of a library conversion.

**Investigate.**
- Decompose the guard into individual checks. For each check, record:
  - the library behaviour it guards against (allocation order, assert, unwrap, panic);
  - its upstream status: issue, fix, first released version;
  - whether that release is reachable under the project's version constraints.
- Separate guard checks from domain policy that would remain anyway.
- Ask whether a lower-level push API moves allocation ownership to the caller.
- Consider contributing the missing check upstream.

**Counterconditions.**
- A fix that turns an abort into an error can still allocate the advertised size, so it does not
  satisfy a prepay contract.
- An upgrade may be blocked by a coupled-family or engine constraint.
- Some checks are domain policy that merely looks like a guard.

**What the base patterns missed.**
- LOP-03 assumes the library already discharges the obligation.
- LOP-11 assumes the limitation has gone away.
- LOP-01 frames the code as reinvention.

None prompts a per-check mapping to upstream fix status.

**Prompting observations (2026-10-27, Interface-checked).**
- A columnar IPC reader that allocates an advertised footer length before seeking.
- The same reader allocating stream bodies before reading them and asserting on malformed
  metadata.
- Several upstream issues and fixes that arrive only in a major version not yet reachable here.

See the [Arrow assessment](libraries/arrow/README.md) and
[CBO-005](codebase/result-data-paths.md#cbo-005).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Map guards to trust boundaries, in both directions.** Ask where uncertainty actually enters,
  not which path happens to call the reader. A full preflight on digest-verified, system-written
  bytes while import or transport paths call the stock decoder is a placement defect even when
  every check is justified; the [hostile-stream probe](../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)
  showed that such an import path can abort the process. Conversely, a guard re-run on bytes a
  trusted producer has just emitted in the same process, after an import-time fingerprint has
  established agreement, is assurance re-established without a trust transition. Decompose each
  guard: which property does each check establish, and which inputs (untrusted bytes, stale
  builds, producer bugs) can violate it? Keep any trusted fast path private so untrusted entry
  points cannot reach it. Counterconditions: a producer in another process, build or version is not
  trusted merely because it shares the codebase, and durable or recorded bytes are untrusted on
  every read.
- **Ask whether a lower-level decoder lets the guard and the allocation share one read.**
  Caller-owned encoded input does not prove ownership of internal accumulation, metadata,
  wrappers, alignment copies or decompression. Check those paths before replacing complete
  grants with a raw input span; preserve consumer-specific schema/lineage rules when sharing
  transport-safety mechanics.
- **Unchecked preconditions.** The hazard can be a library contract asserted only in debug
  builds rather than hostile input. The failure then differs by build profile: a panic in debug,
  a silent wrong result in release. Check whether the same library's companion producer can emit
  inputs that violate the precondition (an isolator feeding a refiner), derive reachable
  violations from source, and prefer a complete cheap precondition, or restructuring the input so
  the violation cannot arise, over screening for observed failures
  ([containment probe](../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)).
  See the comparative reviews of [CBO-005](reviews/result-data-and-prepaid-accounting.md#cbo-005),
  [CBO-041](reviews/declarations-registry-and-python-boundary.md#cbo-041) and
  [CBO-025](reviews/mathematics-preparation.md#cbo-025).

<a id="lop-x010"></a>

## LOP-X010 — A harness hand-builds one side of a protocol whose counterparty could change

**Articulation.** A test, diagnostic or observation harness implements one side of a wire
protocol by hand: an HTTP/2 or gRPC receiver, a varint walker, a static header blob, or an RPC
client. The product side of the same exchange uses a library. The harness bypasses the library
ecosystem, either because of dependency weight or because the counterparty's protocol was taken
as fixed. The cheapest remedy may be to change which protocol the counterparty speaks, rather
than adopt a library for the same protocol.

**Recognize.**
- Raw socket code checking protocol prefaces.
- Frame-type integer constants and byte-literal encoded headers.
- Positional field walks with comments naming schema fields.
- Generated stubs for the protocol already installed, perhaps transitively, but unused.

**Investigate.**
- Can the counterparty be configured to an exporter or protocol the harness can consume with
  the standard library or installed packages? Examples are HTTP instead of gRPC, a file or JSON
  exporter, or a standard collector.
- Are message stubs already installed?
- Is a sans-IO protocol library a smaller step than a server framework?
- Which bounds and fail-closed behaviours must survive?

**Counterconditions.**
- The harness may deliberately exercise the exact production protocol.
- A hand peer can enforce stricter bounds than library defaults.
- Cold-start paths may forbid dependencies.

**What the base patterns missed.**
- LOP-01 frames the remedy as adopting a library for the same protocol.
- LOP-10 concerns integration with an adopted mechanism, not the choice of interchange.

**Prompting observations (2026-10-27, static).** A diagnostic telemetry capture sink with a
hand-written HTTP/2/gRPC server and protobuf walk, whose producer is configured through standard
exporter variables. A hand gRPC client in a database test helper. See
[CBO-051](codebase/developer-tooling.md#cbo-051).

**Revision 3 refinements (comparative review, 2026-10-27).**
- **Investigate the counterparty's build as well as its configuration surface.** Honouring
  standard endpoint variables does not mean every standard transport is compiled in; a server
  built with only a gRPC exporter accepts endpoint settings but cannot be switched to HTTP.
- **Countercondition:** when the counterparty is a third-party binary, changing its protocol may
  need an upstream change that is unavailable until released. The hand peer may then be the
  minimal correct remedy; record its reason where readers look.
- *Prompting observation (2026-10-27):* the SurrealDB 3.3.4 server exporter is `with_tonic()`
  only. See the [comparative review](reviews/developer-tooling.md#cbo-051).

<a id="lop-x011"></a>

## LOP-X011 — A publication contract keeps interrupted work products

**Articulation.** A publication step deliberately keeps a named staging artifact when it fails:
for resume, for inspection of an acknowledged prefix, or as recovery evidence. Library helpers
built on RAII temporary files delete staging on error or drop and choose random names. Adopting
one silently removes a promised artifact and the key needed to resume.

**Recognize.**
- Documentation or contracts saying that interrupted staging "remains".
- Deterministic `.partial`, `.incomplete` or `.pending` names.
- Resume code that compares an existing prefix.
- Failure paths that publish a prefix.
- Recovery scanners that expect named residue.

**Investigate.**
- Is the residue promised (by the architecture or its consumers) or incidental? Who consumes it?
- Can the library's cleanup be disabled while its publication call is still used, and what
  remains if it is?
- Would a lower-level primitive change only the publication step? An example is
  `renameat2(RENAME_NOREPLACE)`.

**Counterconditions.**
- Where residue has no consumer and no resume value, RAII cleanup is a real gain.
- Abandoned temporaries that are merely litter are a different shape, such as a writer killed
  under a ledger lock whose next holder can remove them.

**What the base patterns missed.** LOP-01 and LOP-08 recommend delegating lifecycle to a library.
They do not flag that a library's lifecycle default can contradict a promised failure artifact.

**Prompting observations (2026-10-27, Interface-checked).** An export and workbench `.incomplete`
contract, a canonical `.pending` resume, and a review-pack partial-prefix publication; contrast a
host-admission temporary that is only litter. See the
[comparative review](reviews/host-processes-and-durable-files.md#cbo-003).

<a id="lop-x012"></a>

## LOP-X012 — A retry's classification requires settling an uncertain effect

**Articulation.** A codebase retries an effectful request. Whether another attempt is permitted
depends on more than the returned error: a lost acknowledgment, an ambiguous commit or a
concurrent identical request must first be settled from durable evidence (a receipt, an
idempotency record, a read of the target state). Only a *definite* refusal or conflict authorizes
a new attempt. Generic retry combinators assume that the error alone classifies the attempt and
that the operation is safe to call again. Here the classification is a small protocol between
attempts.

**Recognize.**
- Retry loops that run a readback query in the error branch before deciding.
- Comments such as "unknown outcome keeps its original identity", "never replays" or "settled by
  the immutable receipt".
- Error predicates named `*_refused` or `proves_*` beside `is_conflict`.
- Retry classes that do not count against the attempt cap.

**Investigate.**
- Which errors are *definite* (safe to retry) and which are *uncertain* (settle first)?
- Does the settlement read run under the same clock as the attempts?
- Can the attempt step return its own classification (done, retry counted, retry uncounted), so a
  shared owner keeps pacing, caps and attribution while the step owns settlement?
- Does a library retry accept a step-supplied classification, or only an error predicate?

**Counterconditions.**
- Idempotent requests (reads, keyed upserts with idempotency tokens) need no settlement, and a
  library combinator fits.
- If the server offers request idempotency keys, the settlement read may be replaceable by
  resubmission.

**What the base patterns missed.** LOP-08 frames lifecycle delegation, LOP-09 sees the duplicated
loops and LOP-X005 covers drop-to-cancel semantics. None names the read-before-retry protocol that
makes a combinator-first remedy unsafe at the core sites.

**Prompting observation (2026-10-27, static).** Canonical store activation and admission loops
read an immutable operation receipt between conflict attempts. A generic retry library would place
that readback inside its factory or skip it. See the
[comparative review](reviews/execution-lifecycle-and-observation.md#cbo-010).

<a id="lop-x013"></a>

## LOP-X013 — A library kernel's value semantics differ from the domain's

**Articulation.** A scalar loop that compares, orders, aggregates or tests equality is a natural
candidate for a vectorized library kernel. The kernel may define those operations differently from
the domain on special values: IEEE totalOrder rather than IEEE comparison, so `-0.0 < 0.0` and NaN
is greatest; NaN-propagating rather than NaN-ignoring extrema; SQL three-valued logic rather than
domain missingness; byte rather than collation order. The substitution compiles and passes typical
data, then silently changes results on exactly the values the domain treats carefully. When values
derived under one semantics are stored and later re-checked under the other, previously valid
records start to refuse.

**Recognize.**
- Loops using `f64::min`/`max`, `>=` or `==` on floats that are candidates for Arrow, DataFusion,
  numpy or Polars kernels.
- Recorded extrema or statistics that a reader recomputes and compares.
- Existing normalization expressions such as `CASE WHEN x = 0.0 OR x = -0.0 THEN 0.0`, which show
  the codebase already met the difference elsewhere.
- Predicates documented as "finite" or "IEEE-bit exact".

**Investigate.**
- What ordering and equality does the kernel use? Read its source or docs; names do not tell you.
- Which special values can the domain actually produce or store? Check declared field domains and
  store assertions.
- Is there a recorded value computed under the old semantics that a reader re-checks?
- Can the kernel be kept for the bulk work while the domain semantics are preserved by
  normalization, a guard or a caller-supplied closure (for example `BooleanBuffer::collect_bool`)?

**Counterconditions.**
- If the domain excludes the special values by declaration and enforcement, the difference is moot.
- If both the recorded and the re-checked values move to the kernel's semantics together, and no
  stored record depends on the old ones, the substitution can be correct.
- A deliberate change to the domain rule is a domain decision, not a leverage choice.

**What the base patterns missed.** LOP-04 asks whether a collective operation preserves ordering,
multiplicity and failure attribution, but not value semantics. LOP-03 assumes the adopted
capability fits. LOP-X006 covers bit reproducibility across a stack, not differing definitions
within one operation.

**Prompting observations (2026-10-27, Interface-checked).** Dense trajectory extrema recorded with
`f64::min`/`max` and re-checked by IEEE `==`; a dense time window using IEEE `>=`; `arrow-ord`
59.3.0 comparison and `arrow-arith` extrema using totalOrder; a scalar result predicate that
already normalizes signed zero. See the
[comparative review](reviews/result-data-and-prepaid-accounting.md#cbo-006).

**Reassessment refinement.** Readback equality can hide a durable signed-zero encoding change;
retain the current encoding unless that semantic change is selected. Exact-to-binary64
conversion has its own value contract: adjacent-float rounding midpoints are asymmetric at
powers of two, and a correctly rounded output promise may be stronger than a consumer's
residual-verification requirement. The
[rounding-cell counterexamples](../design_review/evidence/library-leverage-binary64-rounding-cells-2026-10-27/README.md)
Test that proposed formula, not a product replacement. Compare the complete necessary output
contract before treating stronger certification as mandatory.

<a id="lop-x014"></a>

## LOP-X014 — Library-held process-global retention outlives caller ownership

**Articulation.** A library keeps process-global state, such as memo caches, interning tables and
registries. Ordinary calls populate it as a side effect, and it is never released, or only through
an API the caller does not own. The caller's admission and lifetime model (charge before allocate,
release with the owner, bounded retention) cannot see or bound it. In a long-lived process with an
unbounded population of distinct inputs (sweeps, dynamics, many cases), the state grows outside
every budget. Its contents can also influence later results, through shared enclosures or
canonical indices.

**Recognize.**
- `static`, `LazyLock`, `OnceLock` or `thread_local!` maps in library source keyed by input values.
- Documentation saying "cached", "global" or "process-wide".
- Handles or indices into a global table.
- No `clear`, `evict` or capacity API.
- Caller code that pre-registers or chunks library entries to control growth or order, a managed
  response to the same shape.

**Investigate.**
- Which calls populate the state?
- Is the key population bounded by the caller's domain (a few polynomial orders) or by data
  (parameter values)?
- Is there a non-caching entry point, a clear or capacity API, or a scoped context object?
- Does the cached state affect results or their order?
- Can the caller restructure to avoid the populating calls, or confine them to a bounded
  population?

**Counterconditions.**
- A bounded key population makes the retention a fixed cost.
- Some global state is required for identity (symbol interning) and is better managed, for
  example by pre-registered append-only pools, than avoided.
- A short-lived process may tolerate it, and the cache may be the library's main performance lever.

**What the base patterns missed.** LOP-08 covers lifecycles the caller coordinates, not state the
caller cannot own. LOP-X001 covers predicting a call's cost, not retention that outlives the call.
LOP-X002 covers context carriers, not data retention.

**Prompting observation (2026-10-27, Tested).** A computer-algebra library's root cache is
populated by root isolation and refinement with no eviction; a cubic solver's fallback keys it by
per-parameter-point coefficients and takes that fallback on most solves
([containment probe](../design_review/evidence/library-leverage-symbolica-root-containment-2026-10-27/README.md)),
while collocation's use is bounded. The repository's formal-symbol pool is an existing managed
response to the same library's global symbol registry. See the
[comparative review](reviews/mathematics-preparation.md#cbo-025).

## Notes for base-framework maintenance

These are recognition cues and counterconditions found during the library assessment. They fit
existing base patterns, so they are recorded here for optional manual promotion and are not
definitions.
- **LOP-03.**
  - Recognition cue: per-run copies of a tool's configuration file, edited by script, where the
    tool offers a layered overlay channel (a tool config file, profile, environment or flags).
  - Recognition cue: a vendored native suite that ships unused components a build feature could
    enable.
- **LOP-03 and LOP-11.**
  - Countercondition: adopting an upstream driver or newer release also imports its open
    defects. Check the issue tracker for the release considered.

- **LOP-02 and LOP-09** (comparative review). Counter-cue: before treating several translators or
  encodings as fragmentation, check whether they are successive layers of one chain (a semantic
  boundary followed by an ABI dispatch), or spokes from one hub whose per-spoke table is a single
  forward map used in both directions. Such structures rarely drift. Recognition cue: a generated
  binding module with no non-test importers beside a hand-written structural twin in its natural
  consumer. Countercondition for LOP-09: layered surfaces that forward arguments opaquely (a recipe
  passing `*args` to a script) are not fragmented declarations; count only re-declared meanings.
- **LOP-03** (comparative review). Countercondition to the per-run configuration copy cue: overlay
  channels have reference and precedence rules (a tool config may reference only its own
  namespaced groups and be outranked by repository profile settings). When the policy must act on
  the repository's own declarations, a per-run copy can be the correct mechanism.
- **LOP-12** (comparative review). Recognition cue: a parity table marks a reference subsystem
  implemented while the reference includes a generator or helper (sweep sample generation,
  estimated-variance statistics) that the realization lacks. Countercondition: when a library
  would supply only leaf computations and the substance is a statistical or domain-model decision,
  the opportunity is a product or policy choice; route it to functional review or decision work.

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
- A positional result convention standing in for a typed envelope (a result read at
  `num_statements() - 2` at dozens of sites). Considered by the comparative review; LOP-02 with
  LOP-05 frames it adequately.
