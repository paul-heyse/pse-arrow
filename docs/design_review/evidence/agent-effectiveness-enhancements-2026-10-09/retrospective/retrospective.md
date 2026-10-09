# Recent agent activity retrospective

Read-only evidence for repository baseline `d0f2c41818a34539a910654dfea4760771603f45`. Fixed event window: **2026-10-06 00:00:00 America/New_York through 2026-10-09 06:52:57 America/New_York** (04:00:00Z through 10:52:57Z). This reports observed activity and review leads, not product qualification or causal policy effectiveness.

## Method and source coverage

`extract.py` inventories every `~/.codex/sessions/**/*.jsonl` file, chooses candidates using the first/last complete event timestamps, and streams every candidate line with event-time filtering. The 38GB directory yielded **713 scanned files / 6,455,241,249 inventory bytes** after exclusion. Six attributable sessions began before the window. The current planning/review root `01a1203a-a1fc-7b23-a3f0-d7861d61e900` and its descendants are excluded (four files existed by the cutoff; later children start outside the window).

Shell attribution uses literal per-call `workdir`, an explicit absolute shell `cd`, then session/ancestor cwd. Every candidate workspace was scanned; all 108 attributable sessions happened to have pse-arrow metadata cwd. The parser excluded 94,339 literal shell commands attributed elsewhere and left 129 cross-repository references ambiguous. This improves coverage over a cwd-only prefilter without claiming perfect shell interpretation.

Native custom/function calls are matched by call ID. Where an output call ID repeats, the original extraction retained the first output and did not compare payload equality. Later outputs may be legitimate notify/stream outputs; they are not established identical duplicates, and first-output retention may omit subsequent evidence. Literal JavaScript `tools.exec_command` and `tools.write_stdin` calls are extracted without evaluation. Structured native results are matched in order only when cardinality agrees; otherwise per-command flags and native exits are withheld. Call completion, script yield, native running status, native exit and explicit test verdict remain separate. `event-pointers.jsonl.gz` contains identifiers, line pointers, hashes and categories only; `source-manifest.json` includes content-free metadata and event-window SHA256 digests. No raw prompts, command bodies or outputs are copied into these artifacts.

## Descriptive results

**108 sessions / 6 root trees**, **24,377 shell launches**, **3,083 nested process polls**, **26,644 distinct attributable wrappers**. These produce 34,913 records; record totals combine launches, polls and native tools and must not be treated as a test or failure denominator.

| Local event date | Records |
|---|---:|
| 2026-10-06 | 15,868 |
| 2026-10-07 | 7,110 |
| 2026-10-08 | 9,293 |
| 2026-10-09 | 2,642 |

Date, CLI version (0.160.0, 0.160.1, 0.161.0), root/subagent and heuristic task strata are in `metrics.json`. The three largest root trees account for most activity; this is not an independent sample of 108 tasks. Task labels are heuristics, with many unknown and long roots spanning multiple user tasks.

Search/read classifications: **20,034**; exact repeated search/read occurrences beyond the first within one session: **550 across 257 patterns**. Repeated reads are not automatically waste: preservation checks, changing source and progress checks explain several high-frequency examples. Truncation occurs in **2,287 distinct wrappers**; no claim is made that every truncation hid a decisive result.

Native exits are recoverable for **16,204 / 24,377 launches**. The other **8,173** often print only `r.output`, run asynchronously or have an ambiguous batch mapping. Native nonzero exits include search misses, interruptions and deliberate rejection tests. Likewise `completed` means the wrapper completed, not that the underlying operation passed. Two unmatched calls remain at the cutoff; they are not automatically failures.

The corpus has **42 malformed lines**, **30 repeated output call IDs** (first output retained; payload equality not established), **669 dynamic/unparsed command expressions**, and **2,980 ambiguous nested result records**. These diagnostics span all scanned workspaces unless explicitly called attributable. The extractor's shell classifier and flag patterns are heuristics, not a calibrated error-rate estimate.

## Contextual leads and contrary evidence

`audited-leads.json` supplies exact session IDs and source line pointers for eight checked leads.

- **Command/selector recovery:** nonexistent `surreal-status` and `fmt-rust` recipes, one invalid `name|name` filter, and one explicit zero-test selection were observed. The fmt command and invalid filter were corrected in the next relevant call. A `just --show` missing recipe elsewhere was discovery, not an attempted test run. Recipe errors therefore support a convenience lead, not a broad inability claim.
- **Environment/fixture selection:** a bare `.venv` pytest call failed importing `libmkl_gnu_thread.so.3`; the agent inspected the native environment afterward. Native calls also refused a stale lockfile, an unavailable default store, or a server not ready. These are actual boundary refusals, but the reviewed threads were modifying the store/runtime implementation. Transient integration state and incomplete fixture setup are competing explanations.
- **Positive native evidence:** 272 launch strings explicitly select `--native`; 201 classify as build/test/check. The checked canonical-store trace proceeds from boundary refusal to an actual test failure and then explicit one-test passes. Existing routes work when selected with the appropriate fixture/capability setup. Their observed use argues against blanket instruction proliferation.
- **Contention versus observation:** 43 records mention Cargo lock waits after read-only log tails are suppressed. Only four classify as actual build/test/check launch observations and ten as process-poll observations; other records are inline/recipe/wrapper observations. This is not 43 distinct blocked builds. One checked launch `just check-package <package>` does show a genuine build-directory wait, while repeated tails reproduce the same message. No new cap recommendation follows from these counts.
- **Result retrieval and navigation:** one assessment log tail repeats 24 times in one session. Such progress retrieval is a lead for evaluating an optional more convenient result/navigation surface; it does not demonstrate unnecessary compilation. Context7 is already visible: 202 in-scope nested resolve mentions and 263 query mentions; five GitHub search mentions also occur. No optional semantic navigation tool appears in the scoped MCP mention set, but mention absence does not prove unavailability, missed opportunity or lack of equivalent source navigation. Calls are mentions, not independently verified successful MCP outcomes.

## Reproduction and limits

Original measurement command: `scripts/pse-env -- python /tmp/pse-agent-effectiveness-20261009-retrospective/extract.py` — **passed, native exit 0**, read-only transcript extraction, zero execution-failure baseline. That original extractor processed the full candidate corpus; its SHA256 is recorded in `metrics.json`. The source malformed-line count is an input-quality diagnostic, not a clean-zero claim. No product tests, formatters, services, configuration changes or repository edits were performed.

Future reproduction from the repository root:

```bash
scripts/pse-env --resource-class light -- .venv/bin/python docs/design_review/evidence/agent-effectiveness-enhancements-2026-10-09/retrospective/extract.py --output-dir /tmp/pse-agent-retrospective-reproduction
```

This portable parser discovers the repository through Git and exposes `--repo-root`, `--sessions-dir`, `--claude-projects-dir`, `--output-dir`, `--start-utc`, `--cutoff-utc`, `--timezone` and repeatable `--exclude-root` overrides. The historical timestamps and excluded review root are defaults; filesystem locations are derived at execution. Use a separate output directory to preserve the original retained measurement and provenance.

Finalization added the SPDX header, CLI/root discovery, gzip output, formatting, and clearer repeated-ID terminology. Existing pointers were compressed into `event-pointers.jsonl.gz`; unused `inventory.json` was removed. **No corpus rescan followed these changes.** The aggregate counts remain those of the original extractor, with a diagnostic-key rename recorded in its provenance. First-output retention behavior did not change; the earlier wording incorrectly implied identical duplicates. A CLI `--help` check passed for the finalized script; it does not establish a full patched-parser scan.

The retained `build/agent-effectiveness/w1/{metrics.json,report.md,events.jsonl.gz}` records Plan 29's earlier comparator. Its parser is not retained, its windows mix policy eras and runtime/task composition, and its counts are **historical evidence only**. No causal before/after improvement or regression is inferred.

Claude was inventoried only: its pse-arrow directory has 398 JSONL files / 1,009,862,147 bytes, 25 modified since the start; the nested thermo directory has four files with none modified since the start. Modification time does not establish in-window Claude activity. Codex archived sessions were not scanned. Live files can grow; fixed timestamps and per-window digests make the selected events inspectable.

Literal extraction does not interpret JavaScript variables/templates, dynamic workdirs, relative/multiple shell cwd changes or arbitrary result-reordering code. Classifications may pick up words in shell scripts; task types and MCP mentions need judgment. Only the material contextual traces listed in `audited-leads.json` were checked; no exhaustive regex-flag audit, token/productivity estimate, measured latency saving, full tool-outcome linkage or representative task-quality study is claimed.
