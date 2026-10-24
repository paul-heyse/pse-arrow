---
title: "Plan 54b: Cargo freshness and dependency graph"
status: in-progress
date: 2026-10-24
adrs: []
review_sources: [docs/design_review/reviews/design_review_rust-build-reuse_2026-10-24.md, docs/functional_reviews/rust-build-reuse-tooling-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/54-rust-build-reuse.md
doc_topics: [current-work, environment, preparation, validation, libraries]
---

# Plan 54b: Cargo freshness and dependency graph

## Target and foundation

**Proposed:** Cargo avoids redoing unchanged Rust work, manifests describe intended source
and feature relationships, and one inexpensive routine detector maintains that accuracy.
The [coordinator](54-rust-build-reuse.md) owns F04–F06/CB10, scenarios S04/S05 and source
recommendation dispositions. This companion owns its packages and acceptance.

**Interface-checked:** .cargo/config.toml supplies workspace feature unification to all
Cargo consumers; Cargo.toml keeps edited workspace crates incremental and dependencies
non-incremental. Existing arrow_validation routes verify actual selected target/host/build/
test-dev leaves. Retain those authorities. Removing a declaration is not proof of a smaller
compiled graph, and parsed source analysis is not compiler name resolution or macro expansion.

## B01 — Use content freshness without weakening invalidation

Add both supported Cargo configuration keys:

    [unstable]
    checksum-freshness = true

    [build]
    fingerprint = "content"

Merge them into existing tables; retain jobs, resolver, fine-grain locking and target
rustflags. The actual pinned Cargo documents this pair in its
[exact source reference](https://github.com/rust-lang/cargo/blob/3d7cf6e937d6127d0f49881bf689c560b36d35c4/doc/book/src/reference/unstable.md#checksum-freshness).
Do not substitute an environment toggle, disable incremental globally or move artifacts
into a shared build directory. Content freshness changes how Cargo detects input changes;
it does not establish safe cross-checkout artifact sharing or remove sccache path inputs.

Extend scripts/installation_context.py CONFIG_KEYS to admit both declarations and retain
them in the existing parsed configuration identity. Unknown keys continue to refuse. Test
that changed fingerprint policy changes the consumed context, while equivalent formatting
does not. Check the existing governance minimal-config assertion and actual Python producer
selection for compatibility; update a specific assertion only to admit the intended keys,
not to loosen unrelated configuration. No new general Cargo-equivalence interpreter is needed.

All invocations reading repository configuration consume the same policy: recipes, bare
Cargo on the pinned nightly, rust-analyzer and maturin. Build-script rerun-if-changed inputs
still use mtimes. Keep rerun-if-env-changed, native receipts, generated inputs and the existing
write-if-changed behavior; do not describe this change as eliminating all touch-induced work.
Expect a one-time refresh of affected Cargo fingerprints, not a cache flush or toolchain bump.

**Acceptance, proposed:** focused installation-context/build-configuration controls plus a
small real Cargo fixture containing ordinary Rust, generated included Rust, and a declared
build script. Identical-byte touch/restore causes no Rust recompile where content freshness
applies. A real edit, a same-length edit, changed included bytes, and changed declared build
script file/environment inputs rebuild the affected unit and change observed output. Compare
the incremental result with clean recomputation for those cases. Preserve and explicitly
observe the build-script mtime limitation instead of expecting zero work for every touch.
Use a probe-owned target/cache and no live checkout stash/restore manipulation.

Join one representative workspace selected correctness route, a normal rust-analyzer
refresh and interactive/correctness Python installation in D02. The original review's touch
timing is historical evidence for the diagnosis, not current proof of negative invalidation.

## B02 — Correct dependency ownership and inspect manifest removals

Fix impl_diagnostic! in pse-diagnostics by exposing a doc-hidden re-export of its miette
dependency and using $crate-qualified references inside the exported macro. Consumer crates
can then stop declaring miette solely to satisfy expansion. Retain direct miette declarations
where their own source or public diagnostic implementation genuinely uses it. Exercise the
macro from another crate without a direct miette dependency so the test can detect leaked
name resolution. Public error/diagnostic behavior remains unchanged.

Review shear's exact diagnostics against actual source, expansion where needed, build scripts,
generated code, feature activation and declared targets. Remove verified unused declarations,
now-unnecessary lock edges and the genuinely empty orphan identified in the review. The review
reports 25 unused declarations and prototype compile success, but also five wrong removals
from blind --fix. Do not use the reported count as authority or repeat automatic deletion.
In particular, retain linked tests/engine/src/oracle.rs and xtask/src/canonical_recovery.rs;
their unlinked reports were false positives.

Use conventional layout for genuinely single-owner modules when it improves ownership.
Keep deliberate parent-relative shared fixtures and scripts/workspace.rs rather than moving
them solely for the checker. cargo-shear 1.14 has the confirmed
[path-resolution limitation #589](https://github.com/Boshen/cargo-shear/issues/589).
Use precise package-level ignored dependencies and ignored-paths only for diagnosed cases,
with the reason and upstream issue beside them. The redundant-ignore check should expose
obsolete exceptions after an upstream fix. Keep the legitimate optional arrow-cast
force_validate activation, explaining its role under ADR-0170. The overview's
keep-path-dependencies option concerns predeclared workspace crates; it does not repair
external path-module analysis and is not selected as a blanket escape hatch.

For force-validate forwarders, compare actual before/after Cargo units per removed edge,
including selected-root dev dependencies and host/build contexts. Judge the conformance
prototype and remaining benches/runtime/engine/governance/lifecycle candidates individually.
Preserve intended support features and explicit validated Arrow leaves. Equal features on
packages common to two graphs are insufficient by themselves: also inspect added/removed
units, roles and changed dependencies. Reuse the existing Arrow proof and unit-graph tools.

Inspect test-target diagnostics before setting test=false or doctest=false. Check macro-
generated tests, inline and external modules, feature-specific tests, real documentation
examples and the target's public role. Disable only genuinely empty harnesses; preserve active
tests/doctests and legitimate public examples. Do not add speculative future dependencies.
Retire replaced declarations/files/callers with their coherent change once selected tests pass.

**Acceptance, proposed:** external macro expansion and diagnostic behavior controls; selected
compiles for changed crates and default/native conditional consumers; actual unit/feature/
validation comparisons for forwarder changes; and strict shear at zero errors and warnings.
Use just check and selected just check-package routes for the affected default closure,
plus just check-native-package pse-runtime native-solvers and affected backend selections
when those declarations change. Select actual feature-gated canonical/native test code that
made udeps findings false; ordinary --all-targets does not enable every feature. Run the
remaining real doctests/tests through existing correctness-aware recipes. D02 owns their
assembled interaction and scope-end feature qualification, not a new campaign per deletion.

## B03 — Make one routine detector strict and retire obsolete routes

After B02's inspected cleanup and A01's exact shear selection, route the hygiene leaf through:

    cargo shear --locked --check-test-targets --deny-warnings --format json

The exact [1.14.0 implementation](https://github.com/Boshen/cargo-shear/blob/e789f424d9b780d207c06342c63ef95609389fd6/src/lib.rs#L325-L335)
only fails warning findings when --deny-warnings is supplied. Existing audit_tools finding/
execution-error classification remains useful; ensure the strict hygiene group preserves
nonzero findings, rather than advisory deps-report semantics turning them into success.
The check belongs at functional scope end, not turn-end, an edit hook or every compile.

Retire audit-machete, just udeps, their installed-tool requirements and obsolete references/
tests that only protect those retired routes. Preserve cargo-deny/cargo-audit and their
different security/policy responsibilities. deps-report may continue to expose shear under
its documented advisory bundle contract; hygiene owns strict acceptance. Migrate consumers
together and delete replaced routine routes without compatibility shims.

The reason is useful coverage at low cost and avoiding demonstrated unnecessary work/hidden
metadata writes. Multiple detectors are not inherently multiple semantic authorities.
Shear --expand or compiler-assisted analysis remains available as an explicit investigation
when macros/generated code make parsed-source conclusions doubtful. A one-configuration
compile does not prove all optional features or platforms safe, nor does static cfg-agnostic
analysis prove every expansion or build-time use.

**Acceptance, proposed:** focused audit_tools/validation-scope tests exercise no findings,
warning-only findings, errors, processing failure and the distinction between strict hygiene
and advisory deps-report. Verify the assembled command uses both --check-test-targets and
--deny-warnings. Confirm no invocation edits manifests or vendored lockfiles. A scan of
source consumers finds no active retired routine route/tool requirement. Run the actual
strict shear command at B02/B03 completion; D02 supplies the broader scope-end hygiene run.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| B01 | complete | 54b | Both freshness keys consumed consistently and real positive/negative invalidation controls |
| B02 | complete | 54b | Hygienic macro, inspected graph/target cleanup and preserved validation/source behavior; assembled qualification remains D02 |
| B03 | complete | 54b | Strict pinned shear, truthful group exits and retired obsolete routine detectors |

B01 is independent of B02. B03 consumes A01's selected shear and B02's clean basis. Do not
infer a speedup from declaration count or sum individually measured empty doctest timings.
**Proposed:** performance benefit is measured in 54d after the combined behavior is available;
all commands above describe future acceptance, not tests run during plan authoring.

## Execution checkpoint

B01's real pinned-Cargo fixture passed 12 scenarios; seven changed-result cases matched
clean recomputation. Same-length preserved-mtime edits to source, includes and a local
dependency invalidated correctly. Identical source touches/restores launched no rustc.
A declared build-input touch reran its script and compiled the consumer despite generated
write-if-changed, preserving the explicit mtime limitation.

B02 and B03 are integrated. Inspected cleanup removes 42 direct lock edges without moving
package versions, including eight unnecessary validation forwarders. Per-edge and final
unit-graph/Arrow proofs pass; strict shear reports zero findings. The external no-direct-miette
consumer passes three actual behavioral tests. Joined default/native/optional consumer
compilation and scope-end qualification remain in D02. The original external fixture's
related-diagnostic assertion was corrected: forwarding a simple leaf does not invent a
miette related iterator; the product implementation did not need changing. Native scope-end
compilation caught two incorrect serde removals: the conformance and benchmark consumers
load a shared scientific-input fixture outside their target directories. Those declarations
and precise issue-589 exceptions are restored; native contract compilation and strict shear
now pass. The initial 44-edge count is superseded by the corrected net 42.
