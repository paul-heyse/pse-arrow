# Maintained validator generator

Source: the published `protoc-gen-protovalidate-buffa` 0.7.2 package from
https://static.crates.io/crates/protoc-gen-protovalidate-buffa/protoc-gen-protovalidate-buffa-0.7.2.crate.
Archive SHA-256: `c802694e151d562f000d05b04c8ec8d3304c470b15da8f735ac04bdeb8b5392f`.
The original normalized manifest and committed package lock determine the dependency closure.
This is an excluded third-party tool, not a workspace product crate.

Local corrections reuse its CEL parser/schema compiler for explicit preflight; malformed or
unsupported required rules refuse generation. Integer arithmetic and conversions preserve
evaluation errors rather than panicking or wrapping. Logical operators and quantifiers mask
errors according to their decisive boolean results. Generated owned/view validators share the
same lowering and report remaining evaluation errors through the existing typed error owner.

Build the tool explicitly from this directory, using the repository toolchain and its lockfile.
`--preflight DESCRIPTORS --file AUTHORED.proto` (repeat `--file`) checks only selected authored
targets, while retaining their complete import schema. Ordinary plugin stdin runs preflight too.
Success prints exactly `pse-cel-preflight-v1` followed by a newline. Missing/duplicate targets
and duplicate descriptor filenames refuse before rule scanning.

The maintained library entrypoint is `prepare(&CodeGeneratorRequest)`. It owns one
selected scan, its complete import-derived rule visibility, one CEL schema index and
the original `proto_module` output option. It returns an immutable `PreparedValidators`
only after the existing corrected CEL admission succeeds; `PreparedValidators::render`
uses that same scan/index to emit the unchanged owned/view and package/module layout.
`PreparationError::diagnostics` retains structured source/message/field/rule/category
details; the CLI renders them in its existing format. Both ordinary plugin input and
`--preflight` delegate to this entrypoint. Each process prepares its own world: no global
cache or cross-process/cross-world reuse is claimed. Raw `scan`/`emit` helpers remain
test primitives rather than an admitted production-generation route.

This local extraction does not alter the source archive identified above, its original
manifest, or the committed standalone dependency lock. Before using the changed tool,
run its locked tests and explicitly rebuild the prepared installation:

```sh
scripts/pse-env -- cargo test --manifest-path vendor/protoc-gen-protovalidate-buffa/Cargo.toml --locked
scripts/pse-env -- cargo install --path vendor/protoc-gen-protovalidate-buffa --root .local/plan61-tools --locked --force
```

Repository generation remains the caller's separate publication step. Preparation
removes the second scan/schema-index construction inside a plugin invocation; the
external preflight process still prepares independently. This is no selection of an
in-process publisher and no measured performance claim.

This remains a bounded typed lowerer, not a complete CEL checker. Heterogeneous numeric
branches/lists require a dynamic representation to preserve integer precision, so admission
reports Unsupported rather than silently converting integers to doubles. Mixed scalar
integer/double comparisons instead compare exactly, including boundaries, fractions and NaN.
Field CEL on unsupported oneof/WKT representations is refused before emission, including
predefined field rules. Original empty integer-comparison quantifier behavior is retained
while still checking predicate names/types.

Revisit this patch when an upstream release fixes the same rule admission and arithmetic/error
semantics; compare strict error categories and owned/view behavior before replacing it.
