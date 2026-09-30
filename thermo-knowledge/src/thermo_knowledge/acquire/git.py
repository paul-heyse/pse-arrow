# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `git`: a checkout of exactly the declared commit, without history.

The commit is fetched by SHA (depth 1) into a fresh repository and checked out; a declared tag
must peel to that commit; declared submodules are fetched at their recorded commits. The stored
tree holds the checked-out files only. The `git` command line does the work.
"""

from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

from thermo_knowledge.acquire.errors import AcquireError
from thermo_knowledge.acquire.manifest import GitSpec
from thermo_knowledge.acquire.outcome import Outcome
from thermo_knowledge.acquire.store import PIN_LENGTH, TREE_DIR_NAME

LICENCE_NAMES = ("license", "licence", "copying", "copyright", "notice", "unlicense", "authors")
"""Root-level files kept by a sparse checkout, in any case and with any suffix."""

# Bytes as committed: no end-of-line conversion, filters (LFS) or re-encoding, whatever the
# repository's own .gitattributes says. info/attributes outranks .gitattributes.
_ATTRIBUTES = "* -text -eol -filter -ident -working-tree-encoding\n"

_GIT_ENVIRONMENT = {
    "GIT_TERMINAL_PROMPT": "0",
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "LC_ALL": "C",
}


def git_environment() -> dict[str, str]:
    return {**os.environ, **_GIT_ENVIRONMENT}


def run_git(
    arguments: list[str], *, cwd: Path | None = None, source_id: str, input_text: str | None = None
) -> str:
    """Run `git`; a failure raises `AcquireError` naming the source, command and stderr."""
    command = ["git", *arguments]
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            env=git_environment(),
            input=input_text,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
    except FileNotFoundError as error:
        raise AcquireError(f"{source_id}: the git command is not installed") from error
    if result.returncode != 0:
        detail = result.stderr.strip() or result.stdout.strip() or "no output"
        raise AcquireError(
            f"{source_id}: `{' '.join(command)}` failed with exit status {result.returncode}: "
            f"{detail}"
        )
    return result.stdout


def git_version(source_id: str) -> str:
    return run_git(["--version"], source_id=source_id).strip().removeprefix("git version ")


def sparse_patterns(paths: list[str]) -> list[str]:
    """Non-cone sparse-checkout patterns: the declared paths plus root-level licence files."""
    patterns = [path if path.startswith("/") else f"/{path}" for path in paths]
    for name in LICENCE_NAMES:
        folded = "".join(f"[{c.upper()}{c}]" for c in name)
        patterns.append(f"/{folded}*")
    return patterns


def _resolve_tag(source_id: str, spec: GitSpec, tool_version: str) -> None:
    assert spec.tag is not None
    listing = run_git(
        ["ls-remote", spec.url, f"refs/tags/{spec.tag}", f"refs/tags/{spec.tag}^{{}}"],
        source_id=source_id,
    )
    refs: dict[str, str] = {}
    for line in listing.splitlines():
        sha, _, name = line.partition("\t")
        refs[name.strip()] = sha.strip()
    direct = refs.get(f"refs/tags/{spec.tag}")
    peeled = refs.get(f"refs/tags/{spec.tag}^{{}}", direct)
    if peeled is None:
        raise AcquireError(
            f"{source_id}: tag {spec.tag!r} does not exist at {spec.url}; "
            f"the declared commit is {spec.commit} (git {tool_version})"
        )
    if peeled != spec.commit:
        raise AcquireError(
            f"{source_id}: tag {spec.tag!r} at {spec.url} names commit {peeled}, "
            f"not the declared commit {spec.commit}; the tag has moved or the declaration is wrong"
        )


def fetch_checkout(
    source_id: str, url: str, commit: str, destination: Path, sparse: list[str] | None = None
) -> None:
    """Fetch `commit` by SHA at depth 1 into a fresh repository at `destination`, check it out
    and remove `.git`."""
    run_git(["init", "--quiet", str(destination)], source_id=source_id)
    (destination / ".git" / "info").mkdir(exist_ok=True)
    (destination / ".git" / "info" / "attributes").write_text(_ATTRIBUTES, encoding="utf-8")
    run_git(["remote", "add", "origin", url], cwd=destination, source_id=source_id)
    fetch = ["fetch", "--quiet", "--depth", "1", "--no-tags"]
    if sparse:
        # A server that does not support object filters ignores the option.
        fetch.append("--filter=blob:none")
    run_git([*fetch, "origin", commit], cwd=destination, source_id=source_id)
    if sparse:
        run_git(
            ["sparse-checkout", "set", "--no-cone", "--stdin"],
            cwd=destination,
            source_id=source_id,
            input_text="\n".join(sparse_patterns(sparse)) + "\n",
        )
    run_git(["checkout", "--quiet", "--detach", "FETCH_HEAD"], cwd=destination, source_id=source_id)
    head = run_git(["rev-parse", "HEAD"], cwd=destination, source_id=source_id).strip()
    if head != commit:
        raise AcquireError(
            f"{source_id}: fetched {head} from {url} but the declared commit is {commit}"
        )


def resolve_submodule_url(base: str, relative: str) -> str:
    """A submodule URL; `./` and `../` forms resolve against the superproject's URL, read as a
    directory the way git does (`../x` replaces the last component)."""
    if not relative.startswith(("./", "../")):
        return relative
    segments = base.rstrip("/").split("/")
    for part in relative.split("/"):
        if part == "..":
            if len(segments) > 1:
                segments.pop()
        elif part not in {".", ""}:
            segments.append(part)
    return "/".join(segments)


def _submodule(source_id: str, checkout: Path, path: str) -> tuple[str, str]:
    """The recorded commit and the URL of a submodule of the checked-out commit."""
    tree = run_git(["ls-tree", "HEAD", "--", path], cwd=checkout, source_id=source_id).strip()
    mode, _, rest = tree.partition(" ")
    if mode != "160000":
        raise AcquireError(
            f"{source_id}: declared submodule {path!r} is not a submodule of the commit"
        )
    commit = rest.split()[1]
    keys = run_git(
        ["config", "--blob", "HEAD:.gitmodules", "--get-regexp", r"^submodule\..+\.path$"],
        cwd=checkout,
        source_id=source_id,
    )
    name: str | None = None
    for line in keys.splitlines():
        key, _, value = line.partition(" ")
        if value.strip() == path:
            name = key.removeprefix("submodule.").removesuffix(".path")
    if name is None:
        raise AcquireError(f"{source_id}: submodule {path!r} has no entry in .gitmodules")
    url = run_git(
        ["config", "--blob", "HEAD:.gitmodules", "--get", f"submodule.{name}.url"],
        cwd=checkout,
        source_id=source_id,
    ).strip()
    return commit, url


def acquire(source_id: str, spec: GitSpec, work: Path) -> Outcome:
    """Build `<work>/tree/` from the declared commit."""
    version = git_version(source_id)
    if spec.tag is not None:
        _resolve_tag(source_id, spec, version)

    checkout = work / "checkout"
    fetch_checkout(source_id, spec.url, spec.commit, checkout, spec.sparse)
    details: dict[str, str] = {}
    if spec.tag is not None:
        details["tag"] = spec.tag
    if spec.sparse:
        details["sparse"] = ", ".join(spec.sparse)

    for path in spec.submodules:
        commit, relative_url = _submodule(source_id, checkout, path)
        url = resolve_submodule_url(spec.url, relative_url)
        scratch = work / "submodule"
        fetch_checkout(source_id, url, commit, scratch)
        shutil.rmtree(scratch / ".git")
        target = checkout / path
        target.mkdir(parents=True, exist_ok=True)
        shutil.copytree(scratch, target, symlinks=True, dirs_exist_ok=True)
        shutil.rmtree(scratch)
        details[f"submodule {path}"] = f"{commit} {url}"

    shutil.rmtree(checkout / ".git")
    checkout.rename(work / TREE_DIR_NAME)
    return Outcome(
        pin=spec.commit[:PIN_LENGTH],
        resolved=spec.commit,
        urls=[spec.url],
        tool_versions={"git": version},
        details=details,
    )
