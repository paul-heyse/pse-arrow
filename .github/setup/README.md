# Declared GitHub configuration

Everything about the repository that is not a file in the repository lives here as JSON:
merge settings, topics, rulesets, environments. `just gh-setup` applies it with `gh api`
and is **idempotent** — a second run is a no-op apart from the API calls, and a re-run is
how a change to one of these files reaches GitHub.

Why `gh api` and not `gh ruleset`: `gh ruleset` is read-only in gh 2.45. Rulesets are
therefore created by `POST` and updated by `PUT` against the id you look up by name.

`just gh-setup` runs `scripts/gh-setup.sh`, which performs steps 2 to 8 below;
`scripts/gh-setup.sh --dry-run` prints the calls without making them. The default
applies `ruleset-main-full.json`; `--full` is a compatibility alias.
`just gh-setup-check` compares live settings without changing them. **This file is the
reference for what those calls are** — when the script and this document disagree, one
of them is a bug, and the fix belongs in the script.

Everything below assumes:

```bash
export OWNER=paul-heyse
export REPO=pse-arrow
export SETUP=.github/setup
gh auth status   # must be paul-heyse, with the `admin:org`-free but repo-admin scope
```

| File | Applied to |
|---|---|
| `repo.json` | `PATCH /repos/{owner}/{repo}` — merge policy, features, secret scanning |
| `topics.json` | `PUT /repos/{owner}/{repo}/topics` |
| `ruleset-main.json` | historical bootstrap subset; no longer applied |
| `ruleset-main-full.json` | active `main` branch ruleset, including Python checks |
| `ruleset-tags.json` | the `tags` ruleset on `refs/tags/v*` |
| `env-test-release.json` | the `test-release` environment (TestPyPI) |
| `env-release.json` | the `release` environment (PyPI) |

---

## 1. Create the repository and push

```bash
gh repo create "$OWNER/$REPO" \
  --public \
  --disable-wiki \
  --description "$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["description"])' "$SETUP/repo.json")" \
  --homepage "https://paul-heyse.github.io/pse-arrow"

git remote add origin "https://github.com/$OWNER/$REPO.git"
git push -u origin main
git push origin --tags          # design-rev2, design-rev3
```

Push **before** step 4: the `main` ruleset requires status checks that only exist once
the workflows are on the default branch.

## 2. Repository settings and topics

```bash
gh api -X PATCH "repos/$OWNER/$REPO" --input "$SETUP/repo.json"
gh api -X PUT   "repos/$OWNER/$REPO/topics" --input "$SETUP/topics.json"
```

`repo.json` sets: default branch `main`; **squash merge only** (`allow_merge_commit` and
`allow_rebase_merge` false) with the commit title taken from the PR title and the body
from the PR body — that is what makes `governance / pr-title` and git-cliff meaningful;
delete branch on merge; allow update branch; Discussions on; wiki and projects off.

`topics` needs the `application/vnd.github.mercy-preview+json` accept header on very old
`gh` versions; current `gh` sends it automatically.

## 3. Security features

Four separate endpoints — none of them belongs in `repo.json`:

```bash
gh api -X PUT "repos/$OWNER/$REPO/vulnerability-alerts"            # Dependabot alerts
gh api -X PUT "repos/$OWNER/$REPO/automated-security-fixes"        # Dependabot security PRs
gh api -X PUT "repos/$OWNER/$REPO/private-vulnerability-reporting" # SECURITY.md's front door
```

Secret scanning and **push protection** are in `repo.json` under `security_and_analysis`
and are applied by step 2. Verify all of it with:

```bash
gh api "repos/$OWNER/$REPO" \
  --jq '{secret_scanning: .security_and_analysis.secret_scanning.status,
         push_protection: .security_and_analysis.secret_scanning_push_protection.status}'
gh api "repos/$OWNER/$REPO/private-vulnerability-reporting" --jq '.enabled'
```

## 4. Rulesets

Rulesets are matched **by name**, so the apply step is "look up the id, `PUT` if it
exists, `POST` if it does not":

```bash
apply_ruleset() {            # $1 = path to the ruleset JSON
  local name id
  name="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["name"])' "$1")"
  id="$(gh api "repos/$OWNER/$REPO/rulesets" --jq \
        ".[] | select(.name == \"$name\") | .id")"
  if [ -n "$id" ]; then
    gh api -X PUT  "repos/$OWNER/$REPO/rulesets/$id" --input "$1"
  else
    gh api -X POST "repos/$OWNER/$REPO/rulesets"      --input "$1"
  fi
}

apply_ruleset "$SETUP/ruleset-main.json"
apply_ruleset "$SETUP/ruleset-tags.json"
```

**`main`** enforces: no deletion, no non-fast-forward, linear history, signed commits,
pull request required (0 approvals — a sole maintainer cannot approve their own PR;
stale reviews dismissed; threads must be resolved; squash only), and the phase-0 required
status checks with `strict_required_status_checks_policy` (the branch must be up to date
with `main` before merge).

**`tags`** restricts creation, update and deletion of `refs/tags/v*` to bypass actors and
requires signatures, so only the maintainer can cut a release tag.

### The bypass actor, and the fallback

Both rulesets list one bypass actor:

```json
{ "actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "always" }
```

`5` is the built-in **admin** repository role. GitHub occasionally rejects a built-in
role id here with `422 Invalid actors`. If that happens, swap in the maintainer as a
user actor — `218986190` is `paul-heyse`:

```json
{ "actor_id": 218986190, "actor_type": "User", "bypass_mode": "always" }
```

```bash
# Non-destructive rewrite of both files, if and only if the role id is rejected:
python3 - <<'PY'
import json, pathlib
fallback = {"actor_id": 218986190, "actor_type": "User", "bypass_mode": "always"}
for name in ("ruleset-main.json", "ruleset-main-full.json", "ruleset-tags.json"):
    path = pathlib.Path(".github/setup") / name
    data = json.loads(path.read_text())
    data["bypass_actors"] = [fallback]
    path.write_text(json.dumps(data, indent=2) + "\n")
    print("rewrote", path)
PY
```

Confirm the id for any other account with
`gh api users/<login> --jq .id`. Bypass is break-glass only: every bypass push needs a
follow-up issue labeled `governance` (GOVERNANCE.md §4).

### Required checks are staged deliberately

`ruleset-main.json` lists only the checks that report in phase 0:

```text
rust / fmt            rust / clippy        rust / test
rust / codegen-diff   rust / family-check  rust / deny
docs / build          governance / adr-lint governance / pr-title
```

A required check that never reports blocks **every** pull request forever. Once the
Python package first reports on a pull request, re-run with the full list:

```bash
apply_ruleset "$SETUP/ruleset-main-full.json"    # adds python / lint, test, parity
```

`ruleset-main-full.json` has the same `"name": "main"`, so this updates the existing
ruleset rather than creating a second one. `python / test` is a single aggregating job
that `needs` the interpreter matrix — the matrix legs report as
`python / test (3.11)` and `python / test (3.14)`, which are *not* required contexts,
and cannot be, because a matrix leg's check name carries its matrix values.

## 5. Labels

```bash
scripts/labels-sync.sh                 # or: just labels-sync
scripts/labels-sync.sh --dry-run       # to see what it would send
```

Declared in `.github/labels.yml`. The script uses `gh label create --force`, which
creates or updates in place, and never deletes.

## 6. Milestones

One per delivery phase. `gh` has no milestone command, so use the API; this is
idempotent because it checks for the title first:

```bash
for m in "Phase 0 — Foundations" "Phase 1 — Slice A" "Phase 2 — Slice B" \
         "Phase 3 — Slice C" "Phase 4 — Breadth"; do
  if ! gh api "repos/$OWNER/$REPO/milestones" --jq '.[].title' | grep -qxF "$m"; then
    gh api -X POST "repos/$OWNER/$REPO/milestones" -f "title=$m"
  fi
done
```

## 7. Environments

Two environments gate publishing. The `PUT` creates or updates the environment; the
deployment branch policy is a **second** call, because the `PUT` body only declares
*that* custom policies are used, not what they are.

```bash
gh api -X PUT "repos/$OWNER/$REPO/environments/test-release" \
  --input "$SETUP/env-test-release.json"
gh api -X PUT "repos/$OWNER/$REPO/environments/release" \
  --input "$SETUP/env-release.json"

policy() {          # $1 = env, $2 = pattern, $3 = branch|tag
  gh api "repos/$OWNER/$REPO/environments/$1/deployment-branch-policies" \
    --jq '.branch_policies[].name' | grep -qxF "$2" \
  || gh api -X POST "repos/$OWNER/$REPO/environments/$1/deployment-branch-policies" \
       -f "name=$2" -f "type=$3"
}

# test-release: the `main` branch, AND the `v*` tags that actually trigger release.yml
policy test-release 'main' branch
policy test-release 'v*'   tag

# release: only a `v*` tag
policy release 'v*' tag
```

**Why `test-release` carries a tag policy as well as `main`.** `release.yml` is triggered
by pushing a `v*` tag, so the run's ref is `refs/tags/v0.1.0`, not `refs/heads/main`. An
environment restricted to the `main` *branch* would reject that deployment and the
release would stall at the TestPyPI step with a confusing "waiting for approval" state.
The `main` branch policy is kept so the environment can also be used from a branch
workflow later; the tag policy is the one release.yml actually matches.

> **Open item.** `scripts/gh-setup.sh` currently adds only the `main` *branch* policy for
> `test-release`. Add the `v*` *tag* policy — by running the `policy test-release 'v*'
> tag` call above once, or by teaching the script to apply two policies — before cutting
> the first release, or `release.yml` will stall at the TestPyPI step.

`env-release.json` sets the maintainer as a required reviewer with
`prevent_self_review: false` — a sole maintainer must be able to approve their own
release deployment, which is exactly why this is a manual approval gate and not an
automatic one.

## 8. GitHub Pages from the workflow

```bash
gh api -X POST "repos/$OWNER/$REPO/pages" -f 'build_type=workflow' 2>/dev/null \
  || gh api -X PUT "repos/$OWNER/$REPO/pages" -f 'build_type=workflow'
gh api "repos/$OWNER/$REPO/pages" --jq '{build_type, html_url}'
```

`POST` creates the Pages site the first time and returns `409` afterwards; `PUT` updates
it. `docs / deploy` then publishes with `actions/deploy-pages` using the `github-pages`
environment, which GitHub creates on its own.

## 9. PyPI trusted publishing — not a `gh` command

Trusted publishing is configured on PyPI, not on GitHub, and has to be done in a browser
**before** the first release:

- <https://pypi.org/manage/account/publishing/> and
  <https://test.pypi.org/manage/account/publishing/>
- PyPI project name `pse-arrow`, owner `paul-heyse`, repository `pse-arrow`,
  workflow `release.yml`, environment `release` (TestPyPI: environment `test-release`).

Create these as **pending publishers** — the project does not exist on either index yet.
No API token is ever stored in this repository; `release.yml` requests `id-token: write`
and `uv publish --trusted-publishing always` exchanges the OIDC token.

## 10. Verify

```bash
gh api "repos/$OWNER/$REPO" --jq \
  '{default_branch, allow_squash_merge, allow_merge_commit, allow_rebase_merge,
    delete_branch_on_merge, has_discussions, has_wiki, has_projects}'
gh api "repos/$OWNER/$REPO/rulesets" --jq '.[] | {name, target, enforcement}'
gh api "repos/$OWNER/$REPO/rulesets/$(gh api "repos/$OWNER/$REPO/rulesets" \
  --jq '.[] | select(.name=="main") | .id')" \
  --jq '.rules[] | select(.type=="required_status_checks")
        | .parameters.required_status_checks[].context'
gh api "repos/$OWNER/$REPO/environments" --jq '.environments[].name'
gh label list --repo "$OWNER/$REPO" --limit 100 | wc -l     # expect 35
```

Then run `just gh-setup-check`, re-run `just gh-setup`, and check again: zero
configuration differences, no duplicate rulesets or environments. Labels are managed
separately with `just labels-sync`. Publishing remains a separate release operation.
