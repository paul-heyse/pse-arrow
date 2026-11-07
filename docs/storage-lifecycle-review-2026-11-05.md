# pse-arrow storage and lifecycle review (2026-11-05)

This review is for the agent working in this repository. It records what a machine-wide disk
review found about pse-arrow's on-disk footprint, why it keeps growing, what was already
cleaned up, and what should change so that growth stays bounded without manual cleanups.
Nothing in the repository was changed. Take the recommendations as findings to assess, not
as settled design.

## 1. What was observed

At 23:20 EST on 2026-11-05 the root filesystem (1.8 TB) had 204 GiB free. pse-arrow
accounted for about 760 GiB:

| Location | GiB | Notes |
|---|---:|---|
| `~/.local/state/pse-arrow/<checkout_id>/` (24 dirs) | 454 | 23 of them belong to checkouts that no longer exist (204 GiB) |
| ↳ main checkout (`c20f9108…`) `invocations/` | 202 | 279 invocations written between Nov 2 and Nov 5 |
| ↳ main checkout `api` / `products` / `tools` / `fixtures` | 42 | |
| `~/.local/state/pse-arrow/<named store>` | 152 | `surreal-<plan>-<date>`, `plan28-*`, `eff33-*`, `plan50-*` … from Oct 6–27 |
| `~/.cache/pse-arrow` | 66 | sccache 43 (100 GiB cap), native generations 9, plan probes 13 |
| `~/.local/share/pse-arrow/tools/surreal` | 35 | 25 Surreal server builds, each with a ~2.3 GiB `source/` copy |
| Checkout (`target/`, `.local/`, venvs) | 37 | `target/` had been cleaned recently |
| `/tmp/pse-*` | 11 | 96 entries |

Write volume (by file ctime): `~/.local/state/pse-arrow` took **254 GiB on Nov 4 and 158 GiB on
Nov 5**. Free space fell another 20 GiB in the ~30 minutes the review took. Earlier reviews found
the same pattern from other sources: Oct 10, feature-variant incremental dirs and store
generations; Oct 17, ~330 GiB in 21 h of `target/debug` feature-variant builds (274 `pse_runtime`
lib variants across 111 feature sets). On Oct 16 the disk filled completely and crashed PostgreSQL.

## 2. What was already cleaned (2026-11-05, with the maintainer's approval)

462.7 GiB was removed. The log is in `~/.claude/storage-reviews/2026-11-05/execution-log.txt`.

- **Orphaned checkout state (204 GiB):** 22 `~/.local/state/pse-arrow/<checkout_id>` dirs whose
  recorded checkout path no longer exists, such as `pse-arrow-plan65-*`, `numeric-frame`,
  `checked-evaluator` and `pse-plan59-*`. `cac78ee32c21…` (5.6 GiB, `~/.cache/pse-plan59-semantic`)
  was left because one of its invocations names a path that still exists; it needs a look.
- **Invocation binaries (88.7 GiB):** in main-checkout invocations that were settled (have a
  `result.json`), older than 24 h and not cited by plans 64/65, ELF files of 1 MiB or more were
  deleted. All JSON, stdout/stderr and reports were kept. The 29 invocation IDs cited in
  `docs/plans/64*`/`65*` were left fully intact; 7 of them still existed.
- **Named stores of closed plans (134.5 GiB):** every `surreal-*`, `plan28-*`, `plan29-*`,
  `plan50-*`, `eff33-*`, `audit28-*`, `graph-hash-timing-*` and `telemetry-qualification` store.
  Kept: `surreal-plan49-surrealism-20261022` (the selected `PSE_SURREAL_STATE`), `fixtures`,
  `compiler-cache`, `host-admission`, `payload-recovery`, `test-resources`, `private-diagnostics`,
  `recycle-followup-*` and today's `ordinary-*`. Note that plans 34 and 35 cite some of the deleted
  stores as evidence paths; only plans 64 and 65 are open.
- **Surreal tool builds (21.5 GiB):** 12 builds in `~/.local/share/pse-arrow/tools/surreal` that
  no `selection.json`, `release.json`, generation record or fixture config references. Kept:
  `v3.3.0` (running), the three `package-v1-*`, `effective-v1-3549…`, `rust-v1`, `modules`,
  `source-v3.3.4-mx1`, and `3df5f06f…` and `baedb00f…`, which fixtures reference.
- **Misc (14.1 GiB):** `~/.cache/pse-arrow/{plan37-completion-20261012,tool08-scan-probe-20261023-b}`,
  `.local/plan62-layout-journey`, 14 `rustc-ice-*.txt` files in the repo root. Kept because plans
  64/65 cite them or they are in use: `/tmp/pse-plan65-study-capture-*`,
  `.local/plan64-proto-qualification.*`, `.venv-diagnostic-taskdump`.

This was a one-time reset. Nothing below has changed yet, so at the observed rate the same volume
comes back in about two days.

## 3. Root causes

### 3.1 Per-checkout state outlives its checkout

`crates/pse-host/src/paths.rs` keys state by `CheckoutId = sha256(checkout root, git common dir)`
and places it under `$XDG_STATE_HOME/pse-arrow/<checkout_id>`, outside the checkout. Agents follow
AGENTS.md and remove merged worktrees with `git worktree remove`, which cannot know about this
directory. `docs/dev/agent-environment.md#worktrees` states that `pse worktree rm` is deferred
(register R-56) and that "automated owner-aware removal refusal and teardown are unavailable".
The result is that 23 orphans (204 GiB) built up in roughly three days of plan-65 worktree work.

Mapping a state dir back to its checkout is also fragile. Nothing in the state dir records the
checkout root; the review had to decode `meta.json` operation argv (`--working-directory`, or
the `…/target/host-tools/generation.*/pse-host` path), and one directory could not be classified.

### 3.2 Every invocation ledger snapshots multi-GiB binaries and nothing ages them

Each `invocations/<id>/workflow/inventory/` copies the full test inventory:
`group-N/build/artifact-N/{pse-engineering, pse-worker, pse-project, pse-observation-fixture,
pse-engineering-fixture, scientific_worker-*, pse-scientific-worker, pse-deployment}` (1.0–1.5 GiB
each), plus `cargo-pse-correctness` (262 MiB) and `cargo-nextest` (34 MiB). Fixture binaries are
copied again under `workflow/fixtures/`. A single invocation is 2–10 GiB, and 200.6 of the main
checkout's 201.8 GiB of ledgers were these copies. The copies are full copies, not hard links:
in a sample of 40, every `cargo-nextest` was byte-identical (one SHA-256) yet each had its own inode.

`RetentionClass::eligible()` (`paths.rs`) defines when presentation output becomes disposable,
but nothing outside its own module calls it. The governing ADRs rule out the missing piece:

- ADR-0234: "Immutable prepared generations are retained without automatic collection", and
  it removes "automatic host-generation collection".
- ADR-0168: defaults "never grant deletion"; "undeclared/legacy artifacts remain protected";
  it rejects "a universal sweeper".
- `docs/dev/agent-environment.md`: "automatic generation collection is excluded".

The design is therefore "only an explicit owner may delete". No owner ever deletes, so all state
grows without bound.

### 3.3 Plan-scoped stores and evidence have no retirement step

Plan work creates named stores (`surreal-<plan>-<date>`, `plan28-*`, `eff33-maintenance-acceptance-*-retryN`,
`~/.cache/pse-arrow/plan37-completion-*`, `.local/plan6x-*`). Nothing records their owning plan,
and plan closeout doesn't release them. Plans cite absolute `$HOME/.local/state/...` paths as
evidence (plans 34 and 35), which ties a 5–30 GiB live database to a sentence in a document.

### 3.4 Evidence written to `/tmp`

Plan 65 cites `/tmp/pse-bc-*` and `/tmp/pse-plan65-study-capture-*/journey-live` as evidence. `/tmp`
here sits on the root disk and systemd ages it out after 30 days, so this evidence disappears
silently. The other way round, ~96 `/tmp/pse-*` dirs (11 GiB) are scratch that nothing cleans
before then. `paths.rs` already defines `scratch()`, a session scratch root under checkout state,
but these runs bypass it. Temp dirs are also being created directly in the state root
(`~/.local/state/pse-arrow/ordinary-*`, mkdtemp-style).

### 3.5 Tool and native generations accumulate

- `~/.local/share/pse-arrow/tools/surreal` held 25 Surreal builds. Each keeps a full `source/`
  tree (~2.3 GiB), and they are linked through a reference graph (`selection.json` →
  `release.json`, `rust-v1/generations/*/release.json`, fixture `config.json`/`service-launch.json`).
  Nothing collects unreferenced builds.
- `~/.cache/pse-arrow/native` (9 GiB): "automatic generation collection is excluded".
- `target/host-tools/generation.*`: 3 at present, with no stated retention.

### 3.6 systemd user unit files leak

`~/.config/systemd/user` holds 555 `pse-surreal-*.service` files, 8 `pse-compiler-cache-*.service`
and 4 `pse-telemetry-*` units. Of the 84 loaded `pse-surreal-*` units, 1 is running, 79 are
inactive and 3 have failed. They take little disk but show the same missing teardown, and they
slow `systemctl --user` and clutter diagnostics.

### 3.7 Build directories and caches

- `target/` per checkout (ADR-0122) is sound, but feature-set powerset and coverage runs (for
  example `pse-bc-powerset-coverage`) produce many `pse_runtime` variants. Each variant gets its
  own incremental dir and roughly 1 GiB of test binaries in the `build/<pkg>/<hash>/` layout. Today
  `target/debug` has 393 incremental dirs and 2,393 build-script dirs, even after a recent clean.
  Incremental state only pays off in interactive edit loops; agent verification runs keep paying
  for it.
- sccache: `docs/dev/build-performance.md` keeps "the 100 GiB disk budget". A second 100 GiB cap
  applies to the global `~/.cache/sccache`. On Oct 26, 145 GiB of entries older than 48 h were
  removed from the two caches.
- Repo root clutter: rustc ICE dumps land in the repo root (14 were removed), along with stray
  `chemicalfun.log`, `thermofun.log`, `history.txt`, `Untitled` and `__pycache__`.

### 3.8 Nothing reports or budgets disk use

No command reports pse-arrow's footprint by class (ledgers, stores, fixtures, tools, caches,
orphans), and admission doesn't consider free disk space. The disk has filled several times;
on Oct 16 it crashed PostgreSQL mid-COPY.

## 4. Recommended improvements

Ordered by space impact. The first three would have prevented about 90% of what was removed.

1. **Make checkout removal take its state with it.**
   - Write a small `checkout.json` (root path, git common dir, created time) when a state dir is
     created, so ownership is explicit.
   - Ship `pse worktree rm` (R-56). It removes the worktree, its state dir and its target.
   - Add `pse state gc --orphans`, which lists or removes state dirs whose recorded root no
     longer exists, respecting pins (item 4). Agents can then remove worktrees as AGENTS.md asks
     without leaking state.
   - Keep worktrees under one parent (for example the existing empty `~/pse-arrow-wt/`) rather than
     ad hoc siblings in `~` and `~/.cache`, so they stay discoverable.

2. **Stop copying binaries into ledgers.**
   - The custody claim needs a digest, not a copy. Record SHA-256, size and build identity, and
     either hard-link into a content-addressed blob store under the state root (same filesystem;
     identical `cargo-nextest`/`cargo-pse-correctness` then cost nothing) or keep no copy at all.
   - If an exact rerun of an old binary is a real requirement, keep the full copies only for the
     last N invocations or for pinned ones.
   - Once an invocation settles, compact it to its records.

3. **Turn retention eligibility into actual collection, with a budget.**
   - Amend ADR-0234/0168 so that each retention class has a default bound (age, count or bytes)
     rather than "never".
   - Call `RetentionClass::eligible()` from a collector that runs at invocation settle and at
     host start.
   - Keep the explicit-owner principle for scientific data and fixtures, but give
     presentation/ledger material a default lifetime. "Protected unless an owner deletes" means,
     in practice, "kept forever".

4. **Add pins for cited evidence.** Plans cite invocation IDs (plan 65 cites 29) and state paths.
   Provide `pse evidence pin <invocation|path> --by <plan>`, and have collectors skip pinned items.
   At plan closeout, either export the evidence into the repo (small JSON or reports under
   `build/` or the plan's evidence dir) and unpin, or keep the pin deliberately. Don't cite live
   multi-GiB databases or `/tmp` paths as evidence.

5. **Declare plan-scoped stores and release them at closeout.** Named stores should record
   their owning plan (`owner.json`), and plan closeout should release them. A single sentence in
   the plan-closeout guidance, in the same style as the existing worktree note, may be enough.
   `pse state ls` can then flag stores whose owning plan is done.

6. **Route scratch through `scratch()` and clean it per session.** Replace `/tmp/pse-*`,
   `ordinary-*` in the state root, `~/.cache/pse-arrow/<probe>` and `.local/plan*` with the
   existing session scratch root, removed when the session settles unless pinned.

7. **Bound tool and native generations.**
   - For Surreal builds, keep the selected build plus one previous and anything referenced by a
     live fixture or a pin, and drop the `source/` copy after a build is published.
   - Apply the same rule to `~/.cache/pse-arrow/native` and `target/host-tools` generations.

8. **Remove unit files with their services.** Use transient units (`systemd-run --user`, no unit
   file) for per-fixture and per-session Surreal, compiler-cache and telemetry services, or delete
   the unit file on stop and on fixture disposal.

9. **Cut build-dir growth for agent runs.**
   - Set `CARGO_INCREMENTAL=0` for gate, verification and powerset runs, and keep incremental
     builds for interactive loops.
   - Run powerset/feature-matrix campaigns in a dedicated target subdirectory that the campaign
     removes when it finishes.
   - Prune `target/debug/incremental/*` sessions older than about 24 h.
   - Point rustc ICE dumps at `target/` (or disable them for agent runs).

10. **Right-size sccache.** A pse-arrow cap of about 40 GiB is likely enough once ledgers and
    targets are bounded; measure the hit rate before and after.

11. **Make footprint visible and gate on it.**
    - Add `pse state du` (or a storage section in the doctor) that reports usage per class and
      per checkout, plus orphans, unpinned evidence and the free-space trend.
    - Have host admission refuse or warn on new effectful work when free space drops below a
      threshold (for example 100 GiB), naming the largest collectable classes.

## 5. Suggested first steps

1. Read section 3 against current `main` (paths.rs, ADR-0168, ADR-0234, agent-environment.md
   "Worktrees", build-performance.md) and confirm or correct the findings.
2. Decide the policy change in items 2–4 (ADR amendment). Everything else depends on deletion
   being allowed by default for ledger and presentation material.
3. Implement items 1, 2 and 4 first. They address about 90% of the observed growth.
4. Check `~/.local/state/pse-arrow/cac78ee32c21…` (left on purpose), the `fixtures/` contents
   (eleven `surreal-b1-*` restores and eight `pse51ef-*` recovery fixtures) and the 555 unit files.
