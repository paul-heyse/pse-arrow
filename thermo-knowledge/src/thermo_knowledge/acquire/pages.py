# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `pages`: the polite fetcher for sources that have no bulk download.

- `robots.txt` is read once per origin (`urllib.robotparser`); a disallowed URL is never fetched
  and, when any listed URL is disallowed, the source is refused before a single page is requested;
- requests to one host are at least `min_interval_seconds` apart, and at least the host's
  `Crawl-delay` when that is longer; every request counts, `robots.txt` and redirect hops included;
- the `User-Agent` names the project and the purpose and carries no personal contact details;
- each response body is stored as delivered, with a sidecar holding URL, status, headers,
  retrieval time and the body's sha256; a page whose body matches its sidecar is not fetched again,
  so an interrupted run resumes where it stopped;
- consecutive server errors (5xx, 429, transport failures) stop the run instead of being retried.

Redirects are followed here, hop by hop, so that each hop obeys robots.txt and the interval.
"""

from __future__ import annotations

import hashlib
import shutil
from collections.abc import Callable
from dataclasses import dataclass, field
from datetime import datetime
from itertools import islice
from pathlib import Path
from urllib.parse import urljoin, urlsplit
from urllib.robotparser import RobotFileParser

import httpx
import msgspec
from msgspec import Struct

from thermo_knowledge import __version__
from thermo_knowledge.acquire import store
from thermo_knowledge.acquire.enumerators import ENUMERATORS, PageTarget
from thermo_knowledge.acquire.errors import AcquireError
from thermo_knowledge.acquire.manifest import PagesSpec
from thermo_knowledge.acquire.outcome import Outcome
from thermo_knowledge.acquire.runtime import Runtime, format_time
from thermo_knowledge.acquire.store import TREE_DIR_NAME

SIDECAR_SUFFIX = ".response.json"
MAX_REDIRECTS = 5
MAX_CONSECUTIVE_ERRORS = 3
_REDIRECT_STATUSES = {301, 302, 303, 307, 308}
_CHUNK = 1024 * 64


class Sidecar(Struct, forbid_unknown_fields=True):
    """What is known about one stored response, next to its body."""

    url: str
    final_url: str
    status: int
    headers: list[tuple[str, str]]
    retrieved: str
    sha256: str
    size: int
    redirects: list[str]


@dataclass(frozen=True)
class Exchange[R]:
    result: R
    status: int
    headers: list[tuple[str, str]]
    final_url: str
    redirects: list[str]


class TransportFailure(Exception):
    """A request that produced no response."""


def _is_server_error(status: int) -> bool:
    return status >= 500 or status == 429


@dataclass
class PoliteFetcher:
    """Sequential GETs that obey robots.txt and the per-host interval."""

    runtime: Runtime
    client: httpx.Client
    min_interval: float
    _robots: dict[str, RobotFileParser] = field(default_factory=dict)
    _last_request: dict[str, float] = field(default_factory=dict)

    @property
    def user_agent(self) -> str:
        return self.client.headers.get("User-Agent", "")

    def _robots_for(self, url: str) -> RobotFileParser:
        parts = urlsplit(url)
        origin = f"{parts.scheme}://{parts.netloc}"
        if origin not in self._robots:
            self._robots[origin] = self._read_robots(origin)
        return self._robots[origin]

    def _read_robots(self, origin: str) -> RobotFileParser:
        robots_url = f"{origin}/robots.txt"
        parser = RobotFileParser(robots_url)
        try:
            fetched = self._exchange(
                robots_url,
                enforce_robots=False,
                consume=lambda response: response.read().decode("utf-8", "replace"),
            )
        except TransportFailure as error:
            raise AcquireError(f"{robots_url}: cannot read robots.txt: {error}") from error
        if fetched.status in {401, 403}:
            parser.parse(["User-agent: *", "Disallow: /"])
        elif fetched.status >= 500 or fetched.status == 429:
            raise AcquireError(
                f"{robots_url}: robots.txt is unavailable (HTTP {fetched.status}); "
                "not fetching without it"
            )
        elif 400 <= fetched.status < 500:
            parser.parse([])
        else:
            parser.parse(fetched.result.splitlines())
        return parser

    def allowed(self, url: str) -> bool:
        return self._robots_for(url).can_fetch(self.user_agent, url)

    def interval(self, url: str) -> float:
        """The wait between requests to the host of `url`: the manifest's, or `Crawl-delay`."""
        origin = f"{urlsplit(url).scheme}://{urlsplit(url).netloc}"
        parser = self._robots.get(origin)
        delay = parser.crawl_delay(self.user_agent) if parser is not None else None
        return max(self.min_interval, float(delay or 0))

    def _wait(self, url: str) -> None:
        host = urlsplit(url).netloc.lower()
        last = self._last_request.get(host)
        if last is None:
            return
        remaining = self.interval(url) - (self.runtime.clock() - last)
        if remaining > 0:
            self.runtime.sleep(remaining)

    def _mark(self, url: str) -> None:
        self._last_request[urlsplit(url).netloc.lower()] = self.runtime.clock()

    def _exchange[R](
        self, url: str, *, enforce_robots: bool, consume: Callable[[httpx.Response], R]
    ) -> Exchange[R]:
        """GET `url`, following redirects hop by hop; `consume` reads the final response."""
        redirects: list[str] = []
        current = url
        for _ in range(MAX_REDIRECTS + 1):
            if enforce_robots and not self.allowed(current):
                raise AcquireError(f"{current}: disallowed by robots.txt; not fetching it")
            requested = current
            self._wait(requested)
            try:
                with self.client.stream("GET", requested) as response:
                    location = response.headers.get("location")
                    if response.status_code in _REDIRECT_STATUSES and location:
                        redirects.append(requested)
                        current = urljoin(requested, location)
                        continue
                    result = consume(response)
                    return Exchange(
                        result=result,
                        status=response.status_code,
                        headers=list(response.headers.multi_items()),
                        final_url=requested,
                        redirects=redirects,
                    )
            except httpx.HTTPError as error:
                raise TransportFailure(f"{requested}: {error!r}") from error
            finally:
                self._mark(requested)
        raise AcquireError(f"{url}: more than {MAX_REDIRECTS} redirects")

    def get_text(self, url: str) -> str:
        """The body of an index page; any status but 200 is a failure."""
        try:
            fetched = self._exchange(
                url,
                enforce_robots=True,
                consume=lambda response: response.read().decode("utf-8", "replace"),
            )
        except TransportFailure as error:
            raise AcquireError(f"cannot read index page {error}") from error
        if fetched.status != 200:
            raise AcquireError(f"{url}: index page answered HTTP {fetched.status}")
        return fetched.result

    def fetch[R](self, url: str, consume: Callable[[httpx.Response], R]) -> Exchange[R]:
        return self._exchange(url, enforce_robots=True, consume=consume)


def _is_stored(tree: Path, name: str) -> bool:
    """A page is stored when its sidecar records the hash of the body that is there."""
    body = tree / name
    sidecar = tree / f"{name}{SIDECAR_SUFFIX}"
    if not (body.is_file() and sidecar.is_file()):
        return False
    try:
        recorded = msgspec.json.decode(sidecar.read_bytes(), type=Sidecar)
    except (msgspec.DecodeError, msgspec.ValidationError):
        return False
    return store.sha256_file(body) == recorded.sha256


def _check_targets(source_id: str, targets: list[PageTarget]) -> None:
    seen: set[str] = set()
    for target in targets:
        if (
            "/" in target.name
            or target.name.startswith(".")
            or target.name.endswith(SIDECAR_SUFFIX)
        ):
            raise AcquireError(
                f"{source_id}: {target.url}: the enumerator gave the unusable file name "
                f"{target.name!r}"
            )
        if target.name in seen:
            raise AcquireError(
                f"{source_id}: two pages share the file name {target.name!r} "
                f"(one of them is {target.url})"
            )
        seen.add(target.name)


def _retrieve(
    fetcher: PoliteFetcher, runtime: Runtime, work: Path, tree: Path, target: PageTarget
) -> int:
    """Fetch one page; returns its status. Stores the body and its sidecar unless it is an error."""
    incoming = work / "incoming"

    def consume(response: httpx.Response) -> tuple[str, int] | None:
        if _is_server_error(response.status_code):
            return None
        digest = hashlib.sha256()
        size = 0
        with incoming.open("wb") as handle:
            for chunk in response.iter_bytes(_CHUNK):
                handle.write(chunk)
                digest.update(chunk)
                size += len(chunk)
        return digest.hexdigest(), size

    retrieved: datetime = runtime.now()
    fetched = fetcher.fetch(target.url, consume)
    if fetched.result is None:
        return fetched.status
    sha256, size = fetched.result
    sidecar = Sidecar(
        url=target.url,
        final_url=fetched.final_url,
        status=fetched.status,
        headers=fetched.headers,
        retrieved=format_time(retrieved),
        sha256=sha256,
        size=size,
        redirects=fetched.redirects,
    )
    staged_sidecar = work / "incoming.json"
    staged_sidecar.write_bytes(
        msgspec.json.format(msgspec.json.encode(sidecar, order="sorted"), indent=2) + b"\n"
    )
    incoming.replace(tree / target.name)
    staged_sidecar.replace(tree / f"{target.name}{SIDECAR_SUFFIX}")
    return fetched.status


def acquire(runtime: Runtime, source_id: str, spec: PagesSpec, work: Path) -> Outcome:
    """Fetch the enumerated pages into `<work>/tree/`; `work` survives a failed run so the next
    run resumes."""
    tree = work / TREE_DIR_NAME
    tree.mkdir(exist_ok=True)
    with runtime.client_factory() as client:
        fetcher = PoliteFetcher(runtime, client, spec.min_interval_seconds)
        enumerator = ENUMERATORS[spec.enumerator]
        targets = list(islice(enumerator.enumerate(spec, fetcher.get_text), spec.limit))
        if not targets:
            raise AcquireError(f"{source_id}: enumerator {spec.enumerator!r} yielded no pages")
        _check_targets(source_id, targets)

        try:
            refused = [t.url for t in targets if not fetcher.allowed(t.url)]
        except AcquireError as error:
            raise AcquireError(f"{source_id}: {error}") from error
        if refused:
            listing = "\n  ".join(refused)
            raise AcquireError(
                f"{source_id}: refusing to fetch {len(refused)} URL(s) disallowed by robots.txt "
                f"for agent {fetcher.user_agent!r}; nothing was fetched:\n  {listing}"
            )

        consecutive_errors = 0
        failed: list[str] = []
        for target in targets:
            if _is_stored(tree, target.name):
                continue
            try:
                status = _retrieve(fetcher, runtime, work, tree, target)
            except TransportFailure as error:
                status, problem = None, str(error)
            else:
                problem = f"{target.url}: HTTP {status}"
            if status is not None and not _is_server_error(status):
                consecutive_errors = 0
                continue
            failed.append(problem)
            consecutive_errors += 1
            if consecutive_errors >= MAX_CONSECUTIVE_ERRORS:
                raise AcquireError(
                    f"{source_id}: stopping after {consecutive_errors} consecutive server errors "
                    f"(last: {problem}); pages already stored are kept and the next run resumes"
                )
        if failed:
            raise AcquireError(
                f"{source_id}: {len(failed)} page(s) were not retrieved because of server errors "
                f"({'; '.join(failed)}); the next run resumes with them"
            )

        user_agent = fetcher.user_agent
    shutil.rmtree(work / "incoming", ignore_errors=True)
    return Outcome(
        pin=runtime.now().date().isoformat(),
        resolved=None,
        urls=[],
        tool_versions={"httpx": httpx.__version__, "thermo-knowledge": __version__},
        details={
            "enumerator": spec.enumerator,
            "pages": str(len(targets)),
            "min_interval_seconds": str(spec.min_interval_seconds),
            "user_agent": user_agent,
        },
    )
