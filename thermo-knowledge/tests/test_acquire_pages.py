# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Kind `pages`: robots.txt, per-host interval, resumability, stopping on server errors."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from test_acquire_support import (
    FIXED_NOW,
    FakeClock,
    HttpFixture,
    Route,
    independent_tree_hash,
    make_context,
    run_acquire,
    sha256,
    tree_files,
    write_manifest,
)

from thermo_knowledge.acquire.enumerators import stable_name
from thermo_knowledge.acquire.pages import MAX_CONSECUTIVE_ERRORS, SIDECAR_SUFFIX
from thermo_knowledge.acquire.runtime import USER_AGENT

PIN = FIXED_NOW.date().isoformat()


@pytest.fixture
def clock() -> FakeClock:
    return FakeClock()


@pytest.fixture
def server(clock: FakeClock, monkeypatch: pytest.MonkeyPatch):
    monkeypatch.setenv("NO_PROXY", "127.0.0.1")
    monkeypatch.delenv("HTTP_PROXY", raising=False)
    monkeypatch.delenv("http_proxy", raising=False)
    with HttpFixture(stamp=clock) as fixture:
        fixture.routes["/robots.txt"] = Route(b"User-agent: *\nDisallow: /private/\n")
        for number in range(1, 7):
            fixture.routes[f"/p/{number}"] = Route(
                f"<html>page {number}</html>".encode(), headers={"Content-Type": "text/html"}
            )
        fixture.routes["/private/secret"] = Route(b"secret")
        yield fixture


def setup(
    tmp_path: Path,
    server: HttpFixture,
    clock: FakeClock,
    paths: list[str],
    *,
    interval: float = 5.0,
    limit: int | None = None,
):
    urls = ", ".join(f'"{server.url(path)}"' for path in paths)
    body = (
        f'kind = "pages"\nenumerator = "url_list"\nmin_interval_seconds = {interval}\n'
        f"urls = [{urls}]\n" + (f"limit = {limit}\n" if limit else "")
    )
    sources = tmp_path / "sources"
    write_manifest(sources, "webpages", body)
    return make_context(tmp_path, clock=clock), sources, tmp_path / "sources.lock"


def page_paths(count: int) -> list[str]:
    return [f"/p/{number}" for number in range(1, count + 1)]


def test_pages_are_stored_with_a_sidecar_and_recorded(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(3))
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    pin_dir = ctx.raw_dir / "webpages" / PIN
    files = tree_files(pin_dir / "tree")
    name = stable_name(server.url("/p/2"))
    assert files[name] == b"<html>page 2</html>"
    sidecar = json.loads(files[f"{name}{SIDECAR_SUFFIX}"])
    assert sidecar["url"] == server.url("/p/2")
    assert sidecar["status"] == 200
    assert sidecar["sha256"] == sha256(b"<html>page 2</html>")
    assert sidecar["retrieved"] == "2026-09-30T12:00:00Z"
    assert ["content-type", "text/html"] in [[k.lower(), v] for k, v in sidecar["headers"]]
    assert len(files) == 6
    entry = json.loads(lock.read_text())["sources"]["webpages"]
    assert entry["kind"] == "pages"
    assert entry["pin"] == PIN
    assert entry["file_count"] == 6
    assert entry["tree_hash"] == independent_tree_hash(pin_dir / "tree")
    assert not (ctx.raw_dir / "webpages" / ".partial").exists()


def test_a_disallowed_path_is_refused_and_never_requested(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, ["/p/1", "/private/secret", "/p/2"])
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert result.status == "refused"
    assert server.url("/private/secret") in result.message
    assert "robots.txt" in result.message
    assert "/private/secret" not in server.requested()
    assert server.pages_requested() == []
    assert not lock.exists()
    assert not (ctx.raw_dir / "webpages" / PIN).exists()


def test_a_redirect_into_a_disallowed_path_is_refused(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/jump"] = Route(b"", 302, {"Location": "/private/secret"})
    ctx, sources, lock = setup(tmp_path, server, clock, ["/jump"])
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "disallowed by robots.txt" in result.message
    assert "/private/secret" not in server.requested()


def test_redirects_are_followed_hop_by_hop_and_recorded(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/old"] = Route(b"", 301, {"Location": "/p/1"})
    ctx, sources, lock = setup(tmp_path, server, clock, ["/old"])
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    name = stable_name(server.url("/old"))
    sidecar = json.loads(
        (ctx.raw_dir / "webpages" / PIN / "tree" / f"{name}{SIDECAR_SUFFIX}").read_bytes()
    )
    assert sidecar["final_url"] == server.url("/p/1")
    assert sidecar["redirects"] == [server.url("/old")]
    assert sidecar["status"] == 200
    # every hop is a request of its own, spaced like any other
    assert all(gap >= 5 for gap in gaps(server))


def gaps(server: HttpFixture) -> list[float]:
    stamps = [request.stamp for request in server.log]
    return [later - earlier for earlier, later in zip(stamps, stamps[1:], strict=False)]


def test_the_interval_between_requests_is_honoured(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(4), interval=5.0)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    assert server.requested() == ["/robots.txt", "/p/1", "/p/2", "/p/3", "/p/4"]
    assert len(gaps(server)) == 4
    assert all(gap >= 5.0 for gap in gaps(server)), gaps(server)
    assert clock.sleeps == [5.0] * 4


def test_crawl_delay_raises_the_interval(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/robots.txt"] = Route(b"User-agent: *\nCrawl-delay: 7\n")
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(3), interval=2.0)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    page_gaps = gaps(server)[1:]  # robots.txt precedes the first page; its delay was not yet known
    assert page_gaps and all(gap >= 7.0 for gap in page_gaps), gaps(server)


def test_a_longer_manifest_interval_wins_over_a_short_crawl_delay(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/robots.txt"] = Route(b"User-agent: *\nCrawl-delay: 1\n")
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(3), interval=6.0)
    run_acquire(ctx, sources, lock)
    assert all(gap >= 6.0 for gap in gaps(server))


def test_the_user_agent_names_the_project_and_carries_no_contact_details(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(1))
    run_acquire(ctx, sources, lock)
    agents = {request.user_agent for request in server.log}
    assert agents == {USER_AGENT}
    assert "thermo-knowledge" in USER_AGENT
    assert "rate-limited personal research retrieval" in USER_AGENT
    assert "@" not in USER_AGENT
    assert "http" not in USER_AGENT


def test_robots_is_read_once_per_host(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(5), interval=0)
    run_acquire(ctx, sources, lock)
    assert server.requested().count("/robots.txt") == 1


def test_a_second_run_fetches_nothing(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(3))
    run_acquire(ctx, sources, lock)
    requests = len(server.log)
    before = lock.read_bytes()
    [result] = run_acquire(ctx, sources, lock)
    assert result.status == "unchanged"
    assert len(server.log) == requests
    assert lock.read_bytes() == before


def test_repeated_server_errors_stop_the_run_and_the_next_run_resumes(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    broken = {"down": True}
    for number in (3, 4, 5, 6):
        path = f"/p/{number}"
        healthy = server.routes[path]
        server.routes[path] = lambda healthy=healthy: (
            Route(b"unavailable", 503) if broken["down"] else healthy
        )
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(6), interval=1.0)

    [first] = run_acquire(ctx, sources, lock)
    assert not first.ok
    assert "consecutive server errors" in first.message
    # p/1 and p/2 were stored; p/3.. failed MAX_CONSECUTIVE_ERRORS times and the rest were spared
    assert server.pages_requested() == ["/p/1", "/p/2", "/p/3", "/p/4", "/p/5"]
    assert MAX_CONSECUTIVE_ERRORS == 3
    assert not lock.exists()
    assert not (ctx.raw_dir / "webpages" / PIN).exists()
    partial = ctx.raw_dir / "webpages" / ".partial" / "tree"
    assert len([p for p in partial.iterdir() if not p.name.endswith(SIDECAR_SUFFIX)]) == 2

    broken["down"] = False
    before = len(server.log)
    [second] = run_acquire(ctx, sources, lock)
    assert second.ok, second.message
    assert server.pages_requested()[5:] == ["/p/3", "/p/4", "/p/5", "/p/6"]
    assert "/p/1" not in server.requested()[before:]
    assert "/p/2" not in server.requested()[before:]
    files = tree_files(ctx.raw_dir / "webpages" / PIN / "tree")
    assert len(files) == 12
    assert json.loads(lock.read_text())["sources"]["webpages"]["file_count"] == 12
    assert not (ctx.raw_dir / "webpages" / ".partial").exists()


def test_every_page_failing_stops_after_the_threshold(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    for number in range(1, 7):
        server.routes[f"/p/{number}"] = Route(b"oops", 500)
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(6))
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert len(server.pages_requested()) == MAX_CONSECUTIVE_ERRORS
    assert not lock.exists()


def test_an_isolated_server_error_does_not_stop_the_run_but_fails_it(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/p/2"] = Route(b"oops", 503)
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(4), interval=0)
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "1 page(s) were not retrieved" in result.message
    assert server.pages_requested() == ["/p/1", "/p/2", "/p/3", "/p/4"]
    assert not lock.exists()


def test_a_client_error_is_a_recorded_response(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, ["/p/1", "/p/missing"], interval=0)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message
    name = stable_name(server.url("/p/missing"))
    sidecar = json.loads(
        (ctx.raw_dir / "webpages" / PIN / "tree" / f"{name}{SIDECAR_SUFFIX}").read_bytes()
    )
    assert sidecar["status"] == 404


def test_robots_unavailable_stops_the_run(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/robots.txt"] = Route(b"", 503)
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(2))
    [result] = run_acquire(ctx, sources, lock)
    assert not result.ok
    assert "robots.txt is unavailable" in result.message
    assert server.pages_requested() == []


def test_missing_robots_allows_everything(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    del server.routes["/robots.txt"]
    ctx, sources, lock = setup(tmp_path, server, clock, ["/private/secret"], interval=0)
    [result] = run_acquire(ctx, sources, lock)
    assert result.ok, result.message


def test_limit_takes_the_first_urls(tmp_path: Path, server: HttpFixture, clock: FakeClock) -> None:
    ctx, sources, lock = setup(tmp_path, server, clock, page_paths(5), interval=0, limit=2)
    run_acquire(ctx, sources, lock)
    assert server.pages_requested() == ["/p/1", "/p/2"]
