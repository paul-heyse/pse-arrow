---
title: "Plan61a: Contracts, generation and validation"
status: active
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/61-code-intelligence-foundations.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/functional_reviews/code-intelligence-protobuf-foundations-2026-10-31.md]
adrs: []
---

# Contracts, generation and validation

## Target and focused foundation assessment

**Proposed:** supply one declared wire contract and a reproducible generated foundation for
[Plan61](61-code-intelligence-foundations.md). O04/O06/O08 and the shared parts of O01/O05
contribute here. The coordinator owns dispositions and confirmed RC01–RC03.

Current `xtask/src/codegen.rs` assembles and publishes GeneratedTree outputs explicitly;
`private_documents.rs` distinguishes compiled native input/output schemas. Keep that owner.
Do not introduce automatic build.rs regeneration of the same outputs. Registry meanings remain
registry-owned; new wire-only query/evidence/control meanings can be proto-owned. Presentation
shapes remain native Serde/Schemars. A generated type may be used directly where semantics fit;
narrow checked construction adds invariants, not an independently writable mirror model.

Read the [Buffa](../libraries/proto/Buffa.md) generation/reflection sections, [validation](../libraries/proto/protobuf_validation_rust.md)
§§4/6/7, [compilation](../libraries/proto/protobuf_compilation_and_orchestration_protox_protoc_and_buf.md)
and [generator ecosystem](../libraries/proto/protobuf_generator_ecosystem_rust.md) together.
Shared descriptor pools require matching codegen/packaging corpora and options; native CEL
generation and bridge-based validation have different traits, errors and dependencies.

## Packages

<a id="a00"></a>
### A00 — Establish governing realization and integration placement

Consumes the operator-confirmed decisions. Apply the existing ADR/design route to the scoped
contract, metadata/publication and helper realization before affected changes. Assess existing
module/feature placement rather than assume shared contracts require a new crate. Record
current versus proposed authority, and any justified additional rule impact. A required formal
review is not supplied merely by the source functional review. No unrelated audit or completed
plan revision is initiated.

Completion is a published actionable decision route and owned placement, not automatic ADR
acceptance. Any new crate or family-major change uses its governing process; ordinary exact
third-party dependencies remain normal implementation judgment.

<a id="a01"></a>
### A01 — Qualify the consumed contract and exact library composition

Consumes A00. Settle typed request/evidence/failure/context/control meaning with B/C/D consumers,
including three-valued predicates, count precision, raw paths, source/context guards and terminal
facts. Avoid a universal HIR type algebra. Choose exact compatible Buffa/ConnectRPC/validator
runtime and generator versions/features from the reviewed0.9.2/0.9.1/0.7.2 starting set.

During execution, compile a representative generated owned/eager-view/reflection/constraint
and service surface against that exact family. Exercise direct reflection without converting
to a dynamic tree and validation on an actually supported view. Inspect tagged sources first;
use a bounded probe only to settle consequential unknowns. Latest Context7/main examples do
not establish pinned availability. This is a targeted foundation check, not a prototype of
every alternative or an application qualification claim.

Record selected generated APIs, options, unsupported rules and error categories. If the direct
projection/validation route or authenticated transport cannot fit without substantial bridging,
reconcile the reviewed Prost/Tonic alternative before dependent work; do not silently select
another runtime. Preserve scoped authority and actual completion whichever library is selected.

<a id="a02"></a>
### A02 — Publish generated contracts, descriptors and selected validation

Consumes qualified A01. Before affected production changes, establish the decision/review
status required by A00's governing route; filing a proposal or completing the route inventory
alone does not establish that readiness. Integrate pinned local Buf generation into existing explicit codegen
publication. Select a complete import corpus, exact plugin/packaging options and one immutable
descriptor artifact. Buffa VTable reflection and shared pool settings must agree across message
and packaging generators. Preserve encoded custom options/source metadata; reject conflicting
logical filenames before a pool can silently skip duplicates. Include generated sidecars and
origin metadata through their declared owner. Do not hand-edit generated roots.

Use protovalidate-buffa's matched generator for selected structural/cross-field constraints.
Unsupported required rules fail explicitly. Distinguish violations from validator compilation
or execution defects; request macros do not validate all response/stream items. Checked native
construction owns contextual invariants; fresh source/Cargo/CPU facts cannot be proved by CEL.
Do not duplicate Rust type-enforced rules in a second evaluator.

Acceptance: clean generation and comparison agree; representative inputs/outputs compile; direct
reflection preserves presence/type; absent optional versus explicit default, unknown enum,
invalid combination and selected CEL rule behave deliberately. Descriptor imports/options survive
inspection and a conflicting filename refuses. Help/read/ordinary launch starts no generator.
Native directional schemas remain accurate for retained presentation. Actual B/C/D consumers
move with their packages; remove superseded declarations/generators as soon as replaced.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| A00 | planned | Contract/decision integration owner | Governing route and placement established before affected changes |
| A01 | planned | Generated-contract integration owner | Exact consumed library/feature composition and target meaning qualified |
| A02 | planned | Explicit codegen/validation owner | Working generated foundation, complete descriptors, constraints and revealing checks |

**Proposed verification:** targeted generator, validator and schema tests; regeneration belongs
to declaration changes. Q01 owns assembled/static qualification. A01's successful small example
does not establish complete provider behavior or measured benefit. Dynamic validation, hosted
schema distribution, custom storage and lazy decoding retain coordinator triggers.
