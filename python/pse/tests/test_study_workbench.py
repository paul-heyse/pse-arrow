# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public bounded workbench and passive observer composition on two declared points."""

import msgspec
import pytest

import pse
from pse.contracts.documents import (
    StudyCohortRequest,
    StudyCohortPageRequest,
    StudyObservationRequest,
    StudyWorkbenchFilter,
    StudyWorkbenchRequest,
)
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.study_fixtures import point
from pse.tests.study_fixtures import request as study_request
from pse.tests.test_studies import _package


@pytest.mark.component
def test_study_workbench_public_snapshot_and_passive_reconnect(
    inspection_settings: pse.EngineSettings,
    canonical_substrate: CanonicalFixture,
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    # Constant declared science keeps the public journey bounded and independent
    # of nonlinear solver convergence or a production numerical campaign.
    package, case = _package(
        runtime,
        source="""package algebraic { def Root {
        param x:Scalar = 2; annotation report x("constant");
    } }""",
    )
    local = package.admit_study(
        study_request(
            point(case, pse.SolveSettings(), 7),
            point(case, pse.SolveSettings(), 11),
        )
    )
    handle = runtime.start_study(runtime.record_study(local))
    selected = StudyWorkbenchFilter(
        study=handle.study_id.to_hex(),
        project=None,
        branch=None,
        source=None,
        physical_revision=None,
        context=None,
        outcome=None,
        research_reference=None,
    )
    browsing = runtime.study_workbench_page(
        StudyWorkbenchRequest(
            filter=selected,
            maximum=1,
            cursor=None,
        )
    )
    assert len(browsing.headers) == 1
    assert browsing.headers[0].point_count == 2
    assert browsing.next is None
    frozen = runtime.freeze_study_cohort(
        StudyCohortRequest(
            filter=selected,
            maximum_studies=1,
            maximum_members=2,
        )
    )
    assert frozen.receipt.complete
    assert frozen.receipt.member_count == 2
    assert [row.value.ordinal for row in frozen.member_page(StudyCohortPageRequest(maximum=2,cursor=None)).members] == [0, 1]
    request = StudyObservationRequest(
        study=handle.study_id.to_hex(),
        maximum_members=2,
        maximum_events=8,
    )
    with runtime.observe_study(request) as observer:
        first = observer.poll()
        cursor = observer.cursor
        assert first.status.header.study == browsing.headers[0].study
    # Closing an observer cannot issue scientific cancellation, even if workers
    # happened to finish these two small constant operations before this read.
    assert not handle.status().cancelled
    with runtime.observe_study(request, cursor=cursor) as reconnected:
        current = reconnected.poll()
        assert not current.status.header.cancelled
        assert len(current.status.members.members) == 2
    foreign = msgspec.structs.replace(
        cursor, issuing_store=cursor.issuing_store + "-foreign"
    )
    with pytest.raises(Exception, match="cursor issuer or exact selection differs"):
        runtime.observe_study(request, cursor=foreign)
    wrong_context = msgspec.structs.replace(
        cursor.context, source=cursor.context.source + "-other"
    )
    wrong = msgspec.structs.replace(cursor, context=wrong_context)
    with pytest.raises(Exception, match="cursor issuer or exact selection differs"):
        runtime.observe_study(request, cursor=wrong)
    with pytest.raises(Exception, match="narrow"):
        runtime.freeze_study_cohort(
            StudyCohortRequest(
                filter=selected,
                maximum_studies=1,
                maximum_members=1,
            )
        )
    # A frozen population remains exactly the captured population after later polls.
    assert frozen.receipt.member_count == 2
    assert not handle.status().cancelled
    handle.cancel()
