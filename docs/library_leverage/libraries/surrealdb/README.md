# SurrealDB ecosystem

The [v4 discovery briefing](briefing.md) and [coverage account](coverage.md), inspected
2026-11-09, route twelve thematic briefs across existing product and developer functionality.
They reconsider both v4 additions and earlier capabilities. Breaking changes are eligible;
the proposed directions do not establish applicability, adoption or implementation scope.

The production baseline is the v4 source-built server/persistent MX and bounded native
protocol client, plus separately prepared Surrealism and SurrealKit. See the briefing for
current interpretation and the native owners for live selection and acceptance.

Selected reusable contract notes from this discovery:

- [V4 runtime contracts](v4-runtime-contracts.md): value-codec fidelity, extension
  invocation, API authority and observation/replay distinctions.
- [SurrealKit rc.1 contracts](v4-tooling-contracts.md): compilation, semantic
  documents, emitters, migration coordination, tests and bounded progress.

## Historical applicability and types

Library family account from the first applicability assessment (2026-10-27). Each page keeps its reusable types separate from its current applicability; this index routes to them.

The entries below retain that assessment's source/version conditions. In particular, v3
SDK, protocol, parser, telemetry and module conclusions are not refreshed v4 conclusions.
Use the new thematic briefs for changed premises and current research questions.

| Page | Types | Current applicability |
| --- | --- | --- |
| [SurrealDB Rust SDK (`surrealdb` / `surrealdb-engine-api`)](rust-sdk.md) | `surrealdb-rust-sdk/grpc-request-lifecycle-controls`<br>`surrealdb-rust-sdk/grpc-live-subscription` | CBO-009, CBO-016 |
| [SurrealDB protocol crate (`surrealdb-protocol`) with `surrealdb-types` proto conversions](protocol.md) | `surrealdb-protocol/generated-grpc-client`<br>`surrealdb-protocol/conformance-suite` | CBO-009 |
| [SurrealQL server constructs (SurrealDB 3.3 server)](surrealql-server-constructs.md) | `surrealql/stored-function-transaction-guard`<br>`surrealql/single-statement-implicit-transaction`<br>`surrealql/define-param-constants`<br>`surrealql/changefeed-replay`<br>`surrealql/live-select`<br>`surrealql/request-time-ceiling` | CBO-010, CBO-016, CBO-046 |
| [SurrealQL parser and AST (`surrealdb-syn` / `surrealdb-sql`) with `surrealdb-types::ToSql`](syn-and-sql.md) | `surrealdb-syn/parse-validate`<br>`surrealdb-sql/ast-render` | CBO-046 |
| [SurrealDB server tooling (`surreal` CLI, HTTP probes, export/import) and SurrealKit](server-tooling.md) | `surrealdb-server/readiness-probes`<br>`surrealdb-server/logical-export-import`<br>`surrealdb-server/telemetry-export-transport`<br>`surrealkit/schema-rollout` | CBO-048 |
