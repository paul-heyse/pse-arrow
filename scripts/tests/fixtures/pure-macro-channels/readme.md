# Pure macro compiler source fixture

This independent locked Rust workspace retains actual source inputs for the patched Paste,
Pest derive and `py_literal` owners. It contains ordinary positive assertions and six deliberately
uncompilable binaries. The former Python `qualify.py` orchestration is retired; no debug/release
control matrix or diagnostic-matching runner is implied by retaining this fixture.

From the repository root, use the repository-pinned toolchain selected by `rust-toolchain.toml`
and the fixture's own manifest/lockfile. Prepare the native host explicitly before using the
environment adapter. The fixture has no Arrow consumer; plain Cargo is intentional here.

```bash
scripts/pse-env -- cargo test --manifest-path scripts/tests/fixtures/pure-macro-channels/Cargo.toml --locked --lib
scripts/pse-env -- cargo test --manifest-path scripts/tests/fixtures/pure-macro-channels/Cargo.toml --locked --lib --release
```

The library tests assert ordinary token pasting through `paste!`, `item!` and `expr!`, literal
and multiple inline grammar parsing, and `py_literal` nested/complex/byte round trips and malformed
input refusal. They exercise a Rust literal parser, without importing or installing the Python
product. These commands do not automatically run every supplier's own upstream unit tests.

## Explicit compile-refusal cases

The `compile-refusal` feature only makes the negative binaries selectable. Each selected binary
must fail with its owner's guard diagnostic; an unrelated build error is not successful refusal.
For example, the environment value is deliberately present:

```bash
PSE_GUARDED_MACRO_PRESENT=selected scripts/pse-env -- cargo check \
  --manifest-path scripts/tests/fixtures/pure-macro-channels/Cargo.toml \
  --locked --features compile-refusal --bin paste-env-paste
```

This command is expected to return nonzero. Inspect the compiler diagnostic, then select the
other explicit binaries and debug/release modes when those cases are needed:

| Binary | Required owner refusal |
|---|---|
| `paste-env-paste` | `pse tokens-only Paste: environment selectors are unsupported` through a generated `paste!` selector |
| `paste-env-item` | The same guard through generated `item!` |
| `paste-env-expr` | The same guard through generated `expr!` |
| `pest-file` | `pse tokens-only Pest: file grammars are unsupported; use grammar_inline` for a direct file attribute |
| `pest-generated-file` | The same file-grammar guard for a generated attribute |
| `pest-inline-expression` | `pse tokens-only Pest: grammar_inline requires a string literal` for an `include_str!` expression |

`must-not-be-read.pest` is intentionally absent. A file-access panic, missing environment value
or unrelated dependency/compiler failure cannot stand in for these diagnostics. Positive library
tests and negative compiler cases establish their selected behavior only; they do not qualify
all source channels, product deployment or scientific execution. The fixture owns its local
`target/`; never share it with the root target or another checkout.
