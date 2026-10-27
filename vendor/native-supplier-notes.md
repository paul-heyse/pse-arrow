# Local native supplier corrections

These are edited copies of the published, locked crates.io sources, including their
packaged native trees and upstream license files. Package versions stay unchanged;
they are local integrations, not upstream releases. The exact archives, checksums,
packaged VCS revisions and complete byte-preserving deltas are recorded in
[deltas/readme.md](deltas/readme.md). That record includes all added and removed files,
not just the build-script changes described below.

Cargo cache markers and `.cargo-checksum.json` are omitted. The original package
checksums in the linked archive table identify the imported sources; they do not authenticate patched
files. VCS metadata and packaged lockfiles remain upstream snapshots. `Cargo.toml.orig`
does too, except SUNDIALS's deliberately synchronized bindgen 0.73.2 build declaration.
The normalized `Cargo.toml` files select these local sources; production
resolution belongs to the repository's root lockfile.

The four patched normalized manifests connect every eligible SuiteSparse path to
the sibling `suitesparse-sys` package with `version = "=0.1.4"` and a relative path.
Diffsol and diffsol-nl likewise select the sibling diffsol-la/diffsol-nl packages at
`=0.1.1`. This keeps one `links = "suitesparse"` source across optional SUNDIALS and
SuiteSparse features. It requires corresponding exact-version path selections for
the repository's direct dependencies. No dependency override table is needed.
The Diffsol, diffsol-la and diffsol-nl numerical Rust/C sources are unchanged from
their archives. SUNDIALS numerical C sources are locally modified as described below;
the published feature declarations remain intact.
The [derived checked source receipts](deltas/readme.md) authenticate each maintained tree
against its complete delta during the native build. SUNDIALS reports its actual selected
bundled/external branch to the consumer; IDAS and KINSOL share that premise. Linked Diffsol
includes its checked family sources and selected KLU provider receipt as well. Missing qualified
provider evidence leaves ordinary direct calls available and reports an unavailable native build
profile. The separate [faer arithmetic policy](faer-arithmetic.md) records its maintained
binary64 kernel realization and focused controls.
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
  Every bundled-build route also observes present or absent `CI_JOB_ID`,
  `CI_JOB_STARTED_AT`, `SOURCE_DATE_EPOCH` and `TZ`: CMake uses those inputs for the
  public `SUN_JOB_*` metadata macros. Successful standalone-provider builds do not
  consume those inputs and do not emit their watches.
- Diffsol's optional `sundials` build branch observes its SUNDIALS options and both
  selected providers' directories. Its local C benchmark tree is observed because
  compiled translation units also include local headers and other C sources.
- Each bindgen builder observes Cargo inputs alongside the existing `IgnoreMacros`
  and, where present, `ParseSignedConstants` callbacks. These callbacks append;
  upstream custom callbacks never replaced another callback. `CargoCallbacks`,
  directly or through the SUNDIALS delegate below, observes included headers and
  bindgen environment options for Cargo freshness.
- SUNDIALS replaces the blanket Cargo callback with a delegating callback that
  omits headers in this invocation's CMake-installed include namespace. Those
  headers are generated outputs; watching their post-start writes invalidates an
  otherwise unchanged next build. Both lexical and canonical containment must
  identify that namespace, so external aliases and paths escaping it remain
  watched. Successful standalone-provider builds have no generated namespace and
  retain every included-header watch. Environment callbacks still delegate to
  bindgen, while the bundled source-tree, wrapper, KLU/provider directory and
  explicit metadata environment watches retain their existing input roles.
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
algorithm. The build-freshness corrections retain the existing CMake/compiler options. The
bundled SUNDIALS tree also contains the separately described changes below. No speed
or scientific qualification claim follows from this provenance reconciliation.

## Bundled SUNDIALS integration and upstream defect

The Rust wrapper remains 0.6.2, with its published 7.1.1 archive retained as the
reconstruction baseline. The selected native projection is SUNDIALS 7.9.0, authenticated
by the official release asset digest in [`native-source.json`](sundials-sys/native-source.json).
Full path/byte/mode replacement and integration are in the existing complete delta;
[the delta table](deltas/readme.md) counts changes against the original wrapper archive.
The wrapper/build scripts retain provider, freshness, binding and identity wiring. The build
uses namespaced 7.9 options and separate output directories for changed upstream CMake
definitions, preserving earlier caches without reusing their deprecated option state.
The wrapper uses the workspace-selected bindgen 0.73.2 to audit complete 7.9 native type
layouts; the older binder produced one-byte generic vector placeholders despite Clang's
nonzero layout facts. Opaque Rust placeholders are not native allocation evidence.
`wrapper.h` still exposes the KLU integration. The ten native integration files relative
to the released 7.9 projection are:

| Files relative to `vendor/sundials-sys` | Local change |
|---|---|
| `vendor/include/idas/idas.h` | Declares checkpoint population/usage, backward-problem count and accepted-step budget/query APIs. |
| `vendor/src/idas/idaa.c`, `vendor/src/idas/idaa_io.c` | Maintains checkpoint population/high-water accounting, refuses checkpoint growth above its limit, exposes backward count, and bounds checkpoint replay without renewing the owner budget on restoration. |
| `vendor/src/idas/idas_impl.h`, `vendor/src/idas/idas_io.c` | Stores/initializes the IDAS budget/accounting state and exposes accepted-step budget/query functions. |
| `vendor/src/idas/idas.c` | Checks and charges accepted physical steps without renewing the original accepted-step permission. |
| `vendor/include/sunlinsol/sunlinsol_klu.h`, `vendor/src/sunlinsol/klu/sunlinsol_klu.c` | Adds opt-in `SUNLinSol_KLUFixedPattern`, canonical CSC-pattern checks, valid symbolic reuse, read-only work counters and synchronous work notifications. Fixed-pattern owners refuse mutable native accessors; the generic constructor retains its upstream mode. |
| `vendor/src/kinsol/kinsol_aa.c` | Destroys the Q/df/dg arrays using their allocated depth, preserving ownership across valid history-depth changes and partial construction failures. |
| `vendor/src/sundials/sundials_nvector.c` | Destroys all `j` earlier clones when clone `j` fails in either generic clone-array constructor. |

The former upstream-defect workaround is the quadrature history refresh, introduced
by commit `8dc1f9a9e`. In bundled SUNDIALS 7.1.1, `IDACalcIC` corrects the primal and
sensitivity initial state after the quadrature derivative history was initialized.
The local correction refreshes `phiQ[1]` and, when enabled, `phiQS[1]` from that
corrected state without resetting the solver/work counters or the initial quadrature
values. Callback failures propagate before physical integration starts. This is a
numerical behavior correction, not a build-freshness change. The selected 7.9 source follows upstream
`IDASolve` -> `IDAQuadSetup` at the first physical solve, after `IDACalcIC`; it removes the local
post-`IDACalcIC` calls and helper-only factoring. The three corrected-history controls below
passed against this selected source. The checkpoint/budget,
KLU accounting and fixed-pattern additions are local integrations; missing upstream
APIs alone do not make them upstream defects.

The enduring behavior is declared by blueprint §13. The functional controls
are `quadrature_histories_use_corrected_algebraic_initial_state_and_sensitivities`,
`quadrature_histories_use_corrected_algebraic_scheduled_state_and_sensitivities` and
`corrected_quadrature_history_callback_failures_stop_before_physical_steps` in
`crates/pse-backend-native/src/dynamics/idas_initial_tests.rs` were **Tested** with
`scripts/pse-env --native=klu -- cargo nextest run -p pse-backend-native --features
pse-backend-native/idas,pse-backend-native/kinsol,pse-backend-native/diffsol --lib --locked`
and a filter selecting those controls on the bundled 7.9.0 source. All three passed;
the failure baseline is zero. This focused result supports removing the replaced workaround;
it does not establish broader native qualification, whose owner remains blueprint §24.2.
Register R-41 owns the release/report revisit for the quadrature defect.

### 7.9 consumer audit

The exact released headers and implementation preserve the selected numerical classes.
SPGMR/SPFGMR content fields and `SUNLinSolSpace` formulas remain
`n*(m+5)+m*(m+5)+2` and `n*(2*m+4)+m*(m+5)+2` real words; integer payloads remain
one word per cloned serial vector. One new serial-vector operation pointer increases
the selected 64-bit vector/object/ops/content headers from 496 to 504 bytes, within the
existing 512-byte per-vector bound and independent local-error allowance.
`idas_bbdpre.c` still owns four cloned vectors, two empty views, the band matrix and solver;
the replacement envelope, band stride, pivots and six header allowances remain applicable.

`kinsol_aa.c` now owns lazy allocation of three m-vector arrays, two history vectors, R,
and two `2*(m+1)` scalar/pointer arrays. `kinsol_orth.c` adds one optional work vector,
QR data and the optional m-by-m ICWY T matrix. Existing vector/header/QR admission bounds
cover these allocations before entry. Neither lazy allocator updates `KINGetWorkSpace`;
retained accounting includes their three m-vector arrays, three other possible vectors,
headers and QR/scalar/pointer storage separately. The current Anderson depth resets on every
call. Each adapter attempt supplies `min(configured_depth, iteration_cap - 1)`, then derives
the depth afresh for the next attempt while retaining the original admitted storage. This
matches upstream's intended effective depth and avoids entering orthogonalization setup with
zero allocated history when the cap is one.

The released AA destructor used the current depth to destroy arrays whose allocated depth
can differ after an attempt limit changes. The local correction uses `kin_m_aa_alloc`, assigned
before the first allocation and cleared only after cleanup. Null guards protect absent arrays;
the generic clone-array constructors clean up a failed array before returning null. Those
constructors formerly destroyed `j-1` earlier vectors on failure at clone index `j`; their
local correction destroys all `j`. Deterministic test-only clone failures exercise indices
0/1/2 in Q, df, dg and the empty-vector variant, plus orthogonalization allocation failure;
clone/destructor counts check partial ownership without host allocation exhaustion.
The control
`sundials_790_anderson_allocations_follow_allocated_depth_through_resize_failure_and_drop`
and the retained-session depth-restoration control were **Tested** in the same linked
KLU/IDAS/KINSOL/Diffsol package mode described above. Both passed against the corrected
source with a zero failure baseline.

Allocation can now fail before primary
counters reset; such `KIN_MEM_FAIL` returns do not publish prior-session counters.
KINSOL's new damping/depth callback failures retain their native names and evaluation cause.

`IDAReInit` now resets attached linear counters immediately; nonlinear and step counters
retain their native reinitialization meaning. Constraint failures/corrections are separately
reported rather than folded into nonlinear convergence failures. `IDA_TOO_CLOSE` is a named
invalid-input outcome. The ten `IDAGetIntegratorStats` fields, selected history/work-vector
envelopes and local-error getter freshness guards remain applicable. Context error-handler
Push still appends, so Clear precedes installing the bounded sink. KLU retains its
`EPSILON^(2/3)` reciprocal-condition/condition-estimate fresh-factor fallback; the fixed-pattern
integration preserves work notifications, immutable support and failed-factor invalidation.
Workspace reporters remain deprecated but present in 7.9; 8.x removal is a future revisit.

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
7. For bundled SUNDIALS, hold the Cargo unit and private provider fixed, set
   `SOURCE_DATE_EPOCH=0` and `TZ=UTC`, and establish the unchanged fresh control.
   Change each of the four metadata inputs separately, including absence/presence
   of the CI overrides. Confirm the supplier reruns and the generated config header
   and bindings reflect the selected CI value or timestamp. Hold CI overrides
   absent while checking epoch/timezone changes. Restore the original environment
   and establish freshness again. On a successful standalone-provider branch,
   verify these watches are absent and changing an unused metadata input leaves
   the supplier fresh.
   For the bundled branch, compare the emitted header watch set: headers inside
   its own generated include namespace must be absent, while external system and
   KLU header watches remain. Repeat unchanged after restoring the baseline;
   generated header writes must not cause an additional supplier rerun.

These are build freshness controls; numerical solver/runtime qualification is a
separate integration obligation.
