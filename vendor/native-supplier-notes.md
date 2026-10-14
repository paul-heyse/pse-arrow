# Local native supplier corrections

These are the published, locked crates.io sources, including their packaged native
trees and upstream license files. The package SHA256 values below were checked
against the original `.crate` archives when importing them. Every packaged file
matches that archive except the local edits described here. Package versions stay
unchanged; these are local build freshness corrections, not upstream releases.

| Package | Version | Original package SHA256 |
| --- | --- | --- |
| `suitesparse_sys` | 0.1.4 | `9e4ced97d4de36a6c50e381ae4f9d91d89d056576ceedd2252e4c688dd96b15f` |
| `sundials-sys` | 0.6.2 | `6fbd1d53481b83fc758406097f07b668bf243945e2b1f7db93a8f997cf6b44c9` |
| `diffsol` | 0.16.2 | `4b324c954a5178683516e35e06051f02e3e2471d22b8175d9a01b274bd81269d` |
| `diffsol-la` | 0.1.1 | `0fd836b0b52b6459b1e16c4ebfe267599b39c07a2eb638426a4e72a279a4e384` |
| `diffsol-nl` | 0.1.1 | `6b24ab3f56ff94c563ab85e7897ba57f884617d0dc83a081028d28ec800213a1` |

Cargo cache markers and `.cargo-checksum.json` are omitted. The original package
checksums above identify the imported sources; they do not authenticate patched
files. `Cargo.toml.orig`, VCS metadata and packaged lockfiles remain upstream
snapshots. The normalized `Cargo.toml` files select these local sources; production
resolution belongs to the repository's root lockfile.

The four patched normalized manifests connect every eligible SuiteSparse path to
the sibling `suitesparse-sys` package with `version = "=0.1.4"` and a relative path.
Diffsol and diffsol-nl likewise select the sibling diffsol-la/diffsol-nl packages at
`=0.1.1`. This keeps one `links = "suitesparse"` source across optional SUNDIALS and
SuiteSparse features. It requires corresponding exact-version path selections for
the repository's direct dependencies. No dependency override table is needed.
All upstream feature selections and numerical Rust/C sources are preserved.
Two SuiteSparse files retain their published CRLF bytes through a local
`.gitattributes` exception to the repository's LF normalization policy.
`REUSE.toml` records wrapper licenses separately from native trees. SuiteSparse's
native terms are collected in `LICENSES/LicenseRef-SuiteSparse-Upstream.txt`, which
retains the upstream compilation and packaged component/third-party license texts;
each component keeps its own terms and per-file notices. SUNDIALS's Spack files and
Diffsol's imported SUNDIALS examples/reference text retain their specific licenses.

The three patched build scripts make these corrections:

- `suitesparse-sys` observes present or absent `SUITESPARSE_INCLUDE_DIR` and
  `SUITESPARSE_LIBRARY_DIR` on its external-provider branch. Bundled selection
  observes the packaged native tree and publishes its actual selected include/lib
  paths as Cargo links metadata, without consulting an unused external prefix.
- `sundials-sys` observes present or absent `SUNDIALS_INCLUDE_DIR` and
  `SUNDIALS_LIBRARY_DIR` only on its standalone-provider branch, including when an
  attempted system provider subsequently falls back to the bundled tree. It reads
  SuiteSparse `DEP_*` metadata only with `klu`; the KLU library directory is observed
  when CMake consumes it. Cargo-generated `DEP_*` values do not receive caller-env
  rerun directives: upstream dependency/metadata invalidation carries those values.
- Diffsol's optional `sundials` build branch observes its SUNDIALS options and both
  selected providers' directories. Its local C benchmark tree is observed because
  compiled translation units also include local headers and other C sources.
- Each bindgen builder adds `CargoCallbacks` alongside the existing `IgnoreMacros`
  and, where present, `ParseSignedConstants` callbacks. These callbacks append;
  upstream custom callbacks never replaced another callback. Included headers and
  bindgen-observed environment options now participate in Cargo freshness.
- SuiteSparse/SUNDIALS bindgen parsing uses `-fno-builtin` to preserve header
  `size_t` typedef identity instead of Clang builtin integer prototypes. This
  corrects the pinned nightly's runtime-symbol warnings without disabling a lint
  or changing native-code compilation/optimization. See [the upstream explanation](https://github.com/rust-lang/rust-bindgen/issues/1770#issuecomment-1412631382).
- Bindings are rendered through bindgen's existing byte writer and compared with
  the current output. Identical bytes preserve the output mtime; changed or missing
  outputs are written. Input observation still reruns bindgen when required.

Directory watches conservatively cover changes throughout the selected native
include/library directories, including additions and replacements. This may observe
unrelated files within those directories. Default linker search without a known
library directory remains linker-owned; these scripts do not invent a linker search
algorithm. Bundled native trees and existing CMake/compiler options remain intact.
No speed or scientific qualification claim follows from these source corrections.

## Proposed functional probes

The integration owner should use private copies of valid external prefixes and the
same selected Cargo unit, features, target and command throughout each sequence.
Do not mutate shared installations, clean Cargo artifacts or vary global flags.

1. Build once, then repeat unchanged and establish Cargo's fresh control.
2. Append a harmless comment to an actually included private header. The supplier
   build script must rerun and emit the included-header watch; identical bindings
   must retain both their bytes and mtime. Record script, native compiler and Rust
   downstream activity separately.
3. Add a harmless exported integer macro to that header. The same supplier unit must
   rerun and produce changed binding bytes and output mtime; repeat unchanged to
   establish freshness again. Restore the private header and verify recovery.
4. Change one consumed include/library option to another valid private directory
   while keeping the Cargo selection fixed. Confirm the supplier reruns and its
   emitted selected paths change. Exercise absence/presence separately, where a
   valid default provider exists, and a private selected library-file mutation.
5. For bundled SUNDIALS with KLU, mutate the private selected KLU header and verify
   SuiteSparse and SUNDIALS observations, selected `DEP_*` paths and generated
   outputs. Confirm no `rerun-if-env-changed=DEP_SUITESPARSE_*` directive exists.
6. Exercise the optional Diffsol SUNDIALS branch separately. For feature branches
   that do not consume these external options, verify their directives are absent
   and changing an unused provider option leaves that supplier unit fresh.

These are build freshness controls; numerical solver/runtime qualification is a
separate integration obligation.
