# Manual CI checks

The maintainer runs CI when it is useful to review the project. Checks are not part of a
commit, push, pull request, merge, or plan-close process. The `main` ruleset has no
required status checks or required pull request. Local Git hooks are not installed by
`just bootstrap`.

Every check workflow in `.github/workflows/` uses `workflow_dispatch`. Use the GitHub
Actions **Run workflow** button or `gh workflow run <file>.yml --ref <branch>` to run a
committed ref. A workflow file must exist on the default branch before GitHub can
dispatch it. Local recipes inspect the working tree, including uncommitted work; GitHub
runs inspect only the selected ref.

| Workflow | Manual purpose |
|---|---|
| `rust.yml` | Rust formatting, Clippy, tests, codegen, family, docs, coverage and advisory checks. |
| `python.yml` | Python lint, interpreter matrix and parity. |
| `docs.yml` | Build the book with `python3 -m scripts.docs build` and check offline links; `deploy=true` on `main` also publishes Pages. |
| `governance.yml` | ADR, register, licence and agent configuration checks. |
| `repo-hygiene.yml` | Workflow, spelling, TOML and shell lint. |
| `rust-scheduled.yml` | Optional deeper Rust checks; the historical filename no longer denotes a schedule. |
| `register-review.yml` | Review due register rows and open or update a review issue. |
| `solvers-image.yml` | Build and smoke-test the solver image; `publish=true` on `main` also publishes it and prepares a pin update pull request. |
| `wheels.yml` | Manually build wheel/sdist artifacts; the release workflow can also call it. |

`release.yml` is the separate publication flow triggered by a deliberate signed `v*`
tag. It retains its release checks and publishing environments. Ordinary commits and
pushes do not trigger it.

The local recipes remain available on demand: `just ci-fast` for a Rust aggregate,
`just ci-pr` for the broader local aggregate, `just parity-container` for solver-backed
parity, `just quality` for Python static checks, `just docs` for the site and
`just lint-repo` for workflow lint. `just --list` describes the full command surface.
Run only the checks relevant to the question being investigated. When reporting a run,
name its command, mode, scope and result against the zero-failure target; a passing
subset is not complete product qualification.

Workflow and repository settings live in `.github/workflows/` and `.github/setup/`.
`just gh-setup-check` compares declared settings with GitHub; `just --yes gh-setup`
reapplies them when intentionally requested. Neither is a commit or push hook.
