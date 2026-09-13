#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Report whether this working copy is ready to do work, and how to fix it.

Deliberately dependency-free and standard-library only: it has to run *before* the
virtual environment exists, which is exactly when it is most needed.

Three output formats:

``--format=direnv``
    A compact status block. ``.envrc`` calls this on every directory entry, so it must
    be fast and must never touch the network.
``--format=text``
    The same information with fix commands spelled out. Used by the Claude Code
    SessionStart hook.
``--format=json``
    Machine-readable, for agents and CI::

        just doctor --format=json | jq '.checks[] | select(.ok==false)'

Exit status is 1 only when something *blocks work*. A missing solver container or
docker is reported, not fatal: most of the workspace builds without them.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VENV = ROOT / Path(os.environ.get("UV_PROJECT_ENVIRONMENT", ".venv"))


def cargo_tool_pins() -> dict[str, str]:
    """Read the bootstrap authority rather than duplicating its tool pins."""
    text = (ROOT / "scripts/bootstrap.sh").read_text()
    block = text.split("CARGO_TOOLS=(", 1)[1].split(")", 1)[0]
    return dict(re.findall(r'"([a-z0-9-]+)@([0-9.]+)"', block))


REPO_LINTERS = ("actionlint", "ast-grep", "shellcheck")


@dataclass
class Check:
    """One probe, its verdict, and the command that fixes it."""

    name: str
    ok: bool
    detail: str
    fix: str = ""
    blocking: bool = True
    extra: dict[str, object] = field(default_factory=dict)


def run(*args: str, cwd: Path | None = None, timeout: int = 30) -> tuple[int, str]:
    """Run a command, returning (returncode, stripped stdout+stderr)."""
    try:
        proc = subprocess.run(
            args,
            cwd=cwd or ROOT,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return 127, str(exc)
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def venv_bin(name: str) -> Path:
    """Path to a tool inside the project environment, whether or not it exists."""
    if platform.system() == "Windows":
        return VENV / "Scripts" / f"{name}.exe"
    return VENV / "bin" / name


def pyproject() -> dict:
    """The parsed pyproject.toml (single source of truth for every Python pin)."""
    return tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))


def pinned_quality_versions() -> dict[str, str]:
    """Exact pins from [dependency-groups].quality; this script carries no copies."""
    pins: dict[str, str] = {}
    for spec in pyproject().get("dependency-groups", {}).get("quality", []):
        if isinstance(spec, str) and "==" in spec:
            name, _, version = spec.partition("==")
            pins[name.strip()] = version.strip()
    return pins


# --------------------------------------------------------------------- checks


def check_interpreter() -> Check:
    wanted = (ROOT / ".python-version").read_text(encoding="utf-8").strip()
    python = venv_bin("python")
    if not python.exists():
        return Check(
            "python", False, f"{VENV} missing (want {wanted})", "just bootstrap"
        )
    code, out = run(str(python), "--version")
    if code != 0:
        return Check("python", False, f"{VENV} broken: {out}", "just bootstrap")
    actual = out.split()[-1]
    ok = actual == wanted
    return Check(
        "python",
        ok,
        f"{VENV} {actual}" + ("" if ok else f" (.python-version wants {wanted})"),
        "" if ok else "just bootstrap",
        extra={"wanted": wanted, "actual": actual},
    )


def check_uv() -> Check:
    wanted = pyproject().get("tool", {}).get("uv", {}).get("required-version", "")
    if shutil.which("uv") is None:
        return Check("uv", False, "uv not installed", "https://docs.astral.sh/uv/")
    code, out = run("uv", "--version")
    actual = out.split()[1] if code == 0 and len(out.split()) > 1 else "?"
    ok = not wanted or wanted.lstrip("=") == actual
    return Check(
        "uv",
        ok,
        f"uv {actual}" + ("" if ok else f" (pyproject wants {wanted})"),
        "" if ok else f"uv self update {wanted.lstrip('=')}",
    )


def check_env_synced() -> Check:
    """`uv sync --check` is the authority on whether .venv matches uv.lock."""
    if not venv_bin("python").exists():
        return Check("env", False, "no .venv", "just bootstrap-venv")
    if not (ROOT / "uv.lock").exists():
        return Check("env", False, "no uv.lock", "uv lock")
    code, out = run(
        "uv", "sync", "--check", "--locked", "--offline", "--extra", "pyomo", timeout=30
    )
    if code != 0:
        last = out.strip().splitlines()[-1] if out else "out of date"
        return Check("env", False, last[:80], "just py-sync")
    return Check("env", True, ".venv matches uv.lock")


def check_quality_tools() -> Check:
    pins = pinned_quality_versions()
    if not pins:
        return Check("quality", False, "no [dependency-groups] quality pins", "")
    problems, found = [], []
    for tool, wanted in pins.items():
        exe = venv_bin(tool if tool != "import-linter" else "lint-imports")
        if not exe.exists():
            problems.append(f"{tool} not in .venv")
            continue
        code, out = run(str(exe), "--version")
        actual = (
            next((t for t in out.split() if t[:1].isdigit()), "?") if code == 0 else "?"
        )
        found.append(f"{tool} {actual}")
        if actual != wanted:
            problems.append(f"{tool} {actual} != pinned {wanted}")
    if problems:
        return Check(
            "quality",
            False,
            "; ".join(problems),
            "just bootstrap-quality",
            extra={"pins": pins},
        )
    return Check("quality", True, " / ".join(found), extra={"pins": pins})


def check_rust() -> Check:
    wanted = ""
    toolchain = ROOT / "rust-toolchain.toml"
    if toolchain.exists():
        wanted = (
            tomllib.loads(toolchain.read_text(encoding="utf-8"))
            .get("toolchain", {})
            .get("channel", "")
        )
    if shutil.which("rustup") is None:
        return Check("rust", False, "rustup not installed", "https://rustup.rs")
    code, out = run("rustc", "--version")
    if code != 0:
        return Check(
            "rust",
            False,
            out.splitlines()[0] if out else "rustc failed",
            f"rustup toolchain install {wanted}",
        )
    actual = out.split()[1] if len(out.split()) > 1 else "?"
    ok = actual == wanted or not wanted
    return Check(
        "rust",
        ok,
        f"rustc {actual}" + ("" if ok else f" (rust-toolchain.toml wants {wanted})"),
        "" if ok else f"rustup toolchain install {wanted}",
    )


def check_cargo_tools() -> Check:
    code, out = run("cargo", "install", "--list", timeout=60)
    if code != 0:
        return Check(
            "cargo-tools",
            False,
            "cargo unavailable",
            "https://rustup.rs",
            blocking=False,
        )
    installed = dict(re.findall(r"^([a-z0-9-]+) v([0-9.]+)", out, re.MULTILINE))
    pins = cargo_tool_pins()
    problems = [
        f"{name}: {installed.get(name, 'missing')} (want {wanted})"
        for name, wanted in pins.items()
        if installed.get(name) != wanted
    ]
    return Check(
        "cargo-tools",
        not problems,
        "; ".join(problems) if problems else f"{len(pins)} pinned tools match",
        "just bootstrap-rust-tools" if problems else "",
        blocking=True,
    )


def check_git_hooks() -> Check:
    """Check the effective Git hook paths, including core.hooksPath."""
    missing = []
    for name in ("pre-commit", "pre-push"):
        code, out = run("git", "rev-parse", "--git-path", f"hooks/{name}")
        path = ROOT / out
        if (
            code
            or not path.is_file()
            or "pre_commit" not in path.read_text(errors="replace")
        ):
            missing.append(name)
    return Check(
        "git-hooks",
        not missing,
        ", ".join(missing) + " missing"
        if missing
        else "pre-commit and pre-push installed",
        "just bootstrap" if missing else "",
    )


def check_repo_linters() -> Check:
    missing = [t for t in REPO_LINTERS if shutil.which(t) is None]
    if missing:
        return Check(
            "linters",
            False,
            f"missing: {', '.join(missing)}",
            "just bootstrap-linters",
            blocking=False,
        )
    return Check("linters", True, ", ".join(REPO_LINTERS))


def check_extension() -> Check:
    """The built extension must match the checkout's lockfiles (pse-buildinfo)."""
    python = venv_bin("python")
    if not python.exists():
        return Check(
            "extension", False, "no .venv", "just bootstrap-venv", blocking=False
        )
    code, out = run(
        str(python),
        "-c",
        "import pse; b = pse.build_info(); print(b.version, b.cargo_lock_sha256, b.uv_lock_sha256)",
        timeout=120,
    )
    if code != 0:
        last = out.strip().splitlines()[-1] if out else "import failed"
        return Check("extension", False, last[:80], "just py-sync", blocking=False)
    version, cargo_sha, uv_sha = [*out.split(), "", "", ""][:3]
    stale = []
    for name, seen in (("Cargo.lock", cargo_sha), ("uv.lock", uv_sha)):
        path = ROOT / name
        if (
            path.exists()
            and seen
            and hashlib.sha256(path.read_bytes()).hexdigest() != seen
        ):
            stale.append(name)
    if stale:
        return Check(
            "extension",
            False,
            f"pse {version} built from stale {', '.join(stale)}",
            "just py-sync",
            blocking=False,
        )
    return Check("extension", True, f"pse {version} (lockfiles match)")


def check_solvers() -> Check:
    """Never blocking: most of the workspace builds without Ipopt."""
    ipopt = shutil.which("ipopt")
    if ipopt is None:
        docker = shutil.which("docker") or shutil.which("podman")
        manifest = json.loads((ROOT / ".github/setup/solver-images.json").read_text())
        image = manifest["ci"]
        code, out = (
            run(docker, "image", "inspect", image)
            if docker
            else (1, "no container runtime")
        )
        ready = code == 0
        return Check(
            "solvers",
            ready,
            "pinned container present; use just parity-container"
            if ready
            else "pinned solver container missing",
            "" if ready else "just bootstrap-solvers",
            blocking=False,
        )
    code, out = run(ipopt, "-v")
    version = (
        next((t for t in out.split() if t[:1].isdigit()), "?") if code == 0 else "?"
    )
    return Check(
        "solvers",
        code == 0 and version.startswith("3.14."),
        f"ipopt {version} at {ipopt}",
        "use the pinned solver container",
        blocking=False,
    )


def check_external() -> Check:
    pins = tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]
    versions = {p["name"]: p["version"] for p in pins}
    parity = pyproject()["dependency-groups"]["parity"]
    idaes = next(
        p.split("==", 1)[1].split(";", 1)[0]
        for p in parity
        if isinstance(p, str) and p.startswith("idaes-pse==")
    )
    wanted = {
        "arrow-rs": versions["arrow"],
        "datafusion": versions["datafusion"],
        "idaes-pse": idaes,
    }
    problems = []
    for name, tag in wanted.items():
        directory = ROOT / "external" / name
        # Reading copies must match the pins, not merely exist.
        code, out = run("git", "describe", "--tags", "--exact-match", cwd=directory)
        if code or out != tag:
            problems.append(f"{name}: missing or not at {tag}")
    return Check(
        "external",
        not problems,
        "; ".join(problems) if problems else "all reading copies match pins",
        "just fetch-external" if problems else "",
        blocking=False,
    )


CHECKS = (
    check_interpreter,
    check_uv,
    check_env_synced,
    check_quality_tools,
    check_rust,
    check_cargo_tools,
    check_repo_linters,
    check_extension,
    check_git_hooks,
    check_solvers,
    check_external,
)


# --------------------------------------------------------------------- output


def emit_direnv(checks: list[Check]) -> None:
    print("direnv: pse-arrow")
    for c in checks:
        status = "ok" if c.ok else ("MISSING" if c.blocking else "--")
        print(f"  {c.name:<11} {c.detail[:46]:<48} {status}")
        if not c.ok and c.fix:
            print(f"  {'':<11} -> {c.fix}")


def emit_text(checks: list[Check]) -> None:
    for c in checks:
        mark = "PASS" if c.ok else ("FAIL" if c.blocking else "WARN")
        print(f"[{mark}] {c.name}: {c.detail}")
        if not c.ok and c.fix:
            print(f"       fix: {c.fix}")
    blocking = [c for c in checks if not c.ok and c.blocking]
    if blocking:
        print(f"\n{len(blocking)} blocking issue(s). Start with: just bootstrap")
    else:
        print("\nEnvironment ready.")


def emit_json(checks: list[Check]) -> None:
    print(
        json.dumps(
            {
                "repo": str(ROOT),
                "ready": all(c.ok for c in checks if c.blocking),
                "checks": [
                    {
                        "name": c.name,
                        "ok": c.ok,
                        "detail": c.detail,
                        "fix": c.fix,
                        "blocking": c.blocking,
                        **({"extra": c.extra} if c.extra else {}),
                    }
                    for c in checks
                ],
            },
            indent=2,
        )
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--format", choices=("text", "json", "direnv"), default="text")
    args = parser.parse_args(argv)
    checks = [fn() for fn in CHECKS]
    {"direnv": emit_direnv, "text": emit_text, "json": emit_json}[args.format](checks)
    return 1 if any(not c.ok and c.blocking for c in checks) else 0


if __name__ == "__main__":
    sys.exit(main())
