---
title: "Plan60f: Agent interaction and contract diagnostics"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Proposed ADR-0235 and qualification readers consume the retained interaction, wire-disclosure and correction evidence."
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/60-engineering-code-intelligence.md
doc_topics: [current-work, design, libraries, diagnostics]
review_sources: [docs/plan_improvement_reviews/plan60-engineering-intelligence-2026-11-01.md, docs/functional_reviews/plan60-agent-engineering-opportunities-2026-11-01.md]
adrs: [ADR-0235]
---

# Agent interaction and contract diagnostics

## Purpose and foundation

**Proposed:** deliver improvement I06 and functional O05/O06 through existing parser,
envelope, configuration and schema owners. The coordinator owns dispositions. A03 owns
question-shaped evidence; E01/E03 own guidance and assembled use. This document owns F01–F03.
Starting codebase is `42bede27a`, after Plan62's completed
[Outcome](62-code-intelligence-capability-expansion.md#outcome). Protocol4 typed outcomes,
guarded native actions and exact proto-source origins are available foundations, not F work
to rebuild. F01–F03 still require their new discovery/handoff/correction consumers and evidence;
Plan62 completion does not mark these packages complete.
Plan61 supplies generated contracts/validators and original RPC diagnostics. Existing catalog,
Figment composition, ResultReference::encode/decode and diagnostic projections are foundations.
Review probes demonstrated overly wide discovery, invalid-field classification and JavaScript
rounding of numeric nanosecond guards. Preserve exact freshness checks and native identities.
Blueprint §21.7 and ADR-0235 retain their authority; classify concrete publication obligations
before implementation. New wire meanings use authored proto/generation; CLI views remain
Serde/Schemars-owned. No new governing exception, service or dependency is assumed.

## F01 — Concise discovery and reliable addressed handoff

### Authorized execution refinement

Execute F01–F03 from `cea33a310`. `commands` becomes concise parser-derived discovery;
`help` retains full selected metadata. Publish native opaque addresses through common adapters.
Preflight typed presentation choices before providers where response schemas determine them;
reuse actual Figment composition and preserve diagnostic metadata. F03 adds optional authored
Failure validation-disclosure metadata: original/emitted counts only when known, explicit
defect and whole-record coverage/disclosure facts. Native producers encode upstream once then
decode actual emitted details; receivers without original metadata retain unknown completeness.
Raw custody cannot restore selectors/messages removed by RPC publication policy. Protocol7
fences new wire meanings. No alternate validator/error protocol or dormant-schema prototype.

Consumes delivered parser/catalog/reference owners, independently of new structural edges.
Provide a concise command-description view and expansion to full argument/schema metadata
through existing commands. Derive syntax, values and aliases from Clap CommandFactory/public
getters. Build selected deferred commands before introspection; private conditional validation
remains explicitly unavailable. Lazy candidate providers must remain passive.

Publish native-encoded opaque addresses or ready invocations consistently for addressed general
results/candidates, reusing ResultReference::encode and existing actions. Keep original structured
detail available. Human/JSON/NDJSON carry the same address meaning. Clients must not need to
decode/re-encode large integers; do not introduce a second reference identity or relax guards.

Acceptance: concise discovery followed by argument expansion; opaque handoff through ordinary
JavaScript succeeds; integer-preserving structured handoff agrees; stale digest/world refuses.
Include supported source-only non-UTF-8 paths, unavailable metadata, and a rich record under a
record-count limit. Record width, provider demand and count stay independent. Delete duplicated
description/address assembly as callers migrate; preserve full parser and original inspection.

## F02 — Effective input and actionable correction

Consumes existing validation, Figment origins and envelope diagnostics. Explain the top-level
--fields grammar, report valid choices, and classify invalid presentation/request input as such
rather than infrastructure failure. Preserve Clap ErrorKind/context and Thiserror/Miette typed
causes, diagnostic_source, labels, related findings and help in the existing envelope. Do not
parse rendered messages or introduce a parallel error protocol/JSONReportHandler wire shape.

Use ValueSource/typed parsers where explicit-input distinctions matter. Extract provenance
from the same Figment composition as execution. `Tagged<T>`/`RelativePathBuf` serve concrete
typed-source/path needs only; do not duplicate the existing origins map. Effective provenance
is not overwritten-value history. Missing keys and provider errors remain distinguishable.

Acceptance: invalid nested field recovers to supported top-level choices; invalid combinations
fail before providers; --explain distinguishes defaults/overrides; stale references and invalid
schema fields retain distinct causes. Exercise machine and human output, multibyte labels and
related diagnostics. Conversion through Report/IntoDiagnostic must not erase rich metadata.
Remove replaced string-only classifications after consumers migrate.

Indicatif remains observed stderr presentation, separate from work settlement/stream finality.
Test non-TTY output, partial failure and cancellation. Use draw-target/MultiProgress facilities
only for an actual changed concurrent display; InMemoryTerm can verify that rendering. No new
progress renderer, logging bridge or ETA subsystem is mandatory.

## F03 — Schema-addressed validation correction

Consumes selected compiled validation, immutable descriptors, original RPC custody and F02's
shared correction adapter; does not require native graph completion. Project supported Violation,
FieldPath/FieldPathElement and decode_violations detail through static payload/original RPC
inspection. Preserve numbered/indexed paths, rule ID, schema identity and disclosed messages.
Retain original_rpc_details and input bytes independently; do not reconstruct from strings.

Distinguish violations from validator compile/evaluation defects. Pinned protovalidate-buffa0.7.2
RPC details sanitize some map selectors/messages and cap whole violations at4KiB. Preserve
redaction, unavailable locations and truncation without inventing map keys or complete totals.
External dynamic schemas still expose metadata without implying evaluated validation.

Acceptance: nested/repeated-index correction, redacted selector, truncated detail, required-rule
failure and compile/evaluation defects. Schema validation cannot establish native fact truth.
Migrate string-only correction where structured originals exist; keep original expansion.

## Conditional schema capabilities

Functional O07 remains deferred until a selected external-input consumer has imports available
only as descriptors and existing Buf/protoc acquisition is insufficient. The schema acquisition
owner then compares protox Compiler/FileResolver, DescriptorSetFileResolver and ChainFileResolver
with current acquisition. Preserve logical filenames, custom options, complete imports/source
information and compiler identity; use encode_file_descriptor_set where ordinary Prost conversion
loses options. Verify exact release/Edition support and conflicting/missing imports before adoption.
No prototype is required while the trigger is absent.

External runtime-schema validation can separately trigger dynamic Validator assessment;
browsable contract reference can trigger protoc-gen-doc. JSON Schema/PBJSON requires an external
JSON consumer and int64/presence/WKT/Edition assessment. Interoperability tools require a concrete
diagnostic question. These options do not replace typed relations, current presentation or the
selected stack. Coordinator rows own deferred disposition; this schema owner supplies decision
evidence. No registry, listener or second decoder is selected.

## Sources, packages and verification

Use the review's source map and [Clap](../libraries/cli/clap.md) §§3,6–7,9–11,
[Figment](../libraries/cli/figment.md) §§2–8, [diagnostics](../libraries/cli/thiserror_and_miette.md)
protocol/wrapping sections, [Indicatif](../libraries/cli/indicateif.md) §§2–7 and
[validation](../libraries/proto/protobuf_validation_rust.md) §§3.5,6.3,7.2. Context7 discovery
precedes consequential pinned-source verification. Buffa0.9.2 skips unknown text names despite
newer main claims, so keep the qualified strict protoc route. No version bump is selected.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| F01 | complete | Parser/catalog and reference adapters | Concise discovery, full expansion and opaque addresses exercised by actual callers |
| F02 | complete | Request/configuration and envelope adapters | Typed preflight, literal field choices and effective provenance preserve original diagnostics |
| F03 | complete | Schema/payload and RPC inspection | Original structured violations with origin-qualified disclosure, truncation and defect limits |

**Proposed verification:** targeted parser/envelope/reference/schema checks and actual public
interactions accompany changes; regenerate authored contract changes. E03 consumes F01–F03
for discovery-to-answer and correction journeys. Q01 owns scope-end qualification. New guidance
uses working parser/schema syntax. No numerical speed/token/memory target or new Tested product
claim follows from document publication.

## Outcome

**Implemented:** `commands` is concise parser-owned discovery; `help` retains full selected
argument/schema detail. Every serialized result reference includes its native opaque address;
identity encoding is unchanged and decoding rejects structured/address disagreement. Literal
top-level `--fields` choices are preflighted before providers where the selected response
schema defines them. Dots and slashes do not navigate nested data; actual punctuation-bearing
top-level keys remain valid. Opaque JSON retains runtime projection checks. Existing Figment
composition supplies effective provenance without inventing overwritten-value history.

Shared correction preserves typed diagnostics, numbered/indexed field paths, rule/schema
identity and original disclosed RPC detail. Native producers encode upstream once and count
the emitted canonical details. Authored Failure disclosure distinguishes known original totals,
redaction, whole-record truncation, compile/evaluation defects and unknown peer coverage.
Removed raw diagnostic expansion cannot restore redacted selectors/messages. Unary package
context now uses the same borrowed validator once with its actual message identity; no
parallel validator or decoder was introduced. Authored/generated protocol7 fences the new
wire meaning.

**Tested, local Linux:** [the coordinator Outcome](60-engineering-code-intelligence.md#outcome)
owns exact commands and final qualification. Parser/reference/envelope/schema/native-RPC tests
cover pre-provider refusal, literal fields, multibyte labels, related diagnostics, nested/repeated
correction, redaction/truncation and validator compile/evaluation defects. Actual HTTP2
required-input and defect controls passed. Ordinary Node `JSON.parse` copied a real opaque
source address containing a timestamp beyond JavaScript's safe integer range; ordinary
inspection accepted that unchanged address. Structured address disagreement and stale
source/native context remain refused. Public output tests exercised JSON/human/NDJSON,
non-UTF-8 paths, non-TTY progress, partial failure and cancellation. Parser help/schema
snapshots were reviewed and asserted in normal mode after updating stale delivered-contract
expectations; update mode alone receives no acceptance credit.

A mistake corrected was treating opaque JSON projection as a schema-known finite object,
which regressed existing runtime behavior. Explicit scalar/array rejection and typed finite
unions now coexist with opaque runtime checks. Another correction removed a macro-generated
unary path that bypassed validation. Deliberate choices retain the existing diagnostic and
progress owners, native reference identity and disclosure policy; external schema acquisition,
dynamic validation, JSON contracts and interoperability tooling remain trigger-bound. No
performance/token/memory benefit is claimed.
