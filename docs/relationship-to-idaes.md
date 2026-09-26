# Relationship to IDAES

## Not affiliated

**`pse-arrow` is not affiliated with, endorsed by, or sponsored by the IDAES
project, the U.S. Department of Energy, or any of the institutions of the IDAES
consortium.** "IDAES" is used here only to name the project this one is compared
against, which is nominative use; any trademarks are the property of their
owners. Nothing in this repository is an IDAES release, an IDAES fork, or an
IDAES-supported product, and questions about it should come here rather than to
the IDAES project.

IDAES-PSE is Apache-2.0 licensed. `pse-arrow` is `MIT OR Apache-2.0`.

## Clean room

The architecture is a re-implementation, not a translation:

- `external/idaes-pse` is a **reading copy**, checked out at the pinned parity
  tag by `scripts/fetch-external.sh` and gitignored. It is read to understand
  *behaviour* — what a control volume writes, which defaults a solver profile
  carries, what an initializer does in what order.
- **No IDAES code is copied** into this repository. No file, no function body, no
  docstring. The pull-request template carries a clean-room attestation, and
  `external/` is on the agent configuration's deny list for edits.
- The **only** coupling is the parity harness (`python/pse/parity/`), which
  imports `idaes` in a separate environment and compares results.
- Names are preserved where parity requires a two-way mapping — for example the
  property name `enth_mol_phase` and the enumerations listed in blueprint §6.14.
  Those are interface facts, and each is enumerated in the architecture rather
  than absorbed silently.

The architectural decisions are recorded in
[ADR-0003](adr/0003-clean-room-relationship-and-parity-pin.md) (clean-room
relationship and the parity pin) and
[ADR-0032](adr/0032-external-checkouts-are-not-vendored.md) (external checkouts
are fetched, never vendored).

## The parity pin

Parity is measured against exactly one IDAES release:

```
idaes-pse == 2.12.0
```

It is declared once, in the `parity` dependency group of `pyproject.toml`, and
it is the version `scripts/fetch-external.sh` checks out. A lint in
`governance / adr-lint` asserts the two agree, so the reading copy can never
drift from what the tests measure against.

The parity environment is a **separate resolution** (`.venv-parity`, CPython
3.13) because `idaes-pse` classifies 3.10–3.13 while the platform package
targets ≥ 3.11 and is verified on 3.14. Keeping them separate is what stops
IDAES's transitive dependencies — `pydantic`, `sympy`, `networkx`, `pandas` —
from becoming platform dependencies. The parity pre-flight **fails rather than
skips** when the interpreter, the IDAES version or `ipopt` is not what the pin
says.

### How the pin moves

Moving it is a decision, not a dependency bump:

1. Open a PR labelled `adr` with a new ADR that supersedes the parity-pin part
   of ADR-0003, stating what changed upstream and what parity results moved.
2. Update the `parity` group pin and the tag in `scripts/fetch-external.sh` in
   the same PR; the lint checks they agree.
3. Re-run `python / parity` on every supported interpreter and record the
   differences in the ADR's Consequences section with a core principles §D
   evidence label.

Dependabot never moves it: `idaes-pse` is in the `ignore` list of the uv
configuration for exactly this reason (ADR-0035).

## Citing IDAES

When citing the project this one is compared against:

> Lee, A., Ghouse, J. H., Eslick, J. C., Laird, C. D., Dowling, A. W.,
> Bhattacharyya, D., Biegler, L. T., Burgard, A. P., Miller, D. C., &
> Omell, B. P. (2021). The IDAES process modeling framework and model library —
> Flexibility for process simulation and optimization. *Journal of Advanced
> Manufacturing and Processing*, 3(3), e10095.
> <https://doi.org/10.1002/amp2.10095>

`CITATION.cff` references the same paper as the parity reference.

## What "core IDAES-PSE capabilities" means

The scope is the modeling framework, not the example libraries: the process
block lifecycle, unit models and ports, control volumes, the physical and
modular property frameworks, reactions, the generic unit model library, costing,
initialization, scaling, diagnostics, solvers and DAE discretization,
serialization and units, and parameter sweeps. Surrogates, the DMF, the UI,
Pecos, the apps and the `models_extra` libraries are **out of scope**; surrogate
*embedding* would use the physical provider contract (blueprint §9.4). Blueprint §0.2
carries the full table, and the
[capability coverage appendix](authoritative_design/sections/scope-and-open-design.md#capability-coverage-against-idaes)
records the current status of each capability.
