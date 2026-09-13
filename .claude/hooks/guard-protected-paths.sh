#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# PreToolUse on Edit|Write: refuse writes to territory this session does not own.
#
# Three kinds of territory:
#   * output      -- generated sources, build trees, vendored checkouts. Editing the
#                    output instead of the generator produces a red `just codegen-check`
#                    and a change that the next regeneration silently reverts.
#   * decided     -- the blueprint and accepted ADRs. Both are amended through a
#                    labelled decision PR, never in passing.
#   * VCS state   -- .git/.
#
# Exit 2 blocks the call and hands stderr back to the model as the reason.
# PSE_DESIGN_EDIT=1 lifts the "decided" guard for a session that is deliberately
# doing decision work; it never lifts the "output" guard.
set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"
cd "$root" 2>/dev/null || exit 0

file="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))' 2>/dev/null)"
[ -n "$file" ] || exit 0

rel="${file#"$root"/}"
rel="${rel#./}"

block() {
  echo "BLOCKED: $rel" >&2
  printf '%s\n' "$@" >&2
  exit 2
}

case "$rel" in
  .git/*|*/.git/*)
    block "This is VCS-internal state, not source."
    ;;
  target/*|build/*|external/*)
    block "target/, build/ and external/ are not source: they are build output and" \
          "pinned read-only checkouts (\`just fetch-external\`). Nothing here is committed." \
          "Read external/ for behaviour; this repository is a clean-room implementation," \
          "so never copy from it either."
    ;;
  docs/generated/*|crates/*/src/generated/*|python/pse/contracts/*|crates/pse-ipopt-sys/src/bindings.rs)
    block "You are looking at generator output. Fix the generator (crates/pse-schema, or" \
          "the bindgen allowlist in xtask for the Ipopt bindings), then run \`just codegen\`." \
          "\`just codegen-check\` diffs these paths in CI, so a hand edit here is a red build" \
          "and the next regeneration reverts it."
    ;;
  docs/authoritative_design/*)
    [ "${PSE_DESIGN_EDIT:-0}" = "1" ] && exit 0
    block "docs/authoritative_design/ is the authoritative design. It is amended by a" \
          "\`design:\` PR that adds a revision row and cites the ADR that decided the change," \
          "not by an edit in passing. Section numbers are stable citation targets: insert" \
          "§14.3.1, never renumber." \
          "If amending the blueprint IS the work, set PSE_DESIGN_EDIT=1 and say so in the PR."
    ;;
  docs/adr/[0-9][0-9][0-9][0-9]-*.md)
    [ "${PSE_DESIGN_EDIT:-0}" = "1" ] && exit 0
    status="$(awk 'NR > 1 && /^---[[:space:]]*$/ { exit } /^status:[[:space:]]*/ { print $2; exit }' "$file" 2>/dev/null)"
    case "$status" in
      ""|proposed) exit 0 ;;
      *)
        block "This ADR is \`status: $status\`. An accepted record is immutable except for its" \
              "status fields; a decision changes by being superseded (\`just adr-supersede <old> <new>\`)," \
              "which writes symmetric links and a status-history entry. \`just adr-lint\` enforces this" \
              "against origin/main." \
              "If changing the record IS the work, set PSE_DESIGN_EDIT=1 and say so in the PR."
        ;;
    esac
    ;;
esac
exit 0
