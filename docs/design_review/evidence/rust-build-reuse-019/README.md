# Rust build reuse: sccache 0.19 source dispositions

Research date: 2026-10-24. Supports [Plan 54c](../../../plans/54c-cross-location-reuse-and-investigations.md);
the plan owns decisions, implementation status and runtime acceptance. The initial sections record
**Interface-checked** capabilities and source reasoning. Runtime controls below have their
own scope; no remote backend or distributed service was deployed.

Context7 was consulted first for sccache; exact release sources below settle consequential claims.
Two name-specific Context7 resolutions each for Rust guppy/hakari and cargo-shear found unrelated
libraries, so their exact upstream sources were inspected instead. Missing search coverage does
not establish missing capabilities. The supplied library overview predates the newly published
sccache release and overstates released Rust basedirs support.

## Released compiler behavior and relocation

[sccache 0.19.0](https://github.com/mozilla/sccache/releases/tag/v0.19.0), published
2026-10-24, adds locally written Rust dep-info, `.rmeta` artifact notifications including
client-side misses, concurrent output discovery, crate-type non-cacheable statistics and
effective cache-mode reporting. These offer correctness, Cargo pipelining and attribution value;
the release notes establish mechanisms, not a speedup on this workspace.

The [v0.19.0 Rust implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/compiler/rust.rs)
still hashes dep-info environment values, relevant `CARGO_*` values and cwd literally. It has no
basedirs normalization in the Rust hasher. Locally generated dep-info avoids foreign dependency
paths on restoration; it does not remove location-sensitive `OUT_DIR` keys. Therefore the
released candidate is credible for its independent fixes, but is not a demonstrated F01 remedy.

The two normalization candidates remain open and unmerged on the research date:

| Candidate | Exact source examined | Disposition |
|---|---|---|
| [PR2794](https://github.com/mozilla/sccache/pull/2794) | [91dd36202fdb60f1feac5e550500f4abb76a8e09](https://github.com/pamarcos/sccache/blob/91dd36202fdb60f1feac5e550500f4abb76a8e09/src/compiler/rust.rs) | Preserves literal dep-info environment values; disables normalization when any explicit extern exists or a dynamic search entry may load a proc macro. Effective remapping and basedirs therefore cover only a restricted subset, and cannot remove F01's literal `OUT_DIR` distinction. |
| [PR2678](https://github.com/mozilla/sccache/pull/2678) | [3845d98137f86a940c7d1bb830ef7dc17d8d0814](https://github.com/vercel/sccache/blob/3845d98137f86a940c7d1bb830ef7dc17d8d0814/src/compiler/rust.rs) | Strips basedirs from dep-info environment values, including `OUT_DIR`, without proving those values were not embedded literally. Reject this equivalence rule for ordinary use: compiler remapping does not rewrite arbitrary strings produced by `env!` or `option_env!`. |

Source inspection is sufficient to reject a candidate that equates observably different literal
outputs; compiling every fork is unnecessary. Conversely, source plausibility alone does not
qualify a candidate. Keep the actual compiler controls: same-path artifact loss, equal relative
targets in different snapshots, differently named target roots, generated `include!`, literal
environment paths, `file!`/`Location`, generated-content changes, changed extern contents and
local dep-info freshness. Compare restored behavior with direct compilation, alongside hit/miss
attribution. The recording shim does not reproduce source, extern and dep-info input hashing.

Basedirs is a daemon-lifetime policy even with client compilation:
[server StorageHandshake](https://github.com/mozilla/sccache/blob/v0.19.0/src/server.rs)
exports storage basedirs and [IpcStorage](https://github.com/mozilla/sccache/blob/v0.19.0/src/cache/ipc_storage.rs)
reads the handshake. A future mechanism must include supported roots, new worktree behavior,
service generation, producer input admission and caller agreement. Do not restart the shared
daemon for each worktree or share a mutable Cargo target among checkouts.

**Proposed revisit trigger:** a released or explicitly maintained candidate that preserves
literal environment semantics, handles the actual heavy dependency/extern cases, produces
correct local dep-info, and passes the above controls under its own flags and cache population.
A release containing either PR number alone is insufficient. Until then, correct misses remain
the supported boundary. A deliberately stable campaign-owned source/target location may retain
local artifacts where source isolation and lifecycle remain valid; it is a different workflow,
not cross-checkout relocation.

## Wider capabilities on the current single-cache host

### Storage tiers and remote caching

[v0.19.0 MultiLevel documentation](https://github.com/mozilla/sccache/blob/v0.19.0/docs/MultiLevel.md)
and [implementation](https://github.com/mozilla/sccache/blob/v0.19.0/src/cache/multilevel.rs)
support ordered reads, promotion to earlier writable levels, parallel writes, read-only tiers
and `ignore`/`l0`/`all` write policies. The default `l0` requires the first level at startup;
optional slower-level startup failures can be tolerated and retried subsequently. `all` requires
every configured level. Read-only levels are skipped during writes. The implementation's policy
branches are more precise than the document's introductory statement that any successful level
makes writes succeed. Version 0.19 specifically repairs client-side raw-hit promotion and
per-level statistics. Remote read failures are reported as cache errors instead of ordinary misses.

Potential value exists even on one machine: retaining cold history beyond local disk capacity,
recovering after local cache loss, or reusing artifacts from another trusted producer. Added
cost includes transfer latency, storage charges, authentication, availability and backend
operation. Redis on the same machine adds a service and memory pressure without inherently
providing durable recovery. A slower durable object tier has a more distinct role if local
eviction or recovery needs are material. Remote caching cannot repair incompatible Rust keys.

Retain the current managed disk cache for now; no deployed remote endpoint, credentials or
second-tier owner is selected by this investigation. Revisit if C03 identifies expensive repeated
evictions, a concrete recovery objective, or a compatible second producer. Then compare complete
workflow costs and unavailable-tier/read-only/promotion behavior through an isolated endpoint.
Check the selected executable's compiled backend support, not only Cargo feature defaults:
[v0.19.0 manifest](https://github.com/mozilla/sccache/blob/v0.19.0/Cargo.toml) and
[configuration](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Configuration.md).
Production adoption also needs deliberate secret delivery and service-generation inputs; the
existing daemon environment intentionally does not inherit arbitrary credentials or cache knobs.

### Distributed miss execution

[v0.19.0 architecture](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Architecture.md)
and [distributed quickstart](https://github.com/mozilla/sccache/blob/v0.19.0/docs/DistributedQuickstart.md)
describe separate scheduler/build-server processes, authentication, toolchain packaging and
sandboxed execution. This can shorten expensive eligible misses when a compatible remote host
has spare capacity; it is distinct from remote artifact storage. Native solver dependencies,
toolchain identity, trusted outputs and resource ownership add integration work. A configured
scheduler disables the current client-side mode, conflicting with caller-owned compilation and
its supervision contract. Existing local admission cannot automatically supervise remote work.

Retain local execution. Revisit with a named server/operator and a demonstrated miss critical
path worth transferring; require a separate supervision/trust/native-environment design before
deployment. This is a contract constraint and operational tradeoff, not an absence-of-caller proof.

### Reusable graph analysis and shear embedding

At the exact [cargo-hakari 0.9.39 source tag](https://github.com/guppy-rs/guppy/tree/cargo-hakari-0.9.39),
[guppy CargoOptions/CargoSet](https://github.com/guppy-rs/guppy/blob/cargo-hakari-0.9.39/guppy/src/graph/cargo/cargo_api.rs)
simulate selected features with resolver 3, platform and dev-dependency controls and expose
separate host/target feature sets and links. [HakariBuilder](https://github.com/guppy-rs/guppy/blob/cargo-hakari-0.9.39/tools/hakari/src/hakari.rs)
computes unification proposals with platform, exclusion and host/target strategies; these are
library capabilities independent of deploying a workspace-hack crate.

Potential consumers include repeated explanation of graph changes and counterfactual host/target
unification. Embedding adds dependencies, model integration and maintenance. Simulation is not
an exact replacement for the pinned nightly's selected unit graph or compiler timings. Retain
current Cargo outputs; revisit if recurring analysis needs reachability/feature queries that are
awkward or unreliable over those outputs. Compare against actual units, especially workspace-wide
unification and validation/native opt-ins. Deploying hakari additionally changes feature edges,
generated manifests and publishing behavior; analytical use alone does not justify those changes.

[cargo-shear 1.14.0 public source](https://github.com/Boshen/cargo-shear/blob/v1.14.0/src/lib.rs)
exports `CargoShear<W>`, `CargoShearOptions`, `OutputFormat` and option builders, including locked
metadata, package selection and JSON rendering. `run()` returns an exit code and writes the
report; analysis/diagnostic internals remain private. Embedding can remove process-launch
overhead and configure repeated analysis directly, but does not expose a public typed diagnostic
graph or make static analysis equivalent to compiler name resolution. CLI/JSON remains an adequate
boundary for current automation. Revisit embedding for a real long-lived Rust consumer needing
in-process execution, after comparing startup costs and the library's actual public interfaces.
Neither embedding route creates a demonstrated build-time benefit by itself.

## Runtime relocation controls

**Tested:** actual 0.19.0, pinned rustc, a private foreground server/cache and linked
behavioral consumers: 16 cases passed in `/tmp/pse-c01-019-se_dd5mn/receipt.json`
(`scripts/pse-env -- .venv/bin/python /tmp/pse-c01-runtime-controls.py`). Generated includes
hit on repetition and same-path artifact loss; a different target under the same cwd hit.
Changed generated bytes, literal OUT_DIR/CARGO_MANIFEST_DIR values, absolute file!/Location
paths and changed extern bytes produced current behavior and required misses. A real Cargo
fixture restored dep-info for the newly selected target; subsequently changed included bytes
recompiled and returned the changed value. Cross-cwd relocation remained a correct miss.
These are bounded behavioral controls, not complete native/Python qualification.

The corrected workspace reference campaign used separate snapshot-local default targets.
Its second snapshot recorded 167 Rust hits and 42 Rust misses. ZIP inventory identifies
42 duplicate artifact names, including serde, gmp-mpfr-sys, faer and numerica. The first two
have different literal OUT_DIR env-deps in their retained `.d` files; downstream faer/numerica
artifacts also differ. Exact source hashes extern contents, so relocation can propagate
through those dependencies. This does not independently attribute every individual miss.
Aggregate statistics also omit oversized publication failures, as the next control shows.

## Large client-side publication limit

**Tested:** a private deterministic 12 MiB include_bytes! fixture and linked full-byte
checksum consumer reproduced silent publication loss with the default server frame bound.
All three compiles returned correct outputs, but no cache entry was written; the daemon
logged three `frame size too big` errors while aggregated executions, writes and errors
remained zero. With a 128 MiB daemon frame bound, the first compile wrote a 37,765,036-byte
entry and the next two hit. Restarting private servers against that populated cache
produced three hits each with default and raised bounds. All 12 output oracles passed;
all owned daemons stopped. Receipt: `/tmp/pse-large-entry-probe/client-v4/results.json`;
command: `scripts/pse-env -- .venv/bin/python /tmp/pse-large-entry-probe/probe.py`.

**Interface-checked:** 0.19's IpcStorage sends the complete compressed entry in StoragePutRaw.
The server uses tokio-util's default 8 MiB length-delimited frame bound unless
SCCACHE_MAX_FRAME_LENGTH is set. A failed oversized request breaks the connection;
best-effort RecordStats on that connection can then fail too. Consequently zero reported
cache errors is insufficient proof of complete eligible-call accounting. The local-disk
read path can return an entry's filesystem path, explaining why populated large entries
remain readable with the default bound. See exact
[IPC storage](https://github.com/mozilla/sccache/blob/v0.19.0/src/cache/ipc_storage.rs),
[server framing](https://github.com/mozilla/sccache/blob/v0.19.0/src/server.rs), and
[client statistics](https://github.com/mozilla/sccache/blob/v0.19.0/src/commands.rs).

The original reference's absent Symbolica cache entry and near-8-MiB largest entries
therefore require caution: its recovery is not an all-eligible-hit result, despite zero
reported misses/errors. Final joined measurements must use the admitted larger frame
configuration and inspect publication as well as aggregate statistics.

**Tested, integrated managed route:** the actual scripts/sccache wrapper admitted a daemon
with frame 268435456 in both its descriptor and `/proc` environment, and systemd MemoryMax
8589934592. It published a 37,764,932-byte entry, then hit; four linked full-payload oracles
passed with identical output hashes. An explicit 128 MiB override selected a distinct
service generation and reused the populated cache. Both owned services and allocations
were released. Receipt: `/tmp/pse-large-managed-6gsvw_mq/receipt.json`; command:
`scripts/pse-env -- .venv/bin/python /tmp/pse-large-entry-probe/managed-acceptance.py`.
The permanent `just build-cache-probe` now removes first-run artifacts, requires restored
library/dep-info and an actual entry above 8 MiB, then executes a restored-payload checksum
consumer. The selected 256 MiB control passed with a 12,591,274-byte entry.
