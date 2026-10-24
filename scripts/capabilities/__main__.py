# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed STDIO entry; telemetry belongs to the executable process host."""

import os

from scripts.telemetry_process import bootstrap

if __name__ == "__main__":
    os.environ["FASTMCP_TELEMETRY_MODE"] = "native"
    with bootstrap("pse-capabilities"):
        from scripts.capabilities.server import build

        build().server.run(transport="stdio", show_banner=False)
