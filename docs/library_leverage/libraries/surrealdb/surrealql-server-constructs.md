# SurrealQL server constructs (SurrealDB 3.3 server)

Realizations considered: the locally source-built SurrealDB 3.3.4 server. Its patch series is recorded separately and covers SurrealMX storage, gRPC connection ownership and admission, and status details. Language semantics are those of `surrealdb-core` 3.3.4. Upgrade route: none released beyond 3.3.4 as of 2026-10-27.

Research date 2026-10-27. Sources:
- Context7 `/websites/surrealdb`. Pages:
  - `reference/query-language/statements/define/function`: transactional behaviour, permissions.
  - `…/language-primitives/transactions` and `…/statements/cancel`.
  - `…/statements/show`.
  - `learn/querying/real-time/changefeeds`.
  - `…/statements/define/table`: the `CHANGEFEED @duration [INCLUDE ORIGINAL]` clause.
  - `…/statements/define/param`.
  - The JavaScript and Mojo live-query transport notes.
- Installed `surrealdb-core-3.3.4/src/{cf,doc/changefeeds.rs,lq}`: located, not read in depth.
- Installed `surrealdb-protocol-0.13.1` `RequestContext.timeout` documentation.

## Types

### surrealql/stored-function-transaction-guard
- **Need it serves:** keep a repeated guard (preconditions, fences, deadline checks, refusals) in one server-side declaration instead of re-assembling it as script text at each call site.
- **Recognizable code shapes:**
  - `format!("BEGIN; {guard} {body} COMMIT;")`.
  - Shared string constants holding `BEGIN; LET … FOR UPDATE; IF … { THROW … };` preludes.
  - The same `IF time::micros() >= $expiry { THROW … }` line repeated in many scripts.
- **Mechanisms and APIs:**
  - `DEFINE FUNCTION fn::ns::name($arg: type, …) -> type { … } [PERMISSIONS …]`.
  - Bodies may run `SELECT … FOR UPDATE`, mutations, `IF`, `THROW` and `RETURN`.
  - A call made inside a manual transaction joins the caller's transaction. A failure, including a `THROW`, aborts the caller's transaction.
  - A function can return an object or record so the caller can use values the guard established, for example `LET $g = fn::guard(...); … $g.pin …`.
  - `DEFINE FUNCTION OVERWRITE`/`IF NOT EXISTS` and schema export carry functions with the rest of the schema.
- **Obligations discharged:**
  - One authoritative definition of the guard logic, installed with the schema.
  - Server-side atomicity with the rest of the transaction.
  - Typed refusals through `THROW` strings.
  - Callers send shorter scripts.
- **Residual caller obligations:**
  - The outer `BEGIN; … COMMIT;` text, or a single-statement implicit transaction (next type).
  - Binding per-request parameters, such as the deadline and authority identities.
  - Statements that must sit at top level, because `BEGIN`/`COMMIT`/`CANCEL` inside a function body are accepted but have no effect. The same holds inside blocks: they are ignored without error (re-confirmed through Context7 `/websites/surrealdb`, 2026-10-27).
  - Variables a later statement needs are not visible after the function returns: a `LET` inside a function does not leak, so values must come back explicitly.
  - Versioning functions with the schema, for example `fn::x_v1`.
- **Semantic conditions:**
  - The body runs with the caller's permissions. `PERMISSIONS` only restricts who may call.
  - Locks taken with `FOR UPDATE` inside the function belong to the caller's transaction.
  - Errors surface as the statement's error, so a typed refusal string survives.
  - Changing a function's definition is a schema change, so a deployed catalog must move with the client.
- **Version / build conditions:** SurrealDB 2.x and 3.x. The behaviour above is documented for current 3.x. `FOR UPDATE` is 3.x syntax.
- **Limits and negative results:**
  - SurrealQL has no parameterised "transaction template" or macro construct: `DEFINE` cannot make a prelude that wraps an arbitrary caller body.
  - `DEFINE EVENT` fires after a mutation; it is not a precondition wrapper.
  - A guard that must run before arbitrary caller statements can be a function call. Its result can be checked or bound at top level, but the `BEGIN`/`COMMIT` framing stays textual.
  - Search scope: the Context7 `define/function`, `transactions`, `cancel`, `define/event` and `define/param` pages.
- **Evidence:** Interface-checked: the documentation pages listed above.

### surrealql/single-statement-implicit-transaction
- **Need it serves:** atomic execution of a single call without explicit transaction framing.
- **Recognizable code shapes:** `BEGIN; RETURN fn::…(…); COMMIT;` around one statement.
- **Mechanisms and APIs:** each top-level statement outside `BEGIN … COMMIT` runs in its own implicit transaction, so a single `RETURN fn::x(...)` is atomic.
- **Obligations discharged:** atomicity for one statement.
- **Residual caller obligations:** confirming that client-side result handling expects one statement result rather than a transaction-shaped response. Multi-statement scripts still need explicit framing.
- **Semantic conditions:**
  - Conflict and retry classification should be identical, since there is one transaction either way.
  - Statement indices in the response change, because there is one result instead of three framing results.
- **Version / build conditions:** SurrealDB 3.x.
- **Limits and negative results:**
  - It was not verified that the server returns the same typed `TransactionConflict` detail for an implicit transaction as for an explicit commit failure.
  - The documentation describes implicit transactions only qualitatively.
- **Evidence:** Interface-checked: the Context7 transactions page ("implicit transactions" section) and the define/function transactional-behaviour note.

### surrealql/define-param-constants
- **Need it serves:** give stored functions and queries numeric or structural constants (page sizes, byte caps, sentinel counts) without textual substitution into the definition source.
- **Recognizable code shapes:** `.replace("__ROWS__", &N.to_string())` or `/* LIMIT */` placeholders in generated SurrealQL.
- **Mechanisms and APIs:**
  - `DEFINE PARAM [OVERWRITE|IF NOT EXISTS] $NAME VALUE <expr> [PERMISSIONS …]` declares a database-wide parameter, readable as `$NAME` in queries, function bodies and field assertions.
  - Object values allow keyed constants, such as `$page_size.book`.
- **Obligations discharged:**
  - One declaration of the constant inside the schema.
  - No string splicing into function bodies.
  - The value is visible to schema introspection and export.
- **Residual caller obligations:**
  - Generating the `DEFINE PARAM` statements themselves, which is still text, but in one place.
  - Avoiding name collisions with bound query variables.
  - Changing a value is a schema change.
- **Semantic conditions:**
  - A runtime lookup replaces a literal. Whether the planner treats `LIMIT $param` like a literal for optimisation was not checked.
  - A parameter is a stored value, so a mismatch between the client's constant and the server's parameter becomes possible unless both come from one generator.
  - A bound variable of the same name in a query may shadow the parameter. The precedence was not verified.
- **Version / build conditions:** SurrealDB 1.x–3.x.
- **Limits and negative results:** no compile-time constant folding is documented, and byte-identical installed definitions are preserved only if the generator emits the `DEFINE PARAM` deterministically.
- **Evidence:** Interface-checked: the Context7 `define/param` and schema-best-practices pages.

### surrealql/changefeed-replay
- **Need it serves:** observe what changed in a table since a known point using a bounded query. A reader can catch up incrementally instead of re-scanning predicate state.
- **Recognizable code shapes:** polling loops that re-run a full predicate query on a timer to find new or changed rows.
- **Mechanisms and APIs:**
  - `DEFINE TABLE t … CHANGEFEED <duration> [INCLUDE ORIGINAL]`.
  - `SHOW CHANGES FOR TABLE t SINCE <versionstamp | datetime> [LIMIT n]` returns entries with a `versionstamp` and `changes`.
  - `INCLUDE ORIGINAL` stores reverse diffs.
  - Retention is the declared duration, and the server garbage-collects older entries.
- **Obligations discharged:**
  - A durable, ordered, replayable change log maintained by the store inside the writing transaction.
  - Incremental reads by versionstamp cursor as ordinary bounded queries.
- **Residual caller obligations:**
  - Persisting each reader's cursor.
  - Handling a cursor older than retention by falling back to a full scan.
  - Deciding the wake-up cadence: this is still polling, only cheaper and more precise.
  - The extra write cost on every mutation of the table.
  - Mapping changes to domain events.
- **Semantic conditions:**
  - Changes appear in commit order with versionstamps.
  - Only tables that declared a changefeed are covered.
  - Retention-bounded history can expire.
  - Delivery semantics are pull-based, so nothing is lost while the reader is offline within retention.
  - Behaviour with SurrealMX persistence (AOL and snapshot) was not inspected. The changefeed code is in core (`cf/`, `doc/changefeeds.rs`), so it is not tied to a particular storage backend.
- **Version / build conditions:** SurrealDB 1.x–3.x syntax. Changefeed GC follows server configuration.
- **Limits and negative results:** a changefeed is not a push notification. On its own it does not remove a timer. It replaces "re-evaluate state" with "read the delta".
- **Evidence:** Interface-checked: the Context7 changefeeds, `SHOW` and `DEFINE TABLE` pages, and the location of core 3.3.4 source.

### surrealql/live-select
- **Need it serves:** server-pushed notifications of matching record changes. The SDK view is in `surrealdb-rust-sdk/grpc-live-subscription`.
- **Recognizable code shapes:** timer-driven re-queries waiting for state that another writer produces.
- **Mechanisms and APIs:**
  - `LIVE SELECT [DIFF|VALUE …] FROM t [WHERE …] [FETCH …]` returns a live-query UUID.
  - `KILL <uuid>` ends it.
  - Notifications carry an action (CREATE, UPDATE, DELETE) and data.
- **Obligations discharged:** server-side detection and push of changes while the registering session lives.
- **Residual caller obligations:**
  - A long-lived connection per subscriber.
  - Catch-up after a disconnect, because nothing is replayed.
  - A fallback when no notification arrives.
  - `KILL` on exit.
- **Semantic conditions:**
  - Bound to its session and connection.
  - At most once. Correction (2026-10-27): at most once is what the Rust SDK 3.3.4 delivers, not a fixed property. Protocol 0.13.1 reports delivery, resumability and retention per server (`LiveQueryCapabilities`) and accepts a `resume_from` versionstamp cursor. Reassessment inspected the actual source-built server: it advertises no resumption/retention and rejects a nonempty resume cursor; reconnect requires re-registration and authoritative re-read; see `surrealdb-rust-sdk/grpc-live-subscription`.
  - Documentation for several SDKs states the feature is WebSocket-only. The Rust 3.3.4 gRPC client implements it through `Subscribe`, so support depends on the transport and the SDK.
- **Version / build conditions:** the server must allow live queries in its capabilities and advertise `LIVE_QUERIES` over gRPC.
- **Limits and negative results:** it does not suit request-bounded clients without a separate subscription lifecycle. It cannot serve as an authority signal.
- **Evidence:** Interface-checked: the Context7 pages and the SDK source noted in `surrealdb-rust-sdk.md`.

### surrealql/request-time-ceiling
- **Need it serves:** server-enforced bounds on how long a request or statement runs.
- **Recognizable code shapes:**
  - A bound absolute expiry compared with `time::micros()` inside scripts.
  - `TIMEOUT $duration` clauses.
- **Mechanisms and APIs:**
  - The statement-level `TIMEOUT <duration>` clause on SELECT and mutation statements.
  - The RPC `RequestContext.timeout`, a ceiling per request: "the server MAY fail sooner".
  - The server `Limits.max_query_duration` advertisement.
- **Obligations discharged:** server-side abort of long execution, without client clocks.
- **Residual caller obligations:**
  - Absolute deadline semantics across queuing and transport.
  - Deciding whether to commit when the deadline lands between the last write and commit.
  - Typed refusal text.
- **Semantic conditions:**
  - Timeouts are relative and measured from server receipt.
  - The documentation does not settle whether a timeout can fire during commit, or how a timeout that lands at commit is reported.
  - A script guard `IF time::micros() >= $expiry { THROW … }` checks an absolute clock at chosen points, which gives "refuse before commit" control that a timeout does not promise.
- **Version / build conditions:** SurrealDB 3.x, protocol 0.13.
- **Limits and negative results:** it is not a replacement for explicit deadline checkpoints placed before commit.
- **Evidence:** Interface-checked: the protocol 0.13.1 field documentation and the vendored SDK's per-request timeout derivation. Server commit-time behaviour was not inspected.

## Current applicability

### [CBO-010](../../../library_leverage/codebase/persistence-and-transport.md#cbo-010) — Conflict retry and guarded-transaction preludes assembled per site (SurrealDB side)
- **Types:** `surrealql/stored-function-transaction-guard`, `surrealql/single-statement-implicit-transaction` and `surrealql/request-time-ceiling`.
- **Realization:** installed server 3.3.4. No new dependency.
- **Conclusion:** fits with conditions, and is largely adopted already. The generated catalog defines 186 `DEFINE FUNCTION`s, including `fn::pse_execution_v1::deadline` and `fn::pse_project_v1::authority`, and most guarded sites already call them. What remains textual is the `BEGIN … COMMIT` framing and multi-variable preludes such as `PROTECTED_BEGIN`, whose `LET $pin`/`$grant` bindings later statements read. SurrealQL has no construct that wraps a caller body.
- **What it would discharge / what remains local:**
  - Could move further: inline `IF time::micros() >= $pse_rpc_expires_at { THROW … }` guards, which `fn::pse_execution_v1::deadline` already exists to replace, and the remaining inline `LET … FOR UPDATE; IF … THROW` preludes. They would become functions returning the guarded records as an object.
  - Remains local: the outer framing (a Rust typed builder is the natural owner), bind injection, retry pacing and classification (another agent's scope), and the absolute-clock checkpoints.
- **Prerequisites, configuration or extension:**
  - Converting `PROTECTED_BEGIN` into `LET $guard = fn::…_protected(...)` requires every downstream use of `$pin`/`$grant`/`$result_pin` to read `$guard.*`.
  - Single-call sites could drop the framing, but only after confirming identical conflict typing. A grep of non-test `pse-operations` Rust source found 51 literal scripts of exactly the form `"BEGIN; RETURN fn::…(…); COMMIT;"`.
- **Semantic gaps against the CBO's preservation constraints:**
  - The original clock still bounds attempts client-side; the functions only check it.
  - "Each attempt rebuilds the complete decision" is preserved, because a function re-executes per attempt.
  - Typed `exhaust_conflict` is unchanged.
  - `RequestContext.timeout` cannot replace the absolute pre-commit checkpoint.
- **Unsettled premises and what would settle them:**
  - Whether an implicit single-statement transaction yields the same typed conflict detail as an explicit `COMMIT` under SurrealMX SSI. A small probe would settle it (P2).
  - A non-test re-count found 84 lines containing `BEGIN;`, consistent with the CBO's 86.
    - 61 of those lines already call a `fn::` stored function on the same line.
    - 60 pass `$pse_rpc_expires_at` into a guard function.
    - 51 are pure single-call scripts.
    - The inline-text residue is about 20–25 sites, concentrated in `canonical.rs` (`PROTECTED_BEGIN`, schema install), `canonical_selection.rs`, `canonical_staging.rs` and `canonical_files.rs`. See README.
- **Evidence:** Interface-checked: documentation and repository source counts. Proposed.

**Reviewer correction (2026-10-27).** The `BEGIN;` census misses most of the residue, which is composition rather than guard logic. The protected-read envelope is composed from `PROTECTED_BEGIN`: 29 references, about 20 pin re-check postludes in two variants, 29 four-variable protection bind sets, 65 `num_statements().saturating_sub(2)` result-slot computations and three near-identical project-authority wrappers. Pure single-call scripts number 48 by the reviewer's regex, not 51. The comparative review recommends a typed guarded-script envelope that owns framing, the explicit guard variant, the bind set and the result slot, with bodies left as raw SurrealQL; protected preludes may optionally move into `fn::pse_protection_v1::{open,confirm}` at the next schema change. It keeps explicit `BEGIN`/`COMMIT` everywhere, so probe P2 is unnecessary. See the [comparative review](../../reviews/canonical-store-transport.md#cbo-010).

### [CBO-016](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-016) — Waiting by polling (store side)
- **Types:** `surrealql/changefeed-replay` and `surrealql/live-select`.
- **Realization:** installed server 3.3.4. The current catalog defines no changefeed or live query; a grep of generated `surreal.surql` and the Rust and Python sources found none.
- **Conclusion:** fits with conditions for changefeeds as a cheaper bounded catch-up read. LIVE SELECT is unsettled under the bounded profile (see `surrealdb-rust-sdk.md`). Neither removes the need to poll: a changefeed is pull-based, and LIVE needs a fallback.
- **What it would discharge / what remains local:**
  - A changefeed on the claimable-attempt and study-inventory tables would turn each poll into an incremental `SHOW CHANGES … SINCE $cursor LIMIT n` instead of a predicate re-scan.
  - The timer, lease authority and recovery pass stay.
  - A combined design would use LIVE as the wake hint, a changefeed for catch-up and a slow poll as the backstop.
- **Prerequisites, configuration or extension:**
  - `CHANGEFEED <retention>` on the selected tables, which is a schema change with write amplification.
  - Cursor storage per waiter, which can be in-memory for transient waiters.
  - Behaviour when a cursor falls past retention.
- **Semantic gaps against the CBO's preservation constraints:**
  - "Bounded requests" is satisfied by `SHOW CHANGES`.
  - "Observation grants no release authority" is satisfied, since a changefeed is read-only.
  - "No daemon" is satisfied.
  - LIVE conflicts with bounded requests unless a subscription class is introduced.
- **Unsettled premises and what would settle them:** whether the polling cost is material. The CBO states cost grows with waiters, but nothing is measured. The spawned `observe-primary` interpreter is not a store question.
- **Evidence:** Interface-checked: documentation. Proposed.

**Reviewer correction (2026-10-27).** LIVE is not unsettled: the bounded profile refuses it by design (see `surrealdb-rust-sdk.md`). Architecture §3.3 requires a concrete qualified consumer for live subscriptions, changefeeds still poll, and the cost is unmeasured, so the comparative review retains polling. See the [comparative review](../../reviews/canonical-store-transport.md#cbo-016).

### [CBO-046](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-046) — SurrealQL emitted through string templates (constants)
- **Types:** `surrealql/define-param-constants`.
- **Realization:** installed server 3.3.4.
- **Conclusion:** fits with conditions for the numeric placeholders, which are page rows, cohort sizes, byte caps and scan limits in `surreal_execution.rs`, `surreal_study_workbench.rs`, `surreal_projects.rs` and `surreal_artifacts.rs`. It does not apply to placeholders that splice SurrealQL fragments, such as `__HOLDER_ATTACH__` guards or `FITTED_CELLS_ID`.
- **What it would discharge / what remains local:**
  - Integer substitution into function bodies would become `$PSE_*` parameters declared once, in the same generated catalog.
  - Fragment splicing remains a generator concern.
- **Prerequisites, configuration or extension:** generator support for emitting `DEFINE PARAM`, and the catalog validator's handling of the new statement kind (`canonical_maintenance::parse_definitions` accepts only Table, Field, Index, Event and Function, and refuses any other family as "unsupported generated declaration family"; a `Param` arm would have to be added).
- **Semantic gaps against the CBO's preservation constraints:**
  - Byte-deterministic output is preserved.
  - The values become runtime lookups instead of literals; planner effects are unverified.
- **Unsettled premises and what would settle them:** whether `LIMIT $PARAM` plans identically to `LIMIT <literal>` on the hot paths. Settle with an `EXPLAIN` comparison.
- **Evidence:** Interface-checked. Proposed.

**Reviewer correction (2026-10-27).** Ruled out here. `DEFINE PARAM` would add a second runtime representation of constants that the registry owns, and the premise that the catalog is unchecked is partly false: the generated catalog is already parse-validated with `surrealdb-syn` in a unit test and at maintenance activation. See the [comparative review](../../reviews/canonical-store-transport.md#cbo-046).
