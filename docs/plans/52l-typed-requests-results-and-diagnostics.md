---
title: "Plan 52l: Typed requests, results and diagnostics"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
---

# Plan 52l: Typed requests, results and diagnostics

## Target and foundation assessment

**Proposed:** make valid questions constructible from the advertised interface, return useful
typed answers, and explain refusals through the owner that can actually prescribe recovery.
This develops the usability review's [O01](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O01),
[O02](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O02),
[O04](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O04) and
[O07](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#O07).
The [coordinator](52-agent-effectiveness-and-engineering-workflows.md#usability-opportunity-dispositions)
owns their state and the operator's RC01 acceptance. Completed 52j remains the delivered foundation.

**Interface-checked:** `operations.workspace_concepts_query` cannot express all of its
`workspace_domains.query` owner's supported facets and required reference combinations.
`ReadResult.payload` is generic JSON; `bounded()` can replace values with null or omit keys after
acquisition. Host validation and native scientific errors already retain useful locations,
classifications and spans. These strengths support local evolution rather than a new schema,
error framework or universal response format. Rust/native boundary ownership and version-first
admission remain at blueprint §21.1/§21.5; diagnostic authority is §23.2.

The focused assessment applies Core3.4, Heuristics1.0, Test Assessment1.2 and relevant Process
Simulator1.5 constraints. AP-04/AP-05 require meaningful alternatives to govern execution;
AP-07/H5–H8 require useful selection before expensive acquisition where semantics permit it.
Scientific completeness, units and native contextual admission cannot be inferred from JSON shape.
No broad audit, product qualification or measured usability benefit follows from this assessment.

## Ownership and physical design

Host tooling owners define query alternatives and response families; MCP and useful native callers
consume those definitions. Pydantic remains the host contract mechanism. Rust serde/registry types,
Schemars and existing generators own scientific documents, enums, descriptions and directional
schemas. Preserve authoring versus hydrated representations, local reference scope and compiled
fingerprints. Do not mirror scientific fields in Pydantic or hand-edit generated outputs.

Mandatory answer facts include the actual outcome, source/cut, relevant population/completeness
and continuation. Optional explanations have explicit selection and availability. A compact
complete answer and a partial population differ even when equally small. 53e E05 owns protected
assessment acquisition/custody; [52f F01](52f-evidence-analysis-and-trace-correlation.md#F01) owns
comparison semantics. Extend other existing readers at their actual owner where needed; this
companion owns the shared presentation contract and its caller migration, not those authorities.

The [review's mechanism table](../functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md#library-and-mechanism-decisions)
supplies the alternatives and evidence limits. Use Pydantic 2.14.1
[`Field(description=..., examples=...)` and schema metadata](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/json_schema.md),
`Annotated` constraints and [tagged unions](https://github.com/pydantic/pydantic/blob/v2.14.1/docs/concepts/unions.md)
for genuine host alternatives. Generate the serialization schema of the response actually returned;
arbitrary `model_dump(include/exclude=...)` does not establish that schema. Schemars1.2.2
[derive metadata](https://docs.rs/schemars/1.2.2/schemars/derive.JsonSchema.html), Serde tags and
existing deserialize/serialize settings supply the corresponding Rust mechanisms.

## Work packages

<a id="CON01"></a>
### CON01 — Constructible domain requests and executable examples

Move meaningful concept/query alternatives to the existing tooling owner and expose all supported
facets, including origin/question and explicit evidence references where required. Distinguish
missing, null and empty selections according to that owner. Derive choices from its vocabulary;
do not copy an adapter-only enum. Compare a tagged request with separate callables only where
responsibilities genuinely differ. Preserve exact structured discovery without similarity fallback.

Deliver schemas, descriptions and independently authored first-request examples together with MCP,
native callers and generated reference changes. The migration covers every exposed query alternative,
not just the simplest identity lookup. Unknown concepts and malformed references must reach the
appropriate structural or contextual refusal. A future concept should extend its owner and binding,
without editing unrelated dispatch switches. Remove replaced argument parsing and stale examples
when callers move; do not keep a compatibility request language.

Targeted acceptance: representative document/status/topic, research-question and supplied-evidence
queries are constructible from advertised contracts and match independent expected source facts.
Exercise tagged alternatives, unknown identities, absent/null/empty and inappropriate facet combinations.

<a id="CON02"></a>
### CON02 — Typed selected answers and truthful output contracts

Inventory recurring host answer families and their actual readers, including context/concepts,
inventory, product-run and assessment views. Define typed family responses for decision-relevant
outcome/source/selection/completeness distinctions. Reuse an existing owner type where adequate;
do not force unrelated readers through one schema. O06's selection reaches the original reader
before hydration; preserve global denominators and ambiguity work that the answer still requires.

Return bounded selected sections, authentic continuation or explicit unavailable/refused detail.
Never advertise the complete native output schema for a structurally altered payload. Retain a final
size safeguard as a separately typed incomplete/refusal outcome rather than silently deleting required
fields. Avoid serializing all optional sections just to discard them. Exact native/full export remains
available through its existing protected owner. No generic JSON query engine or global response limiter
is selected: the latter truncates after work and can discard schemas, structured content and media.

Migrate actual callers, resource/tool projections and generated reference in the same behavioral slice.
Remove the old walker's use for each replaced family once the useful bounded alternative is proven;
retain only genuinely remaining callers until their migration. Targeted tests independently expect
complete, partial-population, optional-detail-omitted and unavailable answers, validate actual serialized
outputs, and exercise nested/oversized values without changing mandatory meaning. F01/53e supply real
protected-reader acceptance; a small response alone proves no reduction in examined work.

<a id="CON03"></a>
### CON03 — Safe actionable diagnostics across the existing bridge

Project stable owner codes, stage, structured locations, safe interpretation/remedy and authentic
inspection references. Distinguish select, correct input, restart, explicitly prepare and inspect
unsettled work. Reuse thiserror/miette and the PyO3 `DiagnosticReport` bridge, preserving useful
related causes/context/recovery through an explicit privacy-filtered view. Do not return raw input,
credentials or arbitrary exception text, or manufacture a handle before admission. Native settlement
and effect state decide recovery; no generic error class grants retry permission.

Evaluate [serde_path_to_error0.1.20](https://docs.rs/serde_path_to_error/0.1.20/serde_path_to_error/)
for structural Rust decode paths using path segments and the original inner serde_json error,
classification and line/column. Preserve version-first admission and `Deserializer::end()` after
successful deserialization, including duplicate-header/trailing-input refusals. Check custom/untagged
decoder limitations before selection. Native public diagnostic changes take FND00's actual decision
route before implementation; ordinary host projection and exact dependency addition need no placeholder ADR.

Targeted acceptance: hidden tool, unknown selection, nested invalid input, unsupported version,
trailing data, stale interface, missing preparation and interrupted/unknown-effect cases. Independently
assert safe codes/locations/remedies and absence of sensitive values, retaining authentic causes.
Remove superseded rendered-message classification/parsing as its callers migrate.

<a id="CON04"></a>
### CON04 — Shape-first scientific result inspection

Conduct the bounded O07 design inquiry at `native_worker.read_result`, the native result-selection
owner and Python inspection stream. Determine how an exact protected result can expose relation/schema
without exporting its data, then support meaningful row/field selection. Compare schema inquiry plus
the existing IPC reader with owner-native field projection. Choose the smallest realization that
serves ordinary wide-result inspection; honest refusal of every useful wide-row question is insufficient.
Record the chosen contract, required decision route and actual consumers before changing native APIs.

Preserve exact Arrow IPC export. Any selected readable physical view retains field identity, units,
basis, validity and missing/nonfinite distinctions. Do not turn per-row JSON calls or lossy numeric
normalization into the scientific path. Use existing Arrow/DataFusion only where the result owner
permits projection and retains protected identity/lifetime; schema discovery grants no new data authority.

Targeted acceptance uses an actual wide typed result, a compact schema inquiry and independently
checked field/row answer, plus unknown field, stale/released result and output-bound cases. Record
explicitly whether readable physical views are adopted or unnecessary. Broader query-service or solver
scope is excluded. Delete replaced inspection-only export callers if the new route replaces them.

## Dependencies, verification and publication

**Proposed:** CON01/CON03 can deliver useful independent slices. CON02 consumes actual owner selectors
and CON01's relevant vocabulary rather than waiting for a whole companion. CON04 resolves its native
contract before dependent result views. [52m](52m-progressive-guidance-and-operation-availability.md)
consumes these contracts for guidance and lightweight dependencies; [52n](52n-readiness-progress-and-edit-lifecycles.md)
consumes diagnostic/readiness/handle meanings. Updated native schemas and callers must land together.

Use adopted `jsonschema.Draft202012Validator.check_schema()`/`iter_errors()` for independent structural
checks, preserving error paths, schema paths and branch context. Resource scope/escaped JSON Pointers,
enums and input/output direction need focused cases. An explicit `referencing.Registry` is conditional
on a real resolution need and requires an exact direct dependency if used. Prefer adopted proptest for
relevant Rust properties; schema acceptance never implies scientific eligibility or overrides version
admission. Compile affected code and run targeted tests during implementation, with force-validation
where Arrow is consumed. Generate changed declarations as part of their change.

G03 and H02 own actual-agent/assembled acceptance; H03 owns scope-end static/manual checks. Publish
enduring host contracts in the capability guide/reference and actual native changes at their governed
owners. **Interface-checked** foundation statements above are static; all package behavior is
**Proposed**. This document records no new product test, host probe or measured benefit.
