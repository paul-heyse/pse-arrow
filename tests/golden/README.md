# Golden stores

These stores exercise the Wave 1 boundaries in blueprint §3.2 and §24.1. The
`registry` fixture commits the complete registry and explicit empty primitive tables.
`minimal_explicit` commits the original package files under
`tests/fixtures/packages/minimal_explicit`, including a template equation and an
actual case activation, then executes P3 through the ordinary compiler driver.

Each fixture contains actual `manifests/`, `relations/`, `documents/`, and reference
objects produced by the catalog's local storage protocol. `store-index.json` records
the explicit parent roles needed to reopen the manifests. `values.json` records every
decoded relation cell, complete membership, original source bytes, selected SQL
results, and separately the resulting identities. Semantic admission and actual value
comparisons establish correctness; matching hashes alone do not admit a fixture.

`minimal_explicit/p10-fixture` is the distinct complete P10 boundary authorized by
ADR-0056. Its physical Model is loaded through the real authoring loader. The declared
`FixtureP10Inputs@1` importer publishes a complete derived predecessor bundle, and
the driver runs P10 against those pinned parents. The compiled integer result is 3
for the declared expression `1 + 2`. This fixture does not claim P4–P9 execution.

Regenerate a fixture with `cargo xtask golden registry` or
`cargo xtask golden minimal_explicit`. Add `--check` to reopen the stored fixture,
publish a fresh execution, and compare their complete admitted values, sources,
queries, and identities. A one-byte P3 workspace control uses the actual reopened
Model rows and must fail before decoding. The Python golden tests read the registry's
actual IPC files through generated contracts; Python does not write these stores.
