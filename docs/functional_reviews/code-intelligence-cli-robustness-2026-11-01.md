# Code-intelligence and CLI robustness

Reviewed 2026-11-01 against clean source `91f255e4d36dc794ba5adff3c4b676d1c7210822`.
**Interface-checked** source assessment, with the specifically **Tested** and **Measured**
probes below. All recommendations are **Proposed**.

This is one integrated functional review of existing engineering workflows, including their
design-principle alignment. It does not add a feature backlog, render a separate formal design
acceptance verdict, repair production code or reopen completed Plans59–62. The
[current audit](../design_review/audit/README.md), published at the inspected revision, supplies
challengeable context. Blueprint §21.7 owns current contracts; plans retain their original
qualification and conditional scope.

The standard is Core 3.4, efficiency heuristics 1.0, process-simulator profile 1.5,
test-assessment 1.2 and the pse-arrow binding. Scientific profile requirements constrain evidence
truthfulness where relevant; this engineering review establishes no scientific acceptance gate.

## Conclusion

The highest-value correction is demand-driven source detail: a two-entry outline currently
returns megabytes of unrelated literal/documentation facts. The graph incident has a different
cause: file-relative request paths selected a nonexistent directory under `/tmp`. That behavior
is intentional; its generic error projection made it hard to diagnose.

Correcting that request reached native graph evaluation, but still produced an explicitly
incomplete answer. It did not establish a successful complete graph journey. These are three
different outcomes: mistaken scope, inadequate recovery information, and limited native
examination. A coherent correction must preserve those distinctions.

Prioritize O01–O03 and O04's consumer examples. Carry O05–O08 into the same investigation/change
footprint where they expose shared causes; do not turn their conditional research into mandatory
infrastructure. No parser replacement, persistent index, new RPC service or universal result
wrapper is justified by these incidents.

## Incident reconstruction and actual probes

The frozen log window is **2026-11-01 22:05:01–23:05:01 UTC**, ending at the request for this
review. It includes coordinator session `01a1969a-9172-7b41-afe6-002deced89dc` and its seven
audit reviewers. Other repositories' concurrent sessions were excluded. Source requests,
responses and retries were correlated, rather than treating every shell error or truncated
combined tool call as a CLI defect.

Reproduction used the current checkout's prepared CLI and a newly owned native snapshot:
ra_ap_hir/ide 0.0.358, default features, `x86_64-unknown-linux-gnu`, macros/build scripts disabled,
test context disabled. Preparation identity:
`01a19966-abbb-75d3-b7c7-b42b09ac7a6d`. No source changes were included.

| Incident and original evidence | Disposition and current evidence |
|---|---|
| Broad source ranges supplied as graph seeds; coordinator around 22:35 | Expected unavailable response: no unique declaration/callee token. Narrow text witnesses advanced the request. A line range is not necessarily an entity selector. |
| Narrower seeds followed by generic ENOENT; coordinator 22:35:56, request record 9692 and response record 9695 | **Tested:** `pse search --request /tmp/principle-audit-related-request.json --format json` again exited 1. Effective traversal was `/tmp/crates`. A traced single-seed variant attempted missing `/tmp/crates/pse-quantity/src`. Request-relative scope, not graph-inventory corruption, explains this failure. |
| Corrected absolute checkout scope | **Tested:** invocation `01a19968-1f48-77fa-a73f-e3c50f98073b` reached native evaluation and retained both seed identities. Exit 1, failures 0, unknown 1, graph incomplete. Limitations included inactive test modules, imprecise original mappings, block-local nomination and edge-examination coverage. No successful complete answer is claimed. |
| Three outline calls with budgets 50/50/30; durable reviewer session `01a1993d-4f6b-70e3-9526-0eda610f452e`, 22:32:07, response record 61 | Original tool capture reported 13,514,409 omitted bytes and approximately 3.64 million tokens over files totalling 388,572 source bytes. This was an output-delivery failure for the agent, although the CLI calls themselves succeeded. |
| Current isolated outline reproduction | **Measured:** `pse inspect outline crates/pse-operations/src/canonical_execution.rs --budget 2 --format json` emitted 4,280,601 stdout bytes. Budget 50 emitted 4,296,097 bytes. Both carried identical 1,594 facts and 30,915 mappings. Compact mapping JSON accounted for 3,702,544 bytes; facts for 4,276,857 bytes. These are local byte measurements, not latency or memory benchmarks. |
| `--lines 553-606` and reviewer's `1-135` | Caller syntax mistakes, faithfully refused. **Tested:** `31-31` exited 2 with the FIRST:LAST rule; `31:31` succeeded. Help/recovery quality remains improvable. |
| Wrong source path and wrong `contract` position | Caller mistakes, not evidence of unavailable declarations. The nonexistent result-selection path again exited 125 with plain ENOENT; correcting the path/position allowed inspection. A coordinate error's generic UTF-8/range wording did not establish an actual Unicode bug. |
| Declaration refinement before preparation; association without current interpreter | Expected explicit-preparation boundaries. Preparation and a later association query succeeded. No silent preparation is recommended. |
| Symbol query for `contract` returned `CauseContractRow` | Provider-ranked discovery returned candidates and marked non-exhaustiveness. This did not prove exact-name matching or absence of the desired function. |
| Assumed `payload.records` or text in `payload.source` | Consumer assumptions. Search exposes `results`; direct source inspection exposes text in `context` and source identity in `source`. Association views have their own typed shape. Empty guessed fields were not empty search results. |

Timestamps, session UUIDs and record positions identify the original log evidence. Full original
call IDs can be recovered there without exporting entire sessions.

Additional **Tested** counterevidence:

- `pse search --explain --request /tmp/principle-audit-related-request.json` succeeded without
  providers and exposed resolved scope.
- A request containing misspelled `depht` exited 2 and reported the unknown key, allowed keys
  and request-file provenance. This JSON path does not silently accept that typo.
- NDJSON for the failing graph request contained start, diagnostic and terminal records.
  The terminal correctly retained one failure and incomplete coverage; `stop=complete` did not
  mean `complete=true`. Delivery finality and examination completeness are different meanings.
- Invalid-input and missing-source JSON envelopes were emitted on stderr; successful direct
  reads and in-query failure envelopes used stdout. An initial review helper that parsed only
  stdout failed. The corrected helper inspected both streams; this was not a JSON decoder defect.

Temporary requests and captures supported probes; this report preserves their consequential
conditions and results. It does not require those temporary files to become permanent fixtures.
Future regression tests should construct their own sources and references.

## Recommendations

### <a id="O01"></a>O01 — Select source detail before constructing it

**Highest priority. AP-07, DP-08, DP-10, DP-16, DP-20; H5, H7, H10, H16, H26.**

`source_read::rust_parsed` pages declaration entries, then unconditionally runs
`facts::collect` over the entire syntax root. `source_read/facts.rs` copies written and
interpreted literal/doc text and builds per-character coordinate objects. Addressed source
inspection also collects whole-file facts before filtering to its range
(`source_command::show_addressed`). Thus narrowing output records does not narrow the expensive
detail responsibility.

Make ordinary outlines return declaration/navigation information without automatically expanding
every decoded literal and mapping. Determine a declaration/range and requested detail first,
then collect the relevant facts. Distinguish record allowance, detail demand and encoded response
extent; an ordinary agent read needs a useful bounded response, not merely a bounded entry count.
Keep original source identity, edition, parse issues and guarded expansion. Define root-doc
handling explicitly, and preserve malformed written text plus interpretation errors.

The simplest correction belongs in the existing source-reading owner. Keep syntax0.0.360 and
`ast::IsString::escaped_char_ranges` for requested escape-aware mappings. `SyntaxText` slicing
and chunk traversal can avoid unnecessary copies, but avoiding unused mapping construction
matters before optimizing its serialization. An IDE `file_structure` alternative is useful only
when a semantic session already exists; source-only outlines must remain independent.

Dropping fields after full construction reduces transfer but leaves allocations and decoding.
Compressing every mapping, retaining another result service or replacing the parser adds burden
before removing unnecessary work. If a byte/detail cap is introduced, it must preserve
indivisible records and explicit omissions; never silently truncate source witnesses.

Acceptance: huge unrelated strings/docs must not expand a one-entry outline or selected
declaration; explicitly requested escaped/Unicode mappings remain correct; continuation retains
guards and detail semantics. Verify avoided construction as well as output bytes. No speedup
percentage is supported.

Library leads: `ra_ap_syntax.md` addendum §§1–3; `ra_ap_ide.md` §6.3/addendum §5.4.

### <a id="O02"></a>O02 — Carry resolved selector meaning into recovery

**High priority. AP-04, AP-05, DP-01, DP-03, DP-18, DP-21.**

`investigation/config.rs` resolves request-file paths relative to the request file, including
nested traversal selectors. Seed guards bind their own source root; they do not change that
configuration rule. `rust/graph_adapter.rs` lifts seeds before expanding traversal/output paths.
The selection visitor then propagates a missing explicit root as generic I/O.

Preserve file-relative semantics and Figment's input provenance. Validate explicit scope roots
before costly seed lifting where possible, and report the selected operation, resolved path and
origin adjacent to the error. Suggest `search --explain` or a corrected explicit path without
silently changing the request base. Prefer improving this existing resolution/diagnostic owner
over a second path-normalization layer.

Do not ignore missing roots or classify every ENOENT as user error: a disappearing file,
unavailable prepared source, permission failure and malformed explicit selection have different
causes and recovery. A present filesystem file outside the native world also needs a separate
disposition. Acceptance includes requests outside the checkout, relative saved-query paths,
absolute paths, root changes, disappearing sources and unchanged guard validation.

### <a id="O03"></a>O03 — Preserve structured failure and actionable correction across projections

**High priority. AP-02, AP-04, DP-02, DP-19, DP-21, DP-22.**

`investigation::Error::Io` supplies context phase but defaults to failed disposition.
`cli/investigation.rs::execute` emits generic `investigation.provider` and preserves special
query detail only for selected wrapped parser errors. The reproduced ENOENT carried no path,
origin or next action. Top-level `failures=[]` coexisted with a failed search whose diagnostics
and summary contained the failure. These distinct channels are intentional but easy to misuse.

Extend the existing error/envelope contracts with owner-derived operation/path/origin and
recovery facts where available, using typed causes rather than parsing display prose.
Keep usage, stale context, unavailable capability, provider I/O, cancellation, delivery failure
and unobserved settlement distinct. Use existing validation-correction disclosure controls:
safe local corrections must not reconstruct redacted selectors.

Keep stdout/stderr and query-event conventions if they remain useful; document and exercise
their consumption together. Do not require agents to infer success from an empty top-level
failure array, a short payload or terminal `stop=complete`. Avoid a new parallel error hierarchy.
Clap typed parsers/possible-value metadata and miette/thiserror sources can supply correction
structure; they do not automatically classify operational failures.

Acceptance covers equivalent failures in human/JSON/NDJSON, preserved causes, usable next actions,
safe redaction, parse/configuration errors before providers and interrupted delivery.

### <a id="O04"></a>O04 — Make the supported response-reading workflow executable

**High priority for consumer reliability. AP-02, AP-06, DP-08, DP-21, DP-24.**

The observed shape mistakes do not justify flattening every response into `records` or making
`source` always a path/text field. Search results, source inspections, associations and graph
support have different meanings. Source and guarded-result input converge on common dispatch,
but only guarded inputs carry their original freshness premises.

Publish concise schema-backed examples that show how to discover payload shape, join source IDs,
read text, pass opaque addresses unchanged, inspect associations and consume terminal coverage.
Exercise these examples through the actual CLI rather than only snapshots of Rust values.
Use the same parser/schema authorities; avoid hand-maintaining another response catalogue.

Compare source/result forms for the same selected range and view, validating each against its
advertised schema and testing staleness. Include payload field selection, association-context
clearing, graph rows with empty text, both output streams and malformed reference/address pairs.
Existing association and source-addressing tests provide a base, but their selected key
assertions do not establish every projected shape.

### <a id="O05"></a>O05 — Explain preparation and contextual scope without automatic effects

**AP-02, AP-05, DP-05, DP-18, DP-19, DP-24.**

Unprepared semantic refinement should refuse; a stale/missing document interpreter should remain
unavailable. Improve readiness diagnostics with the specific missing capability and existing
explicit preparation command. Distinguish tool availability from active snapshot configuration,
macro/test readiness, source freshness and interpreter freshness.

Prefer existing service/tool owners and source association machinery. Do not add a background
watcher, silently rebuild on reads or reuse another context merely because its project path
matches. Corrected graph scope still returned inactive-module/mapping/nomination/allowance
limitations; a single broad incomplete message should retain the concrete reasons and explain
which additional preparation could actually change them. Some limitations cannot be repaired
by preparation and should not advertise it as a cure.

Acceptance: cold passive source reads; unavailable semantic refinement; explicit preparation;
drained session; stale source; macros/test context disabled; available interpreter with absent
document binding. Undeclared ownership stays undeclared.

### <a id="O06"></a>O06 — Align native search work and answer scope without false negatives

**AP-04, AP-07, DP-07, DP-09, DP-10, DP-15.**

Use native `FindAllRefsConfig.search_scope` and `SearchScope` where they express the requested
occurrence domain, retaining grouped definitions and reference categories. Source roots,
configured crates, module instances and macro mappings are distinct; a path or first module
cannot become universal entity identity.

Symbol discovery requires a less immediate choice. Current investigation requests an effectively
uncapped native population before file/kind filtering. Simply pushing the user's result limit
into the backend can miss later eligible candidates. Exact ide0.0.358 `symbol_search` also
materializes `world_symbols` before taking navigation results: an output cap is not a search-work
bound. Its internal callback search is not an already-public streaming API.

First make exact/prefix/fuzzy discovery semantics and work limits intelligible. Compare retaining
the current complete candidate collection with a narrow library extension for selected-scope,
accepted-result stopping if ordinary workloads justify its upkeep. Reuse native indexes and
matching; do not build a competing semantic matcher or persistent graph. Count/exhaustive and
first-N/exists have different stopping obligations.

For graphs, keep explicit filesystem selection distinct from active snapshot membership and
original mapping availability. Retain partial evidence, support nodes and per-seed attribution.
Audit whether each emitted limitation/stop reason matches actual examined work; do not infer
that a displayed edge allowance caused the corrected query's entire incompleteness.

Acceptance includes many excluded candidates preceding eligible ones, duplicate names, aliases,
dependency scope, Unicode boundaries, plural definitions, inactive files and missing mappings.
Library leads: HIR identity/provenance sections, IDE §§3.2–3.3, rust-analyzer §§15.2–15.4.

### <a id="O07"></a>O07 — Close cancellation assurance at the actual query owner

**AP-06, AP-07, DP-19, DP-20, DP-23. Conditional verification priority.**

Preserve the existing vendored request-snapshot cancellation branch: descendants inherit a
request-local Salsa group without cancelling sibling work or replacing the shared world.
The inspected `CpuTask::with_cancel_hook` does not itself watch parent-token cancellation.
Trace every deadline, disconnect and token path to explicit cancellation and settlement;
token signalling alone may not interrupt an in-flight Salsa query.

This is a concrete assurance question, not a newly reproduced cancellation bug.
The existing cold-symbol cancellation test proves coarse settlement/sibling survival; barrier
tests address nested groups. Neither alone qualifies every CLI disconnect or precise indexing
checkpoint. Compare completing those paths/tests with another cancellation mechanism; prefer
the existing owner when it can express the obligation.

Acceptance includes cancellation before start, during cold indexing/references, disconnect and
deadline, sibling survival, a subsequent fresh query and actual macro-worker drain where macro
preparation is selected. None of those fault scenarios was executed by this review.

### <a id="O08"></a>O08 — Test schema/validation/projection boundaries as composed contracts

**AP-04, AP-05, AP-06, DP-03, DP-14, DP-15, DP-23, DP-24.**

Use the existing proto-authored contract/generator route and Buffa/protovalidate capabilities
where native meanings cross the host boundary. Keep request validation, source resolution,
scientific/semantic validity and presentation as different responsibilities. Generated
validation cannot prevent fact amplification after an already-valid request.

Buffa binary unknown-field preservation does not establish JSON unknown-key rejection.
ProtoJSON presence/naming/integer semantics are not interchangeable with arbitrary Serde.
The misspelled JSON query probe did reject its key; there is no demonstrated silent-acceptance
bug in that path. Verify the exact pinned generated/native paths before generalizing upstream
behavior. Incoming validation also does not prove outgoing response validation.

Prefer targeted conformance/property cases across the existing lowerings over dynamic-message
roundtrips or new reflection on normal paths. Assess descriptor-pool facilities only where
runtime schema identity is actually required. Preserve the direct generated-field projection,
integer/path fidelity and current protocol fencing; changing wire meanings requires the existing
decision/generation route.

Acceptance: omitted versus explicit defaults, oneofs, unknown keys/enums, invalid nested selectors,
large values, malformed messages, outgoing invalid state and generated-schema/public-projection
agreement. Binary budgets do not establish JSON, intermediate-work or CLI-output bounds.
Relevant proto references are Buffa §§7, 10; validation §§6–7; generator JSON-schema/publication
sections; local-RPC lifecycle sections. No migration to another protobuf/RPC family is selected.

## Integration, alternatives and remaining evidence

O01 fixes unnecessary work at its producer; O03 preserves explanations at their producer.
O02 supplies meaningful path facts to O03, while O04 makes the public consumption contract usable.
O05 and O06 prevent an apparent convenience correction from changing contexts or completeness.
O07 retains the existing execution owner. O08 protects the wire/validation boundary as these
corrections move through it. Combining them does not require one universal response or entity type.

Architecturally, AP-01/AP-03/DP-17 favor these existing responsibility owners over another
cross-cutting service. AP-04/AP-05 and DP-01–DP-08 require distinct scope, identity, observation
and interpretation. AP-07 exposes work amplification independently of current output correctness.
DP-11/DP-12 constrain deterministic continuation and bounded traversal; DP-13–DP-16 support
native library composition with qualified limits. DP-18–DP-24 govern effects, settlement,
structured diagnostics, evidence and evolution. These are qualitative assessments, not a
numerical score or formal gate certificate.

No current evidence justifies changing request-file relative-path semantics, merging the native
0.0.358 and structural syntax0.0.360 universes, weakening guards, treating unknown as absent,
silently preparing macros, or importing a new RPC stack. No license exclusion was applied.
Library/current-source research found useful mechanisms, not a reason to adopt every documented
feature. Cargo metadata, rustdoc and compiler references principally constrain contextual identity;
their acquisition/ABI/compiler-facts mechanisms were not exercised here.

Existing source-addressing, composed-investigation, validation and cancellation tests were
inspected, not run. No production implementation, test-only account refresh, broad qualification,
scientific journey, latency campaign or host-authentication benchmark was performed.
Shared setup does not automatically invalidate an oracle: new regression expectations should
independently distinguish wrong scopes, missing details and false completeness.

The source-query/outline and graph probes above establish their named local behavior only.
Native symbol pushdown, every format/projection pair, non-UTF-8 paths, concurrency faults,
macro cancellation, rustdoc/compiler integrations and response-validation coverage retain
investigation obligations. No current scientific or completed-plan qualification is revoked
or expanded by this report.

## Research and publication

The coordinator reconstructed the logs and ran probes; independent read-only mapping and library
research supplied code/test owners and exact-release alternatives. The coordinator reconciled
them, including rejecting the initial hypothesis that the ENOENT proved a graph backend defect.
This is integrated research, not an unrequested formal recommendation-audit stage.

Local references under `docs/libraries/rust_analyzers` were reviewed by capability/relevance,
including their addenda; proto and CLI references were selected by the boundaries above.
Context7 supplied current upstream orientation; consequential version-sensitive recommendations
also used pinned repository/installed source. Useful upstream entry points:
[rust-analyzer architecture](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md),
[Buffa guide](https://github.com/anthropics/buffa/blob/main/docs/guide.md),
[clap typed parser example](https://github.com/clap-rs/clap/blob/main/examples/typed-derive/custom.md).
These moving documents are research leads, not proof of exact pinned behavior.

Recommendations need separately authorized plan creation before implementation. Future planning
should carry their concrete mechanisms, consumer work, evidence gaps and possible protocol/rule
impacts forward, rather than scheduling a generic cleanup or treating all observed refusals as bugs.

The review-owned native session was stopped with `pse code session stop`; its original identity
matched and the stop response established service disappearance and complete settlement.

Publication checks: `just turn-end`, `just docs` and `git diff --check` completed with zero
failures against the zero target. These establish documentation publication, not implementation
of the proposed corrections or broad product qualification.
