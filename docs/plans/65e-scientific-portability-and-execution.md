---
title: "Plan65e: Scientific portability and execution"
status: proposed
date: 2026-11-03
adrs: [ADR-0242, ADR-0243, ADR-0245]
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
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
completion. This companion owns E01–E06. Execution is authorized alongside incomplete Plan64
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

Replace the private root's mechanical Serde source/request/outcome vocabulary with generated
source-owned records, keeping its existing narrow executable factory and parent-side original
screening. Reuse its private bounded channel rather than forcing it through the engineering
service. The implementation removes the old wire structs, conversion pairs and JSON framing
after complete root controls pass. It does not widen the supported mathematical class implicitly.

A universal Proto mathematical IR would duplicate library algebra and introduce conversions
into hot callbacks. Keeping every private wire form indefinitely would repeat scientific
interpretation as families grow. The selected mixed design removes repeated portable
correspondence without making a codec the numerical engine. Proto-first external scientific
contracts remain eligible when independently governed, through explicit native adoption.

## Worker and coherent-batch interface

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

## Packages and migration

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| E01 | in-progress | Actual native capabilities; agreed scientific contract from 65i I01; co-designed 65a metadata | Initial concrete portable consumers are reconciled above; complete source/spec/context/receipt classification and bind each operation's inputs, support/refusal and product-use semantics before selecting projections. Preserve physical/configuration versus value-only change controls; no universal catalogue. |
| E02 | in progress | E01; narrow working 65a/65b and receiving operation from 65i I02 | First remove kernel portable descriptor mirrors while retaining contextual receipt replay and truthful interpretation changes. Generate portable scientific records/adapters and migrate existing producers/readers. Buffa generation and supported `Validate` discharge local correspondence; native reconstruction and completion-owned selectors discharge scientific admission/use. Cover ordering, exact bits, missing/unknown alternatives, scope/coverage and refusal classes. Remove migrated capture/restore pairs and duplicated shape declarations; retain qualification. Test changed native variants against independent known meaning. |
| E03 | in progress | E02 closed-root slice; 65d transport/settlement primitives where shared | Project actual native root/input/source-map/handshake/request/outcome owners, then activate generated bounded envelopes on the existing inherited channel. Preserve provider-free rejection, original registry/body checks, numerical policy and cancellation/drain. Compare direct and child outcomes on the same independently specified roots; inject malformed source, stale image, unknown completion and child failure. Delete replaced private wire/JSON-only helpers and their mechanism-only fixtures. |
| E04 | in-progress | Existing provider slice; 65d custody behavior; 65i dependency policy; ADR-0242 | Source cutover separates per-handle Stop/Close from explicit Shutdown and generation cancellation. Complete generated publication, active-stop/independent-session controls and process/I/O drain acceptance; migrate all selected clients/server/fixtures without confusing mutable handle invalidation with immutable math or descriptor lifetime. |
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

The first E02 slice replaces `PortWire`, `ShapeWire` and `SpecWire` in the kernel portable
owner with the actual `Port`, `ProviderShape`, `ProviderShapes` and `ProviderSpec` declarations.
`ProviderRecord` v2 contains the native descriptor directly. Existing Serde 1.0.229
[`Serialize`/`Deserialize` derives](https://github.com/serde-rs/serde/blob/v1.0.229/README.md)
encode those named fields without a second native correspondence layer;
[`deny_unknown_fields`](https://serde.rs/container-attrs.html#deny_unknown_fields) refuses
unknown struct meaning instead of silently discarding keys. This uses the existing bounded
JSON recipe route during this narrow migration; it does not claim the remaining Buffa
scientific-input cutover is complete. Passive bidirectional source projection describes
untrusted reconstructible data. Only native contextual receipt replay and exact provider-key
comparison issue `AdmittedProvider`; no decoder constructs a factory or executable admission.

The representation change is explicit: shape arrays become native named shape fields, raw
derivative ordinals become the native enum, ProviderRecord advances to v2, admitted recipes
to `pse.admitted-body.v3`, and runtime product envelopes/framing to v5. Bounded readers check
interpretation headers before decoding representation-specific fields, so unsupported
retained interpretations report incompatibility rather than masquerading as the current shape.
The existing exact-artifact framing algorithm, provider semantic key and native scientific
identity remain unchanged; new representation bytes naturally have a new artifact hash.
Qualified receipt indexing and original replay authority remain intact. Shared shape accounting
charges vector capacities and nested allocations exactly once, including retained spare slots.

**Tested, targeted on 2026-11-04:** scoped cache-off `scripts/pse test pse-kernels pse-compiler
pse-runtime --lib` selected and completed nine controls with zero failures or pending tests
against the zero target, with original capture/settlement and actual Arrow validation complete.
They cover real provider receipt reconstruction and independent `x*x+1` execution/refusal,
changed descriptor-key refusal despite qualified byte membership, decoder refusal to issue
receipt authority, unknown-field refusal, native record Data/version/direction metadata,
shape capacity accounting, provider-free direct replay, retired recipe/envelope interpretations,
and large opaque envelope hashing/forged extent. The initial seven-control population passed;
a new source-projection test's misplaced module caused one discovery build failure and was
corrected before this full nine-control rerun. Root-owned generated publication remains.
Compiler-owned Recipe/source/physical co-declarations are subsequently delivered in the source
graph slice below. Remaining E02 includes actual generated receiving adapters and complete
dependency/context reconstruction. E03 replaces private JSON serialization of
root input, source bindings and handshake/request/response on the existing bounded inherited
channel, retaining provider-free admission, original image/build/registry checks, qualified
recipe authority, original-coordinate screening and actual process custody. No kernel/ID owner
acquires a Buffa dependency merely to encode this transitional descriptor representation.

### E02 complete passive Recipe source graph

The final scientific worktree handoff is integrated on main as `f0cba8780`.
`pse_compiler::typed_math::scientific_recipe_projection()` now returns the source-owned graph
of the actual private 17-field `Recipe`. Resolved syntax, support, selected-provider records,
physical/mathematical receipts, native `BodySpec`, applicability and nominal body identity
participate through their actual passive owners. The graph factory admits or restores nothing;
it exposes no live Scheme, modeling Type, provider handle or qualified replay authority.
Source presentation version one does not change the active JSON recipe interpretation three.

**Tested, targeted on main:** `scripts/pse test pse-compiler --lib -E
'test(scientific_recipe_source_graph_) | test(scientific_recipe_native_projection_)'`
passed 2/2 with zero failures, pending tests or build warnings, in configured unbounded,
cache-off mode with prepared compiler/KLU and complete original capture/native settlement.
The controls check complete Data-only source closure and actual admission of an authored
`doubled(x)=x+x` fixture, exact existing JSON after native field reconstruction, and refusal
to restore syntax without original qualification. A worker's unnecessary-qualification warning
was corrected before the clean main rerun. These controls do not exercise a protobuf Recipe.

Next, publish the selected private wire family and borrowed adapters from the same actual
`ProjectionOutput`, descriptor set and generation configuration. Do not generate a second
descriptor world for adapter naming. Full binary decode/adoption, monomorphic same-root
membership receivers, complete original workspace admission and restoration bounds, and the
early original-function/formal-count check remain required. The existing JSON/qualification
route stays active until those replacements and their refusal/drain controls are accepted.

## E03 closed-root envelope cutover

The private v2 profile is `pse.private-native-root.v2`, with source-generated package
`pse.native_root.v2`. It retains the existing bounded u64 little-endian inherited Unix-channel
framing, one closed provider-free solve per child, original attempt/execution/request bindings,
relative remaining lifetime, original-coordinate parent screening and process custody. There
is no new service or process. `RootInput`, `RootBindings`, `Hello`, `Request`, `Response` and
`RootOutcome::{Proposal,Refused}` are the actual owners; proposal decoding does not imply
scientific acceptance. Their graph exposes native math controls, source locations, complete
model diagnostics and passive applicability observations without wire mirrors. Recipe and
registry declaration bytes remain opaque in this first envelope slice; receiving compiler/
physical reconstruction still owns their admission and the existing qualified recipe permit.

Build/file observations use their original native fields. Standard `PathBuf` projection carries
an explicit `PlatformPathBytes` alternative: Unix bytes through
[`OsStringExt::into_vec`/`from_vec`](https://doc.rust-lang.org/stable/std/os/unix/ffi/trait.OsStringExt.html),
or Windows wide units through
[`encode_wide`](https://doc.rust-lang.org/stable/std/os/windows/ffi/trait.OsStrExt.html) and
[`from_wide`](https://doc.rust-lang.org/stable/std/os/windows/ffi/trait.OsStringExt.html).
These APIs preserve non-UTF-8 bytes or ill-formed UTF-16; no Unicode/path normalization or
filesystem effect occurs. Foreign platform data stays independently inspectable; checked
native `PathBuf` construction refuses another platform. The source profile remains local,
with the actual authenticated child image and current native platform.

Buffa 0.9.2
[`Message::try_encode_bounded`](https://docs.rs/buffa/0.9.2/buffa/trait.Message.html#method.try_encode_bounded)
checks encoded size against the caller's allowance before writing, using the shared two-pass
size cache; use its original `ExceedsBudget`/`MessageTooLarge` failure rather than a JSON
counting serializer or a panic-capable encoder. The selected release source supplies this API;
the broad guide's older one-error summary is not its full API contract.
[`DecodeOptions`](https://docs.rs/buffa/0.9.2/buffa/struct.DecodeOptions.html) supplies separate
frame bytes, repeated-element memory, recursion and unknown-field bounds. Generated local
validators/adapters establish structural meaning, while the original source/registry/factory
constructors retain scientific admission. Original typed codec/adoption errors must survive
the private-channel Internal classification with their field/rule/cause details and custody.

**Tested, targeted on 2026-11-04:** scoped, direnv-activated unbounded cache-off
`scripts/pse test pse-contract pse-runtime --lib` discovered and completed the pure-path
control, zero failures/pending/build warnings, with original capture/settlement complete.
The initially expected graph control was excluded because its implementation module is
solver-gated. Actual passive declarations and graph now live in the ungated
`math::native_root_protocol` owner; the gated implementation consumes that same owner and
retains its contextual constructors. The corrected default-feature, direnv-activated unbounded
cache-off `scripts/pse test pse-runtime --lib` source-graph selection separately completed
1/1 with zero failures/pending/build warnings and complete original capture/settlement.
This qualifies the graph's default-feature generation closure, not native solve execution.
The Linux path control preserves non-UTF-8/NUL bytes and refuses native
adoption of inspected Windows-wide data; Windows execution is not qualified. Generated
publication and execution were subsequently exercised by the following envelope cutover;
this initial graph control alone does not qualify native execution.

**Implemented:** actual private root input, source bindings, build/file observations,
handshake, request and proposal/refusal now use source-generated adapters, generated local
validation and bounded Buffa. The JSON counting/frame helpers, protocol Serde derives and
broken-serializer fixture are deleted. Encoding consumes owned records; framing does not
clone recipe, registry or numerical buffers. The existing 16 MiB request / 1 MiB response
limits remain; decoding additionally bounds element memory to four times its frame bound,
recursion to 64 and retained unknown fields to 256. Unknown admitted meanings are refused
after decoding rather than silently normalized. Structural adapters live separately from
private channel functions; native data and generated messages remain inspectable without
KINSOL preparation. Crate-visible private adapter entry points are the selected generator
scope, published on main. The first envelope snapshot below precedes the complete physical
declaration cutover and its later main-branch controls.

Original Buffa encode/decode and generated adoption/validation causes survive inside
`NativeRootWireError` and existing typed runtime custody. Codec byte/element/depth/unknown
allowance exhaustion is ResourceLimit; admitted-channel corruption is Internal. Shared
crossing-error retained-extent accounting includes original vector/path/string capacities
and map selectors, with checked overflow refusal. These structural checks grant no replay
authority or scientific permission. Original problem/options validation precedes encoding,
and the parent retains original-coordinate screening after decoding the child's proposal.
Raw candidate floating-point bits, including nonfinite values, remain representable before
that scientific screening. Existing native-profile hashing and receipt interpretation are
unchanged. The first envelope snapshot left registry JSON and admitted recipe bytes as staged
leaves; the physical declaration cutover below removes registry JSON. The admitted recipe
continues through its original qualification interpretation pending the remaining decision.

**Tested, targeted on 2026-11-04:** genuine user scope, approved direnv unbounded mode,
cache off, explicit compiler/KLU preparation and force-validation through
`scripts/pse test pse-runtime --lib --features solver-kinsol` with
`(test(math::native_child::) & !test(private_native_fake_process)) | test(private_protocol_retains_original_codec_and_rule_causes_with_owning_classes)`
completed 17/17 with zero failures or pending work and complete original capture/settlement.
This includes the exact external native child on independently specified roots, original
residual screening and receiving source maps, provider/physical-context refusals, foreign
attempt/build/request checks, corrupt/truncated/oversized frames, process death/reaping,
queued original clocks, cancellation/late-output charges, readiness and the truthful strict
complete-memory refusal. Independent codec controls preserve binary64 payload bits and
refuse unknown fields/missing outcome; original typed rule causes remain inspectable.
The source-built child uses the existing `pse-native-lane` executable with the narrow xtask
`solver-kinsol` feature; `native-solvers` retains its aggregate meaning. Its explicit original
artifact SHA256 was `9aa7ac6fe49296d566c2dfc9a3857814869a8a2381de43dc0dbd09a733932501`.
The first feature check exposed missing adapter imports and unused imports, corrected before
this run. This run retained eight generated Public-to-Crate visibility warnings; the scoped
generator publication and its clean compile remain integration obligations, not a zero-warning
claim. A separate default-feature typed-error control also passed 1/1. No measured speedup,
complete native RSS bound or full E02/E03 closure is claimed. Remaining work includes full
compiler-owned recipe/physical record conversion, contextual receipt reconstruction and
the coordinator's assembled integration/qualification population.

## Verification, extension and decision handoff

### E02 complete physical declaration exchange

**Implemented and generated on main:** `QuantityRegistryBuilder` and its
complete declaration closure are source-projected at their native owners, including defined
units, derived kinds, conversions, operation policies, optional unit-set positions and ordered
reduction domains. Declaration vectors preserve duplicates for original atomic `build()`
admission. Physical prerequisite predicates and declarations are passive typed data; the
original `PhysicalPreconditions::new` reconstructs their ordered inventory. Neither decoding
nor a declared invariant identity grants scientific permission.

Private profile `pse.private-native-root.v3` carries the actual builder and prerequisite
declarations in `RootInput` interpretation 3. `pse.native_root.v2` remains the generated
namespace: this authenticated ephemeral channel has no retained reader or compatibility
promise, and original-image/profile checks reject an old executable. There is one current
writer/reader. Tag 5 and `registry_digest` are reserved after removing the redundant leaf
checksum. The original request/image binding authenticates exact input bytes; the receiving
owner independently rebuilds the registry and compares the complete native
`physical_identity(registry, prerequisites)` against original `BodySpec.physical` before
minting recipe replay authority. Frame/hash algorithms and scientific semantic keys do not
change. The registry JSON serialization/parser and its mechanism-only fixture are removed;
exact compiler recipe JSON remains intentionally authoritative pending the decision below.

**Tested, targeted:** unbounded, cache-off `scripts/pse test pse-quantity --lib --features fixtures`
completed the complete registry/prerequisite source control 1/1, zero failures, pending work
or build warnings. It preserves exact existing declaration bytes, nonempty prerequisite
values and native physical identity, retains duplicate declarations for `registry.unique_id`
refusal, and distinguishes a valid changed scale despite unchanged declaration identities.
The first test build used an unavailable default constructor (three errors), corrected to
the actual native `new` constructor before this pass. The independent default-feature
`scripts/pse test pse-runtime --lib` complete protocol-graph control separately passed 1/1
with zero failures, pending work or build warnings. These are source-owner/graph checks;
new generated codec activation and actual v3 child execution remain required. Prepared
receiving controls cover nonempty prerequisites, duplicate units, changed unit scale,
changed prerequisite meaning and missing prerequisites through the generated codec.

**Tested, targeted on main, 2026-11-04:** after source-built native-lane publication with
explicit compiler/KLU preparation, `scripts/pse test pse-runtime --lib --features solver-kinsol`
using the 17-control filter above passed all 17, zero failures/pending/build warnings, configured
unbounded mode and cache off, with original capture and native settlement complete. The actual
`PSE_NATIVE_LANE_EXECUTABLE` artifact SHA256 was
`b46a1a1dc5a34cc4a02a3744674fc49e446edcef449af8e320ad7c570697d47f`.
The selection includes the generated typed-registry/prerequisite round trip, original physical
identity and duplicate/scale/prerequisite refusal cases, alongside actual native child execution
and original process/error controls. It resolves the earlier generated visibility warning
obligation for this profile. Full recipe conversion, complete E02/E03 population, scientific
extension and assembled H qualification remain open; no measured speedup or full RSS claim follows.

### Remaining recipe qualification decision

Full protobuf recipe conversion changes more than encoding: `QualifiedScientificRecipe`
currently parses exact JSON to build the scoped structural membership index, and nested
restore/replay requires exact record membership in addition to original request/qualification
binding and complete receipt consumption. Current gates protect physical contract/boundary/
inference/scheme records, physical and mathematical receipt streams, provider descriptors and
prepared calls, selected provider joins, direct graph/function/support products, physical
evidence and nominal type/value records. Raw protobuf fields cannot replace their native
membership interpretation by accident.

**Selected prerequisite direction, not accepted for full cutover:** prefer DOM-free native Serde traversal
preserving the existing structural membership V1 over adding a second typed membership
visitor. Serde's `Serializer::Ok` and compound serializer traits support native traversal
without a JSON value tree; the exact pinned `serde_json` number, map-key, variant and
surviving-subtree semantics remain part of the consumed contract. The
[independent scoped review](../design_review/reviews/design_review_plan65_native_replay_membership_2026-11-04.md)
now accepts the prerequisite architecture at Proposed strength in its follow-up assessment.
[Proposed ADR-0245](../adr/0245-bound-native-recipe-membership.md) records RC01/RC02 before
affected authority/hash implementation: monomorphic exact-byte decode-once receivers index
and consume the same native root; deferred postorder range removal preserves overwritten-map
survival; source-derived lazy-probe/eager-view/complete-validation/native-adoption/index/receipt/
restoration phases use the original workspace. The selected adapter is borrowed eager-view
adoption with preadmitted complete validation, rather than an unspecified owned/sink alternative.
Future admitted protobuf interpretation is `pse.admitted-body.v4`; exact encoding changes its
digest and binding without silently redefining structural V1. Unsupported integer/raw sentinel
normalization refuses explicitly. F01–F03 remain open for implementation and original evidence;
full protobuf activation remains Not Accept. The old JSON byte multiplier does not establish
these new bounds. A versioned owner/tag/bit membership scheme remains a different semantic
option, requiring deliberate complete-consumer migration rather than a claim of V1 preservation.
Retain exact qualification/request binding and complete original receipt draining; ordinary
decoded records must still lack authority. A staged DOM-backed native-root index is an
alternative only if explicitly selected. A whole-payload permit without
per-record membership would remove existing protections and is unsuitable. A new native-root
qualification premise or changed membership/hash interpretation requires the coordinator's
proposed ADR before implementation. Neither an artifact digest nor successful local wire validation
may mint this authority. The existing recipe leaf remains unchanged until a complete approved
replacement and its original reconstruction/refusal controls are in place.

The initial F03 envelope remains unresolved. The complete native Recipe graph is now available;
its generated wire profile, retained capacities and pre-admission bounds still require realization.
The selected source-derived traversal can use Buffa 0.9.2
[`decode_lazy_with_ctx`/`merge_lazy`, fragments and raw elements](https://docs.rs/buffa/0.9.2/src/buffa/view.rs.html)
with one shared decode context. Default subtree getters recreate allowances. The
[decode limits](https://docs.rs/buffa/0.9.2/src/buffa/message.rs.html) do not bound packed-scalar
allocation, eager oneof/map subtrees, spare capacity or reallocation overlap. Some retained
capacities lack public accessors; a qualified source-profile bound or small library-owned
accounting interfaces must precede allocation, rather than infer admission from post-decode
length. Reuse actual descriptor naming and the maintained validator's selected rules for
complete violation/path and uniqueness scratch bounds; no fail-fast/sink substitution exists.
Native adoption includes staging vectors overlapping final collections, original membership/
receipt lifetimes and repeated formal restoration expansion. This source inspection establishes
interfaces and missing realization work, not a measured memory bound or full cutover readiness.

The [Serde 1.0.229 `Serializer` contract](https://docs.rs/serde/1.0.229/serde/ser/trait.Serializer.html)
allows a native summary as `Ok`; `SerializeSeq`, `SerializeMap`, `SerializeStruct` and their
variant counterparts expose elements, entries and completion without requiring a value DOM.
Preservation must follow the [serde_json 1.0.151 native value serializer](https://github.com/serde-rs/json/blob/v1.0.151/src/value/ser.rs)
and [number representation](https://github.com/serde-rs/json/blob/v1.0.151/src/number.rs), including
the actual root's parsed-JSON versus native-value numeric behavior. These are enabling library
interfaces; they do not supply the qualification or surviving-membership contract themselves.

Use source/admit → encode → independent decoder → receiving adoption → actual execution →
outcome/readback journeys. Independent scientific expectations must not come from generated
adapters or the candidate provider. Retain physically independent water/derivative controls
and original equipment balances. Exact derivative actions are checked against independently
constructed asymmetric cases that expose transpose/order mistakes.

Preserve 65i's spatial-without-temporal-closure refusal through reconstruction; use an existing
supported dynamic model for positive preparation and independent inspection. Test a locally
valid derived product with failed governing coverage and a nonselected accuracy attempt.
Raw readers may expose them while ordinary scientific selection refuses or qualifies use.
Interface-only changes must not rebuild unchanged mathematics; changed datum/data/applicability
must invalidate or explicitly readmit affected science despite identical wire shape.

Challenge adding a physical provider capability, a solver outcome and a time/axis-bearing
result without editing unrelated transport code. New science still needs its domain model,
validity/derivative support and evidence. No new fluid model, solver algorithm, remote execution
or distributed decomposition is implied by these structural extension controls.

Targeted implementation tests run with the repository's pinned native routes and explicit
force-validation for Arrow closures. Broad campaigns and hygiene belong to series scope end,
owned by [65h](65h-integration-and-qualification.md). Wire fidelity, actual placement,
scientific correctness and measured benefit are separate claims.

Protocol/kernel/metadata changes follow the coordinator's proposed decision route before
implementation. This plan is not ADR acceptance or current qualification. At closure report
Implemented, named Tested and Measured conditions separately; migrate enduring meaning to
the scientific architecture owners and retire completed planning through the repository route.

## Sources

- [Scientific discovery and exact current owner links](../library_leverage/libraries/proto/briefings/math-solvers-and-scientific-workers.md).
- [Buffa 0.9.2 encoding sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs) and [ConnectRPC 0.9.1 clients](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs).
- [Maintained validation contract](../../vendor/protoc-gen-protovalidate-buffa/README.md), [local RPC reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) and [Buffa reference](../libraries/proto/Buffa.md).
- Native solver interpretation uses the active native-solver-libraries reference; no solver release/ABI move is selected by this plan. Source pointers: `crates/pse-runtime/src/math/native_child.rs`, `scientific_worker/{protocol,proxy,transport,process}.rs`, `crates/pse-kernels/src/{lib,portable}.rs` and `crates/pse-compiler/src/typed_math_portable.rs`.
