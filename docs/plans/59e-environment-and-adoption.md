---
title: "Plan59e: Environment and adoption"
status: complete
date: 2026-10-31
doc_role: plan
doc_retention: while-dependent
doc_retention_reason: "Retained complete series at maintainer request; Plan58, Plan60, proposed ADRs and qualification references consume its evidence."
doc_retirement_trigger: owner-release
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, documentation]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Environment and adoption

Part of [Plan59](59-agent-cli.md).

**Complete:** selected delivery is closed; the [final Plan59 conclusion](59-agent-cli.md#current-state-and-next-step)
records the worktree-removal deferral and qualification limits. Earlier targets and dated
checkpoints below retain their original context; package tables and Outcomes record delivery.

## Current extension — delivered search adoption, 2026-10-31

**Implemented and Tested within the native-agent boundary:** [E06](#e06) carries the operator-confirmed [search review](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md)
adoption through actual `search`/`inspect`/`propose` deliveries. The [coordinator](59-agent-cli.md)
owns the combined target and [59m](59m-composed-rust-investigations.md) owns assembled migration
slices. E06 follows useful public delivery rather than advertising planned syntax. Completed
E03–E05 evidence remains at its original scope; fresh-client search adoption is new acceptance.
E01 is complete for creation/listing; CLI removal is explicitly deferred under R-56.

The current target incorporates the [CLI/test/runtime review](../functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md)
O01/O11. The coordinator owns its dispositions and confirmed rule decisions; historical
implementation evidence below retains its original scope. The selected simplification/testing
wave includes its operating-guidance migrations. This plan covers the environment around the CLI:
- worktrees that are ready on creation;
- Claude's orientation surfaces;
- guidance that makes `pse` the obvious first choice;
- the plan index;
- the edit guidance.

It also carries the continuing communication work that ships with every verb.

Sources: [O22](../functional_reviews/rust-agent-cli-2026-10-29.md#O22),
[O23](../functional_reviews/rust-agent-cli-2026-10-29.md#O23),
[O25](../functional_reviews/rust-agent-cli-2026-10-29.md#O25) (Claude),
[O10](../functional_reviews/rust-agent-cli-2026-10-29.md#O10) items 2–3,
[O11](../functional_reviews/rust-agent-cli-2026-10-29.md#O11) start routes, and
[O12](../functional_reviews/rust-agent-cli-2026-10-29.md#O12).

The amendment incorporates [new O07–O08](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O07),
[I08](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I08),
[I10](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I10) and the
incremental adoption of [I06](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I06)
through [59f](59f-simulation-observability-and-diagnostics.md). The package table and current
Outcomes below own delivered guidance and worktree construction. CLI removal is explicitly
deferred; earlier paused checkpoints retain their historical scope.

The library-integration review's
[I04](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I04)
revises E01's generation protection, and
[I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14)
governs E03's continuing adoption. The coordinator owns the 2026-10-30 RC01–RC04 confirmations
and the required decision route; this documentation revision authorizes no teardown or pruning.

Operator directions:
- No direnv changes; the operator's separate environment work owns them.
- Codex uses the CLI through its shell and the shared AGENTS.md. The 2026-10-31 adoption
  amendment permits the repository-local advisory Bash hook; global configuration and MCP
  integration remain outside this scope.
- Adoption comes from ease and communication, never hooks.

## Baseline

**Worktrees.** `just worktree` passes its arguments to `git worktree add`. The environment guide
(§Worktrees) leaves the rest manual:
- supply `.envrc.local`;
- prepare the host tools;
- the new checkout starts with a cold `target/` and an unprepared `target/host-tools`.

The sccache cache and native provider caches are already shared across checkouts. Each checkout
has its own `target/` (ADR-0122).

**Claude surfaces.**
- `.claude/settings.json` has the edit-guard `PreToolUse` hook. The operator's concurrent changes
  moved session environment activation to a global adapter.
- There is no status line and no orientation context at session start.

**Guidance.**
- AGENTS.md "Start here" lists `just --list`, `scripts/pse-env --explain` and `just bootstrap`.
- The environment guide, validation guide and documentation guide route through recipes and raw
  commands.
- Role contracts (`.agents/roles/*`) and process skills (`.codex/skills/*`, exposed to Claude)
  describe workflows without a CLI.
- `docs/plans/README.md` is a 32 KB narrative index that agents re-read for orientation, 70 times
  in the review's window.

**Edits.** Half of measured edits were Python `str.replace` heredocs, 200 of 512 without a match
check. `ast-grep` is installed and configured (`sgconfig.yml`, used by `lint-ast`).

## Target

- **Worktrees in one command.** `pse worktree new <name> [--base REF] [--branch B] [--warm]`:
  1. runs `git worktree add`;
  2. copies `.envrc.local` with its current behaviour and mode;
  3. prepares host tools with `scripts/bootstrap-host --from <source>`, which copies a
     fingerprint-matching immutable generation ([59a A05](59a-cli-foundation.md#a05)); copying
     preserves source/destination identity and coherent completed bytes during the copy.
     Actual child/resource ownership remains separate from collection-only claims.
     A nonmatching generation
     uses the explicit bootstrap preparation route and reports any build failure;
  4. relies on the already-shared native provider caches;
  5. registers the checkout in the store ([59a A04](59a-cli-foundation.md#a04));
  6. with `--warm`, starts a detached `pse check --workspace` ([59b B03](59b-runs-and-continuity.md#b03))
     so the first real build hits the shared sccache.

  It does not touch direnv.
  Git creation, environment copy, generation preparation, registration and optional warm launch
  have distinct outcomes. If preparation fails after Git succeeds, return the existing checkout
  and failed/pending step with a remedy; do not call it ready or delete it automatically. Another
  participant may already be using the new checkout. A failed warm launch leaves an accurately
  reported prepared checkout and the failed invocation evidence.
- **Listing and removal.**
  - `pse worktree list` shows each worktree's branch, ahead/behind, dirty state, live runs and
    last activity.
  - **Deferred target (R-56):** `pse worktree rm` would first check actual live users and
    referenced artifacts through A04/A05's
    invocation and effect-shaped generation-protection contracts, including process-held readers,
    unrecorded adapter/boundary users, retained executable paths, supervisor/finalizer obligations,
    warm runs and fixture owners. It refuses while users, unresolved
    settlement or required artifacts protect the checkout. An explicit stop-and-settle operation
    may precede removal;
    `--force` must never override that protection. Once safe, `--force` would authorize loss of dirty or
    unmerged work only. Run history remains in the store under A04's typed retention rules;
    fixture evidence, scientific data and protected artifacts do not become disposable logs.
  - Automatic host-generation collection is removed under review O11. A05 retains immutable
    generations and atomic selection until explicit administrative cleanup. Worktree removal
    is a separate destructive action: actual users, unresolved settlement and required evidence
    still protect the checkout. Preserve existing unknown claims; neither a missing PID nor a terminal
    row proves safety. Collection-only claims may retire only after A09 distinguishes them from
    actual resource/admission ownership. No current data deletion is authorized.


- **Focused orientation.** Existing passive status/help remains useful on demand. The generic
  session-start brief, status-line product and automatic handoff hooks are removed from required
  scope. B05 retains session identity; no new Claude/Codex hook is needed for these conveniences.


- **Communication** puts `pse` where agents already look, as each verb ships:
  - AGENTS.md "Start here" becomes a short task-to-verb table ("run a test → `pse test`", "wait
    on a run → `pse wait`", "exact dependency API → `pse api`", …), pointing to
    `pse commands --json` and the generated reference.
  - The environment, validation and documentation guides route their workflows through `pse`.
  - Role contracts and skills name only delivered useful commands; direct files, native Git
    and existing plan/ADR routes remain first-class. Do not recommend deferred wrappers.
  - As [59f](59f-simulation-observability-and-diagnostics.md) ships, guides show passive
    process/operation inspection, follow continuity, study history/current availability, and
    explicit capture/analysis/export escalation. Optional backend/MCP inquiries remain choices;
    ordinary shell workflows and local checkpoints work without them. Observation disconnect
    does not cancel science; acknowledgements and nonterminal waits do not assert subject success.
  - Recipes for covered work delegate to `pse` or are retired with their replacement (59c and
    59d own the retirements).
  - Envelope `next` hints and diagnostic remedies already use `pse` forms ([59a A03](59a-cli-foundation.md#a03)).
  - Deliver guidance alongside each usable producer→consumer slice, including source queries/
    previews and base observation before full semantic/compiler enrichment. Focused owner views expose
    available sections and explicit missing producers; agents can follow typed next actions
    without automatic builds, captures or probe execution.
- **Plan index.** `docs/plans/README.md` becomes a short current-work pointer: active plans, one
  line each, with status owned by their tables. The narrative history moves to the plans' own
  Outcomes or retires to Git history (ADR-0096).
- **Edit guidance.** One AGENTS.md line and a short `.claude/rules/tooling.md` example:
  - `apply_patch`/`Edit` by default;
  - `ast-grep run --pattern … --rewrite … --update-all` for repeated structural Rust changes;
  - assert the expected match count in any unavoidable script edit.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [E01](#e01) | complete | 59e | `new`/`list` implemented with stage evidence, native-byte paths and passive original users; CLI removal explicitly deferred under R-56, retaining native Git administration. Creation recipe retired; direnv behavior unchanged |
| [E02](#e02) | superseded | 59e | Generic session brief/status-line target removed by review O01; existing passive status remains |
| [E03](#e03) | complete | 59e | Native test/runtime/source/product/semantic/API/compiler/worktree guidance and generated reference published; actual implementation and diagnosis use delivered verbs. Deferred commands are not advertised |
| [E04](#e04) | complete | 59e | Current-work pointer links native status owners; lifecycle validate passed with zero errors |
| [E05](#e05) | complete | 59e | Edit guidance in AGENTS.md and `.claude/rules/tooling.md` |
| [E06](#e06) | complete | 59e | Delivered guidance exercised by a delegated native Codex agent through generated/historical search→guarded inspection→declared authored emitter; parser reference regenerated and advisory endpoint tested. Live UI hook trust/injection remains explicitly unverified; P02 owns assembled qualification |

<a id="e06"></a>
### E06 — Adopt delivered search and retire displaced guidance

**Implemented (2026-10-31):** `pse-cli`, its code-intelligence reference and the native
environment guide describe first-arrival discovery, typed/native composed requests, explicit
exhaustiveness/count/order, result fields/grouping/budgets, effective-config explanation,
guarded direct/reference inspection and current test-candidate selection. Old passive command
examples and advisory suggestions are replaced with actual `search`/`inspect`/`propose`
routes. The hook remains advisory and narrowly recognizes useful Rust read/search questions;
generic searches and edit escape hatches remain available. No hook-definition, trust,
global Codex/MCP or direnv change was made.

**Focused adoption evidence:** an executing agent used the actual native CLI to discover
`StreamWriter` declarations, refine to `StreamWriter<W>::emit`, follow its portable source
reference and inspect the original `write_all` body. A bounded-symbol run first exposed a
quota-versus-unknown classification defect, retained for provider correction rather than
reported as success. Explicit exhaustive source discovery and guarded inspection succeeded.
Native test-selection inspection also returned the actual library/feature/filter intent
while keeping admission, listing and fixture readiness unavailable. These are practical
current-agent use. No speedup or complete replacement of specialized external tools is claimed.

**Tested (2026-10-31):** a delegated native Codex agent used the delivered skill and parser
help on the actual checkout to discover generated declarations, paste a returned typed source
reference into `inspect`, inspect its exact original bytes, and follow `inspect origin`'s returned
action to the declared `documents` emitter. The historical variant retained its resolved commit
and blob while origin inspection explicitly identified the independently declared producer
snapshot; it did not claim that the current emitter generated those historical bytes. Representative
origin invocation `01a193e2-d525-715e-bf00-842a8c39fd33` retained this distinction under
`--fields hit`, including mandatory provider, coverage and next-action facts. A mismatched source
guard refused. Missing authored source locations, unsupplied Cargo metadata and an unprepared
document interpreter remained explicitly unknown/unavailable, without implicit preparation.

This establishes actual native agent CLI adoption, including a useful generated-source
refinement. Live Codex/Claude UI hook trust or context injection remains **unverified**; invoking
the advisory endpoint does not establish that attachment. No hook-definition or global runtime
change is selected. The parser-owned command reference has been regenerated through `just codegen`
and its comparison passed in scope-end hygiene. [P02](59-agent-cli.md#p02) owns final integration.

**Correction and deliberate limit:** the advisory parser now recognizes literal single-quoted
Rust search arguments containing `$`, while retaining dynamic double-quoted and mutation
negative controls. Actual endpoint `PreToolUse.additionalContext` and delegated native CLI use
are tested; neither is relabelled as live Codex/Claude UI attachment. No global trust/configuration
change or complete displacement of specialized external tooling is claimed.

**Consumes:** actual public slices delivered through [M03](59m-composed-rust-investigations.md#m03),
including their parser-owned discovery/schema, useful examples and retired-route list. E06 can
ship with each ready slice; M03 does not depend on E06 and is not blocked by later client adoption.
Final series qualification consumes both delivered behavior and E06 adoption evidence.

**Delivers:** update `pse-cli`, its focused references, the native operating guide, generated
command reference and relevant role/agent guidance to prefer the delivered intents. Teach direct
known-file inspection, source/result/evidence locators, modest first answers with useful expansion,
explicit context and preparation, current versus retained evidence and proposal inspection before
ordinary edits. Retain intentional external-tool escape hatches until actual journeys establish
displacement. Preparation/control and test execution remain explicit rather than hidden behind search.

Move repository-local advisory suggestions to useful delivered capabilities in the same slice.
Do not blanket warn on every `rg` or read, enforce adoption, infer intent from an arbitrary shell
pipeline or recommend a mode before it exists. Preserve the existing bounded advisory-hook
contract and native trust/review requirement when hook definitions change. No global Codex,
MCP or direnv change is selected. New command reference is generated from its parser owner.

**Planned verification and deletion:** a fresh Codex/Claude client discovers and completes an
unfamiliar-subsystem search→inspect journey and a useful refinement using delivered guidance;
verify reminder behavior in the actual host where applicable. Preserve any failed or unavailable
host-attachment evidence rather than treating unit/snapshot checks as client adoption. Remove
displaced guide examples, skill routes and obsolete advisory suggestions with their replacement;
do not remove distinct useful external capabilities by instruction. Narrow documentation/agent
wiring checks establish only their own scope. Quantitative benefit claims need representative
measurements; this package requires no broad performance campaign. No fresh client/product
tests are run merely for this plan amendment.

### CLI adoption amendment — 2026-10-31

**Implemented:** the local `pse-cli` skill provides thin ordinary-workflow guidance and a
progressive code-intelligence reference. Shared agent/execution guidance now explains the
CLI preference in terms of actual correctness selection, fixtures, ownership and retained
results. Native environment composition exposes the active checkout's `scripts/pse` as
`pse`, without installation or activation-time compilation.

The shared Rust `agent-hook tooling-advice` endpoint parses bounded Bash syntax and emits
only advisory context. Both runtime registrations preserve direct execution and quietly
allow an unavailable/stale host. Ambiguous shell state, functions, dynamic arguments,
foreign manifests, comments and heredoc data do not produce guessed advice. The matcher is
conservative, not an enforcement boundary or a complete shell interpreter. No permission
rewrite, automatic briefing, global configuration change or MCP service was introduced.
`tree-sitter-bash =0.25.1` is the only new locked package; the existing Tree-sitter 0.27.0
is now an exact direct workspace dependency.

**Tested**, local Linux and pinned toolchain, zero failure baseline: `pse test pse-host
--all-targets -E 'test(agent_hooks::) | test(environment::) | binary(tooling_advice) |
test(cli::tests::help_snapshots) | test(cli::tests::command_tree_is_consistent)' --format json`
passed 30/30, required capture and native settlement complete
(`01a19088-e203-75ec-9690-fce3125ef4a2`). `pse test xtask --lib -E
 'test(native_hook_wiring)' --format json` passed 1/1
(`01a19087-6e3a-739b-8677-995e95ab47f1`). Skill-creator validation passed and both existing
runtime aliases resolve the skill. A fresh shell resolved `pse` to this checkout's launcher;
the configured reminder returned context only, and a disposable checkout without a host
returned quiet exit0. These are executable/protocol checks, not fresh-agent adoption evidence.

**Corrections:** review exposed joined/dynamic manifest selectors, directory-changing `env`
options, shell function/builtin shadowing and startup failures outside the Rust endpoint.
The conservative classifier and fail-open registration address those cases. An initial
combined `--lib --test` invocation was correctly rejected; the accepted command uses
`--all-targets` with an explicit Nextest filter. No rejected invocation receives test credit.

**Remaining client acceptance:** fresh Codex/Claude session adoption and in-host reminder
injection are **not_run**. Codex requires native `/hooks` review/trust of the new definition;
the repository does not bypass that mechanism. The skill and reminder implementation are
available; automatic discovery and live delivery must be checked in refreshed clients before
claiming seamless adoption. E01's then-open removal choice is now settled by the final Outcome below.

**Tested — scope end:** `just ready`, `just turn-end` and `just hygiene` passed against
the zero error/warning baseline. Hygiene includes both strict workspace/all-targets Clippy
modes with keep-going, agent wiring, types, licenses, rustdoc and generated equivalence.
`just codegen` completed all nine targets and `just docs` published 951 chapters with
scoped search. The final formatting delta changes layout only. No scientific, performance
or fresh-client adoption qualification is inferred from these checks.

<a id="e01"></a>
### E01 — Worktrees ready on creation

Current slice: sibling-name `new` and passive `list` are implemented through native Git/gix,
the existing immutable host bootstrap and invocation-state owner. Partial creation preserves the
destination, every original stage and its recovery route; unselected warm work remains skipped.
Git's own repository-local environment list is cleared when selecting another checkout.
The public `worktree_commands` test exercises original preparation exit 42 and subsequent passive
inventory; unit tests cover native-byte linked paths, ancestry, dirty state, unchanged local
configuration, ambient foreign Git selectors and standalone namespace registration.
**Tested:** on 2026-10-30, the real `pse worktree new` journey from source commit `368b62143`
completed Git, configuration, host preparation and registration, and the destination's `scripts/pse
status` immediately succeeded. Independently compared local configuration bytes/mode matched,
prepared executables existed and destination `target/debug` was absent: the matching completed
host was copied without destination compilation. The clean test checkout was at the same merged
commit and was removed with native Git after its commands finished. This does not qualify removal
protection or warm-run completion. The maintainer selected explicit deferral of `rm` and retained
native Git administrative removal (R-56). A passive inventory and terminal history are not
deletion authority. The removal scenarios below are dormant adoption criteria.

**Consumes:**
- [59a A04](59a-cli-foundation.md#a04) (store, checkout ID) and [A05](59a-cli-foundation.md#a05)
  (`bootstrap-host --from`, immutable fingerprinted generations and coherent copy/publication).
  A09 publishes the no-collection contract; collection guard/promotion qualification is not a
  worktree-creation prerequisite. Actual checkout users, required artifacts and unresolved
  settlement still protect removal; no automatic generation collection is selected.
- [59b B03](59b-runs-and-continuity.md#b03) supplies the delivered `--warm` launch.
- **Deferred removal only:** B's independent stop/settlement evidence and C04's actual fixture ownership contract before
  removal of a checkout with those users. A terminal invocation alone does not clear a fixture's
  retained-state decision.

**Delivers:** `pse worktree new|list`. Native Git remains the administrative removal route.

**Deletes:**
- The `just worktree` recipe.
- The manual-preparation steps in the environment guide §Worktrees. Its direnv wording stays
  unchanged under the hold.

**Acceptance:**
- `pse worktree new scratch-x` produces a checkout where `scripts/pse status` works immediately,
  with host tools copied rather than rebuilt and `.envrc.local` present with its original mode.
  A failed copy/build/preparation reports the created checkout's partial state and preparation
  remedy without automatic deletion. Help/list/status do not invoke preparation.

**Deferred removal acceptance (R-56; not current delivery or qualification):**

- `pse worktree rm scratch-x` on a clean, merged worktree removes it without prompting, and
  refuses without `--force` when an uncommitted change exists.
- A clean worktree with a live invocation, warm run, unrecorded adapter using a generation,
  unresolved fixture ownership or required referenced artifacts is protected even with `--force`.
  After an explicitly requested stop
  and demonstrated owner settlement/release, ordinary removal succeeds. Pruning presentation
  logs cannot erase finalizer generations, fixture decisions or scientific artifacts.
- Prepare newer immutable tools while an older detached job runs; the old paths remain usable
  without automatic collection. Exercise coherent source-generation copy and separate checkout
  removal refusal for actual readers, children, reusable sessions and unresolved evidence.
  No generation-GC race campaign is required. A missing original PID never supplies authority
  to delete unknown historical resources.

<a id="e02"></a>
### E02 — Session brief and status line (superseded)

Review O01 removes mandatory session-start briefing and status-line integration. No hook,
renderer, cached brief or new settings entry is required. This is scope retirement, not an
implemented feature. Existing passive status/help and B05 session identity remain supported;
uniquely authored handoff notes are not deleted. Reconsider only for an actual user whose
orientation need is not served by current files, command help and selected result reads.

<a id="e03"></a>
### E03 — Communication and routing (continuing)

**Consumes:** each verb as it ships, including F01–F05's actual supported product contracts and
F06's eventual inquiry decisions. No guide advertises a proposed bridge, backend or MCP adapter
as available. D05/D08 and briefing/handoff wrappers are deferred or removed; guidance must not advertise them.

**Delivers, per verb, in the same change as the verb:**
- its AGENTS.md task-table row;
- the guide route that replaces the old raw or recipe form;
- the role or skill mention where that workflow is described;
- removal of documentation for retired recipes.
- concise interpretation of new results: invocation versus scientific identity; command
  acknowledgement versus subject/settlement outcome; passive versus effectful diagnostics;
  current observation versus retained history; gaps/coverage and optional capture failure.

Affected owners:
- AGENTS.md, including "Start here" and the Execution rhythm's named recipes (`check-package`,
  `unit-package`);
- `docs/dev/{agent-environment,validation-assessment,documentation}.md`;
- `docs/authoritative_design/sections/operations-and-validation.md`, the §24.1 Commands
  paragraph, edited after [59d D06](59d-knowledge-and-workflow-verbs.md#d06) makes design
  documents informing;
- `.agents/roles/*.md`;
- `.codex/skills/{execute-plan,plan-creation,create-plan,functional-extension-review,design-review}/SKILL.md`
  and other skills that describe affected workflows;
- active plans' verification sections, including this series' own, where they name a retired
  recipe as the route to run. Historical plan records are left as they were.

These are shared guidance, not Codex configuration.
Scope the changed-document checks to the named adoption edits through D04; classify unrelated
historical/external issues without initiating cleanup. Architecture edits still use their
normal decision route and revision row; D06's informing guard is not a waiver of authority.

**Acceptance:**
- The AGENTS.md start route gives a short set of task-to-verb entries and points to
  `pse commands` for the rest. It is not a second, hand-written catalog.
- No current guide, skill or active plan route names a retired recipe.
- `just lint-agents` passes.
- A documented local inspection/follow workflow works without a backend or Codex configuration
  change; numerical escalation explicitly names its effects and target, and optional interface
  proposals remain distinguishable from delivered capabilities.

<a id="e04"></a>
### E04 — Slim plan index

**Consumes:** nothing; independent.

**Delivers:** `docs/plans/README.md` reduced to active plans with one line each and links to their
status tables. Narrative status that duplicates plan Outcomes is removed, and lifecycle bindings
stay valid.

**Acceptance:** the index fits on one screen, and `xtask lifecycle validate` passes.

<a id="e05"></a>
### E05 — Edit guidance

**Consumes:** nothing; independent.

**Delivers:** the AGENTS.md line and the `.claude/rules/tooling.md` example described above.

**Acceptance:** the guidance is present and `just lint-agents` passes.

## Verification and handoff

### Final outcome — 2026-10-31

**Implemented:** E01 and the selected Plan59 scope are complete. Creation/listing retain
their recorded implementation and tests. The maintainer selected native Git administrative
removal and explicitly deferred CLI removal under R-56. The complete plan series is retained.

**Mistake corrected:** the handoff described removal refusal as delivered although the parser
exposes only creation/listing. Current guidance now separates delivered inventory from the
deferred retirement boundary; historical acceptance receives no additional credit.

**Deliberate deviation:** removal protection and teardown are unavailable in the CLI.
Future adoption must establish the dormant owner/reader/session/fixture/evidence safety
criteria above, including settlement and force-refusal semantics. Automatic generation
collection remains absent. Live client hook injection is still unverified. Prior qualification
is unchanged; this closeout is documentation-only.

### Code-intelligence adoption and live users

**E03** adds task routes as G/H/I/J verbs actually ship, including explain/references, candidate
tests/completion, API/examples, previews and explicit compiler investigations. No hook starts
analysis, generates rustdoc or compiles automatically. **E01** supplies passive checkout inventory.
Its deferred removal target must protect reusable sessions, their children and required
generations while live/unsettled; explicit stop-and-settle would precede safe removal. Query completion or absence
from scientific run lists is not drain evidence. Each checkout retains independent analysis
context/build state. E02 adds no hook/status-line prerequisite.
**E04** links coordinator-owned G–J scope rather than duplicating status. Per-slice actual CLI
journeys and their distinguishing failure cases precede replaced-caller retirement; scope-end
static/integrated qualification keeps its existing timing and does not block discrete adoption.

Deferred removal acceptance adds two checkouts, active-analysis removal refusal and successful
removal after actual settlement. Codex/editor/MCP/direnv exclusions and ordinary host capacity remain intact.

- **E01:** recorded targeted `pse-host` tests and real creation/listing; native Git cleanup of
  the disposable checkout is not acceptance of CLI removal protection.
- **E02:** no feature-delivery acceptance; verify retired requirements are absent from current guidance.
- **E03–E05:** documentation and agent-configuration changes, checked with `just lint-agents` and
  the documentation link check.
- **Scope end:** the series' scope-end `just docs`/`just governance` runs cover them.
