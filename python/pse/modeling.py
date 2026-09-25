# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public aliases of the registry-generated native model declaration contracts."""

from pse._workflow import (
    CaseDeclaration,
    Contribution,
    Definition,
    Domain,
    Formal,
    Group,
    GroupSlot,
    Instance,
    Literal,
    ModelDeclaration,
    Objective,
    Parameter,
    Port,
    Row,
    Slot,
    Unit,
    Value,
    Variable,
)

__all__ = [
    "CaseDeclaration",
    "Contribution",
    "Definition",
    "Domain",
    "DynamicDeclaration",
    "FitDeclaration",
    "Formal",
    "Group",
    "GroupSlot",
    "Instance",
    "Literal",
    "ModelDeclaration",
    "NativeProviderDeclaration",
    "Objective",
    "Parameter",
    "Port",
    "Row",
    "Slot",
    "Unit",
    "Value",
    "Variable",
]

from pse.contracts.authored import (
    AuthoredDynamicCasesRow as DynamicDeclaration,
)
from pse.contracts.authored import (
    AuthoredFitCasesRow as FitDeclaration,
)
from pse.contracts.authored import (
    AuthoredNativeProvidersRow as NativeProviderDeclaration,
)
