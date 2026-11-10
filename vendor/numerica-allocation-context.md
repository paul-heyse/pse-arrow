# Lexical exact-integer allocation policy

The published baseline, complete delta and byte/mode receipt are recorded in
[the supplier reconstruction owner](deltas/readme.md). The workspace selects that
same maintained source for every Numerica dependency; Symbolica's normalized
manifest also selects its sibling explicitly.

`IntegerAllocationContext` carries only a fresh/cached policy value. `capture()`
records the current thread policy; `fresh()` requests fresh backing; `run()`
installs it for one call and restores the prior policy on return or unwind.
Explicit workers apply the captured value on their own threads. No caller,
admission, cancellation or deadline is stored in an integer or context.

Fresh GMP calls bypass both cleared-cache adoption and insertion. Native Clone,
primitive conversion, borrowed arithmetic and division/remainder therefore cannot
adopt an old oversized cleared allocation. Existing cached integers remain
untouched during the fresh call. Returned integers retain no policy: their later
drop follows the then-current policy and can populate the ordinary cache.
This is a requested-backing property, not allocator usable size or RSS control.
Mathematical values, arithmetic and default caching outside fresh calls are unchanged.
Symbolica workspace storage, coefficient growth and producer safe points need their
own construction controls; this capability alone does not establish those bounds.

The focused standalone control uses the packaged lockfile and selected backend:

```sh
scripts/pse-env -- cargo test --manifest-path vendor/numerica/Cargo.toml \
  --locked --no-default-features --features integer-gmp,float-mpfr \
  --lib integer_allocation_context
```

For standalone Symbolica commands from the repository root, also supply the explicit
source selector below. It directs optional transitive users to the same maintained
Numerica source. The workspace already carries this selection in its patch table.
The standalone Symbolica lockfile retains the previous dependency versions.

```sh
scripts/pse-env -- cargo test --manifest-path vendor/symbolica/Cargo.toml \
  --config 'patch.crates-io.numerica.path="vendor/numerica"' \
  --locked --no-default-features \
  --features integer-gmp,float-mpfr,native_code_generation \
  --lib evaluate::control::tests::
```
