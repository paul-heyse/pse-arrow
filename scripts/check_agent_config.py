#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Verify the agent working environment is internally consistent.

Agent instructions are the one class of document with no compiler and no test: a
`CLAUDE.md` that points at a directory which was renamed six months ago fails silently,
at the moment an agent tries to follow it. This script is the missing compiler. It
checks four things:

1. **The two symlinks resolve.** Skills are canonical in ``.codex/skills`` with
   ``.claude/skills`` pointing at them; agents are canonical in ``.claude/agents`` with
   ``.codex/agents`` pointing at them. One copy each is what keeps the Claude Code and
   Codex views from drifting. A Windows checkout without developer mode materialises a
   symlink as a directory (or as a one-line text file), so a materialised pair is
   accepted when the two trees are byte-identical -- and rejected when they are not,
   which is the drift this exists to catch.
2. **``CLAUDE.md`` starts with ``@AGENTS.md``.** That import is why ``AGENTS.md`` can be
   canonical for both runtimes.
3. **Every repository path named in the instructions exists.** Paths that are created
   later in the build-out live in ``.claude/path-allowlist.txt``, each with a comment
   saying when it appears; a line carrying ``agent-config: ignore`` is skipped.
4. **Every ``just`` recipe named in the instructions exists.** ``just --list`` is
   advertised as the contract, so a recipe that was renamed without updating the prose
   is a broken contract.

Run by ``just lint-agents`` and by the ``governance / agent-config`` CI job.
Standard library only: it has to run before any environment exists.
"""

from __future__ import annotations

import filecmp
import re
import subprocess
import sys
from fnmatch import fnmatch
from pathlib import Path


def repo_root() -> Path:
    """The repository root, resolved from git so no absolute path is ever hard-coded."""
    try:
        out = subprocess.run(
            ("git", "rev-parse", "--show-toplevel"),
            capture_output=True,
            text=True,
            check=True,
            cwd=Path(__file__).resolve().parent,
            timeout=10,
        ).stdout.strip()
        if out:
            return Path(out)
    except (OSError, subprocess.SubprocessError):
        pass
    return Path(__file__).resolve().parents[1]


ROOT = repo_root()
ALLOWLIST_FILE = ROOT / ".claude" / "path-allowlist.txt"
IGNORE_MARKER = "agent-config: ignore"

# Top-level directories that make a token a repository path rather than prose. A
# match is only a candidate; templated and globbed forms are dropped below.
PATH_ROOTS = (
    "docs",
    "scripts",
    "crates",
    "python",
    "tests",
    "xtask",
    "benches",
    r"\.github",
    r"\.claude",
    r"\.codex",
)
PATH_RE = re.compile(
    r"(?<![\w./-])(?:"
    + "|".join(PATH_ROOTS)
    + r")/[A-Za-z0-9_.*{}<>?-][A-Za-z0-9_./*{}<>?-]*"
)
# A path that is a pattern, not a file: globs in a rules `paths:` list, `{slug}` in a
# filename template, `<name>` in a usage line. Nothing to check on disk.
TEMPLATE_CHARS = set("*{}<>?")

BACKTICK_RE = re.compile(r"`([^`\n]+)`")
FENCE_RE = re.compile(r"^\s*just\s+(\S+)")
FRONT_MATTER_RE = re.compile(r"\A---\n.*?\n---\n", re.DOTALL)


def scanned_files() -> list[Path]:
    """Instruction files whose contents are checked, in report order."""
    files = [ROOT / "AGENTS.md", ROOT / "CLAUDE.md"]
    files += sorted((ROOT / ".claude" / "rules").glob("*.md"))
    files += sorted((ROOT / ".claude" / "agents").glob("*.md"))
    skills = ROOT / ".codex" / "skills"
    if skills.is_dir():
        files += sorted(
            p for p in skills.rglob("*.md") if p.name in {"SKILL.md", "REFERENCE.md"}
        )
    return [f for f in files if f.is_file()]


def body_of(text: str) -> str:
    """Drop YAML front matter.

    A rules file's `paths:` globs and an agent's `description:` few-shot examples are
    not references to files in this repository, and treating them as such would make the
    check a source of noise rather than signal.
    """
    return FRONT_MATTER_RE.sub("", text, count=1)


def load_allowlist() -> list[str]:
    """Patterns from .claude/path-allowlist.txt, minus comments and blank lines."""
    if not ALLOWLIST_FILE.is_file():
        return []
    entries = []
    for line in ALLOWLIST_FILE.read_text(encoding="utf-8").splitlines():
        entry = line.split("#", 1)[0].strip()
        if entry:
            entries.append(entry.rstrip("/"))
    return entries


def allowed(ref: str, allowlist: list[str]) -> bool:
    """Does an allowlist pattern cover `ref`, either exactly or as a directory?"""
    return any(
        fnmatch(ref, pattern) or fnmatch(ref, pattern + "/*") for pattern in allowlist
    )


def path_refs(text: str) -> set[str]:
    """Repository paths named in `text`, excluding ignored lines and templated forms."""
    refs: set[str] = set()
    for line in text.splitlines():
        if IGNORE_MARKER in line:
            continue
        for raw in PATH_RE.findall(line):
            ref = raw.rstrip(".,;:)]`'\"").rstrip("/")
            if not ref or set(ref) & TEMPLATE_CHARS:
                continue
            refs.add(ref)
    return refs


def just_recipes(text: str) -> set[str]:
    """Recipe names invoked as `just <name>` in backticks or in a fenced block."""
    names: set[str] = set()
    for raw in BACKTICK_RE.findall(text):
        span = raw.strip()
        if span.startswith("just "):
            names.add(span[len("just ") :].split()[0])
    for line in text.splitlines():
        match = FENCE_RE.match(line)
        if match:
            names.add(match.group(1))
    return {n for n in names if not n.startswith("-")}


def trees_identical(left: Path, right: Path) -> bool:
    """Byte-for-byte equality of two directory trees (no shallow stat comparison)."""
    left_files = sorted(
        p.relative_to(left).as_posix() for p in left.rglob("*") if p.is_file()
    )
    right_files = sorted(
        p.relative_to(right).as_posix() for p in right.rglob("*") if p.is_file()
    )
    if left_files != right_files:
        return False
    match, mismatch, errors = filecmp.cmpfiles(left, right, left_files, shallow=False)
    return not mismatch and not errors and len(match) == len(left_files)


def check_link(link: Path, target: Path, problems: list[str]) -> None:
    """`link` is a symlink to `target`, or a byte-identical copy of it."""
    rel = link.relative_to(ROOT).as_posix()
    target_rel = target.relative_to(ROOT).as_posix()
    if not target.is_dir():
        problems.append(
            f"{target_rel} does not exist; it is the canonical copy for {rel}"
        )
        return
    if link.is_symlink():
        if link.resolve() != target.resolve():
            problems.append(
                f"{rel} is a symlink to {link.resolve()}, expected {target.resolve()}"
            )
        return
    if link.is_dir():
        if trees_identical(link, target):
            print(f"  note: {rel} is a materialised copy of {target_rel}")
            print("        (Windows checkout): identical, but edit the canonical one.")
        else:
            problems.append(
                f"{rel} is a directory, not a symlink, and its contents differ from "
                f"{target_rel}. The two runtimes have drifted. Restore the symlink: "
                f"rm -rf {rel} && ln -s {'../' + target_rel} {rel}"
            )
        return
    problems.append(
        f"{rel} is neither a symlink nor a directory. A Windows checkout without "
        f"developer mode writes the link target as a text file; copy {target_rel} "
        f"over it instead."
    )


def main() -> int:
    """Run every check, report each problem with its fix, and return an exit code."""
    problems: list[str] = []
    allowlist = load_allowlist()

    print(f"agent config check in {ROOT}")

    # 1. symlinks
    check_link(ROOT / ".claude" / "skills", ROOT / ".codex" / "skills", problems)
    check_link(ROOT / ".codex" / "agents", ROOT / ".claude" / "agents", problems)

    # 2. the CLAUDE.md import
    claude_md = ROOT / "CLAUDE.md"
    if not claude_md.is_file():
        problems.append("CLAUDE.md is missing")
    else:
        if claude_md.is_symlink():
            problems.append(
                "CLAUDE.md is a symlink. It must be a real file: a Windows runner "
                "checks a symlink out as a text file and the @AGENTS.md import is lost."
            )
        first = claude_md.read_text(encoding="utf-8").splitlines()[:1]
        if first != ["@AGENTS.md"]:
            problems.append(
                f"CLAUDE.md line 1 is {first!r}, expected '@AGENTS.md' -- without that "
                "import Claude Code never reads the canonical instructions."
            )

    # 3 and 4. references inside the instruction files
    files = scanned_files()
    checked_paths = 0
    checked_recipes = 0
    recipes: set[str] = set()
    summary = subprocess.run(
        ("just", "--summary"), capture_output=True, text=True, cwd=ROOT, check=False
    )
    have_just = summary.returncode == 0
    if have_just:
        recipes = set(summary.stdout.split())
    else:
        print("  note: `just --summary` unavailable; skipping the recipe-name check")

    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        text = body_of(path.read_text(encoding="utf-8", errors="replace"))
        for ref in sorted(path_refs(text)):
            checked_paths += 1
            if (ROOT / ref).exists() or allowed(ref, allowlist):
                continue
            problems.append(f"{rel}: path does not exist: {ref}")
        if have_just:
            for name in sorted(just_recipes(text)):
                checked_recipes += 1
                if name not in recipes:
                    problems.append(f"{rel}: no such just recipe: `just {name}`")

    print(
        f"  {len(files)} file(s), {checked_paths} path reference(s), "
        f"{checked_recipes} recipe reference(s), {len(allowlist)} allowlist entr(ies)"
    )

    if problems:
        print(f"\n{len(problems)} problem(s):", file=sys.stderr)
        for problem in problems:
            print(f"  {problem}", file=sys.stderr)
        print(
            "\nA path that does not exist yet belongs in "
            ".claude/path-allowlist.txt with a comment saying when it appears; a "
            f"line that is illustrative can carry `{IGNORE_MARKER}`.",
            file=sys.stderr,
        )
        return 1

    print("agent config ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
