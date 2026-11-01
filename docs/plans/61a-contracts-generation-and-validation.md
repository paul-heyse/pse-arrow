---
title: "Plan61a: Contracts, generation and validation"
status: complete
doc_retention: while-dependent
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/61-code-intelligence-foundations.md
doc_topics: [current-work, design, libraries, validation]
review_sources: [docs/functional_reviews/code-intelligence-protobuf-foundations-2026-10-31.md]
adrs: [ADR-0235]
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

#### A00 realization assessment — 2026-10-31

**Interface-checked:** blueprint §21.7 and ADR-0233/0234 retain the independent host,
explicit preparation, provider/source qualification and original CPU/process settlement.
The present wave adds test-only qualification inside existing `pse-host` and `xtask`
owners; third-party dependency selection and these tests require no new ADR. No scientific,
Arrow or canonical database policy moves into the host. No new workspace crate is selected.

The subsequent production declaration/publication change requires the ADR/design route for
metadata conventions: new wire meanings are proto-authored, existing product-registry meanings
remain derived, and native directional presentation remains Serde/Schemars. Publish that scoped
decision and its required review status before A02 production consumers. The helper realization
must amend blueprint §21.7 through that same route, preserving ADR-0233/0234 custody and
settlement semantics. The functional review and this qualification do not constitute formal
design-review acceptance; no accepted ADR is amended by this test wave.

Placement is existing xtask explicit generation plus host-local generated contracts and tests.
One generator owns message/service/validation outputs and the complete immutable descriptor
corpus. Qualification fixtures are not a second public schema authority. B/C/D production
consumers must move together at their specified package boundaries after qualification.

Representative contract obligations are: unknown/false/true remain distinct; exact/lower-bound/
unknown counts retain their unit and precision; paths preserve original native bytes; source
and provider/Cargo guards qualify locations; original failures remain distinguishable from
schema/transport/validator defects. One operation starts once, admits operation-local cancel,
and reports Terminal only after owned CPU settlement. Input half-close permits completion;
reset/output loss cancels and drains under the session owner, but leaves completion unobserved
to the disconnected client. These contextual facts are checked by native owners, not CEL.

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

Before generation, add descriptor-bound CEL preflight using the selected CEL/compiler
libraries: parse and type-check expressions, resolve fields/functions against the actual
schema and selected generator support, and report actionable rule/source diagnostics. Reject
malformed expressions and required rules without a supported lowering before publishing
generated output. This is not a second handwritten CEL parser or a string-pattern allowlist.

Investigate generation improvements for valid CEL that the current transpiler mishandles.
Compare emitted Rust and CEL error semantics, especially numeric division/remainder, overflow,
short-circuiting and error propagation. Constant errors may be identified before generation;
field-dependent zero/overflow requires safe generated operations returning typed evaluation
defects rather than a panic. Check whether existing generator options or a compatible fixed
release suffice; otherwise assess a bounded generator correction/upstream contribution or
an existing supported evaluator. Do not reject valid CEL solely because the initial generator
lowering is deficient. Select the coherent route with its representation and dependency costs.

Acceptance includes invalid syntax, unknown fields/functions, incompatible types, constant
arithmetic errors and valid expressions whose data triggers an evaluation error; compare
against the selected CEL implementation or upstream conformance cases. Demonstrate that a
failed preflight cannot publish output and that corrected generation preserves owned/view
validation and structured error categories. Existing selected-rule qualification does not
complete this preflight/generator-correction slice.

Acceptance: clean generation and comparison agree; representative inputs/outputs compile; direct
reflection preserves presence/type; absent optional versus explicit default, unknown enum,
invalid combination and selected CEL rule behave deliberately. Descriptor imports/options survive
inspection and a conflicting filename refuses. Help/read/ordinary launch starts no generator.
Native directional schemas remain accurate for retained presentation. Actual B/C/D consumers
move with their packages; remove superseded declarations/generators as soon as replaced.

## Package state

### Qualification realization — 2026-10-31

#### A02 execution checkpoint

ADR-0235, blueprint revision 323 and the
[scoped prospective review](../design_review/reviews/design_review_code-intelligence-contract-foundation_2026-10-31.md)
publish the governing route at Accept-scoped target strength; the ADR remains proposed.
The implementation replaces the toy protocol with host-local production contracts and uses
the stable `codegen --only code-intelligence` route. A maintained local generator correction
reuses its parser/schema/lowering pipeline; no upstream contact or fork publication is authorized.

Selected supported numeric lowering must preserve exact scalar comparison and typed errors.
Heterogeneous numeric branches/collections requiring a dynamic representation are explicitly
unsupported rather than implicitly rounded to f64; this is a capability limit, not invalid CEL.
A concrete consumer requiring that representation must compare a supported dynamic/library
route before broadening the compiler. The CEL Rust evaluator shares some upstream precision
issues and is not an independent oracle for integer/float boundary cases; those need explicit
specification expectations. Full CEL conformance is not claimed.

The following A01 preparation details describe the original qualification baseline. Production
generation must use the corrected local generator and its positive preflight protocol admission;
the unpatched registry 0.7.2 binary does not satisfy A02 readiness.

Current corrected-tool preparation, after preparing the companion message/packaging/service
generators, is:

```bash
scripts/pse-env -- cargo install --path vendor/protoc-gen-protovalidate-buffa --locked --force --root .local/plan61-tools --target-dir target/plan61-tools-build
scripts/pse-env -- cargo xtask codegen --only code-intelligence
scripts/pse-env -- cargo xtask codegen --only code-intelligence --check
```

The maintained source and archive provenance are in the vendor README. Production admission
requires both the local vendored installation identity and `pse-cel-preflight-v1`; a same-version
registry plugin that ignores CLI arguments is rejected. The full descriptor is derived from the
same Buf image as code generation. Captured authored-file population/bytes and Buf configuration/
lock premises are verified before publication, so origin hashes cannot describe a different
concurrent source revision.

The foundation is test-only: host dev dependencies select Buffa, buffa-types and
buffa-descriptor 0.9.2, ConnectRPC 0.9.1 and protovalidate-buffa 0.7.2. The lockfile
records their resolved companions. No new production wire consumer has moved.

Prepare the generators explicitly from the repository root:

```bash
scripts/pse-env -- cargo install protoc-gen-buffa@0.9.2 protoc-gen-buffa-packaging@0.9.2 connectrpc-codegen@0.9.0 protoc-gen-protovalidate-buffa@0.7.2 --locked --root .local/plan61-tools --target-dir target/plan61-tools-build
```

The Connect generator binary is `protoc-gen-connect-rust`. The fixture's `buf.lock`
pins its validation-schema dependency; `buf.gen.yaml` records message/packaging shared-pool,
VTable, JSON/text, service and validator options. The existing xtask generation owner publishes
the result through `codegen --only plan61`; comparison uses that same target with `--check`.
Ordinary host compilation includes these consumers only under `cfg(test)`, and starts no generator.

**Interface-checked:** published package archives govern over mutable release tags and stale
README/API examples. Buffa's descriptor documentation described VTable as deferred, but its
published code generator contains the actual owned/view implementation. The validator's v0.7.2
Git tag manifest also differs from the published 0.7.2 archive. Neither discrepancy is grounds
for attributing an untested API to the selected resolved family.

Required-rule admission must distinguish successful generator execution from a working rule.
The selected simple bounds/cross-field comparison require revealing owned/view tests. Arbitrary
CEL is not qualified: constant division by zero emitted Rust rejected by `unconditional_panic`
during the investigation. Do not suppress that diagnostic, ignore the rule, or classify a
validator defect as invalid user input. Native checked construction retains contextual invariants;
additional rule shapes need their own demonstrated supported path before production use.

The published plugin's `src/emit/cel_compile.rs` already uses `cel::parser::Parser` and
a schema-aware compiler. Its arithmetic lowering emits bare Rust operators after numeric
promotion, including division/remainder and negation. The preflight work should reuse that
parser/schema information; it cannot prove that a field-dependent denominator is nonzero.
Investigate type-appropriate checked lowering and CEL error propagation, including evaluation
and short-circuit semantics, against upstream cases. Catching a panic in a negative test is
evidence of the defect, not the proposed production remedy. No generator fork or upstream
contact is part of the present qualification result.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| A00 | complete | Contract/decision integration owner | Qualification placement and production decision/review route published above; no ADR acceptance inferred |
| A01 | complete | Generated-contract integration owner | Representative exact-family owned/view/VTable/validation/generated-RPC composition tested; CEL defect limits and production obligations recorded |
| A02 | complete | Explicit codegen/validation owner | Production contracts, full descriptors/origins, shared-input publication and corrected CEL preflight validated; downstream migration remains with B/C/D |

**Proposed verification:** targeted generator, validator and schema tests; regeneration belongs
to declaration changes. Q01 owns assembled/static qualification. A01's successful small example
does not establish complete provider behavior or measured benefit. Dynamic validation, hosted
schema distribution, custom storage and lazy decoding retain coordinator triggers.

### Investigation outcome — 2026-10-31

**Tested:** `scripts/pse test pse-host --lib -E 'test(plan61_)'`, default nextest profile,
22 passed, zero failed against a zero-failure target; 672 tests outside the filter were not
executed. The final compile emitted no warnings. `scripts/pse check xtask --all-targets`
passed through the scope-aware force-validation route, and
`scripts/pse-env -- cargo xtask codegen --only plan61 --check` passed. `just ready` passed
both skills-sync and doctor. These checks qualify the representative test foundation, not
the complete application or scientific scope.

Direct projection borrows owned/view fields without dynamic transcoding. Selected bounds/CEL
comparison validate on owned and eager views, and unary/stream boundaries enforce their actual
request checks. Negative tests preserve malformed-rule error behavior and the arithmetic panic
as defects: malformed CEL is reported in `runtime_error`, not reliably `compile_error`, and
field-dependent division by zero panics. Do not infer clean compiler/evaluator classification
or arbitrary CEL safety from the existence of typed error slots. A02's added preflight and
generation-correction work remains open.

**Mistakes corrected:** preliminary tests assumed strict unknown-field text parsing and a
different malformed-CEL error category. The corrected tests document actual released behavior
and its limits. Review also found global-plugin fallback and a terminal-send/shutdown race;
local installation/version admission and bounded terminal delivery corrected those mechanisms.

**Deliberate boundaries:** retain Buffa/ConnectRPC for the demonstrated shared representation,
validation subset and local transport. No broad alternative prototype, production caller
migration, generator fork or blanket CEL support is claimed. A02/B/C/D and Q01 retain their
remaining obligations; the test fixtures do not authorize production use before the governing
decision/review route is satisfied.

## Outcome

**Implemented:** A02 now supplies production host-local proto contracts, generated owned/eager
views, validators and service bindings. Request predicates and count precision use typed
alternatives; answer demand has one limit owner. Full descriptor imports/options/source metadata
remain available independently of the stripped runtime pool. Generated Rust origin relations
identify declared corpus contributors, not exact Rust declarations or semantic identity.

**Implemented:** one compiled Buf image per corpus feeds descriptor export, descriptor-bound
CEL preflight and every plugin. The shared selected/aggregate staging boundary completes before
publication. Captured source/configuration premises are rechecked; stale, changed or added inputs
cannot publish mismatched origin metadata. The old Plan61 generator target and toy service are
retired; the useful all-field representation fixture remains test-only.

**Tested:** the following scoped commands passed against a zero-failure target:

| Command | Mode, scope and result |
| --- | --- |
| `scripts/pse test pse-host --lib -E 'test(plan61_)'` | Default nextest, production bindings: 22 passed, zero failed, 672 outside selection; zero compile warnings |
| `scripts/pse-env -- cargo test --manifest-path vendor/protoc-gen-protovalidate-buffa/Cargo.toml --target-dir target/plan61-tools-build --lib` | Pinned nightly, test profile: 146 passed, zero failed/warnings; includes 24 emitted-Rust execution goldens |
| `scripts/pse-env -- cargo test --manifest-path vendor/protoc-gen-protovalidate-buffa/Cargo.toml --target-dir target/plan61-tools-build --bin protoc-gen-protovalidate-buffa` | Same toolchain/profile: one CLI target-admission test passed, zero failed/warnings |
| `just ready` | Skills synchronization and doctor passed |
| `scripts/pse test xtask --bin xtask -E 'test(plan61_)'` | Default nextest: six passed, zero failed, 77 outside selection; includes snapshot mutations and publication-stage fault cases |
| `scripts/pse check pse-host --all-targets` | Scoped correctness route passed, zero warnings; xtask all-target compilation also passed |
| `scripts/pse-env -- cargo xtask codegen --only code-intelligence --check` | Selected production/fixture generation and source-relationship comparison passed |
| `just codegen` | Full normal publication passed, including generated ownership map and Ipopt bindings; this is generation, not assembled product qualification |
| `just turn-end` | ADR index and formatting passed |

The maintained generator corrects checked arithmetic/conversions, exact mixed scalar comparison,
logical/quantifier error masking and strict admission. Unsupported oneof/WKT field CEL and
heterogeneous numeric branch/list representations refuse explicitly. This is selected typed
lowering support, not general CEL conformance. No production panic-catching or default dynamic
transcoding bridge was introduced.

**Review:** the prospective review accepts the scoped target; independent implementation review
found no remaining material issue after correction. Publication fault evidence is shared-stage
fault injection plus source-verified aggregate ordering, with actual CEL/descriptor diagnostics
checked separately. It is not an end-to-end failing real-Buf/whole-aggregate campaign.

| Scoped review finding | Disposition | Evidence and limit |
| --- | --- | --- |
| F01 — scoped declaration authority | resolved | Production typed alternatives and owned/view/native-fact tests; live B/C/D migration remains separate |
| F02 — preflight before aggregate writes | resolved | Shared publication stage fault tests preserve current/stale outputs; aggregate placement verified by source, not a real-plugin aggregate failure campaign |
| F03 — supported validation semantics | resolved | Corrected compiler admission, 146 vendor library tests, one CLI test, 24 emitted-Rust goldens and host owned/view/quantifier tests; unsupported dynamic representations remain explicit |

**Mistakes corrected:** an early production schema preserved toy query/limit duplication;
typed questions and a single answer-demand limit replaced it. Review identified mixed numeric
precision loss, masked range errors, incomplete emitter coverage, same-version unpatched-tool
admission and source-hash races. Corrected lowering, positive protocol admission and captured
input guards address those cases. An initially selected library test filter did not exercise
the xtask binary tests; the final verification uses the actual binary target.

**Deliberate deviations and remaining work:** keep a local corrected third-party generator,
with archive provenance and tests, instead of an unverified version bump or new CEL engine.
No upstream contact/push occurred. ADR-0235 remains proposed; its formal status belongs to its
decision PR. B/C/D own actual evidence producers, callers, full managed custody/reconnect and
hard cutover. Q01 owns assembled qualification and series-end hygiene. No scientific or
measured-performance claim follows from this foundation.
