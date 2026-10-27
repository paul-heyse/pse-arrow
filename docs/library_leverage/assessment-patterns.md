# Additional library-leverage patterns

**Assessment output · revision 2, 2026-10-27.** Revision 1 came from the first codebase audit;
revision 2 adds `LOP-X004`–`LOP-X010` and refines `LOP-X001`–`LOP-X003` from the first library
applicability assessment. These are additional framings found during
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
