# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Enumerators: small modules that yield the pages a `pages` source fetches.

One interface, `Enumerator`; `ENUMERATORS` maps the manifest's `enumerator` name to an instance.
An enumerator that needs to read an index page does so through `fetch_text`, which goes through
the polite fetcher (robots.txt, per-host interval), never around it.
"""

from __future__ import annotations

import hashlib
import re
from collections.abc import Callable, Iterator
from dataclasses import dataclass
from pathlib import PurePosixPath
from typing import TYPE_CHECKING, Protocol
from urllib.parse import unquote, urlsplit

if TYPE_CHECKING:
    from thermo_knowledge.acquire.manifest import PagesSpec


@dataclass(frozen=True)
class PageTarget:
    """A page to fetch and the stable file name it is stored under."""

    url: str
    name: str


class Enumerator(Protocol):
    """Yields the pages of a source; each name is stable across runs for the same URL."""

    def enumerate(
        self, spec: PagesSpec, fetch_text: Callable[[str], str]
    ) -> Iterator[PageTarget]: ...


_UNSAFE = re.compile(r"[^A-Za-z0-9._-]+")
_MAX_STEM = 100


def stable_name(url: str) -> str:
    """A file name derived from the URL: readable stem, a short hash of the whole URL and the
    path's extension, so different URLs never collide and the same URL always maps the same."""
    parts = urlsplit(url)
    path = PurePosixPath(unquote(parts.path))
    suffix = path.suffix if re.fullmatch(r"\.[A-Za-z0-9]{1,8}", path.suffix) else ""
    without_suffix = path.as_posix().removesuffix(suffix)
    stem = _UNSAFE.sub("_", f"{parts.netloc}{without_suffix}").strip("_.")
    digest = hashlib.sha256(url.encode("utf-8")).hexdigest()[:8]
    return f"{stem[:_MAX_STEM] or 'page'}-{digest}{suffix}"


def _registry() -> dict[str, Enumerator]:
    from thermo_knowledge.acquire.enumerators.iapws import Iapws
    from thermo_knowledge.acquire.enumerators.janaf import Janaf
    from thermo_knowledge.acquire.enumerators.url_list import UrlList

    return {"iapws": Iapws(), "janaf": Janaf(), "url_list": UrlList()}


ENUMERATORS: dict[str, Enumerator] = _registry()
