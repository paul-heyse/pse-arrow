# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Helpers shared by the acquisition tests: local git repositories, a loopback HTTP server,
a fake clock and manifest text. Contains no tests."""

from __future__ import annotations

import hashlib
import io
import os
import subprocess
import tarfile
import threading
import zipfile
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import UTC, datetime
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import httpx

from thermo_knowledge.acquire.runtime import Context, Runtime

FIXTURES = Path(__file__).parent / "fixtures" / "acquire"

RIGHTS = """
[[rights]]
scope = "code"
basis = "licence_grant"
spdx = "MIT"
statement = "LICENSE file, MIT."
store = "yes"
redistribute = "yes"
commercial = "yes"
attribution = true
share_alike = false
observed = "2026-09-30"
"""


def manifest_text(source_id: str, acquire: str, tier: str = "A") -> str:
    """A valid manifest with the given `[acquire]` body."""
    return (
        f'id = "{source_id}"\ntitle = "{source_id}"\ntier = "{tier}"\n\n'
        f"[acquire]\n{acquire.strip()}\n{RIGHTS}"
    )


def write_manifest(directory: Path, source_id: str, acquire: str, tier: str = "A") -> None:
    directory.mkdir(parents=True, exist_ok=True)
    (directory / f"{source_id}.toml").write_text(manifest_text(source_id, acquire, tier))


# -- git ----------------------------------------------------------------------------------

_GIT_ENV = {
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_AUTHOR_NAME": "Fixture",
    "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
    "GIT_COMMITTER_NAME": "Fixture",
    "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
}


def git(cwd: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", *arguments],
        cwd=cwd,
        env={**os.environ, **_GIT_ENV},
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()


def commit_all(repo: Path, message: str) -> str:
    git(repo, "add", "-A")
    git(repo, "commit", "--quiet", "-m", message)
    return git(repo, "rev-parse", "HEAD")


def write(repo: Path, relative: str, text: str) -> None:
    path = repo / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(text.encode())


@dataclass(frozen=True)
class GitFixture:
    """An origin repository with three commits, two tags and a submodule entry."""

    path: Path
    url: str
    commits: list[str]
    sub_commit: str

    @property
    def first(self) -> str:
        return self.commits[0]

    @property
    def second(self) -> str:
        return self.commits[1]

    @property
    def third(self) -> str:
        return self.commits[2]

    def files_at(self, commit: str) -> dict[str, bytes]:
        """Blob contents of a commit, straight from git (gitlinks are not files)."""
        files = {}
        for line in git(self.path, "ls-tree", "-r", commit).splitlines():
            meta, _, name = line.partition("\t")
            mode, kind, _sha = meta.split()
            if kind == "blob":
                blob = subprocess.run(
                    ["git", "cat-file", "blob", f"{commit}:{name}"],
                    cwd=self.path,
                    env={**os.environ, **_GIT_ENV},
                    capture_output=True,
                    check=True,
                )
                files[name] = blob.stdout
        return files


def make_git_fixture(root: Path) -> GitFixture:
    """Commits: first (tag `v0.1`, lightweight), second (tag `v0.2`, annotated), third (head).

    The second commit has the submodule `vendor/sub`; `.gitattributes` asks for CRLF
    conversion of `*.txt`, which an untouched store must not apply.
    """
    sub = root / "sub"
    sub.mkdir(parents=True)
    git(sub, "init", "--quiet")
    write(sub, "lib.txt", "library\n")
    sub_commit = commit_all(sub, "sub")

    origin = root / "origin"
    origin.mkdir()
    git(origin, "init", "--quiet")
    git(origin, "config", "uploadpack.allowFilter", "true")
    git(origin, "config", "uploadpack.allowAnySHA1InWant", "true")
    write(origin, ".gitattributes", "*.txt text eol=crlf\n")
    write(origin, "README.md", "readme one\n")
    write(origin, "LICENSE", "MIT licence text\n")
    write(origin, "NOTICE.md", "notice\n")
    write(origin, "data/a.txt", "alpha\nbeta\n")
    write(origin, "docs/notes.txt", "notes\n")
    write(origin, "tools/run.sh", "#!/bin/sh\necho run\n")
    first = commit_all(origin, "first")
    git(origin, "tag", "v0.1")

    write(origin, "data/a.txt", "alpha\nbeta\ngamma\n")
    write(origin, "data/b.json", '{"b": 2}\n')
    write(
        origin,
        ".gitmodules",
        '[submodule "vendor/sub"]\n\tpath = vendor/sub\n\turl = ' + sub.as_uri() + "\n",
    )
    git(origin, "add", "-A")
    git(origin, "update-index", "--add", "--cacheinfo", f"160000,{sub_commit},vendor/sub")
    git(origin, "commit", "--quiet", "-m", "second")
    second = git(origin, "rev-parse", "HEAD")
    git(origin, "tag", "-a", "v0.2", "-m", "annotated")

    write(origin, "README.md", "readme three\n")
    git(origin, "add", "README.md")
    git(origin, "commit", "--quiet", "-m", "third")
    third = git(origin, "rev-parse", "HEAD")
    return GitFixture(origin, origin.as_uri(), [first, second, third], sub_commit)


# -- http ---------------------------------------------------------------------------------


@dataclass
class Route:
    body: bytes = b""
    status: int = 200
    headers: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class Request:
    path: str
    user_agent: str
    stamp: float


class HttpFixture:
    """A loopback HTTP server with a request log; routes map a path to a `Route`."""

    def __init__(self, stamp: Callable[[], float] = lambda: 0.0) -> None:
        self.routes: dict[str, Route | Callable[[], Route]] = {}
        self.log: list[Request] = []
        self._stamp = stamp
        fixture = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self) -> None:  # noqa: N802
                fixture.log.append(
                    Request(self.path, self.headers.get("User-Agent", ""), fixture._stamp())
                )
                route = fixture.routes.get(self.path.split("?")[0])
                if callable(route):
                    route = route()
                if route is None:
                    route = Route(b"not found", 404)
                self.send_response(route.status)
                for name, value in route.headers.items():
                    self.send_header(name, value)
                self.send_header("Content-Length", str(len(route.body)))
                self.end_headers()
                self.wfile.write(route.body)

            def log_message(self, format: str, *args: object) -> None:
                pass

        self._server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self._thread = threading.Thread(
            target=lambda: self._server.serve_forever(poll_interval=0.01), daemon=True
        )

    def __enter__(self) -> HttpFixture:
        self._thread.start()
        return self

    def __exit__(self, *exc: object) -> None:
        self._server.shutdown()
        self._server.server_close()
        self._thread.join()

    def url(self, path: str) -> str:
        return f"http://127.0.0.1:{self._server.server_address[1]}{path}"

    def requested(self) -> list[str]:
        return [request.path for request in self.log]

    def pages_requested(self) -> list[str]:
        return [path for path in self.requested() if path != "/robots.txt"]


def make_tar(members: dict[str, bytes], *, mode: str = "w:gz") -> bytes:
    buffer = io.BytesIO()
    with tarfile.open(fileobj=buffer, mode=mode) as bundle:
        for name, data in members.items():
            info = tarfile.TarInfo(name)
            info.size = len(data)
            bundle.addfile(info, io.BytesIO(data))
    return buffer.getvalue()


def make_zip(members: dict[str, bytes]) -> bytes:
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as bundle:
        for name, data in members.items():
            bundle.writestr(name, data)
    return buffer.getvalue()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def md5(data: bytes) -> str:
    return hashlib.md5(data, usedforsecurity=False).hexdigest()


# -- clock and context --------------------------------------------------------------------


class FakeClock:
    """A clock that only moves when `sleep` is called; `sleeps` records every wait."""

    def __init__(self) -> None:
        self.now = 1000.0
        self.sleeps: list[float] = []

    def __call__(self) -> float:
        return self.now

    def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds


FIXED_NOW = datetime(2026, 9, 30, 12, 0, 0, tzinfo=UTC)


def make_context(
    tmp_path: Path,
    *,
    clock: FakeClock | None = None,
    repo_root: Path | None = None,
    client_factory: Callable[[], httpx.Client] | None = None,
) -> Context:
    from thermo_knowledge.acquire.runtime import make_client

    clock = clock or FakeClock()
    runtime = Runtime(
        clock=clock,
        sleep=clock.sleep,
        now=lambda: FIXED_NOW,
        client_factory=client_factory or make_client,
    )
    return Context(
        raw_dir=tmp_path / "raw", repo_root=repo_root or tmp_path / "repo", runtime=runtime
    )


def run_acquire(
    ctx: Context, sources_dir: Path, lock_path: Path, ids: tuple[str, ...] = ()
) -> list:
    """`tk acquire` without the CLI: the results, one per source."""
    from thermo_knowledge.acquire.manifest import load_sources
    from thermo_knowledge.acquire.runner import Acquirer, select

    manifests = select(load_sources(sources_dir), list(ids))
    return Acquirer(ctx, lock_path).run(manifests)


def tree_files(root: Path) -> dict[str, bytes]:
    """Every file under `root` by relative POSIX path."""
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file() and not path.is_symlink()
    }


def independent_tree_hash(root: Path) -> str:
    """The tree hash computed here, not by the code under test."""
    lines = sorted(
        (path.relative_to(root).as_posix(), hashlib.sha256(path.read_bytes()).hexdigest())
        for path in root.rglob("*")
        if path.is_file()
    )
    text = "".join(f"{digest}  {name}\n" for name, digest in lines)
    return hashlib.sha256(text.encode()).hexdigest()
