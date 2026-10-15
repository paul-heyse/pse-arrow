---
title: Deployment assurance and preparation
status: done
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
The target below is now implemented for the selected execution slice; its bounded Outcome separates targeted evidence from 39d acceptance.
Blueprint §5.3 owns producer identity and the separate deployment-local guarantee,
blueprint §14.3 owns preparation/reconstruction trust and blueprint §24 owns the
qualification boundary; this plan changes their realization through the decision route.

## Authoring foundation and focused assessment

Authoring source baseline: `36f15656d`. The review's diagnosis was made at `d5e8606bb`;
its retained historical results remain evidence for that scope, rather than qualification
of the authoring baseline. The source routes below describe that authoring baseline; delivered behavior is recorded in the Outcome. Core 3.4 and Efficient Architecture 1.0 support changing the physical operation
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
| <a id="b2"></a>B2 — Common startup owner and scratch lifetime | Existing `observe_deployment_runtime`, retained-result job and actual role verification contracts. | Migrate worker strict/local startup and Python retained result onto one admitted operation; install the worker startup signal owner before observation, keep store effects separate, retain only result bytes after native drain. Targeted runtime and actual composition controls below. Delete worker synchronous receipt/observation composition and full-envelope result retention after replacements pass. | Done |
| <a id="b3"></a>B3 — Protect complete role evidence and artifacts | Accepted RC02 boundary; existing graph/dep-info/review/current-input interpretation. B1's final grant interpretation is needed only for changed assurance fields. | Seal consumed volatile bytes and exact artifact association independently of mutable Cargo paths. Migrate independent caller review and capture validation to the closed historical view while retaining separate current verification. Targeted cross-role refresh/reconstruction, tamper, missing-member and interruption controls. Delete review dependence on mutable captured `.d`/generated-input paths. | Done |
| <a id="b4"></a>B4 — Select and install one actual Python realization | B3 protected role product and exact context; existing native/environment/install owners. | Describe effective preparation before effects, feed selected realization through installation, retain actual installer child context and raw/transformed/imported identities. Targeted warm/compiling-context, override, exact RPATH and mismatch controls. Remove strict install-first/independent-recapture ordering when all strict consumers move; ordinary editable sync retains its own ordinary behavior. | Done |
| <a id="b5"></a>B5 — Apply the operator-selected assurance route | B1 explicit guarantee choice and any required coordinator ADR/design route; B3 preserves the evidence that route consumes. | Implement only the sound bounded binding B1 established for the current strict consumer, with generated selector, helper, feature and grammar-shadow refusal controls; retire whole-caller grant invalidation only where replaced. Preserve existing ordinary Local and explicit strict entry. If investigation fails, dependent implementation waits for the concrete operator decision; neither retaining strict grants nor restating the existing Local default resolves F02. Every remaining Plan 38 strict obligation survives unless expressly changed. | Done |
| <a id="b6"></a>B6 — Stream nested child diagnostics | Existing role log/result readers and capture stdout parsing contract; independent of B1–B5. | Forward diagnostics before child exit, preserve identical raw stdout evidence and terminal status, expose meaningful stages and authentic child drain. Targeted controlled-child visibility, malformed stdout, nonzero exit and cancellation controls. Delete whole-build diagnostic buffering after consumers move. | Done |

B2 and B6 can proceed independently of producer-guarantee choice. B3 can preserve the
current strict evidence contract before B1 settles any changed channel interpretation.
B4 requires a usable protected role product, rather than only an agreed schema. B5's rule
change depends on the explicit decision and actual evidence/binding, not completion of
unrelated startup work. The same capture files are touched by B3/B6 and the same composition
roots by B2/B5; the implementation coordinator assigns edit ownership even when logical
dependencies permit separate work.

## Planned verification and completion

The controls below were specified at authoring, with a zero-failure baseline. The Outcome records the subset actually executed and the acceptance retained by 39d. During packets,
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

The authorized execution slice is B1–B6 and C1–C5, including their supporting production changes. 39a and the full 39d/remaining Plan38 campaign remain deferred. B2 startup ownership and retained-result changes, B3 protected historical evidence, B4 actual-Maturin-child capture and B6 raw streaming are integrated; targeted checks and bounded actual composition probes are complete, with the unexamined acceptance stated below.

B1's real pinned-rustc Paste probe demonstrated generated environment selection absent from compiler dep-info. Stock Paste has no pure switch; stock Pest cannot provide the current py_literal file workflow through a pure entry point. The operator explicitly selected guarded pure owners on 2026-10-14. [ADR-0189](../adr/0189-pure-macro-producer-channels.md) and the [independent bounded target review](../design_review/reviews/design_review_plan39-pure-macro-channels_2026-10-14.md) precede dependent assurance changes. Paste refuses every Env segment before access; Pest validates actual literal inline attributes and calls the existing generator with inclusion disabled; py_literal preserves its exact published grammar in one inline authority. Tokens-only includes supplied compiler token/span context and still requires current exact source/helper/feature review. Relevant caller code remains scientific identity. Unknown or effectful owners retain conservative rules. Exact local transitive source pins are governed by matching vendor manifests and selected lock entries; replacing the old blanket patch ban is recorded as RC03 in the bounded review.

Independent implementation review found and corrected missing cross-root alias/context evidence, a dry-run observer configuration, a release-only parser regression and Rustup proxy invocation ambiguity. Historical namespace inventories explicitly retain topology/digest framing; only separately consumed raw members are copied. Historical success never proves current freshness. Final all-role current association may still refuse an earlier role after a later executor refresh; the deferred strict campaign must resolve actual current premises without rewriting originals to protected copies.

## Selected execution outcome — 2026-10-14

**Implemented:** B2 removes synchronous worker receipt/observation and store-first composition. Worker strict/local startup shares the admitted runtime owner with Python, under one original cancellation/deadline before observation. Native work joins before scratch is released; only escaped hashes/strings/result ownership remains charged through subsequent store open. Strict refusal never falls back to Local. Python keeps independent loaded-image checks and cancellation checkpoints around store effects.

**Implemented:** B3 seals a versioned historical sidecar with raw unit graph/metadata/argv/streams/status, per-unit compiler dep-info associations, consumed generated inputs including outside package roots, exact original versus protected file identities, selected source/helper reviews, protected role bytes, complete namespace topology/digest framing, cross-root aliases and entry/target absence. Only consumed raw members are copied; namespace inventories are explicitly not full raw namespace copies. The historical reader reconstructs those premises after original Cargo paths disappear. Deployment validation and RPATH extraction consume the protected context, with no live legacy-JSON fallback. Current artifact/source/configuration/environment qualification remains independent and conservative.

**Implemented:** B4 makes Maturin's actual Cargo child the sole selected Python compilation. One private single-use adapter records actual argv/cwd and allowlisted SDK/compiler/provider selections, protects explicit PyO3 configuration, and requires qualified sealed raw evidence before installation continues. Actual sole-cdylib graph validation supports Maturin's manifest-selected omission of `--crate-type`; explicit conflicting or unknown contexts refuse before build. Cargo proxy invocation identity is preserved. Independent loaded import/full SHA and exact RPATH-only replay remain mandatory, with transformation evidence retained. The superseded strict pre-install/independent-recapture ordering is removed; ordinary editable sync is unchanged.

**Implemented:** B5 installs exact pinned guarded Paste/Pest/py_literal sources and source-qualified `tokens-only.v1` review for the three current strict role contexts. Whole-caller grants disappear only for these current exact pure owners/helpers/features. Stock/unknown/changed owners remain conservative. Caller-body changes still rekey scientific identity; owner/helper/feature changes revoke the grant. Paste's unchanged compiler-version build effect retains its separate review. No native/tool completeness was inferred from observed hashes. The root local-source policy checks vendor containment, exact package/version and unique actual lock selection.

**Implemented:** B6 concurrently drains/spools exact child stdout/stderr, forwards stderr and rendered ordinary Cargo JSON compiler diagnostics before exit, and preserves real status/partial streams. Maturin receives the original JSON without duplicate local rendering. Preview limits do not validate or discard raw evidence. Existing live/result logs remain the consumers; incomplete capture never seals successful history.

**Tested, zero-failure baseline:**

- `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw just unit-package pse-runtime deployment_startup`: six passed, 491 outside the filter. Includes original expiry, cancellation/abandonment through actual native drain, receipt bounds/stop and exact retained extent.
- Force-validated worker/xtask build (`scripts.arrow_validation build -p xtask --bins --no-default-features --features package-fixtures --locked`) passed. Force-validated `check -p pse-py --no-default-features --locked` passed. These compile selected consumers, without qualifying their scientific journeys.
- Force-validated xtask Nextest filter `test(producer_streamed_child_) | test(producer_historical_) | test(producer_observed_installer_arguments_) | test(producer_tokens_only_) | test(producer_dynamic_macro_review_) | test(producer_pest_preferred_)`: 14 passed, 53 outside the filter. Includes actual tiny worker/cdylib executor refresh, generated inputs and macro caller reconstruction after original dep-info/workspace/legacy JSON deletion, plus substituted associations, tampering/absence and live diagnostic ACK controls. These captures deliberately remain unqualified for native execution.
- `RUSTC_WRAPPER= scripts/pse-env --recipe-default-class compile -- python3 scripts/tests/fixtures/pure-macro-channels/qualify.py`: 42 tests and twelve expected compiler refusals across debug/release, zero warnings/failures. The exact published py_literal grammar SHA is independently checked. This isolated pure closure has no Arrow or solver dependency and uses its own target.
- Explicit-light unittest selection for `test_producer_cargo`, `test_producer_deployment`, `test_python_extension` and `test_validation`: 90 passed. Additional correction controls cover actual Cargo jobs/color, same-inode proxy ambiguity and deleted legacy-context inspection. Expected refusal fixtures are assertions, not failing qualification.
- Force-validated `nextest run -p pse-tests-governance --test no_patch_tables --locked --no-fail-fast`: two passed. Exact local source/lock policy and existing CPU/linker constraints survive.

**Tested, actual worker composition:** an owned private profile and counting socket endpoint exercised the built worker. A malformed receipt returned the receipt parse error with zero connections. A separate 128 MiB sparse receipt was interrupted only after `/proc/<owned-child>/fd` showed its actual receipt reader open; the worker returned `math evaluation cancelled`, with zero connections. Both enclosing admission scopes completed successfully after owned process drain. Evidence is `/tmp/pse39-worker-startup-vgsfrwk_` and `/tmp/pse39-worker-cancel-m92cpz9b`. No canonical server was started or altered.

**Tested, actual installer capture/refusal:** actual pinned Maturin/PyO3 with the current capture tool compiled a small nested extension, then reused it Fresh. The command was `scripts/pse-env --resource-class compile -- .venv/bin/python /tmp/pse39-maturin-real-lo0g0_cm/run-probe.py compiling-nested fresh-nested`, with PyO3 0.29.2, the pinned Rust toolchain, isolated Python 3.14 and explicit empty Rust wrappers. Both Cargo children succeeded, each sealed historical closure independently verified (20 unit associations, 1206 protected members), and both preserved the same raw artifact/identity. Deliberately incomplete native/macro declarations made the strict adapter refuse installation; post-refusal import remained unavailable. The first probe exposed Maturin's omitted crate-type switch and was preserved separately before correction. Evidence is `/tmp/pse39-maturin-real-lo0g0_cm/probe-observations.json`. This establishes compiling/Fresh selection, raw protection and truthful refusal, not successful qualified installation.

**Interface-checked:** source-grant expansion in the current runtime, worker and Python producer-profile unit graphs selects the guarded owners without caller grants or macro-channel refusals, while overall eligibility remains false without complete native/current-input evidence. No broad native completeness grant or historical reinterpretation is minted.

Remaining acceptance is deliberately with 39d: successful qualified actual Python installation/import, all-role current artifact/input association, assembled worker/Python/store journeys, full hygiene/manual/scientific/parity qualification and selected measurements. In particular, later executor refresh may invalidate an earlier role's current inputs even though historical evidence remains intact. These results do not close the principal findings or the whole Plan39/remaining Plan38 campaign. 39a was not implemented in this slice. The source/form restrictions, review obligations and future trigger remain in proposed ADR-0189; ADR status is unchanged.

Final selected integration: after root formatting, the combined affected operational/adapter/validation unittest selection passed **207 tests**, and the same force-validated xtask filter passed **14 tests** again, including the actual omitted-crate-type regression. `just ready` and `just turn-end` returned zero failures. Documentation publication rendered/indexed **441 chapters**; native scope extraction for Plan39 reported **11 complete, 17 open, zero unknown**, with no errors. These are selected code/document checks; full 39d static/scientific qualification remains unexecuted.

## Remaining-design execution outcome — 2026-10-14

**Implemented:** the finite preparation operation replaces destructive per-role executor refresh. The proposed ADR-0189 amendment and [independent preparation target review](../design_review/reviews/design_review_plan39-preparation-operation_2026-10-14.md) precede this correction. Target-local exclusion is acquired before dependency setup; the selected capture tool is bootstrapped before the source/native baseline. The operation refreshes the bounded selected executor-family union once, then tracks actual exact executor compilations. A first context requires non-Fresh successful compiler evidence. Later Fresh reuse requires preceding compilation within this operation and unchanged dependency-complete source, reviewed inputs/environment/native/absence and original executable/output premises. Maturin's actual child context remains authoritative; unknown families refuse. Selected writer descendants retain the guard; it neither replaces host admission nor excludes arbitrary bare Cargo.

The original output/evidence premises are checked before a known executor can run again and after capture. Failed/incomplete and completed operations are terminal. Historical sidecars retain preceding compilation and its original premises before operation state commits; original-path current verification remains independent. Build-script output JSON alone is not proof that the script executed in that child. The implementation keeps full native/SDK distinctions conservatively; it does not guess a projection to make roles share a context or grant native completeness.

**Tested, zero-failure baseline:** `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env --resource-class compile -- .venv/bin/python -m scripts.arrow_validation nextest run -p xtask --bin xtask --no-default-features --locked -E 'test(producer_preparation_) | test(producer_streamed_child_) | test(producer_historical_) | test(producer_observed_installer_arguments_) | test(producer_tokens_only_) | test(producer_dynamic_macro_review_) | test(producer_pest_preferred_)' --no-fail-fast` passed **23**, with **53 outside the filter**. Nine new operation controls exercise finite sharing, terminal/concurrent ownership, guard descendants, source/native/absence/output mutation and failed children; fourteen retained controls preserve historical reconstruction, installer selection, macro qualification and diagnostic streaming. Tiny actual compiler fixtures deliberately remain native-unqualified. The force-validated `scripts.arrow_validation check -p pse-runtime -p xtask --all-targets --no-default-features --locked` also passed for the selected integrated packages before the final targeted corrections.

**Tested:** the explicit-light combined unittest selection for producer adapters, Python extension, validation, environment, activity, host admission, manager/drain observation and the two startup-status controls passed **211**. The command was `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw scripts/pse-env --resource-class light -- .venv/bin/python -m unittest scripts.tests.test_producer_cargo scripts.tests.test_producer_deployment scripts.tests.test_python_extension scripts.tests.test_validation scripts.tests.test_pse_env scripts.tests.test_activity scripts.tests.test_host_admission scripts.tests.test_native_operation.ManagerObservationTests scripts.tests.test_native_operation.DrainObservationTests scripts.tests.test_surreal_server.SurrealSupervisorTests.test_new_launch_reobserves_unavailable_listener_under_original_clock scripts.tests.test_surreal_server.SurrealSupervisorTests.test_validation_launch_keeps_published_admission_closed`. Negative fixture diagnostics were expected assertions; the enclosing test command exited successfully. No canonical server or fixture was activated.

**A mistake made and corrected:** the first 23-control run passed 22 and failed one: a tampered cached build-script executable reached Cargo before the later consistency check rejected it. The check now precedes child effects, and mutation controls require absence of a second stdout spool, status and raw build evidence. The complete selected rerun passed 23. Independent implementation review also corrected late capture-tool bootstrap, missing native review assertions and descendant guard retention.

**Deliberate boundary:** actual all-role PSE captures, successful qualified installation/import, current source/native/SDK completeness, assembled/scientific/recovery/signal qualification, scope-end static/manual checks and measurements remain **not run**, explicitly deferred by the maintainer. Actual SDK differences may still truthfully refuse cross-role reuse and require source-qualified assessment in D1; the tiny fixtures do not settle that question. This completes the remaining preparation design correction, not F02/F03 closure or 39d acceptance. Formal ADR status is unchanged.

Root `just turn-end` exited zero after integration (ADR index and formatting). Its advisory Ruff auto-fix stage printed **19 remaining diagnostics** against the zero target; that stage uses `--exit-zero`. These remain for D3's explicitly deferred static repair/qualification, rather than becoming a quality baseline or a claim of lint success. Four owned Python files received formatting and one equivalent literal-concatenation safe fix. No deferred static/manual or scientific qualification was established.
