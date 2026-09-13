# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The numpy boundary refuses nulls and stays lazy (blueprint §21.6, §24.1)."""

import math

import numpy as np
import pyarrow as pa
import pytest

from pse._array import NullableColumnError, to_ndarray


@pytest.mark.unit
def test_dense_column_converts_zero_copy() -> None:
    column = pa.array([1.0, 2.0, 3.0], type=pa.float64())
    assert to_ndarray(column).tolist() == [1.0, 2.0, 3.0]


@pytest.mark.unit
def test_nullable_column_is_refused() -> None:
    column = pa.array([1.0, None, 3.0], type=pa.float64())
    with pytest.raises(NullableColumnError) as caught:
        to_ndarray(column)
    assert caught.value.null_count == 1
    assert "NaN" in str(caught.value)


@pytest.mark.unit
def test_copy_requires_a_reason() -> None:
    column = pa.chunked_array([[1.0, 2.0], [3.0]], type=pa.float64())
    # A chunked column cannot be converted without a copy at all, so the
    # default zero-copy request is refused rather than silently honoured.
    with pytest.raises(ValueError, match="zero_copy_only"):
        to_ndarray(column)
    copied = to_ndarray(column, copy_reason="chunked solver vector, §24.1 oracle")
    assert copied.tolist() == [1.0, 2.0, 3.0]


@pytest.mark.unit
def test_single_chunk_column_still_needs_no_copy() -> None:
    column = pa.chunked_array([[1.0, 2.0, 3.0]], type=pa.float64())
    with pytest.raises(ValueError, match="zero_copy_only"):
        to_ndarray(column)


@pytest.mark.unit
def test_negative_zero_survives() -> None:
    column = pa.array([-0.0, 0.0], type=pa.float64())
    converted = to_ndarray(column)
    assert math.copysign(1.0, float(converted[0])) == -1.0
    assert math.copysign(1.0, float(converted[1])) == 1.0
    assert np.signbit(converted).tolist() == [True, False]


@pytest.mark.unit
def test_import_pse_does_not_import_the_scientific_stack(
    no_numpy_on_import: frozenset[str],
) -> None:
    assert no_numpy_on_import == frozenset()
