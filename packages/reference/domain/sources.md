<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
<!-- Copyright (c) 2026 Paul Heyse -->

# Domain schema, version 1.0.0

This package declares the domain schema above `pse.physical`. It holds no rows: packages
that cite a reference declare its source entity, and data banks declare the rows.

`models/provenance.pse` declares the provenance kinds and the provenance roles. An entity
whose kind refines `source` may be named as the source of a dataset, a constant or a test.
`publication`, `software_release`, `release_artifact` (a located file of a release),
`oracle_test` and `computation` (a reference computed with named software releases) refine
it. Identifier values (`Id<doi>`, `Id<isbn>`) and locators are data; nothing parses them.

A role is a member of `Role`. The kernel acts only on the facets a member declares:
`synthetic` and `oracle_input` data are test-only, so a root outside a test fixture that
reads them is refused; `derived` and `fitted` data name their lineage. `published` and
`measured` declare no facet. A role describes how the data were obtained; it does not
certify them.
