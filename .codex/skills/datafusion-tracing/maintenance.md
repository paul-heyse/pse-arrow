# Maintaining and transferring the reference

The reader is self-contained. Python 3.11+ and local files support `scripts/reference.py`;
`rg` and `ast-grep` are optional reader tools. Acquisition, regeneration and behavioral research
have separate prerequisites. Nothing installs the skill into user configuration.

## Change a contract or claim

Author semantic records under `authoring/capabilities/`; keep source-backed claims separate from
runtime observations. A runtime evidence record names exact test IDs in the retained receipt.
Changes to fixture/lock/runner bytes invalidate that receipt until the relevant tests run again.
The generator rejects runtime claims without successful named assertions.

`build/contracts.py` preserves upstream documentation and type structure; `build/model.py`
retains the compact inventory and full contract collection. Reachability is still classified by
`build/visibility.py`. Update the authoring or generator, then regenerate generated content:

```bash
python3 build/build.py
python3 build/test_contracts.py
python3 build/verify.py
python3 scripts/invalidation.py
```

The preservation check compares every operation with its actual raw capture, resolves local
contract links, checks capture/identity distinctions, tests aliases and exercises UTF-8 bounded
continuation. The verifier separates digest integrity from fresh regeneration. A result from
`--skip-rebuild` checks stored bytes only. Runtime probes are not implicit build dependencies.

`content/capabilities/dependencies.json` records evidence dependencies and neighboring candidate
surfaces. `scripts/invalidation.py` reports which briefs need review when either changes. New
crate surface can change an old comparison even when its previous selected API is unchanged.
This is a review aid, not automatic proof that unlinked effects are absent.

## Reproduce consumer observations

Probe fixtures and Cargo.lock live in `build/fixtures/probe-crate`. The manifest records the
stable compiler identity. Execution copies those fixtures into an external capsule; no consumer
repository is a build working directory. The capsule argument must be outside working repos.

```bash
python3 scripts/run_probes.py --capsule /path/to/external/capsule
```

The runner builds with `--locked`, records compiler and resolved features, runs the named public
consumer tests, and retains raw captures and logs. `--target-cache` can reuse an external Cargo
cache. Missing tools are `blocked`; compilation or assertion failures are `failed`.

The new SDK fixture uses the recorded OpenTelemetry 0.31 / bridge 0.32 profile. The in-memory
exporter's default shutdown clears its records: retrieve completed spans after flush and before
shutdown when inspecting those records. These observations do not certify an OTLP receiver.

The legacy small-query suite can be rerun separately with `build/probes.py`, setting
`DATAFUSION_TRACING_CAPSULE` to an external capsule. Its verdict vocabulary and scope remain
separate. Rebuild afterward to refresh projections of those observations.

## Repin and evidence acquisition

`build/acquire.py` is the networked acquisition stage. The manifest names exact package pins,
corpus commits, and a dated nightly for private rustdoc. Private capture bytes are retained
because hosted rustdoc cannot reproduce them. Keep the source and producer identity for each
capture rather than joining document-local IDs across artifacts. The format adapters accept
only the recorded supported formats and checked shapes.

Verify consequential changed APIs/features against exact upstream releases. Current documentation
is a discovery lead. Use the companion DataFusion skill for DataFusion/Arrow boundary
research when available, and retain the exact supporting evidence here so the bundle stays
independent. Retain source/archive hashes, licenses and notices. Repinning
can invalidate grammar, reachability, runtime assumptions and candidate comparisons independently.

## Portable bundles

```bash
python3 scripts/package.py reader --output /path/to/reader.tar.gz
python3 scripts/package.py research --output /path/to/research.tar.gz
python3 /path/to/unpacked/bundle/scripts/verify_bundle.py
python3 scripts/qualify_transfer.py --archive /path/to/reader.tar.gz --destination /path/to/fresh-copy
```

The reader includes full contracts, reviewed briefs, source/capture evidence and the standalone
reader. Research adds build machinery, fixture source/locks and evaluation receipts. Compiler
targets, hidden build caches, recursively nested bundles and source-checkout dependencies are
excluded. Both archives carry per-file SHA-256 manifests. `qualify_transfer.py` requires `strace`
for filesystem/network observation and uses an empty environment plus isolated Python readers.
Those subprocesses deny file access outside the bundle/Python runtime and deny network access.

Offline reading and offline compilation are different promises: the latter requires the named
compiler and dependency archives/cache. Test transfer from an unrelated directory with the
original skill and hidden source caches unavailable. Record real commands, observed file/network
access and limitations, rather than relying only on a static path scan.
