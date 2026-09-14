---
id: ADR-0049
title: Pin serde_json and uuid as direct dependencies for the manifest codec and explicit identities
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-41, DM-52]
blueprint: [§3.1, §5.1, §20.2]
review: not-required: a pin addition inside ADR-0018's accepted policy, with no new dependency family and no change to the resolved graph
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: a hash frame, a relation payload or a second document codec is routed through `serde_json`, or a pass other than the authoring `assign-ids` command mints a UUID
verification: `tests/governance/tests/pins_match_blueprint.rs` (the §3.1 rows equal `[workspace.dependencies]`); `just family-check`

---

# ADR-0049: Pin serde_json and uuid as direct dependencies for the manifest codec and explicit identities

## Context

Wave 1 implements the snapshot manifest (blueprint §20.2), the canonical extension metadata of §4.4 and the `explicit` id policy of §5.1. The manifest and the extension metadata are JSON documents, and an `explicit`-policy entity's id is a UUIDv7 minted by the authoring tool. `serde_json` 1.0.151 and `uuid` 1.26.1 already resolve in the committed `Cargo.lock` as transitive dependencies, so promoting them to direct `=` pins adds two rows and no packages.

## Scope

Binds which crates may declare the two pins and what they may be used for. It does not add a dependency family (blueprint §3.1 "Supply chain"), does not change the resolved graph, and leaves the authoring document formats (`serde-saphyr` for YAML, `toml` for `package.toml`, ADR-0021) untouched.

## Drivers

Correctness: one codec authority per serialization, not a hand-rolled writer beside serde. Reproducibility: the manifest and extension metadata are byte-compared in tests, so the codec must be the pinned one. Supply chain: both crates are already in the graph, so the direct pin costs nothing and makes the ownership visible to `pins_match_blueprint`.

## Options

Hand-roll the manifest and extension-metadata JSON — rejected: a second codec authority beside `serde`, and the one that would drift. Encode the manifest as TOML through the existing `toml` pin — rejected: the Python side reads the same document with `msgspec` JSON (§21.5) and would need a second parser. Mint `explicit` ids only in Python — rejected: `pse authoring assign-ids` is a Rust command and P1 must reject an unidentified entity without an interpreter present.

## Outcome

`serde_json = "=1.0.151"` is a direct dependency of `pse-catalog` (the `pse.manifest.v2` codec, §20.2), `pse-relations` (the canonical `ARROW:extension:metadata` codec, §4.4) and `xtask` (generator and report output). It is **never** a hash input and never carries relation data: every hashed byte goes through the `pse-ids` framing of §5.3, and every relation value is an Arrow array. `uuid = { version = "=1.26.1", features = ["v7"] }` is a direct dependency of `pse-authoring` only, used by `ids::assign_ids` to mint an `explicit`-policy identity at authoring time; P1 never mints an id — it rejects a missing one with `parse.missing_id` (§5.1).

### Consequences

Two more `=` pins to move in a `kind/library-upgrade` PR, and two more rows that `pins_match_blueprint` asserts against blueprint §3.1. A `serde_json::Value` must not reach `pse-ids`: the crate-boundary rule that `pse-ids` owns hashing is what keeps JSON off the identity path.

### Compensating controls

The §3.1 rows name the owning crates and state the non-uses ("never a hash input", "assign-ids only"), so an owner that is not listed is a visible manifest diff. `blake3_owner` keeps hashing in `pse-ids`, where `serde_json` is not a dependency.

### Confirmation

`just governance` runs `pins_match_blueprint`, which fails if a `[workspace.dependencies]` entry has no §3.1 row or a row disagrees with the manifest; `just family-check` asserts the resolved graph still holds one version per family.

## Pros and cons

Promoting an already-resolved transitive crate to a direct pin is cheap and makes an implicit dependency explicit; the cost is two more rows that a library upgrade has to move together with the rest of §3.1.

## More information

Blueprint §3.1 (supporting crates), §5.1 (id policies), §20.2 (manifest), §4.4 (extension metadata); ADR-0018 (pins and lockfile), ADR-0021 (`serde-saphyr` for YAML); [plan 03](../plans/03-wave-1-foundations.md) packet K-2.

## Status history

- 2026-09-13 — accepted. A short record within ADR-0018's accepted pin policy; review not required at maintainer discretion.
