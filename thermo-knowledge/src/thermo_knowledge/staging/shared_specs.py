# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Tables that the chemicals and thermo payloads both ship, byte for byte.

The regulatory inventories under `Law/` and the Bell 2018 table under `Phase Change/` sit in both
repositories with identical content, so one declaration serves both readers (each reader keeps its
own staged copy and cites its own file). Paths are relative to the payload's data directory
(`chemicals/` or `thermo/`).

The inventories carry no thermodynamic content; they are staged because the payload contains them.
Two of them state no heading, so their first line is a data row (the source library reads them with
pandas, which takes that row as a heading); the others state one. Compressed files are decompressed
and the zip archive's one member is read; `_locator` counts lines of the decompressed text.
"""

from __future__ import annotations

from thermo_knowledge.staging.tabular import Delimited, delimited

_TSCA_FLAGS = "UV;E;F;N;P;S;R;T;XU;SP;TP;Y1;Y2"

SHARED_TABLES: tuple[Delimited, ...] = (
    delimited(
        "law_canada_dsl",
        "Law/Canada Feb 11 2015 - DSL.csv.gz",
        "CASRN:c;Registry:i",
    ),
    delimited(
        "law_ec_inventory_nlp",
        "Law/EC Inventory No Longer Polymers (NLP).csv",
        "CASRN:c",
        headings=False,
    ),
    delimited(
        "law_echa_tonnage_bands",
        "Law/ECHA Tonnage Bands.csv.zip",
        "CASRN;Tonnage band",
        headings=False,
    ),
    delimited(
        "law_einecs",
        "Law/EINECS 2015-03.csv.gz",
        "CASRN:c",
    ),
    delimited(
        "law_epa_cdr_2012",
        "Law/EPA 2012 Chemical Data Reporting.csv",
        "CASRN;DOM_MFG_LB:f;IMPORTED_LB:f;VOLUME_EXPORTED:f",
    ),
    delimited(
        "law_epa_cdr_2012_gz",
        "Law/EPA Chemical Data Reporting - 2012.csv.gz",
        "CASRN:c;Domestic:f;Imported:f;Exported:f",
    ),
    delimited(
        "law_hpv",
        "Law/HPV 2015 March 3.csv",
        "CASRN:c",
        headings=False,
    ),
    delimited(
        "law_spin",
        "Law/SPIN Inventory 2015-03.csv.gz",
        "CASRN:c",
    ),
    delimited(
        "law_tsca",
        "Law/TSCA Inventory 2016-01.csv.gz",
        "CASRN:c;" + ";".join(f"{name}:b" for name in _TSCA_FLAGS.split(";")),
    ),
    delimited(
        "phase_change_bell_2018",
        "Phase Change/Bell 2018 je7b00967_si_001.tsv",
        "Tc:f;Pc:f;c0:f;c1:f;c2:f;name;inchikey",
    ),
)
