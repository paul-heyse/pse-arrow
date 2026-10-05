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
    The same information with fix commands spelled out. ``just ready`` runs it
    (``just doctor``) after an environment change.
``--format=json``
    Machine-readable, for agents and CI::

        just doctor --format=json | jq '.checks[] | select(.ok==false)'

Exit status is 1 only when something *blocks work*. A missing solver container or
docker is reported, not fatal: most of the workspace builds without them.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tomllib
import urllib.parse
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VENV = ROOT / Path(os.environ.get("UV_PROJECT_ENVIRONMENT", ".venv"))


def cargo_tool_names() -> list[str]:
    """Read the bootstrap authority rather than duplicating its tool list."""
    text = (ROOT / "scripts/bootstrap.sh").read_text()
    block = text.split("CARGO_TOOLS=(", 1)[1].split(")", 1)[0]
    return re.findall(r'"([a-z0-9-]+)"', block)


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


def run(
    *args: str,
    cwd: Path | None = None,
    timeout: int = 30,
    env: dict[str, str] | None = None,
) -> tuple[int, str]:
    """Run a command, returning (returncode, stripped stdout+stderr)."""
    try:
        proc = subprocess.run(
            args,
            cwd=cwd or ROOT,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
            env=env,
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


def quality_tool_names() -> list[str]:
    """Tool names from [dependency-groups].quality; this script carries no copies.

    The group declares floors, not exact pins, so only the distribution name is read;
    whatever uv.lock resolved is the version that runs.
    """
    names: list[str] = []
    for spec in pyproject().get("dependency-groups", {}).get("quality", []):
        if isinstance(spec, str):
            name = re.split(r"[<>=!~;\[ ]", spec, maxsplit=1)[0].strip()
            if name:
                names.append(name)
    return names


def native_library_path() -> Path | None:
    """The solver library directory native recipes put on the library path, if prepared.

    ``scripts/native-execution-env.sh`` exports ``$IPOPT_DIR/lib`` because the extracted
    solver libraries carry the image's own runpath (``/opt/pse-solvers/lib``), which
    hides the extension's rpath from their dependencies. The prefix is ``IPOPT_DIR`` when
    set, else the one ``native_cache.py`` has already extracted; this never runs docker.
    """
    if os.environ.get("IPOPT_DIR"):
        return Path(os.environ["IPOPT_DIR"]) / "lib"
    if platform.system() != "Linux":  # native solver builds are Linux-only
        return None
    spec = importlib.util.spec_from_file_location(
        "_doctor_native_cache", ROOT / "scripts/native_cache.py"
    )
    if spec is None or spec.loader is None:
        return None
    try:
        cache = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cache)
        prefix = cache.prepared_solver(cache.cache_root(dict(os.environ)))
    except (ImportError, OSError, ValueError):
        return None
    return None if prefix is None else prefix / "lib"


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
    """Any uv that can read uv.lock is fine; no release is required."""
    if shutil.which("uv") is None:
        return Check("uv", False, "uv not installed", "https://docs.astral.sh/uv/")
    code, out = run("uv", "--version")
    if code != 0:
        return Check(
            "uv", False, out.splitlines()[0] if out else "uv failed", "uv self update"
        )
    actual = out.split()[1] if len(out.split()) > 1 else "?"
    return Check("uv", True, f"uv {actual}")


def check_env_synced() -> Check:
    """`uv sync --check` is the authority on whether .venv matches uv.lock.

    Every dependency is checked exactly. A difference in the project's own package is
    left to ``check_extension``: a native build is installed by maturin
    (``just py-sync-native``), and uv would replace it with a build without the
    solvers. uv's JSON plan (a preview schema) names the packages; if it cannot be read,
    the exit status decides, which can only over-report.
    """
    if not venv_bin("python").exists():
        return Check("env", False, "no .venv", "just bootstrap-venv")
    if not (ROOT / "uv.lock").exists():
        return Check("env", False, "no uv.lock", "uv lock")
    command = (
        "uv",
        "sync",
        "--check",
        "--locked",
        "--offline",
        "--output-format",
        "json",
    )
    try:
        proc = subprocess.run(
            command, cwd=ROOT, capture_output=True, text=True, timeout=30, check=False
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return Check("env", False, str(exc)[:80], "just py-sync")
    if proc.returncode == 0:
        return Check("env", True, ".venv matches uv.lock")
    project = pyproject()["project"]["name"]
    try:
        changed = {
            change["name"] for change in json.loads(proc.stdout)["sync"]["changes"]
        }
    except (ValueError, KeyError, TypeError):
        changed = set()
    if changed == {project}:
        return Check(
            "env", True, f".venv matches uv.lock ({project} is the extension check's)"
        )
    out = (proc.stderr or proc.stdout).strip()
    last = out.splitlines()[-1] if out else "out of date"
    return Check("env", False, last[:80], "just py-sync")


def check_quality_tools() -> Check:
    """Every quality tool is present in .venv; the version that resolved is reported."""
    tools = quality_tool_names()
    if not tools:
        return Check("quality", False, "no [dependency-groups] quality entries", "")
    # CLI wrappers can initialize runtimes even for --version.
    # Read installed distribution metadata through the selected interpreter instead.
    code, out = run(
        str(venv_bin("python")),
        "-c",
        "import importlib.metadata as m, json, sys; "
        "installed = {d.metadata['Name'].lower().replace('_', '-'): d.version "
        "for d in m.distributions()}; "
        "print(json.dumps({n: installed.get(n, '?') for n in sys.argv[1:]}))",
        *tools,
    )
    installed = json.loads(out) if code == 0 else {}
    problems, found = [], []
    for tool in tools:
        exe = venv_bin(tool if tool != "import-linter" else "lint-imports")
        if not exe.exists():
            problems.append(f"{tool} not in .venv")
            continue
        found.append(f"{tool} {installed.get(tool, '?')}")
    if problems:
        return Check(
            "quality",
            False,
            "; ".join(problems),
            "just bootstrap-quality",
            extra={"tools": tools},
        )
    return Check("quality", True, " / ".join(found), extra={"tools": tools})


def check_rust() -> Check:
    wanted = ""
    components: list[str] = []
    toolchain = ROOT / "rust-toolchain.toml"
    if toolchain.exists():
        pin = tomllib.loads(toolchain.read_text(encoding="utf-8")).get("toolchain", {})
        wanted = pin.get("channel", "")
        components = pin.get("components", [])
    if shutil.which("rustup") is None:
        return Check("rust", False, "rustup not installed", "https://rustup.rs")
    install = f"rustup toolchain install {wanted} --profile minimal" + "".join(
        f" -c {name}" for name in components
    )
    # `rustup run` fails for an absent toolchain instead of auto-installing it. The pin is
    # a dated nightly whose rustc reports its own release and commit date, so a successful
    # run of that channel is the identity check.
    code, out = run("rustup", "run", wanted, "rustc", "--version")
    if code != 0:
        return Check(
            "rust", False, out.splitlines()[0] if out else "rustc failed", install
        )
    actual = " ".join(out.split()[1:]) or "?"
    code, listed = run(
        "rustup", "component", "list", "--toolchain", wanted, "--installed"
    )
    # Installed components are listed with the host triple appended (`clippy-<host>`),
    # except target-independent ones (`rust-src`).
    installed = listed.split() if code == 0 else []
    missing = [
        name
        for name in components
        if not any(item == name or item.startswith(f"{name}-") for item in installed)
    ]
    ok = not missing
    return Check(
        "rust",
        ok,
        f"rustc {actual} ({wanted})"
        + ("" if ok else f"; missing components: {', '.join(missing)}"),
        "" if ok else install,
    )


def check_cargo_tools() -> Check:
    toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"][
        "channel"
    ]
    code, out = run(
        "rustup", "run", toolchain, "cargo", "install", "--list", timeout=60
    )
    if code != 0:
        return Check(
            "cargo-tools",
            False,
            "cargo unavailable",
            "https://rustup.rs",
            blocking=False,
        )
    installed = dict(re.findall(r"^([a-z0-9-]+) v([0-9.]+)", out, re.MULTILINE))
    tools = cargo_tool_names()
    missing = [name for name in tools if name not in installed]
    return Check(
        "cargo-tools",
        not missing,
        f"missing: {', '.join(missing)}" if missing else f"{len(tools)} tools present",
        "just bootstrap-rust-tools" if missing else "",
        blocking=True,
        extra={
            "installed": {name: installed[name] for name in tools if name in installed}
        },
    )


def check_docs_tools() -> Check:
    """The publisher's declaration owns documentation versions, including CI installs."""
    pins = tomllib.loads((ROOT / "docs/site.toml").read_text())["tools"]
    problems = []
    for name, version in pins.items():
        code, output = run(name, "--version")
        if code or not re.search(rf"(?:^|\s)v?{re.escape(version)}(?:\s|$)", output):
            problems.append(f"{name}: expected {version}")
    return Check(
        "docs-tools",
        not problems,
        "; ".join(problems) or "declared versions present",
        "just bootstrap-docs" if problems else "",
        blocking=True,
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


#: Imports the extension and reports its provenance, plus whether the dynamic linker
#: loaded the native solvers (Linux ``/proc/self/maps``; native builds are Linux-only).
EXTENSION_PROBE = (
    "import pathlib, pse; b = pse.build_info(); m = pathlib.Path('/proc/self/maps'); "
    "n = m.exists() and 'libipopt' in m.read_text(); "
    "print(b.version, b.cargo_lock_sha256, b.uv_lock_sha256, int(n))"
)


def check_extension() -> Check:
    """The built extension must import and match the checkout's lockfiles (pse-buildinfo).

    It is imported the way native recipes import it, with the prepared solver libraries
    on the library path, so a native build is recognized rather than reported missing.
    """
    python = venv_bin("python")
    if not python.exists():
        return Check(
            "extension", False, "no .venv", "just bootstrap-venv", blocking=False
        )
    env = dict(os.environ)
    library = native_library_path()
    if library is not None:
        env["LD_LIBRARY_PATH"] = os.pathsep.join(
            part for part in (str(library), env.get("LD_LIBRARY_PATH", "")) if part
        )
    code, out = run(str(python), "-c", EXTENSION_PROBE, timeout=120, env=env)
    if code != 0:
        last = out.strip().splitlines()[-1] if out else "import failed"
        return Check(
            "extension",
            False,
            last[:80],
            "just py-sync, or just py-sync-native for the native solvers",
            blocking=False,
        )
    version, cargo_sha, uv_sha, linked = [*out.split(), "", "", "", ""][:4]
    native = linked == "1"
    sync = "just py-sync-native" if native else "just py-sync"
    kind = "native solvers, " if native else ""
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
            sync,
            blocking=False,
        )
    return Check("extension", True, f"pse {version} ({kind}lockfiles match)")


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
            "pinned solver image present"
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


OPERATIONS = ROOT / "crates/pse-operations"


def operational_store_url() -> str:
    """PSE_DATABASE_URL, else the default declared once in pse-operations."""
    url = os.environ.get("PSE_DATABASE_URL", "")
    if url:
        return url
    source = (OPERATIONS / "src/store.rs").read_text(encoding="utf-8")
    found = re.search(
        r'^pub const DEFAULT_DATABASE_URL: &str = "([^"]*)";$', source, re.MULTILINE
    )
    return found.group(1) if found else ""


def is_local_database(url: str) -> bool:
    """A Unix socket or loopback host: probing it never touches the network."""
    parts = urllib.parse.urlsplit(url)
    hosts = urllib.parse.parse_qs(parts.query).get("host", [])
    host = hosts[0] if hosts else (parts.hostname or "")
    return not host or host.startswith("/") or host in {"localhost", "127.0.0.1", "::1"}


def expected_schema_fingerprint() -> str:
    """The schema fingerprint this checkout generates (ADR-0114 Outcome 23)."""
    source = (OPERATIONS / "src/generated/fingerprint.rs").read_text(encoding="utf-8")
    found = re.search(
        r'^pub const SCHEMA_FINGERPRINT_HEX: &str = "([0-9a-f]{64})";$',
        source,
        re.MULTILINE,
    )
    return found.group(1) if found else ""


def check_operational_store() -> Check:
    """Never blocking: ephemeral work runs without the operational store (ADR-0114)."""
    setup = (
        "just db-bootstrap (once), then just db-status; docs/dev/operational-store.md"
    )
    url = operational_store_url()
    psql = shutil.which("psql")
    if psql is None or not url:
        detail = "psql not found" if psql is None else "no store URL"
        return Check("opstore", False, detail, setup, blocking=False)
    if not is_local_database(url):
        return Check(
            "opstore",
            False,
            "remote store not probed here",
            "just db-status",
            blocking=False,
        )

    def query(sql: str) -> tuple[int, str]:
        try:
            proc = subprocess.run(
                [psql, "-X", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-d", url, "-c", sql],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
                env={**os.environ, "PGCONNECT_TIMEOUT": "2"},
            )
        except (OSError, subprocess.SubprocessError) as exc:
            return 127, str(exc)
        return proc.returncode, (
            proc.stdout if proc.returncode == 0 else proc.stderr
        ).strip()

    code, out = query(
        "SELECT current_setting('server_version_num'), current_setting('server_version'),"
        " (SELECT coalesce(obj_description(oid, 'pg_namespace'), '<none>')"
        " FROM pg_namespace WHERE nspname = 'pse_ops')"
    )
    if code != 0:
        first = (
            out.splitlines()[0].removeprefix("psql: error: ") if out else "no answer"
        )
        return Check(
            "opstore", False, f"unreachable: {first}"[:80], setup, blocking=False
        )
    version_num, version, recorded = out.split("|", 2)
    version = version.split(" ", 1)[0]
    if int(version_num) < 180000:
        return Check(
            "opstore",
            False,
            f"PostgreSQL {version}; 18 or newer is required",
            "upgrade the server; docs/dev/operational-store.md",
            blocking=False,
        )
    expected = expected_schema_fingerprint()
    if not recorded:
        return Check(
            "opstore",
            True,
            f"PostgreSQL {version}; no pse_ops schema yet (create explicitly with just db-create)",
            blocking=False,
        )
    if recorded != f"pse.ops.schema.v1 {expected}":
        return Check(
            "opstore",
            False,
            f"PostgreSQL {version}; pse_ops schema is another build's",
            "drain workers, close store generations, then just db-migrate for a declared supported source; preserve unsupported sources",
            blocking=False,
        )
    return Check(
        "opstore", True, f"PostgreSQL {version}; schema current", blocking=False
    )


CHECKS = (
    check_interpreter,
    check_uv,
    check_env_synced,
    check_quality_tools,
    check_rust,
    check_cargo_tools,
    check_docs_tools,
    check_repo_linters,
    check_extension,
    check_solvers,
    check_operational_store,
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
