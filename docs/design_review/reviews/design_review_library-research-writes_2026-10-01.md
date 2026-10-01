# Library-research write scope governance review — 2026-10-01

## 1. Scope, drivers and coverage

| Field | Assessment |
|---|---|
| Subject | Proposed ADR-0147: the `library-research` write scope, the standing-write-scope clause in the common worker contract, Codex sandbox inheritance, Claude's adapter tool grant, AGENTS.md and §24.4 routing, and the carried-forward ADR-0139 coordination policy |
| Standard | Core/template 3.3; process-simulator 1.3 (not applicable); `binding/pse-arrow.md` |
| Tier / purpose | Design / target, bounded to development governance |
| Reviewer | Independent delegated design reviewer, 2026-10-01 |
| Baseline | HEAD `88b653d7d977acfc1d80e41d337835e4843ff209` plus the uncommitted changes to `.agents/roles/{library-research,worker,README}.md`, `.claude/agents/library-research.md`, `.codex/agents/*.toml`, `AGENTS.md`, `docs/authoritative_design/sections/design-change-workflow.md` and the untracked `docs/adr/0147-library-research-writes.md` |
| Decision | Behavioral/semantic adequacy: satisfied at static policy level. Architectural fitness: satisfied. Overall: **Accept** (with the ADR-route bookkeeping in F01 required in the acceptance commit) |
| Disposition owner | ADR-0147 and blueprint §24.4; F01 belongs to the ADR acceptance route; the low findings are proposed for ADR-0147's author before acceptance. No production plan is affected |

Functional target: an evidence worker can leave durable evidence (a dated probe folder, or a fix to an assigned shared library skill) without a second worker, while production code, design, decisions, plans and configuration stay outside its reach and the configuration does not claim enforcement the runtimes do not apply.

Inspected: the subject diff and new ADR; the ADR-0139 record and its precedent review; all role contracts; Claude and Codex adapters; `scripts/agent-hooks.py`; `.claude/settings.json`; `.codex/hooks.json`; `.codex/config.toml`; AGENTS.md off-limits and runtime sections; the binding; the `adr` and `execute-plan` skills; the shared skill store README at `~/.local/share/library-skills/`; Codex 0.159.2 role sources at the tag; non-secret keys of `~/.codex/config.toml`; Claude user and local settings.

Excluded: other sessions' dirty work (`crates/`, `docs/generated/`, `docs/plans/25e…`, ADR-0145/0146 and their review), production qualification, model quality and native library-research task execution.

## 2. Decomposition, ownership and dependencies

| Responsibility | Owner and consumed boundary |
|---|---|
| Which effects a role may have (standing scopes) | Shared role contracts; library-research `.agents/roles/library-research.md:13–22` |
| Narrowing effects per assignment | The coordinator's brief; narrow-never-widen (`.agents/roles/worker.md:7–9`) |
| Repository-specific evidence locations | AGENTS.md (`AGENTS.md:366–369`); the shared contract defers to it, which keeps the contract byte-identical across repositories and the copier template |
| Native tool surface | `.claude/agents/library-research.md:4` (Write, Edit added); Codex role files declare model, effort and instructions only |
| Runtime sandbox and approval | The session; Codex children inherit it (§5); Claude permission rules and sandbox are session-wide |
| Protected-path enforcement | `scripts/agent-hooks.py:42–74` and `.claude/settings.json` deny rules — unchanged, applying to every role's Write/Edit/patch inside the working copy |
| Shared skill sources | The skill store under `~/.local/share/library-skills/`, governed by its README ("one writer per skill at a time", repository-independent) and each skill's `MAINTENANCE.md` |

The split follows reasons for change: runtime facts live in the adapters and §24.4, role behavior in the shared contract, repository locations in AGENTS.md. Because the shared contract names its locations indirectly ("the library-evidence locations this repository's AGENTS.md names"), one text serves every repository; the template and thermo-knowledge copies are byte-identical (§10).

## 3. Contracts, authority and constraints

| Contract | Owner and scope | Enforcement and failure |
|---|---|---|
| Permitted writes: (1) new dated evidence folder with README, (2) an assigned shared skill, repository-independent, one writer, (3) scratch outside the repository | `library-research.md:13–18`; AGENTS.md:367–369 names `docs/design_review/evidence/` | Instruction. Claude Write/Edit inside the repository pass through the guard hook (blocks authoritative design, accepted ADRs, generated paths); Bash writes unguarded |
| Never: production code, tests, design, ADRs, plans, STATUS, pins, configuration, generated paths, another worker's folder | `library-research.md:19–20` | Hook-protected subset as above; the rest is instruction plus the coordinator's `git status` review |
| No commit, push or hook-owned sync unless assigned; list every file written | `library-research.md:20–21` | Return obligation is the main compensating control |
| A brief narrows, never widens | `library-research.md:21–22`; `worker.md:8–9` | A widening brief is a "conflicting contract" the worker reports (`worker.md:12–13`) |
| Capability-map evidence (`just evidence-regen`) stays with the coordinator | `AGENTS.md:369` | The recipe writes `docs/capability-maps/evidence/` (`scripts/evidence-regen.sh:15`), outside the permitted locations |

Consistency with neighbors: the other evidence roles remain read-only (`code-mapper.md:9`, `design-reviewer.md:19`, `implementation-reviewer.md:13`). `test-agent.md:8–9` already holds a standing write scope (build, temporary, test data), which the new `worker.md` sentence now describes correctly instead of contradicting. The hook returns no protection outside the repository (`agent-hooks.py:46–48`), matching "paths outside the repository, including shared library skills … are editable" (`AGENTS.md:308–310`). The user-level `additionalDirectories` includes the skill store, and pse-arrow's local settings use `bypassPermissions`, so Claude writes there work without prompts. `execute-plan/SKILL.md:28` ("Read-only evidence tasks can run alongside implementation") remains sound because new dated folders are disjoint from implementation edit ownership. No contradiction with AGENTS.md off-limits, `agent-hooks.py` or `settings.json` was found.

Observation (not a finding): evidence folders are a standing scope, not brief-opt-in, so a research worker may create evidence folders the coordinator did not request; ADR-0096's current-working-set rule and the return's file list let the coordinator remove them. Acceptable.

Physical-semantics and numerical-stage tables are not applicable: the subject allocates development work and formulates or solves no models.

## 4. Representative change scenarios

| ID | Scenario | Observation |
|---|---|---|
| S01 | A research brief needs a probe that settles a library claim | Worker writes `docs/design_review/evidence/<new folder>/` with README, lists files; coordinator integrates. No second worker. Naming convention unstated (F05) |
| S02 | A brief asks library-research to fix a capability map | `docs/capability-maps/` is not a permitted location and AGENTS.md:369 keeps that evidence with the coordinator; a brief cannot widen scope, so the worker reports the conflict (`worker.md:12–13`). The coordinator or an executor edits. Not hook-blocked — instruction only |
| S03 | Two research workers (possibly from different repositories' sessions) assigned the same shared skill | "One writer per skill" is satisfiable within one coordinator's session but not checkable across sessions; the store has no commits, and the compensating `git status` is repository-only (F04) |
| S04 | A Codex parent session started read-only | The child inherits the parent's permission profile and approval policy (`child_config.rs`, `apply_spawn_agent_runtime_overrides`); writes fail and the worker reports `blocked` with the missing prerequisite (`worker.md:12`). Under this maintainer's `danger-full-access` default the change grants Codex no new effective permission — the removed keys were already ignored |
| S05 | A brief asks library-research to run `just evidence-regen` | The recipe writes outside the permitted locations and AGENTS.md:369 names it the coordinator's; the worker declines and reports |
| S06 | Runtime substitution: a future Codex re-applies role sandbox keys, or a runtime gains per-agent path scoping | Covered by ADR-0147's revisit trigger; the adapters would regain a native declaration without changing the shared contract |

Governance-policy extension (a further role gaining a standing scope) and runtime-adapter substitution (S06) are the relevant variation axes; the worker-contract clause supports the former without editing other roles.

## 5. Mechanisms and execution

Codex (codex-cli 0.159.2), read at tag `rust-v0.159.2`: `agent-roles/src/agent_role_config.rs` parses a role file as `RawAgentRoleFileToml` with `#[serde(flatten)] config: ConfigToml`, so `sandbox_mode` parses without error, but `core/src/agent/role.rs::apply_role_to_config_inner` copies only `developer_instructions`, `model`, `model_reasoning_effort`, `model_reasoning_summary`, `model_verbosity`, `personality`, `service_tier`, and capability *reductions* (shell tool, apps, plugins, memory tool, request-permissions tool disabled; skills disabled) into `AgentRoleOverrides`. Its module comment: "Roles may customize the child or reduce its capabilities, but never replace the parent session's authority." `core/src/agent/child_config.rs::apply_spawn_agent_runtime_overrides` copies the turn's approval policy, cwd and permission profile into every child. At `rust-v0.148.0`, `role.rs` applied the whole role layer at session-flag precedence; the bounded override struct first appears at `rust-v0.149.0`. The claim "dropped since 0.149; roles inherit the session sandbox" is therefore **Interface-checked** (published tag source, not the installed binary build; the binary's version and loader strings are consistent; no child spawn executed by the reviewer).

Claude Code 2.1.287: the current documentation (code.claude.com/docs/en/sub-agents, fetched 2026-10-01) states permission allow/deny rules and sandbox settings apply session-wide; per-subagent controls are `tools`, `disallowedTools`, `permissionMode` and `hooks` (frontmatter hooks scoped to that subagent); `permissionMode` is ignored when the parent runs in `bypassPermissions`, `acceptEdits` or auto mode (pse-arrow's local settings use `bypassPermissions`). The ADR's "only the tool list and hooks are per agent" is accurate in effect; two sentences overstate it (F03). The docs are current, not version-pinned.

Compensating mechanisms: the guard hook protects in-repository Write/Edit/patch for every role, including Claude subagents (Codex child-thread hook firing not verified by execution; not_run). The return lists every file written. The coordinator's `git status` covers repository writes but not the shared store (F04).

## 6. Architectural assessment and gates

| Foundation | Verdict | Evidence |
|---|---|---|
| AP-01 Separation of concerns | satisfied | Standing scopes in role contracts, repository locations in AGENTS.md, narrowing in the brief, native tool grants in adapters, runtime enforcement facts in §24.4 |
| AP-02 Stable contracts | satisfied | The write scope is explicit, closed (whitelist plus deny list) and non-widenable by a brief; the shared text is identical across consumers |
| AP-03 Composition | satisfied | The worker-contract clause composes role scopes with briefs; no role-specific coordination machinery |
| AP-04 Domain model and authority | satisfied | The model distinguishes the consequential differences — standing vs. brief-narrowed effects; repository vs. shared store vs. scratch; evidence vs. authority (capability maps and design remain coordinator-owned); instruction vs. runtime enforcement — and these govern the adapter and routing text |
| AP-05 Explicit structure | satisfied | Locations, exclusions, commit rules and return obligations are declared; the non-enforcement is stated rather than implied |
| AP-06 Local reasoning | satisfied | A worker determines its permitted effects from two contracts, the brief and one AGENTS.md sentence |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 Authority | pass | Contract, AGENTS.md and §24.4 agree; ADR-route bookkeeping (supersession, revision row, inbound link) must land with acceptance (F01) |
| G2 Semantic fidelity | pass | ADR-0139's coordination decisions survive; minor omissions to restore (F02) |
| G3 Validity | pass | Off-limits protections unchanged; Codex instruction budget within limits (§10) |
| G4 Hidden behavior | pass | Instruction-only enforcement, Bash reach and session inheritance are stated (ADR-0147:108–112, §24.4:44–45) |
| G5 Consistency/recovery | pass | Repository writes reviewable via `git status`; cross-session shared-skill writes have weaker recovery (F04, low) |
| G6 Transformation/reuse | pass | One shared text, identical in template, pse-arrow and thermo-knowledge |
| G7 Truthful claims | pass | The configuration no longer declares no-op sandbox modes; the Claude per-agent wording is slightly overstated (F03); verification cites a smoke run not located (F06) |
| G8 Library leverage | pass | Native tool grants and session inheritance; no bespoke guard |
| G9 Architectural fitness | pass | All six foundations satisfied |
| PS-G1–PS-G3 | not applicable | No physical, well-posedness or numerical behavior |

## 7. Findings

| ID | Severity | Finding | Evidence | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | Medium (acceptance precondition) | The supersession of ADR-0139 is not recorded in the decision route | `docs/adr/0147-library-research-writes.md:12` has `supersedes: []` although Scope (:40–41) says it supersedes ADR-0139; `docs/adr/0139-selective-agent-coordination.md:12–13` remains `accepted` / `superseded-by: null`; no ADR-0147 revision row exists in `docs/authoritative_design/blueprint.md` (`adr` skill §4); `docs/plans/README.md:60–61` still routes the role rollout to ADR-0139. The §24.4 Decision line (`design-change-workflow.md:13`) already cites only the proposed record | After acceptance the index would show two records owning §24.4 with no supersession link; retiring ADR-0139 would leave an inbound link pointing to a deleted file | In the acceptance commit: `supersedes: [ADR-0139]` (`just adr-supersede ADR-0139 ADR-0147`, or name the retired predecessor if ADR-0139 is deleted under ADR-0096); add the blueprint revision row; repoint `docs/plans/README.md:60–61` to ADR-0147 | `python3 scripts/adr.py lint` clean for 0147/0139; row present; no inbound ADR-0139 link outside history rows |
| <a id="f02"></a>F02 | Low | "Carried forward … unchanged" drops some of ADR-0139's surviving obligations | ADR-0139 compensating control "Existing edit protections, generated-code rules, memory caps, pinned commands and test timing apply" (`0139…:88`) becomes hook/deny-rule-specific text at `0147…:116–119`, without memory caps, pinned commands or test timing; ADR-0139's consequence that loaded instructions stay within the cumulative Codex document budget (`0139…:79–81`) is absent from `0147…:106–112`, though both are claimed to carry forward at `0147…:78` | When ADR-0139 retires, the instruction-budget rationale loses its decision owner, so the next AGENTS.md growth has no recorded reason to stay small | Add one sentence to ADR-0147's compensating controls restoring the memory-cap/pin/test-timing clause and the instruction-budget constraint, or qualify "unchanged" | Line-by-line Outcome comparison shows no unexplained omission |
| <a id="f03"></a>F03 | Low | The Claude per-agent capability statement is slightly overstated | `.agents/roles/README.md:33–34` "Claude roles differ only in their tool lists"; `0147…:57–58` enforcement "only through a PreToolUse guard keyed on `agent_type`". Current docs also offer `disallowedTools`, `permissionMode` (ignored under the bypass parent pse-arrow uses) and subagent-scoped frontmatter `hooks` | A future maintainer may misjudge Option 2's cost: a frontmatter hook in the one adapter is cheaper than an `agent_type`-keyed guard, though equally unable to stop Bash. Selection unchanged | README: "…differ in their tool lists; frontmatter hooks and permission modes are unused here". ADR Option 2: mention the subagent-scoped hook | Wording matches the documentation |
| <a id="f04"></a>F04 | Low | "One writer per shared skill" is not checkable across sessions, and the compensating control does not cover the store | `library-research.md:17–18` ("one writer per skill"); the store README allows any session or assigned worker to edit; the thermo-knowledge ADR-0009 grants the same scope; `0147…:116–117` relies on the repository's `git status`; `~/.local/share/library-skills` has no commits ("does not have any commits yet"), only staged copies | Two coordinators (pse-arrow and thermo-knowledge) assigning the same skill concurrently can silently overwrite each other's edits, undetectable by the coordinator's listed-file review | Before its first write, the worker runs `git -C ~/.local/share/library-skills status --short -- skills/<name>` and reports unexplained pre-existing changes instead of editing over them; the compensating control names the store's `git diff` for skill writes. An initial store commit (maintainer decision) would give a baseline | A brief assigning a dirty skill produces a reported conflict, not an overwrite |
| <a id="f05"></a>F05 | Low | The evidence folder naming convention is unstated | `AGENTS.md:367` "new dated topic folders"; existing folders use `<topic>-<YYYY-MM-DD>/` (`docs/design_review/evidence/blueprint-rev4-2026-09-13`, `full-arrow-datafusion-2026-09-14`); the identical shared contract serves thermo-knowledge, whose convention is `YYYY-MM-DD_<topic>`; pse-arrow requires lowercase kebab-case names (AGENTS.md *Doc conventions*) | A worker carrying habits across repositories creates an inconsistently named folder requiring a rename | Name the pattern in the AGENTS.md sentence, e.g. "``docs/design_review/evidence/<topic>-<YYYY-MM-DD>/``" (~+20 bytes) | New folders match the existing pattern |
| <a id="f06"></a>F06 | Low | The verification field cites evidence the reviewer could not locate | `0147…:15` "Native write behavior is exercised by a library-research smoke run in a sibling repository" — no path, commit or receipt named; the thermo-knowledge evidence folders and ADR-0009 do not identify one | `adr` skill §5 requires the verification to "name something that exists"; the claim is unauditable | Cite the smoke run's commit or evidence path, state it as a dated coordinator receipt, or remove it | The cited artifact resolves |

No finding blocks the policy decision. F01 is mechanical route bookkeeping the ADR process performs at acceptance; F02–F06 are wording and controls the author can fix before acceptance.

Strengths: removing the no-op `sandbox_mode` lines makes the configuration truthful; indirect location naming keeps one shared contract; narrow-never-widen resolves all four failure cases without new machinery.

## 8. Library/runtime fit and ownership cost

| Capability | Mechanism | Fit and limits |
|---|---|---|
| Per-role write scoping, Codex | None for writes; 0.159.2 roles can only reduce shell/apps/plugins/memory/request-permissions/skills — no path scope, no patch-tool removal | Dropping the keys and relying on the contract is correct; disabling the shell for this role would block probes, its purpose |
| Per-role write scoping, Claude | `tools` (adopted), frontmatter `hooks` (unused), `disallowedTools` | The Write/Edit grant is necessary; a frontmatter hook could restrict Write/Edit to permitted prefixes but leaves Bash open — partial |
| Protected paths | Existing shared guard hook and anchored deny rules | Unchanged; they apply to the new grant inside the repository |

Ownership cost: one sentence per adapter and one AGENTS.md sentence, maintained alongside the template. No generator, hook or telemetry.

## 9. Alternatives and tradeoffs

| Alternative | Assessment |
|---|---|
| Current baseline (ADR-0139 read-only) | Every evidence artifact needs a second worker; the Codex declarations claimed enforcement 0.149+ does not apply (verified, §5) |
| Proposed (instruction + native grant) | One handoff for evidence. The boundary rests on contract adherence and the coordinator's review of the return. Residual risks: (a) a misdirected in-repository write — visible in `git status`, hook-blocked for protected paths; (b) Bash writes anywhere — Bash was already granted to every read-only evidence role under ADR-0139, so (b) is not new risk; the increment is Write/Edit for one role. Codex effective permissions are unchanged |
| Per-agent guard (Claude frontmatter hook or `agent_type`-keyed guard) | Partial (Write/Edit, not Bash; not Codex), adds per-repository and template maintenance for little added guarantee; reasonable to revisit if an out-of-scope write occurs — already a revisit trigger |
| Simplest viable | Same as proposed |

Selection: the proposed design. Claim 2 holds — the residual risk lives mostly in Bash and the shared store (F04), which neither alternative closes.

## 10. Verification (2026-10-01)

| Claim | Command / evidence | Result and limit |
|---|---|---|
| Subject content and consistency | `git diff -- .agents .claude/agents .codex/agents AGENTS.md docs/authoritative_design/sections/design-change-workflow.md`; reading of role contracts, hook, settings, skills | **passed** (static) |
| Codex version | `codex --version` → `codex-cli 0.159.2` | **passed** |
| Codex roles ignore sandbox/approval keys and inherit the session | `gh api repos/openai/codex/contents/codex-rs/{core/src/agent/role.rs,core/src/agent/child_config.rs,agent-roles/src/agent_role_config.rs}?ref=rust-v0.159.2`; the same `role.rs` at `rust-v0.148.0` and `rust-v0.149.0` | **passed** (Interface-checked from tag source); installed binary build not compared beyond version and loader strings; no child spawn executed (not_run) |
| Session sandbox for this maintainer | Non-secret keys of `~/.codex/config.toml` (`sandbox_mode = "danger-full-access"`, `approval_policy = "never"`); `codex debug prompt-input "x"` in pse-arrow renders "`sandbox_mode` is `danger-full-access` … Approval policy is currently never" | **passed** |
| Claude per-agent controls | `claude --version` → 2.1.287; code.claude.com/docs/en/sub-agents fetched; `~/.claude/settings.json` and `.claude/settings.local.json` inspected | **passed** with the F03 nuance; docs current, not version-pinned |
| Codex instruction budget | `wc -c AGENTS.md ~/.codex/AGENTS.md` → 28,099 + 2,808 = 30,907 bytes; with the pending ~60-byte global edit ≈ 30,967 of 32,768 (default `project_doc_max_bytes = 32768`, present in binary strings, no override in config), ≈ 1.8 KiB margin. `/AGENTS.md`, `/home/AGENTS.md`, `/home/paul/AGENTS.md` and override files absent. `codex debug prompt-input` includes the tail of both files and the ADR-0147 bullet untruncated | **passed** for current files; the +60-byte edit is computed, not rendered. ADR-0147 grew AGENTS.md by 291 bytes |
| Agent configuration coherence | `python3 scripts/check_agent_config.py` (the `just lint-agents` recipe) → "agent config ok" | **passed** |
| ADR lint | `python3 scripts/adr.py lint` → 3 problems | **failed**: (1) 0147 `review:` path absent until this review is published; (2) `docs/adr/README.md` stale (hook-owned regeneration); (3) ADR-0144 evidence label (out of scope). Re-run after publication and F01 |
| Template and sibling identity | `cmp` of `.agents/roles/library-research.md` against `/home/paul/project-template/template/.agents/roles/library-research.md` (worktree and `v0.3.1`) and thermo-knowledge's copy | **passed** (identical); `worker.md` differs from the template only in the skill source path (line 10), as expected |
| Hook coverage of the new grant | Reading of `scripts/agent-hooks.py:42–74` | **passed** (static); `just setup-test` and Codex child-thread hook firing **not_run** |
| Native library-research write behavior (smoke run) | Not located (F06) | **not_run** |
| Product qualification, hygiene, model-quality trials | Outside scope | **not_run** |

## 11. Authority changes and disposition

ADR-0147 supplies the replacement rationale; §24.4 and AGENTS.md are already amended. On acceptance, record the supersession of ADR-0139, add the blueprint revision row and repoint `docs/plans/README.md:60–61` (F01); retire ADR-0139 under ADR-0096 once its surviving meaning is in ADR-0147, including F02's restorations. The precedent review `design_review_agent-coordination_2026-09-30.md` remains cited by ADR-0147's More information and may move to `git:<commit>:<path>` when retired. F02–F06 are proposed for correction by ADR-0147's author before acceptance; none needs a plan row. No SHOULD exception or MUST gap is required, and this review creates no recurring requirement.

## 12. Decision

**Behavioral/semantic adequacy:** satisfied at static policy level. The permitted-writes boundary is coherent with the worker contract, the other role contracts, AGENTS.md off-limits, `agent-hooks.py` and `.claude/settings.json`; all four failure cases resolve through narrow-never-widen or session inheritance. The Codex runtime claim is confirmed from 0.159.2 source; the Claude claim is confirmed with a wording nuance; the instruction budget has ≈ 1.8 KiB margin.

**Architectural fitness:** satisfied; all six foundations hold; G9 passes.

**Overall: Accept** the library-research write scope and the carried-forward coordination policy at Implemented/static level, with F01's bookkeeping in the acceptance commit and F02–F06 recommended before acceptance (non-blocking).

| Priority | Change | Findings | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Supersession, blueprint revision row, inbound link | F01 | `adr.py lint` clean for 0139/0147 | ADR acceptance route |
| 2 | Restore omitted carried-forward obligations | F02 | Outcome comparison | ADR-0147 author |
| 3 | Store pre-write check / store-diff control | F04 | Contract and ADR text | ADR-0147 author / maintainer |
| 4 | Wording: Claude per-agent controls, evidence folder pattern, smoke-run citation | F03, F05, F06 | Text inspection | ADR-0147 author |

This acceptance does not certify native agent behavior, model quality, the enclosing simulator or any dirty production work.
