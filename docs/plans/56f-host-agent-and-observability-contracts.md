---
title: "Plan 56f: Host, agent and observability contracts"
status: draft
date: 2026-10-25
adrs: []
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md]
scenario_sources: [docs/design_review/audit/system-architecture.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, diagnostics, environment, preparation]
---

# Plan 56f: Host, agent and observability contracts

## Target and foundations

**Proposed:** finite public operation contracts project faithfully into hosts; boundary meaning
survives generation; telemetry questions/export/loss observations state their real semantics;
native reads pay proportionate preparation without losing source/image or drain guarantees.
This companion owns DA-015/040/042/043 and consumes A/B/C/E products.
[Plan 56](56-design-principle-alignment.md) owns rule decisions and finding status.

Reuse Rust/serde document authority, schemars 1.2.2, the closed existing emitter, strict directional
wire/version/fingerprint checks, generated msgspec contracts, FastMCP 4.1.3 and the derived CLI.
The image witness, source fencing, original-error redaction and existing preparation receipt remain
needed. Intervening runtime-selector exceptions do not eliminate broad per-call identity work.

## Boundary and host design

Under RC02, owner-local serde enums generate their own uniquely scoped Python enum types;
scientific registry vocabularies keep their existing canonical types. Document sets publish only
to their responsibility's target roots. Preserve input/output direction, tagged choice, version
admission, no-Any and generation equivalence; unknown constructs fail generation.

Keep scalar bounds in the owning Rust constraint/constants and derived JSON Schema, emitted as
`Annotated[..., msgspec.Meta(...)]` by the existing generator. Shared constants can be referenced
by schemars range attributes; runtime checked construction remains enforcement. Cross-field
invariants stay with the owner. If bootstrap needs early enforcement, emit the specific shared
predicate from its owned rule rather than maintain an independent host definition or add a general
validation language.

Preserve closed enum-key map meaning in schema and Python. Select `dict[OwnerEnum, Value]`
for the generated Python map: pinned msgspec 0.21.1 decoding preserves known enum keys and
rejects unknown keys. Qualify actual schemars output against serde
for the pinned enum/map shape; use a small owner-derived schema adapter if needed, never erase
keys to unconstrained `str`. Numeric schema annotations alone do not establish Rust validation.
Evidence comparison classifications/status/side and settings samplers become owned closed types.

A finite responsibility-owned operation declaration links public identity, typed request/result,
receiver category, defaults/bounds, effects and actual prerequisite kind. Generate repetitive
CLI/PyO3/facade/MCP metadata/envelopes/adapters; retain small typed receiver invocation glue where
methods genuinely differ. This is the supported public surface, not serialization of every
runtime function or a second workflow language. B's kind lifecycle projection is a different,
scientific contract consumed by this host boundary.

Supplied-only evidence comparison routes through the tooling evidence executable with executable
currency/source admission and no product extension/runtime/store prerequisite. Product operations
retain their real physical/deployment/native requirements. Generated projections replace the
hand `BINDINGS`/worker request-type/default sets and the Plan51 four-host mirrors together.

## Telemetry semantics

Owner event enums/typed indexes supply both counters and declared names. Arrays can remain an
internal layout, but an independent positional zip cannot assign populations. Instrument kind,
monotonicity, temporality, unit and histogram buckets govern valid query operations. A `rate` over
a gauge or `p95` over a non-histogram refuses; the query adapter never invents counter semantics.

Export ordinary declared owner stages through a small marker/constructor and existing filter
layer. Keep required SDK request lifecycle continuity explicitly eligible; finite-capture inner
spans remain available without entering ordinary export by crate prefix. Preserve scientific
meaning and reuse identities independent of telemetry selection.

Select SDK `internal-logs` diagnostics for loss observation using the pinned processor's own
structured events: first queue-drop observed during life; exact cumulative count only if the
orderly shutdown event is observed. Store incarnation/time/scope in existing passive health and
do not feed these diagnostics back into the failing exporter. Sampling, queue loss, export failure,
confirmed ingestion, unavailable and abrupt-loss unknown remain distinct. No continuous exact drop
counter or completeness claim is made.

The SDK's experimental bound metrics are an unselected alternative: its processor captures a
global meter at construction, before current host installation, so merely enabling the feature
can capture a no-op meter. Do not silently mutate globals from `Host::build`, replace the processor
or patch the SDK for this scope.

## Currency and native lifetime

Replace repeated whole-workspace hashing/environment deny-lists using the actually consumed
installed extension/executable's producing Cargo unit/package evidence, attached to the existing
preparation receipt. Cover supported transitive units/features/targets, rustc environment inputs,
build-script declared/default-directory inputs, relevant Cargo config/flags, interpreter/PyO3/
maturin selection and consumed native installation identities. Conservative producing-package
closure is acceptable for supported opaque inputs; a final-crate `.d` file alone is insufficient.

Qualify this finite supported route before removing its old guarantee. This does not revive
ADR-0189/Plan39's withdrawn all-role/source-completeness or strict-producer campaigns. Unknown
unsupported modes refuse explicitly and do not count DA-043 closed. Reuse endpoint-held input/
artifact witnesses, hash changed relevant inputs, and bind admission to the image actually loaded
in the worker. One shared currency contract applies to the evidence executable as well as the
extension; no parallel freshness ledger is added.

Under RC06, retain a foreground endpoint-owned read worker/admitted multi-page workload. Reuse
only exact immutable source/physical/provider/preparation context; page cuts bind inputs, receiver
and worker incarnation. Recheck changed supported basis and image identity before reuse; drain
before replacement. Effects/crash-isolated operations can retain dedicated children with their
actual native owners. Endpoint EOF/shutdown drains its workers; this is not a daemon or persistent
compiler-product cache. Cancellation never implies rollback or releases charged native work before
actual drain. Pool width follows existing admission, not a new workstation CPU/RAM partition.

If RC06 is rejected, fresh dedicated children remain; currency can still improve and one explicitly
admitted page workload can amortize its own preparation without reusing workers across calls.
The reduced scope/cost consequence must remain explicit.

## Packages

### F01 — Scoped boundary enum, constraints and keyed metrics

With E01, extend catalogue contributions/output roots and enum/constraint lowering under RC02.
Migrate telemetry settings and evidence comparison wire vocabularies/key domains. Define owner
event identity at its producer and consume it in counters/declarations/export. Remove duplicated
sampler/bound sets, positional name zips and root semantic telemetry-role declarations after all
consumers migrate. Preserve independent receiving validation and necessary SDK checks.

### F02 — Projected public operations and actual prerequisites

Declare the finite operation surface at its responsibility owner and generate host projections.
Migrate Plan51 operations and the native agent bindings/worker dispatch/defaults; preserve faithful
CLI behavior and structured effects. Add configured capability access from B05 and exact physical/
permission views from A03/B01 as actual consumers, not standalone unused APIs. Split supplied-only
comparison onto the tooling executable. Delete replaced envelopes, mirrors and test fixtures.

### F03 — Typed diagnostics, remedies and metric questions

Consume B06/C02 failures/effects; emit redacted remedies from typed class/stage/recovery facts.
Preserve private supplied-source text. Distinguish observation event names from registry diagnostic
codes and choose suitable diagnostic severity. Admit telemetry question operations from declared
instrument semantics and return honest unknown/unsupported cases. No prose parsing grants retry.

### F04 — Declared span export and SDK loss health

Replace prefix export with owner-stage eligibility and explicit SDK lifecycle selection. Enable
SDK internal diagnostics and capture structured drop events into passive local health at their
actual precision. Guard against recursive export and preserve pure host construction/global
installation order. Delete prefix-based ordinary export assumptions and unavailable-by-build
selection where SDK evidence can truthfully be observed.

### F05 — Producing-artifact currency

First settle a bounded extraction slice for the existing ordinary editable dev/native domain,
including correctness/interactive selection and the existing selected profiles/features, plus the
fixed product/tooling executable build routes. Start from `maturin_cargo_command`,
`capture_build_graph`/`arrow_validation.resolve` and the existing native-test unit/artifact
association: capture the actual selected Cargo unit graph and `compiler-artifact`/
`build-script-executed` records, then qualify dep-info/environment and build-script input
interpretation against this pinned Cargo. JSON artifact records identify outputs; they do not by
themselves enumerate complete inputs. Preserve Maturin-installed byte witnessing after its
legitimate transformations. The extraction decision and revealing controls precede receipt
replacement, not a promise that one final depfile supplies the contract.

Current cross-compilation, custom Maturin/PGO and unmanaged-provider cases retain their explicit
conservative preparation path outside this initial optimization. If extraction is unavailable for
an ordinary supported route, keep its current checks and settle the bounded alternative before
claiming that route corrected; do not expand into an all-role producer proof service.

Qualify the supported producing receipt using both changed relevant and unrelated input controls;
include raced artifact replacement between admission/import. Extend it to the executable route.
Only then remove supersets, deny-lists, second-interpreter import and full unchanged-binary digest
from each read. If evidence exposes an unsupported input class, resolve that finite route or keep
the affected finding obligation open; a receipt that silently omits it is not a replacement.

### F06 — Endpoint/workload-owned read lifetime

Consume E02/E03's materialized deployment/allocation and F05 currency under RC06. Reuse an admitted
read context/pages; bind cursor/worker incarnation and retire on changed premises. Migrate native
worker transport, both dispatch routes and the foreground CLI session; preserve effect isolation,
backpressure and exact protection/cancellation ownership. Delete per-page repeated admission and
redundant imports only when their current guarantees are supplied by the retained owner.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| F01 | planned; 56a physical boundary implemented/tested, RC02 pending | 56f | Scoped closed vocabulary/constraint/key meaning and typed metric populations survive every generated boundary |
| F02 | planned | 56f | Intended public hosts agree on contracts/defaults/effects and actual prerequisites without parallel operation lists |
| F03 | planned | 56f | Typed diagnostic remedies and instrument-correct questions preserve scope and redaction |
| F04 | planned | 56f | Ordinary export follows stage policy; SDK queue loss observed with truthful lifetime/final-count precision |
| F05 | planned | 56f | Supported producing receipt and actual image witness replace per-read supersets/deny-lists without lost inputs |
| F06 | planned; RC06 governs retention | 56f | Admitted read/page lifetime amortizes preparation while source/image, protections and physical drain remain exact |

F01/E01 share the declaration/codegen owner. F02 consumes actual A03/B01/B05 and E02 contracts,
but its finite operation declaration can be designed early. F03 consumes B06/C02/F01. F04 proceeds
once its local vocabulary exists. F05 is independent of worker retention; F06 requires its working
currency and E deployment/allocation, not all E scope. No worker reuse substitutes for stale-image
admission or reopens I10 compiler persistence.

**56a prerequisite — 2026-10-25:** source-owned physical identity declarations emit aliases to
existing Rust wrappers and preserve typed Python domains. Physical inspection request/output
documents use the existing directional emitter; record definitions precede dependency-ordered
evaluated aliases. The actual PyO3 surface, Python facade, MCP binding and derived CLI expose the
same native physical projection. [56a](56a-model-meaning-and-physical-interpretation.md) owns this
slice's acceptance. General owner-local enum/key/bound fidelity, responsibility partition and
generated operation declarations remain F01/F02 work; RC02 and RC06 are not adopted.

## Library basis and acceptance

Context7 was consulted first. Schemars [attributes](https://github.com/GREsau/schemars/blob/master/docs/_includes/attributes.md)
explain serde naming and `range` constraints; exact 1.2.2 installed map/derive sources and tests
settle the selected shape rather than assuming a current documentation example covers it.
The [msgspec supported-type contract](https://jcristharif.com/msgspec/supported-types.html)
and a pure `scripts/pse-env -- python` probe under installed 0.21.1 established a known string-enum
key roundtrip and unknown-key rejection. Its standalone JSON Schema left key constraints implicit,
so the Rust-owned schema/emitter must preserve that vocabulary explicitly; Python decoding alone
does not qualify host schema fidelity. This did not import the product extension or start a host.
Cargo's [build-script contract](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/build-scripts.md)
and [unstable freshness documentation](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/unstable.md)
separate compile inputs from build-script change detection. Existing repository generation and
receipt owners remain the integration authority.

Exact SDK 0.33.1 `trace/span_processor.rs` emits
`BatchSpanProcessor.SpanDroppingStarted` and `BatchSpanProcessor.SpansDropped` with
`dropped_span_count`; its installed source outranks behind-main documentation. The
[SDK logging design](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md)
provides context, not a claim of complete ingestion. No library was bumped; the bounded msgspec probe above establishes only its stated decode behavior.

**Planned:** independent expected schema keys/defaults/effects across actual hosts; unknown enum/map
keys and invalid numeric/cross-field values; metric reordering and inadmissible queries; coarse
versus finite inner spans; deterministic queue pressure showing first-drop versus orderly final
count and abrupt-loss unknown. Currency controls separately change consumed source/env/native
inputs and an unrelated workspace leaf, replace the installed image during admission, reuse pages,
cancel reads and replace workers while protections survive. A supplied-only comparison must work
without product-extension/store startup. No source-only pass establishes current call latency,
full native qualification or unexercised supported input modes.
