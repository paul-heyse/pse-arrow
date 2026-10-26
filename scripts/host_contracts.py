# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-only strict host record base, independent of capability composition."""

from pydantic import BaseModel, ConfigDict


class Contract(BaseModel):
    """Closed immutable host input/output records with validated defaults."""

    model_config = ConfigDict(
        extra="forbid", strict=True, frozen=True, validate_default=True
    )
