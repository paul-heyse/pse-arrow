# Probe G3-P1: direct protocol client vs vendored SDK, canonical script shapes

This is evidence for the comparative leverage review of
[CBO-009](../../../library_leverage/codebase/persistence-and-transport.md#cbo-009) (G3 SurrealDB,
recommendation B), dated 2026-10-27. The source baseline is `9cb8e4f4a`, with an uncommitted
working tree; the probe reads the vendored SDK and `pse-operations` scripts only.

## Question

Recommendation B for CBO-009 replaces the vendored SurrealDB SDK with a client the repository
owns. That client would be built on the generated `SurrealDbServiceClient<T: GrpcService>` from
`surrealdb-protocol` 0.13.1 (feature `rpc`) and the conversions in `surrealdb-types` 3.3.4
(feature `proto`).

The question: does a minimal client of that kind behave the same way the vendored SDK path
does for the script shapes `pse-operations` uses? The comparison covers:

- the statement count (`num_statements`);
- the decoded results for the three main `take` shapes. In `pse-operations` there are about
  155 uses of `Option<Object>`, 58 of `Vec<Object>` and 51 of `Value`;
- the error kind, `query_details()` and conflict classification;
- error-trailer (`surreal-error-bin`) decoding.

From the code the probe needed, it also estimates what a repository-owned façade would have to
reproduce.

## Versions

| Item | Version |
|---|---|
| Server | Pinned source-built SurrealDB `3.3.4+pse.mx.1.0.0.0b38c33f2ed46d50`. Release receipt `v3.3.4-mx1.0.0-effective-v1-3549c2a97bd565fd2497`, binary sha256 `f555c781…2707a`, matching the receipt. |
| SDK path (a) | Vendored `surrealdb` 3.3.4 via path `vendor/surrealdb`, with features `protocol-grpc` and `rustls` (the workspace declaration), plus `vendor/surrealdb-engine-api`. |
| Direct path (b) | `surrealdb-protocol` =0.13.1 (`rpc`), `surrealdb-types` =3.3.4 (`proto`), `tonic` =0.14.6 (default transport, plain `Channel`) |
| Runtime | tokio 1.53.2, prost 0.14.4, hyper 1.11.1, h2 0.4.19 |
| Lock | The workspace `Cargo.lock` was copied and cargo pruned it. Every one of the 320 resolved third-party packages matches a workspace lock version (checked by script). |
| Toolchain | Workspace `rust-toolchain.toml` (nightly-2026-09-29), copied into the scratch crate |

## Fixture setup

The probe used its own disposable server. It never read `$PSE_SURREAL_STATE`, the managed
primary, or the supervisor's state, admission or systemd units.

- **Launch.** The pinned binary ran directly from `probe/fixture/run.sh` under `scripts/pse-env --`.
  That script mirrors the supervisor's argument set: `--deny-guests --deny-net`, finite
  query/transaction timeouts and `SURREAL_GRPC_MAX_MESSAGE_SIZE=4194304`. It also drops
  inherited `SURREAL_*`/`OTEL_*` variables. The supervisor's own fixture route
  (`scripts/tests/surreal_fixture_check.py`) was not used because it creates systemd units and
  host-admission records.
- **Storage.** `memory` (in-memory MX engine, nothing persisted), bound to a scratch loopback
  port (45529).
- **Credentials.** A random root password in a 0600 scratch file, passed only through the child
  environment. It does not appear in any retained output.
- **Database.** Namespace `probe`, plus a fresh database `probe_<uuid>` for each run.
- **Schema and data.** Schemaless tables, each run defining every table the scripts touch. The
  three functions `PROTECTED_BEGIN` calls (`fn::pse_execution_v1::deadline`,
  `fn::pse_execution_v1::protection_grant` and `fn::pse_files_v1::ticket_open`) are copied
  verbatim from `crates/pse-operations/src/generated/surreal.surql`. One extra function,
  `fn::probe_v1::echo`, serves the single-call shape. The data is one guard, a live and an
  expired protection, one result set, two memberships and a conflict cell.
- **Shutdown.** The server was stopped with SIGTERM by its scratch port and shut down cleanly
  (`outputs/server.log`).

## Paths compared

- **(a) SDK.** The configuration `connect_client` uses in `pse-operations`:
  `Config::new().bounded_requests().grpc(GrpcConfig::new().max_message_size(4 MiB))`.
  - Signin and use carry `RequestContext::control(deadline)`.
  - Queries carry `RequestContext::new(deadline)` with `.bind(...)`.
  - Exception: the interactive-transaction case (4a) uses the plain SDK profile, as the
    repository's `canonical_conflict_tests::held()` does. The bounded profile rejects
    `begin()`, which takes no `RequestContext`: "Bounded gRPC requires an original
    RequestContext", recorded in the outputs.
- **(b) Direct.** `probe/src/direct.rs` uses the generated client over a plain tonic `Channel`.
  The SDK rules that decide observable results are transcribed into it:
  - frame assembly and statement compaction;
  - the `take` rules;
  - `status_to_error` with trailer decoding;
  - `proto_error` / `proto_details` / `kind_str`;
  - the builder's script trim.

  The bounded lifecycle tier is deliberately left out (see Limits).

Parity means `PartialEq` equality of the complete observation, including every `Error` field
(`code`, `message`, `details`, `cause`). Each script's observation includes `num_statements` and
the result of `take::<Shape>(i)` for every index from 0 to n+1. The pse-operations
`transport::checked` + `complete_response` decision is compared separately.

## Commands

```bash
# scratch: /tmp/claude-1000/<session>/scratchpad/probes/surreal-direct-client/{crate,target,server,outputs}
scripts/pse-env -- <scratch>/server/run.sh > <scratch>/server/server.log 2>&1     # background
scripts/pse-env -- bash -c "cd <scratch>/crate && cargo build --target-dir <scratch>/target"
PROBE_DETAIL=<out>/untrimmed-detail.txt \
  scripts/pse-env -- <scratch>/target/debug/surreal-direct-client-probe <port> <passfile>
PROBE_TRIM=1 PROBE_DETAIL=<out>/trimmed-detail.txt \
  scripts/pse-env -- <scratch>/target/debug/surreal-direct-client-probe <port> <passfile>
```

`probe/Cargo.toml` points the SDK path at `vendor/surrealdb` relative to this folder; the
scratch build used the absolute path. The raw outputs are in `probe/outputs/`. The `*-summary`
files hold the parity table and the `*-detail` files hold full `Debug` of both sides. In the
detail files, "(identical)" means the direct observation was byte-equal.

## Outcome

Result: *Measured* on one fixture run per mode, on the 2026-10-27 workstation, against the
in-memory server described above.

- Trimmed façade (`PROBE_TRIM=1`): **37/37 parity**.
- Untrimmed: **36/37 parity**. The one divergence is explained under Divergences.

| # | Case | Observable compared | Result |
|---|---|---|---|
| 1 | Capabilities, attach session, root signin, use ns/db | <ul><li>Signin succeeds with the same token variant (`Access`, no refresh).</li><li>Use succeeds; the server echoes ns/db.</li><li>`RETURN [session::ns(), session::db()]` agrees.</li><li>Capabilities (direct only): limits 4 MiB message / 256 KiB chunk / 90 s / 256 records; `SESSIONS, TRANSACTIONS, LIVE_QUERIES, REFRESH_TOKENS`; encodings `zstd, gzip, identity`.</li></ul> | parity |
| 2a | `PROTECTED_BEGIN` + `SELECT * FROM ONLY …` + `COMMIT` (canonical_results shape) | <ul><li>`num_statements = 11`.</li><li>BEGIN slot 0 = NONE; the consumer SELECT is at index 9 and COMMIT at 10.</li><li>All 3 shapes at every index agree, including `Expected object, got bool` at index 2 for `Option<Object>`.</li><li>`checked` is Ok.</li></ul> | parity (4/4) |
| 2b | `PROTECTED_BEGIN` + `LET $rpc_result = SELECT …` / deadline / IF-THROW / `RETURN $rpc_result` / `COMMIT` (select_names shape) | <ul><li>`num_statements = 14`.</li><li>`Vec<Object>` at 12 gives 2 rows. Non-row indices give `Expected object, got none`, or `got bool` for the `deadline` call.</li></ul> | parity (4/4) |
| 2c | Unguarded `RETURN {…}` (testing::result_payload_counts shape) | 1 statement; Object / `[Object]` / Value agree | parity (4/4) |
| 2d | Multi-statement: rows, empty, scalar, NONE | `Option<Object>` on 2 rows gives `Tried to take only a single result…`; empty gives None; scalar gives `Expected object, got number`; NONE gives None | parity (4/4) |
| 3a | 2a with the expired protection, raising `THROW 'PSE_REFUSAL:immutable_selection_protection_expired'` | <ul><li>Per-index errors are identical: NotExecuted (-32003), the Thrown statement, then Cancelled (-32005).</li><li>`checked` picks `kind=Thrown`, `is_thrown=true` on both paths.</li></ul> | parity (2/2) |
| 3b | 2a with an expired `$pse_rpc_expires_at`, so the THROW is raised inside `fn::…::deadline` | `checked` gives Thrown `PSE_REFUSAL:original_canonical_rpc_deadline_expired` | parity (2/2) |
| 4a | Interactive SSI conflict: two transactions update one cell, the second commit fails (unary failure) | <ul><li>First commit Ok.</li><li>Second gives `Query` / `query_details = TransactionConflict`, with cause `Internal("Transaction conflict: Write conflict…")`.</li><li>Direct trace: gRPC `Aborted` with a `surreal-error-bin` trailer that decodes to kind Query, detail `TransactionConflict`, retryable=true.</li></ul> | parity (2/2) |
| 4b | Conflict inside a single-call `BEGIN; UPDATE; sleep(600ms); UPDATE; COMMIT;`, with a concurrent writer committing at +150 ms (query-frame failure) | <ul><li>Per-statement errors are identical.</li><li>`checked` gives `Query`/`TransactionConflict` "Cannot COMMIT: Transaction conflict…", no cause.</li><li>The substantive error takes precedence over NotExecuted.</li></ul> | parity (2/2) |
| 5 | Single-call `BEGIN; RETURN fn::probe_v1::echo($x); COMMIT;` | <ul><li>`num_statements = 3`: BEGIN NONE, the object at index 1, COMMIT NONE.</li><li>All 3 shapes and `checked` agree.</li></ul> | parity (4/4) |
| 6 | Failing root signin, then correct signin on the same session | <ul><li>`NotAllowed` / `Auth(InvalidAuth)` "There was a problem with authentication" on both paths.</li><li>Direct trace: gRPC `PermissionDenied` with a trailer decoding to NotAllowed/Auth, retryable=false.</li><li>The later correct signin succeeds on both paths.</li></ul> | parity (2/2) |
| 7 | Parse error `BEGIN; SELEC * FROM nothing; COMMIT;` (whole-request failure, unary trailer) | `Validation`; gRPC `InvalidArgument` with trailer, kind Validation | untrimmed: **divergent** (message only); trimmed: parity |
| 8 | `BEGIN; RETURN 1; RETURN 2; CREATE …; COMMIT;`: statements that never emit frames | <ul><li>5 parsed statements give `num_statements = 3`, compacted so COMMIT is at index 2.</li><li>The skipped CREATE had no effect.</li></ul> | parity (3/3) |

In every case where a trailer was present, the server sent the typed `surreal-error-bin`: unary
signin, commit, and the parse failure. The status-code fallback was never reached.

## Divergences, and the rules the façade must pin

1. **Builder script trim (observed divergence).** `method::query::join_queries` removes trailing
   `;` and whitespace from each submitted query before sending it. A client that sends the script
   verbatim gets the same results and errors. Only text that echoes the source differs: the
   parse-error excerpt showed `COMMIT;` rather than `COMMIT`. The trim also changes the
   `query.len()` term in the statement-allocation ceiling. With the trim reproduced, parity was
   37/37.
2. **Statement compaction (reproduced, needed for parity).** Indices that never receive a frame
   are dropped and the rest re-enumerated, which moves the COMMIT slot (case 8). `Kill` results
   are skipped but still use up an index. A façade that indexes by `query_index` would move every
   statement after a short-circuit and break callers that rely on "the consumer SELECT is
   immediately before COMMIT".
3. **`take` conversion rules (reproduced, needed for parity).** They live in `opt/query.rs`:
   - `Option<T>`:
     - `[]` gives None, `[x]` gives `x`, and `[NONE]` gives None;
     - more than one element gives the `Tried to take only a single result…` Internal error;
     - a non-array is converted directly.
   - `Vec<T>`: a non-array is wrapped.
   - `Value`: a missing index gives `Value::None`.
   - Conversion failures map to `Error::internal(e.to_string())`, and the exact `SurrealValue`
     messages ("Expected object, got bool") come from `surrealdb-types`.
   - An error slot is returned and removed by any `take`.
4. **Error rebuild (reproduced, needed for parity).** The pieces are:
   - `proto_error` → `Error::from_wire(message, kind_str, details)` plus the cause chain;
   - `proto_details` rebuilding `{kind, details}`;
   - the trailer-first `status_to_error` and its code fallback.

   `Error::from_wire` is `#[doc(hidden)] pub` in `surrealdb-types` 3.3.4. It is the only route
   to the kind-specific codes (-32003 NotExecuted, -32005 Cancelled) and typed details. A
   façade therefore depends on a hidden API, which is the gap the review's U1 item "public
   `proto::SurrealError → Error`" names.
5. **`checked` ordering (behaviour note, not a divergence).** `IndexedResults::take_errors`
   returns a `HashMap`, so when more than one *substantive* error is present, the error
   `transport::checked` picks is chosen in hash order. In all probed scripts there was exactly
   one substantive error, so this never showed. A façade can make the choice deterministic, for
   example the lowest index. That is a strict improvement, not a regression.
6. **Bounded profile and interactive transactions.** `begin()`/`commit()` are refused under
   `bounded_requests()`. Only the test helper `held()` uses them, on the plain profile. A façade
   either keeps that test path on a separate plain client or adds request-context-carrying
   transaction RPCs. The probe did the latter through the direct client, with identical conflict
   classification.

## What a repository-owned façade must reproduce

This is an estimate from the probe code, which is *Interface-checked* and *Measured* for the
listed shapes only.

`probe/src/direct.rs` is 527 lines including comments. About 350 of them are near-verbatim
transcriptions of SDK rules:

| Area | Lines | Content |
|---|---|---|
| `take` rules | ~70 | Option/Vec/Value rules, `take_errors`, `num_statements` |
| Query frame assembly | ~180 | Begin/Batch/End/Error frames, batch-sequence checks, repeated-Begin and index-bounds refusal, End-count and terminal-trailer checks, compaction, script trim |
| Unary RPCs | ~135 | Connect + `GetCapabilities` + `AttachSession` + signin (`UserCredentials`) + `Use` + begin/commit |
| Error mapping | ~95 | Trailer decode with an 8 KiB envelope, code fallback, `proto_error`/`proto_details`/`kind_str` |

The review's estimate of about 1k lines of client semantics is consistent with this. The ~0.5k
probed here cover the *result and error semantics*. The remainder is the bounded lifecycle tier,
which recommendation B moves rather than rewrites:

- admission;
- the owned HTTP/2 driver and its join;
- the wire preflight with `ResponseLimits` before prost;
- deadline-derived `RequestContext.timeout`;
- payload ownership and the statistics observer;
- acknowledged-setup restoration;
- trace injection;
- disconnect and drain.

Further surface used in production that a façade needs but that was not probed:

- `select((table, key))`. The SDK lowers it to `SELECT * FROM $what` and then applies the
  Option/Vec rules.
- `IndexedResults::check()`.
- Scalar `take` shapes (`Option<bool>`, `Vec<String>`, `Option<i64>`, about 20 uses). They go
  through the same generic rule with a different `T`.
- `not_found_details()` classification in `select_existing`, which is a `surrealdb-types`
  accessor and unchanged.
- `disconnect()`.

## Limits

- This is one run per mode on one in-memory fixture with schemaless tables, not the production
  schema, the MX `aol=sync` persistent store, or the supervised profile. Only the conflict case
  (4b) is timing-based (a 600 ms window against a +150 ms writer), and it gave parity in both
  runs.
- The direct path uses a plain tonic `Channel`. The bounded owned-HTTP/2 transport and the wire
  preflight are not exercised. The direct path's `RequestContext.timeout` is a fixed 30 s,
  whereas the SDK derives it from the original deadline. Both paths are therefore *Proposed* to
  be equal with respect to frame semantics only.
- The guarded read's nested result-pin branch (`$result_pin != NONE`) was not driven. Live
  queries, streaming (`query_stream`), import/export and `run` were out of scope.
- `Debug`-level equality covers every `Error` field and value variant. It does not establish
  equivalence for value kinds absent from the fixture, such as datetime, uuid, geometry or
  files.
