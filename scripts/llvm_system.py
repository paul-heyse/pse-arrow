# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Stage and select a relocatable LLVM installation; dry-run unless --apply is given."""

from __future__ import annotations

import argparse
import datetime
import fcntl
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

SELECTOR = Path("/opt/llvm-current")
ENVIRONMENT = Path("/etc/environment")
SERVICE_ENVIRONMENT = Path("/etc/environment.d/60-pse-llvm.conf")
TOOLS = ("clang", "clang++", "llvm-config")
LINKS = tuple(Path("/usr/local/bin") / name for name in TOOLS)
SYSTEM_PATH = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"


def capture(command: list[str], *, env: dict[str, str] | None = None) -> str:
    return subprocess.check_output(
        command, env=env, text=True, stderr=subprocess.PIPE, timeout=60
    ).strip()


def sha(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def variables(prefix: Path) -> dict[str, str]:
    return {
        "PSE_LLVM_PREFIX": str(prefix),
        "CLANG_PATH": str(prefix / "bin/clang"),
        "LIBCLANG_PATH": str(prefix / "lib"),
        "LLVM_CONFIG_PATH": str(prefix / "bin/llvm-config"),
    }


def environment_text(original: str) -> str:
    """Preserve other PAM settings and the existing PATH, without shell evaluation."""
    lines = original.splitlines()
    paths = [line for line in lines if line.startswith("PATH=")]
    if len(paths) > 1:
        raise ValueError("/etc/environment contains multiple PATH declarations")
    path = shlex.split(paths[0][5:])[0] if paths else SYSTEM_PATH
    parts = [part for part in path.split(":") if part and part != str(SELECTOR / "bin")]
    updates = {"PATH": ":".join([str(SELECTOR / "bin"), *parts]), **variables(SELECTOR)}
    for key, value in updates.items():
        indexes = [i for i, line in enumerate(lines) if line.startswith(key + "=")]
        if len(indexes) > 1:
            raise ValueError(f"/etc/environment contains duplicate {key}")
        line = f'{key}="{value}"'
        if indexes:
            lines[indexes[0]] = line
        else:
            lines.append(line)
    return "\n".join(lines) + "\n"


def clean_env(prefix: Path) -> dict[str, str]:
    # No inherited LD_LIBRARY_PATH, license values, shell startup or Python venv.
    return {"PATH": f"{prefix / 'bin'}:{SYSTEM_PATH}", "LANG": "C", **variables(prefix)}


def validate(prefix: Path, version: str) -> dict:
    prefix = prefix.resolve()
    env = clean_env(prefix)
    observed = capture([str(prefix / "bin/llvm-config"), "--version"], env=env)
    if observed != version:
        raise ValueError(f"LLVM version differs: expected {version}, found {observed}")
    for switch, expected in (("--prefix", prefix), ("--libdir", prefix / "lib")):
        actual = Path(
            capture([str(prefix / "bin/llvm-config"), switch], env=env)
        ).resolve()
        if actual != expected:
            raise ValueError(f"llvm-config {switch} still points at {actual}")
    resource = Path(
        capture([str(prefix / "bin/clang"), "-print-resource-dir"], env=env)
    ).resolve()
    resource.relative_to(prefix)
    if not (resource / "include/stddef.h").is_file():
        raise ValueError("relocated Clang is missing resource headers")
    libraries = list((prefix / "lib").glob("libLLVM.so.*"))
    if not libraries:
        raise ValueError("missing shared LLVM library")
    libclang = prefix / "lib/libclang.so"
    links = {}
    for path in (
        prefix / "bin/clang",
        prefix / "bin/llvm-config",
        libclang,
        libraries[0],
    ):
        linked = capture(["ldd", str(path)], env=env)
        if "not found" in linked:
            raise ValueError(f"unresolved shared libraries: {path}")
        for line in linked.splitlines():
            parts = line.split()
            if "=>" in parts and len(parts) >= 3 and parts[2].startswith("/"):
                dependency = Path(parts[2]).resolve()
                if dependency.is_relative_to(Path.home()) or str(dependency).startswith(
                    "/home/"
                ):
                    raise ValueError(
                        f"relocation still loads a home library: {dependency}"
                    )
        links[path.name] = linked
    # Exercise libclang loading without inheriting an interactive library search path.
    capture(
        [
            sys.executable,
            "-c",
            "import ctypes,sys; ctypes.CDLL(sys.argv[1])",
            str(libclang),
        ],
        env=env,
    )
    with tempfile.TemporaryDirectory(prefix="pse-llvm-link-") as directory:
        source = Path(directory) / "main.c"
        binary = Path(directory) / "main"
        source.write_text(
            "#include <stddef.h>\nint main(void) { return sizeof(size_t) == 0; }\n"
        )
        # This is a relocation smoke check, not a linker comparison or benchmark.
        capture(
            [
                str(prefix / "bin/clang"),
                "-fuse-ld=mold",
                str(source),
                "-o",
                str(binary),
            ],
            env=env,
        )
        capture([str(binary)], env=env)
        cpp = source.with_suffix(".cpp")
        cpp.write_text(
            "#include <vector>\nint main() { return std::vector<int>{1}.size() != 1; }\n"
        )
        capture([str(prefix / "bin/clang++"), str(cpp), "-o", str(binary)], env=env)
        capture([str(binary)], env=env)
    return {
        "version": observed,
        "prefix": str(prefix),
        "resource_dir": str(resource),
        "shared_libraries": links,
        "c_and_cpp_execution": "passed",
    }


def stage(source: Path, destination: Path, version: str) -> dict:
    """Copy, make dependencies self-contained, remove home RUNPATHs, then test."""
    if destination.exists() or destination.is_symlink():
        raise ValueError(f"staging destination already exists: {destination}")
    if not shutil.which("patchelf"):
        raise ValueError("patchelf is required; install it before applying")
    source = source.resolve(strict=True)
    env = clean_env(source)
    if capture([str(source / "bin/llvm-config"), "--version"], env=env) != version:
        raise ValueError("source LLVM does not match the requested version")
    llvm = next((source / "lib").glob("libLLVM.so.*"))
    linked = capture(["ldd", str(llvm)], env=env)
    z3 = [
        line.split()[2]
        for line in linked.splitlines()
        if line.strip().startswith("libz3.so") and "=>" in line
    ]
    if len(z3) != 1 or not Path(z3[0]).is_file():
        raise ValueError("expected one resolved Z3 dependency")
    shutil.copytree(source, destination, symlinks=True)
    z3_file = Path(z3[0]).resolve()
    shutil.copy2(z3_file, destination / "lib" / z3_file.name)
    soname = capture(["patchelf", "--print-soname", str(z3_file)])
    if not soname or Path(soname).name != soname:
        raise ValueError("invalid Z3 SONAME")
    link = destination / "lib" / soname
    if link.name != z3_file.name:
        link.unlink(missing_ok=True)
        link.symlink_to(z3_file.name)
    patched = set()
    for directory in (destination / "bin", destination / "lib"):
        for path in directory.rglob("*"):
            if not path.is_file():
                continue
            real = path.resolve()
            if not real.is_relative_to(destination.resolve()):
                raise ValueError(f"installation contains an external symlink: {path}")
            if real in patched:
                continue
            with real.open("rb") as stream:
                elf = stream.read(4) == b"\x7fELF"
            if elf and "DYNAMIC" in capture(["readelf", "-l", str(real)]):
                subprocess.run(
                    ["patchelf", "--set-rpath", "$ORIGIN/../lib:$ORIGIN", str(real)],
                    check=True,
                )
                patched.add(real)
    report = validate(destination, version)
    report.update(
        source=str(source),
        source_clang_sha256=sha(source / "bin/clang"),
        z3_sha256=sha(z3_file),
        patched_elf_files=len(patched),
    )
    (destination / ".pse-llvm-install.json").write_text(
        json.dumps(report, indent=2) + "\n"
    )
    return report


def fingerprint(path: Path) -> str:
    if path.is_symlink():
        return "link:" + str(path.readlink())
    if path.is_file():
        return "file:" + sha(path)
    if path.exists():
        raise ValueError(f"refusing to replace a non-file path: {path}")
    return "absent"


def atomic_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", dir=path.parent, prefix=".pse-llvm-", delete=False
    ) as stream:
        temporary = Path(stream.name)
        stream.write(text)
    temporary.chmod(0o644)
    temporary.replace(path)


def atomic_link(path: Path, target: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.pse-llvm-{os.getpid()}")
    temporary.symlink_to(target)
    try:
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)


def backup(directory: Path) -> dict:
    directory.mkdir(parents=True, exist_ok=False, mode=0o700)
    records = []
    for index, path in enumerate((SELECTOR, ENVIRONMENT, SERVICE_ENVIRONMENT, *LINKS)):
        state = fingerprint(path)
        record: dict[str, object] = {"path": str(path), "before": state}
        if state.startswith("file:"):
            saved = directory / str(index)
            shutil.copy2(path, saved)
            stat = path.stat()
            record.update(
                backup=str(index),
                uid=stat.st_uid,
                gid=stat.st_gid,
                mode=stat.st_mode & 0o777,
            )
        records.append(record)
    manifest = {"schema": "pse-llvm-system-v1", "records": records}
    (directory / "rollback.json").write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest


def require_trusted_backup(directory: Path) -> None:
    stat = directory.stat()
    if stat.st_uid != 0 or stat.st_mode & 0o022:
        raise ValueError(
            "rollback directory must be root-owned and not writable by others"
        )


def require_unchanged(path: Path, expected: str) -> None:
    if fingerprint(path) != expected:
        raise ValueError(f"concurrent administrator change: {path}")


def rollback(directory: Path) -> None:
    if os.geteuid() != 0:
        raise ValueError("rollback requires root")
    require_trusted_backup(directory)
    manifest = json.loads((directory / "rollback.json").read_text())
    allowed = {
        str(path) for path in (SELECTOR, ENVIRONMENT, SERVICE_ENVIRONMENT, *LINKS)
    }
    if (
        manifest.get("schema") != "pse-llvm-system-v1"
        or {r["path"] for r in manifest["records"]} != allowed
    ):
        raise ValueError("invalid rollback manifest")
    # Check all paths before changing any: do not overwrite later administrator edits.
    for record in manifest["records"]:
        observed = fingerprint(Path(record["path"]))
        if observed not in (record["before"], record.get("after", record["before"])):
            raise ValueError(f"path changed since migration: {record['path']}")
        if (
            record["before"].startswith("file:")
            and "file:" + sha(directory / record["backup"]) != record["before"]
        ):
            raise ValueError("rollback backup bytes changed")
    for record in reversed(manifest["records"]):
        path = Path(record["path"])
        before = record["before"]
        if before == "absent":
            path.unlink(missing_ok=True)
        elif before.startswith("link:"):
            atomic_link(path, Path(before[5:]))
        else:
            saved = directory / record["backup"]
            if "file:" + sha(saved) != before:
                raise ValueError("rollback backup bytes changed")
            with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
                temporary = Path(stream.name)
            shutil.copy2(saved, temporary)
            os.chown(temporary, record["uid"], record["gid"])
            temporary.chmod(record["mode"])
            temporary.replace(path)
    print(
        "Restored system files and links. New logins/services will use the previous selection."
    )


def apply(source: Path, destination: Path, version: str) -> dict:
    if os.geteuid() != 0:
        raise ValueError("--apply requires root; rerun this exact script with sudo")
    with Path("/var/lock/pse-llvm.lock").open("w") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return apply_locked(source, destination, version)


def apply_locked(source: Path, destination: Path, version: str) -> dict:
    if destination.parent != Path("/opt") or destination == SELECTOR:
        raise ValueError(
            "installation must be a versioned directory directly under /opt"
        )
    if destination.exists():
        marker = destination / ".pse-llvm-install.json"
        if not marker.is_file():
            raise ValueError("existing destination is not managed by this installer")
        validate(destination, version)
    else:
        with tempfile.TemporaryDirectory(
            prefix=".llvm-stage-", dir="/opt"
        ) as temporary:
            staged = Path(temporary) / "installation"
            stage(source, staged, version)
            for path in [staged, *staged.rglob("*")]:
                os.chown(path, 0, 0, follow_symlinks=False)
                if not path.is_symlink():
                    path.chmod(path.stat().st_mode & 0o755)
            staged.replace(destination)
        final_report = validate(destination, version)
        marker = destination / ".pse-llvm-install.json"
        metadata = json.loads(marker.read_text())
        metadata.update(final_report)
        atomic_text(marker, json.dumps(metadata, indent=2) + "\n")
    stamp = datetime.datetime.now(datetime.UTC).strftime("%Y%m%dT%H%M%S.%fZ")
    directory = Path("/var/backups/pse-llvm") / stamp
    manifest = backup(directory)
    print(
        f"Rollback: sudo python3 {Path(__file__).resolve()} --rollback {directory}",
        flush=True,
    )
    updates = {
        ENVIRONMENT: environment_text(
            ENVIRONMENT.read_text() if ENVIRONMENT.exists() else ""
        ),
        SERVICE_ENVIRONMENT: "# LLVM selection for systemd user service environments.\n"
        + "\n".join(f"{key}={value}" for key, value in variables(SELECTOR).items())
        + f"\nPATH={SELECTOR / 'bin'}:${{PATH}}\n",
    }
    try:
        for record in manifest["records"]:
            path = Path(record["path"])
            require_unchanged(path, record["before"])
            target = destination if path == SELECTOR else SELECTOR / "bin" / path.name
            record["after"] = (
                "link:" + str(target)
                if path == SELECTOR or path in LINKS
                else "file:" + hashlib.sha256(updates[path].encode()).hexdigest()
            )
            atomic_text(
                directory / "rollback.json", json.dumps(manifest, indent=2) + "\n"
            )
            if path == SELECTOR:
                atomic_link(path, destination)
            elif path in LINKS:
                atomic_link(path, SELECTOR / "bin" / path.name)
            else:
                atomic_text(path, updates[path])
            record["after"] = fingerprint(path)
            (directory / "rollback.json").write_text(
                json.dumps(manifest, indent=2) + "\n"
            )
        report = validate(SELECTOR, version)
        report["rollback"] = str(directory)
        (directory / "acceptance.json").write_text(json.dumps(report, indent=2) + "\n")
    except BaseException:
        rollback(directory)
        raise
    return report


def verify_processes(version: str) -> dict:
    report = validate(SELECTOR, version)
    command = "command -v clang; command -v clang++; command -v llvm-config; llvm-config --version; clang -print-resource-dir"
    for shell in (["/bin/bash", "--noprofile", "--norc", "-c"], ["/bin/bash", "-lc"]):
        observed = capture([*shell, command], env=clean_env(SELECTOR))
        lines = observed.splitlines()
        for name, path in zip(TOOLS, lines[:3], strict=True):
            if Path(path).resolve() != (SELECTOR / "bin" / name).resolve():
                raise ValueError(f"shell startup overrides relocated {name}: {path}")
        report["login" if "-lc" in shell else "noninteractive"] = observed
    # A standard retained PATH must also resolve the root-owned /usr/local/bin links.
    observed = capture(
        ["/bin/sh", "-c", command], env={"PATH": SYSTEM_PATH, "LANG": "C"}
    )
    for name, path in zip(TOOLS, observed.splitlines()[:3], strict=True):
        if Path(path).resolve() != (SELECTOR / "bin" / name).resolve():
            raise ValueError(f"standard PATH does not resolve selected {name}")
    report["standard_path"] = observed
    # Run --verify as the logged-in user after daemon-reload/relogin to test service inheritance.
    if os.geteuid() != 0:
        service = subprocess.run(
            [
                "systemd-run",
                "--user",
                "--quiet",
                "--wait",
                "--pipe",
                "--collect",
                "/bin/sh",
                "-c",
                command,
            ],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        if service.returncode:
            raise ValueError(
                "user service verification failed: " + service.stderr.strip()
            )
        for name, path in zip(TOOLS, service.stdout.splitlines()[:3], strict=True):
            if Path(path).resolve() != (SELECTOR / "bin" / name).resolve():
                raise ValueError(f"user service still selects an older {name}")
        report["user_service"] = service.stdout
    else:
        report["user_service"] = (
            "not_run: run --verify as the logged-in user after systemctl --user daemon-reload"
        )
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--plan", action="store_true")
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--verify", action="store_true")
    mode.add_argument("--rollback", type=Path)
    mode.add_argument(
        "--stage-check",
        type=Path,
        help="validate relocation in a new disposable directory, without system changes",
    )
    parser.add_argument("--source", type=Path)
    parser.add_argument(
        "--destination", type=Path, default=Path("/opt/llvm-23.1.2-z3-5.1")
    )
    parser.add_argument("--version", default="23.1.2")
    args = parser.parse_args()
    try:
        if args.rollback:
            rollback(args.rollback.resolve())
            return 0
        if args.verify:
            result = verify_processes(args.version)
        else:
            source = args.source or Path(capture(["llvm-config", "--prefix"]))
            source = source.resolve(strict=True)
            if args.apply:
                result = apply(source, args.destination, args.version)
            elif args.stage_check:
                result = stage(source, args.stage_check.resolve(), args.version)
            else:
                result = {
                    "mode": "plan",
                    "source": str(source),
                    "source_version": capture(
                        [str(source / "bin/llvm-config"), "--version"]
                    ),
                    "destination": str(args.destination),
                    "selector": str(SELECTOR),
                    "system_files": [
                        str(ENVIRONMENT),
                        str(SERVICE_ENVIRONMENT),
                        *map(str, LINKS),
                    ],
                    "actions": [
                        "copy and retain the home installation",
                        "bundle the resolved Z3 shared library",
                        "replace ELF RUNPATHs with origin-relative paths",
                        "validate relocated binaries, libclang, resource headers, C/C++ compilation and execution",
                        "publish root-owned installation and atomically replace selector/links",
                        "back up system files and preserve rollback state",
                        "verify fresh login and user-service environments after activation",
                    ],
                    "root_required_for_apply": True,
                }
        print(json.dumps(result, indent=2))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"LLVM setup failed: {error}", file=sys.stderr)
        if isinstance(error, subprocess.CalledProcessError) and error.stderr:
            print(error.stderr, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
