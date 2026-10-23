# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded study browsing, exact frozen populations and passive reconnectable observation."""
from __future__ import annotations

from collections.abc import Iterator
from types import TracebackType
from typing import TYPE_CHECKING, Self

import attrs
import pyarrow as pa

from pse import codec
from pse._build import (
    _NativeFrozenStudyCohort,
    _NativeRuntime,
    _NativeStudyKpiAnalysis,
    _NativeStudyObserver,
)
from pse._inspection import TableStream
from pse.contracts.documents import (
    StudyCohortDocument,
    StudyCohortRequest,
    StudyKpiRequest,
    StudyObservationCursor,
    StudyObservationPage,
    StudyObservationRequest,
    StudyWorkbenchPage,
    StudyWorkbenchRequest,
)
if TYPE_CHECKING:
    from pse._runs import RunHandle


@attrs.frozen
class StudyKpiAnalysis:
    """Original population and selected Arrow aggregates; minimum is sample-local."""
    _handle: _NativeStudyKpiAnalysis

    @property
    def cohort(self) -> StudyCohortDocument:
        return self._handle.cohort

    @property
    def included(self) -> tuple[int, ...]:
        return tuple(self._handle.included)

    @property
    def unavailable(self) -> tuple[int, ...]:
        return tuple(self._handle.unavailable)

    def batches(self) -> tuple[pa.RecordBatch, ...]:
        """Return only the selected native aggregation, preserving actual quantity/unit columns."""
        return tuple(pa.record_batch(batch) for batch in self._handle.batches())


@attrs.frozen
class FrozenStudyCohort:
    """One complete native snapshot with lazy original protected detail."""
    _handle: _NativeFrozenStudyCohort

    @property
    def document(self) -> StudyCohortDocument:
        return self._handle.document

    def result(self, member: int, relation: str, *, start: int, end: int) -> TableStream:
        return TableStream(self._handle.result(member, relation, start=start, end=end))

    def aggregate(self, request: StudyKpiRequest) -> StudyKpiAnalysis:
        return StudyKpiAnalysis(self._handle.aggregate(codec.encode_json(request)))


@attrs.frozen
class StudyObserver:
    """Observation owns no scientific cancellation authority."""
    _handle: _NativeStudyObserver

    def poll(self) -> StudyObservationPage:
        return self._handle.poll()

    @property
    def cursor(self) -> StudyObservationCursor:
        return self._handle.cursor()

    def attach_run(self, run: RunHandle) -> None:
        """Attach original local telemetry after a poll discovers its exact attempt."""
        self._handle.attach_run(run._handle)

    def follow(self, *, interval: float = 0.1) -> Iterator[StudyObservationPage]:
        """Poll selected owners; stopping iteration never cancels the study."""
        if not 0 <= interval <= 60:
            raise ValueError("observer interval must be between zero and 60 seconds")
        import time
        while True:
            page = self.poll()
            yield page
            if page.status.headers[0].terminal and all(
                source.position.history_complete for source in page.sources
            ):
                return
            time.sleep(interval)

    def close(self) -> None:
        self._handle.close()

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type: type[BaseException] | None,
                 exc: BaseException | None, tb: TracebackType | None) -> None:
        self.close()


class StudyWorkbenchMixin:
    """Public runtime composition; the native owner performs selection and admission."""
    _handle: _NativeRuntime

    def study_workbench_page(self, request: StudyWorkbenchRequest) -> StudyWorkbenchPage:
        return self._handle.study_workbench_page(codec.encode_json(request))

    def freeze_study_cohort(self, request: StudyCohortRequest) -> FrozenStudyCohort:
        return FrozenStudyCohort(self._handle.freeze_study_cohort(codec.encode_json(request)))

    def observe_study(self, request: StudyObservationRequest, *,
                      cursor: StudyObservationCursor | None = None) -> StudyObserver:
        return StudyObserver(self._handle.observe_study(
            codec.encode_json(request), cursor=None if cursor is None else codec.encode_json(cursor)
        ))
