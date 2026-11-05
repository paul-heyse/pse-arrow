---
id: ADR-0224
title: Inspect protected results through native typed views
status: proposed
date: 2026-10-24
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-07, DP-21, PS-10, PS-12]
blueprint: [§19.2, §21.1, §21.5, §23.2]
review: "not-required: scoped observational inspection binding within existing Rust-authoritative boundaries; maintainer-authorized Plans52l/55/65; Plan65 F04 source-backed assessment preserves full native admission"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A concrete inspection needs another scientific query authority or decoding loses native interpretation or protection.
verification: Selected roots and full/projected/full decoder reuse preserve complete file/schema/layout protection, all original scalar keys and retained relation predicates; omitted-field local validation is explicitly unavailable; native inspection selection/context/cursor/cancellation and unchanged full-admission controls retain independent expectations.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/52l-typed-requests-results-and-diagnostics.md#CON03, docs/plans/52l-typed-requests-results-and-diagnostics.md#CON04]
---

# ADR-0224: Inspect protected results through native typed views

## Context

The canonical Arrow reader protects exact terminal run/attempt/result identity, while an agent
currently exports IPC merely to learn a relation's shape. Decode failures also lose structural
locations on their way through the existing Python diagnostic bridge (blueprint §19.2/§23.2).

## Scope

Add native-owned bounded result schema/readable inspection documents and structural decode-location
projection through the existing Rust/Python document catalogue. This extends accepted boundaries;
scientific model, schema, source, outcome, result protection and native admission authority do not move.
The maintainer explicitly selected compact readable physical values as part of Plan52l execution.
On 2026-10-25 the maintainer authorized Plan55's complete execution, including a version-two
extension for nested readable values and bounded exact context. The original selected evidence
below retains its earlier scope; the subsequent Plan55 confirmation records version-two
implementation and its selected qualification separately.

## Drivers

An agent should construct a meaningful selected question, discover fields before exporting data,
and inspect a small attributable answer from a wide original relation. Original type/metadata,
units/basis/validity and missing/nonfinite distinctions must survive, and schema-only reads must not
consume data batches. Decode locations describe input structure rather than invented authored identities.

## Options

Keep IPC-only access; reconstruct scientific records in Pydantic; or extend native protected readers
with typed inspection projections and generated contracts. Select the native extension: it reuses
admission, schema, scientific output selection and lifetime owners without another query service,
per-row RPC loop, scientific Python mirror or second result store. Projection before transport removes
unneeded data transfer; stored IPC decoding/validation can still require the original full batch.

## Outcome

### Consequences

Rust owns inspection requests/results and decode-location definitions. The existing catalogue emits
directional schemas/Python records, and canonical MCP consumes those original documents. Schema/field
and row selection carry original run/attempt/relation, manifest and schema identity. Readable values
are explicit bounded projections; exact IPC remains the lossless programmatic path. Missing, nonfinite,
unavailable and omitted detail never become zero, null substitution or successful scientific assurance.

### Plan55 nested inspection amendment — 2026-10-25

The result inspection request and response roots use the existing versioned wire mechanism at
version two (`FlatV2`); other document owners retain their versions. Active Rust, PyO3, generated
Python and MCP consumers migrate together, and version-one requests are rejected before current
body decoding rather than coerced through a compatibility API.

One typed Arrow field path selects a root and struct-field/list-element steps. Native traversal
checks parent validity and per-row list offsets, preserving null containers, empty lists, absent
elements and null elements distinctly. Exact scalar widths, IEEE class/bits and physical context
remain native meanings. Shallow descriptors and bounded schema/context detail pages avoid repeating
entire nested datatype trees. Continuations bind issuer, original result selection, manifest,
schema, complete row window and selected paths; each request reopens/protects that exact result.
Schema/context pages consume no data batches. At the Plan55 cutover, selected readable
materialization retained original full-batch decoding and validation; no decode-speed improvement
was claimed.

Keep decision-critical units/basis/identity/quality/usability inline and supplementary exact
context linked with explicit completeness. Indivisible unsupported extent refuses honestly;
exact IPC remains available for bulk/programmatic access. Plan55 C02 must make a representative
real nested value readable with independently checked meaning; a more precise refusal alone is
insufficient. The existing reader/lifetime authority is unchanged.

### Plan65 selected-hydration amendment — 2026-11-04

**Proposed; Interface-checked:** selected readable inspection decodes the requested complete
Arrow roots, existing context roots, all scalar primary-key roots and every bound relation-check
dependency. Preserve whole-file schema/footer/block extent, complete-message layout, original
protection and allocation grants before decoding. Validate every retained root with the existing
immutable native `ValidationContext::column` / `PreparedLocalContract::evaluate` contracts,
evaluate all original relation predicates, and check scalar-key uniqueness across the entire
original batch before slicing the requested row window.

This is a narrower observational guarantee: omitted roots' local values are not validated.
Hashes establish original-byte integrity, not omitted-field semantic validity. Expose the actual
inspection-check coverage in the result; readable completeness does not become full-relation
admission, scientific permission or publication eligibility. Advance only the affected native
inspection request/response roots to version three and migrate their active consumers together;
reject superseded roots before body decoding. Other document owners retain their versions.
The full IPC/scientific path still requires its complete `FieldCheckedBatch` and native checks.
Do not create a synthetic relation, arbitrary-subset `FieldCheckedBatch`, new store or admission
certificate from an inspection observation.

Arrow 59.3 [`FileDecoder::with_projection`](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/reader/struct.FileDecoder.html#method.with_projection)
skips unselected top-level array hydration. Construct its projection per read from the retained
full admitted schema so selected state cannot leak into a later full read. The existing
[admitted-file owner](../../crates/pse-columnar/src/ipc_file.rs) retains complete schema/layout
preflight; no bespoke IPC decoder or second footer authority is needed. Native
[column validation](../../crates/pse-relations/src/validate/prepared.rs) supplies the retained-root
checks. Bind original relation predicates with the existing
[row-check owner](../../crates/pse-relations/src/validate/row_checks.rs), derive their dependencies
with DataFusion 55.1 [`Expr::column_refs`](https://docs.rs/datafusion-expr/55.1.0/datafusion_expr/enum.Expr.html#method.column_refs),
and preserve all predicate evaluation rather than silently dropping checks after projection.
Missing or unsupported dependency binding refuses before access.

The native snapshot owner retains source acquisition, cached original bytes, cancellation,
failure, close and escaped-buffer accounting. The complete selected IPC block is still read;
selected arrays can retain its full backing buffer, and nested selection hydrates the complete
root. Existing decoded-byte observations keep that meaning. No memory, latency or throughput
benefit is claimed without complete-path measurements. A union requiring all fields naturally
uses the complete root population.

### Compensating controls

Protected readers stay alive through actual inspection and close/drain normally. Changed/released
selections refuse. Decoding retains native version/header admission, duplicate-key/trailing-input
refusal and contextual validation. Structured serde paths may be unknown for custom/untagged decoders;
map keys can be supplied data, so host projection redacts unsafe paths and original messages.
Use thiserror/miette and the existing report bridge; no generic retry or fabricated recovery handle.

### Confirmation

Plan52l CON03/CON04 own implementation and targeted native/Python/actual-agent acceptance. This
record remains proposed. **Tested, 2026-10-24:** the reconciled native Python boundary selection
passed 39 tests; the real nineteen-column canonical inspection test passed with force-validation;
the bounded main-branch fresh-Codex journey passed nineteen stages. The
[Plan52l Outcome](../plans/52l-typed-requests-results-and-diagnostics.md#outcome--2026-10-24)
records commands, original cuts, negative/refusal/lifetime controls and the distinct final
preparation checks. These establish the stated selected contracts, not whole-product
qualification or measured benefit; proposed decision status is unchanged.

**Implemented/Tested, 2026-10-25:** Plan55 delivered version-two typed nested paths, shallow
context/child/metadata pages and original-window continuation across Rust/PyO3/public Python/MCP.
The selected force-validated native result/model controls, four canonical protected-result
controls and 50 public-native boundary unit cases passed. An actual public-native/fresh-MCP
retained LP outcome read matched original JSON and independently expected nested `state=fresh`,
then reopened at the same manifest on a new connection. Explicit composite logical-type metadata
omission keeps context/readable completeness false; scientific usability remains unknown. The
original metadata reader retains exact supplementary bytes and indivisible oversize refusal.
The [Plan55f Outcome](../plans/55f-qualification-and-publication.md#verification-and-outcome--2026-10-25)
records commands, repaired failures, original cleanup and exclusions. No whole-product,
complete-context or speed claim follows, and formal decision status remains proposed.

## Pros and cons

The original result/schema and diagnostic owners stay authoritative and become directly useful to
agents. New generated document and bounded projection code needs maintenance, but avoids duplicating
scientific fields or serializing entire results merely to answer a small question.

## More information

[Plan52l](../plans/52l-typed-requests-results-and-diagnostics.md) owns execution and evidence;
[the coordinator](../plans/52-agent-effectiveness-and-engineering-workflows.md#usability-rule-decisions)
records accepted RC01 and the separate conditional mechanisms. The usability review's O04/O07
supply functional motivation. Enduring publication follows the existing design-edit/revision route.
[Plan65f F04](../plans/65f-diagnostics-and-result-delivery.md#selected-hydration-and-inspection-assurance--2026-11-04)
owns the selected-hydration implementation, coverage changes and targeted acceptance. Earlier
Plan52/55 test evidence above does not establish the new projected path.

## Status history

- 2026-10-24: Proposed before native changes under maintainer-authorized Plan52l execution.
- 2026-11-04: Amend the proposed target before Plan65 F04 implementation: selected-root hydration
  with explicit observational check coverage, preserved whole-source/predicate/key protection and
  unchanged full native admission. New mechanism support is Interface-checked; earlier tested
  scopes remain attributed above. Formal status remains proposed.
