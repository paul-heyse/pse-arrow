# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `janaf` and `iapws` enumerators: synthetic index pages with the structure each enumerator
reads (invented species, codes, titles and slugs, written for these tests) in, pages and stable
names out; a page that does not have the expected structure is an error naming it."""

from __future__ import annotations

import json
import re
from collections.abc import Callable, Iterator
from pathlib import Path

import pytest
from test_acquire_support import (
    FIXED_NOW,
    FIXTURES,
    FakeClock,
    HttpFixture,
    Route,
    make_context,
    run_acquire,
    tree_files,
    write_manifest,
)

from thermo_knowledge.acquire.enumerators import ENUMERATORS, PageTarget
from thermo_knowledge.acquire.errors import AcquireError
from thermo_knowledge.acquire.manifest import PagesSpec
from thermo_knowledge.acquire.pages import SIDECAR_SUFFIX

ENUMERATOR_FIXTURES = FIXTURES / "enumerators"
PIN = FIXED_NOW.date().isoformat()

JANAF_INDEX = "https://tables.example/formula.html"
JANAF_CODES = "https://tables.example/dat/janaf.json"
IAPWS_INDEX = "https://documents.example/technical-guidance/release"

JANAF_FORMULA_HTML = (ENUMERATOR_FIXTURES / "species-index.html").read_text()
JANAF_CODES_JSON = (ENUMERATOR_FIXTURES / "species-codes.json").read_text()
IAPWS_HTML = (ENUMERATOR_FIXTURES / "documents-index.html").read_text()

IAPWS_SLUGS = [
    "QuartzCond",
    "QuartzVisc",
    "BrineLiquid",
    "FluxWater",
    "Brine-Surf",
    "TBLX",
    "Note9",
    "Note8",
]


def spec(enumerator: str, *urls: str) -> PagesSpec:
    return PagesSpec(enumerator=enumerator, min_interval_seconds=0, urls=list(urls))


def reader(pages: dict[str, str]) -> tuple[Callable[[str], str], list[str]]:
    """A `fetch_text` over fixed pages that records which URLs were read."""
    requested: list[str] = []

    def fetch_text(url: str) -> str:
        requested.append(url)
        if url not in pages:
            raise AcquireError(f"{url}: index page answered HTTP 404")
        return pages[url]

    return fetch_text, requested


def enumerate_pages(
    enumerator: str, pages: dict[str, str], *urls: str
) -> tuple[list[PageTarget], list[str]]:
    fetch_text, requested = reader(pages)
    targets = list(ENUMERATORS[enumerator].enumerate(spec(enumerator, *urls), fetch_text))
    return targets, requested


def janaf_pages(formula: str = JANAF_FORMULA_HTML, codes: str = JANAF_CODES_JSON) -> dict[str, str]:
    return {JANAF_INDEX: formula, JANAF_CODES: codes}


def with_codes(codes: object) -> str:
    return json.dumps(codes)


# -- janaf ----------------------------------------------------------------------------------


def test_janaf_yields_the_index_the_code_list_and_every_table() -> None:
    targets, requested = enumerate_pages("janaf", janaf_pages(), JANAF_INDEX)
    assert requested == [JANAF_INDEX, JANAF_CODES]
    assert targets[0] == PageTarget(JANAF_INDEX, "formula.html")
    assert targets[1] == PageTarget(JANAF_CODES, "janaf.json")
    tables = targets[2:]
    assert len(tables) == 12
    assert tables[0] == PageTarget("https://tables.example/tables/Zq-001.txt", "Zq-001.txt")
    assert tables[-1] == PageTarget("https://tables.example/tables/Zq-012.txt", "Zq-012.txt")
    assert [t.name for t in tables] == [f"Zq-{n:03d}.txt" for n in range(1, 13)]


def test_janaf_names_and_urls_are_unique_and_stable() -> None:
    first, _ = enumerate_pages("janaf", janaf_pages(), JANAF_INDEX)
    second, _ = enumerate_pages("janaf", janaf_pages(), JANAF_INDEX)
    assert first == second
    assert len({t.name for t in first}) == len(first)
    assert len({t.url for t in first}) == len(first)
    assert all("/" not in t.name and not t.name.startswith(".") for t in first)
    assert not any(t.name.endswith(SIDECAR_SUFFIX) for t in first)


def test_janaf_derives_the_code_list_and_tables_from_the_index_url() -> None:
    base = "http://127.0.0.1:9/site/formula.html"
    pages = {base: JANAF_FORMULA_HTML, "http://127.0.0.1:9/site/dat/janaf.json": JANAF_CODES_JSON}
    targets, _ = enumerate_pages("janaf", pages, base)
    assert targets[1].url == "http://127.0.0.1:9/site/dat/janaf.json"
    assert targets[2].url == "http://127.0.0.1:9/site/tables/Zq-001.txt"


@pytest.mark.parametrize(
    "urls",
    [(), (JANAF_INDEX, "https://tables.example/name.html")],
)
def test_janaf_needs_exactly_one_index_url(urls: tuple[str, ...]) -> None:
    with pytest.raises(AcquireError, match="exactly one URL"):
        enumerate_pages("janaf", janaf_pages(), *urls)


def test_janaf_empty_index_is_an_error_naming_the_page() -> None:
    empty = "<html><body><pre>\nJCODE FORMULA NAME PAGE\n</pre></body></html>"
    with pytest.raises(AcquireError, match="formula.html: no species rows"):
        enumerate_pages("janaf", janaf_pages(formula=empty), JANAF_INDEX)
    with pytest.raises(AcquireError, match="formula.html: no species rows"):
        enumerate_pages("janaf", janaf_pages(formula=""), JANAF_INDEX)


def test_janaf_index_without_a_listing_is_an_error_naming_the_page() -> None:
    no_pre = "<html><body><p>Formula index</p></body></html>"
    with pytest.raises(AcquireError, match="formula.html: no species rows"):
        enumerate_pages("janaf", janaf_pages(formula=no_pre), JANAF_INDEX)


def test_janaf_unreadable_code_list_is_an_error_naming_the_file() -> None:
    with pytest.raises(AcquireError, match="janaf.json: not valid JSON"):
        enumerate_pages("janaf", janaf_pages(codes="<html>moved</html>"), JANAF_INDEX)
    with pytest.raises(AcquireError, match="janaf.json: expected an object"):
        enumerate_pages("janaf", janaf_pages(codes="[]"), JANAF_INDEX)
    with pytest.raises(AcquireError, match="janaf.json: expected `display` and `index`"):
        enumerate_pages("janaf", janaf_pages(codes=with_codes({"display": []})), JANAF_INDEX)
    with pytest.raises(AcquireError, match="janaf.json: the `index` list is empty"):
        enumerate_pages(
            "janaf", janaf_pages(codes=with_codes({"display": [], "index": []})), JANAF_INDEX
        )


def test_janaf_code_list_must_be_consistent() -> None:
    document = json.loads(JANAF_CODES_JSON)
    bad_code = {**document, "index": [*document["index"][:-1], "../etc/passwd"]}
    with pytest.raises(AcquireError, match=r"janaf.json: entry 11 of `index` is '../etc/passwd'"):
        enumerate_pages("janaf", janaf_pages(codes=with_codes(bad_code)), JANAF_INDEX)
    duplicate = {**document, "index": [*document["index"][:-1], document["index"][0]]}
    with pytest.raises(AcquireError, match="listed more than once: Zq-001"):
        enumerate_pages("janaf", janaf_pages(codes=with_codes(duplicate)), JANAF_INDEX)
    short = {**document, "display": document["display"][:-1]}
    with pytest.raises(AcquireError, match="`display` has 11 entries but `index` has 12"):
        enumerate_pages("janaf", janaf_pages(codes=with_codes(short)), JANAF_INDEX)


def test_janaf_index_and_code_list_must_describe_the_same_tables() -> None:
    document = json.loads(JANAF_CODES_JSON)
    fewer = {key: value[:-1] for key, value in document.items()}
    with pytest.raises(AcquireError, match=r"lists 12 species but .* lists 11"):
        enumerate_pages("janaf", janaf_pages(codes=with_codes(fewer)), JANAF_INDEX)


def test_janaf_a_page_that_cannot_be_read_stops_the_enumeration() -> None:
    with pytest.raises(AcquireError, match="dat/janaf.json: index page answered HTTP 404"):
        enumerate_pages("janaf", {JANAF_INDEX: JANAF_FORMULA_HTML}, JANAF_INDEX)


# -- iapws ----------------------------------------------------------------------------------


def test_iapws_yields_the_index_and_every_document_pdf_in_page_order() -> None:
    targets, requested = enumerate_pages("iapws", {IAPWS_INDEX: IAPWS_HTML}, IAPWS_INDEX)
    assert requested == [IAPWS_INDEX]
    assert targets[0] == PageTarget(IAPWS_INDEX, "release.html")
    assert [t.name for t in targets[1:]] == [f"{slug}.pdf" for slug in IAPWS_SLUGS]
    assert [t.url for t in targets[1:]] == [
        f"https://documents.example/technical-guidance/release/{slug}.download" for slug in IAPWS_SLUGS
    ]


def test_iapws_names_and_urls_are_unique_and_stable() -> None:
    first, _ = enumerate_pages("iapws", {IAPWS_INDEX: IAPWS_HTML}, IAPWS_INDEX)
    second, _ = enumerate_pages("iapws", {IAPWS_INDEX: IAPWS_HTML}, IAPWS_INDEX)
    assert first == second
    assert len({t.name for t in first}) == len(first)
    assert len({t.url for t in first}) == len(first)
    assert all("/" not in t.name and not t.name.startswith(".") for t in first)


@pytest.mark.parametrize("urls", [(), (IAPWS_INDEX, "https://documents.example/technical-guidance")])
def test_iapws_needs_exactly_one_index_url(urls: tuple[str, ...]) -> None:
    with pytest.raises(AcquireError, match="exactly one URL"):
        enumerate_pages("iapws", {IAPWS_INDEX: IAPWS_HTML}, *urls)


def test_iapws_empty_or_unrelated_index_is_an_error_naming_the_page() -> None:
    for html in ("", "<html><body><h3>Releases</h3></body></html>", "<html>Maintenance</html>"):
        with pytest.raises(AcquireError, match=f"{IAPWS_INDEX}: no document entries"):
            enumerate_pages("iapws", {IAPWS_INDEX: html}, IAPWS_INDEX)


def test_iapws_a_missing_document_class_is_an_error_naming_the_page() -> None:
    start = IAPWS_HTML.index("<h3>Guidelines</h3>")
    end = IAPWS_HTML.index("<h3>Advisory Notes</h3>")
    without_guidelines = IAPWS_HTML[:start] + IAPWS_HTML[end:]
    with pytest.raises(AcquireError, match=rf"{IAPWS_INDEX}: no documents listed under Guidelines"):
        enumerate_pages("iapws", {IAPWS_INDEX: without_guidelines}, IAPWS_INDEX)


def test_iapws_documents_under_an_unknown_heading_are_an_error() -> None:
    renamed = IAPWS_HTML.replace("<h3>Guidelines</h3>", "<h3>Technical Reports</h3>")
    with pytest.raises(AcquireError, match="unexpected heading 'Technical Reports'"):
        enumerate_pages("iapws", {IAPWS_INDEX: renamed}, IAPWS_INDEX)


def test_iapws_a_link_to_another_place_is_an_error() -> None:
    foreign = IAPWS_HTML.replace(
        'href="https://documents.example/technical-guidance/release/QuartzCond"',
        'href="https://example.org/elsewhere/QuartzCond"',
    )
    assert foreign != IAPWS_HTML
    with pytest.raises(
        AcquireError, match=rf"{IAPWS_INDEX}: the link .*elsewhere.* is not a document page"
    ):
        enumerate_pages("iapws", {IAPWS_INDEX: foreign}, IAPWS_INDEX)


def test_iapws_a_slug_listed_twice_is_an_error() -> None:
    doubled = IAPWS_HTML.replace("release/QuartzVisc", "release/QuartzCond")
    with pytest.raises(AcquireError, match="the document slug 'QuartzCond' is listed twice"):
        enumerate_pages("iapws", {IAPWS_INDEX: doubled}, IAPWS_INDEX)


# -- end to end over a loopback server --------------------------------------------------------


@pytest.fixture
def clock() -> FakeClock:
    return FakeClock()


@pytest.fixture
def server(clock: FakeClock, monkeypatch: pytest.MonkeyPatch) -> Iterator[HttpFixture]:
    monkeypatch.setenv("NO_PROXY", "127.0.0.1")
    monkeypatch.delenv("HTTP_PROXY", raising=False)
    monkeypatch.delenv("http_proxy", raising=False)
    with HttpFixture(stamp=clock) as fixture:
        yield fixture


def acquire(tmp_path: Path, clock: FakeClock, source_id: str, body: str):
    sources = tmp_path / "sources"
    write_manifest(sources, source_id, body, tier="B")
    ctx = make_context(tmp_path, clock=clock)
    lock = tmp_path / "sources.lock"
    [result] = run_acquire(ctx, sources, lock)
    return ctx, lock, result


def test_janaf_end_to_end_stores_the_index_the_code_list_and_the_tables(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/formula.html"] = Route(
        JANAF_FORMULA_HTML.encode(), headers={"Content-Type": "text/html"}
    )
    server.routes["/dat/janaf.json"] = Route(
        JANAF_CODES_JSON.encode(), headers={"Content-Type": "application/json"}
    )
    codes = json.loads(JANAF_CODES_JSON)["index"]
    for code in codes:
        server.routes[f"/tables/{code}.txt"] = Route(
            f"T(K)\tCp\n0\t{code}\n".encode(), headers={"Content-Type": "text/plain"}
        )
    body = (
        'kind = "pages"\nenumerator = "janaf"\nmin_interval_seconds = 5\n'
        f'urls = ["{server.url("/formula.html")}"]\n'
    )
    ctx, lock, result = acquire(tmp_path, clock, "janaf_loopback", body)
    assert result.ok, result.message
    files = tree_files(ctx.raw_dir / "janaf_loopback" / PIN / "tree")
    bodies = {name for name in files if not name.endswith(SIDECAR_SUFFIX)}
    assert bodies == {"formula.html", "janaf.json", *(f"{code}.txt" for code in codes)}
    assert files["Zq-005.txt"] == b"T(K)\tCp\n0\tZq-005\n"
    assert files["formula.html"].decode() == JANAF_FORMULA_HTML
    sidecar = json.loads(files[f"Zq-005.txt{SIDECAR_SUFFIX}"])
    assert sidecar["url"] == server.url("/tables/Zq-005.txt")
    assert sidecar["status"] == 200
    entry = json.loads(lock.read_text())["sources"]["janaf_loopback"]
    assert entry["file_count"] == 2 * (2 + len(codes))
    # the index pages are read for enumeration and stored; the tables are fetched once each
    requested = server.requested()
    assert requested.count("/formula.html") == 2
    assert requested.count("/dat/janaf.json") == 2
    assert all(requested.count(f"/tables/{code}.txt") == 1 for code in codes)
    assert requested.count("/robots.txt") == 1
    assert all(
        later - earlier >= 5
        for earlier, later in zip(
            [r.stamp for r in server.log], [r.stamp for r in server.log][1:], strict=False
        )
    )


def test_janaf_end_to_end_malformed_index_fetches_no_table(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/formula.html"] = Route(b"<html><pre>\n</pre></html>")
    body = (
        'kind = "pages"\nenumerator = "janaf"\nmin_interval_seconds = 0\n'
        f'urls = ["{server.url("/formula.html")}"]\n'
    )
    _ctx, lock, result = acquire(tmp_path, clock, "janaf_loopback", body)
    assert not result.ok
    assert "formula.html: no species rows" in result.message
    assert not any(path.startswith("/tables/") for path in server.requested())
    assert not lock.exists()


def test_iapws_end_to_end_follows_the_download_redirects(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/robots.txt"] = Route(b"User-agent: *\nDisallow: /private/\nDisallow: /admin/\n")
    html = IAPWS_HTML.replace("https://documents.example", server.url(""))
    server.routes["/technical-guidance/release"] = Route(
        html.encode(), headers={"Content-Type": "text/html"}
    )
    for number, slug in enumerate(IAPWS_SLUGS):
        server.routes[f"/technical-guidance/release/{slug}.download"] = Route(
            b"", 302, {"Location": f"/public/documents/id{number}/{slug}.pdf"}
        )
        server.routes[f"/public/documents/id{number}/{slug}.pdf"] = Route(
            f"%PDF-1.4 {slug}".encode(), headers={"Content-Type": "application/pdf"}
        )
    body = (
        'kind = "pages"\nenumerator = "iapws"\nmin_interval_seconds = 5\n'
        f'urls = ["{server.url("/technical-guidance/release")}"]\n'
    )
    ctx, lock, result = acquire(tmp_path, clock, "iapws_loopback", body)
    assert result.ok, result.message
    files = tree_files(ctx.raw_dir / "iapws_loopback" / PIN / "tree")
    bodies = {name for name in files if not name.endswith(SIDECAR_SUFFIX)}
    assert bodies == {"release.html", *(f"{slug}.pdf" for slug in IAPWS_SLUGS)}
    assert files["TBLX.pdf"] == b"%PDF-1.4 TBLX"
    sidecar = json.loads(files[f"TBLX.pdf{SIDECAR_SUFFIX}"])
    assert sidecar["url"] == server.url("/technical-guidance/release/TBLX.download")
    assert sidecar["redirects"] == [sidecar["url"]]
    assert re.fullmatch(r".*/public/documents/id5/TBLX\.pdf", sidecar["final_url"])
    assert sidecar["status"] == 200
    entry = json.loads(lock.read_text())["sources"]["iapws_loopback"]
    assert entry["file_count"] == 2 * (1 + len(IAPWS_SLUGS))
    stamps = [request.stamp for request in server.log]
    assert all(later - earlier >= 5 for earlier, later in zip(stamps, stamps[1:], strict=False))


def test_iapws_end_to_end_malformed_index_fetches_no_document(
    tmp_path: Path, server: HttpFixture, clock: FakeClock
) -> None:
    server.routes["/technical-guidance/release"] = Route(b"<html>Maintenance</html>")
    body = (
        'kind = "pages"\nenumerator = "iapws"\nmin_interval_seconds = 0\n'
        f'urls = ["{server.url("/technical-guidance/release")}"]\n'
    )
    _ctx, lock, result = acquire(tmp_path, clock, "iapws_loopback", body)
    assert not result.ok
    assert "no document entries" in result.message
    assert not any(path.endswith(".download") for path in server.requested())
    assert not lock.exists()
