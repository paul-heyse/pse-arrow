# Opt-in allocation-accounted view generation

The consumed profile revision is owned by the companion runtime's `pse-patch.md`.

Source: published `buffa-codegen` 0.9.2 archive, SHA-256
`bb4bea10ef85ea3d06a20412c67cb612527feb25f33367eab850139441fb5dd9`.
Retained VCS metadata identifies `1f5d80e2c8d31c147742d8ec826a3219c4fcd4a7`.
The published manifest, original manifest and Apache license are retained.
Compare against `https://static.crates.io/crates/buffa-codegen/buffa-codegen-0.9.2.crate`.

`CodeGenConfig::allocation_accounted_views` defaults to false. Opting in emits
pre-growth eager/lazy view decoder calls to the maintained runtime: singular and
oneof boxes, repeated/map stores, packed scalar extenders and lazy fragments/raw
elements. Shared runtime unknown-field accounting applies when a context carries
the budget. Generated owned messages remain stock. External/WKT message decode
routes use the actual child decoder's default-false capability under an attached
budget. This also covers ordinary imports generated separately with stock settings.
Only opted-in decoders declare the capability; accounted external children are
supported. A lazy parent's raw-fragment scan makes no child decode claim. Explicit
traversal checks each deferred decoder's capability before calling it.

The budget and its exclusions are described in the companion runtime's
`pse-patch.md`. In particular, default lazy getters recreate contexts and do not
share this phase budget. Accounted traversal must explicitly merge exposed raw
fragments under the shared context. No complete receiving-phase qualification or
production activation follows from enabling this generator option.

Changes: `src/lib.rs`, `src/view.rs`, `src/lazy_view.rs`; manifest test registration;
new `tests/allocation_compile.rs` and `tests/allocation_fixture.rs.inc`. The compile
test is included by the existing `pse-codegen` integration target so the
workspace's exact lock and validation route apply to the supplier controls. It
generates real eager/lazy decoder implementations and executes exact-fit,
one-byte-short, fragment, scalar, duplicate-map, oneof and unknown-field controls
against the maintained runtime. Default output is compared byte-for-byte against
an independently compiled published registry 0.9.2 generator on the same descriptors,
and toggling the option back off must reproduce the default. Reproduce from the checkout root:

```sh
scripts/pse-env -- scripts/pse test pse-codegen:buffa_allocation
```

Use approved checkout activation and an actual user-systemd scope; no shared
target directory, upstream acceptance or universal memory bound is claimed.
