# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
#
# Type stub for the compiled extension module `pse._native`.
#
# Hand-written, deliberately: a generated stub would be one more artifact to keep
# in sync, and the surface is small (plan §5, O7). `test_native_stub_surface`
# diffs the names declared here against `dir(pse._native)` at runtime, so the
# stub cannot drift without a red test.
#
# `build_info()` returns the raw mapping (strings plus the two embedded lockfiles as
# bytes); `pse._build.build_info()` is the typed
# form every other caller uses.

__version__: str

def build_info() -> dict[str, str | bytes]: ...
