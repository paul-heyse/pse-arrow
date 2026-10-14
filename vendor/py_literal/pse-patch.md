# Pure macro channel patch

Upstream: `py_literal 0.4.0`, published crates.io archive
`py_literal-0.4.0.crate`, SHA-256 `102df7a3d46db9d3891f178dcc826dc270a6746277a9ae6436f8d29fd490a8e1`.
Original MIT and Apache-2.0 license texts and upstream source notices are retained.
`Cargo.toml.orig` retains the upstream manifest; `Cargo.toml` selects the same locked
runtime dependencies exactly and excludes unretained upstream examples/dev tooling.

The exact published grammar bytes (SHA-256
`5ce3a6b819b2ced5448442248f9fdde5bdf7023709b9a7a1ceeb7b4443a3c0a3`) are now the sole literal inline
grammar in `src/parse.rs`. The original grammar file and include constant are
removed. Runtime literal parsing/formatting remains upstream, with no duplicated
writable grammar and no ambient grammar-file selection.

The debug assertion macro's temporary is renamed from `value` to
`asserted_value`: on the pinned compiler the original name collides with the
generated `Rule::value` variant when this package is checked as a local source.
Only that local binding changes; debug assertions and grammar behavior remain.

The documentation example imports the already required `num_bigint` and
`num_complex` crates directly, so the otherwise unused upstream `num` dev
dependency is omitted. This allows the exact library unit controls to run in
the isolated compiler-control dependency graph without another workspace.

The existing empty-set formatting panic test explicitly discards its expected
return value to satisfy the pinned compiler's must-use warning; its expected
panic and production formatting behavior are unchanged.
