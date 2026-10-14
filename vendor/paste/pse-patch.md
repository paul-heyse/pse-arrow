# Pure macro channel patch

Upstream: `paste 1.0.15`, published crates.io archive
`paste-1.0.15.crate`, SHA-256 `57c0d7b74b563b49d38dae00a0c37d4d6de9b432382b2892f0574ddcae73fd0a`.
Original MIT and Apache-2.0 license texts and upstream source notices are retained.
`Cargo.toml.orig` retains the upstream manifest; `Cargo.toml` selects the same locked
runtime dependencies exactly and excludes unretained upstream examples/dev tooling.

The shared parsed `Segment::Env` branch refuses before environment access; `paste`,
`item` and `expr` all use that branch. Identifier/lifetime/case/doc transformations
remain upstream. The original build script remains a separate qualified compiler
version observation; this patch restricts invocation channels, not build effects.

A source/helper/feature/compiler-context review must qualify this actual owner.
The package name, version or this note alone grants no producer eligibility.
