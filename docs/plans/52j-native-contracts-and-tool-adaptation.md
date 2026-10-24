---
title: "Plan 52j: Native contracts and tool adaptation"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, environment, libraries, diagnostics]
---

# Plan 52j: Native contracts and tool adaptation

## Prerequisite delivery — 2026-10-24

The [52i Outcome](52i-canonical-capabilities-and-progressive-reveal.md#verification-and-outcome--2026-10-24)
records ADP02's delivered source-only prerequisite: actual typed host callables, strict canonical
validation, bounded payloads and preserved native incompleteness/refusals. The exact Pydantic
dependency and accepted host/scientific rule distinction are implemented. Native schema/decoder
integration, richer independent outcomes/handles/cancellation and all-family adaptation remain
under the package owners below. The numeric coordinator alone owns their progress state.

## Target and responsibility

**Proposed:** make existing operations directly usable through the single FastMCP surface with
trustworthy input schemas, native admission, useful structured outcomes and authentic handles.
This companion develops the unified review's [O04](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md#O04),
[O05](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md#O05) and
[O07](../functional_reviews/unified-fastmcp-capability-interface-2026-10-24.md#O07).
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns package progress,
source-qualified opportunity dispositions, rule decisions and Q02/Q04 outcomes. This document
specifies work; it is not another status ledger or permission to change scientific semantics.

[52i](52i-canonical-capabilities-and-progressive-reveal.md) owns exposure metadata, catalogue,
reveal and guidance composition. [52k](52k-unified-entry-point-and-derived-interfaces.md) owns
attachment, lazy resources, actual host behavior and conditional derivative interfaces.
Operation integration belongs here, including changes to an existing tool when preserving its
parser shape would force a second interpretation of defaults, effects or outcomes. Completed
52a–52e capabilities remain delivered; their new interface adaptations are ADP work.

## Foundation assessment and retained authority

The source review examines FastMCP4.1.3 and Pydantic2.14.1, with bounded library probes rather
than actual Codex/native workflow qualification. Its current-source conclusions support two
integration routes, not a universal Pydantic scientific model. The contract remains
[blueprint §4.2](../authoritative_design/sections/schema-and-relations.md#section-4-2) and
[§21.5](../authoritative_design/sections/workflows-and-results.md#section-21-5).

| Existing foundation | Consequence for the target |
|---|---|
| `pse-codegen/src/codegen/jsonschema.rs` emits source-authoring grammar. | Preserve contextual omissions and authored aliases. Hydrated serde structs include spans/integrity that an author does not supply. |
| `pse-runtime/src/boundary_documents.rs` and `xtask/src/codegen/schemas.rs` publish Rust-owned documents. | Bind each native operation to its actual schema family and decoder; a similar type name is insufficient. |
| `experiments.main` dispatches outer argparse; `test_archives.archive` and `active_diagnostics.execute` accept namespaces. | Separate typed operations from parsing where this removes reconstruction, while retaining their real execution/resource owners. |
| `validation.execute`, native assessment and result readers have separate responsibilities. | Expose selection, invocation and results without scraping printed handles or interpreting terminal return as complete settlement. |
| `codex_session` selects existing language/database connections. | Reuse identity, pin and position conventions through lazy internal adapters rather than publicly attaching more servers. |

AP-01/AP-03 support a narrow exposure integration beside each coherent operation owner, with
ordinary modules rather than another service boundary. AP-02/AP-04/AP-05 require advertised
shapes to reach the actual native decoder and preserve distinctions that affect behavior.
AP-06 favors independently testable host requests and source-only reads. AP-07 favors prepared
schema/adapters and coarse native calls with bounded detail, rather than per-row scientific
JSON crossings or reloading every schema for each invocation. These are scoped design judgments,
not an audit refresh, performance measurement or implementation acceptance.

The operator selected one endpoint, native schema authority and interface adaptation. New-review
RC02 was explicitly accepted on 2026-10-24: Pydantic may own host-tool contracts; generated/native
scientific contracts remain Rust-owned. Resolve the broad Pydantic prohibition in the applicable
Python rule during ADP02 before dependent implementation. RC05 acceptance preserves existing
effectful admission; endpoint launch belongs to 52k. Actual changes to a governed public
Python/native boundary follow the existing decision route before implementation. Ordinary tool
refactors and dependency declarations need no placeholder ADR. Rule decisions live at the
coordinator, not in this contextual summary.

## Contract realization

Host functions use typed request/result definitions and FastMCP `FunctionTool`. Prefer existing
dataclasses/typed mappings plus `Annotated` constraints and reusable `TypeAdapter`s; use
`BaseModel` where its modeling features help. Declare the exact direct Pydantic dependency in
the owning dependency group when first-party code uses it. Do not rely on a transitive pin.
Strict structured MCP values and CLI text parsing are different boundaries. Define unknown-field
handling, default validation and discriminated alternatives explicitly. Preserve omitted,
explicit null and empty values using field-set tracking, selective serialization or `MISSING`
where appropriate; do not normalize all three into one state. See the versioned
[TypeAdapter](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/type_adapter.md)
and [strict validation](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/strict_mode.md)
contracts. Unchecked construction/copy updates, partial validation and duck-typed serialization
are not shortcuts for a final admitted boundary.

Rust-owned requests use a narrow custom `Tool(parameters=..., output_schema=...)`, publishing
generated schemas without handwritten scientific field mirrors. The binding must also name the
native receiving operation. Its `run(arguments)` invokes the native decoder/admission before
effects; custom Tool registration does not supply that validation automatically. The exact
[FastMCP Tool contract](https://github.com/PrefectHQ/fastmcp/blob/v4.1.3/fastmcp_slim/fastmcp/tools/base.py)
is the implementation reference. Use an object-root host envelope referencing a native document
when necessary, without recreating its fields. Preserve dialect, `$defs`, local reference
resolution, tagged unions, numeric constraints and validation/serialization shape distinctions.
Any reference rebasing is generated. Object outputs fit `ToolResult.structured_content`.

JSON-Schema-to-Pydantic conversion is conditional consumer adaptation, never sole native
admission: the inspected converter can drop unsupported regex constraints with a warning.
Dynamic models/core-schema hooks require a specific fidelity benefit. Existing PyO3/JSON is
the default native route. Protobuf/gRPC is unselected for schema alignment alone; reopen only
for a meaningful process/service replacement benefit, with all consumers and retirement of
the replaced route addressed. Do not add a parallel production operation pathway.

Outcome contracts retain the operation's distinctions: malformed invocation, unavailable
prerequisite, stale context, domain refusal, incomplete observation, interruption, transport or
schema failure and unexpected error. Use native codes/causes and structured field locations,
with deliberate redaction. Do not invent one result enum that erases native differences.
Protocol success is not scientific success. PS-10/PS-11 require original assurance and
model/revision/preparation context to survive exposure.

Return short summaries and authentic invocation/result/resource handles; use selected native
readers for large logs, Arrow payloads, archives and traces. Bound nested inventory/checkpoint/
error content as well as top-level pages. Continuation retains source identity and incomplete
coverage. Observer cancellation detaches; action cancellation uses its native owner and retains
drain/settlement distinctions. A request timeout cannot undo a blocking native operation, and
reconnection/reveal never implicitly retries effects. No Tasks/Docket scheduler is selected.

## Work packages

<a id="ADP01"></a>
### ADP01 — Bind generated native schemas to actual admission

Resolve Q02 on a small set of real operations before general native migration: one authored
document, one runtime boundary request/result with tagged enums/local references, and a useful
read or preparation operation that can be reached by the first combined slice. Select actual
consumers through the current native catalogue; retain the registry authoring/hydrated
distinction rather than choosing convenient schemars output universally.

Implement the custom Tool binding from generated schema identity to native entry function and
explicit operation context. Reuse prepared schemas per loaded interface revision. Where the
current Python facade already exposes the operation, use its supported path; otherwise compare
a narrow native binding with a faithful process adapter before changing the public boundary.
Schema validation supplements contextual admission, never moves policy into Python.

Prerequisite: CAP01's stable component identity/registration slice and an executable native owner,
not completion of all 52i/52k work. Supply the native binding contract to ADP04 and schema
fingerprint/loaded-revision facts to 52k. Native schema producer changes include regeneration and
their actual consumers in the same change; never hand-edit generated schemas or contracts.

Targeted acceptance: valid authored input admits without parser spans/integrity; malformed,
unknown-field, unsupported enum/variant and stale contextual input refuse before effects.
Independently expected cases exercise recursive/local references and numeric restrictions.
Verify the tool actually calls the native receiver, including a case that is schema-shaped but
contextually inadmissible. Client output validation alone does not establish producer admission.
Record Q02's supported schema/decoder families and any concrete constrained migration at the
coordinator; an unresolved family cannot be silently advertised as supported.

<a id="ADP02"></a>
### ADP02 — Typed host requests and faithful bounded outcomes

Establish the first useful host operation and its result/detail reader through FunctionTool,
using existing context/navigation or invocation inspection owners. Inputs select an explicit
checkout/source/subject and bounded demand; outputs retain native identities, scope, omissions,
continuation and applicable next operations. Adopt only shared meanings justified by G01;
operation-specific outcomes need not inherit a universal envelope.

Implement reusable constrained host types and validation/serialization adapters where consumed.
Clarify the accepted host-only Pydantic rule scope first. Use suitable native handle wrappers or
existing discriminated documents rather than conflating filesystem paths, Git worktrees,
database namespaces and invocation identities. Keep credentials outside public requests.
Extend owner-returned results when callers currently have to recover a handle from printed text.

Prerequisite: CAP01's metadata slice and selected callable owner. This can proceed alongside
ADP01 and make the first discovery→guidance→real action→result journey useful. G01 consumes its
interaction meanings; 52f consumes native comparison/trace evidence, not a new telemetry store.

Targeted acceptance: strict wrong-type/unknown-field refusal; independent absent/null/empty
expectations; bounded nested results with valid continuation; explicit partial empty output;
redacted field errors and a preserved native cause. Demonstrate observer detachment separately
from action cancellation and owned drain. Keep actual producer outcome separate from a
client-side schema exception. Remove replaced output scraping as real consumers migrate.

<a id="ADP03"></a>
### ADP03 — Extract parser-bound operation kernels

Resolve Q04 first on archive/replay and active diagnostics, then apply the decision to validation
and experiment dispatch. For each owner compare an explicit typed callable request/function/result
with a structured process adapter. Choose extraction when an adapter would reconstruct parser
defaults, exit handling or lifecycle policy. Keep a real process boundary where it owns execution;
the objective is coherent operations, not eliminating subprocesses.

For `test_archives`, preserve selected package/test/features, producer/source identity, explicit
output paths, owned report resources and replay's new invocation identity. For `experiments`,
separate dispatch from leaf operations so outer help cannot consume archive help. For
`active_diagnostics`, preserve exact subject identity and helper ownership; cancelling the
diagnostic helper cannot cancel its observed subject. For validation, retain the actual selected
closure, force-validation, detached observation and terminal/drain/result distinctions. Adapt to
current Plan53 owner contracts without absorbing Plan53 test-architecture work.

Prerequisite: ADP02's needed host/result slice, plus the corresponding executable owner. Register
real operations and migrate their callers together. Remove namespace reconstruction, printing/
scraping and replaced agent-facing dispatch once consumers move; retain independently useful
native/low-level commands. Do not retain a compatibility parser solely as historical evidence.

Targeted tests exercise successful requests and prior refusal/lifetime cases at each extracted
owner, including wrong source, existing output, interrupted replay and disappeared/replaced
diagnostic subject. Actual MCP consumers must receive the same authentic handle/cause, not
merely pass a new DTO test. Preserve a usable leaf-help route for any retained CLI. Q04's outcome
names the chosen boundary per affected owner; one representative success does not close others.

<a id="ADP04"></a>
### ADP04 — Native, language and database integration adapters

Use ADP01's binding for native project/revision, scientific result, evidence and diagnostics
operations. Expose existing operation units rather than scalar/row RPC loops. Protected result
selection, source cuts, temporal context, admission and explicit preparation stay native. Retain
supplied-only/offline paths without acquiring a canonical database merely to describe/read them.
Where a genuinely process-owned command remains, consume its structured output and report
startup/transport failure separately from its native domain outcome.

Compose existing Rust/Python navigation/edit and selected database MCP services through internal
providers/proxies, with 52k owning acquisition, identity and teardown. The adapter owns positions,
source versions, partial edit results and native errors. A language rename cannot claim to
migrate generated declarations/PyO3 strings; protected edits retain preview/apply reconciliation.
Database query tooling does not bypass canonical project-operation admission or become agent
memory. Proxy namespaces map to the same CAP component identity, not another public server.

Prerequisite: relevant ADP01/02 slices and 52k's lazy resource contract for connected operations.
Pure/source/offline adapters do not wait for all specialist integrations. Targeted acceptance
uses actual native input/result decoding, wrong service identity, unavailable dependency, stale
edit positions, partial edits, detached observation and teardown of the connection actually
owned. Reuse native operation tests; do not run a whole scientific qualification per wrapper.

<a id="ADP05"></a>
### ADP05 — Reconcile every tool family and retire replaced exposure

Reconcile migrations in the live repository, not only Plans47/50/51/52 or the examples above,
against CAP05's exhaustive source-backed classification and the
[review inventory](../functional_reviews/agent-tool-discovery-and-access-2026-10-24.md#what-an-agent-can-already-use).
CAP05 owns classification/exposure completeness; ADP05 owns actual adaptation, caller migration
and retirement. Consume that same definition for callable native/host operations, internal
specialist adapters, instruction/resources, unavailable optional capabilities, planned capabilities
and internal helpers with no independent agent operation. No copied argument/default catalogue
or permanent second migration ledger. Missing classification returns to CAP05 rather than being
decided in a parallel list; arbitrary shell execution is not a substitute for missing leaves.

| Family to reconcile | Native owners and adaptation/preservation obligation |
|---|---|
| Orientation, process skills, roles and planning | AGENTS/docs/current-work/process owners become discoverable instructions; retrieving a planning skill cannot execute it. |
| Conceptual context, source/generator reach and Git rationale | workspace context/tasks/sources, affected and declared production routes retain source cuts and unresolved reach. |
| Navigation and protected structural edits | semantic navigation/edits, mcpls/Pyrefly/rust-analyzer and structural tools retain real support, previews and protected-path refusal. |
| Knowledge, libraries and attributed recall | Knowledge/research/session readers retain exact conceptual selection, version evidence, bounded provenance and historical attribution; no similarity discovery. |
| Environment, session and optional-tool preparation | Environment explanation stays effect-free; installation, startup and readiness remain explicit owner operations. |
| Compilation, caches and storage inspection | Build/check/cache/storage owners retain selected closure/mode; inventory does not authorize deletion or a benchmark. |
| Functional/scientific verification techniques | Selection, assessment, Python/native/parity, coverage/mutation/fuzz/reference owners retain force-validation, true population and unexecuted campaign limits. |
| Workload observation and authentic results | Activity/watch, run inspection and result readers retain live/recorded basis, continuation and terminal/drain/settlement. |
| Scientific and active diagnostics | Frozen packs/review/context plus active diagnostics/trace readers retain offline behavior, exact subject identity and helper cancellation. |
| Canonical store and specialist database tooling | Surreal/toolkit/module/diagnostic connections retain parser/version capability, bytes, service identity and explicit effects. |
| Projects, revisions, scientific data, studies and results | Existing native/Python owners retain revision/temporal context, guarded admission, protected selection and actual assurance. |
| Development/research evidence | Evidence import/query/offline/lifecycle owners retain source membership, original codecs and rebuildable projection authority. |
| Experiments, archive/replay, isolation and recovery | Experiment/archive/bisect/compare/reference/portable owners retain distinct resource identities, owned execution and cleanup obligations. |
| Schemas and generated consumers | Registry/codegen/native-stub owners expose producer-first generate/check operations; generated paths remain protected. |
| Documentation, decisions and lifecycle | Document metadata/scope/publication/ADR/register owners retain status authority; preflight neither accepts a decision nor authorizes retirement. |
| Collaboration, Git and maintenance | Worktree/Git/upgrade/family/version/setup/turn-end/ready/hygiene owners retain ownership, exact pins, explicit effects and execution timing. |

This reconciliation continues as coherent families migrate through ADP01–04; it is not a late
deferral of their necessary consumers. New capabilities already authorized elsewhere join their
appropriate family. Proposed G04/G05/F work stays labeled planned until its owner delivers it;
this package does not implement every repository proposal merely to complete the inventory.

Acceptance compares CAP05's actual callable targets with migrated handlers, native/process
receivers and consumers. Return newly found leaves to CAP05 for a specific disposition and owner;
an aggregate recipe name cannot stand in for its subcommands.
Exercise an unfamiliar useful operation outside the four original plans, then inspect actual
callers to remove duplicate agent command declarations, parsers and output scrapers that the
canonical route replaced. Preserve independent low-level executors. Supply exhaustive coverage
and remaining limitations to G02/H01/H02, without claiming representative journeys qualify all
families.

## Verification and handoff

**Proposed:** package checks use targeted owner tests and compilation of touched closures, with
force-validation for Arrow-consuming correctness paths. Independent malformed/contextual and
absent/null/empty cases are necessary because a schema-derived round trip can reproduce its own
mistake. 52h owns assembled host/native journeys and scope-end static/integrated qualification;
no full campaign or hygiene run follows each adapter. No speed/productivity claim is Measured
without a corresponding workload measurement.

Completion requires actual migrated consumers, exhaustive family dispositions and deletion of
replaced exposure machinery. Record implementation, focused tests and remaining assembled
qualification separately at the coordinator. Q02/Q04 are settled by named schemas/receivers and
per-owner boundary choices; unresolved material cases keep dependent exposure unqualified.
Publish enduring contract/guidance at its existing owner through the established decision route
where required; do not move native authority into this companion or retain a second public
interface for historical comparison.
