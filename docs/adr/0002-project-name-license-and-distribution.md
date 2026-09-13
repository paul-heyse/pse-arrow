---
id: ADR-0002
title: Name the project pse-arrow, prefix crates pse-, dual-license MIT OR Apache-2.0
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-11, DM-55]
blueprint: [§3.2, §26]
review: not-required: naming, licensing and distribution carry no semantic surface; blueprint §26 lists the prefix as "placeholder, to confirm"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: The PyPI name `pse` becomes available, or a crates.io name collision forces a different crate prefix at the phase-0 publish
verification: `tests/governance/tests/every_crate_registered.rs` (crate prefix); `governance / reuse` (`reuse lint` over the SPDX headers)

---

# ADR-0002: Name the project pse-arrow, prefix crates pse-, dual-license MIT OR Apache-2.0

## Context

Blueprint §3.2 leaves the `pse-` prefix as a placeholder and §26 carries "crate naming and repository layout" as an open decision. The repository needs one name on GitHub, on crates.io and on PyPI before anything is published.

## Scope

Binds the repository name, the crate prefix, the Python import name, the PyPI distribution name and the license expression. It does not bind when anything is published (ADR-0034 and the register cover that).

## Drivers

One name that reads the same in three registries; a license that lets the Rust and Python ecosystems both consume the result; a prefix short enough for 23 crates.

## Options

`idaes-rs` or anything with `idaes` in it — rejected: implies affiliation (ADR-0003). Import name `pse_arrow` — rejected: the import name is what appears in every example; `pse` is short and unambiguous inside the package. Apache-2.0 only — rejected: the Rust ecosystem's default is the dual license.

## Outcome

Repository `paul-heyse/pse-arrow`; crate prefix `pse-*`; Python import name `pse`; PyPI distribution `pse-arrow` because PyPI `pse` is already taken; license `MIT OR Apache-2.0` with `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSES/` and a two-line SPDX header on every authored source file.

### Consequences

The distribution name and the import name differ, which every install instruction has to state. `REUSE.toml` covers headerless file types by glob so Markdown needs no header; the code generator emits the header itself.

### Compensating controls

`reuse lint` in `python / lint`; `tests/governance/tests/every_crate_registered.rs` asserts every workspace member carries the prefix and is listed in the blueprint §3.2 layout.

### Confirmation

`pip install pse-arrow` then `import pse` is the documented pair; `README.md` and `docs/README.md` state it in the first paragraph.

## Pros and cons

The mismatch between distribution and import name is a real papercut, accepted because `pse` is the name the blueprint uses throughout §3.2 and renaming 23 crates later is worse.

## More information

Blueprint §3.2 (workspace layout) and §26 (open decisions); `REUSE.toml`; ADR-0003 for the non-affiliation statement.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
