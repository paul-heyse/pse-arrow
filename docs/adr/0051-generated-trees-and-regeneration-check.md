---
id: ADR-0051
title: Extend the generated-tree list and make codegen --check a regeneration-equivalence check
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-52, DM-53]
blueprint: [§4.2, §24.1]
review: not-required: an extension of ADR-0031 inside its accepted convention; the generated set grows and the check gets stronger, neither changes where generated code lives
evidence: Proposed
supersedes: []
superseded-by: null
revisit: the Ipopt bindgen arm starts running its generator in CI's solver container, or the hand-written Rust manifest struct is generated (register row R-27)
verification: `just codegen-check` (`cargo xtask codegen --check`); `tests/governance/tests/codegen_regeneration.rs`

---

# ADR-0051: Extend the generated-tree list and make codegen --check a regeneration-equivalence check

## Context

ADR-0031 commits generated sources and checks them with `git diff --exit-code` plus an untracked-file check. That is hygiene: it proves the working tree is clean, not that re-running the generator reproduces the committed tree. Wave 1 adds two generated artifacts blueprint §4.2 implies but does not name — the serde document structs `pse-authoring` parses package documents into, and the `docs/generated/` layout — and phase 0's deferral of regeneration equivalence (register row R-20's neighbourhood) ends when the generators exist.

## Scope

Extends ADR-0031's generated-path list and strengthens its check. It does not move a generated path, does not change the `@generated` header convention and does not reintroduce build-script generation. The Ipopt bindgen arm keeps its phase-0 behaviour until its generator can run.

## Drivers

Correctness: a committed generated tree that the generator no longer reproduces is a silent fork of the authority. Reviewability: a document struct that is hand-written beside the registry's `DocumentSpec` is a second column list, which `no_shadow_structs` exists to forbid. Determinism: a two-way diff catches a generator that drops a file as well as one that adds one.

## Options

Keep hygiene-only checking — rejected: it cannot distinguish a current tree from a stale one. Regenerate in place and diff the working tree — rejected: a failing check would leave the checkout modified. Check only one direction — rejected: a generator that stops emitting a file would pass.

## Outcome

**Generated paths** are ADR-0031's four plus `crates/pse-authoring/src/generated/documents.rs` — the serde document structs generated from the registry's `DocumentSpec`, consumed by `pse_authoring::document::load_package`. `docs/generated/` has a declared layout: `README.md`, `relations/<ns>.md`, `enums.md`, `extension_types.md`, `passes.md`, `rules.md` and `schema/authoring.schema.json`, each carrying the `@generated` marker.

**`cargo xtask codegen --check`** regenerates every target into a temporary directory and diffs it against the committed tree **in both directions** — a file in one and not the other is a failure either way — and keeps ADR-0031's untracked-file check. It never writes into the working tree.

**`python/pse/contracts/GENERATED.sha256`** is written by `cargo xtask codegen --only python`, not by a Python script: one generator owns the tree and its manifest.

**The bindgen arm** keeps a hygiene-only diff for `crates/pse-ipopt-sys/src/bindings.rs` until the generator runs inside the solver container (`--only bindgen` without `IPOPT_DIR` exits 2 with a notice). That exception is named here so it is not read as regeneration equivalence.

**The Rust `Manifest` struct** in `pse-catalog` is hand-written in phase 0 and guarded by a parity test against the registry's `ManifestSpec`, which is the authority for the generated Python `msgspec` struct and the documentation. Generating it is register row R-27.

### Consequences

`codegen --check` costs a full regeneration instead of a `git diff`, so it is a `local` recipe and a CI job, not a pre-commit hook. Every generator must be deterministic — no `HashMap` iteration reaching output — because the check now compares two runs, not a run against a commit.

### Compensating controls

`crates/pse-schema/tests/codegen_determinism.rs` asserts two runs are byte-equal before the tree is ever written; `tests/governance/tests/codegen_regeneration.rs` runs the two-way check inside `just test`; `no_shadow_structs` keeps a hand-written mirror of a generated struct out of the tree.

### Confirmation

`just codegen-check` and `just governance`; the `rust / codegen-diff` job runs the same command. A failure names the differing path and the direction.

## Pros and cons

A stronger check costs build time on every run; the weaker one cannot tell a current tree from one whose generator changed underneath it, which is exactly the failure ADR-0031 was written to prevent.

## More information

Blueprint §4.2 (what is generated), §24.1 (test layers), §20.2 (the manifest), §22.1 (authoring documents); ADR-0004, ADR-0031; register rows R-20 and R-27; [plan 03](https://github.com/paul-heyse/pse-arrow/blob/8950dd3d6ddb3aa7c78acc0db7d0601497b302a8/docs/plans/03-wave-1-foundations.md) packets R-1, A-6, A-8, A-9 and R-2.

## Status history

- 2026-09-13 — accepted. Evidence is `Proposed`: the generated-path list and the check contract are written; `codegen_regeneration` lands with packet R-2.
