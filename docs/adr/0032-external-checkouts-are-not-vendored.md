---
id: ADR-0032
title: Fetch external reading copies into a gitignored external/ rather than vendoring them
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-48, DM-02]
blueprint: [§3.1, §0.2]
review: not-required: a repository-hygiene decision; neither review raised a finding against it
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A CI job needs an external tree that a network fetch cannot supply reproducibly
verification: `governance / adr-lint` asserts the IDAES tag in `scripts/fetch-external.sh` equals the `parity` group pin; `.gitignore` covers `/external/`

---

# ADR-0032: Fetch external reading copies into a gitignored external/ rather than vendoring them

## Context

The blueprint reads `idaes-pse/`, `arrow-rs/` and a DataFusion checkout for behaviour and for API facts (§3.1's closing note). Vendoring them would add hundreds of megabytes and three trees with their own histories to a repository whose own content is text.

## Scope

Binds how external source is obtained and where it lives. Which tags are used is derived from the pins, not chosen here.

## Drivers

Reading copies are derived artifacts; the authoritative statement of which version is read is the pin, not a checkout; a vendored tree invites edits that no pin describes.

## Options

Git submodules — rejected: they pin a commit that can diverge from the dependency pin, giving two authorities. Vendor the trees — rejected: size, and a second copy that can be edited.

## Outcome

`external/` is gitignored and populated by `scripts/fetch-external.sh`, which clones `idaes-pse` at the parity tag, `arrow-rs` at the arrow pin and `datafusion` at the datafusion pin, reading the arrow and datafusion tags from `Cargo.lock`.

### Consequences

A fresh clone cannot read external source until `just fetch-external` runs; `just doctor` reports it as missing rather than failing silently.

### Compensating controls

A lint asserts the script's IDAES tag equals the `parity` dependency-group pin, so the reading copy cannot drift from what parity tests against.

### Confirmation

`governance / adr-lint` runs the tag check on every PR; agent settings deny edits under `external/`.

## Pros and cons

Fetching costs network in a fresh environment; vendoring costs a second authority for three versions.

## More information

Blueprint §3.1 (closing note on the vendored checkout), §0.2 (source paths); ADR-0003.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
