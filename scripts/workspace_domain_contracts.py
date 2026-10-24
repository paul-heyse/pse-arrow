# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owner-defined exact concept requests, independent of presentation frameworks."""

from __future__ import annotations

from typing import Annotated, ClassVar, Literal, get_args

from pydantic import BaseModel, ConfigDict, Field, StringConstraints, TypeAdapter

ExactText = Annotated[
    str, StringConstraints(min_length=1, max_length=4096, pattern=r"\S")
]


class QueryBounds(BaseModel):
    model_config = ConfigDict(extra="forbid", strict=True, frozen=True)
    native_kind: ClassVar[str]
    limit: Annotated[int, Field(ge=1, le=100)] = 20
    offset: Annotated[int, Field(ge=0)] = 0


class MetadataQuery(QueryBounds):
    """Conjunctive case-sensitive facets, never lexical or similarity matches."""

    owner: ExactText | None = None
    role: ExactText | None = None
    topics: Annotated[list[ExactText], Field(max_length=16)] = Field(
        default_factory=list
    )
    document: ExactText | None = None
    publication_scope: Literal["Current", "Reference", "History"] | None = None
    native_status: ExactText | None = None


class DocumentQuery(MetadataQuery):
    native_kind = "document"
    concept: Literal["document"]
    identity: ExactText | None = Field(
        default=None,
        description="Exact repository document path; omit for faceted discovery.",
        examples=["docs/README.md"],
    )


class DecisionQuery(MetadataQuery):
    native_kind = "decision"
    concept: Literal["decision"]
    identity: ExactText | None = Field(
        default=None,
        description="Authored ADR identity, independent of filename; omit for faceted discovery.",
        examples=["ADR-0101"],
    )


class ContractSectionQuery(MetadataQuery):
    native_kind = "contract"
    concept: Literal["contract-section"]
    identity: ExactText = Field(
        description="Exact stable section ID.", examples=["20.5"]
    )


class PlanPackageQuery(MetadataQuery):
    native_kind = "plan_package"
    concept: Literal["plan-package"]
    identity: ExactText = Field(
        description="Native ScopeItem ID or document#package, including the selected document.",
        examples=[
            "docs/plans/52-agent-effectiveness-and-engineering-workflows.md#CON01"
        ],
    )


class DeclarationQuery(MetadataQuery):
    native_kind = "declaration"
    concept: Literal["declaration"]
    identity: ExactText = Field(
        description="Exact declaration key, optionally qualified by declaration kind.",
        examples=["relation:alpha@1"],
    )


class GeneratorQuery(MetadataQuery):
    native_kind = "generator"
    concept: Literal["generator"]
    identity: ExactText = Field(
        description="Exact generator path, optionally #symbol.",
        examples=["src/generator.rs#emit"],
    )


class GeneratedOutputQuery(MetadataQuery):
    native_kind = "generated_artifact"
    concept: Literal["generated-output"]
    identity: ExactText = Field(
        description="Exact generated output path.", examples=["src/generated/alpha.rs"]
    )


class OriginalReference(BaseModel):
    """Explicit original-owner reference; never scan arbitrary historical payloads."""

    model_config = ConfigDict(extra="forbid", strict=True, frozen=True)
    adapter: Literal["invocation", "diagnostic", "assessment"]
    path: ExactText = Field(
        description="Explicit original evidence path or issued handle."
    )
    codec: ExactText | None = None
    member: ExactText | None = None


class DiagnosticReference(OriginalReference):
    adapter: Literal["diagnostic"]


class EvidenceReferenceQuery(QueryBounds):
    native_kind = "evidence_reference"
    concept: Literal["evidence-reference"]
    selected_reference: OriginalReference


class DiagnosticQuery(QueryBounds):
    native_kind = "diagnostic"
    concept: Literal["diagnostic"]
    selected_reference: DiagnosticReference


class ExperimentQuery(QueryBounds):
    """Constructible explicit selection; experiment adapter remains unimplemented."""

    native_kind = "experiment"
    concept: Literal["experiment"]
    selected_reference: OriginalReference


class ResearchQuestionQuery(QueryBounds):
    native_kind = "research_question"
    concept: Literal["research-question"]
    identity: ExactText = Field(
        description="Caller-owned exact question identity.", examples=["RQ-7"]
    )
    origin: Literal["user-question", "agent-interpretation"]
    question: ExactText = Field(
        description="Explicit question or interpretation text.",
        examples=["Which result contract owns this field?"],
    )


ConceptQuery = Annotated[
    DocumentQuery
    | ContractSectionQuery
    | DecisionQuery
    | PlanPackageQuery
    | DeclarationQuery
    | GeneratorQuery
    | GeneratedOutputQuery
    | EvidenceReferenceQuery
    | DiagnosticQuery
    | ExperimentQuery
    | ResearchQuestionQuery,
    Field(discriminator="concept"),
]
QUERY_ADAPTER = TypeAdapter(ConceptQuery)
QUERY_TYPES = get_args(get_args(ConceptQuery)[0])
CONCEPTS = {
    get_args(model.model_fields["concept"].annotation)[0]: model.native_kind
    for model in QUERY_TYPES
}
SOURCE_CONCEPTS = tuple(
    concept
    for concept, model in zip(CONCEPTS, QUERY_TYPES, strict=True)
    if issubclass(model, MetadataQuery)
)


def parse_query(value: object) -> ConceptQuery:
    """Validate native CLI/JSON input using the same declaration exposed to agents."""
    return QUERY_ADAPTER.validate_python(value)
