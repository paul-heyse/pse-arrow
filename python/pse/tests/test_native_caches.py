# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Final-phase native residency and independent Python/Arrow ownership."""

import gc
from pathlib import Path

import pyarrow as pa
import pytest

import pse
from pse.tests.test_native_registry import publication_index


@pytest.mark.component
def test_resident_reads_keep_exported_buffers_after_publication_close(
    native_inspection_publication: Path, inspection_settings: pse.EngineSettings
) -> None:
    index = publication_index(native_inspection_publication)
    publication = pse.open(
        index.root.location, version=index.root.version, settings=inspection_settings
    )
    for _ in range(2):
        with (
            publication.table("artifact", "reference", "schema_relations") as stream,
            pa.RecordBatchReader.from_stream(stream) as reader,
        ):
            batches = list(reader)
            retained = batches[0].column("name").slice(0, 1)
            expected = retained.to_pylist()
            batches.clear()
    resident = next(
        row for row in publication.cache_usage() if row.name == "pse.cache.resident"
    )
    assert resident.hits > 0
    assert resident.pinned_bytes is not None
    assert resident.pinned_bytes > 0
    publication.close()
    gc.collect()
    assert retained.to_pylist() == expected
    del retained
    gc.collect()
    resident = next(
        row for row in publication.cache_usage() if row.name == "pse.cache.resident"
    )
    assert resident.pinned_bytes == 0
    assert resident.live_bytes is not None
    assert resident.live_bytes > 0


@pytest.mark.unit
def test_cache_settings_refuse_overflow_and_missing_working_headroom(
    tmp_path: Path,
) -> None:
    with pytest.raises(pse.InspectionError):
        pse.CacheSettings(working_bytes=0)
    with pytest.raises(pse.InspectionError):
        pse.EngineSettings(
            memory_limit_bytes=1024,
            batch_size=7,
            threads=1,
            spill_dir=str(tmp_path),
            max_spill_bytes=1024,
            cache=pse.CacheSettings(working_bytes=512, resident_bytes=1024),
        )
