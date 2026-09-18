---
title: Current-function architecture acceptance
status: active
date: 2026-09-17
adrs: [ADR-0068, ADR-0069]
---

# Current-function architecture acceptance

**Implemented; execution deferred:** `just architecture-acceptance <new-directory>`
is the final Plan 08 campaign. Run it after the complete implementation and deletion
cut. It covers the existing Rust functions, linked/unlinked native solver behavior,
fresh Delta publications, cold Python readers, four current engineering examples,
generation, dependency families, quality, documentation and feature configurations.
It has no dependency on future simulator, Pyomo or NL implementations.

Each command runs once, with its normal feature/profile settings and unchanged
timeouts. The command stops at the first failure and retains `checks.json`, exact
commands, exit codes, elapsed times and per-command logs. The failure baseline is
zero. Test counts and exclusions remain in the native test-runner logs. A subsequent
fix should rerun the affected gate; a fresh complete campaign is needed only when
the change invalidates broader evidence.

The engineering directories contain actual planning/execution/reopen timings,
selected publications, row counts and memory measurements. `source.diff` and source
revision/status identify the source boundary, including tracked changes. Untracked
source must be retained with the working tree for reproducibility.

Successful commands establish their named checks. Independently assess Plan 08
Q01–Q14 and G1–G7 against the resulting evidence; the runner does not manufacture
review verdicts or claim that a fast static check proves a lifecycle property.

During implementation, use `just check-library`, `just check-package`, contract
generation and genuinely isolated `just unit-package` selections. Compiler, storage
and solver journeys remain integration checks even when declared in a library test.
