#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Apply the declared GitHub configuration in .github/setup/ to the repository,
# idempotently (PUT/PATCH; rulesets are matched by name and updated in place).
#
#   scripts/gh-setup.sh            declared main ruleset (ruleset-main-full.json)
#   scripts/gh-setup.sh --full     compatibility alias for the default
#   scripts/gh-setup.sh --check    read-only comparison with declared settings
#   scripts/gh-setup.sh --dry-run  print what would be sent
#
# `gh ruleset` is read-only in gh 2.45, so rulesets go through `gh api`. Creating the
# repository itself is a one-off (`gh repo create`, see .github/setup/README.md).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
SETUP=".github/setup"
REPO="${PSE_GH_REPO:-paul-heyse/pse-arrow}"
MAIN_RULESET="$SETUP/ruleset-main-full.json"
DRY=0
for arg in "$@"; do
  case "$arg" in
    --full) MAIN_RULESET="$SETUP/ruleset-main-full.json" ;;
    --check) exec python3 scripts/github-config.py ;;
    --dry-run) DRY=1 ;;
    *) echo "usage: $0 [--full] [--dry-run] [--check]" >&2; exit 2 ;;
  esac
done

api() { # method path [--input file | -f k=v ...]
  if [ "$DRY" = 1 ]; then echo "gh api -X $*"; return 0; fi
  gh api -X "$@" >/dev/null
}

say() { printf '\033[1m==> %s\033[0m\n' "$*"; }

say "repository settings ($REPO)"
api PATCH "repos/$REPO" --input "$SETUP/repo.json"
api PUT "repos/$REPO/topics" --input "$SETUP/topics.json"

say "security features"
api PUT "repos/$REPO/vulnerability-alerts"
api PUT "repos/$REPO/automated-security-fixes"
api PUT "repos/$REPO/private-vulnerability-reporting"


apply_ruleset() { # file
  local file="$1" name id
  name="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["name"])' "$file")"
  id="$(gh api "repos/$REPO/rulesets" --jq ".[] | select(.name == \"$name\") | .id")"
  if [ -n "$id" ]; then
    say "ruleset '$name' (update #$id)"
    api PUT "repos/$REPO/rulesets/$id" --input "$file"
  else
    say "ruleset '$name' (create)"
    api POST "repos/$REPO/rulesets" --input "$file"
  fi
}
apply_ruleset "$MAIN_RULESET"
apply_ruleset "$SETUP/ruleset-tags.json"

apply_environment() { # name file policy-name policy-type
  local name="$1" file="$2" pname="$3" ptype="$4"
  say "environment '$name'"
  api PUT "repos/$REPO/environments/$name" --input "$file"
  # Deployment branch/tag policy, idempotent by name.
  if [ "$DRY" = 0 ]; then
    if ! gh api "repos/$REPO/environments/$name/deployment-branch-policies" \
         --jq '.branch_policies[].name' 2>/dev/null | grep -qx "$pname"; then
      gh api -X POST "repos/$REPO/environments/$name/deployment-branch-policies" \
        -f "name=$pname" -f "type=$ptype" >/dev/null
    fi
  else
    echo "gh api -X POST repos/$REPO/environments/$name/deployment-branch-policies -f name=$pname -f type=$ptype"
  fi
}
# release.yml runs on a tag ref, so test-release needs the tag policy as well as main.
apply_environment test-release "$SETUP/env-test-release.json" main branch
apply_environment test-release "$SETUP/env-test-release.json" 'v*' tag
apply_environment release "$SETUP/env-release.json" 'v*' tag

say "pages (workflow deploy)"
if [ "$DRY" = 0 ]; then
  gh api "repos/$REPO/pages" >/dev/null 2>&1 \
    || gh api -X POST "repos/$REPO/pages" -f build_type=workflow >/dev/null
else
  echo "gh api -X POST repos/$REPO/pages -f build_type=workflow"
fi

say "milestones"
for m in "Phase 0 — Foundations" "Phase 1 — Slice A" "Phase 2 — Slice B" "Phase 3 — Slice C" "Phase 4 — Breadth"; do
  if [ "$DRY" = 0 ]; then
    gh api "repos/$REPO/milestones?state=all" --jq '.[].title' | grep -qxF "$m" \
      || gh api -X POST "repos/$REPO/milestones" -f "title=$m" >/dev/null
  else
    echo "gh api -X POST repos/$REPO/milestones -f title='$m'"
  fi
done
say "done"
