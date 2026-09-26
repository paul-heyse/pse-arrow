---
description: GitHub Actions conventions and the gotchas that have broken workflows
paths:
  - ".github/**"
---

# Editing workflows

When reviewing workflow edits, `just lint-repo` runs `actionlint` and `zizmor` over
`.github/workflows`. It is a manual check, not a push prerequisite.

## Gotchas

- **GitHub Actions does not support YAML anchors.** Duplicate the block and say why in a
  comment above it.
- **`yaml.safe_load` accepts duplicate keys silently** (last wins), so a plain parse
  check will happily pass a job whose second `if:` disabled the first. Only actionlint
  sees it.
- **`windows-*` runners default to PowerShell.** Any bash needs
  `defaults.run.shell: bash` at the job level, or the step dies with a `ParserError`.
- **A `#` line inside a `>-` folded scalar is content, not a comment** — it gets
  evaluated as part of the expression. Put notes above the key.
- **An empty env var is not an unset one.** `FOO: ${{ cond && 'x' || '' }}` sets `FOO=""`.
  Use a conditional step instead.

## Conventions

- **Every action is pinned by commit SHA** with the version in a trailing comment.
  `pinact` keeps them current and Dependabot opens the bumps; a tag reference will not
  pass review.
- `permissions: contents: read` at the top of every workflow, widened per job only where
  a job actually needs it.
- Tool versions are env vars at workflow level and come from the same pins as local
  development. Never `pip install ruff` or `cargo install` an unpinned tool in a job.
- `concurrency` per ref for manually dispatched checks.
- No `RUSTFLAGS` in CI, and `sccache` stays off there — both change what is being
  verified relative to a local `just ci-fast`.
- Jobs that need a solver run in the pinned `ghcr.io/paul-heyse/pse-solvers` image
  referenced by digest, not by tag.
- Every test invocation passes `--features pse-relations/force-validate`. A job that
  drops it is running a weaker check than `just test` and will not say so.
- Check workflows use `workflow_dispatch` only. Keep publishing inputs explicit and
  do not reintroduce required status checks into the `main` ruleset.
