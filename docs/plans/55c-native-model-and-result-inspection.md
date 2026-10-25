---
title: "Plan 55c: Native model and result inspection"
status: complete
date: 2026-10-25
adrs: []
review_sources: [docs/functional_reviews/mcp-development-effectiveness-2026-10-25.md]
doc_role: plan
doc_owner: docs/plans/55-mcp-development-effectiveness.md
doc_topics: [current-work, design, numerical-execution, validation]
---

# Plan 55c: Native model and result inspection

## Target and foundation

**Proposed:** let Codex inspect authored scientific changes and meaningful nested results through
native owners without reconstructing physics in Python or interpreting opaque model handles.
Owns O08/O09 and the native contribution to [A01/O10](55a-capability-contracts-clients-and-recovery.md#A01).
The [coordinator](55-mcp-development-effectiveness.md) owns opportunity dispositions.

**Interface-checked:** at `391c949b5`, native `DeclarationInventory`, `ModelingInspection` and
project comparison already describe declarations, routes/procedures, topology, instantiated
members and lineage, including default owners, overrides and presets. The
[Python facade](../../python/pse/_modeling.py) exposes `declarations`, `project_compare`,
`project_source_detail`, `preparation_discovery` and `inspect`. Inspection performs contextual
admission/instantiation rather than dispatching a solve. Its current PyO3 method constructs
preparation settings from limits implicitly; canonical inspection needs that policy explicit.

The [result inspector](../../crates/pse-runtime/src/workflow/result_inspection.rs) currently
assembles selected and all non-value-role context descriptors before opening results, using
full nested Arrow datatype JSON. The 64 KiB bound is independent of the 32-context-field bound.
Scalar projection returns `UnsupportedStorage` for nested arrays. Original result readers
decode and validate complete batches; selected presentation does not establish reduced IPC
decode work. The source review's retained fifteen-field context refusal proves an availability
gap, not excessive value acquisition or a newly reproduced failure here.

Use blueprint §21.1/§21.5 and ADR-0224's native directional boundary/protection contract. Concrete
mechanisms come from the [Serde/Schemars overview](../libraries/Serde_serde_json_and_Schemars.md),
native modeling/result owners and existing generation, not another rmcp/server/Python codec.

## Target contracts and alternatives

For model development, admit finite supplied source documents within one owned specialist child.
Return native generated observations with their source, physical and preparation-policy basis.
Comparison admits both sides within one Runtime. Equal logical names under different physical
closures do not establish scientific equivalence: semantic comparison refuses, while explicitly
source-only comparison reports Source/Unknown impact. Local revision tokens are not reopenable
handles after that child closes. The response cannot imply subsequent numerics are eligible.

Expose complete selected inspection policy along with case and solve settings in a native-owned
request. This aligns inspection with preparation discovery's actual cut. Select declaration/detail
populations natively; avoid a host loop repeatedly building complete snapshots for each changed
logical object. A complete comparison may still need both inventories, and answer paging alone
does not reduce that work. Preserve original native admission diagnostics for inadmissible cases.

For result inspection, select shallow native descriptors, exact child/context detail and typed
Arrow child navigation. Initially support struct children and explicit list elements sufficient
for the representative fixture. Keep existing top-level selectors within the same reader;
do not add a second production inspector or invent JSON Pointer syntax for Arrow arrays.
JSON Pointer remains applicable to actual JSON documents only.

Separate bounded schema/context-detail pages from row-value windows. Bind continuation to the
original issuer/source, run, attempt, relation/schema, manifest, complete requested window,
selected paths and page position. Use immutable descriptor reuse where identical content really
repeats; no persistent descriptor registry/cache is selected. Start from the current readable
page bound. Inspect the actual fixture's encoded context before selecting any justified finite
increase, and respect downstream exact transport bounds.

Inline the context needed to interpret the selected value: unit/dimension, basis/convention,
identity, quality/usability and unresolved physical context. Link remaining exact metadata with
explicit completeness. A reference does not prove that the agent has read that context. Parent
validity governs nested access: null container, empty list, absent index and null element have
different outcomes. Preserve signed/unsigned widths, IEEE class/bits and signed zero, exact
integers and complete-or-unavailable text. Unsupported shape is not fabricated null.

Compare the selected path/page design with a finite-bound increase and compact shared descriptors
using the actual selected population. The simpler choices may fix descriptor-only availability
and should be used where sufficient, but neither alone supplies unsupported nested cells. Exact
Arrow IPC/native export remains the route for bulk/programmatic or unsupported shapes. Selection
precedes readable materialization while retaining required original full-batch validation.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| C01 | complete | 55c | Exact native declaration/inspection/comparison adapted with explicit source/physical/policy cuts |
| C02 | complete | 55c | Native context representation and selected nested values make a representative result readable |
| C03 | complete | 55c | All affected native/public/generated consumers and scientific examples use the new contracts |

<a id="C01"></a>
### C01 — Model-development observations through the native boundary

Extend native binding declarations and owned worker composition for declaration/detail,
inspection and comparison operations. Source inputs and options are finite typed documents,
not arbitrary Python constructors or persistent model sessions. The original Rust receivers own
source admission, expansion/preparation policy, lineage and comparison meaning. Mark contextual
effects accurately; “before solve” does not mean universally passive. A04 consumes an explanation
of those actual requirements rather than pretending to dry-run native admission.

Use the exact native transport route (`native_contracts.EXACT_READS` or its deliberate successor).
Generic `responses.bounded` can replace long strings with null, shorten lists and cut nested
structures; it must not process these scientific documents. Project/bound them natively, then
transport intact, or issue a typed native refusal. Migrate SchemaSet, PyO3, public Python,
canonical adapters and generated references together where their contracts change.

**Acceptance:** change an inherited default and a preset/override, inspect native lineage and
compare independently specified source changes. Different physical closures refuse semantic
comparison and permit explicit source-only comparison with unknown scientific impact. An
inadmissible case returns its native reason without solve. Changed source/policy invalidates
observation applicability. Oversized detail is paged/refused natively, never silently sliced.

<a id="C02"></a>
### C02 — Read a meaningful nested protected value

First identify the retained case's actual relation/schema/manifest and inspect its full descriptor
population in a bounded fixture. Registry `runtime.study_outcomes` v2 is a plausible representative
because it has fifteen fields including nested start/diagnostic/attempt structures, but that
correspondence is an inference, not established identity of the historical refusal. Regenerate a
representative case if the old one cannot be reopened; preserve the earlier evidence unchanged.
Choose independently expected value/context cells before exercising the new projection.

Settle native path/context-page documents, the actual finite descriptor allowance and required
physical context under ADR-0224/blueprint §21.1/§21.5's decision route before changing semantics.
Extend the single native reader and its protected lifetime; do not make a host staging/export
copy the scientific authority. A cancelled/partial terminal remains cancelled/partial; readable
cells do not establish usability or successful solve. Released selection/cancelled acquisition
retains the original protection and close/drain obligations.

**Acceptance:** a selected nested value actually becomes readable under finite bounds with
independently checked physical/context meaning. A better `context_extent` refusal alone is not
completion. Cover parent nulls, empty/absent/null list elements, unsupported storage, holes/source
row coordinates, exact/nonfinite values, changed manifest/window/path continuation, released
selection and cancellation. No decode-speed claim follows from selected materialization.

<a id="C03"></a>
### C03 — Native examples and complete consumer migration

C03 is a continuing integration owner: necessary consumers, exact transport and generated
contracts migrate with each C01/C02 slice, before that slice can be accepted. Only the final
completeness/example reconciliation waits for both packages; no temporarily broken boundary
or second production path is permitted by the package split.

Attach representative Rust schema-owned examples for inspection policy, comparison mode and
nested result requests. Carry them through `boundary_documents.rs`, native JSON Schemas,
`pse-codegen` directional msgspec generation, PyO3/public Python and canonical SchemaSet/resource
projections. A01 owns the host presentation convention; these native declarations own scientific
meaning. No new contract/example store or parallel handwritten Python schema is needed.

Examples with placeholder identities are constructible illustrations. Runtime fixture requests
must use actual issued identities. Validate both schema directions, exact response variants and
decode paths; distinguish selected/admitted/observed from prepared/eligible/executed/readable.
Regenerate through `just codegen`, remove replaced projection/caller/tests after targeted checks
pass, and keep original native export for its independent bulk-access role.

**Acceptance:** current MCP and derived CLI describe the same new directional contracts; actual
fixture values survive their exact route without changed strings/lists/nulls/context. Stale
client/native wire or source cuts refuse before effects. The component/public Python/native
journeys selected in 55f exercise the consumer changes, not just generated-code equality.

## Verification and Outcome — 2026-10-25

**Implemented:** C01–C03 deliver exact native model declaration/inspection/comparison access and
version-two protected result inspection throughout Rust, PyO3, generated Python, public Python,
MCP and CLI. Declaration windows select IDs before cloning bodies. Typed struct/list paths,
shallow child/context pages, scalar-width/IEEE/null distinctions and original-window continuation
remain native. Schema/metadata reads acquire no data batches; selected values retain original
full-batch admission/validation and protection. Version-one inspection requests refuse before
current-body decoding. No compatibility implementation remains.

**Tested:** the force-validated native `pse-runtime` nextest selection (original `scripts/select.py` route) selected pure population passed
13 tests and its selected canonical population passed four tests, both against zero failures.
The latter includes original physical context/exact IPC, nested paths/continuation, nonempty
multi-page reads under the original 256 MiB observer/two compiler leases and connected exact-range
cancellation. A real canonical model declarations→explain→inspect→compare journey passed with
explicit original preparation policy and changed-source observations. Public native boundary
and generated-stub consumers passed all 50 selected Python unit cases. Exact filters, original
invocations and local evidence belong to [55f](55f-qualification-and-publication.md#verification-and-outcome--2026-10-25).

**Tested, actual readable value:** a single existing LP fixture point produced a real retained
`runtime.study_outcomes` row. Public-native JSON and a fresh MCP read were equal; independently
expected source row zero and nested `state = fresh` were observed, then reopened on another
connection at the same manifest without scientific replay. This is usable inspection evidence,
not proof of scientific usability: the original document reports usability unknown, and explicit
metadata deferral keeps context/readable completeness false.

**Corrected design detail:** composite logical-type metadata repeated recursive trees and filled
the existing 64 KiB response bound before values. Only that composite entry is now deferred with
its exact key/byte count, including ancestor omissions. Other scientific/extension metadata stays
inline; the original metadata reader supplies exact supplementary bytes or refuses indivisible
oversize. The bound was not raised. Independent final source review found no material issue;
that static judgment does not replace actual tests. The original tuple/list comparator failure
and invalid harness source/admission attempts remain recorded. No decoding speed, general solve
qualification or complete-context claim is made.
