# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""The numpy boundary (blueprint §21.6).

This is the only module outside the parity harness and the tests that is allowed
to import numpy, and it imports it lazily inside :func:`to_ndarray` so that
``import pse`` never drags numpy into ``sys.modules``. The import-linter contract
"numpy/scipy only at the array boundary" and the ast-grep rule
``to-numpy-only-in-array-boundary`` both name this module.

A nullable Arrow column converted to a bare ``ndarray`` turns null into NaN,
collapsing the distinction the extension types exist to protect, so the null
count is asserted before any conversion. Conversion is zero-copy by default:
a copy has to be asked for by naming the reason at the call site.
"""

from typing import TYPE_CHECKING

import pyarrow as pa

if TYPE_CHECKING:
    import numpy as np


class NullableColumnError(ValueError):
    """A column carrying nulls was asked to become a bare ``ndarray``.

    numpy has no null; the conversion would silently turn every null into NaN.
    Carry the validity mask alongside, or filter the column first.
    """

    def __init__(self, null_count: int) -> None:
        """Build the error from the offending column's null count.

        Args:
            null_count: How many nulls the column carries.
        """
        super().__init__(
            f"refusing to convert a column with {null_count} null(s) to ndarray: "
            "numpy has no null and would encode it as NaN (blueprint §21.6). "
            "Filter the column or carry the validity mask alongside."
        )
        self.null_count = null_count


def to_ndarray(
    column: pa.Array | pa.ChunkedArray,
    *,
    copy_reason: str | None = None,
) -> "np.ndarray[tuple[int, ...], np.dtype[np.generic]]":
    """Convert an Arrow column to a numpy array across the §21.6 boundary.

    Args:
        column: The Arrow array or chunked array to convert. It must carry no
            nulls.
        copy_reason: Why a copy is acceptable here. ``None`` (the default)
            requests a zero-copy conversion, which pyarrow refuses rather than
            silently copying. Passing a reason permits the copy and records why.

    Returns:
        The converted numpy array. ``-0.0`` survives the conversion: neither
        pyarrow nor numpy normalises the sign of zero.

    Raises:
        NullableColumnError: If the column carries one or more nulls.
        ValueError: If a zero-copy conversion was requested but pyarrow cannot
            provide one -- a chunked column, or a type whose Arrow layout does
            not map onto a numpy buffer. Name a ``copy_reason`` to allow it.
    """
    null_count = column.null_count
    if null_count != 0:
        raise NullableColumnError(null_count)
    import numpy as np  # noqa: PLC0415

    return np.asarray(column.to_numpy(zero_copy_only=copy_reason is None))
