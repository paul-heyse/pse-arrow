# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The survey index: the loader for the source survey records (`survey/*.toml`), the
disposition format and the residue report (`docs/survey.md`)."""

from __future__ import annotations

from thermo_knowledge.survey_index.diagnostics import Code, SurveyDiagnostic
from thermo_knowledge.survey_index.dispositions import (
    Disposition,
    DispositionSet,
    RefIndex,
    load_dispositions,
    needs_disposition,
)
from thermo_knowledge.survey_index.loader import SurveySet, load_surveys
from thermo_knowledge.survey_index.report import SourceResidue, render_report, residue

__all__ = [
    "Code",
    "Disposition",
    "DispositionSet",
    "RefIndex",
    "SourceResidue",
    "SurveyDiagnostic",
    "SurveySet",
    "load_dispositions",
    "load_surveys",
    "needs_disposition",
    "render_report",
    "residue",
]
