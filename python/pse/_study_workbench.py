# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded study browsing and passive reconnectable observation."""

from __future__ import annotations

import time
from typing import TYPE_CHECKING, Self

import attrs

from pse import codec
from pse._build import (
    _NativeFrozenStudyCohort,
    _NativeRuntime,
    _NativeStudyObserver,
)
from pse._inspection import TableStream

if TYPE_CHECKING:
    from collections.abc import Iterator
    from types import TracebackType

    from pse._runs import RunHandle
    from pse.contracts.documents import (
        StudyCohortHeaderPage,
        StudyCohortMemberPage,
        StudyCohortPageRequest,
        StudyCohortReceipt,
        StudyCohortRequest,
        StudyObservationCursor,
        StudyObservationCursorOutput,
        StudyObservationPageOutput,
        StudyObservationRequest,
    )


@attrs.frozen
class FrozenStudyCohort:
    """One complete native snapshot with lazy original protected detail."""

    _handle: _NativeFrozenStudyCohort

    @property
    def receipt(self) -> StudyCohortReceipt:
        return self._handle.receipt

    def header_page(self, request: StudyCohortPageRequest) -> StudyCohortHeaderPage:
        return self._handle.header_page(codec.encode_json(request))

    def member_page(self, request: StudyCohortPageRequest) -> StudyCohortMemberPage:
        return self._handle.member_page(codec.encode_json(request))

    def result(
        self, member: int, relation: str, *, start: int, end: int
    ) -> TableStream:
        return TableStream(self._handle.result(member, relation, start=start, end=end))

@attrs.frozen
class StudyObserver:
    """Observation owns no scientific cancellation authority."""

    _handle: _NativeStudyObserver

    def poll(self) -> StudyObservationPageOutput:
        return self._handle.poll()

    @property
    def cursor(self) -> StudyObservationCursorOutput:
        return self._handle.cursor()

    def attach_run(self, run: RunHandle) -> None:
        """Attach original local telemetry after a poll discovers its exact attempt."""
        self._handle.attach_run(run._handle)  # noqa: SLF001 -- package-native facade composition

    def follow(self, *, interval: float = 0.1) -> Iterator[StudyObservationPageOutput]:
        """Poll selected owners; stopping iteration never cancels the study."""
        if not 0 <= interval <= 60:
            message = "observer interval must be between zero and 60 seconds"
            raise ValueError(message)
        while True:
            page = self.poll()
            yield page
            if page.status.header.terminal and all(
                source.position.history_complete
                or source.continuity == "history_unavailable"
                for source in page.sources
            ):
                return
            time.sleep(interval)

    def close(self) -> None:
        self._handle.close()

    def __enter__(self) -> Self:
        return self

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        tb: TracebackType | None,
    ) -> None:
        self.close()


class StudyWorkbenchGlue:
    """Public runtime composition; the native owner performs selection and admission."""

    @property
    def _handle(self) -> _NativeRuntime:
        """Read-only native owner supplied by the concrete Runtime facade."""
        raise NotImplementedError

    def _host_freeze_study_cohort(
        self, request: StudyCohortRequest
    ) -> FrozenStudyCohort:
        return FrozenStudyCohort(
            self._handle.freeze_study_cohort(codec.encode_json(request))
        )

    def _host_observe_study(
        self,
        request: StudyObservationRequest,
        *,
        cursor: StudyObservationCursor | StudyObservationCursorOutput | None,
    ) -> StudyObserver:
        return StudyObserver(
            self._handle.observe_study(
                codec.encode_json(request),
                cursor=None if cursor is None else codec.encode_json(cursor),
            )
        )
