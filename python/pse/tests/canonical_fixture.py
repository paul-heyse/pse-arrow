# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit per-test mutable database and local runtime borrower ownership."""

from __future__ import annotations

import attrs

import pse


@attrs.frozen
class CanonicalFixture:
    state: str
    database: str
    resource: str
    borrowers: list[pse.Runtime] = attrs.field(factory=list, repr=False)

    def runtime(
        self,
        settings: pse.EngineSettings,
        *,
        producer: str | None = None,
        ephemeral: bool = False,
    ) -> pse.Runtime:
        runtime = pse.Runtime(
            settings,
            substrate=self.state,
            database=self.database,
            producer=producer,
            ephemeral=ephemeral,
        )
        self.borrowers.append(runtime)
        return runtime

    def drain(self) -> None:
        # Optional caches may retain another borrower's source protection. Evict
        # all of them before disconnecting; live external aliases still refuse.
        for runtime in self.borrowers:
            runtime.clear_program_cache()
        for runtime in self.borrowers:
            runtime.close()
        self.borrowers.clear()
