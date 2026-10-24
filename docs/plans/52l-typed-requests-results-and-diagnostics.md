---
title: "Plan 52l: Typed requests, results and diagnostics"
status: complete
date: 2026-10-24
adrs: [ADR-0224]
review_sources: [docs/functional_reviews/fastmcp-usability-and-tooling-evolution-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
---

# Plan 52l: Typed requests, results and diagnostics

## Execution checkpoint — 2026-10-24

CON01–CON04 are complete at their selected host/native and bounded actual-consumer acceptance
boundaries; the [Outcome](#outcome--2026-10-24) records verification and limits. The isolated branch integrated the completed main-branch Plan53
changes, passed its targeted native and Python consumer checks, and returned to main at
`69f645f72`. The worktree is removed; its historical build artifacts and logs remain under
`build/plan52l-worktree-evidence-69f645f72`, preserving failed attempts and their original paths.
The unrelated `history.txt` edits remain untouched. Plan53 retains its own scope and status.

Generated contracts/reference and the compiled API surface comparison passed. The fresh main-branch Codex
journey passed all nineteen stages, including the tagged concept request, complete instruction
resource, exact Rust action and authentic result/drain inspection. This is bounded interface
acceptance, not whole-Plan52 qualification. The final fresh STDIO action and protected reader also
exercised the demand-shaped pure Rust route after the efficiency corrections.

The maintainer then authorized reviewing and correcting inefficient recent execution paths.
Heavy qualification was interrupted rather than continued while these changes were made.
The bounded follow-up removes preparation unrelated to actual selected demand, preserves exact
artifact/source/selection admission, and consolidates aggregate generation without losing
independent failure diagnostics. The canonical inspection fixture retains its real isolated
store and protected-reader assertions; it does not require a native execution receiver or
authored scientific input capsule. Production cancellation policy and scientific validation
remain intact. Targeted verification and the selected scope-end checks passed after correcting
findings and two accidental finite-allocation OOM interruptions. No speedup or complete
efficiency-alignment claim is made. 52m/52n consume the delivered contracts; broader F/G/H
qualification and proposed ADR acceptance retain their own owners.

The maintainer authorized CON01–CON04 and selected the compact readable physical-value view
alongside schema discovery and exact IPC. ADR-0224 records the native inspection/diagnostic
extension before code changes; it remains proposed. Implementation started on main from
`93cf65dc8` after recorded registry conversion. Fresh attachments establish their actual
loaded/store identity; old endpoint failures around reset are not evidence of current result
behavior. The root owns integration and acceptance of the implemented host, native, generated
and scientific inspection changes.

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
owners. The original **Interface-checked** foundation above remains a historical static assessment;
the checkpoint and outcome distinguish implementation and exercised scope from that starting point.

## Verification

**Tested — zero-failure baseline:** the efficiency follow-up's composed host controls passed:
`scripts/pse-env --resource-class compile -- .venv/bin/python -m unittest
scripts.tests.test_native_tests.ManagedRustSelectionTests scripts.tests.test_test_run
scripts.tests.test_scientific_inputs scripts.tests.test_test_requirements
scripts.tests.test_codegen_builds scripts.tests.test_validation` — 144 tests, zero failures,
28.280 seconds on 2026-10-24. These cover retained preparation fences, selected-demand routing,
scientific capsule scope, signal/terminal behavior, generator recipe composition and validation
closeout; subprocess controls do not replace the separately exercised real Rust/store paths.

**Tested — zero-failure baseline:** `scripts/pse-env --resource-class compile --
.venv/bin/python -m scripts.arrow_validation nextest run --no-fail-fast -p xtask --bin xtask
--locked -E 'test(codegen::tests::)'` — 17 tests passed with explicit force-validation,
67 unselected tests, on 2026-10-24. The original compile failure from mismatched schema error
types was corrected with explicit conversion to the aggregate check owner's error type.
No measured end-to-end speedup follows from either control run.

**Tested — zero-failure baseline, 2026-10-24:**

| Command or acceptance route | Scope and result |
|---|---|
| `.venv/bin/python -m unittest scripts.tests.test_capabilities scripts.tests.test_capability_read_contracts scripts.tests.test_capability_presentation scripts.tests.test_capability_diagnostics scripts.tests.test_capability_reference scripts.tests.test_workspace_domains scripts.tests.test_capability_rust_tests` through the checkout environment | 86 host/schema/diagnostic/adapter tests passed; no product solve claim. |
| `just unit-package pse-runtime 'test(canonical_readable_wide_result_retains_physical_context_protection_and_exact_ipc)' --features canonical-tests` | One real isolated canonical test passed with force-validation and the original successful 24 GiB compile owner, 1,066 unselected; registered nineteen-column result, three rows, original metadata/context, exact IPC, manifest/field/range refusals and retained-reader release/drain. |
| `just py-unit python/pse/tests/test_standalone_document_wire.py python/pse/tests/test_native_boundary_contracts.py python/pse/tests/test_studies.py -k 'standalone_document_wire or native_boundary_contracts or study_version_precedes_nested_current_decode'` | 39 actual force-validated native Python consumers passed on the reconciled branch; later corrections were preparation, test organization and lint repairs. This retains the original execution cut rather than asserting a new final-binary run. |
| `scripts.tests.capability_codex_check --output build/capabilities-52k/typed-contracts-52l-main-accepted --timeout 1200` through the checkout interpreter | Nineteen actual fresh-Codex stages passed on main: tagged query, full guidance, one authentic Rust action and native result/drain inspection. This is the retained main acceptance before the efficiency follow-up, not all H02 journeys. |
| Fresh FastMCP STDIO describe/select/action/read | After the efficiency changes, one actual `pse-ids::float::tests::f32_nan_variants_collapse_too` selection passed; invocation `92d42c89f67441608ae2ddb70caf2ed5`, assurance `pass`, owned drain `complete`. Authentic output and final typed reader remain under `build/capabilities-52l/efficiency-fresh-mcp`. No worker, canonical execution observer or scientific capsule was prepared. |
| Five focused receipt/profile controls, then 25 extension/pure-route controls | Changed artifact bytes and invalid thread/profile claims refuse; profile checks avoid duplicate byte reads; stale extension eligibility precedes import. All passed. These overlap the earlier composed controls; counts are not summed as unique coverage. |
| `just governance-tests` through the compile-default owner | All 25 current governance tests passed; no skips. |

**Tested — static/publication scope:** `just codegen` published all ten generated schema targets
and the derived reference; the aggregate comparison, installed Python contracts/stubs and family
checks passed. Default and no-default workspace/all-target `just clippy` passed with
force-validation and `-D warnings`. The same full workspace/all-target Clippy command also passed
with `pse-py/native-solvers,pse-tests-conformance/native-acceptance,pse-benches/native-process,pse-runtime/canonical-tests`
and the linked native environment. `just lint-py`, `just typecheck` and `just docs-rust` passed;
Pyrefly reported zero diagnostics with three existing suppressed diagnostics. The original hygiene
bundle retains its interruptions/findings; repaired leaves establish their stated scopes, not a
retroactive uninterrupted aggregate pass. The original failed logs and native receipts remain.
Final `just governance` passed every selected leaf, and `just docs` published the documentation
book with scoped search. Proposed ADR-0224's evidence label changed to Tested; its status did not change.

**Measured observation, not a speedup comparison:** Cargo reported 11m06s preparing the changed/cold
canonical feature graph and shared runtime test binary; Nextest then ran its one selected test in
13.915s. Six generator launches became one aggregate launch, plus the still-independent installed-stub
launch. The observed aggregate generator comparison took 8.7s under its warm conditions. These are
different workloads and cannot establish an end-to-end speedup or a benchmark result. The later
governance comparison after Rust source corrections took 54.7s including Cargo preparation;
these are individual observations, not a representative average.

## Outcome — 2026-10-24

**Implemented / Tested:** CON01–CON04 deliver eleven tagged concept-query alternatives, fourteen
typed selected host answer families, safe owner-native diagnostics and native schema/readable result
inspection alongside exact IPC. Existing owner/registry declarations generate the consumed schemas;
scientific fields were not mirrored in Pydantic. Real native version/header/body/trailing admission,
original result identity/metadata, protected-reader lifetime and explicit output refusals remain.
Enduring contracts are published in the capability guide, native catalogue and blueprint
§19.2/§21.1/§21.5/§23.2 under proposed ADR-0224 and its existing revision row.

**Implemented / Tested:** the authorized efficiency follow-up removes unused worker/observer and
scientific-capsule preparation, stale-extension import, duplicate prepared-artifact hashing,
overlapping aggregate generation and unrelated historical-resource sweeps at validation closeout.
The [bounded test-account follow-up](../design_review/test-audit/execution-efficiency.md#bounded-execution-follow-up--2026-10-24)
preserves finding lineage, contrary evidence and remaining limits. It does not refresh the entire
test audit or close all DTA findings/Plan53 obligations. Current resources still receive authentic
terminal cleanup; no historical database, fixture or evidence was deleted.

**Mistakes made and corrected:** the coordinator passed explicit `--resource-class compile`,
overriding the maintainer's uncapped default with a 24 GiB allocation. Two Clippy attempts hit that
cgroup limit. Journal evidence identified the cause; corrected recipe-default placement verified
sixteen jobs and uncapped applicable ancestors. Agent/environment guidance now makes that distinction
explicit. A one-off MCP checker also assumed an incorrect result wrapper; the native action was
not retried. The authentic issued invocation was inspected through the advertised typed reader,
and corrected caller assertions passed. Static findings were repaired rather than accepted as a
baseline: Python typing/conditions, feature-gated helpers, declaration ordering, Copy/assignment
syntax, reduction conditions, test-only fixture allowances, FFI safety premises and sparse entry
iteration. Production lint and scientific assertions retain their obligations.

**Deliberate choices and limits:** adopt the bounded readable physical-value view, not IPC-only
inspection; retain exact IPC and original full-batch validation. The acceptance fixture uses actual
registered columns and opaque unit/quantity identities, without claiming resolved scientific
eligibility. Keep its private admitted-ingestion unit route because no existing integration target
provides an equivalent helper; its shared unit-binary compile cost remains. Keep standalone narrow
generation and bootstrap publication routes. No solver convergence, throughput/memory campaign,
full feature powerset, wheel/platform, entire Plan52 or remaining Plan53 qualification is claimed.
ADR-0224 remains proposed. 52m/52n and that ADR consume this plan, so it remains retained under
ADR-0096 rather than being retired while those dependencies are live. The worktree is merged and
removed; unrelated `history.txt` edits are preserved. No push is implied.
