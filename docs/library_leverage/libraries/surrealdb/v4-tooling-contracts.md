# SurrealKit rc.1: selected tooling contracts

Researched 2026-11-09 against `v1.0.0-rc.1`; **Interface-checked** from the
primary references below. These are reusable technical observations, independent
of any repository deployment. No local execution or performance probe ran.

## `surrealkit/engine-semantic-compilation`

`compile::compile(&[CompileModule], &CompileOpts)` applies selected definitions
to a disposable embedded engine and introspects its normalized schema. It follows
Sync preparation, including special sequence handling, rather than arbitrary raw
SQL execution. Its interpretation is the embedded engine's version; an expected
engine mismatch is diagnostic, not emulation of another release. Compiler template
variables exclude ambient `SURREALKIT_VAR_*`; network and scripting are restricted.
Embedded backend features must actually be selected by the build.

[Tagged compiler](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/compile.rs)
and [feature declarations](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/Cargo.toml)
establish the interface and implementation conditions, not general equivalence to
every patched standalone server.

## `surrealkit/semantic-document-and-emission`

Schema document v2 is a draft, version-qualified interchange contract. Structured
kinds and operation-sensitive field presence support generation beyond one read
record shape. Its digest covers a semantic projection, excluding provenance and
diagnostics; it is not a database-state, history or execution-identity digest.
Unknown STRUCTURE keys and unsupported engines produce diagnostics. A lenient
fallback is not evidence of complete representation. Named types and extension
metadata require particular coverage scrutiny.

[Normative v2 document](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/schema-document-v2.md)
owns the exact presence matrix, digest projection and adapter rules.

External emitter protocol 1 consumes v2 on stdin and returns files in one JSON
response. SurrealKit owns publication and its output manifest, stale-file removal,
unchanged-byte handling and `--check`. Relative output paths and disjoint ownership
are enforced. Replacement uses individual files; do not infer one atomic directory
transaction. A Rust emitter remains an additional implementation, not a bundled
capability.

[Emitter protocol](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/docs/typegen/external-emitters.md)
defines publication and diagnostic handling.

## `surrealkit/native-migration-coordination`

The rc.1 migration adapter recognizes named v4 field transformations and undeclared
field cleanup. It separates migration tracking from ordinary definition identity,
strips an already-executed clause on subsequent definition updates, and removes
then restores indexes that block a new migration. This creates a real uniqueness
gap when a removed index enforced uniqueness. Completion of statement submission,
background record conformance and application cutover are distinct events.

[Migration adapter](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/migrate.rs)
and [rc.1 migration notes](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/MIGRATING.md)
are the release-specific sources. Reverse transformation and `await_migrations`
are useful coordination facilities; neither proves lossless application rollback.

## `surrealkit/test-report-and-settlement`

The tester can compose declarative suites with an external executable and report
the latter as a test case. The tagged runner installs its Ctrl-C watcher for
`--exec`, not universally for every invocation. Parallel fail-fast paths abort
suite tasks. These observations require specific cleanup/child-settlement checks
before assigning stronger physical-disposal guarantees to the harness.

[Runner source](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/tester/runner.rs)
is implementation evidence, not proof against arbitrary process termination.

## `surrealkit/bounded-progress`

`ProgressSink`, task-local `capture` and `propagate` carry bounded progress records
separately from structured operation outcomes. The default sink limits are 4096
lines or 512 KiB; overflow is counted. A spawned task needs explicit propagation.
Process-wide logger installation can fail when another logger is already installed.
An empty capture therefore does not prove an absence of work or failure.

[Progress implementation](https://github.com/surrealdb/surrealkit/blob/v1.0.0-rc.1/crates/surrealkit/src/progress.rs)
owns the precise limits and failure behavior.
