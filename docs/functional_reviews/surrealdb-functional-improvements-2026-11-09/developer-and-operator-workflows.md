# Developer and operator workflows

Supporting assessment for the [principal review](../surrealdb-functional-improvements-2026-11-09.md).
Inspected 2026-11-09 at `4f07766942ff3b21274fdfb686ad483031439462` and its dirty
working tree. Selected mechanisms are **Interface-checked**; behavior and benefits below
are **Proposed**. No tool build, database fixture, MCP server or product test was run.
Core 3.4, efficient architecture 1.0, process-simulator 1.5, test assessment 1.2 and
the binding guide this argument, particularly AP-03/AP-06/AP-07 and DP-15.

<a id="O15"></a>
## O15 — Consolidate executable database contracts under native fixture custody

**Proposed direction: use SurrealKit suites as the shared database-contract layer, with an
independent native consumer and one native outer service owner.** Add `test --exec` only
where suite-owned provisioning removes duplicate setup for that consumer. This extends an
existing capability rather than introducing a universal replacement test framework.

Today the [native SurrealKit fixture](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-operations/tests/native_service/surrealkit.rs)
stages generated schema, seed, rollout and actor inputs against a verified Rust-owned service.
It checks report size and outcomes, parity/drift and disposal. The
[declarative contract](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/scripts/tests/fixtures/surrealkit/canonical.toml)
contains exact Decimal/bytes and refusal cases. An empty denied response is challenged by
privileged readback; it is not accepted as proof that no write happened.

After the improvement, a developer changes a generated field or stored operation, runs its
focused contract group, and gets database assertion, independent client interpretation and
cleanup results in one understandable report. Shared seed/actor/module inputs should replace
repeated bootstrap and signing code. Keep deliberate malformed cases and independently derived
expectations at the call site so shared generation cannot normalize away the tested mistake.

SurrealKit's SQL, permission, metadata, behavior and HTTP cases, `schema_from=both`, actor
sessions and structured reports provide the native mechanisms. Retain a separate application
oracle for native codecs and original scientific interpretation. A generated suite can establish
declaration agreement without independently establishing that the declaration has the right meaning.

| Composition | Judgment |
| --- | --- |
| SurrealKit database assertions plus independent Rust reader in existing fixture | Recommended: consolidates actual database setup while retaining the two assurance roles. |
| Suite-provisioned logical DB and `test --exec` consumer | Recommended selectively where shared actors/schema remove a second logical setup path. Native owner still controls process, credentials, original deadline and physical drain. |
| Move all client/lifecycle assertions into TOML | Not selected: weakens the actual client/error/ownership evidence and can retain another interpreter of the same domain policy. |
| Keep separate full fixtures for every assertion | Retain only where restart/crash/isolation is the assertion; otherwise it repeats unrelated physical construction. |

Tagged [rc.1 runner source](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/runner.rs)
distinguishes structured case counts from outer `Result` and installs Ctrl-C handling for
`--exec`. Parallel fail-fast behavior and child settlement need explicit checks; generic
cleanup wording is not proof against arbitrary process termination. Choose one owner for
logical teardown and preserve infrastructure, assertion and cleanup failures separately.

Delivery examples: a generated numeric mistake is detected by an independent expected value;
denied mutation leaves no privileged-readable record; Sync and rollout construction agree;
timeout, child failure and interrupted fail-fast leave no owned logical DB or connection.
The native service remains available to independent tests without unrelated solvers. This
does not request a broad test-audit engagement or change Plan66 scientific acceptance.

Plan71 is being executed concurrently under the operator's direction, although its inspected
coordinator still says draft. Its [fixture efficiency and selected diagnostics owner](../../plans/71d-fixture-efficiency-and-selected-diagnostics.md)
and [original attempt evidence owner](../../plans/71b-nextest-execution-and-original-attempt-evidence.md)
already address shared exclusions, reuse, feedback and report interpretation. O15 proposes
SurrealDB-specific contract consolidation through those owners; it neither diagnoses those
in-flight capabilities as absent nor schedules a competing fixture or report layer.

<a id="O16"></a>
## O16 — Provide structured, scoped development access instead of repeated shell interpretation

**Proposed direction: make native CLI/SurrealKit structured operations the ordinary operator
surface, and offer read-scoped project MCP as an optional composition of those capabilities.**
Assess SQL/GraphQL as independent inspection clients, not automatic replacements for native
product contracts. This supports richer development use without inventing a web product.

Today [tool preparation](https://github.com/paul-heyse/pse-arrow/blob/4f07766942ff3b21274fdfb686ad483031439462/crates/pse-host/src/surrealkit.rs)
builds rc.1 with `cli`, SDK4/gRPC and rustls; it omits `mcp` and embedded KV. Native readers
and project commands already carry issuer/operation selections. The useful enrichment is a
developer asking for schema drift, definition references, migration status or a selected suite
result and receiving structured facts with the relevant source selection and original outcome.

Prefer subprocess JSON/library reports where they remove terminal parsing without importing
the entire tooling dependency closure. Enable embedded compile for O02 explicitly when used;
do not make every routine read compile a backend. Embedded/programmatic APIs are appropriate
only where they simplify the complete caller, including logging, resources and error reporting.

The [rc.1 feature declarations](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml)
separate these capabilities. The supplied [SurrealKit addendum](../../libraries/surrealDB/SurrealKit.md)
describes local MCP and selection APIs. A prepared read-scoped MCP process is a useful optional
adapter for interactive agents: it can expose inspection, lint and selected test authoring
facts. Its target/configuration is not native ownership of a listener or permission to apply
arbitrary schema/data changes. Client confirmation fields are not an authorization boundary.

**Standard client decision.** Keep the native typed front door as the preferred product and
operator contract. A PostgreSQL listener or generated GraphQL read surface can simplify an
independent evidence analyst's existing query/inspection workflow; select it when its type,
catalog, authorization and pagination subset covers that workflow. This is a conditional
capability with a plausible consumer, not a demand for an already existing caller. An API
generated from schema may eliminate resolver code, but its public schema is still a contract.
Do not infer unrestricted PostgreSQL, pgvector, MySQL or MongoDB wire behavior from connection
success. GQL path functionality is assessed with O08; GraphQL has a different role.

For browser access, cookies/JWT/refresh and durable HTTP sessions become relevant only with
that selected participant. Durable authentication state creates a persistence and revocation
obligation. The native local target does not need it merely to inspect a project. Vite/Effect
generation, templates, Action/container packaging, mql.js/Mongoose and Cloud MCP remain
alternative integration shapes; no separate frontend, hosted database management service or
Node stack is recommended solely to consume them.

Delivery examples: an agent obtains schema drift and a complete test report without parsing
colored logs; an unauthorized adapter cannot apply a rollout; stale target selection is refused;
an independent read client sees the same authorized values and continuation as the native owner.
Rule impacts concern interface selection, effect admission and any new listener; changing these
requires existing decision routes rather than treating a tool installation as approval.

## Discovery correspondence and limits

| Captured direction | Assessment |
| --- | --- |
| Declarative cases, actors, fixture inputs, parity and report consumption | O15: consolidate database contracts and keep independent native interpretation. |
| External test orchestration and physical cleanup | O15: selected composition under native outer custody; no blanket teardown guarantee. |
| Programmatic APIs, modules/targets/seeds and config/template variables | O15/O16 and O03: one effective input identity and owner, not a second bootstrap/configuration layer. |
| MCP and structured developer access | O16: optional scoped adapter to useful facts, with explicit feature preparation and effect limits. |
| SQL-wire, GraphQL, HTTP sessions, cookies/JWT | O16: conditional independent client; preserve exact authorized semantics and revoke/expire state deliberately. |
| Templates, Vite/Effect, packaging, Node/Cloud companions | O16: alternatives assessed but not selected for the recommended local Rust target. |

The [discovery test brief](../../library_leverage/libraries/surrealdb/briefings/contract-testing-and-developer-workflows.md)
and [tooling contracts](../../library_leverage/libraries/surrealdb/v4-tooling-contracts.md)
supply release-specific leads. Current fixture and runner source were rechecked. Full MCP
tool permissions, standard-dialect compatibility and interrupted composition remain bounded
delivery questions, not fresh qualification claims. No new framework, plugin configuration,
installed service or accepted plan change follows from this document.
