# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Arrow transport protocols for registry-governed physical inputs."""

from typing import Protocol, TypeAlias


class ArrowArrayInput(Protocol):
    """An Arrow C Data producer, including a record batch."""

    def __arrow_c_array__(
        self, requested_schema: object = None
    ) -> tuple[object, object]:
        """Export a schema and array capsule."""
        ...


class ArrowStreamInput(Protocol):
    """An Arrow C Stream producer, including a table or batch reader."""

    def __arrow_c_stream__(self, requested_schema: object = None) -> object:
        """Export an Arrow stream capsule."""
        ...


ArrowInput: TypeAlias = ArrowArrayInput | ArrowStreamInput


class ArrowSchema(Protocol):
    """A complete schema imported through Arrow's C Schema interface."""

    def __arrow_c_schema__(self) -> object:
        """Export the registered schema capsule, including metadata."""
        ...
