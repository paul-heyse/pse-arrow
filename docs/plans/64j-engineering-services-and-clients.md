---
title: "Plan64j: Engineering services and clients"
status: in-progress
date: 2026-11-03
doc_role: plan
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [current-work, design, validation]
review_sources: [docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md]
adrs: [ADR-0239]
---

# Engineering services and clients

## Target and boundary

**Proposed:** deliver a working authenticated local engineering service and an independent
generated Rust client for contract discovery, selected physical inspection, typed exact-parent
edits, preparation, finite study submission, original-operation control and protected readback.
This companion owns the [Proto improvement review](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I03)
I03 and its runtime/client I06 contributions. The [coordinator](64-process-simulator-target.md)
owns combined adoption, finding disposition, decisions and final Plan64 acceptance. This is
delivery scope, including the real client journey; a descriptor or interface prototype cannot close it.

The maintainer selected all six review recommendations and accepted review-local RC01–RC04
on 2026-11-03. RC03's effectful service and optional foreign client contract route through
proposed ADR-0239 and the coordinator's architecture revision before dependent implementation.
RC01/RC02's source-derived projection and breaking interpretation changes are consumed from
[64i](64i-engineering-contracts-and-validation.md); RC04's local worker is owned by
[64k](64k-prepared-scientific-workers.md). ADR proposal is not acceptance or execution evidence.
Delivered B/C representations may be replaced with their callers and fixtures in the same change.
The client requires no Python product; foreign-language bindings are optional later consumers.
Plan authoring does not start production implementation; execution retains its authorization boundary.

Authoring baseline is `2550fa665`, with concurrent 64d–64g and capture corrections. Those
sources and their qualification claims are moving. Core 3.4, efficiency heuristics 1.0,
ProcessSimulator 1.5 and test-assessment 1.2 apply. This document records a selected target
and planned evidence; source reading does not qualify the service or scientific workflows.

## Foundation assessment and actual consumers

`crates/pse-runtime/src/host_operations/project.rs` already declares `ProjectAction`,
`ProjectOperationRequest` and `EffectFacts`. Its scientific dispatch currently lives in
`crates/pse-runtime/src/bin/project/runtime.rs`, alongside file reading, presentation and
runtime cleanup. Factor reusable typed handlers into the runtime library and migrate this
CLI to them. The engineering service calls those same handlers, admission, preparation,
completion and publication owners. It must not interpret scientific definitions or implement
a second workflow engine. Operational study browse must retain its existing ability to avoid
unneeded scientific construction; do not make every read prepare a scientific runtime.

`workflow/modeling/documents.rs`, `project_preparation_wire.rs`, `workflow/completion.rs`
and `workflow/portable_payload.rs` supply edit, preparation, permission and product semantics.
`pse-operations::canonical_study_workbench` owns protected study selection/capture/readback;
`native_service.rs` owns configured canonical-service custody. Extend these owners only for
missing consumed behavior. The independent host remains opaque to scientific policy.
The governing contracts remain blueprint §19, §20, §21.1, §21.5 and §21.7.

Actual migration covers the project runtime CLI dispatch and conversions, the new effectful
service, its generated Rust client and supported client examples/fixtures. It also covers
selected preparation discovery, modeling inspection/declaration pages, study observations,
cohort/result reads and their producing runtime/operations paths where I06 removes duplicate
projection. Existing host `product_reader.rs` and `cli/product.rs` Query/Investigate/Follow
remain passive; they acquire no edit, preparation, submission, capture or cancellation effect.
64h owns reference-exchange producers/comparators and I05; 64i owns shared generation and
producer-selection contracts. Those consumers are coordinated, not duplicated here.

## Library realization and selection

Use the existing Buffa/ConnectRPC family. Current workspace pins are `buffa =0.9.2`
(`json`, `text`), `buffa-descriptor =0.9.2` (`reflect`, `json`, `text`),
`buffa-types =0.9.2` (`reflect`, `json`), `connectrpc =0.9.1` and
`protovalidate-buffa =0.7.2`. The existing host generation configuration selects Buffa
`reflection=true`, `reflect_mode=vtable`, `shared_descriptor_pool=true`, packaging,
`protoc-gen-connect-rust` and the maintained validator plugin. 64i extends the owning
generation route; this plan consumes its messages, descriptors, validators and service/client
bindings rather than keeping hand-maintained Rust/wire twins. Dependency declarations stay
in Cargo; intentional additions/changes are exactly pinned and scoped lock changes are reviewed.

Select generated service traits with `RequestContext`, `ServiceRequest<'_, M>`,
`ServiceResult<M>` and server-streamed `ServiceStream<M>` observations. Use
`Router`/`ConnectRpcService` with the existing authenticated Unix/Hyper hosting pattern;
the consumed transport requires ConnectRPC `client`/`server` features, which are distinct
from its default codec/compression features. Do not add Axum, TLS or companion health/reflection
packages merely to host this local boundary. Health/reflection, if later selected, report
their own facts and cannot establish scientific readiness.

Generate the independent client's service binding and use
`client::Http2Connection::connect_with_connector` with an authenticated connector, following
`pse-host/src/semantic/rpc.rs`. `ServiceRequest::to_owned_message()` gives retained actions
their own data; `to_owned_view()`/Buffa `OwnedView` can retain existing encoded metadata.
`MessageView` and `ViewEncode` serve selected binary reads where they remove rebuilding.
`to_owned_view()` repeats a decode walk and retains backing bytes; view response bodies are
protobuf-only in this release, so JSON consumers need their supported owned representation.
Do not serialize native preparation or Arrow execution just to introduce borrowed views.

The [ConnectRPC overview](../libraries/proto/ConnectRPC_for_Rust.md),
[Buffa overview](../libraries/proto/Buffa.md) and
[discovery briefing](../library_leverage/libraries/proto/briefing.md) are awareness inputs.
Selected API evidence is the [0.9.1 guide](https://github.com/connectrpc/connect-rust/blob/v0.9.1/docs/guide.md),
[0.9.1 runtime](https://docs.rs/connectrpc/0.9.1/connectrpc/), installed 0.9.1
`Cargo.toml`/`src/request.rs`/`src/client/http2.rs`, and [Buffa 0.9.2 views](https://docs.rs/buffa/0.9.2/buffa/view/index.html).
Context7 resolved no matching Rust library; its [protocol documentation](https://connectrpc.com/docs/protocol/)
supports protocol semantics only. Go examples and development-only APIs are not Rust API evidence.
Direct linked typed calls remain useful for native consumers. A second RPC abstraction or
wholesale Tonic migration adds mappings without advancing this target. Unary commands and
server-streamed observations suffice; full-duplex/browser request streaming is unnecessary.

## Operation, connection and lifetime contracts

Describe-contract returns immutable schema identity, complete descriptor references, semantic
links and owner-declared effects/directions. Selected inspection separately returns actual
revision/context, quantities, units/bases/datums, selected providers, capabilities and refusals.
Static discovery grants neither admission nor scientific permission. Exact-parent edit requests
consume 64i's additions/replacement positions/removals/retain semantics and original source,
document/fit ownership; stale parents and incompatible physical selections refuse atomically.

Preparation admits the selected case and requested capabilities through existing native owners.
Study submission carries finite cases, requested products, exact source/context and the owning
operation's request identity. Return its original operation/run identity and retain it before
the client relies on acknowledgement. Same-identity repetition follows that owner's actual
repeatability contract; changed request under the same identity refuses. Nonrepeatable capture
or export cannot be declared safely replayable by an RPC annotation. Ambiguous acknowledgement
resolves status/receipt at the original owner; it never silently creates another computation.

Run submission binds the actual native preparation identity as well as its service-local
operation name. That identity participates in the retained whole-request fingerprint and
must match a live preparation before dispatch. An exact original envelope can resolve its
canonical registration after service retirement without reconstructing preparation; a new
preparation with the same local name cannot silently substitute different science. The
initial unreleased v1 adds this explicit field before first publication; the earlier draft
without it is not an accepted compatibility contract.

Status, cancellation and retained products address the original identity. Cancellation requests
cooperative stop through the existing owner; accepted cancellation is distinct from native
settlement. Diagnostic-only structural outcomes, failed siblings, unavailable products and
scientific refusal remain inspectable with original completion permissions. Transport success
cannot promote them to usable scientific states. Preserve constraint violations with structured
field/rule/source identity, validator compilation/evaluation failures, contextual admission,
scientific refusal/failure, resource limits, cancellation, incompatible interpretation,
infrastructure failure and ambiguous effect as distinct categories through typed details.

Service start/preparation is explicit. Connect/read/discovery neither builds nor launches it
or a configured database. Authenticate the prepared service's actual executable/invocation
custody on every initial connection and reconnect before HTTP/2 negotiation; socket spelling,
UID or peer PID alone is insufficient. The listener admits local clients from kernel peer
credentials and explicit local access policy before effectful dispatch, never caller-supplied
identity metadata. Bind the session to its actual checkout/service incarnation. Replaced or
stale owners refuse; descriptor identity is not a credential. Use existing managed ownership
and connection machinery without importing analyzer-only policy into product semantics.

Progress is observational, scoped to retained original facts and supported cursors. Stream
disconnect, dropped future, deadline and missing heartbeat do not prove stop or settlement.
Reconnect reauthenticates and reads original status/products, reporting expired observation
coverage explicitly; it does not resubmit. The execution/service owners retain custody through
cooperative joins or owned termination/reap, including shutdown and partial failure. Slow
observers have bounded buffering/backpressure and cannot retain unbounded scientific state or
block unrelated cancellation. No durable generic resume engine is introduced.

## Selection, materialization and capture

Carry requested fields/pages/products into the real producer using 64i's shared selection
contract before construction/encoding. Metadata schema identity and selection are separate
from scientific completeness. Compact owned responses can release a large backing buffer;
borrowed views can retain it. Choose at the consumed boundary and keep ownership visible through
encoding, consumption, cancellation and release. Selected/full/empty reads preserve count,
continuation and refusal semantics. Lazy nested decode failures remain failures.

Inspect `modeling/documents.rs::bound_observation` with 64i: it currently measures serialized
extent before returning the value. Remove a duplicate pass only when the selected encoder
preserves refusal before publication and pre-allocation/resource protections. No larger bounds,
partial successful output or omitted capture members may make the optimization pass.
G08 in [64g](64g-analyses-studies-and-applications.md#g08--atomic-capture-and-unrelated-liveness)
remains the sole atomic membership/protection/receipt transaction owner. Streams/pages and
service adapters consume its protected capture; they cannot replace it with observer lifetime.
Final persisted study/service closeout consumes G08's implemented writer coverage, lost-ACK
replay and original-owner qualification, even if early nonpersistent service work is ready.

## Package state

Every package starts **planned**. The coordinator owns cross-plan disposition; these rows own
J progress. Local acceptance names intended tests, not tests already written or run.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| J01 | in-progress | Co-design 64i's first project action, preparation/inspection and scoped completion contracts; agree operation identities/effects and error categories before bindings harden. | Shared typed library handlers replace binary dispatch; the CLI uses them. Owner-local handler/CLI tests challenge read without scientific construction, source/context refusal and identical typed outcomes. Delete migrated binary dispatch/conversion choreography and its mechanism-only tests. |
| J02 | in-progress | J01 plus 64i's generated service projection, constraints and typed edit slice. | Explicit authenticated local hosting and an independent generated Rust client work. Local transport tests reject stale/replaced/unauthorized peers on initial and reconnected connections, and verify no read implicitly prepares. Delete replaced service-specific wire mappings; retain distinct native JSON presentation only where consumed. |
| J03 | in-progress | J02 plus working native edit/preparation/study handlers. | Inspect/edit/prepare/submit/status/cancel/read products work end to end. Typed owner/client tests include add/replace/delete/retain, stale parent, missing/default distinctions, intended science failure, duplicate original request and changed-request conflict. Remove replaced B/C adapters/callers/fixtures together. |
| J04 | in-progress | J03's original operation ownership and retained reads. | Observational progress, reconnect, bounded observer release and shutdown work. Drop observation while work continues, reconnect to the original run, lose a submit ACK, request cancellation and prove actual settlement separately. No automatic submit replay; remove duplicated polling/status translations where replaced. |
| J05 | in-progress | 64i's producer-selection slice plus J03/J04 real responses. | Runtime/client I06 producer/encode/consume paths use selected responses. Selected/full/empty and malformed nested read tests cover continuation, retained buffer release, extent refusal, cancellation and partial output. Delete duplicated projection/extent passes only with equivalent protection. No latency/memory benefit claim without whole-lifetime measurement. |
| J06 | planned | All functional J scope, working selected 64d–64g cases and consumed G08 capture/publication slices. | Independent client delivery is qualified and handed off. Assembled inspect/edit/prepare/study/readback journey includes original physical checks, failed siblings, protected products, disconnect/reconnect, duplicate/cancel/lost ACK and shutdown. Delete any remaining replaced route; reconcile support and permissions with 64h H05/H07. |

J01 and 64i's first contract slice proceed together; neither waits for a universal catalogue.
J02 can develop against that implemented slice while later schema families proceed. J03's
scientific journey needs working selected native handlers, not merely generated interfaces.
J05 selection changes coordinate shared producer files with 64i and their 64h consumers.
64k worker execution is independent of this service; adding another engineering product or
provider availability field should project its existing owner without a new dispatch engine.
An E/F distributed/cyclic case challenges actual axes/events/capability inspection and retained
products; advertising an event never implies unsupported integrator sensitivities.

## Current implementation checkpoint — 2026-11-03

Shared library handlers now serve the project CLI and generated engineering transport.
Source admission, exact-parent file edits, selected declaration headers, original source-file
reads, full physical inspection, preparation discovery and declared-case preparation/run
have working native owners. The descriptor world and source-owned effect declarations
reach the independent client. Connections authenticate the original managed service payload;
retained native tasks own work independently of a dropped transport waiter. Status,
cancellation, progress and selected Arrow IPC reads address original run identities.

Focused local controls cover source retention, unknown ingress, outgoing validation,
multibyte byte bounds, pending cancellation, progress and shutdown. The separately managed
service/client journey now exercises original source revisions, exact parent changes,
lost-submit-ACK reconnect, original run/attempt matching and protected Arrow readback.
Its numerical assertion reads the original run's resolved variable and engineering
contexts: the authored characteristic magnitude and shared relative fraction establish
the coordinate error budget. The square fixture's analytical positive-root error is
compared with that budget; native convergence is not an output-error certificate.
The prepared frontend and its original canonical fixture owner both completed orderly
shutdown. These results cover the declared algebraic slice; full Study acceptance remains open.
Independent review identified self-referencing dependency/shutdown, preparation-name reuse,
same-PID executable replacement, ephemeral result-attempt matching and aggregate ingress
accounting gaps. Corrections and focused controls remain in this implementation slice;
the earlier passing controls do not establish those corrected behaviors.
The initial run frontend covers declared algebraic cases; actual StudyRequest preparation,
issued canonical study creation, all serializable operation families, bounded study/cohort
products and native modeling/provider inspection remain required J scope. An algebraic
journey does not close these obligations or qualify the broader D–G scientific families.

## Verification and completion

**Proposed verification:** implementation uses `scripts/pse check <pkg> --all-targets` and
targeted `scripts/pse test <pkg> --lib -E '<filterset>'`, with explicit force-validation for
Arrow-consuming closures. Declaration/generator changes run `just codegen`. The integrator
selects assembled component/native/service journeys, `just hygiene`, governance/docs and
applicable native/powerset checks once functional scope is complete. Scientific references
remain 64h-owned; Python checks are selected only for affected independent reference consumers.
Report actual commands, modes, scope, zero failure baseline, results and exclusions. Selected
API/source reasoning and round trips cannot establish physical or operational qualification.

**Tested, 2026-11-03:** the prepared external-service target passed two tests, zero failures
against the zero baseline, using `scripts/pse-env --native=compiler,klu --
scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-runtime --test engineering_service
--features engineering-service,property-coolprop,solver-kinsol,product-reader,native-study-tests,force-validate
--locked --no-fail-fast` in an original native application scope with the explicitly prepared
`PSE_ENGINEERING_ENDPOINT`. This includes the source-derived numerical budget assertion,
same-run identity checks, stale/replaced endpoint refusal and reconnect controls. It does
not qualify the pending full Study API, all operation families, cohort products or D–G scope.

Completion requires a usable independent generated client, all selected consumer migrations
and deletions, truthful original outcomes/permissions, authenticated reconnect, proven custody
through settlement and G08-backed protected study readback. Publish enduring contracts through
the coordinator's architecture/ADR route and retire completed planning material by the existing
lifecycle policy. At close, record built behavior, a mistake corrected and deliberate deviations.

**Current checkpoint:** Full execution is authorized. The CLI delegates to the shared native
project handler. Source-owned engineering service contracts and the original-operation
session core are implemented; generated transport, full editing/readback migration and
acceptance controls remain in progress. Disk exhaustion interrupted initial generation;
the maintainer cleared build output, requiring cold preparation and affected reruns.
J01–J06 completion and quantitative performance benefits remain unestablished.
