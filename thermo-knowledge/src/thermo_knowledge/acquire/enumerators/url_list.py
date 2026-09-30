# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Enumerator `url_list`: the URLs are listed in the manifest's `urls` key."""

from __future__ import annotations

from collections.abc import Callable, Iterator
from typing import TYPE_CHECKING

from thermo_knowledge.acquire.enumerators import PageTarget, stable_name
from thermo_knowledge.acquire.errors import AcquireError

if TYPE_CHECKING:
    from thermo_knowledge.acquire.manifest import PagesSpec


class UrlList:
    def enumerate(self, spec: PagesSpec, fetch_text: Callable[[str], str]) -> Iterator[PageTarget]:
        if not spec.urls:
            raise AcquireError("enumerator url_list: the `urls` key of [acquire] is empty")
        for url in spec.urls:
            yield PageTarget(url, stable_name(url))
