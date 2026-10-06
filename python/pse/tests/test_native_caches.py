# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native buffer ownership and process budget refusal at the Arrow boundary."""

import gc
from pathlib import Path

import pyarrow as pa
import pytest

import pse


@pytest.mark.component
def test_arrow_buffers_remain_owned_after_stream_close(
    inspection_settings: pse.EngineSettings,
) -> None:
    retained = None
    expected = None
    for _ in range(2):
        with (
            pse.registry_table(
                "reference.schema_relations", settings=inspection_settings
            ) as stream,
            pa.RecordBatchReader.from_stream(stream) as reader,
        ):
            batches = list(reader)
            retained = batches[0].column("name").slice(0, 1)
            expected = retained.to_pylist()
            batches.clear()
    assert retained is not None
    gc.collect()
    assert retained.to_pylist() == expected


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
            cache=pse.CacheSettings(working_bytes=512, metadata_bytes=1024),
        )


@pytest.mark.component
def test_registry_stream_cancellation_and_unknown_relation_are_observable(
    inspection_settings: pse.EngineSettings,
) -> None:
    with pytest.raises(pse.InspectionError, match="unknown declared"):
        pse.registry_table("reference.absent", settings=inspection_settings)
    stream = pse.registry_table(
        "reference.schema_relations", settings=inspection_settings
    )
    stream.cancel()
    with pytest.raises(
        (pse.InspectionError, pa.ArrowInvalid, pa.ArrowIOError), match="cancel"
    ):
        pa.RecordBatchReader.from_stream(stream)
    stream.close()
