# Pure macro channel patch

Upstream: `pest_derive 2.9.2`, published crates.io archive
`pest_derive-2.9.2.crate`, SHA-256 `284b60557f2c4a2e72ad3f2d34d42685a2fa4a6a61d0d2a10c0ae2a5e916c2cf`.
Original MIT and Apache-2.0 license texts and upstream source notices are retained.
`Cargo.toml.orig` retains the upstream manifest; `Cargo.toml` selects the same locked
runtime dependencies exactly and excludes unretained upstream examples/dev tooling.

The actual derive input is parsed with the locked Syn helper. Every `grammar`
attribute is refused, and every accepted `grammar_inline` attribute must contain
a literal string. At least one inline grammar is required. The unchanged public
`pest_generator::derive_parser(input, false)` owns grammar parsing/validation and
code generation; disabling inclusion also avoids its unconditional current-dir
observation. Multiple inline grammars retain upstream concatenation semantics.

Qualification binds actual owner and complete selected helper/feature/compiler
context. Unknown or changed helper channels do not inherit a tokens-only grant.
