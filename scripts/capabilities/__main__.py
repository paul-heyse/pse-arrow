# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Fixed STDIO server entry point; no arbitrary command or launch-time setup."""

from scripts.capabilities.server import build

if __name__ == "__main__":
    build().server.run(transport="stdio", show_banner=False)
