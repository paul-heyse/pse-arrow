---
title: "Plan65k: Scientific artifacts and native reconstruction"
status: in-progress
date: 2026-11-05
adrs: [ADR-0245, ADR-0246]
review_sources: [docs/plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, numerical-execution, preparation, validation]
---

# Scientific artifacts and native reconstruction

## Target and ownership

**Proposed:** deliver complete scientific artifact exchange and reconstruction without treating
wire decode, historical evidence, current scientific admission and live delegation as the same
operation. This companion implements November 5 review [I09](../plan_improvement_reviews/plan65-proto-capabilities-and-codebase-integration-2026-11-05.md#I09)
and the artifact portion of I01/I05. It owns existing packages **E02/E03**, moved from 65e
without renaming, resetting status or upgrading evidence. It introduces no duplicate K-package
for that work. The coordinator owns disposition; 65i owns scientific meaning, 65e owns mutable
provider sessions/batching, A/B own mechanical generation/boundary policy, C/F own storage and
result delivery, and H owns assembled qualification.

Authoring baseline is clean main `1230a7e833fe31a809a077107bf2943cef3fc255`.
Core 3.4, heuristics 1.0, process-simulator 1.5 and test-assessment 1.2 apply. The moved
checkpoints below retain their original source/date/conditions, not this authoring baseline.
No product build, fault probe or numerical test is performed by this revision. Full Recipe
protobuf activation and live native-body transfer remain unestablished. Current JSON
interpretation three remains active until a complete selected replacement qualifies.

## Three consumed contracts and activation decisions

| Consumer | Required meaning and target | Activation prerequisite |
| --- | --- | --- |
| Independent/canonical receiving reconstruction | Reconstructible source, original physical/context selection and exact receipt/membership interpretation; receiver performs contextual admission | Complete record population, original root/receipt consumption, bounded phases, retained old-policy disposition and actual receiving controls |
| Original live parent to its one provider-free private child | Parent retains original admission/evidence and proposal screening; child receives only the checked mathematical/execution closure it needs | ADR-0246 scope, checked supplier packing/restoration including MPFR and relocated symbols, full stage/layout/guard preservation and physical custody |
| Prepared provider session | Immutable specification is exchanged here; mutable prepared handles, cancellation, batch state and process shutdown stay in 65e | E04/E05 capability and session controls; no serialized live closures/workspaces |

No artifact digest or generated validator constructs a scientific capability. Portable request
reconstruction and live delegation are distinct operations, not two interchangeable production
routes for one operation. Retained evidence stays readable only under its promised original
interpretation; otherwise provide explicit migration/incompatibility. Source/display artifacts
can be rebuilt where disposable, without redefining protected scientific history.

The closed child's generated envelope may remain useful even if its body uses checked native
packing: framing/identity and body realization discharge different contracts. Compare the
existing source reconstruction, generated portable body and live native delegation over full
preparation, transfer, restoration, derivative/evaluator setup, execution, original-coordinate
verification and reap. Do not infer a faster whole solve from an encoding comparison.

## Library realization and bounded foundation assessment

Use Buffa/codegen 0.9.2 source-derived records and the maintained allocation-accounted view
profile where selected. `CodeGenConfig::allocation_accounted_views` and a shared
`DecodeContext::with_allocation_budget` debit requested backing growth before allocation;
they do not cover owned adoption, validation, mathematical normalization, allocator metadata or
RSS. Explicit lazy descendants require the actual accounted capability checks. The exact
[runtime](../../vendor/buffa/pse-patch.md) and [generator](../../vendor/buffa-codegen/pse-patch.md)
contracts characterize the maintained functionality; stock element limits are not equivalent.
Generated `Validate` from the maintained 0.7.2 validator discharges admitted local rules, not
receiving reconstruction or per-record qualification. [Validator source contract](../../vendor/protoc-gen-protovalidate-buffa/README.md).

Native membership V1 uses the actual monomorphic native root and original receipt consumers.
Serde compound serializer traversal can avoid a DOM only while preserving exact consumed
number/map-key/variant/surviving-subtree semantics. The existing ADR-0245 prerequisite and
limits below remain binding on that route. An unrelated typed visitor, a whole-payload permit
or raw Proto fields cannot replace membership by shape agreement.

For live transfer, load the active native-solver-libraries skill and inspect the maintained
Symbolica/Numerica sources selected by Cargo before implementation. The existing supplier
investigation below establishes gaps, not a released checked transfer API. Require complete
non-rational/MPFR coefficients, symbol remap and canonical packed ordering, native stage and
source occurrence restoration, quantity/layout/obligation correspondence and all settings.
Same-width symbols or `PreparedBody` equality alone are insufficient. Unknown supplier bounds
remain a blocked activation prerequisite; do not impose a new host resource cap or fabricate
an RSS guarantee. Neither proposed ADR is accepted or implemented by this document.

## Delivery sequence and targeted acceptance

1. Reconcile the exact immutable record and receiving consumer population with E01/65i, then
   complete one canonical/private native receiving slice against working A/B records.
2. Finish qualification/receipt membership and complete phase admission before binary activation.
   Keep allocation, validation, adoption and mathematical reconstruction limits separate.
3. Settle the live-child supplier route independently. If its checked packing/restoration cannot
   establish the consumed contract, retain the currently qualified source reconstruction and
   record the actual unresolved capability; no placeholder bypass is allowed.
4. Migrate each selected producer and receiver together; remove the replaced DTOs, decode/index
   or reconstruction path only after targeted controls establish its replacement. E06/H then
   integrate scientific inspection, store receiving and assembled native failure journeys.

For E02 test exact bits, memberships/ordering, absent and extra receipts, changed datum/provider
context, foreign qualification, unsupported values and historical/current interpretation.
For E03 compare direct versus actual source-built child using independent root expectations;
include stale image, malformed/truncated source, unknown completion, cancellation, parent loss
and actual reap. Live-transfer controls additionally include non-rational coefficients and
symbol permutations across packing boundaries. The stopped native-lane preparation supplies
no execution pass. Controls proposed here do not assert support for new scientific classes.

## Packages and migration

| ID | State | Prerequisite | Delivery and targeted acceptance |
| --- | --- | --- | --- |
| E02 | in progress | E01; narrow working 65a/65b and receiving operation from 65i I02 | First remove kernel portable descriptor mirrors while retaining contextual receipt replay and truthful interpretation changes. Generate portable scientific records/adapters and migrate existing producers/readers. Buffa generation and supported `Validate` discharge local correspondence; native reconstruction and completion-owned selectors discharge scientific admission/use. Cover ordering, exact bits, missing/unknown alternatives, scope/coverage and refusal classes. Remove migrated capture/restore pairs and duplicated shape declarations; retain qualification. Test changed native variants against independent known meaning. |
| E03 | in progress | E02 closed-root slice; 65d transport/settlement primitives where shared | Project actual native root/input/source-map/handshake/request/outcome owners, then activate generated bounded envelopes on the existing inherited channel. Preserve provider-free rejection, original registry/body checks, numerical policy and cancellation/drain. Compare direct and child outcomes on the same independently specified roots; inject malformed source, stale image, unknown completion and child failure. Delete replaced private wire/JSON-only helpers and their mechanism-only fixtures. |

E02/E03 retain their previous labels. Old 65e anchors redirect here; this table alone owns
their progress. The detailed historical assessments below supply preserved prerequisites and
evidence, not additional independently tracked F-package identities. References to E02
qualification F01–F03 below retain historical assessment labels, not 65f package identities
or additional current status owners. Current membership/receipt/phase-bound and receiving
activation obligations belong to E02; closed-child reconstruction, screening and physical
settlement belong to E03. The explicit gates and delivery sequence above govern current work.

**E02 realization:** Buffa 0.9.2 generated correspondence and the maintained accounted eager
view support are used only within their checked decoder profile. The same native root must
be indexed/adopted/restored under the original membership/receipt contract; successful local
validation is not receiving admission. The [exact generator profile](../../vendor/buffa-codegen/pse-patch.md)
and [runtime budget contract](../../vendor/buffa/pse-patch.md) define capability checks and
phase exclusions. This package closes all actual receiving consumers, not only scalar examples.

**E03 realization:** use the already generated envelope and bounded inherited framing for
attempt/handshake/outcome correspondence; use the selected E02 body route rather than assuming
native packing exists. [The closed-child source](../../crates/pse-runtime/src/math/native_child.rs)
owns original channel/parent screening, and [ADR-0246](../adr/0246-live-admitted-body-delegation.md)
characterizes the narrower proposed delegation premise. A proposal decoded through a generated
envelope remains subject to independent original-coordinate verification and actual reap.

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

The private `pse.scientific_recipe.v4` family now publishes messages, eager/lazy views,
maintained validation, native owned/borrowed adapters, policy and provenance from one actual
`ProjectionOutput`, descriptor set and generation configuration. Both adapter families compile
inside the private compiler owner; passive decoding grants no reconstruction authority.
Production encoding and qualification still use interpretation three.

**Tested, targeted on 2026-11-04:** `scripts/pse test pse-compiler pse-modeling --lib -E
'test(scientific_recipe_owned_and_borrowed_) | test(scientific_recipe_source_graph_) |
test(scientific_recipe_native_projection_) | test(resolved::records::restoration_tests::)'`
passed 7/7 against zero failures, with no build warnings, configured unbounded placement,
cache off, prepared compiler/KLU and complete original capture/settlement. The actual authored
Recipe survives owned and borrowed binary adoption with exact existing JSON; root/nested
unknowns, missing nested syntax and malformed digest width refuse. The restoration controls
reject a forged unbounded formal count before allocation and charge every repeated selection's
formal storage. Capture and restore share that accounting; existing native qualification
remains required. The separate xtask `scientific_recipe_bundle_` control passed 1/1 and full
`just codegen` published fourteen targets; its operations recursion warning remains a G08
correction obligation rather than clean workspace evidence.

Monomorphic same-root membership receivers, complete original workspace admission and full
restoration bounds remain required. The existing JSON/qualification route stays active until
those replacements and their refusal/drain controls are accepted.

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

### Native membership prerequisite integration

Main `f3e298eea` integrates the bounded prerequisite. Data-only
`NativeMembershipLimits`/`NativeMembershipCounts` expose Serde traversal extents;
`RecordAuthority::from_authenticated_native_root` remains unsafe and builds its own sorted
index from the borrowed native root. No caller-supplied index is accepted. Native membership
queries reuse the same normalizer, while typed budget/profile failures retain their source
through quantity, modeling, kernel and compiler restoration. Production JSON v3 remains the
live route; the monomorphic binary canonical/private receivers are test-only. Their by-value
reconstruction consumes the same Recipe that was indexed and retains original binary bytes
separately from the active JSON leaf.

**Tested, targeted before main integration:** `scripts/pse test pse-ids --lib -E
'test(scientific_replay::)'` passed 18/18; `scripts/pse test pse-compiler --lib -E
'test(scientific_recipe_native_) | test(scientific_recipe_owned_and_borrowed_) |
test(scientific_recipe_source_graph_)'` passed 10/10, zero failures, pending records or build
warnings. Conditions were configured unbounded placement, cache off, prepared compiler/solver
and complete original capture/native settlement. Controls include the actual Recipe's full
surviving-node DOM differential, scalar binary reconstruction, local digest/interpretation/
context refusals, and component replay with two real inference operations and two declaration
boundaries: positive complete drain and missing/extra/reordered streams. That component scope
does not claim the scalar body consumed those operations. The independent implementation
review closed two prerequisite defects: erased prepared-provider membership errors and moving
vector-growth overlap. Counts now cover three times counted traversal slots and actual owned
key capacities; they do not bound decoded roots, validation, adoption or complete RSS.

The isolated checkout was retired after its committed 18-file content matched main exactly;
its original evidence is retained outside the checkout. Provider population, nonempty
mathematical-proof ordering, real canonical/private authentication and full receiving admission
remain open. No production activation or complete F01–F03 acceptance follows.

### Library-owned pre-growth prerequisite

Main `44708962e` integrates the independently reviewed maintained Buffa/Buffa-codegen
0.9.2 profile. `CodeGenConfig::allocation_accounted_views` defaults to false and is
selected only for the private Recipe world. `DecodeContext::with_allocation_budget`
shares a caller-owned `Cell` across descendants/merges; the runtime debits the full
new backing layout before each fallible vector/box growth, retaining earlier charges
for overlap and replacement. Layout overflow, exhausted allowance, unaccounted decoder
and actual reservation failure remain distinct typed refusals. These APIs are maintained
source functionality, not claims about the unmodified release; see the exact
[runtime contract and source](../../vendor/buffa/pse-patch.md) and
[generator contract and controls](../../vendor/buffa-codegen/pse-patch.md).

The profile covers opted-in eager/lazy decode stores, packed extenders, singular/group/
oneof boxes, fragments/raw elements and unknown-field backing. Child capability checks
close ordinary-import and external-decoder bypasses. Explicit lazy traversal must check
`LazyMessageView::allocation_accounted()` for **every selected root and child** before
`decode_lazy_with_ctx`/`merge_lazy`: stock generated overrides can bypass the trait's
default guard. Getters recreate contexts, so the selected complete probe must use raw
fragments/elements with its original context. Owned decoding, validation/adoption,
default caches, allocator metadata, stack and RSS remain outside this decoder claim.

**Tested before main integration, configured unbounded/cache-off:** allocation controls
passed 7/7, stock view controls 98/98, packed extenders 7/7, and the existing
`pse-codegen:buffa_allocation` target passed its one wrapper control executing seven
real generated controls, including stock-import refusal and accounted-external acceptance.
The fixture compares default output byte-for-byte with the published 0.9.2 generator
on those descriptors; it does not qualify every generator configuration. Source-pin
controls passed 2/2 and `scripts/pse check pse-codegen --all-targets` passed, all with
zero failures/build warnings and complete original capture/settlement. Main readiness
passed after integration. Both changed supplier archives were independently compared
with the cached published source. The clean checkout's 102 owned paths matched main
before removal. Main regeneration passed all 14 targets; the combined
`pse-ids`/`pse-compiler` native membership/Recipe selection passed 28/28 with zero
failures/build warnings and complete capture/settlement. The actual generated decoder
fixture also passed on main (one wrapper executing seven controls, zero build warnings).
Full receiving-phase admission and activation remain open.

The complete F03 envelope remains unresolved. The native graph and library enforcement
prerequisite are now available. Main `d1e597de1` adds the source-generated complete lazy
decode probe, sharing the final descriptor world, generator configuration and actual
type/field naming with eager adapters. Every selected lazy root/child checks its accounting
capability before low-level decode or merge. Singular fragments merge before descendant
traversal; repeated raw children are visited individually; eager carveouts use their
accounted eager decoders. Allocation, element and unknown-byte cells remain shared for the
whole walk. Typed resource errors retain their original library error source. This is a
decode prerequisite, not the full validation/adoption/restoration envelope. The source
review cross-checked 119 reachable lazy structs and 230 deferred edges; pre-integration
compiler controls passed 14/14 and generator traversal controls 3/3. The xtask bundle
control's compilation completed, but storage exhaustion prevented artifact capture and its
test body did not run. Final main generation passed all 14 targets with unchanged outputs.
The integrated main compiler selection subsequently passed 15/15, zero failures/build warnings,
pending records or incomplete original capture/settlement, using configured unbounded/cache-off
compiler/KLU/solver and `scripts/pse test pse-compiler --lib -E
'test(scientific_recipe_native_) | test(scientific_recipe_probe_) |
test(scientific_recipe_owned_and_borrowed_) | test(scientific_recipe_source_graph_)'`.
The initial main xtask selector omitted `product-schemas` and matched zero tests; it is not
bundle acceptance. The corrected `scripts/pse test xtask --bin xtask --features product-schemas
-E 'test(scientific_recipe_bundle_)'` passed its one control on main, zero failures/build
warnings/pending and complete original capture/settlement, in the same prepared native mode.
The isolated checkout has been retired
after content reconciliation. Receiving-phase admission and activation continue on main.
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

The actual borrowed view already receives complete validation through
`decode_scientific_recipe_view` → `decode_view_root` → `wire.validate()` before native
construction and `validate_projection()`. The maintained validator emits direct eager-view
implementations; no temporary owned protobuf tree or duplicate validation pass is needed.
See [borrowed crossing](../../crates/pse-contract-proto/src/support/borrowed.rs), the
[maintained message emitter](../../vendor/protoc-gen-protovalidate-buffa/src/emit/mod.rs),
and [repeated-rule lowering](../../vendor/protoc-gen-protovalidate-buffa/src/emit/repeated.rs).
The receiving envelope must cover absent/default rule sites, complete violation vectors,
path insertion/capacity overlap, a copied map key per descendant violation, formatted
messages and allowed-list join temporaries. Three root uniqueness checks allocate reference
HashSets in separate scopes; their scratch peak is the maximum, not the sum. The borrowed
canonical map observes last-write-wins entries without allocation but has quadratic
worst-case iteration. Current finite-CEL lowering uses integer comparisons/stack cells;
future rules cannot silently inherit that profile or assumptions about nested validator
error propagation. Native adapter staging vectors overlap final `Arc`/tree collections.
These are source-grounded admission obligations, not established complete bounds.

### Reconstruction population and native mathematics limit

Further read-only inspection at `165faf093` follows reconstruction beyond decode and
membership. [`ResolvedRecord::restore`](../../crates/pse-modeling/src/resolved/records.rs)
shares one function product per identity, checks cycles/depth, and validates selected
formal counts against the original function before charging syntax entries and allocating
formal vectors. Required source facts include the sum of selected formal counts, not just
distinct functions. Explicit value/type/quantity records restore their declared recursive
members; indexed types do not instantiate an implicit Cartesian product. Price the actual
boxes, strings and collection populations, including staging/final-collection overlap.

Physical receipt replay clones complete entries/boundaries and keeps them throughout
reconstruction; mathematical proof replay also clones entries and consumed branch bytes.
Their recursive populations and overlapping lifetimes must enter the same receiving
reservation. Authenticated proof replay consumes recorded results rather than rerunning
the enclosed admission-only proof closures. Provider restoration constructs descriptors
and prepared facts, not provider workers. See the actual
[physical replay owner](../../crates/pse-quantity/src/resolved_receipts.rs),
[proof replay owner](../../crates/pse-math/src/typed_receipts.rs) and
[provider restoration](../../crates/pse-compiler/src/typed_math/provider_records.rs).

Native mathematical reconstruction remains a consequential unresolved condition.
`reconstruct_recipe` invokes `Request::admit_inner` and lowers source mathematics again.
Body occurrence/slot limits bound selected construction work, but do not by themselves
bound Symbolica coefficient bytes. For example, a source DAG of exact-integer repeated
multiplication can grow coefficients far faster than its node count. This is a source-derived
possibility, not an executed probe; the existing positive-integral-Pow defense does not
cover multiplication. Partial construction can likewise allocate before returned-operation
count checks. Do not infer a byte bound from these post-construction counts.

Symbolica 3.0.1 `AtomView::{add,mul,pow,div}_with_ws_into` normalizes through reusable
`Workspace` buffers without a byte-admission or cancellation argument. Its recycling
cache does not bound peak live temporaries. Repository `ConstructionAdmission::try_grow`
can extend the original live allowance before explicit allocations, but these arithmetic
operators do not invoke it. Evaluator `direct_translation(true)`, alias DAGs, abort checks
and optimizer limits apply to evaluator construction, not every earlier reconstruction
operation. Formal-symbol registration also grows global native metadata in chunks of 8192;
a chunk count is not an established byte bound. Sources are the exact-release
[Atom arithmetic](https://docs.rs/symbolica/3.0.1/src/symbolica/atom.rs.html),
[workspace implementation](https://docs.rs/symbolica/3.0.1/src/symbolica/state.rs.html),
[typed arithmetic](../../crates/pse-math/src/typed.rs),
[partial construction](../../crates/pse-math/src/typed_partial.rs),
[construction admission](../../crates/pse-math/src/construction.rs) and
[library construction](../../crates/pse-math/src/library.rs).

Before F03 activation, resolve normalization, coefficient/partial growth and symbol
registration alongside the source-derived passive populations. A payload multiplier,
retained-capacity measurement, decoder-only budget or post-allocation operation refusal
does not establish the required pre-allocation envelope. Current JSON v3 remains active;
no canonical/private binary cutover or complete F01–F03 acceptance follows from this work.

The next bounded design investigation should compare preserving already admitted native
body facts against budgeting the full repeated preparation operation. A prepared-body transfer
could avoid re-lowering, support analysis and derivative reconstruction, but is not established
by serializing existing atoms or stages. The maintained Symbolica
[`Atom::import`/`import_with_map` and `rename`](../../vendor/symbolica/src/atom/representation.rs)
normalize during symbol remapping, including unchanged mappings; calling the ordinary
`PreparedBody` constructor also repeats analysis. Any selected transfer therefore needs a
supplier-owned checked canonical-atom load/remap that does not normalize, a native structural
restore path, and preservation of original qualification, per-record membership and exact
receipt consumption. Current replay drains receipts through lowering, so a different
authority premise requires the proposed ADR route before implementation. This is a candidate
for investigation, not an adopted bypass or a new universal mathematical IR.

Alternatives retain their actual limits: supplier allocation enforcement would have to cover
coefficient arithmetic, transformations and global metadata, not only Atom buffers; a
conservative preflight needs justified arithmetic/partial/replacement bounds and an explicit
unsupported-profile refusal; local materialization barriers can change mathematical
realization and do not alone cover later substitution. Existing process custody and opaque
allowances establish neither of those pre-allocation guarantees. A separately qualified
kernel-enforced address-space ceiling would be a different closed-lane mechanism and would
not qualify canonical in-process restoration. No new process limit is selected here.

The candidate population is the existing native `PreparedBody`, not a new mathematical
language. Its stages already distinguish Block, Require, Domain, Applicability, Branch and
Provider. A bounded base-body snapshot would need the admitted stages/atoms, layouts and
formal slots, output/effect slots, obligations and quantity maps, optional analyzed expressions,
provider specifications, checked members/sources/occurrence tokens and accounting populations.
Instance bindings, provider workers, original `Arc`/owner identities and demanded evaluator
workspaces remain outside it. The existing same-process `BodyRetention` can reuse an
`Arc<AdmittedBody>` after semantic/physical checks, but there is no passive cross-session
constructor that avoids analysis. `PreparedBody` equality omits some accounting fields and
is therefore insufficient as transfer acceptance. See
[`PreparedBodyData` and construction](../../crates/pse-math/src/execution.rs),
[native stage definitions](../../crates/pse-math/src/guarded.rs),
[admitted-body owner](../../crates/pse-compiler/src/typed_math.rs) and
[actual retained-body reuse](../../crates/pse-compiler/src/workspace/modeling/executable/grouped.rs).

Supplier feasibility research finds no existing safe checked loader in the inspected exact
3.0.1 source: `Atom::read` checks format/extent but enters internal unchecked representation;
ordinary views/iterators assume valid packing and current-state references. `StateMap::is_empty`
does not certify exact symbol definitions or state generation. Even an injective rename can
change canonical ordering; generic comparison may materialize large coefficients, while
symmetry/custom callbacks and polynomial/field coefficients add transformation and metadata
obligations. A plausible first supplier profile would need safe packing inspection before
ordinary views, authenticated canonical-producer provenance, exact state/symbol compatibility,
prepriced output/order scratch and a bounded remapper excluding unsupported transformations.
Identity-only loading avoids ordering repair but does not solve fresh-session reconstruction.
Exposing the unchecked reader or suppressing normalization flags is not a valid prerequisite.
These are bounded research conclusions, not established library functionality; precise source
owners are `vendor/symbolica/src/{atom/representation.rs,atom/coefficient.rs,coefficient.rs,state.rs,normalize.rs}`.

Keep current semantic-body and BodySpec definitions separate from any new payload encoding.
The existing source/layout/physical/admission meaning must not silently become an atom-byte
hash. Full source/physical/provider receipts remain attached until a reviewed native contract
establishes their exact snapshot correspondence and complete consumption. Base `AdmittedBody`
is Value-order mathematics: transferring it would not by itself avoid later demand-specific
`PreparedSupport` differentiation, compaction or evaluator construction. Any such extension
needs its own exact output/coordinate/order population and admission contract. First establish
source-admission versus transferred-body equivalence and mutated layout, missing/duplicate
tokens, stale symbol/physical/provider context and original receipt/refusal controls before
using this candidate to discharge F03.

Further supplier inspection refines the possible checked-transfer prerequisite. An exact-rational
profile alone cannot cover ordinary current recipes: `literal_admitted` uses `Atom::num(f64)`
outside its small-integral case, and Symbolica 3.0.1 converts that to MPFR-backed Float coefficients.
Preserve the original coefficient realization; converting these to rationals is not equivalent.
The supplier Float representation retains precision plus radix-16 text for both components.
Its later `SerializedFloat::to_float` reaches unchecked Numerica parsing, so a checked loader
must validate that representation before publication, while avoiding unadmitted numeric
materialization during inspection. Exact sources are the native
[literal owner](../../crates/pse-math/src/typed.rs), Symbolica's
[coefficient representation](../../vendor/symbolica/src/coefficient.rs) and
[packed atom coefficient](../../vendor/symbolica/src/atom/coefficient.rs).

The candidate next prerequisite is supplier-owned checked canonical transfer, covering the
actual coefficient population, bounded layout/remapping and preadmitted simultaneous buffers.
An authenticated original canonical producer and independently checked safe packing are distinct
premises. Injective symbol remapping can disturb ordering; comparison can materialize coefficients,
so neither ordinary rename/normalize nor an assumed allocation-free sort discharges the bound.
Explicit unsupported/reorder-required refusal is preferable to an unbounded fallback. Existing
`PreparedBody::compact` demonstrates structural construction without analysis, but intentionally
drops optional views and token inventories; it is not a complete restore constructor.
Supplier transfer and a private full-body restore therefore remain separate prerequisites.
The authority investigation first examines the existing original parent/child channel and exact
consumed receipt associations; no durable/untrusted artifact admission or binary Recipe activation
is selected by these source findings. A new proposed decision must precede that authority change.

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

### Live admitted-body delegation

[Proposed ADR-0246](../adr/0246-live-admitted-body-delegation.md) concludes the bounded next-design
investigation. The first consumer is the existing original private provider-free root child,
whose live parent already owns the admitted body and independently screens the proposal.
`compile_problem` consumes that body, bindings and numerical controls; full receipt/registry
consumption belongs to source reconstruction. Retain complete original admission evidence with
the parent and transfer the exact execution/diagnostic closure plus attempt-bound commitments.
The child performs no source/receipt replay and gains no general scientific or publication
authority. Canonical/durable replay retains ADR-0245's separate requirements.

Supplier packing validation and complete native restoration must precede activation. Preserve
Float realization and every guard, obligation, applicability/branch result and attribution
inventory. Strictly increasing symbol IDs do not preserve canonical sums: mapping 256→511
and 257→512 reverses the little-endian packed term ordering despite equal widths. Require
supplier-established order preservation or bounded repair; source-order registration alone
is insufficient. Subsequent support/derivative/evaluator preparation retains independent
resource obligations. No supplier/native implementation or binary activation is claimed here.
The current consolidation instruction ends this investigation at a reviewable proposed decision;
implementation remains remaining E/F03 scope rather than work opened during qualification.

## Sources

- [Scientific discovery and exact current owner links](../library_leverage/libraries/proto/briefings/math-solvers-and-scientific-workers.md).
- [Buffa 0.9.2 encoding sinks](https://github.com/anthropics/buffa/blob/v0.9.2/buffa/src/encode_sink.rs) and [ConnectRPC 0.9.1 clients](https://github.com/connectrpc/connect-rust/blob/v0.9.1/src/client/mod.rs).
- [Maintained validation contract](../../vendor/protoc-gen-protovalidate-buffa/README.md), [local RPC reference](../libraries/proto/Local_RPC_and_Diagnostics_in_Rust.md) and [Buffa reference](../libraries/proto/Buffa.md).
- Native solver interpretation uses the active native-solver-libraries reference; no solver release/ABI move is selected by this plan. Source pointers: `crates/pse-runtime/src/math/native_child.rs`, `scientific_worker/{protocol,proxy,transport,process}.rs`, `crates/pse-kernels/src/{lib,portable}.rs` and `crates/pse-compiler/src/typed_math_portable.rs`.
