#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.14.*"
# dependencies = ["packaging==25.0"]
# [tool.uv]
# exclude-newer = "2026-10-01T00:00:00Z"
# ///
import json, sys, packaging
print(json.dumps({"python": sys.version.split()[0], "packaging": packaging.__version__}))
