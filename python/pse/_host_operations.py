# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive compiled host operation metadata from the original native owner."""
from pse._build import _host_operation_catalogue
from pse.contracts.documents import OperationCatalogue


def host_operation_catalogue() -> OperationCatalogue:
    """Return the compiled product operation catalogue without constructing a runtime."""
    return _host_operation_catalogue()
