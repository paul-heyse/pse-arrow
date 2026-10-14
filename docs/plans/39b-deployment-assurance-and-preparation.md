---
title: Deployment assurance and preparation
status: in-progress
date: 2026-10-14
adrs: [ADR-0164, ADR-0183, ADR-0189]
review_sources: [docs/design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md]
doc_owner: docs/plans/39-implementation-friction-and-codex-effectiveness.md
doc_topics: [architecture, preparation, validation, environment]
---

# Plan 39b: Deployment assurance and preparation

## Purpose and boundary

This companion to [Plan 39](39-implementation-friction-and-codex-effectiveness.md)
owns the corrections for the friction review's [F01](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f01),
[F02](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f02),
[F03](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f03)
and [F06](../design_review/reviews/design_review_implementation-friction-and-codex-effectiveness_2026-10-14.md#f06).
Worker and imported Python startup must consume the same owned observation operation;
completed strict role captures must remain independently reviewable after later role
preparation; installation must consume the selected Python realization; and nested build
diagnostics must reach the existing live log consumer. Producer assurance must state
which guarantee its actual consumers require before choosing a narrower mechanism.

The coordinator owns rule decisions and finding dispositions. This document owns its
packet state. [Plan 39d](39d-qualification-and-plan38-cutover.md) owns final assembled
qualification, including every remaining Plan 38 obligation, after the selected new
Plan 39 functional work and investigations are complete.
This draft schedules work; it reports no new executed product checks or acceptance.
Blueprint §5.3 owns producer identity and the separate deployment-local guarantee,
blueprint §14.3 owns preparation/reconstruction trust and blueprint §24 owns the
qualification boundary; this plan changes their realization through the decision route.

## Current foundation and focused assessment

Authoring source baseline: `36f15656d`. The review's diagnosis was made at `d5e8606bb`;
its retained historical results remain evidence for that scope, rather than qualification
of the authoring baseline. The source routes below were inspected while preparing this
document. Core 3.4 and Efficient Architecture 1.0 support changing the physical operation
boundary while keeping scientific identity, deployment compatibility, exact artifact
association and outer provenance distinct. Process Simulator 1.5's scientific checks
remain with their existing owners; an associated executable never proves physical or
numerical validity.

| Current owner | Established foundation and remaining gap |
|---|---|
| `crates/pse-runtime/src/math/portable.rs` | `observe_deployment_runtime` already admits receipt/namespace work, checkpoints the original stop/deadline and joins native work. Its retained lease currently covers the complete startup work envelope rather than only escaped observation data. `observe_local_runtime` already has retained-result accounting. |
| `xtask/src/bin/support/deployment.rs` | Actual worker root verifies its executable, but calls synchronous deployment observation, receipt reading and artifact verification before creating the local-only startup interrupt owner. The strict branch bypasses the owned runtime operation. |
| `crates/pse-py/src/workflow.rs::open_recording` | The actual imported root already calls the owned deployment operation and independently verifies the loaded image. Its startup observation lease lives through canonical connection/open. |
| `xtask/src/producer_identity.rs` | Actual selected unit graph, compiler dep-info, context, source consistency and reviewed executor channels support fail-closed strict capture. `macro_caller_input_basis` deliberately includes generated suppliers' compilation closures. Whole-caller grants repeat manual I/O review after unrelated consumed code changes. |
| Same capture owner | `selected_build_output_observations` preserves selected logs and `.d` files under build-script output directories. It does not preserve every rustc unit dep-info and generated caller input needed by independent reconstruction. Later package-family cleanup can remove them. |
| `scripts/producer_deployment.py` and `scripts/python_extension.py` | Capture selects each role and validates actual association. Correctness installation separately invokes Maturin; Python capture later builds independently. `bind_python_artifact` already independently imports the module and checks exact admitted RPATH replay. |
| Capture and validation logs | The outer capture streams to role logs, and validation/result readers already consume live logs. The selected Cargo build inside `producer_identity::run` uses `Command::output()`, withholding child diagnostics until completion. |

The [product inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/product-inquiry.md)
and [tooling inquiry](../design_review/evidence/implementation-friction-and-codex-effectiveness-2026-10-14/tooling-inquiry.md)
supply the wider observations. Their useful distinctions survive this plan: no permanent
Python startup reservation leak was established; increasing a truthful namespace bound
does not itself violate ownership; the historical numeric LLVM suffix cause remains
unproved; and a successfully reproduced warm installer transformation does not prove a
fresh compiler realization. Ordinary development uses the current effective environment.
Historical evidence does not authorize silently restoring an earlier PATH or wrapper.

## Rule decisions and producer guarantee comparison

| Review item | Decision at plan authoring | Route and dependent work |
|---|---|---|
| RC01 | **Accepted**, 2026-10-14, after comparison: retain ordinary deployment-local compatibility and explicit selective qualification, including the remaining strict Plan 38 campaign, with a bounded sound-binding correction. | B1 resolves technical binding uncertainty before B5 changes assurance. Unknown channels remain conservative; no sound correction returns a concrete choice to the operator with F02 open. Any strict reviewed-capture interpretation change follows the coordinator's ADR/design route; no implicit reinterpretation of historical products or remaining Plan 38 scope. |
| RC02 | **Accepted**, 2026-10-14: one effective preparation boundary and protected artifact/evidence lifetime, with installation consuming the selected realization. | B3/B4 implement the accepted boundary; coordinator routes the enduring ADR-0164/qualification explanation through the normal decision process. Physical evidence carrier is implementation judgment. Repeated executor refresh removal was not accepted automatically. |

ADR-0164 already separates opaque deployment-local compatibility from reviewed selective
producer qualification; ADR-0183 preserves actual native import/ABI/build-generation
qualification and makes optional disk memo persistence depend on a named consumer. The
question is which supported consumer needs which guarantee, rather than whether a hash
can be made less sensitive.

| Alternative | Actual named consumers and supplied guarantee | Required work, cost and consequence |
|---|---|---|
| Keep selective cross-build producer equivalence | Explicit `QualifiedProducer` publication/reconstruction, the requested Plan 38g all-role strict capture/imported-association campaign and `canonical_deployment_actual_receipts_enforce_selected_role` consume reviewed actual producer identity. It permits relevant-source compatibility across differently built artifacts when the complete contract establishes it. | Preserve exact code identity; develop a sound qualified macro/helper and invocation-channel binding if F02 is to close by narrowing review invalidation. Unknown/generated channels cannot be waived. Ownership repair B2–B4/B6 still applies. Retaining whole-caller grants preserves strict behavior but leaves F02's manual cost open. |
| Use the existing deployment-local guarantee for ordinary persisted preparation | `NativeRuntime::open_recording` and worker `deployment::open` already select `ReplayAdmission::Local` when no receipt is selected. Canonical publication/acquisition consumes the versioned `ReplayAdmission` boundary. No additional ordinary consumer requiring cross-build equivalence was established by inspected source. | Exact receiving artifact/interpreter, executable-backed mappings and effective configuration must agree. A rebuild can invalidate reuse even if relevant scientific source is unchanged. Keep explicit strict entry and the remaining Plan 38 campaign. This changes the promised reuse scope if selected in place of the stronger guarantee; it cannot be described as equivalent selective assurance. |

**Operator-selected RC01 target, 2026-10-14:** retain the already supported
deployment-local route for ordinary persisted preparation and explicit `QualifiedProducer`
for the current strict campaign, including every remaining Plan 38 obligation. B1 investigates
a sound narrower macro external-I/O assurance binding for that **current strict consumer**;
B5 implements it only after its consequential design premises are settled. Ordinary Local
is already implemented and cannot itself remedy F02. If no sound narrower binding is found,
retain conservative strict refusal and bring a concrete alternative guarantee/scope decision
to the operator. Retaining manual cost leaves F02 open. Acceptance selects this bounded
correction route; it does not establish a sound narrower binding before B1 investigates it.

B1 must make the open selective design consequences concrete. Paste environment selectors
can be produced by another macro or helper under unchanged immediate caller text. Pest
grammar selection depends on arguments, generated invocations, preferred-path absence and
actual compiler consumption. Owner/helper executable closure, selected unit feature/cfg
context, generated emitters and argument suppliers are changing premises. A syntactic
`Env`/`grammar` search, normalization of caller bytes or a larger allowlist does not establish
their completeness. A bounded investigation must either define and independently attack a
sound channel binding, with explicit unsupported cases that refuse, or retain conservative
whole-caller review. It must not predeclare the binding implemented or sound.

Bound B1 to the Paste/Pest owner/helper contracts and their actual selected runtime, worker
and Python unit closures in the current strict campaign. Trace selecting arguments,
generated emitters and feature/grammar premises only as far as needed to establish those
channels. Its deliverable names the exact supported invocation forms, necessary supplier
closure, invalidation conditions, independent counterexamples and conservative unknown
case. It does not build a general Rust macro interpreter or another source-review service.
If the supported forms cannot be closed soundly, report that concrete missing premise and
the affected strict consumer; the operator can then decide whether to retain the cost,
constrain supported forms or explicitly change the guarantee/scope.

## Target contracts

### Owned startup observation and retained results

The actual composition root supplies loaded artifact identity, actual role, selected receipt
and one original startup deadline/cancellation owner before substantial observation starts.
Worker strict and local branches use `MathService::observe_deployment_runtime`; local
replay admission remains its separate receiving guarantee. Python uses the same operation
while keeping loaded-image checks before and after observation. Store connection/open
remains an explicit subsequent effect with its actual cancellation/acknowledgement owner.
Observation does not mint store permission or publish readiness.

The runtime preflights receipt extent and admits complete work, then reads, parses and
verifies through the checkpointed native owner. Malformed/unknown/changed receipts, role
mismatch, changed loaded bytes/current inputs, insufficient allowance, expired deadline or
cancellation cannot mint a producer or readiness. Preserve the distinction between a
supported unqualified outcome and an operational error; do not turn every startup failure
into `None`. Strict receipt selection must not fall back to local permission after refusal.

Use the existing retained-result job capability to release receipt/namespace scratch only
after actual native completion/drain. Retain only escaped observation strings/hashes and
optional producer data under their actual allocation owner. A cancelled future requests
stop and keeps work ownership until join; finishing the startup signal listener does not
prove that native work drained. Python's retained result should survive connection/open
with this smaller result lease; the former full work-envelope lease is removed. Preserve
original typed stop cause and actual loaded-root safety restrictions.

### One preparation operation, exact evidence closure

The deployment preparation owner composes existing environment/native/build/install owners.
Before effects it selects role, package/target/platform/profile/features, provider
reuse/rebuild intent, Rust wrappers and tool executables, PyO3 SDK/configuration and
role-specific extension linking. This description is derived from the owners, not another
writable environment authority. Read-only description must not build providers. Deliberate
explicit overrides remain visible; unsupported contexts refuse precisely.

Select **owned copies of the consumed volatile evidence and selected role artifacts** as
the initial physical carrier. Seal them in the existing capture's private evidence output
after the existing before/after consistency checks and before the next role can refresh
their mutable build paths. This directly serves independent review and retry without
copying the full target tree. A staged generation could instead protect all selected
outputs through installation/review, but it imposes a longer shared build-tree lifetime
and stronger coordination over each role's actual contexts. Reopen that choice only if
consumed evidence cannot be captured faithfully or artifact dependencies genuinely require
the staged lifetime. Neither alternative introduces a permanent service, new registry,
alternative target directory or custom Cargo scheduler.

The completed capture must preserve a closed, independently readable view containing:

- Actual selected unit graph/keys/edges, role/profile/feature/platform context, compiler
  command/configuration observations and relevant nonsecret effective environment evidence.
- Each compiler dep-info file actually used for unit/artifact association, with its exact
  raw bytes, original path and association linkage. Include every consumed generated caller
  input required to reproduce the macro caller basis, even outside package roots. Build-script
  logs alone are insufficient; aggregate Cargo rerun hints cannot replace rustc dep-info.
- Exact consumed source/body observations and their captured raw bytes where independent
  reconstruction otherwise depends on mutable/disposable material; preserve environment
  dep-info lines, qualified owner/helper reviews and caller/channel context selected by B1.
- Native/configuration search namespaces, complete member sets and explicit absence premises,
  file/link identities and relevant bytes. A list of present selected files does not prove
  that a nearer grammar/header/configuration candidate was absent.
- Protected actual role artifact bytes and raw identity, actual build terminal status,
  input consistency outcomes and authorized transformation evidence. A failed or interrupted
  role is incomplete and cannot become a successful sealed capture.

Keep original locations distinct from protected evidence locations. Independent historical
review reads the sealed raw view and reconstructs its associations without requiring a live
`.d` path. Current deployment qualification separately observes actual current artifact,
source/configuration/environment/native and member/absence premises through their existing
owners. Old bytes or a copied digest cannot grant freshness, transfer execution evidence or
prove that a current missing file is still absent. A changed premise invalidates the affected
retry; it never silently switches compilation context.

Do not overwrite completed or failed capture directories. Reuse the existing ownership,
artifact/reference and retention contracts; preserve protected completed roles while a failed
role retries, and expose any incomplete closure explicitly. Removal of selected executor
refresh is a separate assurance question: keep it wherever it establishes an otherwise
unobserved input. If implementation proposes consolidation or elimination, identify the
exact supplied fact and establish equivalent actual-input assurance before applying that
change through the coordinator's decision route.

### Installation consumes the selected realization

Strict deployment preparation selects the Python realization once and passes it to its
installation owner. Maturin remains the installation/transformation capability; the
integration must prove that the selected raw artifact is the one installation consumes.
If installation recompiles or selects another context, it must capture and select that
actual realization before association, or refuse. A warm all-Fresh install and a compiling
install are different evidence cases. Neither a sibling capture nor a configuration
filename proves the installer's actual effective context.

At the installer boundary preserve actual Cargo child argv and working directory, selected
nonsecret effective SDK/configuration inputs, relevant wrapper/tool/provider selection,
role/profile/features, raw artifact identity and installed identity. Secrets or arbitrary
environment dumps are unnecessary. Keep the existing independently observed import path
and full-byte association after the admitted RPATH-only transformation; replay the declared
transformation on an owned raw copy and require exact installed bytes. Reordered authorized
appended paths retain their current exact multiplicity/prefix contract. Numeric LLVM
suffixes, matching text sections or stripped-symbol equivalence are not admitted substitutes.
The install owner must refuse a different checkout, module or transformation.

### Diagnostics reach current consumers

The nested capture build emits named preparation/configuration, refresh, build, sealing and
association transitions to its existing diagnostic stream. Forward Cargo stderr as it is
produced and preserve exact raw stdout JSON for evidence parsing; tee stdout only where an
existing consumer needs it. Never mingle human stage text into the machine evidence stream.
Keep final child status and refusal details even when output is partial. Drain/reap owned
children and close logs on cancellation or failure before releasing process/evidence owners.
A quiet link is not a new hang inference or deadline policy. Existing role logs, validation
live output and result readers remain the consumers; no dashboard or event store is needed.

## Execution packets

| Packet | Prerequisite | Delivered boundary and verification | State |
|---|---|---|---|
| <a id="b1"></a>B1 — Settle selective channel-binding uncertainty | Accepted RC01 two-scope target; baseline source and named consumers remain available. | Retain the selected guarantees and their consequences. Investigate qualified owner/helper plus actual generated invocation/channel closure for the current strict campaign. Produce a sound bounded binding with revealing controls, or preserve conservative refusal and return a concrete guarantee/scope choice to the operator with F02 open. The technical decision constrains B5; no speculative binding lands first. | Done |
| <a id="b2"></a>B2 — Common startup owner and scratch lifetime | Existing `observe_deployment_runtime`, retained-result job and actual role verification contracts. | Migrate worker strict/local startup and Python retained result onto one admitted operation; install the worker startup signal owner before observation, keep store effects separate, retain only result bytes after native drain. Targeted runtime and actual composition controls below. Delete worker synchronous receipt/observation composition and full-envelope result retention after replacements pass. | In progress |
| <a id="b3"></a>B3 — Protect complete role evidence and artifacts | Accepted RC02 boundary; existing graph/dep-info/review/current-input interpretation. B1's final grant interpretation is needed only for changed assurance fields. | Seal consumed volatile bytes and exact artifact association independently of mutable Cargo paths. Migrate independent caller review and capture validation to the closed historical view while retaining separate current verification. Targeted cross-role refresh/reconstruction, tamper, missing-member and interruption controls. Delete review dependence on mutable captured `.d`/generated-input paths. | In progress |
| <a id="b4"></a>B4 — Select and install one actual Python realization | B3 protected role product and exact context; existing native/environment/install owners. | Describe effective preparation before effects, feed selected realization through installation, retain actual installer child context and raw/transformed/imported identities. Targeted warm/compiling-context, override, exact RPATH and mismatch controls. Remove strict install-first/independent-recapture ordering when all strict consumers move; ordinary editable sync retains its own ordinary behavior. | In progress |
| <a id="b5"></a>B5 — Apply the operator-selected assurance route | B1 explicit guarantee choice and any required coordinator ADR/design route; B3 preserves the evidence that route consumes. | Implement only the sound bounded binding B1 established for the current strict consumer, with generated selector, helper, feature and grammar-shadow refusal controls; retire whole-caller grant invalidation only where replaced. Preserve existing ordinary Local and explicit strict entry. If investigation fails, dependent implementation waits for the concrete operator decision; neither retaining strict grants nor restating the existing Local default resolves F02. Every remaining Plan 38 strict obligation survives unless expressly changed. | In progress |
| <a id="b6"></a>B6 — Stream nested child diagnostics | Existing role log/result readers and capture stdout parsing contract; independent of B1–B5. | Forward diagnostics before child exit, preserve identical raw stdout evidence and terminal status, expose meaningful stages and authentic child drain. Targeted controlled-child visibility, malformed stdout, nonzero exit and cancellation controls. Delete whole-build diagnostic buffering after consumers move. | In progress |

B2 and B6 can proceed independently of producer-guarantee choice. B3 can preserve the
current strict evidence contract before B1 settles any changed channel interpretation.
B4 requires a usable protected role product, rather than only an agreed schema. B5's rule
change depends on the explicit decision and actual evidence/binding, not completion of
unrelated startup work. The same capture files are touched by B3/B6 and the same composition
roots by B2/B5; the implementation coordinator assigns edit ownership even when logical
dependencies permit separate work.

## Planned verification and completion

All verification here is **Proposed**, with zero failures as its baseline. During packets,
compile touched packages through `just check-package` and use targeted unit selections,
with the repository's explicit force-validation routes for actual Arrow-consuming closures.
No package claims assembled product qualification. Select actual test filters at execution
from current source; named existing controls below are preservation anchors, and new controls
belong with the new mechanism rather than tests for deleted implementation details.

| Boundary | Revealing targeted evidence |
|---|---|
| Startup, B2 | Existing runtime startup admission/receipt controls plus real worker composition: interrupt/deadline during receipt/namespace observation, large/malformed/changed receipt, insufficient allowance and role mismatch; no readiness/producer on refusal; native join before scratch release; smaller result ownership through slow/failed connection. Python keeps independent before/after loaded-module checks. |
| Closed evidence, B3 | Capture worker, prepare Python with the actual selected executor-family refresh, then independently reconstruct worker caller/artifact association with original mutable `.d` and generated-input paths unavailable. Missing/corrupt raw member, substituted unit association, namespace growth/new shadow and concurrent source/configuration changes refuse; interrupted sealing remains incomplete. |
| Installer, B4 | `scripts.tests.test_producer_deployment` and `scripts.tests.test_python_extension` extend their actual selection/association controls: supported override, warm reuse, actual compiler-child context change, wrong checkout/module, exact admitted transformation and extra-byte mutation. Actual imported-byte journey is finally run by 39d. |
| Assurance, B5 | Preserve `producer_dynamic_macro_review_binds_actual_caller_inputs_and_environment`, `producer_pest_preferred_grammar_revokes_review_and_requires_actual_consumption`, actual selected-owner review and unknown-I/O refusal. The narrower selective route must distinguish unrelated caller-body edits from changed generated selectors/helper closure/feature context/grammar presence. Preserve exact local receiving configuration/role checks and separate strict/local keys, without promoting historical unqualified products. |
| Diagnostics, B6 | A controlled child emits stderr then blocks until the outer reader acknowledges receipt: the diagnostic must appear before exit. Complete raw stdout JSON and child status remain exact. Failed/cancelled children preserve partial output, original stop and actual drain; existing live result consumers read the same role log. |

Tests reconstructing the sealed view should use raw independently retained compiler evidence,
not expected identities generated by the changed sealing function itself. Full current
artifact verification and imported Python association remain separate from that historical
reconstruction. Preserve `canonical_deployment_actual_receipts_enforce_selected_role` for
the strict campaign and both explicit strict/local receiving-acquisition reconnect controls.
Changes to effective native context require the affected native/import checks; the plan
does not infer native completeness from scripted installer unit tests.

At functional scope completion, 39d runs the relevant assembled integration/component,
native/managed/Python, performance and static/manual qualification once, with all remaining
Plan 38 obligations, actual strict captures and independent scientific assessment still
selected unless the operator expressly changes them. Each result names command, mode,
scope, conditions and zero baseline; failed, blocked, retained and fresh evidence stay
distinct. Quantitative startup/build/token savings require a named measurement and are
not prerequisites to correcting the source-established lifecycle/observability defects.

Completion requires migrated actual consumers and deletion of replaced composition,
mutable-review dependencies, independent strict install/capture selection and diagnostic
buffering within the chosen scope. The coordinator resolves F01/F02/F03/F06 only from their
respective closure evidence, not packet scheduling. Enduring deployment/qualification
meaning moves to its authoritative owners through the confirmed decision route, then this
companion retires with the series under ADR-0096. The eventual Outcome will distinguish
implemented/tested/measured claims, a mistake corrected and deliberate deviations; none is
invented at draft stage.

## Selected execution checkpoint — 2026-10-14

The authorized execution slice is B1–B6 and C1–C5, including their supporting production changes. 39a and the full 39d/remaining Plan38 campaign remain deferred. B2 startup ownership and retained-result changes, B3 protected historical evidence, B4 actual-Maturin-child capture and B6 raw streaming are integrated; targeted acceptance and actual composition probes are being completed.

B1's real pinned-rustc Paste probe demonstrated generated environment selection absent from compiler dep-info. Stock Paste has no pure switch; stock Pest cannot provide the current py_literal file workflow through a pure entry point. The operator explicitly selected guarded pure owners on 2026-10-14. [ADR-0189](../adr/0189-pure-macro-producer-channels.md) and the [independent bounded target review](../design_review/reviews/design_review_plan39-pure-macro-channels_2026-10-14.md) precede dependent assurance changes. Paste refuses every Env segment before access; Pest validates actual literal inline attributes and calls the existing generator with inclusion disabled; py_literal preserves its exact published grammar in one inline authority. Tokens-only includes supplied compiler token/span context and still requires current exact source/helper/feature review. Relevant caller code remains scientific identity. Unknown or effectful owners retain conservative rules. Exact local transitive source pins are governed by matching vendor manifests and selected lock entries; replacing the old blanket patch ban is recorded as RC03 in the bounded review.

Independent implementation review found and corrected missing cross-root alias/context evidence, a dry-run observer configuration, a release-only parser regression and Rustup proxy invocation ambiguity. Historical namespace inventories explicitly retain topology/digest framing; only separately consumed raw members are copied. Historical success never proves current freshness. Final all-role current association may still refuse an earlier role after a later executor refresh; the deferred strict campaign must resolve actual current premises without rewriting originals to protected copies.
