<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Thermodynamic domain, version 1.0.0

The modeling documents are the declaration authority. The `chemistry` package declares
the phase kind and the canonical `liquid` and `vapor` phases, which every library
names instead of declaring its own. This package carries no data: data banks and
property packages add rows against it (ADR-0126).
