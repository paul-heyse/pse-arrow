---
id: ADR-0003
title: Re-implement IDAES clean-room and pin parity to idaes-pse 2.12.0
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-42, DM-48, DM-59]
blueprint: [§0.2, §3.1, §6.14, §25]
review: not-required: a provenance and scope statement; neither review raised a finding against it
evidence: Proposed
supersedes: []
superseded-by: null
revisit: IDAES releases a version that changes an enumeration preserved by name in blueprint §6.14, or drops a CPython version the parity environment needs
verification: `python / parity` pre-flight `test_00_preflight` asserts `idaes.__version__ == "2.12.0"`; `governance / adr-lint` asserts the IDAES tag in `scripts/fetch-external.sh` equals the `parity` dependency-group pin

---

# ADR-0003: Re-implement IDAES clean-room and pin parity to idaes-pse 2.12.0

## Context

The platform reproduces core IDAES-PSE capabilities (blueprint §0.2) and is parity-tested against a pinned IDAES release (§3.1, §25), which makes the relationship to an existing Apache-2.0 project a licensing and honesty question, not only a technical one.

## Scope

Binds how IDAES source may be used, which IDAES release parity is measured against, and how that pin moves. It does not bind the parity tolerances (blueprint §24) or which subsystems are in scope (§0.2).

## Drivers

Clean-room provenance must be defensible; parity results are meaningless without a pinned reference; users must not read the project as an IDAES product.

## Options

Fork `idaes-pse` — rejected: the architecture shares no structure with it (D1, D2), and a fork inherits an import graph the platform deliberately does not have. Float the parity pin to the latest release — rejected: a parity failure would be indistinguishable from an upstream change.

## Outcome

`external/idaes-pse` is checked out at the pinned tag and read for behaviour; no IDAES code is copied. The only coupling is the parity harness. Parity is pinned at `idaes-pse==2.12.0` in the `parity` dependency group and moves only by a new ADR. `docs/relationship-to-idaes.md` carries the non-affiliation and trademark statement and cites Lee et al. 2021.

### Consequences

`external/` is gitignored and fetched by `scripts/fetch-external.sh`; the parity environment is a separate resolution (`.venv-parity`) so that IDAES's transitive dependencies (`pydantic`, `sympy`, `networkx`, `pandas`) can never become platform dependencies.

### Compensating controls

Enumerations deliberately preserved by name are enumerated in blueprint §6.14 so a reader can see exactly where names are shared and why; the PR template carries a clean-room attestation.

### Confirmation

The parity job fails — never skips — when the interpreter, the IDAES version or `ipopt` is not what the pin says (root `conftest.py`, `--parity`).

## Pros and cons

Clean-room costs re-deriving behaviour that could be read off a source file; that cost is the price of an architecture that is not a translation of IDAES's object graph.

## More information

Blueprint §0.2 (scope), §6.14 (preserved enumerations), §25 (phase exits); `docs/relationship-to-idaes.md`; Lee et al. 2021, *J. Adv. Manuf. Process.* 3(3) e10095.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
