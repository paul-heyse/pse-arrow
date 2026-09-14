#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only comparison of GitHub settings with the declared full configuration."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SETUP = ROOT / ".github/setup"
REPO = os.environ.get("PSE_GH_REPO", "paul-heyse/pse-arrow")


def api(path: str) -> object:
    result = subprocess.run(
        ["gh", "api", f"repos/{REPO}/{path}".rstrip("/")],
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(result.stdout) if result.stdout.strip() else None


def compare(expected: object, actual: object, path: str = "") -> list[str]:
    """Compare declared fields, ignoring server-owned fields and list ordering."""
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            return [f"{path}: missing object"]
        return [
            error
            for key, value in expected.items()
            for error in compare(value, actual.get(key), f"{path}/{key}")
        ]
    if isinstance(expected, list):
        if not isinstance(actual, list) or len(expected) != len(actual):
            return [f"{path}: expected {len(expected)} entries, got {actual!r}"]
        remaining = list(actual)
        for value in expected:
            match = next(
                (
                    i
                    for i, candidate in enumerate(remaining)
                    if not compare(value, candidate, path)
                ),
                None,
            )
            if match is None:
                return [f"{path}: missing declared entry {value!r}"]
            remaining.pop(match)
        return []
    return [] if expected == actual else [f"{path}: want {expected!r}, got {actual!r}"]


def declared(filename: str) -> dict:
    return json.loads((SETUP / filename).read_text())


def api_object(path: str) -> dict:
    result = api(path)
    if not isinstance(result, dict):
        raise TypeError(f"{path}: expected a JSON object")
    return result


def api_list(path: str) -> list:
    result = api(path)
    if not isinstance(result, list):
        raise TypeError(f"{path}: expected a JSON array")
    return result


def check() -> list[str]:
    errors = compare(declared("repo.json"), api(""), "repo")
    errors += compare(declared("topics.json"), api("topics"), "topics")
    rules = api_list("rulesets")
    for filename in ("ruleset-main-full.json", "ruleset-tags.json"):
        wanted = declared(filename)
        found = next((r for r in rules if r["name"] == wanted["name"]), None)
        if found is None:
            errors.append(f"missing ruleset: {wanted['name']}")
        else:
            errors += compare(wanted, api(f"rulesets/{found['id']}"), wanted["name"])
    for name, policies in (
        ("release", [("v*", "tag")]),
        ("test-release", [("main", "branch"), ("v*", "tag")]),
    ):
        wanted = declared(f"env-{name}.json")
        actual = api_object(f"environments/{name}")
        errors += compare(
            wanted["deployment_branch_policy"], actual["deployment_branch_policy"], name
        )
        wait_timer = next(
            (
                r["wait_timer"]
                for r in actual["protection_rules"]
                if r["type"] == "wait_timer"
            ),
            0,
        )
        errors += compare(wanted["wait_timer"], wait_timer, name + "/wait-timer")
        reviewers = next(
            (
                r
                for r in actual["protection_rules"]
                if r["type"] == "required_reviewers"
            ),
            {},
        )
        observed = [
            {"type": r["type"], "id": r["reviewer"]["id"]}
            for r in reviewers.get("reviewers", [])
        ]
        errors += compare(wanted["reviewers"], observed, name + "/reviewers")
        if wanted["reviewers"]:
            errors += compare(
                wanted["prevent_self_review"],
                reviewers.get("prevent_self_review"),
                name + "/self-review",
            )
        branches = api_object(f"environments/{name}/deployment-branch-policies")[
            "branch_policies"
        ]
        errors += compare(
            [{"name": n, "type": t} for n, t in policies], branches, name + "/policies"
        )
    errors += compare(
        {"enabled": True}, api("private-vulnerability-reporting"), "security"
    )
    api("vulnerability-alerts")  # Enabled returns 204; failures are not swallowed.
    errors += compare(
        {"enabled": True, "paused": False},
        api("automated-security-fixes"),
        "dependabot",
    )
    errors += compare({"build_type": "workflow"}, api("pages"), "pages")
    return errors


def main() -> int:
    try:
        errors = check()
    except (
        OSError,
        subprocess.CalledProcessError,
        ValueError,
        TypeError,
        KeyError,
    ) as exc:
        print(f"GitHub configuration check failed: {exc}", file=sys.stderr)
        return 1
    for error in errors:
        print(error, file=sys.stderr)
    print(f"GitHub configuration: {len(errors)} difference(s), baseline zero")
    return int(bool(errors))


if __name__ == "__main__":
    sys.exit(main())
