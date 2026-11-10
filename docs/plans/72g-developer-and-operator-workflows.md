---
title: "Plan72g: Developer and operator workflows"
status: complete
date: 2026-11-09
adrs: [ADR-0267]
doc_role: plan
doc_owner: docs/plans/72-surrealdb-functional-improvements.md
doc_topics: [current-work, libraries, deployment, validation, diagnostics]
review_sources: [docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md, docs/functional_reviews/surrealdb-functional-improvements-2026-11-09/developer-and-operator-workflows.md]
---

# Developer and operator workflows

**Proposed.** Deliver O15/O16 through the existing native fixture, command and tool-preparation
owners. A developer can change a contract, run its focused database cases and independent native
reader, and retrieve a structured result. An operator can inspect schema interpretation, drift,
transition readiness and selected evidence without parsing terminal logs or acquiring accidental
write authority. The [coordinator](72-surrealdb-functional-improvements.md) owns opportunity
dispositions; this companion owns package progress.

## Baseline and target foundations

**Interface-checked.** At `21f703887cc307f986975f9a2fba998f8bd2afa7` plus the concurrent tree,
the [native fixture](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/crates/pse-operations/tests/native_service/surrealkit.rs)
already verifies an owned loopback endpoint, provisions independent namespace credentials,
stages schema/seeds/rollouts, bounds reports and checks teardown. The source declaration
[suite](https://github.com/paul-heyse/pse-arrow/blob/21f703887cc307f986975f9a2fba998f8bd2afa7/scripts/tests/fixtures/surrealkit/canonical.toml)
includes numeric and denied-write witnesses. The selected tool is source-pinned SurrealKit
rc.1, built with CLI and SDK4/gRPC, without embedded backends or MCP.

Keep the native service as outer physical owner and use SurrealKit for the database contract
and selected logical provisioning it actually supplies. Do not create a second test runner,
result ledger or schema authority. Shared fixtures can centralize setup; independent malformed
inputs and expected values retain the ability to detect a wrong generated declaration.
This applies core AP-03/AP-06/AP-07, test assessment and blueprint §20/§24.

[Plan71](71-workflow-and-simulation-test-feedback.md) is actively executing under the maintainer's
direction regardless of its draft marker. Consume its original-attempt/report, fixture-exclusion,
CLI-reference and release contracts as delivered. This series adds SurrealDB-specific consumers;
it changes neither Plan71's target nor Plan66 scientific acceptance. Recheck affected source
before editing shared fixture/host surfaces and integrate with their active owner.

## Packages

### Execution checkpoint — 2026-11-10

Execution is authorized under proposed ADR-0267. Extend the existing native-owned SurrealKit
lane and compose one prebuilt connection-only independent consumer through test --exec.
Native owners retain endpoint/credential/deadline and complete physical namespace/child cleanup;
assertion, setup, consumer and cleanup outcomes stay distinct. Existing structured commands and
Plan71 result references remain the operator route. Unrestricted MCP tools also mutate project
files; Viewer credentials alone cannot make that surface read-only. No MCP/SQL-wire/GraphQL
listener is added. Plan71 is complete and supplies infrastructure, not a new execution scope.

The current source extends the existing owned suite with a prebuilt connection-only native
`--exec` consumer, independent full-u64/byte expectations, external failure/interruption and
logical cleanup readback. It preserves original work and cleanup failures independently.
Structured project commands consume native graph/path snapshots, compact study/run summaries,
bounded follow and original-incarnation persistence observations. Existing schema/drift,
transition and original-result commands retain their own owners. No terminal scraping or new
report ledger is introduced. Compilation, native suite and ordinary reader receiving now pass
the selected controls linked in the Outcome.

G04 selects the typed native route. The selected MCP surface also mutates local project files,
so Viewer database credentials do not make that adapter read-only. SQL-wire and generated
GraphQL would add listeners and another interpretation/continuation boundary without removing
the original issuer, exact-value and selected-result contracts served by the native reader.
They are not dormant production paths. Reopen for a named standard-client participant with
a complete native type, authority and continuation mapping that simplifies its whole workflow;
no browser/session or Node companion is introduced for this Rust operating target.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [G01](#G01) | Complete | Existing native fixture and prepared tool; selected 72a/72c contract as each changes | Shared contract groups and independent native readback exercise the changed declaration/operation with one physical owner. |
| [G02](#G02) | Complete | G01 suite/actor inputs; existing Plan71 result and fixture owners | Selected external consumers reuse logical provisioning and report assertion/infrastructure/cleanup outcomes without duplicate setup. |
| [G03](#G03) | Complete | Existing native command/output contracts; 72a schema inspection and 72b transitions when available | Structured inspection, drift/test/transition views carry exact selection, authority and original result references. |
| [G04](#G04) | Complete | G03 operation contracts; exact selected tool/client features | Resolve optional adapter branches by callable/type/authority fit; implement fitting read-scoped adapters and retain explicit nonselected outcomes. |

<a id="G01"></a>
## G01 — Share database contracts and preserve independent interpretation

Use SurrealKit SQL, permission, metadata, behavior and HTTP cases with shared modules, actors
and seeds. Group cases by the semantic contract they exercise, not one full server per assertion.
Keep Rust consumers for exact native codec interpretation, original response completeness and
physical lifecycle. Extend the existing lane as 72a/72c change generated types and operations;
do not block useful tests on every other companion being complete.

Use `schema_from=both` only for a selected Sync/rollout equivalence claim. It does not establish
historical data effects. Reuse G01 cases in 72b's receiving, but provide its independent old/new
data witnesses there. Report semantic assertion failure separately from provisioning and disposal.
The [rc.1 test implementation](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/runner.rs)
is the exact contract source; a successful outer result is not zero failed cases.

Migrate repeated setup/signing/report interpretation and delete the superseded helpers and
fixtures in the same change. Keep distinct physical fixtures only for a restart, crash,
exclusive effect or independent failure domain actually under test. Targeted acceptance:
an intentionally wrong numeric/identity declaration fails independent readback; denied mutation
leaves no privileged-readable row; a fixture setup failure remains distinguishable from a case
failure. No broad scientific run is needed to demonstrate this database contract improvement.

<a id="G02"></a>
## G02 — Compose external consumers under the original owner

Use `test --exec` where a native application consumer can reuse the suite's endpoint, actors
and selected schema rather than construct another logical database. The native fixture retains
the physical server, credential handoff, original enclosing deadline and teardown. The suite owns
the inner logical setup it creates; it must return teardown outcome without discarding the
external consumer's original failure. Current fixture composition remains the direct target
where invoking another executable adds no shared preparation benefit.

The serious risk here is leaked ownership or a falsely successful report after interruption.
Inspect the tagged runner's Ctrl-C and fail-fast paths; add focused cases for child failure,
timeout, interrupted parallel work and failed logical cleanup. This is a lifecycle control,
not an efficiency benchmark. Delegate the removed inner preparation once; retire its replaced
Rust bootstrap path, while retaining independent consumer expectations and malformed input.
Join outcomes into Plan71's existing report/reference readers rather than emit another ledger.

<a id="G03"></a>
## G03 — Make structured facts the ordinary operating route

Expose the existing native CLI/SurrealKit JSON or library reports through responsibility-owned
operations for schema status/drift, installed interpretation, focused tests and transition
preview/status. Requests select the intended issuer and input/operation identity; responses
separate facts, unavailable coverage and permitted next effects. Inspection never prepares a tool,
starts a database, applies a transition or runs an analyzed query implicitly.

Use subprocess structured output when it keeps the prepared tool independent. Use programmatic
APIs only where embedding removes a concrete boundary without expanding ordinary product
dependencies or creating process-global logging conflicts. Prepare embedded compile explicitly
for 72a; an ordinary read must not build a database backend. Consume bounded progress separately
from terminal outcome through 72h and Plan71's existing output/release contract.

Delete terminal-text scraping and redundant projection branches that the new structured reader
replaces. Targeted acceptance uses a real stale target, denied operation, incomplete report and
unprepared tool; each returns the original meaningful state and recovery reference. A developer
can obtain drift and case results without colored-log parsing. No interaction benchmark is required.

<a id="G04"></a>
## G04 — Resolve useful adapter choices without multiplying production paths

For local project MCP, inspect rc.1's callable tools and exact feature selection against G03's
already defined operations. Adopt a separately prepared read-scoped adapter when Viewer/explicit
tool restrictions enforce that scope and it composes the existing selection/outcome contract.
If the tool surface cannot prevent effects or preserve original result selection, retain the
structured native route and record that contract gap. Confirmation text is not authorization.
The [feature manifest](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml)
and [review O16](../functional_reviews/surrealdb-functional-improvements-2026-11-09/developer-and-operator-workflows.md#O16)
identify the selected release and concrete capabilities.

Apply the same source-based decision to PostgreSQL/MySQL or generated GraphQL inspection of
selected evidence. The deciding uncertainty is fidelity of types, permission and continuation,
not whether another caller is already written. Use a representative authorized read and denied
mutation only where source contracts leave a serious semantic risk. If a standard client fits,
implement one read projection and remove the replaced bespoke adapter; otherwise retain the native
contract. A listener or public-contract change follows its applicable decision route before launch.

Cookies/JWT refresh and durable HTTP sessions are included only with a selected browser/HTTP
participant and explicit expiry/revocation ownership. Vite/Effect, templates, Node/mql.js/Mongoose,
Cloud MCP and packaging variants are not selected for the local Rust target. Revisit when such
a participant makes an existing workflow simpler; do not add a dormant server or frontend now.

## Verification and source coverage

**Proposed verification.** Compile affected host/operations targets and run focused G01–G04
tests during implementation, retaining explicit force-validation where Arrow is consumed.
Integrated receiving belongs to [72i](72i-integrated-acceptance-and-closeout.md), not each package.
No product test, tool preparation or MCP installation was run to author this document.

| Review direction | Owned route |
| --- | --- |
| Suites, actor probes, seeds/modules, parity and independent oracles | G01; data migration effects remain 72b's contract. |
| External tests, report categories and physical disposal | G02 with Plan71 fixture/evidence owners. |
| JSON/library APIs, configuration identity, preparation and progress | G03; effective transition inputs are 72b, progress interpretation is 72h. |
| MCP, SQL/GraphQL and HTTP/session boundaries | G04 explicit fit/authority decision and selected adapter implementation. |
| Web/Node/hosted companions | G04 nonselected branches with participant-based triggers. |

## Outcome

**Implemented**, 2026-11-10. G01–G04 extend the existing owned SurrealKit suite with a
prebuilt connection-only `test --exec` native reader. The suite owns logical provisioning;
the original native fixture owns endpoint, credentials, deadline and physical settlement.
Independent u64-maximum and byte expectations exercise a real foreign consumer. Deliberate
consumer mismatch and interruption preserve the original failure plus cleanup outcome.
Parent-side bounded reports and native namespace inventory reject false success and remove
only owned logical populations. No duplicate database bootstrap or result ledger is added.

Ordinary `pse project` graph/path, study/run summary, finite follow and persistence reads use
typed connection-only product readers. They perform no implicit database startup, preparation
or mutation. Developer graph description/static explanation/explicit experiment remain distinct.
Existing schema/drift/transition/result routes retain their original owners.

**Tested** in the [operator receiving record](72-surrealdb-functional-improvements.md#operator-and-diagnostic-receiving):
the complete native SurrealKit fixture passed against zero failures, including original
contract/parity/drift cases, deliberately wrong setup/seed inputs, setup and seed interruptions,
external pass/fail/interruption, logical inventory cleanup and physical disposal. The ordinary
runtime reader also passed against zero, including selected-store/foreign cursor refusal,
summaries, real fixed-Viewer follow and original-server persistence inspection.

**Mistake corrected.** The independent helper expected a bare digest while the selected
semantic identity is `sha256:<64 lowercase hex>`. It now validates that actual contract.
Native JWT Authenticate also required F's narrow neutral-token correction. Diagnostic phases
and error classes are retained without exposing credentials or raw child setup messages.

**Deliberate deviations.** SurrealKit remains a prepared subprocess. MCP's filesystem effects
cannot be fenced by Viewer credentials; SQL/GraphQL listeners would add an interpretation and
continuation participant without simplifying the chosen native operation. Those branches and
browser/Node companions remain unselected with named participant triggers above. Blueprint
§21.7 and the substrate guide retain operating contracts under proposed ADR-0267. G01–G04
implementation is complete; 72i retains series acceptance and retirement.
