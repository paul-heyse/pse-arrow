# Recommendation-audit evidence

Supporting evidence for the [integrated review](../../../reviews/design_review_in-use-library-remediation_2026-10-21.md)
and its [net-change companion](../../../reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
The companion owns editorial dispositions; this page records the coordinator's bounded
source inspection and executed library probe, not implementation progress.

- **Date:** 2026-10-21.
- **Input review:** `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`, original principal path.
- **Product source:** `a791c533eca0f65b195895f071e60cd1e38e121c`, unchanged at audit start.
- **Method:** source-led independent reassessment, with a selective probe where an open
  library question could change the recommendation. No product/native build or server run.

## Identity codec hook probe

**Question.** R17 leaves open whether msgspec encodes a `bytes` subclass through its native
base64 route, bypassing `enc_hook` and forcing a replacement Python value representation.

**Executed command:**

```bash
scripts/pse-env -- .venv/bin/python docs/design_review/evidence/in-use-library-remediation-2026-10-21/recommendation-audit/identity-hook-probe.py
```

**Tested:** Python 3.14.8, msgspec **0.21.1** (checked by the script), local Linux,
2026-10-21. Exit 0; zero failures against the zero-failure baseline. The final run covers:

- a sixteen-byte subclass matching the current identity's representation and width rule;
- a non-`bytes` value class, for comparison;
- direct and nested `msgspec.Struct` encoding and typed decoding;
- hexadecimal `enc_hook` and `dec_hook` invocation for both representations;
- malformed-width refusal;
- the runtime erasure of `typing.NewType` role aliases.

Both representations invoke the hooks and recover the original bytes. The bytes subclass
can remain for the proposed hexadecimal document codec: replacement is not required by this
mechanism. NewType role aliases supply static distinctions, not runtime role evidence.
The script and [captured output](identity-hook-probe.txt) make the result reproducible.

**Limits.** The probe uses small equivalent identity classes, not the product extension or
the complete generated, version-first document codec. It does not establish every supported
Python version, generator behavior, boundary ABI, or role/domain admission. Preserve explicit
hooks at each affected codec entry and the existing strict/version-first checks. This result
does not qualify an implemented R17.

**Documentation route.** Context7 resolved msgspec to `/websites/jcristharif_msgspec` and
queried custom hooks/bytes subclasses/NewType. Its returned
[NewType documentation](https://jcristharif.com/msgspec/supported-types.html#newtype)
describes base-type behavior but did not settle subclass-hook dispatch. The exact pinned
runtime probe above settles that narrower question. Current product representation:
`python/pse/contracts/values.py`, `SemanticId`; current row/document codec split:
`python/pse/codec/__init__.py` and `_generated.py`.

## Time and physical-coordinate boundary

**Interface-checked.** PyO3 **0.29.2**, `src/conversions/std/time.rs`:

- `Duration::extract` (20–59) accepts `datetime.timedelta`, refuses negative values, and
  converts microseconds to nanoseconds.
- `Duration::into_pyobject` (63–82) uses `subsec_micros()`, discarding sub-microsecond parts;
  conversion also has Python timedelta range limits.
- The limited-API extraction branch exists; ABI availability is distinct from semantic fit.

The source is the exact installed crate selected by `Cargo.lock`. Context7 resolved PyO3
to `/pyo3/pyo3`; the conversion query returned broad ABI documentation, so the exact-release
source, rather than an inferred documentation guarantee, supplies these claims.

**Implemented:** `crates/pse-py/src/workflow.rs`, `SimulationSettings::new` (2319–2375),
keeps physical `start`, `end`, `samples` and `initial_step` as Float64 while converting
`time_limit` to a Rust `Duration`. Thus R18 must distinguish wall-clock budgets from physical
coordinates and precision-bearing intervals. A blanket seconds-to-timedelta substitution
would narrow supported physical values. No new execution was needed to establish this
conversion limitation; a full boundary change still needs precision/range and physical-unit
acceptance in its implementing plan.

## Native allocation-owner lifetime

**Interface-checked:** PETSc **3.24.0**, `src/sys/memory/mal.c` (187–246), exposes
process-global `PetscTrMalloc`, `PetscTrFree`, `PetscTrRealloc` and `PetscMallocSet`;
its documented setup precedes initialization. Context7 resolution `/websites/petsc_release`
and its [PetscMallocSet reference](https://petsc.org/release/manualpages/Sys/PetscMallocSet/)
orient this lookup; the cached 3.24.0 source supplies the pinned contract.

**Implemented:** `crates/pse-backend-native/src/petsc.rs`, `PROCESS`, `Admission::enter`
and `Admission::ready` (62–159), serialize PETSc ownership and retain process initialization
independently of attempts. `crates/pse-petsc-sys/bridge/pse_petsc.c` (196–203) initializes
through `PetscInitializeNoArguments` and later finalizes through the process owner.

**Proposed condition for R16:** thread-local attribution selects an allocation's owner at
allocation time; a later free/reallocation must use retained allocation owner/size provenance,
not whichever attempt is currently active. Initialization/process allocations need their own
lifetime. Accounting is exact only for the intercepted allocator domain; MKL/BLAS and other
bypasses are not covered by this hook. The original native inventory already records the
header and bypass limits. Keep foreign-envelope admission until coverage, concurrency and
release accounting are established. No allocator hook was installed or run in this audit.

## Covariance provenance and validity

**Implemented, source-inspected:**

- `crates/pse-runtime/src/math/settings.rs`, `ParameterCovariance::admit` (121–146), checks
  distinct parameters, square extent, approximate symmetry and nonnegative diagonal, not
  positive semidefiniteness.
- `crates/pse-runtime/src/workflow/uncertainty.rs`, `propagate_checked` (147–210), uses the
  supplied matrix in `J Σ Jᵀ`, rejects nonfinite output, and copies the supplied `run_id`.

Consequently a genuine fit identifier attached to modified covariance values is not certified
provenance. R20 must hydrate the retained fit covariance or match complete ordered contents
and relevant validity/dependencies to it, rather than merely resolve the identifier.

The symmetric matrix `[[1, 2], [2, 1]]` has a positive diagonal and passes the stated shape
predicate, yet `J = [1, -1]` gives `J Σ Jᵀ = -2`. This is a source-derived counterexample,
**not an executed Rust regression test**. It supports narrowly scoped F10: external covariance
admission must establish statistical validity, including positive semidefiniteness under an
explicit numerical tolerance, or withhold/refuse the result. Singular valid covariance remains
admissible; demanding strict positive definiteness would reject legitimate inputs. Use a
suitable in-use numerical-library decomposition; no new handwritten eigensolver is proposed.
One **Interface-checked** enabler is pinned faer **0.24.4**,
`src/linalg/solvers.rs:406–455`: `self_adjoint_eigenvalues(Side)` returns sorted eigenvalues
or `EvdError` without computing eigenvectors. Its square/self-adjoint preconditions, working
allocation and failure handling remain the admission owner's responsibility; the PSD
tolerance and retained numerical-validity contract remain **Proposed**. Context7 resolved
`/sarah-quinones/faer-rs` but returned no match for this API; exact pinned source, not search
silence, establishes availability.

## Coverage boundary

The independent review and bounded researchers inspect additional store/result/preparation
sources documented in the companion and integrated review. Original probes remain historical:
none of their timing, engine behavior or numerical results was rerun here. RocksDB concurrency,
guarded evaluator benefits, revised compiler retention, native allocator coverage, and complete
Python/Arrow boundary journeys remain adoption evidence, not current passes.

## Direction-aware document schemas

**Interface-checked:** schemars **1.2.2**, `src/generate.rs:176–184`, selects the
Serialize/Deserialize contract in `SchemaSettings` before constructing a generator;
`SchemaGenerator` retains its settings and exposes read-only `settings()`/`contract()`.
Use a shared catalogue with separate generator contexts per direction and preserve that
direction in schema-set identity. A single catalogue does not require one mutable generator
to change contracts between roots. Context7 `/gresau/schemars` returned the upstream
[contract explanation](https://github.com/gresau/schemars/blob/master/docs/3-generating.md);
the exact pinned implementation establishes the API details.

The `solver-diffsol`-gated Shooting variants in `workflow/run.rs` belong to `RunRequest` and
`RunReport`, which are not themselves JsonSchema document roots. This example alone does not
establish feature variation in the exported document catalogue at `xtask/src/codegen/schemas.rs`.
R19's compiled-catalogue fingerprint remains a proposed way to detect drift; actual
feature-dependent membership/shape remains P6 settling evidence until examined directly.
