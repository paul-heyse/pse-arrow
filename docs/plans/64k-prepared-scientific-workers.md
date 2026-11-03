---
title: "Plan64k: Prepared local scientific workers"
status: planned
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
| [`native_child.rs`](../../crates/pse-runtime/src/math/native_child.rs), [`native_thread.rs`](../../crates/pse-runtime/src/native_thread.rs) | Existing native thread/process custody retains admission through actual join/reap. Its closed provider-free one-shot root protocol is a different placement; reuse suitable ownership primitives, never route provider calls through a whole-root solve. |
| [`semantic/rpc.rs`](../../crates/pse-host/src/semantic/rpc.rs), [`buf.gen.yaml`](../../crates/pse-host/proto/buf.gen.yaml) | Existing ConnectRPC/HTTP2 over authenticated Unix streams, generated services and bounded decoding are integration patterns. Scientific execution must not depend on a semantic-analysis session or host-private policy. |

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
| Existing generator/validation closure | Extend the declared `protoc-gen-buffa`, `protoc-gen-connect-rust` and maintained Protovalidate-Buffa `=0.7.2` routes through 64i. Generate local field/oneof/length rules, then use kernel/physical admission for context. `buf.gen.yaml` and the maintained validator README bound supported rules; unsupported CEL never silently passes. |
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
| K01 | planned | Execution authorization; proposed ADR-0240 decision route; actual kernel/water owners and 64i contract agreement | Settle exact field, support/refusal, scope/context and process preparation contracts. Add owner-local tests for malformed demand, incompatible physical/data/datum identity and undeclared linearization; coordinate any shared provider correction with 64i/64d. |
| K02 | planned | K01; implemented 64i narrow generation/validation slice | Source-owned protocol, generated clients/services and checked mappings cover preparation/evaluation/stop/close with typed failures. Targeted codec/conformance tests attack exact bits, absent/unknown alternatives, shape/order/extent overflow and preserved diagnostic classes; regenerate through `just codegen`. |
| K03 | planned | K01 process/profile decision; K02 working transport | Deliver explicit preparation, authenticated owned process, persistent session, sync proxy and independent I/O/native lanes. Targeted actual-process controls establish wrong-peer refusal, original deadline, cancellation/late response, crash/disconnect/unknown completion, no replay, context invalidation and actual reap with leases held. |
| K04 | planned | K02/K03; current 64d D02 factory available | Deliver the actual water helper calling `WaterFactory`, admit the same configuration and integrate explicit registration/placement selection through existing runtime ingress. Targeted direct/proxy tests cover repeated coherent CompleteJets calls, all three orders and regimes, every other demand's honest refusal, changed data/datum/branch and outside-envelope cases. |
| K05 | planned | K04; selected authored pump/turbine closures and independent controls available from 64d | Run targeted selected-case controls during integration; then qualify complete in-process/worker equipment journeys, original equations/balances/domains, independent water/derivative evidence and interruption. Measure the complete matched solve and repeated-case route, cold and warm, including preparation/transfer/drain and retained memory; no benefit claim from a codec benchmark. |
| K06 | planned | K01–K05 functional scope implemented; root scope-end timing | Root/integrator runs selected assembled integration/component/native journeys, hygiene and manual checks once at scope end, coordinates 64h evidence, publishes enduring contract/operator/extension guidance and removes replaced choreography. Close only with working prepared launch, actual worker equipment evidence and truthful performance/qualification limits. |

Packages are integration units, not automatic agent assignments. K03 need not wait for unrelated 64i projections or 64j service delivery; K04 can use ready K03 process slices.
During implementation compile touched packages and run targeted tests via `scripts/pse check`
and `scripts/pse test`, with selected native features and explicit force-validation wherever
Arrow is consumed. Root owns `just turn-end`; do not insert hygiene or broad campaigns per
package. K06 selects actual commands after the changed closure is known and reports command,
mode, scope, zero failure baseline and result; no plan-authoring product qualification is due.

## Verification, migration and completion

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

**Current checkpoint / outcome:** Proposed only. First executable work is K01 after explicit implementation authorization and the binding route; no worker, new scientific capability,
test result or performance benefit is claimed by this plan.
