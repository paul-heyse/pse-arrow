# Canonical server source integration

`../surreal_source.py` owns the exact upstream revision, engine version, build
features, package selection and original compiler provenance. `../surreal_server.py` owns
admission, deployment resources and the selected state. ADR-0214 records the
storage and transport decisions; Plan49a owns execution status.

The installer acquires the exact upstream Git object and applies the patches in
order. It verifies every patched source file and the checksum-pinned DiskANN
source, builds with the repository toolchain in that acquired tree's ordinary
`target/`, with an explicit recorded `panic="unwind"` release override, then records
the executable and lockfile digests. The compiled version
includes the patch identity. Package reuse checks the recipe, original complete receipt,
lockfile and executable bytes. The original compiler/environment context is retained as
provenance, never compared to the caller's worktree or runtime loader/cache settings.
A one-time verified selection adopts existing complete installations in place; receipts and
targets retain their original paths and bytes. Ordinary setup/tests refuse missing or corrupt
packages without compiling. Only explicit `build-server` may build an absent package, and it
also reuses completed packages. Interrupted compilation still requires its exact original
build request. See [installed server reuse](../../docs/dev/surreal-substrate.md#installed-external-server-reuse).
There is no released-binary fallback.

| Input | Reason |
|---|---|
| `mx-1.0-minimal.patch` | Exact engine requirement and the three owned `ByteSlice` to `Vec` return conversions. The minimal stage preserves SI. |
| `mx-1.0-controls.patch` | Explicit SSI, finite memory/age/queue settings, typed nonretrying engine failures, native metrics and revealing adapter tests. Persistence failures preserve unknown commit outcomes. |
| `grpc-connection-ownership.patch` | Authentic accepted TCP identity, connection-bound sessions and bounded cleanup ownership through physical socket loss; applies beneath plain and TLS serving. Cross-connection session reuse refuses. |
| `grpc-session-admission.patch` | Reserves the global session cap atomically across accepted connections and keeps the permit charged until native cleanup completes. |
| `grpc-status-details.patch` | Preserves native structured errors on unary gRPC failures in an independently bounded 8 KiB binary metadata value. Explicit commit conflicts retain typed retry meaning; generic messages never become conflict evidence. |
| `canonical-profile.patch` | Refuses persistent startup unless AOL is synchronous, fsync is every append, and periodic snapshots have a valid interval larger than 30 seconds; invalid configuration cannot silently disable snapshots. |
| `diskann-0.56.0.patch` | One explicit `Arc::as_ref()` at the generic batch call; fixes compilation without changing the vector-index capability or algorithm. The installer checks the original crate archive before applying it. |
| `pinned-compiler.patch` | Removes ignored/redundant manifest fields, inherits declared lints, replaces deprecated module float constants with identical primitive constants, and uses the atomic method to which the deprecated spelling delegates. Moves only `cfg_aliases` to its compatible patch release, whose three expression-position semicolon removals resolve the Nix build warning. |
| `surrealism-locked-build.patch` | Uses locked Cargo build/metadata for modules and requires an already declared `.cargo/config.toml`; module compilation does not write project configuration. |
| `surrealism-build-features.patch` | Carries explicit module feature selections from the CLI into the locked Cargo build. |
| `surrealism-global-controllers.patch` | Adds process-wide fail-fast controller admission whose permit follows the physical WASM Store through idle pooling/cache eviction; propagates a typed capacity refusal. |
| `surrealism-memory-count.patch` | Limits each admitted Store to one linear memory so the controller count times its per-memory allowance bounds aggregate linear memory. |
| `surrealism-controller-deadline.patch` | Preserves the original context/module deadline across queueing, cold instantiation, initialization and invocation; handles cancellation and zero remaining time before allocation/work. Adds focused deadline controls. |

The upstream integration is described by the [MX release migration notes](https://github.com/surrealdb/surrealmx/releases/tag/v1.0.0),
[SurrealDB adapter](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/kvs-mem/src/lib.rs)
and [MX options](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/options.rs).
Compiler compatibility follows the repository-pinned Rust sources:
`core/src/num/f64.rs` defines each deprecated module constant as the corresponding
primitive constant; `core/src/sync/atomic.rs` implements `fetch_update` by calling
`try_update` with unchanged arguments.

Build without touching a catalog:

```sh
scripts/pse-env --resource-class compile -- .venv/bin/python scripts/surreal_server.py build-server
```

Run actual server controls against the executable path in the returned receipt:

```sh
scripts/pse-env --resource-class light -- .venv/bin/python -m scripts.tests.surreal_mx_check --binary /path/from/receipt/surreal
```

Those controls create and drain their own temporary server and data. They exercise
the native gRPC CLI, a tiny global session cap, cross-connection refusal, raw
socket loss without client cleanup, synchronous reopen, actual periodic snapshot
publication, root-authenticated storage metrics with anonymous filtering, pinned
snapshot isolation and reader release after cancellation and socket loss,
transaction rollback and acknowledged-commit recovery after SIGKILL
and an incomplete AOL tail, a malformed framed-log header refusing startup as
corruption, and an 8 MiB catalog limit refusing a transaction without publishing
partial rows. Add `--strace /usr/bin/strace` on Linux to attach the
explicit tracer after authenticated readiness, inject an AOL `fsync` EIO, observe
unknown commit and poisoned-store errors, reconcile the original uncertain
operation after restart, and interrupt snapshot creation before atomic replacement.
The tracer must have permission to attach to the fixture's own child; the tested
tracer is strace 6.8. These are explicit syscall-error and process-kill fault
models. They do not establish power-loss durability.

`fresh-engine --drained` is the explicit destructive design-phase pivot. It accepts
a retired profile only to verify closed admission and actual owned process drain,
discard generated catalog/payload state, and publish fresh credentials and
identities. Interrupted disposal resumes the same durable pending identity. Its
result is closed and requires current schema installation, executable readmission
and explicit activation. Ordinary startup cannot reopen the retired engine.

Initialize either a new setup or the closed result of `fresh-engine`, using the
current already-built initializer artifact:

```sh
scripts/pse-env -- .venv/bin/python scripts/surreal_server.py initialize --state /selected/state --initializer-command /current/build/xtask canonical-init /selected/state
```

`initialize` reuses `derived_rebuild_pending` and the exact live lifecycle nonce;
it creates no parallel admission authority. Setup and fresh-engine mint a durable
ROOT-bootstrap selection bound to the new instance and credential digest. Only
the exact initial initialization or prepared, unsubmitted rollout owner can bind
it into its maintenance launch. A private authenticated ROOT acknowledgment retires
that selection; recovery never infers permission to create accounts from missing files.
This permits an initial toolkit rollout without ordinary readiness or schema creation.
The managed maintenance listener installs VIEWER explicitly, invokes the child with
`PSE_CANONICAL_INITIALIZER`, validates interpretation, and physically drains before
settling to quiesced. An interrupted command resumes the same target and pending
operation. Normal `start` opens admission only after this settlement.
