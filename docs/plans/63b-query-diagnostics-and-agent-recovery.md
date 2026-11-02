---
title: "Plan63b: Query diagnostics and agent recovery"
status: proposed
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/63-code-intelligence-cli-robustness.md
doc_topics: [current-work, design, libraries, diagnostics, validation]
review_sources: [docs/functional_reviews/code-intelligence-cli-robustness-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# Query diagnostics and agent recovery

## Purpose and foundations

**Proposed:** O02–O04 and the common/native-public O08 boundary. The
[coordinator](63-code-intelligence-cli-robustness.md) owns combined decisions and disposition;
this companion owns B01–B04, including shared diagnostic declarations and generated consumers.

`investigation/config.rs` resolves paths relative to their contributing input and uses Figment
for precedence/provenance. It currently returns a Request for execution while its explain
product retains origins separately. Selection/native errors can lose the operand and origin
before `investigation::Error` is projected. `query_error_record` retains special parser details
but generic I/O can become an unhelpful context/provider message.
Existing ErrorRecord, validation disclosure/correction and NextAction are suitable seams.

## Target and library-backed realization

One resolution operation supplies the effective request and available input provenance to
explanation, preflight and execution. Retain structured origin information until a boundary
has produced its safe public projection. A caller need not infer a missing selector's origin
from prose or repeat configuration resolution. Preserve request-file and saved-query bases,
explicit-value precedence, whole-array replacement and tagged-alternative replacement.

Classify invalid explicit selection, changing source, permission failure, unavailable native
membership, provider failure, cancellation, delivery and unknown settlement at the owner that
knows the cause. A missing explicit root before acquisition is a selection refusal; ENOENT
after observation may be a source race rather than invalid input. Do not classify by matching
display text or treating every OS error number as one domain meaning.

The diagnostic carries the known operation, safe resolved operand, origin, typed cause and
available recovery. Optional facts remain unknown when not available. Use the existing
NextAction command representation with real arguments for safe read/explain/preparation actions;
an action is advice, not authorization or automatic execution. Redaction happens under the
original disclosure contract and cannot be undone by downstream correction.

Keep current stdout/stderr conventions and distinct typed payloads. Top-level command failures
and in-query diagnostic/terminal events retain their roles; examples and schemas explain how
to consume both. Do not copy one query failure into competing mutable error authorities.
Terminal delivery does not imply exhaustive examination or physical completion.

| Selected library mechanism | Role, evidence and limits |
|---|---|
| Figment0.10.19 `find_value`/`find_metadata` and value tags | Recover actual contributing providers for keys before typed extraction erases them; inspect exact installed behavior for nested arrays/alternatives |
| Figment typed extraction and Error metadata/path/profile/chains | Preserve parse/configuration causes rather than reparse formatted errors |
| clap4.6.7 `ArgMatches::value_source`, typed value parsers and possible-value metadata | Distinguish supplied values from defaults; share validation/help/correction vocabulary; help/version remain successful control flow |
| thiserror2.0.21 source relationships and miette7.6 Diagnostic/related diagnostics | Carry original typed causes into the existing diagnostic projection without imposing renderer text on machine consumers |
| Buffa0.9.2 and protovalidate-buffa0.7.2 | Author native meaning in proto, generate validation and consume typed fields directly; wire decoding alone proves neither query-domain validity nor outgoing validity |

See [Figment](../libraries/cli/figment.md) §§6–8,
[clap](../libraries/cli/clap.md) §§6–7, 9, 11 and
[diagnostics](../libraries/cli/thiserror_and_miette.md) §§2–5.
Context7 confirms useful provenance/parser concepts; qualify exact pinned APIs before using
newer examples such as richer error-origin interfaces. Existing
`reset_changed_alternatives` and domain validation remain authoritative for their responsibilities.
Do not create a second key/provenance registry.

For native changes, use [Buffa](../libraries/proto/Buffa.md) §§7/10 and
[validation](../libraries/proto/protobuf_validation_rust.md) §§6–7 with exact generated and
vendored validation sources. Native proto Failure and public ErrorRecord can express related
facts without identical Rust/wire layout. Preserve direct generated-field/VTable projection.
Binary unknown fields, JSON unknown keys, field presence and domain constraints have separate
policies; verify each actual lowering instead of assuming library-wide behavior.

## Packages

### B01 — Resolved input and diagnostic declaration slices

Trace inputs through config resolution, selection, native Failure, investigation errors,
ErrorRecord, schema and all output modes. Define the shared operation/origin/correction facts
at existing owners. Carry Figment's known origin into execution with the effective Request;
do not resolve again later. Keep native paths lossless where supported and redact only under
the original policy. Validate cheap explicit selector roots before semantic seed lifting,
without opening every source or preparing a world during explain.

Deliver the first slice with a missing request-selected root consumer, its public schema and
regression test. If wire meanings change, edit authored proto/generator, update its protocol fence,
regenerate with `just codegen` and migrate sender/receiver/validators/projections together.
The pure source CLI remains Serde/Schemars-owned. Schedule the normal architecture/ADR route
only for an actual enduring-contract change, not simply because generated fields change.

Acceptance: a request under a temporary directory exposes its resolved missing root, input
origin and safe explain action; the same absolute-root request preserves its original meaning.
Invalid explicit scope is detected before native work. Explain remains passive.

### B02 — Selection, race and context causes

Migrate explicit path roots, file acquisition and graph traversal/output selection to the
shared facts and owner classification. Keep independent seed guards and selector bases.
Retain a disappearing file's observed premise and classify subsequent failure separately from
an initially nonexistent root. Permission errors retain their cause. C01 supplies the distinct
prepared-world membership/readiness facts, without making this owner interpret HIR.

The resolver/provenance slice is independently usable before native membership work. C01 consumes
that implemented slice, not completion of all B02 scenarios. The shared membership scenario is
qualified after C01 supplies its classification.

Acceptance: defaults, saved queries, request files and explicit CLI overrides select the same
intended request with correct per-key provenance; nested discriminator changes remove stale
alternative fields; explicit false is not mistaken for omission. Include root changes,
permissions, absent files and filesystem files not present in the native snapshot.

### B03 — Failure, validation and recovery projection

Migrate changed producers to typed errors and common projections; remove replaced string-only
conversions for those crossings and their obsolete tests. Preserve phase, disposition, original
causes and correction disclosure through query events and command envelopes. Parser/schema
refusals precede provider effects when the selected schema can establish them.

Extend producer-local generated/native/public controls for absent versus explicit defaults,
oneof combinations, unknown enums/keys, nested violation paths, malformed and large values,
and outgoing invalid states. Do not require a generic dynamic-message transcode on normal paths.
Where output validation is needed, put it at the existing producing boundary, not merely in a
test decoder. Unsupported validator rules must retain an explicit refusal/limit.

Acceptance: equivalent causes yield consistent structured meaning across human, JSON and
NDJSON; safe corrections remain useful and redacted input remains unrecoverable. Failure during
delivery retains incomplete settlement evidence. An empty top-level failure array cannot be
the reader's sole success predicate for an in-query failure.

### B04 — Executable agent consumption and guidance

Make a compact set of native host integration examples/tests exercise discovery → schema →
query/inspect → guarded expansion → result interpretation. Show `results` versus direct-source
`context`, joining source IDs, association shape, graph support, both streams and terminal coverage.
Use actual parser/generated schemas rather than a parallel hand-written response catalogue.
Compare direct-source and guarded-result forms for the same view while preserving freshness
differences; test stale and malformed locator/address pairs and field/detail projection.
Explicitly cover association-context clearing for the selected view, and graph/support rows
whose text is legitimately empty while their source/relationship evidence remains available.

Integrate A's detail selection and C's real readiness/cancellation messages as their slices
become available. Update the existing pse-cli guidance and developer command owner with tested
examples at completion. Unit tests retain focused ownership; these journeys do not replace them.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | implemented; verification pending | Query recovery | Typed effective-key origins retained through execution; passive explain precedes explicit-root preflight and common recovery/schema projection |
| B02 | implemented; verification pending | Query recovery | Missing roots, observed disappearance, source changes, permission errors and typed native-context reasons retain distinct premises |
| B03 | in progress | Query recovery | Common command/query recovery projection, original causes and Tree-sitter details migrated; assembled native validation/disclosure controls remain with the coordinator |
| B04 | in progress | Agent consumption | Schema-backed request/error/source-guard journeys authored in `agent_interaction`; assembled execution and final guidance remain |

## Verification and handoff

**Proposed:** targeted config/model/envelope tests and host investigation/source-addressing/
composed-investigation targets. Expected origins, classifications and fields must be independently
specified, not computed with the same resolver/adapter being tested. Reconstruct temporary
requests in fixtures; do not depend on the review's temporary files or mutable session logs.

**Interface-checked:** existing resolution, diagnostic and proto/public seams support this target.
Latest upstream documentation is orientation; exact release behavior remains the implementation
basis. No new tests or product changes were executed during authoring. B01 can start alongside
A01; B04 finishes only after the selected A/B/C behavior and schemas work.

## Execution checkpoint — 2026-11-01

The maintainer authorized the full series after authoring. The source/public recovery slice now
retains Figment provider/file metadata as typed origins on the effective request; array origins
belong to their whole replacement values, and the selected scope name remains in `request.scope`
instead of replacing its contributing origin. Named scopes participate in tagged-alternative
reset alongside defaults, saved queries, request files and explicit overrides.

Execution preserves that resolved object. Explain returns before filesystem preflight; ordinary
execution checks explicit working answer/traversal roots before Begin or native seed lifting.
Missing roots expose the resolved native operand, contributing input, original filesystem cause
and a safe passive-explain action. Replay retains literal original arguments and query overrides,
removes the Report-only envelope field projection, and withholds advice if an argument needs
redaction or cannot be represented as Unicode argv. Source metadata/read/walk owners preserve
observed disappearance, source change and permission causes rather than flattening those
crossings. Command and query diagnostics share optional recovery facts; existing Tree-sitter
parser metadata remains source-owned. Native context reasons come from C/root's existing
semantic Failure owner, with preparation advice limited to a known missing native query session.

Focused tests are authored for origin/base preservation, whole-array attribution, scope alternative
replacement, explicit false, missing-root refusal, acquisition disappearance, native permission
classification, redaction and typed native recovery. Agent integration journeys are authored for
schema-backed passive explanation/refusal/replay, stdout query failures versus stderr command
failures, NDJSON terminal interpretation, source-ID joins, matching direct/guarded inspection,
and stale/malformed addresses. They await the coordinator's assembled Q campaign.

**Tested:** the coordinator's `scripts/pse test pse-host --lib -E
'test(semantic::wire::tests::) | test(source_read::) | test(source_command::batch::) |
test(source_query::) | test(detail_tests::) | test(investigation::config::) |
test(investigation::recovery_tests::) | test(envelope::)'` ran 105 selected default-feature tests:
104 passed and one native-wire test failed, against the zero-failure target. All 17 selected
configuration/recovery tests passed, including CLI replay, preflight-to-walk disappearance,
raw-path redaction and consistent command/query facts. Provider-selection/acquisition tests
were omitted by that filter and remain required. The original combined run is invocation
`01a19a5d-d0cf-7112-812b-617aa4b22fa8`; its sole failure belongs to the native wire's non-UTF8
query fixture. Earlier compile attempts were blocked by concurrent native constructor/generation
migrations and supply no test credit. No assembled acceptance claim follows.

Next: execute the omitted provider tests, settle native wire controls, then integrate B04's
native/association journeys and guidance with the single Q campaign.
