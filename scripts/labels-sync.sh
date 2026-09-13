#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Apply .github/labels.yml to the repository with `gh label create --force`, which
# creates a label or updates its colour and description in place. Idempotent: a second
# run is a no-op apart from the API calls.
#
# This script never deletes a label. Removing one from labels.yml leaves it on the
# repository; delete it by hand (`gh label delete <name>`) once you have confirmed
# nothing is filtering on it. `--dry-run` prints what would be sent and calls nothing.
#
# The file is parsed by a strict stdlib-only Python parser (no PyYAML — this must work
# from a bare checkout before `just bootstrap` has created .venv). The accepted shape is
# documented at the top of .github/labels.yml; anything else is a hard error rather than
# a silent skip, because a mis-parsed label set is worse than no sync at all.
#
# Usage: scripts/labels-sync.sh [--dry-run] [--repo <owner/name>]
#        just labels-sync

set -euo pipefail

REPO="paul-heyse/pse-arrow"
DRY_RUN=0

while [ $# -gt 0 ]; do
	case "$1" in
	--dry-run)
		DRY_RUN=1
		shift
		;;
	--repo)
		REPO="${2:?--repo needs an owner/name argument}"
		shift 2
		;;
	-h | --help)
		sed -n '3,20p' "$0"
		exit 0
		;;
	*)
		printf 'labels-sync: unknown argument: %s\n' "$1" >&2
		exit 2
		;;
	esac
done

ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
LABELS_FILE="$ROOT/.github/labels.yml"

if [ ! -f "$LABELS_FILE" ]; then
	printf 'labels-sync: not found: %s\n' "$LABELS_FILE" >&2
	exit 1
fi

if [ "$DRY_RUN" -eq 0 ] && ! command -v gh >/dev/null 2>&1; then
	printf 'labels-sync: gh is not installed; see https://cli.github.com\n' >&2
	exit 1
fi

# Emits one tab-separated "name<TAB>color<TAB>description" line per label. Tabs cannot
# appear in a value because the parser rejects them, so the separator is unambiguous.
parse_labels() {
	python3 - "$LABELS_FILE" <<-'PY'
		import re
		import sys

		path = sys.argv[1]
		ENTRY = re.compile(r'^- name: "([^"\t]+)"$')
		COLOR = re.compile(r'^  color: "([0-9a-f]{6})"$')
		DESC = re.compile(r'^  description: "([^"\t]*)"$')

		def die(lineno: int, line: str, why: str) -> None:
		    sys.exit(f"labels-sync: {path}:{lineno}: {why}\n  {line!r}")

		with open(path, encoding="utf-8") as handle:
		    lines = handle.read().splitlines()

		rows: list[tuple[str, str, str]] = []
		seen: set[str] = set()
		index = 0
		while index < len(lines):
		    line = lines[index]
		    stripped = line.strip()
		    if not stripped or stripped.startswith("#"):
		        index += 1
		        continue
		    entry = ENTRY.match(line)
		    if not entry:
		        die(index + 1, line, 'expected a label entry: - name: "<name>"')
		    if index + 2 >= len(lines):
		        die(index + 1, line, "entry is truncated: color and description must follow")
		    color = COLOR.match(lines[index + 1])
		    if not color:
		        die(index + 2, lines[index + 1], 'expected: color: "<6 lowercase hex digits>"')
		    desc = DESC.match(lines[index + 2])
		    if not desc:
		        die(index + 3, lines[index + 2], 'expected: description: "<one line>"')
		    name = entry.group(1)
		    if name in seen:
		        die(index + 1, line, f"duplicate label name: {name}")
		    if len(desc.group(1)) > 100:
		        die(index + 3, lines[index + 2], "description exceeds GitHub's 100-character limit")
		    seen.add(name)
		    rows.append((name, color.group(1), desc.group(1)))
		    index += 3

		if not rows:
		    sys.exit(f"labels-sync: {path}: no labels found")

		for name, color, description in rows:
		    print(f"{name}\t{color}\t{description}")
	PY
}

count=0
while IFS=$'\t' read -r name color description; do
	[ -n "$name" ] || continue
	count=$((count + 1))
	if [ "$DRY_RUN" -eq 1 ]; then
		printf 'would sync: %-24s #%s  %s\n' "$name" "$color" "$description"
		continue
	fi
	printf 'syncing: %s\n' "$name"
	gh label create "$name" \
		--repo "$REPO" \
		--color "$color" \
		--description "$description" \
		--force
done < <(parse_labels)

if [ "$DRY_RUN" -eq 1 ]; then
	printf '\nlabels-sync: %d labels parsed; nothing sent (--dry-run).\n' "$count"
else
	printf '\nlabels-sync: %d labels synced to %s.\n' "$count" "$REPO"
	printf 'labels-sync: labels removed from labels.yml are NOT deleted; do that by hand.\n'
fi
