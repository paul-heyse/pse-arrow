# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The public passive inspector reads exact source metadata without preparing it."""

from pathlib import Path

import pytest

import pse
from pse import codec
from pse.contracts.documents import PreparationDiscoveryRequest
from pse.tests.canonical_fixture import CanonicalFixture
from pse.tests.test_modeling_kernel import on_primitives, physical


@pytest.mark.component
def test_project_preparation_discovery_is_passive_and_reports_unchecked_presence(
    inspection_settings: pse.EngineSettings, canonical_substrate: CanonicalFixture
) -> None:
    runtime = canonical_substrate.runtime(inspection_settings)
    root = Path(__file__).resolve().parents[3]
    manifest = on_primitives(
        (root / "tests/fixtures/packages/minimal_explicit/package.toml")
        .read_text()
        .replace('id_policy = "explicit"', 'id_policy = "named"')
    )
    context = physical(runtime)
    source = runtime.modeling_from_documents(
        [
            {
                "package.toml": manifest,
                "models/model.pse": "package p {def Root {var x:Scalar;eq e:x*x==1;}}",
            }
        ],
        context,
    )
    runtime.project_create(
        "create-discovery", "discovery-project", "Preparation inspection"
    )
    revision = runtime.project_fork(
        "discovery-project",
        "discovery-main",
        "discovery-problem",
        "fork-discovery",
        source,
    )
    selected = runtime.modeling_revision(revision, context)
    declaration = next(
        row.declaration_id for row in selected.declarations() if row.name == "Root"
    )
    # Exercise the generated strict document's real identity and collection codecs.
    request = codec.decode_json(
        (
            '{"document_version":1,"root":"'
            + declaration.to_hex()
            + '","instance":"'
            + declaration.to_hex()
            + '","bindings":{"arguments":{},"scope":{},"facts":[],"demand":[],'
            '"formulation":{"omitted":[],"elastic":{}}},'
            '"limits":{"depth":64,"items":100000,"members":100000,'
            '"body_occurrences":null,"body_slots":null},"durable_after":null}'
        ).encode(),
        PreparationDiscoveryRequest,
    )
    before = next(
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.modeling_products"
    )
    report = selected.preparation_discovery(request)
    after = next(
        cache
        for cache in runtime.resource_usage().caches
        if cache.name == "pse.cache.modeling_products"
    )
    assert report.revision_id == selected.revision_id
    assert report.resident == ()
    assert not report.residents_truncated
    assert report.durable is not None
    assert report.durable.rows == ()
    assert report.durable.next is None
    assert report.durable_eligibility == "unchecked"
    assert (before.hits, before.misses, before.entries, before.retained_bytes) == (
        after.hits,
        after.misses,
        after.entries,
        after.retained_bytes,
    )
    runtime.clear_program_cache()
    cleared = selected.preparation_discovery(request)
    assert cleared.resident == ()
    assert cleared.durable_eligibility == "unchecked"
