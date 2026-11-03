---
title: "Plan64 Proto integration and scientific extensibility"
status: proposed
date: 2026-11-03
doc_role: review
doc_owner: docs/plans/64-process-simulator-target.md
doc_topics: [design, libraries, validation]
---

# Plan64 Proto integration and scientific extensibility

## Combined judgment

**Proposed:** improve Plan64's resulting system around an owner-derived engineering
boundary that serves inspection, exact-parent authoring, operation access and independent
exchange. Generate useful representations and checks from existing semantic owners;
retain their distinctions through clients and execution. Develop an explicit prepared
scientific-worker contract as a related extension, with complete provider evaluations and
derivative actions rather than scalar RPCs.

This is worthwhile beyond serialization: another property/unit family or analysis product
should gain discoverable contracts, intelligible diagnostics and usable external access
without repeating its meaning in several adapters. The desired end state is one scientific
system with several useful representations and clients, and explicit ownership of effects.

The maintainer authorizes reconsideration of delivered Plan64 scope and accepts breaking
design changes during this design stage. Completion of B/C is not a reason to retain an
inferior interface. Recommendations below include replacement of delivered mechanisms
where that completes the intervention; migration and deletion belong in the same adopted
scope. New capabilities that are independently deliverable are identified separately
because of their responsibilities, not because they touch completed code.

This review actions the seven hypotheses in the [Proto discovery briefing](../library_leverage/libraries/proto/briefing.md).
It recommends six complementary interventions, with conditions and closure evidence for
subsequent plan revision. They are proposals, not newly accepted obligations or evidence
that Plan64's existing completion claims are wrong.

## Baseline, scope and evidence

**Interface-checked:** inspected 2026-11-03 at `2efcc0cb9`, including concurrent working-tree
changes in scientific declarations, CoolProp integration, compiler controls, atomic capture,
registry/codegen outputs and xtask conformance. This is a moving source baseline, not a
committed implementation snapshot. The [Plan64 coordinator](../plans/64-process-simulator-target.md)
and 64a–64h retain package status; 64a A02/A03 and delivered B/C are foundations, D–G are
being expanded, and H retains its finite-population/reference/qualification obligations.

The selected standard is Core 3.4, efficiency heuristics 1.0, ProcessSimulator 1.5,
test-assessment 1.2 and the current pse-arrow binding. Source and architectural reasoning
establish the recommendations at Proposed/Interface-checked strength. No numerical run,
provider-worker prototype, codec experiment or performance measurement was conducted.
No architectural or scientific qualification verdict is implied.

Adjacent scope includes host descriptor tools and service custody, Rust-owned product
documents/actions, generation, canonical studies/results, diagnostics and independent
reference exchange. These intersections matter because they would consume or enforce
the proposed contracts. Their enclosing plans retain their own qualification and progress.
The review does not start a broad diagnostic audit, completion audit or general rewrite
of unrelated host, database or solver infrastructure.

### Decisive source observations

Source paths below are relative to the repository root; architecture citations name the
stable section owners rather than source positions.

| Observation | Consequence for the target |
| --- | --- |
| `pse-schema::FieldContract` and resolved contracts own relation/physical facets; `pse-codegen` renders model, Arrow, storage and document projections (blueprint §4/§6). | A descriptor may project these meanings; it cannot replace the admission of a foreign registry merely by matching names or hashes. |
| `crates/pse-runtime/src/host_operations/project.rs` already owns typed `ProjectAction`, `ProjectOperationRequest` and macro-declared `EffectFacts`; `pse-codegen/src/codegen/documents.rs` publishes direction-specific compiled schemas. | Not every boundary meaning is registry-owned. Generate from the actual Rust boundary owner as well as registry owners, rather than building another global engineering catalogue. |
| `workflow/modeling/documents.rs` owns exact-parent `SourceDelta`; `canonical/delta.rs` checks parent membership and edit intentions. `project_preparation_wire.rs` explicitly distinguishes bits, quantity IDs, Missing, Set, Tuple and ordered rows. | Generic field mutation and ordinary protobuf scalar defaults cannot substitute for these contracts. Delivered representations are candidates for coherent replacement, not contracts to preserve accidentally. |
| `workflow/completion.rs` owns scientific permission, scoped products, original diagnostics and lineage; `workflow/portable_payload.rs` checks selected IPC and lineage. | Transport adapters consume these decisions. Schema validity or a successful RPC cannot mint scientific permission. |
| `pse-kernels/src/{lib,provider,portable,shape}.rs` separates immutable provider meaning, prepared demand, coherent state evaluation and owner-sealed linearization. | A worker contract must support demanded derivative actions and attempt-local state, rather than manufacture transferable proof from serialized tokens. |
| `pse-host/src/product_reader.rs` delegates opaque passive product requests; `cli/product.rs` separates Query, Investigate and Follow. Native project execution already dispatches typed actions to existing owners. | Reuse discovery and lifecycle experience, but keep passive readers passive. Introduce explicit engineering commands where effects are intended. |
| `pse-host/proto/buf.gen.yaml` selects Buffa vtable reflection and local validation; the maintained validator generator has strict CEL admission/error corrections. | Existing Proto capability can be extended. Generator, runtime and local patches form the relevant integration closure. |

## Recommendations

<a id="I01"></a>
### I01 — Derive engineering boundary projections from their actual owners

**Proposed alignment improvement; integrate with remaining A/C/G contracts, including
replacement of delivered projections.** Add a versioned protobuf projection target to the
existing generation path. Registry-owned structures derive from registry declarations;
Rust-owned operation/documents derive from their typed declaration and effect owner.
Explicit wire choices such as stable field numbers belong beside the projection owner.
Do not infer them from field order, enum presentation ordinals or descriptor hashes.

Generate messages, complete descriptor artifacts, semantic-role links, boundary adapters
and supported local constraints together. Preserve input/output direction and standalone
document interpretation independently of protobuf package evolution. Retain one declaration
for each fact, while allowing the wire projection to make distinct encoding decisions.
Reserve removed wire numbers. An unsupported projection must fail generation rather than
fall back to an untyped JSON bag.

Start with preparation discovery/bindings and an actual project operation, then a scoped
completion product. Preserve finite scientific bits, explicit missingness, identities,
ordered rows and Set/Tuple distinctions. Static descriptors expose shape and links to
meaning; selected revision/physical-context inspection returns actual quantities, datums,
provider availability and refusals. A static option must not advertise runtime readiness.

**Alternatives and judgment:** retain Serde/Schemars for intentional JSON presentation and
existing native documents, deriving shared facets rather than manually synchronizing them.
Prefer generated Buffa projections initially because reflection, validation and the service
family already compose there. A narrow `proto_rs` shadow adapter is a credible alternative,
not disproved by the cost of a global registry replacement. Its second runtime/mapping and
validation obligations currently make it the weaker whole-interface choice. Reopen that
judgment if a representative recursive adapter removes material machinery while preserving
all consumers. A global protobuf-first replacement has no demonstrated benefit over the
existing semantic owners and would broaden migration into storage/science without necessity.

**Migration and closure:** replace the corresponding hand-maintained mapping/metadata and
its tests once every consumer moves; retain distinct formats only for distinct consumers.
Demonstrate that one new field/alternative reaches generated clients, descriptor inspection,
rules and documentation through its owner. Include exact bits/IDs/order/missingness and
old/new wire-number controls. Shared-schema round trips alone cannot establish physical
meaning: independently specified examples must challenge the projection.

**Principles:** AP-01/02/04/05, DP-01/02/06/16/24; H15, H19; PS-01/12.

<a id="I02"></a>
### I02 — Unify boundary constraints and typed edit intent, preserving contextual admission

**Proposed alignment and capability improvement; revise delivered authoring boundaries
alongside D/G consumers.** Declare locally decidable constraints once at their actual
boundary owner and derive JSON/schema and Protovalidate checks where both are consumed.
A declaration page's allowed size is a useful first case. Required alternatives, local
collection bounds and request-local relationships are appropriate candidates. Invoke
validation at uncontrolled ingress and explicitly on outgoing mappings where producer
errors can enter; carry structured field/rule/source identity through diagnostics.

Use descriptor-driven inspection and editable candidates to produce the existing typed
exact-parent operation. Preserve additions, replacement positions, explicit removals,
document/fit ownership and original source attribution. An external client can author a
new property/unit definition and submit its delta without becoming another scientific
interpreter. The UI/form is optional; the usable contract and a small generated client
are the bounded target.

Parent existence/membership, physical meaning, source validity, scientific permission and
transactional predicates remain contextual operations. They must not be copied into CEL.
The same rule may have several enforcing consumers, but it must not acquire separately
editable definitions. Unchanged admitted data may retain established properties where
premises hold; new external input still needs the relevant enforcement.

**Countercase and alternatives:** generated Buffa vtable reflection is read-only. A mutable
`DynamicMessage` candidate can use its checked `try_set`/`try_clear` implementation, but
conversion from generated values costs a wire round trip and descriptor pools must match.
`FieldMask` cannot encode exact-parent deletion or all omitted/default intent. Retain direct
typed operations where generic candidates add machinery without improving the client.
The maintained CEL generator refuses unsupported required rules; neither silent omission
nor an unqualified switch to runtime CEL is an acceptable convenience fallback.

**Migration and closure:** remove replaced local envelope checks and string-only translations,
while retaining contextual checks. Exercise one add/replace/delete/retain edit, stale parent,
unknown alternative, missing versus default, incompatible unit/datum and rejected admission
leaving the original revision intact. Challenge malformed rules, owned/view agreement and
outgoing-invalid mappings. Distinguish constraint violations from compilation/evaluation,
contextual admission and infrastructure failures.

**Principles:** AP-04/05/06, DP-03/07/15/21; H18/19/20; PS-01/02/04.

<a id="I03"></a>
### I03 — Make a finite engineering operation service a coherent client of the native system

**Proposed functional improvement; a bounded related delivery that may be incorporated
into the revised Plan64.** Offer a generated engineering interface that describes contracts,
inspects a selected revision, submits typed edits, prepares a case, submits a finite study,
reads its original outcomes/products and requests cancellation. This gives independent
engineering clients useful access and tests whether the shared boundary actually works.
It is justified by the credible authoring/study workflow, even without an existing caller.

Product-owned dispatch calls the same typed runtime operations, admission, completion and
publication owners used locally. Factor reusable action handlers out of binary glue where
needed and migrate local consumers to those handlers. Keep the independent host opaque to
scientific policy. A passive product-reader Query or Follow must never gain command effects;
explicit service commands carry declared effects and use the existing native owners.

Describe-contract responses expose immutable schema identity and declared links/effects.
Selected inspection reports actual context/capability/refusal. Submission returns original
operation/run identity; duplicate requests follow the owning operation's actual repeatability
contract. Status, cancel and retained readback address that original identity. A progress
stream is observational: disconnect, timeout and cancellation request do not imply native
settlement. Reconnection reads original retained facts and does not resubmit computation.
Keep diagnostic-only structural outcomes and failed siblings visible without promoting them
to usable scientific states.

**Alternatives and countercase:** expose existing typed native calls directly for linked
consumers. Use ConnectRPC for independent clients rather than another RPC abstraction or a
wholesale Tonic migration. Full-duplex bidi needs HTTP/2 and browser request streaming has
limits; unary commands plus server-streamed observations are sufficient for this target.
Health/reflection are separate companion capabilities and cannot establish science readiness.
Public hosting, multi-tenant authorization and distributed solving are separable decisions.
An authenticated local service is the first placement; host custody must be established on
each actual connection, not inferred from schema discovery or socket-path spelling.

**Closure:** an independent generated client performs inspect/edit/prepare/study/readback,
including stale context, intended scientific failure, duplicate submission, observation
disconnect/reconnect and cancellation while native work continues. G08 remains the sole
capture transaction owner: streaming cannot replace atomic membership/protection/receipt.
Delete replaced dispatch/conversion choreography once migrated. The change must not add a
second execution engine or compatibility path around the new shared handlers.

**Principles:** AP-01/03/06/07, DP-05/18/19/20/21; H8/21/23; PS-10/11/12.

<a id="I04"></a>
### I04 — Specify a prepared scientific-worker boundary at coherent evaluation granularity

**Proposed scientific extension; adopt required contract corrections wherever current
providers need them, and deliver worker execution as an explicit related package.** A
persistent isolated worker can contain foreign runtime failures and make non-Rust science
accessible. Keep `ProviderSpec`, prepared demand and attempt-local ownership as the contract.
Preparation binds exact implementation/data revisions, ordered ports, physical quantities,
units, shapes, derivative source/order and smoothness. It returns an attempt-bound worker
handle; evaluation supplies inputs and the prepared demand, returning one complete coherent
state or the actual typed refusal/failure.

Preserve `CompleteJets`, `SelectedColumns`, `JacobianAction`, `TransposeAction` and
`WeightedSecondAction` as distinct demands. Do not turn an action into a full matrix or
one RPC per scalar. Multi-state batching, when useful, needs explicit point identity,
state-reset/order and independent failure semantics: the present single-state `fill` does
not establish that capability. The local proxy verifies extents and completion. Issuing
owner-sealed linearization evidence additionally requires declared linearization support
and re-establishment of the current attempt/session-bound derivative context against the
expected context; otherwise it returns no token. Shape and completion alone cannot
establish branch or inner-solve context. Never deserialize a worker-supplied token
into local proof; worker replacement/reconnection invalidates its contexts.

**Placement and alternatives:** retain current in-process CoolProp for supported water
science unless isolation buys a worthwhile benefit. A worker is valuable for a selected
foreign implementation or containment need, not merely because RPC exists. Define how the
synchronous provider call waits on an owned worker without blocking an async executor or
allowing an RPC deadline to abandon native work. Cancellation retains custody until the
worker stops or an owned process is terminated and reaped. Do not automatically replay an
evaluation after ambiguous loss, especially where mutable linearization state matters.

The first worker may use the existing water contract as a controlled comparison; that
establishes isolation/interchange, not new scientific breadth. A later non-Rust family
must bring its own source/data/derivative/envelope evidence. Keep remote deployment and
distributing an entire solve as separately decided capability changes.

**Closure:** compare complete native and worker-backed cases with identical science,
including each derivative demand as supported or explicitly refused, changed data/branch identity, outside-envelope trial,
stale linearization, worker crash, cancellation and settlement. Independent physical and
derivative checks remain necessary. Measure a representative complete solve before claiming
latency/throughput benefit; do not hide callback overhead behind a codec microbenchmark.

**Principles:** AP-02/03/07, DP-08/10/15/19/20; H3/8/12/21; PS-01/02/06/07/09.

<a id="I05"></a>
### I05 — Join reference interchange and compatibility to scientific interpretation

**Proposed improvement; integrate with H02–H04 and G's imported artifacts/observations.**
Use generated metadata envelopes for independently populated case specifications and
reference outputs where this simplifies cross-language exchange. Include exact configuration,
source/data/formulation, units/basis/datum, required output coverage and actual failed,
unavailable or unattempted outcomes. Keep Arrow IPC/Parquet for tabular scientific values
and existing source/publication owners for provenance; protobuf is not another result store.

Publish complete descriptor artifacts and apply Buf structural compatibility checks to
externally consumed contracts. Add independently authored semantic-change controls for
changes that wire checks cannot establish: a datum change, enum reinterpretation, stronger
validation, different branch or changed output applicability. Preserve unknown binary fields
where required, while explicitly checking JSON/text conversion and presence behavior.
Descriptor compatibility does not admit scientific equivalence or define content identity.

**Alternative and countercase:** retain current JSON/Arrow exchange when it already serves
the case; the chosen envelope should replace equivalent conversion/catalogue work rather
than surround it. One authoritative structural declaration is useful for interoperability,
but using the same production calculation to populate expected answers destroys independence.
H01's unresolved denominator cannot be closed by a schema or generated test population.

**Closure:** carry one ordinary native/reference case through independent construction,
comparison and retained readback; include intended failure, missing required outputs,
wrong datum and semantically incompatible yet wire-compatible evolution. If exchange ownership
changes, migrate both producers and comparators together and delete the replaced path.

**Principles:** AP-02/04/06, DP-04/11/22/23/24; H20; PS-01/12/13.

<a id="I06"></a>
### I06 — Push selection before encoding and use borrowed views at real boundaries

**Proposed execution improvement; revisit C's delivered metadata/read paths with G/E
consumers.** Carry requested fields, pages and products to the actual producer before
constructing or encoding full inventories. Where an encoded protobuf boundary is useful,
use Buffa `MessageView`/`OwnedView` and `ViewEncode` to inspect or forward selected metadata
without rebuilding every owned string/byte payload. Keep prepared native and Arrow execution
native; introducing serialization to use a view is counterproductive.

`documents.rs::bound_observation` currently serializes to an extent-counting writer before
returning its value. That is a concrete two-consumer boundary to inspect when a new encoder
will traverse the same selected response. Consolidate extent enforcement only if the new
owner preserves refusal-before-publication and all existing resource protections; a later
encoder must not silently remove the earlier admission guarantee.

**Countercase and closure:** views can retain a large backing buffer; repeated/maps still
allocate, dynamic editing bridges encode/decode and lazy views defer nested errors. Prefer
owned compact projections when they release much more memory. Keep canonical pre-allocation
wire checks and complete observation/capture coverage. Compare selected/full and empty-result
reads, retained backing-buffer lifetime, cancellation and partial output. Measure the whole
read/encode/consume lifetime before asserting reduced memory or faster operation. Remove
the replaced projection/pass and its callers only after these guarantees move.

**Principles:** AP-07, DP-10/16/20/22; H5/6/7/16/17; PS-11/12.

## Integration, decisions and independent challenge

I01/I02 are the shared foundation, and should be specified before more D/G boundary
variants harden. I03 supplies a useful consumer and should co-design that foundation rather
than wait for a complete universal catalogue. I05 can reuse its selected projections in
parallel with reference construction. I06 applies where those operations expose real
conversion work. I04 depends on provider semantics, not completion of a general engineering
service; do not make local scientific delivery wait for a worker framework.

These priorities are consequence-based, not a serial execution lane. Incorporate coherent
breaking replacements of B/C into the adopted intervention, including native callers,
generated schemas, client adapters, relevant historical interpretations and test fixtures.
Do not keep old/new production mechanisms indefinitely as comparison evidence. Retain
independent reference implementations because they discharge a different assurance role.

The combined intervention deliberately avoids forcing every meaning into protobuf or the
registry. Rust action effects, selected physical context, canonical transactions and solver
capabilities each have an existing owner. Static discoverability, contextual admission,
scientific permission and physical settlement remain separate facts. A future model family
may require a genuinely new domain concept; the target is localized expression and propagation,
not a promise that no core contract will ever change.

Challenge the completed design with an E/F extension from a lumped inventory model to a
distributed or cyclic configuration. Source-owned axes, phase allocation, event transfers
and physical continuity must reach inspection and retained trajectory products through
their existing temporal/scientific owners. A descriptor advertising an event field cannot
imply that the selected integrator supports sensitivities across it. Reuse of preparation
must follow the changed science/layout, while a value-only change retains compatible
structure. This is the extension-locality discriminator for dynamics, rather than an
additional obligation to implement a new dynamic profile during interface migration.

### Rule impacts proposed for adoption

| Reference | Proposed impact and route |
| --- | --- |
| RC01 | Refine 64a A03's generated-adapter target to include Rust-owned operation/document projections as well as registry-owned science; update blueprint §4/§21 through the decision route if enduring contracts change. No global declaration-authority migration is recommended. |
| RC02 | Deliberately revise versioned external/durable boundary interpretations where I01/I02/I05 break them. Preserve semantic hashing/admission owners; no descriptor-derived replacement hash is proposed. Durable/public contract changes follow their ADR route; dependency additions alone need no ADR. |
| RC03 | Add an explicit effectful engineering service and optional foreign client contract if I03 is adopted. This may revise the native/foreign boundary policy; it must not silently widen passive observers or create a required Python product. |
| RC04 | An isolated provider binding, and especially remote placement, needs the applicable kernel/binding/placement decision before implementation. This review does not repeal the current exclusion of distributed solving or qualify remote deployment. |

A fresh design-reviewer supplied independent source-grounded judgment alongside the library
research contribution. The principal corrections incorporated here are recognizing the actual
Rust-owned action/effect catalogue, separating static descriptor metadata from contextual
physical meaning, and preserving provider action/linearization custody. These prevent the
seemingly common interface from becoming a second global catalogue or scientific interpreter.
No additional broad audit or recommendation-audit campaign was initiated.

### Briefing coverage and next adoption decision

| Briefing hypothesis | Review result |
| --- | --- |
| 1 generated engineering contracts | I01 recommended, refined to actual distributed owners and dynamic context |
| 2 scientific implementations | I04 bounded worker target; provider corrections can intersect Plan64 directly |
| 3 compiled constraints | I02 recommended for declared local rules, with contextual admission preserved |
| 4 editing/inspection | I01/I02 recommended as typed operation clients, with explicit mutation limits |
| 5 reference contracts | I05 recommended selectively, retaining independent expectations and columnar values |
| 6 engineering services | I03 bounded target, including explicit effect and settlement ownership |
| 7 materialization | I06 selected-boundary improvement; quantitative benefit remains unmeasured |

Library research used Context7 first and exact installed sources for consequential details.
The decisive upstream contracts are [Buffa code generation 0.9.2](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/),
[descriptor reflection 0.9.2](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/index.html),
[view ownership 0.9.2](https://docs.rs/buffa/0.9.2/buffa/view/index.html),
[ConnectRPC 0.9.1](https://docs.rs/connectrpc/0.9.1/connectrpc/),
[the published proto_rs 0.11.26 archive](https://static.crates.io/crates/proto_rs/proto_rs-0.11.26.crate)
and [Buf compatibility rules](https://buf.build/docs/breaking/rules/).
Rust ConnectRPC and proto_rs Context7 searches returned unrelated results; released and
installed source supplied their selected contracts. Local CEL behavior is additionally
bounded by `vendor/protoc-gen-protovalidate-buffa/README.md` and its maintained implementation.
Companion health/reflection packages and development proto_rs 0.12 capabilities were not
treated as capabilities supplied by current core pins. No fresh complete API/reference
census or end-to-end generator comparison was acquired.

The next operator decision is which coherent amendments to incorporate into Plan64 and
which related capability packages to authorize. Plan revision should retain these anchors,
target behavior, required consumer migrations/deletions, rule impacts and discriminating
closure evidence. The coordinator's F01–F06 and existing recycle I01–I12 dispositions keep
their original identities and owners; this document's I01–I06 are review-local references.
No plan status, accepted design or production code is changed by publication.
