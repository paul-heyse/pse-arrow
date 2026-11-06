---
title: "Plan65e: Scientific portability and execution"
status: in-progress
date: 2026-11-03
adrs: [ADR-0242, ADR-0243]
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md, docs/plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, numerical-execution, preparation, validation]
---

# Scientific portability and execution

## Target and baseline

**Proposed:** give scientific source, preparation, capability, invocation and outcome products
one explicit portable interpretation wherever exchange or independent inspection is useful.
Keep library-owned mathematical programs, numerical buffers and solver workspaces in their
effective native representations. Direct and isolated execution consume the same scientific
meaning; placement does not create a second formulation or derivative authority.

The [coordinator](65-proto-integrated-target.md) owns the combined design, rule decisions and
completion. This companion owns E01/E04/E05/E06. [65k](65k-scientific-artifacts-and-native-reconstruction.md)
owns E02/E03, including their historical evidence and remaining qualification. Execution is authorized alongside incomplete Plan64
as of 2026-11-04, including replacement of overlapping mechanisms. Individual deliveries
consume actual native capabilities and working interfaces; whole-Plan64 completion is not a
starting gate. Current source was inspected at
`f61286a5fa5894b480efe8584c8b3d9b76d1b1fb`, including concurrent projection edits. No product
or science changes were made while authoring. The source reads are **Interface-checked**;
every package and benefit below remains **Proposed**.

The present routes expose useful variation: `pse-kernels` has `ProviderSpec`, `ProviderDemand`,
`PreparedProviderRequest`, `ProviderLinearizationToken` and explicit `ExecutionScope`;
`scientific_worker` transports a prepared water binding; `math/native_child.rs` reconstructs
a closed provider-free KINSOL root using private Serde documents; `typed_math_portable`,
kernel portable specifications and quantity receipts contain separate capture/restore forms.
These are actual migration populations, not a demand to serialize arbitrary native state.
Blueprint §7/§14–§18 supply existing scientific contracts; Plan65 may improve their realization.

[65i](65i-scientific-operations-and-product-semantics.md) now owns scientific operation,
product-use and consuming-dependency semantics from review I01–I03. This companion owns
their portable realization and execution placement. Native semantic agreement in 65i I01
precedes the general declaration design; working generated delivery follows narrow A/B
slices. Neither a new crate nor a complete scientific catalogue is needed to agree the
first operation's required meanings and independent discriminators.

The review-driven amendment was prepared against
`5f4fa0246c0b9fac68493b608f0b2eefe79f116d` with concurrent dirty source preserved. Earlier
source observations retain their original scope and date; the added semantic and dependency
obligations are Proposed implementation work, not newly tested scientific behavior.

## Execution population — 2026-11-04

The initial source reconciliation at `d63fcbb11` identifies these concrete migration units.
Existing source-owned worker protocol, physical type and qualified receipt projections are
reused; their existence does not complete the following portable consumers.

| Native owner | Portable population and receiving responsibility |
| --- | --- |
| `pse-compiler::typed_math_portable` | `Recipe` and `SpecWire` reconstruct compiler-owned source/physical context; qualified restore remains native. |
| `pse-kernels::portable` | `ProviderRecord`, `SpecWire`, `ShapeWire` and `PortWire` bind provider identity, ordered ports and shapes; supported demand and implementation admission remain native. |
| `pse-runtime::math::native_child` | `RootWire`, `SourceWire`, `Request`, `Response` and startup handshake serve the actual closed provider-free KINSOL child. Preserve source screening, bounded channel and process custody with complete producer/consumer cutover. |
| Runtime completion and simulation observation | Original scope, local validity, governing decision, provisional samples and selected accuracy attempt expose recorded native meaning. A decoded observation cannot grant new scientific use. |

I01's first native agreement is available before new generation: `DeclaredExecution` already
composes route, procedure payload, analysis and original preparation. A/B must project that
owner composition and construction role; no second operation registry is needed. The existing
analytic integrated model supplies positive native execution, while spatial-without-temporal-
closure remains a refusal control. Full portable E02 delivery waits for the narrow working A/B
slice, not every property/equipment family remaining in Plan64.

E04's first source cutover follows proposed ADR-0242: Stop/Close retire only their exact handle,
while explicit authenticated Shutdown closes admission and cancels/drains the full worker.
Each prepared generation has its own cancellation control. The server may receive one numerical
request and one control request concurrently, while native mutation stays on the single lane;
Stop signals cancellation before waiting for that lane to settle. Only Shutdown acknowledgement
sets the parent's graceful-drain observation, and actual process/I/O custody remains separate.
Proxy disposal sends Shutdown rather than using the first prepared handle as a process-wide
close. Native and real-transport two-handle controls cover independent survival and stale contexts.
Generated publication and actual-process acceptance remain required before E04 completion;
source changes alone do not establish that acceptance.

**Tested, targeted on 2026-11-04:** after root-owned generated publication `1a141f1f5`,
`scripts/pse test pse-runtime --lib --features property-coolprop` selected and completed four
controls with zero failures against the zero-failure target: the actual-process
`action_capable_native_owner_preserves_every_product_through_actual_transport`,
`handle_controls_cancel_only_exact_context_and_shutdown_closes_admission`,
`abandoned_observer_does_not_release_a_native_job_admission`, and
`shutdown_has_an_explicit_wire_operation_and_acknowledgement`. The real worker control
completed cooperative active cancellation, exact Closed acknowledgement, surviving-handle
evaluation, stale/replayed-request refusal and explicit Shutdown. Native capture and settlement
were complete under original user-scope custody, explicit cache-off execution and consuming-
closure Arrow validation. The admission control was initially nested and undiscovered; it was
moved into the actual test module and the four-test population was verified. This qualifies
the selected E04 slice, not every worker/scientific consumer or assembled H acceptance.

The numerical lane uses Tokio 1.53.3 `Semaphore::try_acquire_owned`; its
`OwnedSemaphorePermit` moves into the native job rather than remaining on an observing RPC.
Observer cancellation therefore cannot admit a second numerical mutation before the original
native job settles. See the [versioned semaphore contract](https://docs.rs/tokio/1.53.3/tokio/sync/struct.Semaphore.html#method.try_acquire_owned).
The existing I/O owner polls at most two RPC futures using futures-util 0.3.34
`FuturesUnordered`, allowing a control operation beside numerical evaluation without detached
tasks or a second driver. Dropping that collection cancels pending transport futures, while
the native job and parent process reservation retain their own settlement duties; the
[versioned stream contract](https://docs.rs/futures-util/0.3.34/futures_util/stream/struct.FuturesUnordered.html)
describes polling and drop semantics. Those mechanisms use existing pinned dependencies and
express the worker's serial mutable lane, not a new host-wide concurrency limit.

E02/E03 package status now has one owner in
[65k’s package table](65k-scientific-artifacts-and-native-reconstruction.md#packages-and-migration).
Their labels and earlier evidence are preserved; this extraction is not completion or reset.

## Selected design and alternatives

Use the pure structural contract owner from [65a](65a-contract-foundations-and-generation.md)
for tags, shape, local constraints and typed options. Scientific owners supply physical
meaning and contextual constructors. Generate Buffa records, checked native adapters,
descriptor metadata and independent exchange bindings together. Keep exact scalar bits,
ordered quantity/unit identities, precision/formulation/implementation identities and
declared approximation scope explicit. A schema fingerprint is not a scientific cache key.

The operation carries its admitted route/procedure, physical/data/formulation context,
required initial/boundary/observation inputs, derivative/validity requirements and product
assessment contracts from 65i. Generated options expose those decisions; native authored
guards and checked functions govern them. The receiver must preserve unsupported scientific
combinations rather than infer support from matching descriptors or dimensions.

Three portable products are deliberately distinct:

| Product | Content and admissible consumer |
| --- | --- |
| Reconstructible scientific input | Authored body/source, resolved binding context, physical registry interpretation, unknown/row ordering, exact parameters, solver/evaluation policy and complete dependencies. A receiving compiler reconstructs and admits it; serialized admission labels confer no authority. |
| Prepared capability/demand | Exact implementation/data/configuration identity, ordered ports, derivative product/order, validity regime and immutable prepared handle. The selected implementation confirms support before numerical execution. |
| Outcome/evidence | Native termination and status, candidate availability, original-coordinate qualification, physical closure, usable extent and context. Transport success cannot substitute for any scientific conclusion. |

Outcome projection includes the original task/step/point product scope, local mathematical
validity, governing request coverage/qualification and selected accuracy attempt where relevant.
Use the native completion/product selectors; do not recompute permission in wire clients.
Raw/provisional evidence may remain readable without becoming an ordinary usable result.
A decoded prior permission reports its original decision and cannot issue current admission.

Retain source versus optimized mathematical artifact distinctions. Do not reconstruct authored
expressions from optimized evaluator instructions or claim that an exported instruction set
preserves source shape. Opaque evaluator/provider closures, native thread state and factorizations
remain process-local. Reuse native library artifact export only with its exact producer/version/
profile contract and an actual receiving consumer; otherwise exchange reconstructible source.

The private root's artifact/envelope realization is selected in 65k against its actual
receiving contract, retaining the narrow executable factory, bounded inherited channel and
parent-side original screening. Generated correspondence does not itself select a protobuf
mathematical body or qualify live native packing. Remove replaced wire/reconstruction
mechanisms only at the complete selected consumer cutover; no new mathematical class follows.

A universal Proto mathematical IR would duplicate library algebra and introduce conversions
into hot callbacks. Keeping every private wire form indefinitely would repeat scientific
interpretation as families grow. The selected mixed design removes repeated portable
correspondence without making a codec the numerical engine. Proto-first external scientific
contracts remain eligible when independently governed, through explicit native adoption.

## Worker and coherent-batch interface

### Transactional handoff and explicit placement finish — November 5 amendment

This section integrates review [I02/I07](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I02)
into E04. Current source admits Prepare into the handle map before awaiting job submission,
and exits the shared native loop when its response receiver has departed. Correct these
source-established hazards without conflating one method waiter with parent/process loss.

Reserve queue capacity using Tokio 1.53.3 `Sender::reserve_owned` before reversible Prepare
bookkeeping; commit that bookkeeping and `OwnedPermit::send` synchronously with no intervening
await. A failed reservation has no native effect. Preserve independent immediate Stop/Shutdown
signals so active indivisible FFI can observe cancellation rather than waiting behind itself.
The native Job retains its `OwnedSemaphorePermit` until mutation and bookkeeping settle.
[Capacity/transfer contract](https://docs.rs/tokio/1.53.3/tokio/sync/mpsc/struct.Sender.html#method.reserve_owned).

Default disposition on **definite observing-method departure** during Prepare is performing-owner
retirement after preparation/native work settles. No new recovery registry is selected.
Loss of an Evaluate waiter on a still-live authenticated parent connection does not retire
unrelated prepared handles. A successful oneshot send is not a wire/client ACK: later network
reply loss remains an uncertain original outcome under existing parent/process custody.
Explicit parent/connection loss still permits the existing whole-process containment policy.
ConnectRPC 0.9.1's deadline wrapper can drop the method future through Tokio `timeout_at`;
it does not settle the native job. [Exact wrapper](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/service.rs).

At the scientific placement owner provide explicit result-bearing finish: close admission,
request the appropriate handle/process shutdown, attempt independent process reap and I/O join,
and preserve original typed scientific and cleanup failures. Outstanding handles/activity
retain their owner and charges until actual settlement. Failure to establish reap or I/O
completion must not release admission merely because cleanup was attempted. Drop remains
best-effort abandonment containment and cannot certify a successful finish.
Tokio `JoinSet` retains outcomes and requires joining after abort; tokio-util 0.7.19
`TaskTracker` observes destruction but neither rejects post-close spawns nor preserves task
results. Do not substitute it for performing-owner result collection.
[JoinSet](https://docs.rs/tokio/1.53.3/tokio/task/struct.JoinSet.html),
[TaskTracker](https://docs.rs/tokio-util/0.7.19/tokio_util/task/struct.TaskTracker.html).

E04's actual controls must cover cancellation before/after reservation, definite non-submission,
accepted native work, abandoned Prepare, lost waiter with an independently usable second
handle, late completion and actual parent loss. Combine original failure, reap failure and
I/O panic/delayed destruction, preserving every result and admission through the last physical
owner. Remove pre-enqueue mutation, unconditional reply-loss shutdown and ignored cleanup
results from the positive explicit finish path only after replacement controls pass.
These are planned corrections; no new fault exercise or product implementation is claimed.

Generalize the post-Plan64 worker as an explicitly prepared local placement. Preserve separate
attempt, process/session, preparation generation and invocation identities. Every Evaluate
references an admitted handle and exact demand. `Stop(handle)` cancels only that handle's work;
`Close(handle)` retires only that prepared state after its invocations settle. Process shutdown
closes admission, cancels and drains all remaining sessions, then reaps the child and joins I/O.
Global session clearing is not the general target semantics.

Add capability-declared `EvaluateBatch` for implementations that can evaluate useful groups of
independent states or one explicitly ordered stateful batch. Each input has a point identity;
the response preserves input order and per-point success/refusal. A batch-wide infrastructure
failure and a point-local physical refusal are separate outcomes. Do not replay completed or
uncertain points automatically. Shared preparation and buffer transfer are amortized; the
backend must expose a genuine grouped operation, not hide a scalar RPC loop.

Prepared linearizations and derivative tokens are opaque references bound to the admitted
handle, input/branch context and provider generation. Invalidating the context invalidates
the token. Support all existing kernel demand alternatives structurally, but reject unavailable
products honestly. Never synthesize an action product by secretly materializing a dense matrix
unless the selected scientific implementation explicitly supplies that realization.

Use direct native calls by default for repeated callback evaluation. An isolated route is
selected explicitly for independent implementation, foreign-runtime isolation or demonstrable
operation benefit. Shared connection machinery does not globalize a single serial lane across
unrelated attempts. Per-session native state stays owner-affine; concurrent independent sessions
may run on independent admitted lanes. All retained buffers and reservations survive through
actual native/I/O settlement, including cancellation and stale late replies.

Buffa 0.9.2 `MessageView`, `OwnedView`, `ViewEncode`, `SizeCachePool` and backing-aware `Rope`
are eligible physical mechanisms at source/artifact boundaries. They do not remove required
native numeric buffer construction. Select compact copies where retaining a small view would
pin a large allocation. ConnectRPC clients and transports provide protocol work; the prepared
handle and native custody still establish scientific continuity and release.

For scientific metadata, Buffa `DynamicMessage::from_options` reads 65a's typed options in
the selected `DescriptorPool`; generated `ReflectMessage::for_each_set` avoids an unnecessary
owned dynamic bridge for read-only inspection. Neither mechanism executes physical admission.
Use maintained generated `Validate` for supported local shapes, then 65i's native receiving
operation and product-use rules. Precise conditions are in
[Buffa 0.9.2 reflection](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/index.html),
[generation APIs](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/) and the maintained
validator source linked below. Interpretation/validation/mapping changes affect their actual
consumers; the whole descriptor world must not become a blanket mathematical preparation key.

### Production finish ownership — execution refinement

Source investigation during resumed execution establishes that `NativeSession::close` does
not directly own every provider. `pse_math::assembly::CaseWorker` owns the provider bank;
retained KINSOL/POUNCE callbacks, conditional-unit shared evaluators and Horizon dynamic
workers can own or borrow those workers. Complete E04 through these actual owners, including
replacement, reset, scratch recycling and final scope departure. The transactional transport
slice does not implement this remaining contract.

Proposed [ADR-0247](../adr/0247-consuming-native-retirement.md), linked to ADR-0240/0242,
records the kernel contract before activation; formal acceptance remains separate. The
proposed realization uses consuming `Provider::finish`, binding/bank
finish and `CaseWorker::finish`, with explicit forwarding through owning wrappers. Borrowed
oracle aliases release their borrow; the actual evaluator owner finishes only after native
callbacks are destroyed. Attempt every independent provider retirement and retain its key
and original typed cause. A returned error reports settled failure; unreaped native work
remains owned by the original performing finish/supervisor operation, with admission retained.
Do not release custody merely because a Shutdown, kill, wait or I/O-join attempt failed.

Extend `execution::Retained`'s existing type-erased session/workflow storage with the
monomorphized consuming finisher supplied at actual construction. Make clear, incompatible
replacement, independent reset and final finish result-bearing; clear retains its existing
adapter-only meaning. Retire provider banks before successful scratch publication, replacing
normal recycle-on-Drop with explicit recycle-on-finish. Propagate outcomes from the owner-affine
native scope through existing native-thread/session joins and `MathRuntimeError::settle`;
retain original scientific failure and independent teardown failures together. Drop remains
abandonment containment, not positive retirement evidence.

Constructor rejection has the same consumption duty. Retain original typed refusal and all
already-built supplier cleanup failures; the mathematical owner's shared
`SupplierConstructionFailure<E>` preserves concrete scientific errors without thread-safety
or string-erasure requirements. A distinct non-cloneable root owner issues borrowed callback
handles; callbacks cannot perform owning retirement. Extract actual shared evaluators only
after callback execution has departed, so idle aliases cannot retain provider custody.
Independent retirement also contains panic-payload destruction before visiting siblings.

Owner-thread serial-scope admission now identifies the registry's concrete adapter with
`Any::type_id`, retaining its actual thread-scope checks. This is local type qualification;
it supplies no scientific identity. Rust documents that trait-object pointer equality can
change with duplicated or deduplicated vtables across codegen units. Sources:
[`ptr::eq`](https://doc.rust-lang.org/stable/std/ptr/fn.eq.html),
[`Any::type_id`](https://doc.rust-lang.org/stable/std/any/trait.Any.html#tymethod.type_id), and
[actual serial-scope owner](../../crates/pse-backend-native/src/execution.rs).

The pinned [ConnectRPC 0.9.1 HTTP/2 source](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/http2.rs)
detaches its shared-client worker and connection driver; an I/O-thread join alone cannot
recover their inner connection and outer task results. Preserve generated clients and codecs
while comparing a released public `ClientTransport` adapter over explicitly owned Hyper H2
`SendRequest`/handshake. Keep the driver `JoinHandle<Result<_, hyper::Error>>` with the existing
I/O owner, collect inner/outer failures, destroy pending transport tasks and explicitly join
the I/O thread. Sources: [ClientTransport](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs)
and [Hyper 1.11.1 HTTP/2 ownership](https://docs.rs/hyper/1.11.1/hyper/client/conn/http2/index.html).
This is connection ownership glue, not a second protocol implementation or request registry.

**Interface-checked limitation:** Context7's Hyper source account was reconciled with the
installed 1.11.1 implementation. The public connection future returns `Result<(), hyper::Error>`;
retain that result, executor task joins and actual thread completion. Its internal
`ConnMapErr` logs and erases some HTTP/2 errors, and `ConnTask` returns `()`. Dropped-sender/EOF
shutdown may therefore expose completion without the original internal error. This target
claims custody and library-exposed outcomes, not complete recovery of errors Hyper erases.
The [exact HTTP/2 source](https://github.com/hyperium/hyper/blob/v1.11.1/src/proto/h2/client.rs)
and [public connection source](https://github.com/hyperium/hyper/blob/v1.11.1/src/client/conn/http2.rs)
characterize that boundary; a future requirement for those hidden causes needs a separately
justified library change. Local and wire retirement still retain every cause their owners
actually observe, including late undeliverable RPC failures.

The receiving stress control exposed a concrete `BrokenPipe`: dropping the last client sender
after native Shutdown acknowledgement can race the server's remaining HTTP/2 writes. The
transport must explicitly initiate peer-led graceful shutdown from the completed native
Shutdown owner, keep polling Hyper's pinned server connection, and retain the parent sender
through driver completion only after the actual requested/decoded Shutdown. Publish that
I/O-owner state before synchronously delivering the acknowledgement. Startup, failed exchange
and cancellation without an acknowledgement retain the abrupt drop-first path; they cannot
wait indefinitely for a peer that has not agreed to stop. Preserve server errors as typed
`WorkerError::Connection` rather than flattening their original Hyper cause. Hyper's
[server Connection contract](https://github.com/hyperium/hyper/blob/v1.11.1/src/server/conn/http2.rs)
requires continued polling after `graceful_shutdown`; no broken-pipe exemption is introduced.

Evaluation and grouped-evaluation calls use no second `CallOptions::with_timeout` deadline.
ConnectRPC 0.9.1's `encoded_timeout` floors the wire unit and `client_deadline` constructs a
separate clock from that duration; it can win before the original scientific scope and expose
a protocol timeout instead of the original scientific expiry. Keep the default absent timeout
for these two calls, original parent pre-dispatch/wait/receipt/post-copy `ExecutionScope`
checks and native `Prepare.remaining_ns`. Already expired original scopes still refuse before
dispatch. Administrative calls retain their original `scope.remaining(30s)` transport allowance.
This removes a competing deadline owner without extending the scientific allowance or
reclassifying genuine transport failures. Characterization: pinned
[client timeout encoding and enforcement](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs)
and the [native transport policy](../../crates/pse-runtime/src/scientific_worker/transport.rs).

Local library causes without a complete allocation contract report unknown retained extent.
The existing `usize::MAX` retention sentinel conservatively refuses finite retention; it is
not a measured allocation. Preserve the original local typed cause. The worker wire restores
only declared known failure leaves and keyed retirement/settlement shapes; other diagnostics
cross as explicitly opaque observations with no native owner or restoration authority. Bound
the complete failure tree's depth, nodes and text before known-type restoration; refuse an
oversized outcome rather than silently dropping causes. The source-owned
[`Failure`](../../crates/pse-runtime/src/scientific_worker/protocol.rs) and
[`failure/refusal`](../../crates/pse-runtime/src/scientific_worker/handler.rs) composition use
the same generated Buffa correspondence and bounded decode owner as ordinary responses.

Acceptance covers every concrete owner above, shared versus borrowed callbacks, failed session
construction/replacement, multiple simultaneous cleanup failures, successful reap after failed
acknowledgement, unknown reap, pending method guards at runtime shutdown and scratch reuse only
after explicit retirement. Preserve selected-handle versus whole-process shutdown semantics.

The November 5 retirement source is integrated on main (`386e1557b`, consuming source
`797d9beb3`). Local original/constant/output/screening and fit/sample assessments now return
only after evaluator retirement; fallible preparation precedes worker acquisition where
possible. Partial sample construction and later-bank cleanup retain both scientific refusal
and keyed cleanup failures. Retained native values carry their consuming finisher through one
typed erased owner, eliminating the retirement-time downcast and separate closure allocation.
Known action failures compose directly with independent retirement failures. E04 remains in
progress for its complete consumer and lifecycle population; final targeted acceptance is
recorded in the coordinator's wrap-up checkpoint.

Expanded fitting controls also exposed stale prepared-support retention in the executable
cache. The receiving request now rebinds shared compiled code only after full compiler-issued
`PreparedSupport` equality, retaining the current descriptor-batch lease. Original source/issuer
checks remain strict; an identical hash grants no source authority. The implementation owners
are [`CompiledBody::with_receiving_support`](../../crates/pse-math/src/execution.rs),
[`ImplicitPrograms::bind`](../../crates/pse-compiler/src/workspace/modeling/executable/implicit.rs)
and the runtime's [modeling bind loop](../../crates/pse-runtime/src/math/modeling.rs).
This deliberately retains and charges the finite whole descriptor batch until its last factory
departs, rather than adding another ownership registry or guessing a smaller retained extent.
The fitting fixture now creates contextual packages with their actual runtime and injects
memory refusal into the execution context that owns transient work. Explicit consuming finish
replaces the former test assumption that Drop could make scratch reusable.

## Packages and migration

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| E01 | in-progress | Actual native capabilities; agreed scientific contract from 65i I01; co-designed 65a metadata | Initial concrete portable consumers are reconciled above; complete source/spec/context/receipt classification and bind each operation's inputs, support/refusal and product-use semantics before selecting projections. Preserve physical/configuration versus value-only change controls; no universal catalogue. |
| E04 | in-progress | Existing provider slice; 65d custody behavior; 65i dependency policy; ADR-0242; proposed ADR-0247 before generic finish activation | Preserve delivered per-handle Stop/Close versus Shutdown. Transactional reservation/handoff and guarded definite method-departure disposition are implemented and included in the passing resumed runtime controls. Complete result-bearing provider/bank/retained-owner finish above, custody through reap/I/O join and every cancellation/late-result/independent-handle control. Tokio reserve_owned/OwnedPermit::send supply queue transfer, not native settlement. Complete generated publication and process acceptance; delete replaced submission/shutdown and ignored-cleanup branches with migrated clients/server/fixtures. |
| E05 | in progress | E04; ADR-0243 precedes kernel and binding activation | Deliver capability-declared coherent batching with exact point identities and per-point outcomes. Integrate the current water implementation where it can reuse preparation across independent states and the action-capable provider control for demand fidelity. Mixed valid/invalid states, branch changes, nonfinite/truncated responses and failure after partial completion discriminate behavior. Remove redundant per-point RPC orchestration from migrated consumers. |
| E06 | planned | E02–E05 functional population; 65i product-use contract; 65f outcome and 65g readers | Integrate generated capability/outcome inspection and independent exchange across supported post-64 modes; `DynamicMessage::from_options` exposes metadata while native selectors govern use. Prove raw versus scientific access, original task/step/point coverage and final-attempt interpretation. Record matched direct/isolated complete-operation measurements without presumed speedup. Supply actual population/limits to 65h and remove superseded selected paths. |

The E05 prerequisites are a bounded implementation question: does the selected native provider
support coherent reuse across a group without changing point/state semantics? Inspect and test
that implementation before exposing the capability. If not, retain honest scalar-only support
for it and deliver the batch route through the independently specified action-capable control;
do not advertise production batching. The coordinator must then narrow the promised population
or select a genuinely useful batch-capable scientific implementation before declaring the
production batching obligation complete. A synthetic control alone cannot close that promise.

## E05 grouped implementation assessment and kernel contract

**Interface-checked:** the selected `coolprop-sys =8.0.0` exposes native array operations
`AbstractState_update_and_common_out`, `AbstractState_update_and_1_out`,
`AbstractState_update_and_5_out` and `PropsSImulti`. They are genuine grouped library routes,
not a Rust/RPC loop over scalar evaluation. The scalar `Provider::fill` and optional
`supports_batch`/`fill_batch` contracts retain separate capabilities; scalar support grants no
grouped support and the default batch implementation refuses without invoking scalar work.

The [selected CoolProp 8.0.0 C wrapper](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/CoolPropLib.cpp)
distinguishes these routes. The AbstractState array calls reuse a handle but swallow individual
point exceptions, leaving output slots unchanged; initialize every output slot to NaN and
require all requested fields before adoption. `PropsSImulti` instead constructs one state per
group, processes input arrays and accepts arbitrary selected outputs. Its caller supplies
matrix capacities; returned dimensions must match the admitted population/output shape,
and zero dimensions report total failure. Flattening is point-major. The output separator
uses actual `LIST_STRING_DELIMITER` configuration. The selected
[configuration declaration](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/include/CoolProp/detail/configuration_keys.h)
sets its default to comma. The initial admitted native profile uses that pinned default and
requires exactly six returned columns; configuration mismatch is a typed whole-group refusal,
without configuration mutation or replay. The selected Rust binding exposes no config getter,
and `get_global_param_string` does not expose configuration. This is a binding limitation,
not absence of upstream configuration introspection: the C++ `get_config_string` and
`get_config_as_json_string` owners in
[Configuration.cpp](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/Configuration.cpp)
are a future binding-eligibility lead; an actual supported exported C ABI still needs verification.

The [selected core implementation](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/CoolProp.cpp)
parses normal, first- and second-derivative output strings. `Phase` resolves through the
[native state owner](https://github.com/CoolProp/CoolProp/blob/ae81610e7d23efc57f9d051c8e70a4d66e87537f/src/AbstractState.cpp).
Candidate fixed outputs are `P`, `Hmolar`, `Smolar`, `Umolar`, `Phase` and
`d(P)/d(Dmolar)|T`; selected jets can use native derivative strings such as
`d(Hmolar)/d(T)|Dmolar`. Bad states/outputs can remain nonfinite while other outputs succeed;
global success never establishes per-point completeness. Exact derivative ordering and
homogeneous-phase applicability need independent controls before being advertised.

Use `COOLPROP.exclusive_access()` across `PropsSImulti` and any original `errstring` retrieval:
its total-failure path writes a process-global error slot. The
[coolprop-sys 8.0.0 access contract](https://docs.rs/coolprop-sys/8.0.0/coolprop_sys/struct.CoolPropLib.html)
requires this guard for global error attribution. It avoids repeating an already performed
group merely to recover its error. The guard excludes other in-process CoolProp readers during
the call; compare that physical tradeoff against native low-level grouping before claiming a
throughput benefit. No global application/RPC mutex or automatic retry is introduced.

The supported process profile also retains the authenticated image and no-global-mutation
premise: the current repository exposes no global CoolProp reference/configuration setters.
`PropsSImulti` creates a fresh HEOS state, and existing scalar `WaterFactory::create` likewise
creates a fresh state after factory preparation. Exclusive locking prevents simultaneous
mutation; it does not undo a completed external mutation. In particular, a common native h/u
shift can satisfy `h = u + p/rho` while violating the factory's original datum. Admitting
global reference/configuration mutation is an observable requalification trigger for both
fresh-state routes. Re-establish original anchor/datum meaning or assess the retained-state
grouped library route before claiming support for that expanded profile; no library-wide
configuration freeze is currently established.

The initial production capability is independent-state, value-only water batching
through one native `PropsSImulti` call, with derivative batch demands explicitly unsupported
until qualified. Factor shared physical admission at the actual water owner: finite input
envelope, actual phase, positive isothermal response, pressure envelope, local h/s datum
offsets and `h = u + p/rho` consistency must remain identical to scalar execution. The bulk
call must return those guard observations alongside selected values; ordinary bulk P/H/S
alone does not discharge them. Numerical kernels keep their native representations.

ADR-0243 precedes activation. The kernel/binding contract establishes:

- A capability query over the exact prepared demand; no default scalar-loop batch fallback.
- Whole-envelope identity uniqueness, framing and checked extents, followed by per-point
  scientific admission of exact raw inputs. Blanket recursive finite validation/adoption must
  not erase promised mixed-validity outcomes. Zero eligible points produce attributed refusals
  without entering the library.
- Stable original point identity/order with an eligible-point index mapping, one grouped native
  call and explicit per-point success/refusal. Malformed native dimensions or untrustworthy
  completion remain batch failures without claiming untouched/replayable points.
- Owned input/result storage with checked extents, accounting for C-wrapper copies/matrices
  and retained opaque provider admission. No response aliases transient native scratch.
- Cancellation/deadline checks before and after the non-cooperative foreign call, retaining
  native/process/I/O custody until actual settlement. An observer timeout cannot release buffers
  or turn missing acknowledgement into replay permission.

The implementation uses `ProviderBatchPoint` for original IDs, raw input cells and numeric
seeds, `ProviderBinding::evaluate_batch` for checked frame storage and independent point
shape/runtime admission, and `ProviderBatchOutcome` for attributed products/refusals.
Scope, output-envelope and observation wrappers forward one actual group; none synthesizes
a scalar loop. Water's one shared qualification helper serves both scalar and grouped paths.
The worker's source-owned `Prepared` response advertises this exact demand's native batch
capability; `EvaluateBatch` and `Batch` retain the original handle, shared invocation sequence
and original ordered point IDs. Raw `Vec<u64>` input bits survive generated validation until
native per-point admission. No transferable linearization proof is introduced.

**Tested, targeted on 2026-11-04:** following main-checkout generated publication
`28f1d32c0` and the test-only API/qualification correction `c3b03b891`, the original scoped
`scripts/pse test pse-runtime --lib --features property-coolprop` selection completed nine
controls with zero failures, pending tests or build warnings against the zero target. Actual
water transport preserved mixed raw NaN/envelope/phase refusals beside independent IAPWS
values, exact capability support/refusal, original point order, replay and duplicate-frame
refusal with surviving preparation, and native process/admission settlement. Active grouped
Stop waited for native return; malformed final-point shape, identity, absence and nonfinite
cells never published the first point's otherwise valid partial result. Binary raw-bit,
default-unsupported/no-scalar-fallback and the prior four E04 lifecycle controls also passed.
Original capture and settlement were complete under cache-off execution and actual Arrow
consuming-closure validation. Initial discovery failed on one unavailable test API and one
unnecessary qualification; the correction retained native active/population and admission
assertions. E05 remains in progress for remaining scientific populations, consumer adoption
and complete-operation measurement; these controls do not establish speedup or assembled H.

## E02 native portable descriptor cutover

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#e02-native-portable-descriptor-cutover). This heading remains for existing references.

### E02 complete passive Recipe source graph

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#e02-complete-passive-recipe-source-graph). This heading remains for existing references.

## E03 closed-root envelope cutover

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#e03-closed-root-envelope-cutover). This heading remains for existing references.

## Verification, extension and decision handoff

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#verification-extension-and-decision-handoff). This heading remains for existing references.

### E02 complete physical declaration exchange

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#e02-complete-physical-declaration-exchange). This heading remains for existing references.

### Remaining recipe qualification decision

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#remaining-recipe-qualification-decision). This heading remains for existing references.

### Native membership prerequisite integration

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#native-membership-prerequisite-integration). This heading remains for existing references.

### Library-owned pre-growth prerequisite

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#library-owned-pre-growth-prerequisite). This heading remains for existing references.

### Reconstruction population and native mathematics limit

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#reconstruction-population-and-native-mathematics-limit). This heading remains for existing references.

### Live admitted-body delegation

Substantive design, original evidence and remaining obligations moved without a status change to
[65k](65k-scientific-artifacts-and-native-reconstruction.md#live-admitted-body-delegation). This heading remains for existing references.

## Symbolica capability refinement — November 6

**Interface-checked:** review of the complete expanded
[Symbolica reference](../libraries/scientific_calcs/symbolica.md), the maintained Rust 3.0.1
supplier, Numerica 3.0.1 and the published Rust 3.0.2 source identifies the following
investigations for E01 capability classification and Plan64h original-equation assessment.
These refine mathematical implementation choices;
they do not claim an upgraded supplier, executed numerical benefit or broader qualification.

* **Exact identity recognition.** Rust 3.0.2 `AtomCore::{check_zero,check_zero_with,
  check_equal_with}` and `ZeroTestSettings` are candidates for source-class/affine recognition
  in `class_evidence` and `implicit_affine`. Positive identity can follow exact rational
  combination, supported trig/hyperbolic normalization and algebraic-field normalization;
  sampling never establishes zero. Only established identity may enlarge admitted affine
  scope. Inconclusive results preserve missing evidence; negative results involving opaque
  functions do not establish scientific nonlinearity. `check_equal_with` can exactify stored
  float operands before subtraction. Preserve original domain exclusions, parameter
  assumptions, coefficients and attribution. Local 3.0.1 `zero_test` has different semantics
  and is not a substitute. Before production selection, rebase the maintained supplier delta
  and establish bounded symbolic execution: sample/precision settings do not bound exact
  fallback cost, and this API supplies no cancellation callback. Characterization:
  [Rust 3.0.2 atom API](https://docs.rs/crate/symbolica/3.0.2/source/src/atom/core.rs),
  [algorithm/settings and upstream controls](https://docs.rs/crate/symbolica/3.0.2/source/src/atom/zero_test.rs).
* **Prepared precision diagnostics.** Reuse the existing exact `ScalarCache` evaluator and
  `ExpressionEvaluator::map_coeff_with_prec` to compare selected f64 values/partials with
  higher-precision `Float` values without repeating symbolic preparation. Select an actual
  cancellation or conditioning case and preserve its original physical comparison.
  `ErrorPropagatingFloat` supplies first-order significance propagation, not a rigorous fresh
  roundoff enclosure. Numerica `RealBall` certifies basic arithmetic but explicitly does not
  certify transcendental operations, and the current Symbolica evaluation domains do not
  provide a ready RealBall evaluator. Neither replaces the IBEX original-point uncertainty
  supplier for exp/log constitutive relations. Sources:
  [maintained evaluator](../../vendor/symbolica/src/evaluate/evaluator.rs),
  [existing prepared owner](../../crates/pse-math/src/library/admission.rs),
  [Numerica interval contract](https://docs.rs/crate/numerica/3.0.1/source/src/domains/float/interval.rs)
  and [error propagation](https://docs.rs/crate/numerica/3.0.1/source/src/domains/float/error.rs).
* **Selective exact blocks and stable formulas.** Compare Numerica `Matrix::{solve,
  solve_any}`/fraction-free solving and Symbolica `Atom::solve(...).over(...).wrt(...)` for
  small admitted rational/algebraic blocks; keep parameter specializations, branch conditions
  and expression-growth admission explicit. Existing parameter-dependent sparse affine and
  bounded cubic owners remain the reference implementation. `series`, `together`, `cancel`
  and polynomial conversion can derive stable constitutive forms, but finite series need
  remainder evidence for enclosure claims and canceled singularities retain original guards.
  Do not rationalize measured coefficients automatically. Sources:
  [maintained solve API](../../vendor/symbolica/src/atom/core.rs),
  [Numerica matrix algorithms](https://docs.rs/crate/numerica/3.0.1/source/src/tensors/matrix.rs)
  and [existing affine realization](../../crates/pse-math/src/implicit_affine.rs).

Already-used multi-output evaluation, direct alias DAGs, CPE/Horner optimization, sparse
derivative selection, Dualizer products, provider pullbacks and compiled scalar execution
are existing leverage, not newly missing implementation. `merge` checks parameter count,
not semantic argument correspondence; use only with an owner-established mapping and an
actual preparation benefit. Local Rust's portable API is `export_instructions`; Python
`get_instructions`/save-load terminology does not supersede K's checked f64 transfer owner.
Local `nsolve_system` is a possible small high-precision diagnostic: its separate residual
and dense derivative evaluators, plain Newton steps and absolute-step stop supply neither
bounds/globalization, original task cancellation nor a root certificate. It is not selected
as the primary large native solver. Sources:
[optimizer](../../vendor/symbolica/src/evaluate/optimize.rs),
[instruction API](../../vendor/symbolica/src/evaluate/instruction.rs) and
[numerical system implementation](../../vendor/symbolica/src/solve.rs).

The original physical-interior refusal is addressed by ADR-0248's point-local eligibility
contract, independently of engineering acceptance budgets. Keep Estimated evidence limits;
additional symbolic capabilities do not establish a finite correction neighborhood or root
existence. Each selected further increment needs a real affected consumer, source-backed
mechanism, targeted controls, deletion of its replaced path and the existing execution owners.

**Conditional option, not selected for the current branch refusal:** a complete prepared
128/256-bit comparison observer could supply explicitly Estimated arithmetic provenance for
a broader supported population. Pinned 3.0.1 already provides `map_coeff_with_prec`,
`Float::from_rational_round(..., precision, RoundingDirection::Nearest)` for admitted real
coefficients and `Float::with_val(precision, actual_f64)` for exact stored binary64 inputs
at precision at least 53. `Float::from(f64)` fixes precision at 53 and is unsuitable for this
sweep. Registered constants must be recomputed at the requested precision; unsupported or
complex coefficient/domain cases must refuse rather than discard components. Sources:
[coefficient mapping](../../vendor/symbolica/src/evaluate/evaluator.rs),
[Float evaluation domain](../../vendor/symbolica/src/evaluate/domain.rs) and
[Numerica precision constructors](https://docs.rs/crate/numerica/3.0.1/source/src/domains/float/multiprecision.rs).

This is not a per-block substitution for current enclosure vectors. Retain/reuse the exact
selected derivative preparation before `bounded_evaluator` converts it to f64, and carry
higher precision through the existing regional stage/chain-rule schedule, binding transforms,
duplicate accumulation and original-bound subtraction. Compare output values/gradients and
residuals/Jacobian entries on corresponding original branches. Replaying opaque f64 provider
answers or passing rounded f64 intermediates assesses only part of the arithmetic. The
current scheduler/assembly and f64 byte estimates need explicit precision-aware realization
and MPFR limb accounting; library instruction evaluation has no cancellation checkpoint
inside a block, and registered-constant conversion currently contains an unwrap. See
[derivative conversion owner](../../crates/pse-math/src/library/admission.rs),
[regional execution](../../crates/pse-math/src/execution.rs) and
[original assembly](../../crates/pse-math/src/assembly.rs).

Any selection requires a prior evidence-contract decision that distinguishes outward
enclosure from observed precision discrepancy and estimated convergence. Compute differences
before conversion, retain nonzero underflow truthfully, bind source/point/branch/order and
precision/backend identities, and withhold unresolved cancellation or branch disagreement.
Agreement at two precisions is not an error bound or exactness proof. Do not populate the
current enclosure-specific `SquareResidual` fields silently or allow an enclosure-requiring
consumer to accept weaker provenance. Prefer an exact predicate/projection repair when it
supports the concrete population with materially less execution and interpretation burden.

## Sources

- [Scientific discovery and exact current owner links](../library_leverage/libraries/proto/briefings/math-solvers-and-scientific-workers.md).
- [Buffa 0.9.2 encoding sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs) and [ConnectRPC 0.9.1 clients](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs).
- [Maintained validation contract](../../vendor/protoc-gen-protovalidate-buffa/README.md), [local RPC reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) and [Buffa reference](../libraries/proto/Buffa.md).
- Native solver interpretation uses the active native-solver-libraries reference; no solver release/ABI move is selected by this plan. Source pointers: `crates/pse-runtime/src/math/native_child.rs`, `scientific_worker/{protocol,proxy,transport,process}.rs`, `crates/pse-kernels/src/{lib,portable}.rs` and `crates/pse-compiler/src/typed_math_portable.rs`.
