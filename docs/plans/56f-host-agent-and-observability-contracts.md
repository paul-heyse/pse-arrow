---
title: "Plan 56f: Host, agent and observability contracts"
status: complete
date: 2026-10-25
adrs: [ADR-0227, ADR-0231]
review_sources: [docs/design_review/audit/README.md, docs/design_review/audit/system-architecture.md, docs/design_review/reviews/design_review_retained-native-read-currency_2026-10-26.md]
scenario_sources: [docs/design_review/audit/system-architecture.md, docs/design_review/reviews/design_review_retained-native-read-currency_2026-10-26.md]
doc_role: plan
doc_owner: docs/plans/56-design-principle-alignment.md
doc_topics: [design, diagnostics, environment, preparation]
---

# Plan 56f: Host, agent and observability contracts

## Completion audit — 2026-10-27

**Historical audit:** the following observations record the inspected input cut. Current repair acceptance and completion are recorded in the [audit-gap closure Outcome](#audit-gap-closure-outcome--2026-10-27); historical failures and bounded passes remain evidence.

**Interface-checked:** all F01–F06 accepted scope and ordinary consumers inspected at
`57cce1756318357a5797f99e83f1ad9115661368`, with revised ADR-0231/review acceptance.
F02–F04 retain bounded completion. F01, F05 and F06 remain incomplete for the specific obligations
below. The coordinator owns DA-040/043 disposition and [CA04–CA08](56-design-principle-alignment.md#completion-audit--2026-10-27).
Earlier passing receipts retain their original conditions; the prior blanket F01–F06 closure is withdrawn.

### F01: finish actual owner and consumer adoption (CA04/CA05)

`pse-runtime/src/tooling/telemetry.rs` still declares the sampler as `String` and validates six
names and scalar maxima manually. `scripts/telemetry_config.py` repeats the sampler/bound sets;
`telemetry_process.py` selects sampler behavior with substring tests. The generated settings keep
`sampler: str` and merely nonnegative constraints where the owner requires positive values and
maxima. Accepted F01 explicitly required removing these parallel definitions. Finish the owner-local
closed sampler and scalar constraint/constants, derive schema/Python and migrate actual host
selection/enforcement. Preserve independently necessary receiving enforcement against that declaration.
No generic validation framework or duplicate artifact validation is requested.

`xtask/src/development/evidence_query/comparison.rs` retains finite comparison `classification`,
`correspondence` and `side` as strings. Its Rust request owns a map keyed by
`EvidenceComparisonSection`, but published request schema has no closed property-name domain and
generated Python uses `dict[str, ...]`. Output schemas/Python also erase finite classification/side
meaning. Migrate these actual owners and generated consumers to their closed vocabulary/key domains;
preserve original owner statuses that are legitimately open. The generator capability alone does
not complete this adoption. Targeted controls should decode real production declarations at each
boundary, cover unknown keys/variants and scalar endpoints, and verify unchanged valid semantics.

**Tested, narrow discrepancy:** on this local Linux checkout, the actual generated JSON Schema and
msgspec request decoder admitted `sections: {"not_a_section": {"limit": 1}}` (zero schema errors).
The already published pure evidence executable, opened through its held descriptor, refused the
same request at native document decoding (exit 1) with a valid retained supplied snapshot. No store,
product extension or scientific execution was used. The first executable probe omitted required
`--input` and exited 2 before decoding; the corrected probe supplied the snapshot and settled the
question. This is contract-discrepancy evidence, not an executable malfunction or full evidence journey.

### F05: honor explicit requested compatibility (CA06)

`python_extension.run_stages("admit", mode, profile, ...)` invokes `admit_import` without the
requested profile. For dev, `capture(native=False)` intentionally consumes the installed publication's
mode, and `currency_check_installed` consumes its profile. `PreparationResult` nevertheless reports
the caller's requested mode/profile. The explicit public StageRequest thus can report dev/dev while
selecting a native/producer image. Finish explicit-stage mode/profile propagation and truthful
result reporting, with compatible and incompatible requested-stage controls. Keep generic installed
capture semantics: an issued artifact is valid even when incompatible with this specific request.

**Tested, selection only:** `scripts/pse-env -- .venv/bin/python /tmp/pse-56-audit-probes.py` used a
private temporary typed native/producer publication and called the real `run_stages` for dev/dev.
It returned exit 0, reported dev/dev and passed the native image to its one import call. Only
`subprocess.check_call` was stubbed: this demonstrates selection/reporting, not an actual binary import.
The fixture was removed. Successful constructor publication/failure preservation remains supported.

### F06: finish foreground CLI adoption and required journey evidence (CA07/CA08)

`capability_cli_runtime.invoke_on` calls `capabilities_describe` on every operation/page and refuses
`interface.restart_required`. Description calls `Interface.status`, which reads/hashes serving
sources through `Sources.check`. Direct MCP dispatch's scan-free `require` does not remove this
normal CLI preflight. Its selection receipt also calls `Interface.status`; deleting only the
description refusal would leave those repeated scans. Remove source freshness from automatic operation admission while retaining
the distinct loaded-wire compatibility check and explicit owner-controlled drain/reload. Optional
explicit provenance diagnosis can retain its separate purpose. Cover the actual CLI invocation/session
route, including successive operations and a changed-source observation that cannot revoke the loaded
contract. Do not infer safe arbitrary Python editing from the absence of a watcher.

**Tested, in-process public route:** the same probe used actual FastMCP server/client and `invoke_on`:
two successful `capability_readiness` calls caused four `Sources.check` calls through automatic
description and selection. Supplying a changed-path
observation at that seam caused stale-endpoint refusal. A direct MCP call succeeded with zero
additional source checks. No source files were edited. An earlier harness incorrectly expected two
checks and aborted after the CLI observation; the corrected observation counts all actual calls.
Logs are `/tmp/pse-56-completion-audit-probes.log` and
`/tmp/pse-56-completion-audit-probes-final.log`; the settled native decoder probe is
`/tmp/pse-56-completion-audit-evidence-probe.json`. Expected product discrepancies were reproduced;
this is not host attachment, a native functional run or broad qualification.

**Qualification outstanding (CA08):** revised review S04 and its acceptance require actual explicit
new-generation selection/reload with native drain and original cursor/incarnation boundaries.
Located unit controls establish identity separation/retention, actual controls replace a mutable
installation alias, and actual EOF/fresh attachment establishes settlement. They do not exercise
publishing/selecting a second issued generation while an old cursor is retained. Complete that
specific actual journey through the intended foreground/native route, preserving original deadlines,
physical custody and reference lifetime. No implementation defect in the core retention mechanism is
asserted from this evidence gap. [56g](56g-qualification-and-publication.md#completion-audit--2026-10-27)
owns the joined acceptance; no broad campaign is required by this audit.

**Supported scope and benefits:** finite projections, typed remedy consumers, instrument-correct
questions, precise SDK observations, immutable construction/publication, actual retained native
context/protection/cancellation/EOF and supplied-only separation retain their recorded evidence.
The B06 producer classification defect belongs to B06, not a separate F03 reopening. At the audited cut, universal source-scan
removal was not established; current adoption is recorded in the closure Outcome. No speed, memory or capacity improvement was measured.

## Audit-gap implementation — 2026-10-27

Historical implementation checkpoint; current completion and qualification are in the closure Outcome below.

The maintainer authorized CA04–CA08 closure. Pinned Schemars/serde declarations now own the actual
sampler, scalar limits and finite comparison judgments. A schema-only owner adapter preserves enum
map keys without mirroring their strings; the existing emitter projects typed Python keys. Generated
msgspec constraints and type inspection drive telemetry receiving admission and environment limit
conversion. Original open statuses, valid wire spellings and document version 1 remain intact.

Explicit stages select exact mode/profile/validation compatibility while generic installed capture
keeps its meaning. Automatic interface projections inspect loaded wire contracts rather than serving
sources. The foreground client uses a scoped FastMCP transport and requests native drain before the
SDK starts disconnect: the requestless `capabilities_drain` control closes admission, joins original
owners and acknowledges only their successful settlement. Concurrent close observers now share one
manager drain; cancelled observers and sibling failures cannot permit an early successor.

Focused generated-owner, SDK-adapter, explicit-stage and host-lifetime controls pass. Independent
review found the concurrent-close issue, which was repaired and reassessed without a remaining
material source finding. Final regeneration, actual two-publication/cursor/CLI/EOF acceptance and
shared scope-end checks are complete at the conditions in the closure Outcome below.

## Target and foundations

**Proposed:** finite public operation contracts project faithfully into hosts; boundary meaning
survives generation; telemetry questions/export/loss observations state their real semantics;
native reads consume owner-published immutable generations and retain admitted preparation through
its useful lifetime, preserving original context, scientific permission and physical drain.
This companion owns DA-015/040/042/043 and consumes A/B/C/E products.
[Plan 56](56-design-principle-alignment.md) owns rule decisions and finding status.

Reuse Rust/serde document authority, schemars 1.2.2, the closed existing emitter, strict directional
wire/version/fingerprint checks, generated msgspec contracts, FastMCP 4.1.3 and the derived CLI.
Keep original-error redaction and the existing preparation receipt. Under amended ADR-0231 and the
[attributed focused review](../design_review/reviews/design_review_retained-native-read-currency_2026-10-26.md),
controlled construction establishes artifact validity once and atomically publishes its typed
contract. Consumers attach the issued immutable generation; reads and internal handoffs do not
reconstruct source/build currency or revalidate its image. Distinct requested-interface/mode
compatibility, external request decoding and scientific/context/lifetime boundaries retain their
own duties. F05/F06 are substantially delivered; the completion audit identifies their remaining adoption and acceptance.

## Maintainer-directed artifact amendment — 2026-10-27

Controlled construction and editing establish artifact validity once. F05/F06 now target
ADR-0231's owner-published immutable generation contract. The amended target removes
source/environment/Cargo-record rescans and repeated image validation from reads. Build/install/publication failures
preserve the preceding generation; only a completed generation is published. A consumer attaches
that generation, checks distinct requested compatibility, and retains it through actual drain.
Serving-code edits require explicit drain/reload, including lazy imports. Source edits, linting,
mutable installation aliases and later publication do not revoke an attached generation.

Earlier source-change and alias-change refusal obligations below record the original execution
cut and are superseded where they conflict with this amendment. Qualification must exercise
publication failure preservation, lint/source independence and explicit replacement without cursor
rebinding. The original request/cursor/physical/scientific/Arrow boundaries remain separate.

## Boundary and host design

The selected RC02 implementation supplies responsibility-scoped catalogues, owner-local serde
enums with uniquely scoped Python enum types, and constraint lowering. Scientific registry
vocabularies keep their canonical types; document sets publish only to their responsibility's
target roots. [56e's Outcome](56e-product-deployment-and-resource-ownership.md#verification-and-outcome--2026-10-26)
owns that E01/F01 delivery and its qualification. Preserve input/output direction, tagged choice,
version admission, no-Any and generation equivalence; unknown constructs fail generation.

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

Supplied-only evidence comparison routes through the tooling evidence executable's published generation,
without a product extension/runtime/store prerequisite. Product operations
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
SDK shutdown-attempt diagnostic is observed, independently of shutdown success. Store incarnation/time/scope in existing passive health and
do not feed these diagnostics back into the failing exporter. Sampling, queue loss, export failure,
confirmed ingestion, unavailable and abrupt-loss unknown remain distinct. No continuous exact drop
counter or completeness claim is made.

The SDK's experimental bound metrics are an unselected alternative: its processor captures a
global meter at construction, before current host installation, so merely enabling the feature
can capture a no-op meter. Do not silently mutate globals from `Host::build`, replace the processor
or patch the SDK for this scope.

## Currency and native lifetime

Use the existing preparation owner/receipt to issue a typed published artifact contract. Explicit
preparation runs the trusted build/install route, records the resulting mode/profile, validation
and compiled features, and copies the successful final extension or executable into a private
immutable generation. Publish its descriptor atomically only after construction and copy complete.
Build, installation, copy or publication failure preserves the preceding publication; incomplete
candidates never become selected. The owner also controls replacement and reclamation, retaining
referenced generations and provider resources through attached consumers' actual drain.

Construction observations such as Cargo unit/artifact records, stdout, dep-info and input/environment
selection may support construction or diagnose failure. They are not ongoing consumer proof
obligations. Remove source/environment/Cargo-record scans, `Witness.inputs`, image digests/fences
and redundant construction validation from read admission and internal handoffs. Do not replace the
same scanner with a new typed provenance framework or ledger. A source edit affects a future build;
it does not invalidate an older successfully issued generation or imply semantic equivalence of
the edited source. Lint rewrites and mutable build/install aliases likewise cannot revoke it.

A consumer attaches the exact immutable generation path once. Check distinct requested interface
or mode compatibility at attachment only where an independent requested contract can differ;
projections from the same admitted owner do not require duplicate agreement checks. Missing
publication or incompatible requested capabilities require explicit owner preparation/selection,
not hidden read-time rebuilding or source freshness inference. The fixed evidence executable uses
the same publication lifecycle without acquiring product-extension/store prerequisites.

Under RC06, the foreground endpoint/workload owns the attached read worker and exact admitted
physical/provider/preparation/deployment context. Bind changing requests/page cuts, receiver and
cursor to that worker incarnation; a newer publication never silently rebinds existing cursors.
An attached generation remains usable while its owner allows it. Explicit replacement drains the
old worker before attaching the selected generation. Python serving-code edits require drain and
reload before edited code serves operations, including lazy imports; source scanning does not
substitute for that controlled editing contract.

Preserve necessary external request decoding, operation/response correlation, original physical
and scientific permission, deadlines and context transitions. These do not repeat artifact validity
checks. Effects/crash-isolated operations may retain dedicated children with their actual native
owners. EOF/shutdown drains endpoint workers; cancellation neither implies rollback nor releases
charged native work before settlement. Existing shared admission covers retained preparation,
active/queued work and responses without a new workstation CPU/RAM partition. This is not a daemon
or persistent compiler-product cache. An explicitly admitted multi-page workload remains a
conforming fallback where cross-call worker retention is unnecessary; it consumes the same issued
publication without restoring per-read freshness validation.

## Packages

### F01 — Scoped boundary enum, constraints and keyed metrics

Consume E01's implemented catalogue contributions/output roots and enum/constraint lowering under
RC02. Complete remaining telemetry vocabulary/key-domain adoption and define owner event identity
at its producer, then consume it in counters/declarations/export. Remove remaining duplicated
sampler/bound sets, positional name zips and root semantic telemetry-role declarations after all
consumers migrate. Preserve owner-local closed evidence comparison types, independent receiving
validation and necessary SDK checks. The delivered generator slice does not complete typed metric
population or telemetry adoption.

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

### F05 — Owner-published immutable artifact generations

Implement amended ADR-0231 for ordinary editable dev/native construction, interactive/correctness
selection and the existing selected profiles/features, plus fixed product/tooling executable
routes. Reuse the existing preparation receipt as the typed published descriptor. Successfully
construct/install and copy the final image into a private immutable generation before atomically
publishing its identity/path, mode/profile, validation and compiled capability selection.
Construction/publication failure leaves the previous issued generation usable. Reclamation must
respect attached owners; partial candidates are never an admission option.

Consumers use the published contract and attach its exact immutable path. Keep only genuinely
distinct requested-interface/mode compatibility at attachment. Delete read-time producer/source/
environment/dep-info scans, `Witness.inputs`, secondary-interpreter imports and image digests/fences,
including obsolete collectors/callers/fixtures with no remaining construction or diagnostic use.
Do not qualify a universal extractor or retain a parallel fallback scanner for the ordinary route.
Cross-compilation, custom Maturin/PGO and unmanaged-provider cases retain their explicitly scoped
construction policies outside this initial route; they do not authorize expanding this scope.

Targeted acceptance exercises successful extension/executable construction, incomplete candidate
exclusion and failed build/install/copy/publication preservation of the old generation. Lint,
source/environment changes and mutable installed aliases must leave an issued generation usable;
they need not be classified as semantically equivalent source edits. Challenge actual distinct
interface/mode incompatibility where present and prove consumers use the issued immutable path.
The old source/producer/alias-change refusal criteria are superseded, not retained as guarantees
to reproduce. F05 remains in progress until these actual construction/attachment controls pass.

### F06 — Endpoint/workload-owned read lifetime

Consume E02/E03's materialized deployment/allocation and F05's issued immutable generation under
RC06. Attach once and carry the admitted typed owner through reads and internal handoffs without
artifact revalidation. Preserve original physical/provider/preparation context and opaque cursor/
worker incarnation, independently bound requests/responses, original deadlines and scientific
permission. Source edits, alias changes and a newer publication do not implicitly retire or rebind
the attached generation. Explicit generation selection or serving-code reload drains/replaces the
owner; edited Python code, including lazy imports, serves only after that explicit transition.

Migrate both dispatch routes, native worker transport and foreground CLI session. Preserve effect
isolation, backpressure, original protection/cancellation ownership and actual settlement on EOF,
shutdown and replacement. Delete replaced per-page admission/import/freshness mechanisms and their
obsolete refusal tests. Targeted actual page/context/cursor/cancel/EOF/reload/replacement controls
must establish the amended lifetime; earlier passing image/source refusal controls do not do so.
F06 remains in progress until those controls and the final canonical protected-cursor journey pass.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| F01 | complete — CA04/CA05 Implemented/Tested | 56f | Actual telemetry/evidence enums, numeric constraints and finite map keys are adopted through generated and host consumers |
| F02 | complete at installed/native, supplied-only executable/MCP/CLI and public project journeys | 56f | Finite declarations project contracts/defaults/effects and actual prerequisites without parallel operation lists |
| F03 | complete at typed remedy and instrument-query acceptance | 56f | Typed diagnostic remedies and instrument-correct questions preserve scope and redaction |
| F04 | complete at SDK/filter, installed disabled bridge and selected capture acceptance | 56f | Ordinary export follows stage policy; queue-loss observations retain actual lifetime/shutdown-attempt precision |
| F05 | complete — CA06 Implemented/Tested | 56f | Explicit requested mode/profile/validation is enforced; constructor publication remains authoritative |
| F06 | complete — CA07/CA08 Implemented/Tested | 56f | Scan-free loaded contracts, one original drain and actual two-generation/native/CLI ownership acceptance pass |

F01/E01 share the declaration/codegen owner. F02 consumes actual A03/B01/B05 and E02 contracts,
but its finite operation declaration can be designed early. F03 consumes B06/C02/F01. F04 proceeds
once its local vocabulary exists. F05 construction/publication is independent of worker retention;
F06 consumes its issued generation and E deployment/allocation, not all E scope. Both remain in
progress until the amended target's actual controls pass. Retention does not add producer
revalidation or reopen I10 compiler persistence.

**56a prerequisite — 2026-10-25:** source-owned physical identity declarations emit aliases to
existing Rust wrappers and preserve typed Python domains. Physical inspection request/output
documents use the existing directional emitter; record definitions precede dependency-ordered
evaluated aliases. The actual PyO3 surface, Python facade, MCP binding and derived CLI expose the
same native physical projection. [56a](56a-model-meaning-and-physical-interpretation.md) owns this
slice's acceptance. At this checkpoint, general owner-local enum/key/bound fidelity, responsibility
partition and generated operation declarations remained F01/F02 work, and RC02/RC06 were not
adopted. The selected D/E contribution below records the subsequent catalogue/codegen delivery;
operation declarations and RC06 retention remain open.

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
separate compile inputs from build-script change detection. These sources supported the earlier
producing-input candidate; they do not require read-time reconstruction under the amended target.
Existing generation and preparation/publication owners remain the integration authority.

Exact SDK 0.33.1 `trace/span_processor.rs` emits
`BatchSpanProcessor.SpanDroppingStarted` and `BatchSpanProcessor.SpansDropped` with
`dropped_span_count`; its installed source outranks behind-main documentation. The
[SDK logging design](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md)
provides context, not a claim of complete ingestion. No library was bumped; the bounded msgspec probe above establishes only its stated decode behavior.

**Planned:** independent expected schema keys/defaults/effects across actual hosts; unknown enum/map
keys and invalid numeric/cross-field values; metric reordering and inadmissible queries; coarse
versus finite inner spans; deterministic queue pressure showing first-drop versus shutdown-attempt cumulative
count and abrupt-loss unknown. Artifact acceptance follows the revised review's S02–S04: successful
construction publishes only a complete immutable generation; build/install/copy/publication failures
preserve the preceding publication; lint/build-control/source/environment changes and mutable
installation aliases do not revoke or redirect the issued generation. No post-publication digest or
producer rescan is part of acceptance. Challenge genuinely distinct requested-interface/mode
compatibility at attachment where that boundary exists. Review S05–S06 preserve original context,
pages/cursors, deadlines, cancellation, EOF and native drain during explicit reload/replacement.
Exercise the actual extension/executable routes and both dispatch routes plus foreground CLI;
a supplied-only comparison must work without product-extension/store startup. Replace obsolete
source/producer/alias refusal tests with these construction/lifetime controls and delete the old
mechanism after its consumers migrate. No source-only pass establishes current call latency, full
native qualification or unexercised supported build modes.

## Selected 56b adoption — 2026-10-25

**Implemented/Tested:** [56b Outcome](56b-scientific-operations-and-permission.md#verification-and-outcome--2026-10-25) owns current completion/product permission, required qualifiers,
strict replay/profile versions and executable direct capability settings through generated
Rust/Python contracts. Typed failure classes, source references and admission effect knowledge
feed redacted host remedies. The installed correctness extension, actual STDIO MCP and derived
CLI agree on an original failed model's permission and retained diagnostic checks; host contract
unit controls remain separate evidence. This completes only B's necessary F01/F03 adoption.
At this checkpoint, general declaration/metric fidelity, all-host operation envelopes, wider
settlement/telemetry, image-currency redesign and reused page/worker lifetimes remained planned;
RC02/RC06 were pending. Subsequent D/E delivery supplies the selected RC02 catalogue/codegen
prerequisite below, with the wider F obligations still open.

## Selected 56c prerequisites — 2026-10-26

**Implemented/Tested:** [56c Outcome](56c-canonical-contracts-and-effects.md#verification-and-outcome--2026-10-26) owns canonical key/vocabulary/association generation, exact attributed settlement with both original/readback causes and selected installed Python/STDIO MCP/CLI agreement. These supply F01/F03 prerequisites. At that checkpoint, responsibility-scoped catalogue generation and RC02/RC06 were pending. D/E subsequently supplies the selected catalogue/codegen prerequisite below; wider host agreement, metric/telemetry semantics, image-witness redesign and retained page/worker lifetimes remain F01–F06 work. The public acceptance does not establish desktop-host attachment or end-to-end trace export.

## Selected 56d/56e prerequisites — 2026-10-26

**Implemented/Tested:** [56e's Outcome](56e-product-deployment-and-resource-ownership.md#verification-and-outcome--2026-10-26)
owns the selected responsibility-scoped catalogue and product/tooling partition, owner-local enums,
constraint lowering, migrated Python/host enum consumers and full ten-target generation. This is
the delivered E01/F01 generator slice under selected RC02/RC03; it replaces the earlier pending
prerequisite statements without closing F01's metric work. Materialized production allocations,
frozen prepared-input closure, shared leaf observations and the acyclic service borrower boundary
are available to remaining host work. Exact-resource orphan recovery preserves original failure
outcomes and custody; it does not establish retained endpoint/page workers or producing-artifact
currency.

[56d's Outcome](56d-publication-and-workload-fit.md#verification-and-outcome--2026-10-26)
and 56e own their selected workload, recovery and ownership evidence. At that prerequisite cut,
the selected installed public tests passed 8/8 and native parent/history controls 4/4. Ordinary
reference activation was blocked before initialization by a preserved unrelated live legacy service,
as recorded at E's Outcome; actual receiver activation was unqualified. Finite operation projections,
typed metric populations/questions, span/loss telemetry, producing-artifact currency and retained
read/page lifetimes were still open, and RC06 was a separate coordinator rule choice. These describe
the original prerequisite cut; the current F package state and consolidated outcome follow below.

## Verification and Outcome — 2026-10-26

**Historical evidence cut:** the Outcome below records the original implementation and tests at
its named source/installed-image conditions. Its producer/source/environment/installed-alias refusal
criteria are superseded by the 2026-10-27 maintainer amendment and revised F05/F06 definitions.
Those recorded passes/failures remain unchanged and do not qualify immutable publication or its
explicit reload/replacement lifecycle. Current package state and planned acceptance above govern
the remaining work; both F05 and F06 remain in progress.

**Historical implementation account:** the selected F01–F06 slices were integrated at the coordinator's `1e5a6e79a` source
baseline. The completion audit identifies remaining F01/F05/F06 obligations. RC02/RC03 and selected RC06 supply responsibility-owned contracts and foreground read
retention under proposed ADR-0231. Earlier dated prerequisite sections describe their original cuts;
their pending statements do not override this package table and outcome. Plan56 owns finding/rule
dispositions; [56g](56g-qualification-and-publication.md) owns assembled qualification and comprehensive
static-check accounting. This companion remains in progress for the remaining final journeys below.

**Implemented:** all 53 existing project/research/study Python/PyO3 wrappers and the 21 runtime plus
one tooling native-agent operation rows derive from their responsibility owners. Their typed receiver
glue, original defaults, Rust-owned resource budgets, structured effects and actual prerequisites
remain distinct. The replaced wrapper bodies, private command envelope, handwritten host
envelopes/default sets and positional metric-name assignments are removed. Plan51 remains closed;
this work aligns existing functionality. Shared modeling `Limits` validates positivity before
compilation, reuse or native specialization; the duplicate remote declaration is removed.

**Implemented:** ordinary Rust export requires the exact `pse.owner.stage` marker, with the exact
SDK request lifecycle explicitly eligible; finite capture retains its separate inner-event contract.
SDK 0.33.1 structured diagnostics record first queue-drop observation during process life and a
cumulative dropped-span count only when the shutdown-attempt diagnostic is observed, before
shutdown control is sent or awaited. Incarnation, clock and scope survive passive health. The count
does not establish successful shutdown, queue drain, export or ingestion; shutdown status remains
independent and abrupt-exit buffered loss stays unknown. Python SDK queues remain unavailable.

**Implemented:** ordinary producing receipts retain actual selected Cargo units/artifacts/dep-info,
compiler and build-script input meaning, selected tools/environment/provider identities and the
Maturin-installed byte association in the existing owner receipt. Supported retained reads reuse
unchanged input/image witnesses without a whole unchanged source/binary byte hash or a second
interpreter import on each page. Unsupported producing observations retain conservative preparation
or refuse; no parallel ledger or all-role producer-proof campaign is introduced. Retained workers
own actual immutable Runtime/physical/modeling objects, source and loaded-image witnesses, and
owner-declared opaque cursor associations. Offsets remain selectors. Changed premises retire the
worker; observer cancellation retains charged work until settlement and EOF closes the actual owners.
Mutable diagnostics, effectful operations and supplied-file/tooling routes retain dedicated lifetimes.

**Implemented:** final integration corrected the evidence executable's compiler context under
`ba87baa51`. Its build and executable children consume the same passive composition without local
shell selection, remove inherited `LD_LIBRARY_PATH` and track that normalized absence. The parent
native environment is unchanged. Actual Cargo-declared environment inputs remain checked; unrelated
native-provider, BINDGEN and CMake selections are not added as broad supplemental inputs to the pure
tooling route. Older receipts require explicit owner rebuild before reuse.

**Implemented:** protected canonical reads now distinguish a strict study conclusion from a
scientific completion receipt. Study parents carry `[1, Conclusion]`; compact indexed native
parent-run/revision association and the original strict owner decoder preserve that meaning.
A study conclusion yields no scientific receipt or product-permission claim. Malformed or
misassociated tuples refuse; original object-shaped scientific/recovery receipt behavior remains.
This correction is integrated, with fresh composed canonical acceptance still pending below.

### Selected functional evidence

The failure baseline for each selection is zero. Counts below name the exercised population;
filtered-out tests are outside that selection, not additional passing evidence.

- **Tested:** `cargo nextest run -p pse-runtime --locked --no-fail-fast` with explicit
  `pse-runtime/force-validate` and the SDK-loss, production-span-filter, operational/runtime metric,
  preparation-key and finite metric-domain filter passed **18/18**, with 1,040 outside the filter.
  `/tmp/pse-56efg-pure-sdk-metric-tests.log` retains the exact filter. The selected
  `pse-operations` canonical metric-key/owner/statistics filter passed **5/5**, with 165 outside the
  filter, in `/tmp/pse-56efg-canonical-metric-controls.log`.
- **Tested:** selected instrument-query/passive-health controls passed **19/19** in
  `/tmp/pse-56efg-query-health-controls.log`. Runtime modeling-limits, preparation controls,
  project defaults/effects/projection, exact IPC export and host prerequisite/preservation controls
  passed **6/6**, with 1,048 outside the filter, under the `pse-runtime` nextest selection recorded in
  `/tmp/pse-56efg-runtime-operation-limits-controls.log`.
- **Tested:** actual Maturin preparation/install/admission exercised all four ordinary dev/native ×
  interactive/correctness modes at profile `dev`, using `producing-cargo-inputs-v1`, at their earlier
  recorded source cuts. The earlier native correctness sync installed force-validation and native features with SHA-256
  `bf695c09b0a7be3dad512f1d6e0045438420eadcf6a010622e679b57140a26a4`;
  `/tmp/pse-56efg-final-native-correctness-sync.log` records the prepared artifact and admission.
  These qualify their recorded routes and unchanged mechanisms; they do not establish all four modes
  at the latest integrated source, every profile or unsupported build mode. The latest native
  correctness rebuild is recorded separately below.
- **Tested:** actual dev-interactive and native-correctness receipt controls distinguished changed
  consumed Rust from an unchanged unselected leaf, compiler-environment changes, selected native
  provider receipt overlays, artifact byte replacement and an isolated OS inode-replacement race.
  The native provider overlay was selected only for the native receipt. Real reversible source bytes
  were restored and final original-owner admission returned current. Evidence is in
  `/tmp/pse-56efg-actual-dev-interactive-receipt-controls.log` and
  `/tmp/pse-56efg-actual-native-correctness-receipt-controls-qualified.log`. The image-race fixture
  is disposable; these controls do not claim mutation of a managed native installation or receiver
  execution. Actual tooling executable build and receipt controls passed in
  `/tmp/pse-56efg-actual-evidence-build.log` and `/tmp/pse-56efg-actual-evidence-receipt-controls.log`.
- **Tested:** the actual supplied-only evidence executable, MCP and foreground CLI agreed while
  the product extension was intentionally unloadable and the canonical store absent. The receipt
  `build/qualification/56efg-evidence-supplied-11481da9a635/receipt.json` records passed routes,
  zero failures, no product-extension import and no canonical-store creation. Its independently
  supplied metadata selection establishes no scientific execution or backend ingestion.
- **Tested:** `scripts/pse-env --native -- .venv/bin/python -m unittest` selecting
  `scripts.tests.test_native_read_native.{ActualNativeContextTests,ActualNativeEndpointTests,ActualNativeCliSessionTests}`,
  `scripts.tests.test_telemetry_process.ProcessTests.test_actual_disabled_native_bridge_retains_typed_passive_health`
  and `scripts.tests.test_telemetry_profiles.CaptureBridgeTests` passed **8/8** in
  `/tmp/pse-56efg-final-native-read-cli-health-capture-controls.log`. This exercises actual retained
  native objects, both MCP dispatch routes, foreground CLI EOF, the disabled installed health bridge
  and capture-only cancellation ownership; it is not an actual desktop-host attachment or trace
  ingestion campaign. This selection retains its original installed-image/source conditions;
  it is not reattributed to the later study/evidence corrections.
- **Tested:** the two `ActualNativeAdverseTests` passed **2/2** in
  `/tmp/pse-56efg-actual-native-read-image-cancel-drain.log`: owned installed-image replacement
  retires the old worker and restores original inode/bytes; authenticated pidfd SIGSTOP leaves a real
  request queued while observer cancellation/endpoint EOF retain worker allocation until actual drain.
  The queued control does not establish entry into Rust or a numerical solver.
- **Tested:** installed-dev source/context/MCP/CLI and exact compiled-catalogue controls passed
  **4/4**. The wrong-effect negative uses the actual compiled catalogue return and changes only its
  selected fact. Actual installed-dev project facade tests in
  `python/pse/tests/test_project_operation_facade.py` passed **9/9** using a fresh owned pytest
  base directory; `/tmp/pse-56efg-actual-dev-project-facade-owned-temp.log` retains that selection.
- **Tested:** the evidence executable's owned pure compiler-context controls passed **30/30** in
  `/tmp/pse-56efg-evidence-owned-pure-context-controls.log`. Compiler and runtime loader probes with
  no `LD_LIBRARY_PATH` passed in `/tmp/pse-56efg-evidence-pure-loader-probes.log`. These establish
  the selected environment correction; old-receipt replacement and fresh composed supplied-only
  acceptance still require the explicit build and final journey under their owner.
- **Tested:** original study-conclusion owner controls passed **3/3** in
  `/tmp/pse-56efg-original-study-conclusion-controls-final.log`, including strict decoding and
  original parent association. The latest native-correctness rebuild in
  `/tmp/pse-56efg-final-study-repaired-native-correctness-sync.log` installed SHA-256
  `af05e8761b9956fc914e11f0868e85590c64af8c54d5d6daf9028f6ce019d39f` with
  `force-validate`, `native-solvers` and `solver-highs`. Installation/admission is separate from the
  pending current-source canonical and project journeys.

### Corrections and qualification limits

**Interface-checked/Implemented:** independent source review exposed three structured-effect
mismatches and a copied `8192` engine batch default. The responsibility owners and projected consumer were corrected;
the subsequent review found no additional material gap in its examined contracts at `30f1`.
This is source-review judgment and focused regression evidence, not whole-system qualification.

**Interface-checked:** the subsequent independent review at `1e5a6e79a` found the loader-context and
study-conclusion corrections resolved within its examined paths, with no additional material gap.
Fresh actual-host behavior remains a separate acceptance obligation. The reference receiver's
successful activation belongs to [56e's Outcome](56e-product-deployment-and-resource-ownership.md#verification-and-outcome--2026-10-26);
it does not substitute for F's final canonical or public project journeys.

**Implemented — deliberate scope choices:** SDK experimental bound metrics remain unselected;
loss health consumes the pinned processor's structured diagnostics without replacing its processor
or installing globals during host construction. Foreground retention follows existing admission,
while unsupported producing modes keep conservative owner checks. Scientific outcomes and retained
result protections remain with their original owners.

**Tested:** preserved failed attempts include the original Cargo directives-path probe, a private
read-only receipt-copy permission error repaired only in its driver, supplied-evidence public
fingerprint extra-key refusal repaired in its driver, and compiled catalogue tuple/list mismatch
repaired through the original codec's JSON normalization. The first project facade pytest attempt
ended with a shared temporary-directory ownership `INTERNALERROR` before tests ran; the successful
selection used a fresh owned base directory without altering the shared path. None of these attempts
is counted as a pass or erased by its later focused correction.

**Tested — preserved composed failure:**
`build/qualification/56efg-canonical-retained-final/receipt.json` records **1/3 passed, 2 failed**
against the zero-failure baseline: project selection passed; knowledge currency falsely reported
stale and nested result inspection failed. The diagnosis repeated the original checks through a
private wrapper in `/tmp/pse-56efg-canonical-decoding-diagnosis.log`. That failed receipt remains
failed after the loader-context and strict study-parent corrections; it is not a fresh acceptance
receipt. Two initial SQL lifetime compile failures and a correct moving-source preparation refusal
also precede the successful three-control study selection and retain their original outcomes.

**Tested:** the initial `just hygiene` composite had **seven failing leaves**. Repairs brought the
failing leaf checks to the zero target, including full Pyrefly **572 → 0 diagnostics** with its seven
existing deliberate suppressions and default/no-default Clippy zero findings. The failed composite
retains that outcome; this is not a substituted aggregate pass. Final `just governance` passed every
required check at `build/assessment/20261027T000821.554421Z-governance-2370456-2736a4`, with zero
failures. G03 owns the full command/mode/leaf accounting and remaining aggregate obligations.

**Tested:** the final integration `just typecheck` leaf reports **0 diagnostics**, with seven
existing deliberate suppressions, in `/tmp/pse-56efg-final-integration-typecheck.log`; the final Ruff
selection reports **0 findings** in `/tmp/pse-56efg-final-integration-ruff.log`. These current-source
leaf results do not replace the earlier failed hygiene composite or qualify the pending journeys.

**Pending:** the coordinator's explicit evidence-executable rebuild and final supplied-only composed
acceptance after the pure-context correction; the final held canonical opaque-cursor/protected-reader
cancellation and EOF journey; and the final public project CLI journey against the rebuilt current
extension. Their completion is required before this
companion claims its selected host/lifetime acceptance. No selected control above establishes the
505-case scientific campaign, solver fleet, IDAES parity, other platforms, distribution, performance
benefit or measured latency/speedup. Unsupported cross-compilation, custom Maturin/PGO and
unmanaged-provider cases keep their documented conservative routes.


## Final publication and acceptance Outcome — 2026-10-27

This historical Outcome supersedes the earlier pending execution checkpoints; its broad completion claims are corrected by the completion audit above. Earlier receipts keep their
original source, mode, date and pass/fail boundaries; the maintainer-directed construction contract
supersedes their source-change and mutable-alias-change refusal criteria.

**Implemented:** `artifact_publication` issues typed immutable generations through the existing
preparation owner. Explicit Cargo/Maturin construction copies a candidate, observes the exact
candidate's compiled contract once and atomically publishes its descriptor. Failure before commit
preserves the prior selection. If replacement may have committed before interruption, both possible
issued images remain retained. Strict producer deployment publishes/selects its own exact associated
image instead of borrowing an ordinary installation. Explicit live import checks remain requested
observations; ordinary sync consumes its constructor's successful import without repeating it.

**Implemented:** native attachment consumers use that issued image once. Their image path performs no source,
compiler-environment, Cargo stdout/dep-info, binary digest or repeated image-stat validation.
The ordinary native input scanner, observer branch, xtask commands/module and their obsolete tests were
deleted; automatic CLI serving-source scans remain CA07. Serving-code edits require explicit drain/reload before edited code serves operations,
including lazy imports. Live request/wire/cursor/physical/scientific/Arrow and resource-drain checks
retain their distinct meanings. New publication never silently replaces a loaded generation.

**Tested, baseline zero:**

- The six owner/strict-adapter modules (`test_artifact_publication`, `test_python_extension`,
  `test_evidence_currency`, `test_evidence_store`, `test_producer_cargo`, `test_producer_deployment`)
  passed **87/87** through `.venv/bin/python -m unittest`. Root's later ten-module repair selection
  passed **118/118**; the final two constructor-fixture modules passed **29/29**. These populations
  overlap. Logs: `/tmp/pse-56efg-publication-strict-owner-controls-repaired.log`,
  `/tmp/pse-56efg-publication-final-repair-tests.log`,
  `/tmp/pse-56efg-publication-final-owner-fixture-repairs.log`.
- Reader/readiness/preparation controls passed **46/46**, affected caller/composer controls
  **68/68**, and native/Python routing controls **27/27** through the named unittest selections
  in `/tmp/pse-56efg-publication-final-reader-unit.log`,
  `/tmp/pse-56efg-publication-callers-unit-repaired.log` and
  `/tmp/pse-56efg-publication-native-routing-unit.log`. The serving-contract selection passed
  **32/32**, with seven source-only acceptance controls at
  `/tmp/pse-controlled-serving-tests.log` and `/tmp/pse-controlled-serving-contract-rerun.log`.
- Actual `scripts.python_extension prepare dev dev` and `prepare native dev` passed through their
  original environment owner, both with correctness validation. Dev generation
  `c045fc514b114487be05644200c377a1` and native generation
  `32dfdf57f8444ce1bece7e2226485962` were published. The latter's compiled image is
  `af05e8761b9956fc914e11f0868e85590c64af8c54d5d6daf9028f6ce019d39f`, with
  `force-validate`, `native-solvers`, `solver-highs`. Actual dev endpoint reuse/drain passed **1/1**.
  Logs: `/tmp/pse-56efg-published-dev-construction-owned.log`,
  `/tmp/pse-56efg-published-native-construction.log`,
  `/tmp/pse-56efg-published-dev-actual-endpoint-repaired.log`.
- `scripts.evidence_store build` published the pure fixed executable. Actual executable, MCP and
  foreground CLI supplied-only comparison passed with no extension import and no canonical store:
  `build/qualification/56efg-published-evidence-supplied-alias/receipt.json` and
  `/tmp/pse-56efg-published-evidence-supplied-alias.log`. One independently supplied failure resolved;
  no scientific execution or ingestion claim follows.
- The actual native context/endpoint/adverse/CLI plus disabled-health and capture selection passed
  **10/10** under `scripts/pse-env --native -- .venv/bin/python -m unittest` conditions in
  `/tmp/pse-56efg-published-native-read-controls.log`. It includes mutable-alias replacement while
  the retained reader continues and a fresh worker attaches the issued private image, then original
  endpoint EOF/drain. Queued SIGSTOP/cancel does not claim Rust or solver entry.
- Actual canonical project/knowledge/protected-result cases passed **3/3** at
  `build/qualification/56efg-published-canonical/receipt.json`. Its live handoff supported the two
  `ActualCanonicalReadAdverseTests`, **2/2**, and three selected installed Python/project CLI/
  portability journeys, **3/3** with the default sixteen xdist lanes and a fresh owned basetemp.
  Logs: `/tmp/pse-56efg-publication-canonical-adverse.log`,
  `/tmp/pse-56efg-publication-projects.log`. The original fixture lifecycle reported all contexts
  and kernel scopes drained and its owned state retired; receipts remain retained.

**Interface-checked:** independent implementation review found two material gaps—strict deployment
selecting ordinary publication and interrupted rename deleting a possibly selected image. Both
were repaired and independently reassessed with no remaining material finding in that scope.
This source assessment is separate from actual execution. `just check-package xtask --features
full-codegen` passed after scanner retirement. Generation, static/manual accounting and final
publication belong to [56g's closeout](56g-qualification-and-publication.md#final-selected-qualification-outcome--2026-10-27).

### A mistake made and corrected

The initial implementation inferred validity again from mutable Cargo records; actual Clippy
rewrote a control spool and falsely refused a valid installed image. The maintainer's correction
removed that inference rather than patching more filenames. An old qualification helper then
selected the new immutable image from its receipt where it had previously selected the mutable
alias; its replacement interfered with one dev attachment. The helper now selects/authenticates
only the owned alias, restores original inode/bytes, and the fresh sequential dev endpoint and
supplied-only controls pass. Initial fixture/arity/type/style failures and exact repairs remain
in their logs. A root attempt using internal `--resolved` without its required environment failed
before build/effects; the ordinary constructor entry passed.

### Deliberate deviations and limits

Construction and controlled editing establish artifact validity once. Duplicate validation is not
retained as assurance. The constructor owns failure contingencies; consumers retain only distinct
requested compatibility and live operation ownership. No new daemon, ledger, host CPU/RAM cap or
source-proof campaign was added. No speed/memory/capacity improvement is measured. Wider 505-case,
solver fleet/parity, feature powerset, release/wheel, remote-CI, desktop-host, other-platform and
100,000-point qualification remain unclaimed. Formal ADR statuses remain proposed.

## Audit-gap closure Outcome — 2026-10-27

**Implemented/Tested:** F01–F06 and CA04–CA08 are complete for the selected local Linux target.
Actual telemetry sampler/bounds and comparison classifications/sides/correspondence/conditions/map
keys now flow from Rust owners through Schemars and generated msgspec consumers. Host conversion
uses those declared bounds before timeout quantization; SDK adapters preserve all six sampler
meanings. Closed comparison vocabularies and section keys retain their schema meaning without
closing original open status strings or changing document version 1. Explicit metadata-contract
failure remains effective under optimized Python; no assertion or duplicated scalar vocabulary
supplies production admission.

Explicit stages enforce their requested mode/profile/validation before import or publication;
generic installed capture retains its distinct meaning. Automatic CLI projections use the loaded
wire contract without serving-source scans. Scoped STDIO transport and `capabilities_drain` keep
native admission closed and original work owned through settlement before SDK disconnect.
Concurrent manager-close observers share one original drain, retaining cancellation and sibling
failures before successor admission. Replaced source-scan and per-observer drain interpretations
are deleted; no reader image/source/build-log validation is restored.

The final affected Python selection passed **142/142** against zero; Rust evidence-comparison
controls passed **10/10** and the actual supplied-only executable/MCP/CLI journey passed with no
product-extension import or canonical-store creation. Both named actual CA08 journeys passed
**1/1** each: retained G1 cursor/pages survive actual G2 publication, known retired cursors refuse,
fresh attachment selects G2, endpoint successor/reconnect works, and foreground EOF waits beyond
the SDK disconnect grace while the authenticated worker is stopped. Exact session-family settlement
and original enclosing allocation/kernel-scope drain are separately proven. An inherited parent
charge is retained rather than mistaken for a worker-owned allocation; premature loss refuses.

[56g's Outcome](56g-qualification-and-publication.md#audit-gap-closure-outcome--2026-10-27)
owns actual command/receipt conditions, repaired source and oracle failures, final static checks,
independent assessment and wider exclusions. No required F implementation or acceptance remains;
optional measurement, desktop-host and unsupported-platform work is not newly selected. DA-040/043
close in the coordinator; no speed, memory or capacity gain is measured.
