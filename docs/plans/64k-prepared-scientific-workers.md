---
title: "Plan64k: Prepared local scientific workers"
status: in-progress
date: 2026-11-03
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md]
adrs: [ADR-0240, ADR-0238]
---

# Prepared local scientific workers

## Target, authority and baseline

**Proposed:** deliver a working, explicitly prepared, persistent local scientific worker and
a synchronous provider proxy that runs the existing CoolProp water implementation in that
owned process. This is required implementation scope, including actual equipment use and
failure settlement; a protocol specification or design probe alone cannot complete it.
The [coordinator](64-process-simulator-target.md) owns combined authorization, review
dispositions and decision/lifecycle integration. This companion owns K01–K06 and realizes
[Proto review I04](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I04).
The 2026-11-03 instruction authorizes plan creation; it does not authorize this new production work.

**Interface-checked:** inspected HEAD is `2550fa665c8b5fcffb88447f6818b372c5a62df5`, plus
concurrent capture changes in `pse-operations`' canonical capture/workbench/native-service
owners and D–G companion closeout edits. Those changes and their evidence remain at their
owners. Core3.4, efficiency heuristics1.0, ProcessSimulator1.5 and test-assessment1.2 apply.
This authoring inspected source and library interfaces; it ran no tests, builds or measurements.

The operator accepted RC01–RC04 on 2026-11-03, including breaking delivered-scope revisions; the coordinator records their decisions once. RC04's local binding route is proposed
ADR-0240 and blueprint §9.11 before dependent implementation; ADR-0238 remains the water
science binding. RC01/RC02 projection and version changes consume 64i and proposed ADR-0239.
Local worker selection grants no remote-deployment or distributed-solving permission.

## Inspected foundations and responsibility boundaries

| Owner | Existing contract and consequence |
| --- | --- |
| [`pse-kernels`](../../crates/pse-kernels/src/lib.rs), [`provider.rs`](../../crates/pse-kernels/src/provider.rs) | `ProviderSpec`, `ProviderFactory::create_scoped`, `PreparedProviderRequest`, `ProviderBinding::fill` and caller-owned completion already distinguish physical meaning, demanded products and attempt custody. Keep them authoritative. |
| [`coolprop.rs`](../../crates/pse-backend-native/src/coolprop.rs) | `descriptor`, `WaterFactory::prepare`, `ProviderFactory::create` and `registration` supply authenticated HEOS Water, a fresh owned `AbstractState`, ordered T/molar-density inputs and p/h/s outputs. Invoke this factory inside the worker; do not copy its science or C ABI adapter. |
| [`coolprop/tests.rs`](../../crates/pse-backend-native/src/coolprop/tests.rs), [`steam_tests.rs`](../../crates/pse-runtime/src/workflow/modeling/steam_tests.rs) | Existing IAPWS values, datum, derivative, phase and authored-equipment controls supply comparison cases, not fresh worker qualification. |
| [`native_child.rs`](https://github.com/paul-heyse/pse-arrow/blob/9de6d7d94e22890bf2c8ba73c2d0b47e0d700562/crates/pse-runtime/src/math/native_child.rs), [`native_thread.rs`](../../crates/pse-runtime/src/native_thread.rs) | Existing native thread/process custody retains admission through actual join/reap. Its closed provider-free one-shot root protocol is a different placement; reuse suitable ownership primitives, never route provider calls through a whole-root solve. |
| [`semantic/rpc.rs`](../../crates/pse-host/src/semantic/rpc.rs), [typed authored-world generation](../../xtask/src/codegen/code_intelligence.rs) | Existing ConnectRPC/HTTP2 over authenticated Unix streams, generated services and bounded decoding are integration patterns. Scientific execution must not depend on a semantic-analysis session or host-private policy. |

64i owns source-derived contract projections and compiled local constraints; give it the provider-specific field and operation requirements in K01. K02 needs the implemented narrow
projection/generation slice, not all of 64i. 64j's engineering service is an independent
sibling: neither its dispatcher nor a foreign engineering client is a worker prerequisite.
64d D02 owns water science and equipment definitions; 64h owns wider reference correspondence.
K05 qualifies placement substitution for selected cases and does not close those populations.

Put the proxy, transport integration and process/session custody in existing `pse-runtime`
owners, with a small `pse-scientific-worker` executable in that same crate. Keep generic
provider semantics in `pse-kernels` and water implementation in `pse-backend-native`.
No new crate, required Python product, canonical database or general worker framework is
selected. Reconcile the generated-output ownership with 64i before publishing its first outputs.

## Prepared operation and scientific fidelity

Preparation consumes the actual admitted registry/physical interpretation, `ProviderSpec`,
factory configuration, exact executable/native image and data identities, requested products,
placement selection and original execution scope. The child reconstructs and admits actual
physical meaning and prepares `WaterFactory`; comparing descriptor digests alone is insufficient.
Return an attempt/session/generation-bound handle only after both sides agree on the exact
contract and actual product support. Descriptor inspection and launch never build, download
or implicitly prepare a missing executable/native artifact.

The selected first worker is the existing liquid/vapor/supercritical HEOS Water factory,
using `coolprop-sys =8.0.0` and the existing `property-coolprop` → backend `coolprop` feature.
Preserve its actual loaded-version/revision and image authentication, Water JSON/data identity,
regime selection, fitted gas constant/molar mass and ideal-only reference anchor. T is K,
density is mol/m³, p is absolute Pa, h is J/mol and s is J/(mol·K), resolved through actual
quantity/unit identities. Record exact datum conditions and offset bits; retain the identical
h/u shift. Global reference-state mutation, phase imposition, wet-state derivative claims
and wider fluid support remain outside this binding's admitted science.

One evaluation sends a prepared handle, invocation identity, ordered input bits and runtime
directions/cotangents/weights, and returns one coherent complete requested product or typed
refusal. Immutable descriptors, datum and layout are prepared once; do not send/reinterpret
their complete descriptions in each callback. Keep all five demands distinct:

| Demand | Required boundary meaning and first-water support |
| --- | --- |
| `CompleteJets` | Values and output-major First/Second partial arrays in existing kernel order; the first water implementation supports Value, First and Second. |
| `SelectedColumns` | Ordered unique input columns and, at Second, their selected square; current water worker explicitly refuses. |
| `JacobianAction` | Direction-major input seeds and output-major forward products; current water worker explicitly refuses. |
| `TransposeAction` | Selected-output cotangents and cotangent-major input products; current water worker explicitly refuses. |
| `WeightedSecondAction` | Selected-output weights, one weighted gradient and input-major second actions; current water worker explicitly refuses. |

Transport all variants and preserve support/refusal at preparation; never emulate a requested
action with a full matrix, numerical differences, zeros or scalar RPC fan-out. A later shared
water-owner capability can replace a refusal only with matching in-process and worker evidence.
One scalar-valued property may be a selected output of a coherent call; it is not a scalar-RPC
API. Multi-state batching is not required: adding it needs separate point identity, state reset,
ordering and independent point-failure semantics; repeated single-state `fill` is no batch claim.

Use binary Protobuf with packed `fixed64` IEEE-754 bit arrays for numerical payloads and exact
datum/configuration bits; apply `to_bits`/`from_bits` without decimal or unit normalization.
Preserve signed zero in transport, then enforce actual finite/domain rules at their owners.
Shapes, selected-output/column order, derivative source/order/smoothness, interpretation and
implementation/data revisions remain explicit. Unsupported versions/variants, unknown required
meaning, duplicate ordinals, extent overflow or wrong capabilities refuse before native work.
Remote output must pass exact `ProviderExtents`, complete-fill, finite/envelope and current
scope checks before `ProviderBinding` exposes a completion; partially written/stale buffers
never become readable success. Typed trial/domain, contract, resource, cancellation, deadline,
terminal infrastructure and unknown-completion outcomes retain attribution across the boundary.

`ProviderLinearizationToken` stays owner sealed and is never serialized into transferable proof.
The current water worker declares no linearization support and returns no token. For an
action-capable implementation, the local owner may seal only after declared capability and
re-establishment of the current actual attempt/session/generation, input, branch/inner-solve
and child context against the expected context. Shape and completion cannot establish that
premise. Wire context references are evidence inputs, not tokens; stale or unestablished
context refuses or yields no token. Reconnect/replacement invalidates every prepared context.

## Local placement, supervision and resource lifetime

Use an explicitly selected executable with authenticated local Unix-stream custody and
ConnectRPC unary evaluation; no network listener or ambient provider-name dispatch is needed.
Bind the expected child process identity and executable observation during preparation, verify
kernel peer credentials at every connection, and reject another session/process even if its
schema matches. Own the endpoint/channel and launch nonce; do not trust caller-supplied PID
metadata or allow reconnect to attach to an unrelated process. Child parent-loss handling
must terminate native work; process isolation is containment, not a hostile-code sandbox.

The synchronous proxy executes on the existing native blocking lane. An independently driven
async I/O owner runs ConnectRPC; child native calls also run outside async executor threads.
Keep one active evaluation per mutable attempt binding, with independent attempts eligible
for parallel execution through existing shared admission. Do not serialize independent attempts
globally or add a second uncoordinated CPU pool. Override `create_scoped` to retain the original
`ExecutionScope`: `EvaluationContext` alone has cancellation and result allowance, not a deadline.
Queueing, launch/admission, transport, native evaluation and cleanup share that original scope;
derive RPC timeouts from remaining time without restarting the clock.

The initial unreleased v1 protocol represents an unbounded original scope by absence of
`Prepare.remaining_ns`; a present positive value is the original remaining allowance.
The child establishes a finite deadline once and never restarts it on evaluation. This
corrects the draft's implicit 600-second lifetime before first publication; no accepted or
deployed client requires compatibility. Earlier fixed-budget qualification snapshots retain
their original limits and do not establish this corrected deadline behavior.

Lifecycle is prepare → ready → evaluating → ready, or stopping → drained → closed/failed.
Cancellation/deadline stops admission and signals the actual evaluation; a transport future
being dropped is not native settlement. Retain the caller's custody and admission until work
stops, or terminate the owned child and wait/reap it. Drain must have a finite escalation
policy under the existing supervisor; preserve the original timeout/cancellation cause and
any cleanup failure. A native crash, lost connection after dispatch or late reply cannot
publish success: mark completion unknown where appropriate, invalidate the binding and reap.
No automatic evaluation replay, session restoration or retry after ambiguous loss is selected.
A fresh explicit preparation may start another attempt; it cannot renew old proof or deadlines.

Admit known request/response/scratch/decoder storage before allocation and account for live
copies, prepared metadata, process/native opaque allowances, stacks, queued and active work.
Keep leases through actual drain/reap, not RPC completion. Reuse the maintainer's shared
unbounded/explicit finite placement policies; protocol/result bounds are not host RAM or CPU
caps. Distinguish accounted bytes from total foreign/RSS memory; refuse a demanded strict
whole-child bound if the current native allocator contract cannot establish it.

## Library mechanisms and bounded investigations

**Interface-checked:** Context7 found Buffa but no matching Rust ConnectRPC library after
two scoped resolutions; exact installed 0.9.1/0.9.2 source supplied the consequential APIs.
The [briefing](../library_leverage/libraries/proto/briefing.md), [ConnectRPC overview](../libraries/proto/ConnectRPC_for_Rust.md)
and [Buffa overview](../libraries/proto/Buffa.md) orient the following selected mechanisms.

| Mechanism | Concrete realization, condition and source |
| --- | --- |
| ConnectRPC `=0.9.1` | Enable `client` at the proxy and `server` at the helper; current workspace default `gzip`/`zstd`/`json` features remain declared, but science uses binary messages. Generated service trait/extension and client use `Router`, `ConnectRpcService`, `Http2Connection::connect_with_connector`, `ClientConfig` and `CallOptions::with_timeout`; [released client API](https://docs.rs/connectrpc/0.9.1/connectrpc/client/index.html). Disable invocation replay despite connection recovery facilities. |
| Buffa `=0.9.2` | Existing workspace `json`/`text` features do not select a scientific JSON path. Generated messages use `Message::encode_to_vec` and `DecodeOptions` size/recursion/unknown-field bounds; [released runtime](https://docs.rs/buffa/0.9.2/buffa/). 64i owns source-derived mechanical projections; method/session records add placement meaning only. |
| Existing generator/validation closure | Extend the typed Buffa/Connect and maintained Protovalidate-Buffa `=0.7.2` [composition](../../xtask/src/codegen/proto_projection/composition.rs) through 64i. Generate local field/oneof/length rules, then use kernel/physical admission for context. The actual admitted configuration and maintained validator preparation bound supported rules; unsupported CEL never silently passes. |
| Tokio `=1.53.3`, Hyper `=1.11.1`, hyper-util `=0.1.21` | Existing `rt-multi-thread`/`sync`/`time`/`process`/`net`/`io-util`, Hyper `client`/`server`/`http2` and hyper-util `tokio` features support the independent I/O driver and local HTTP2 composition. Reuse `UnixStream::peer_cred`, `TokioIo`, the existing native-thread join and owned-child reap patterns. |
| CoolProp C ABI `8.0.0` | Existing factory owns `AbstractState_factory/update/keyed_output/first_partial_deriv/second_partial_deriv/free`; [tagged C ABI](https://github.com/CoolProp/CoolProp/blob/v8.0.0/include/CoolProp/CoolPropLib.h) and [low-level API](https://coolprop.org/coolprop/LowLevelAPI.html). The worker calls this adapter instead of recreating it. |

K01 must settle source-derived provider projection wiring with 64i, the actual executable
preparation/profile route, authenticated channel ownership and finite stop escalation against
the selected supervisor. These are bounded integration investigations with implemented
conformance criteria, not an option to defer worker delivery. Source-scoped limitations may
change local plumbing or require a shared contract correction; escalate those decisions before
dependent code. Retain the selected water science and working-worker target. In-process-only
execution is simpler and remains the default control; persistent isolation buys foreign-failure
containment and interchangeable placement at the cost of process/transport lifetime and callback
latency. Remote scheduling, a new scientific model and a universal worker framework add no needed
capability to this controlled delivery.

## Package state

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| K01 | in-progress | Execution authorization; proposed ADR-0240 decision route; actual kernel/water owners and 64i contract agreement | Settle exact field, support/refusal, scope/context and process preparation contracts. Add owner-local tests for malformed demand, incompatible physical/data/datum identity and undeclared linearization; coordinate any shared provider correction with 64i/64d. |
| K02 | in-progress | K01; implemented 64i narrow generation/validation slice | Source-owned protocol, generated clients/services and checked mappings cover preparation/evaluation/stop/close with typed failures. Targeted codec/conformance tests attack exact bits, absent/unknown alternatives, shape/order/extent overflow and preserved diagnostic classes; regenerate through `just codegen`. |
| K03 | in-progress | K01 process/profile decision; K02 working transport | Deliver explicit preparation, authenticated owned process, persistent session, sync proxy and independent I/O/native lanes. Targeted actual-process controls establish wrong-peer refusal, original deadline, cancellation/late response, crash/disconnect/unknown completion, no replay, context invalidation and actual reap with leases held. |
| K04 | in-progress | K02/K03; current 64d D02 factory available | Deliver the actual water helper calling `WaterFactory`, admit the same configuration and integrate explicit registration/placement selection through existing runtime ingress. Targeted direct/proxy tests cover repeated coherent CompleteJets calls, all three orders and regimes, every other demand's honest refusal, changed data/datum/branch and outside-envelope cases. |
| K05 | in-progress | K04; selected authored pump/turbine closures and independent controls available from 64d | Run targeted selected-case controls during integration; then qualify complete in-process/worker equipment journeys, original equations/balances/domains, independent water/derivative evidence and interruption. Measure the complete matched solve and repeated-case route, cold and warm, including preparation/transfer/drain and retained memory; no benefit claim from a codec benchmark. |
| K06 | planned | K01–K05 functional scope implemented; root scope-end timing | Root/integrator runs selected assembled integration/component/native journeys, hygiene and manual checks once at scope end, coordinates 64h evidence, publishes enduring contract/operator/extension guidance and removes replaced choreography. Close only with working prepared launch, actual worker equipment evidence and truthful performance/qualification limits. |

Packages are integration units, not automatic agent assignments. K03 need not wait for unrelated 64i projections or 64j service delivery; K04 can use ready K03 process slices.
During implementation compile touched packages and run targeted tests via `scripts/pse check`
and `scripts/pse test`, with selected native features and explicit force-validation wherever
Arrow is consumed. Root owns `just turn-end`; do not insert hygiene or broad campaigns per
package. K06 selects actual commands after the changed closure is known and reports command,
mode, scope, zero failure baseline and result; no plan-authoring product qualification is due.

## Current implementation checkpoint — 2026-11-04

The source-owned worker protocol, generated transport, authenticated process custody,
independently driven I/O and synchronous native-lane proxy are integrated. The actual
WaterFactory supplies CompleteJets in liquid, vapor and supercritical regimes, retaining
its physical/data/configuration identity and honest refusal of unsupported demands.
An asymmetric action-provider control exercises all five native action products without
dense substitution or a manufactured linearization token.

The earlier 21-test snapshot passed and recorded complete-call timing/accounting. Independent
review then identified the draft's implicit 600-second binding lifetime and early release
of the combined reservation during failure cleanup. Source corrections retain the original
optional deadline and hold admission through both child reap and I/O join; deterministic
long-lived-deadline and delayed-I/O controls are added. The expanded 25-test campaign now
passes, including optional wire/range validation, original-scope RPC timeouts and reordered
action products. Complete-call timing/accounting was recorded again with actual child reap
and zero retained reservation afterward. The independent process integration now passes
its three original-process controls: complete water jets across orders/regimes/repeated
calls, original refusal/cancellation without stale success, and admission retained through
actual kill/reap. The latest integrated equipment results are recorded below; their matched
complete-operation measurement campaign remains open. Both earlier campaigns are single-campaign observations,
not a speedup claim or independent validation of CoolProp; original host/library-load and
accounted-versus-RSS limitations remain explicit in their retained source recordings.

K01–K04 therefore contain implemented and targeted tested protocol, process and water-proxy
slices, but their complete package acceptance and final generated-world reconciliation
remain open. K05 is not complete. A fresh scientific-input capsule supplies both the
captured physical package and the actual authored steam/oracle closure. The three direct
equipment cases previously reached a nominal `VolumeFlow` mismatch for `flow/rho`; the three
worker cases refused a hard-coded standard-registry profile before execution. The earlier
document-ingress case lacked its captured physical type declarations. The integrated correction
below carries the actual unadmitted physical declarations through generated preparation and
native builder admission with exact descriptor/context checks, preserving nominal meaning.

The historical targeted `pse-runtime --lib` steam/inspection selection with `engineering-service,
native-study-tests,product-reader,property-coolprop,solver-kinsol` completed nine controls:
one passed and eight failed against zero failures. Seven failures are the scientific cases
above; the inspection byte-reencoding assertion incorrectly assumed canonical protobuf map
ordering and has been corrected to check wire and native meaning, pending its rerun. Original
capture/native settlement completed. These results do not establish equipment execution,
placement equivalence or benefit. Earlier missing-data/predicate failures remain historical.

Joint execution at main `83485d6f6` consumes Plan65 E04/E05's current shared protocol:
process-wide explicit Shutdown, handle-scoped Stop/Close and capability-declared EvaluateBatch.
Their targeted controls and publication supersede the earlier protocol assumptions; K01–K04
must be reconciled against these owners rather than creating another worker/batch path.
Those controls do not discharge the equipment, independent-science or matched-measurement
obligations below.

The November 5 shared lifecycle also consumes provider/bank/native owners explicitly,
preserves independent action and retirement failures, and initiates peer-led HTTP/2 drain
after actual native Shutdown. [Plan65e](65e-scientific-portability-and-execution.md#transactional-handoff-and-explicit-placement-finish--november-5-amendment)
owns that contract; [65h's receiving checkpoint](65h-integration-and-qualification.md#source-integration-wrap-up--2026-11-05)
owns its current targeted acceptance and limits. This handoff adds no equipment qualification.

Main `165faf093` now integrates the physical-context and authored-operation correction.
Generated Prepare v2 transports the actual passive registry builder and provider declaration;
the child uses native builders, reconstructs its original context and compares the descriptor
before creating a provider. Old tag/name reservations prevent silently adopting the former
standard-registry assumption. The parent retains the registry declaration and its reservation
through factory/binding lifetime. Generated Request/Response adapters replace the handwritten
wire forest; malformed response adoption remains unknown completion, distinct from an actual
native provider refusal. Numerical-job admission is released after calculation/control
settlement and before reply publication, removing the next-request busy race.

The actual Water provider now supplies native SelectedColumns first/second derivatives through
CoolProp's `first_partial_deriv`/`second_partial_deriv` machinery, preserving requested column
and output order and shared support checks. It computes requested derivatives rather than
manufacturing an action token or substituting dense action products. Unsupported action
demands remain refusals. See the selected
[CoolProp 8.0.0 native interface](https://github.com/CoolProp/CoolProp/blob/v8.0.0/include/AbstractState.h)
and implementation in `pse-backend-native/src/coolprop.rs`.

Targeted pre-integration native Water controls passed 12/12, zero failures/build warnings.
The preceding runtime iteration passed 37/44: its seven failures identified selected-column
support, the calculation/reply admission race, finite diagnostic extent and bounded solver
eligibility. The integrated corrections address those causes in source, but final runtime
and actual-process rebuilds were interrupted by storage exhaustion before executing tests.
The final-source actual-process selection now passes on main:
`scripts/pse test pse-runtime:scientific_worker --features
engineering-service,native-study-tests,product-reader,product-operators,property-coolprop,solver-kinsol,solver-ipopt`
passed 3/3, zero failures/build warnings/pending records, original capture and settlement
complete, configured unbounded/cache-off with compiler/KLU/solver explicitly prepared.
It exercises CompleteJets and selected first/second columns with output/column permutations
across all three water regimes, original refusal/cancellation without stale success and
actual kill/reap admission retention. Direct and worker routes use the same CoolProp science;
this is final-source process/representation evidence, not independent validation of that
science. The fresh main capsule selection also passed all 44 runtime controls with zero
failures, build warnings or pending records and complete original capture/settlement:
`scripts/pse test pse-runtime --lib --features
engineering-service,native-study-tests,product-reader,product-operators,property-coolprop,solver-kinsol,solver-ipopt
-E 'test(scientific_worker::) | test(workflow::modeling::steam_tests::)'`, in the same prepared
unbounded/cache-off native mode. It includes all six authored direct/worker equipment cases
and registered document ingress. Original hydraulic/IAPWS and released single-phase reference
expectations passed without changing their numerical tolerance or bounds. This discharges
those failures at their selected scientific states, not full K05, other fluids/wet states,
the matched measurement campaign or wider Plan64 family acceptance.

Next work is broader independent
water/datum/derivative and original equipment-balance evidence; interruption and settled
cleanup; and the required matched cold/warm complete equipment measurements. The earlier
single-call observations do not satisfy that full campaign. K06's assembled acceptance,
scope-end checks, operator/extension guidance and enduring publication remain planned.

## Verification, migration and completion

**Tested, local Linux after explicit native/worker preparation:**
`scripts/pse test pse-runtime --lib --features engineering-service,native-study-tests,product-reader,property-coolprop,solver-kinsol -E 'test(scientific_worker::)'`
passed 25 selected tests, and
`scripts/pse test pse-runtime:scientific_worker --features engineering-service,native-study-tests,product-reader,property-coolprop,solver-kinsol`
passed all three actual-process tests. Both used the native correctness route and have
zero failures against zero; unselected runtime tests were not exercised. They cover the
source-described process/custody and water-demand controls, not a full equipment campaign.
The earlier equipment failures and the latest passing selection are recorded by
[64f](64f-specialized-process-families.md#current-steam-integration-checkpoint--2026-11-04).

**Proposed verification:** distinguish bit-preserving interchange from independent science.
Matched placements deliberately share CoolProp and therefore cannot discover its common model
or datum errors. Use independently transcribed [IAPWS R6-95(2018), Table7](https://iapws.org/public/documents/k3PSK/IAPWS95-2018.pdf)
controls with documented mass/mole and datum mapping, centered derivative observations at
admitted interior states, thermodynamic identities and original equipment balance checks.
Keep reference expectations independent of the wire mapper; retain the source's fitted
constants rather than silently substituting newer physical constants.

Conformance uses an explicitly bound action-capable provider to exercise every transported product
and context mismatch independently of water's declared refusals. Inject malformed, truncated,
nonfinite, oversized and reordered replies; unsupported action/order combinations; changed
regime/data/interpretation; foreign or stale expected context; concurrent attempt isolation;
and cancel/crash/disconnect before and after native dispatch. Assert no stale completion,
automatic replay, leaked child, prematurely released lease or async executor blockage.

Keep both in-process and worker-backed registrations because they are distinct selected
placements of one science, not old/new compatibility paths. Migrate any replaced launch,
request-mapping or cleanup choreography and all its consumers together, then delete its code,
tests and fixtures when replacement controls pass. Preserve existing one-shot native-root and
canonical study workers whose responsibilities remain distinct; do not rename them into this
contract. Later science families add a source/data/datum/derivative/domain implementation at
their owner and reuse this placement contract; changed science needs its own evidence.

Completion requires all K packages, the selected actual-process/equipment journey and failure
settlement evidence, matched complete-call measurements without a presumed speedup threshold,
and enduring documentation at the coordinator's decision-owned architecture/operator routes.
No pass qualifies wet states, other fluids, remote execution or the wider Plan64 population.
At closure record **Implemented**, named **Tested** and **Measured** conditions separately,
the mistake corrected and deliberate deviations; retire this companion through the coordinator
only after its enduring meaning and inbound references have owners.

**Current conclusion:** Full execution is authorized. The worker's generated transport and
actual-process controls have been exercised after explicit preparation following the earlier
disk cleanup. Selected equipment controls now pass; remaining acceptance is described above;
actual-process transport is no longer wholly unrun. K01–K05 remain in progress and K06
planned. Full scientific acceptance and a performance benefit remain unestablished.
