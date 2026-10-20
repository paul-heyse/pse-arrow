# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated optional-tool entry: apply finite regular-file limits before exec."""

import os
import resource
import sys

if __name__ == "__main__":
    allowance = int(sys.argv[1])
    resource.setrlimit(resource.RLIMIT_FSIZE, (allowance, allowance))
    os.execv(sys.argv[2], sys.argv[2:])  # noqa: S606 -- exact tool and literal argv.
