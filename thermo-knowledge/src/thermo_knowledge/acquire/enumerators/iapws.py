# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Enumerator `iapws`: IAPWS releases, supplementary releases, guidelines and advisory notes.

One index page (`https://iapws.org/technical-guidance/release`, the single entry of `urls`) lists
all four document classes, each under its own `<h3>` heading. Every document is a
`<p class="content_list_title document"><a href="<index>/<slug>">` entry. The PDF of a document is
`<document page>.download`, which the site redirects to `/public/documents/<id>/<file>.pdf`; the
polite fetcher follows that redirect hop by hop.

The enumerator yields the index page (as `release.html`, so the snapshot records what the index
listed) and then `<slug>.pdf` for every document, in page order.
"""

from __future__ import annotations

import posixpath
import re
from collections.abc import Callable, Iterator
from dataclasses import dataclass
from html.parser import HTMLParser
from typing import TYPE_CHECKING
from urllib.parse import urljoin, urlsplit, urlunsplit

from thermo_knowledge.acquire.enumerators import PageTarget
from thermo_knowledge.acquire.errors import AcquireError

if TYPE_CHECKING:
    from thermo_knowledge.acquire.manifest import PagesSpec

INDEX_NAME = "release.html"
DOCUMENT_CLASSES = ("Releases", "Supplementary Releases", "Guidelines", "Advisory Notes")
_SLUG = re.compile(r"[A-Za-z0-9][A-Za-z0-9._-]*")


@dataclass(frozen=True)
class _Entry:
    section: str
    href: str
    title: str


class _DocumentList(HTMLParser):
    """Collects `(section, href, title)` for every document entry, in page order."""

    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.entries: list[_Entry] = []
        self._section = ""
        self._heading: list[str] | None = None
        self._in_title = False
        self._href: str | None = None
        self._text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        values = dict(attrs)
        if tag == "h3":
            self._heading = []
        elif tag == "p" and {"content_list_title", "document"} <= set(
            (values.get("class") or "").split()
        ):
            self._in_title = True
        elif tag == "a" and self._in_title and self._href is None and values.get("href"):
            self._href = values["href"]
            self._text = []

    def handle_data(self, data: str) -> None:
        if self._heading is not None:
            self._heading.append(data)
        if self._href is not None:
            self._text.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "h3" and self._heading is not None:
            self._section = " ".join("".join(self._heading).split())
            self._heading = None
        elif tag == "a" and self._href is not None:
            title = " ".join("".join(self._text).split())
            self.entries.append(_Entry(self._section, self._href, title))
            self._href = None
        elif tag == "p":
            self._in_title = False


class Iapws:
    def enumerate(self, spec: PagesSpec, fetch_text: Callable[[str], str]) -> Iterator[PageTarget]:
        if len(spec.urls) != 1:
            raise AcquireError(
                "enumerator iapws: the `urls` key of [acquire] must hold exactly one URL, the "
                f"release index page; found {len(spec.urls)}"
            )
        index_url = spec.urls[0]
        parser = _DocumentList()
        parser.feed(fetch_text(index_url))
        parser.close()
        documents = _documents(index_url, parser.entries)
        yield PageTarget(index_url, INDEX_NAME)
        for slug, page_url in documents:
            yield PageTarget(f"{page_url}.download", f"{slug}.pdf")


def _documents(index_url: str, entries: list[_Entry]) -> list[tuple[str, str]]:
    """`(slug, document page URL)` per entry; any structure that is not understood is an error
    naming the index page."""
    if not entries:
        raise AcquireError(
            f"{index_url}: no document entries found (expected IAPWS document lists)"
        )
    index = urlsplit(index_url)
    index_path = index.path.rstrip("/")
    seen_classes: set[str] = set()
    documents: list[tuple[str, str]] = []
    slugs: dict[str, str] = {}
    for entry in entries:
        if entry.section not in DOCUMENT_CLASSES:
            raise AcquireError(
                f"{index_url}: document {entry.title or entry.href!r} is listed under the "
                f"unexpected heading {entry.section!r}; expected one of {', '.join(DOCUMENT_CLASSES)}"
            )
        seen_classes.add(entry.section)
        page = urlsplit(urljoin(index_url, entry.href))
        slug = posixpath.basename(page.path)
        if (
            page.netloc != index.netloc
            or posixpath.dirname(page.path) != index_path
            or not _SLUG.fullmatch(slug)
        ):
            raise AcquireError(
                f"{index_url}: the link {entry.href!r} of {entry.title!r} is not a document page "
                f"directly under {index_path}/"
            )
        if slug in slugs:
            raise AcquireError(
                f"{index_url}: the document slug {slug!r} is listed twice "
                f"({slugs[slug]!r} and {entry.title!r})"
            )
        slugs[slug] = entry.title
        documents.append((slug, urlunsplit((page.scheme, page.netloc, page.path, "", ""))))
    missing = [name for name in DOCUMENT_CLASSES if name not in seen_classes]
    if missing:
        raise AcquireError(
            f"{index_url}: no documents listed under {', '.join(missing)}; "
            "the page structure may have changed"
        )
    return documents
