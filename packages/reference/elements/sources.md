<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Elements, version 1.0.0

The 21 declared engineering elements use the
[CIAAW abridged standard atomic weights, 2024](https://www.ciaaw.org/abridged-atomic-weights.htm),
converted from g/mol to kg/mol. They describe normal terrestrial materials; they
are neither isotope-specific masses nor a complete periodic table.

This separate package namespace permits potassium's symbol `K` and the physical
unit kelvin's symbol `K` to retain distinct admitted entity names and identities.
Both the ordinary loader and fixture generator enforce actual package closure.

The independent molecular-weight controls use the source uncertainties for carbon
(0.002 g/mol) and hydrogen (0.0002 g/mol), propagated conservatively through the
declared elemental counts. Altering an admitted atomic mass changes molecular
weight even when all stored IDs are retained.
