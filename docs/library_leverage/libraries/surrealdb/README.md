# surrealdb

Library family account from the first applicability assessment (2026-10-27). Each page keeps its reusable types separate from its current applicability; this index routes to them.

| Page | Types | Current applicability |
| --- | --- | --- |
| [SurrealDB Rust SDK (`surrealdb` / `surrealdb-engine-api`)](rust-sdk.md) | `surrealdb-rust-sdk/grpc-request-lifecycle-controls`<br>`surrealdb-rust-sdk/grpc-live-subscription` | CBO-009, CBO-016 |
| [SurrealDB protocol crate (`surrealdb-protocol`) with `surrealdb-types` proto conversions](protocol.md) | `surrealdb-protocol/generated-grpc-client`<br>`surrealdb-protocol/conformance-suite` | CBO-009 |
| [SurrealQL server constructs (SurrealDB 3.3 server)](surrealql-server-constructs.md) | `surrealql/stored-function-transaction-guard`<br>`surrealql/single-statement-implicit-transaction`<br>`surrealql/define-param-constants`<br>`surrealql/changefeed-replay`<br>`surrealql/live-select`<br>`surrealql/request-time-ceiling` | CBO-010, CBO-016, CBO-046 |
| [SurrealQL parser and AST (`surrealdb-syn` / `surrealdb-sql`) with `surrealdb-types::ToSql`](syn-and-sql.md) | `surrealdb-syn/parse-validate`<br>`surrealdb-sql/ast-render` | CBO-046 |
| [SurrealDB server tooling (`surreal` CLI, HTTP probes, export/import) and SurrealKit](server-tooling.md) | `surrealdb-server/readiness-probes`<br>`surrealdb-server/logical-export-import`<br>`surrealdb-server/telemetry-export-transport`<br>`surrealkit/schema-rollout` | CBO-048 |
