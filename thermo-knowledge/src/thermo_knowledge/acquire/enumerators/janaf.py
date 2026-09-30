# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Enumerator `janaf`: the NIST-JANAF Thermochemical Tables.

The formula index (`https://janaf.nist.gov/formula.html`, the single entry of `urls`) is a plain
`<pre>` listing with no links: each row gives the species' JCODE, formula, name and page number.
The links are made by the site's search script, which reads `dat/janaf.json`: `display` holds one
`"<formula>, <name> (<phase>)"` label per species and `index` the matching table code
`<El>-<nnn>`. The table of a species is `tables/<code>.html`, with the tab-delimited data in the
sibling `tables/<code>.txt`.

The enumerator yields the formula index (`formula.html`) and the code list (`janaf.json`), so the
snapshot records what the site said, and then `<code>.txt` for every species in index order. The
number of rows in the formula index must equal the number of codes; a difference is an error.
"""

from __future__ import annotations

import json
import re
from collections.abc import Callable, Iterator
from html.parser import HTMLParser
from typing import TYPE_CHECKING
from urllib.parse import urljoin

from thermo_knowledge.acquire.enumerators import PageTarget
from thermo_knowledge.acquire.errors import AcquireError

if TYPE_CHECKING:
    from thermo_knowledge.acquire.manifest import PagesSpec

INDEX_NAME = "formula.html"
CODES_NAME = "janaf.json"
CODES_PATH = "dat/janaf.json"
_CODE = re.compile(r"[A-Z][a-z]?-\d{3}")
_ROW = re.compile(r"^\s*\d+\s+\S+\s+.*\S\s+\d+\s*$")


class _PreText(HTMLParser):
    """The text inside `<pre>` elements."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.text: list[str] = []
        self._depth = 0

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag == "pre":
            self._depth += 1

    def handle_endtag(self, tag: str) -> None:
        if tag == "pre" and self._depth:
            self._depth -= 1

    def handle_data(self, data: str) -> None:
        if self._depth:
            self.text.append(data)


def _index_rows(index_url: str, html: str) -> int:
    parser = _PreText()
    parser.feed(html)
    parser.close()
    rows = [line for line in "".join(parser.text).splitlines() if _ROW.match(line)]
    if not rows:
        raise AcquireError(
            f"{index_url}: no species rows found (expected a <pre> listing of JCODE, formula, "
            "name and page)"
        )
    return len(rows)


def _codes(codes_url: str, text: str) -> list[str]:
    try:
        document = json.loads(text)
    except json.JSONDecodeError as error:
        raise AcquireError(f"{codes_url}: not valid JSON: {error}") from error
    if not isinstance(document, dict):
        raise AcquireError(f"{codes_url}: expected an object with `display` and `index` lists")
    display, index = document.get("display"), document.get("index")
    if not (isinstance(display, list) and isinstance(index, list)):
        raise AcquireError(f"{codes_url}: expected `display` and `index` to be lists")
    if not index:
        raise AcquireError(f"{codes_url}: the `index` list is empty")
    if len(display) != len(index):
        raise AcquireError(
            f"{codes_url}: `display` has {len(display)} entries but `index` has {len(index)}"
        )
    codes: list[str] = []
    for position, code in enumerate(index):
        if not (isinstance(code, str) and _CODE.fullmatch(code)):
            raise AcquireError(
                f"{codes_url}: entry {position} of `index` is {code!r}, not a table code "
                "like 'Al-001'"
            )
        codes.append(code)
    duplicates = sorted({code for code in codes if codes.count(code) > 1})
    if duplicates:
        raise AcquireError(
            f"{codes_url}: table codes listed more than once: {', '.join(duplicates)}"
        )
    return codes


class Janaf:
    def enumerate(self, spec: PagesSpec, fetch_text: Callable[[str], str]) -> Iterator[PageTarget]:
        if len(spec.urls) != 1:
            raise AcquireError(
                "enumerator janaf: the `urls` key of [acquire] must hold exactly one URL, the "
                f"formula index page; found {len(spec.urls)}"
            )
        index_url = spec.urls[0]
        rows = _index_rows(index_url, fetch_text(index_url))
        codes_url = urljoin(index_url, CODES_PATH)
        codes = _codes(codes_url, fetch_text(codes_url))
        if rows != len(codes):
            raise AcquireError(
                f"{index_url}: the formula index lists {rows} species but {codes_url} lists "
                f"{len(codes)}; the two do not describe the same tables"
            )
        yield PageTarget(index_url, INDEX_NAME)
        yield PageTarget(codes_url, CODES_NAME)
        for code in codes:
            yield PageTarget(urljoin(index_url, f"tables/{code}.txt"), f"{code}.txt")
