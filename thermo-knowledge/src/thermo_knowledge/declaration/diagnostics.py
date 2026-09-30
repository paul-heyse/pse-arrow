# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Structured refusals of the declaration loader (meta-model section 6)."""

from __future__ import annotations

from collections.abc import Iterable
from enum import StrEnum

import msgspec


class Code(StrEnum):
    """Stable diagnostic codes. A code is never reused for another meaning."""

    # Documents and structure.
    TOML_SYNTAX = "toml-syntax"
    MISSING_MANIFEST = "missing-manifest"
    UNKNOWN_KEY = "unknown-key"
    MISSING_DOC = "missing-doc"
    MISSING_FIELD = "missing-field"
    INVALID_VALUE = "invalid-value"
    MISSING_ENFORCEMENT = "missing-enforcement"
    UNKNOWN_CHECK = "unknown-check"
    BAD_PSE_MARK = "bad-pse-mark"
    # Modules and names.
    DUPLICATE_MODULE = "duplicate-module"
    MISSING_SCHEMA = "missing-schema"
    BAD_SCHEMA = "bad-schema"
    DUPLICATE_NAME = "duplicate-name"
    BAD_NAME = "bad-name"
    UNKNOWN_NAME = "unknown-name"
    UNREACHABLE_NAME = "unreachable-name"
    USES_CYCLE = "uses-cycle"
    EXTENDS_CYCLE = "extends-cycle"
    CONTRACT_CYCLE = "contract-cycle"
    # Types and units.
    BAD_TYPE = "bad-type"
    BAD_UNIT = "bad-unit"
    UNIT_NOT_COHERENT = "unit-not-coherent"
    BAD_QUANTITY_EXPRESSION = "bad-quantity-expression"
    # Kinds and relations.
    MISSING_IDENTITY = "missing-identity"
    REFINEMENT_IDENTITY = "refinement-identity"
    BAD_IDENTITY_ATTRIBUTE = "bad-identity-attribute"
    BAD_PROVENANCE = "bad-provenance"
    BAD_ATTRIBUTE = "bad-attribute"
    MISSING_UNIT_FROM = "missing-unit-from"
    BAD_UNIT_FROM = "bad-unit-from"
    BAD_DEFAULT = "bad-default"
    BAD_CHECK = "bad-check"
    BAD_UNIQUE = "bad-unique"
    BAD_RELATION = "bad-relation"
    BAD_ABSENCE = "bad-absence"
    BAD_TRANSPOSITION = "bad-transposition"
    TRANSPOSITION_KINDS = "transposition-kinds"
    # Forms.
    FRAMEWORK_ROLE = "framework-role"
    SUBJECT_NOT_KIND = "subject-not-kind"
    SLOT_SHAPE = "slot-shape"
    BAD_FAMILY = "bad-family"
    BAD_BINDING = "bad-binding"
    BAD_OUTPUT_OBSERVABLE = "bad-output-observable"
    # Expressions (expressions.md section 4).
    EXPRESSION_SYNTAX = "expression-syntax"
    EXPRESSION_GRAMMAR = "expression-grammar"
    UNKNOWN_FUNCTION = "unknown-function"
    BAD_CALL = "bad-call"
    SLOT_SUBJECTS = "slot-subjects"
    INDEX_SCOPE = "index-scope"
    NOT_INDEX_SET = "not-index-set"
    DIMENSION_MISMATCH = "dimension-mismatch"
    DIMENSIONLESS_REQUIRED = "dimensionless-required"
    BASIS_MISMATCH = "basis-mismatch"
    BAD_POWER = "bad-power"
    BAD_DERIVATIVE = "bad-derivative"
    OUTPUT_DIMENSION = "output-dimension"
    MISSING_OUTPUT = "missing-output"
    UNKNOWN_OUTPUT = "unknown-output"
    LOCAL_ORDER = "local-order"
    BAD_IMPLICIT = "bad-implicit"
    RESIDUAL_COUNT = "residual-count"
    # Entities.
    ABSTRACT_ENTITY = "abstract-entity"
    BAD_ENTITY = "bad-entity"
    BAD_ENUM_MEMBER = "bad-enum-member"
    # The pipeline's shape contract (meta-model section 5).
    PIPELINE_CONTRACT = "pipeline-contract"
    # Projection.
    IDENTIFIER_TOO_LONG = "identifier-too-long"
    IDENTIFIER_COLLISION = "identifier-collision"


class Diagnostic(msgspec.Struct, frozen=True, kw_only=True, order=True):
    """One refusal: a stable code, where it was found and what is wrong.

    A refusal inside expression text also carries its position there: `line` and `column`,
    both counted from 1 (0 means the refusal has no position in text).
    """

    document: str
    construct: str
    code: str
    message: str
    line: int = 0
    column: int = 0

    def __str__(self) -> str:
        where = f"{self.document}: {self.construct}" if self.construct else self.document
        at = f" (line {self.line}, column {self.column})" if self.line else ""
        return f"{where}: [{self.code}] {self.message}{at}"


class DeclarationError(Exception):
    """A declaration was refused; `diagnostics` lists every refusal found."""

    def __init__(self, diagnostics: Iterable[Diagnostic]) -> None:
        self.diagnostics: tuple[Diagnostic, ...] = tuple(diagnostics)
        super().__init__("\n".join(str(item) for item in self.diagnostics))
