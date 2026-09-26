# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Typed declarations and owned handles over the single native computation pipeline."""

from collections.abc import Mapping, Sequence
from typing import Self, TypeAlias

import attrs
import msgspec

from pse import codec
from pse._build import (
    DiagnosticReport,
    EngineSettings,
    NativeAttempt,
    ProgressEvent,
    SimulationSettings,
    SolveSettings,
    _NativeModelRevision,
    _NativePhysicalContext,
    _NativePreparedCase,
    _NativePreparedFlow,
    _NativePreparedOperation,
    _NativePreparedStrategy,
    _NativePublicationAttempt,
    _NativeRunHandle,
    _NativeRunResult,
    _NativeRuntime,
    _NativeStart,
    _NativeStrategyResult,
)
from pse._inspection import TableStream
from pse.contracts import authored as a
from pse.contracts import runtime as result_contracts
from pse.contracts.values import ContentHash, SemanticId

SolverCapability = result_contracts.RuntimeSolverCapabilitiesRow

# Aliases expose generated contracts without creating a second declaration language.
ModelDeclaration = a.AuthoredComputationModelsRow
CaseDeclaration = a.AuthoredComputationModelsFieldCasesItem
Definition = a.AuthoredComputationModelsFieldDefinitionsItem
Formal = a.AuthoredComputationModelsFieldDefinitionsItemFormalsItem
Literal = a.AuthoredComputationModelsFieldDefinitionsItemLiteralsItem
Unit = a.AuthoredComputationModelsFieldDefinitionsItemUnitsItem
Domain = a.AuthoredComputationModelsFieldDomainsItem
Group = a.AuthoredComputationModelsFieldGroupsItem
GroupSlot = a.AuthoredComputationModelsFieldGroupsItemSlotsItem
Variable = a.AuthoredComputationModelsFieldCasesItemVariablesItem
Port = a.AuthoredComputationModelsFieldCasesItemVariablesItemPort
Parameter = a.AuthoredComputationModelsFieldCasesItemParametersItem
Instance = a.AuthoredComputationModelsFieldCasesItemInstancesItem
Slot = a.AuthoredComputationModelsFieldCasesItemInstancesItemSlotsItem
Contribution = a.AuthoredComputationModelsFieldCasesItemInstancesItemContributionsItem
Row = a.AuthoredComputationModelsFieldCasesItemRowsItem
Objective = a.AuthoredComputationModelsFieldCasesItemObjective
Value = a.AuthoredComputationModelsFieldCasesItemValuesItem


class _AnalysisDocument(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    """Transport envelope; Rust admits the typed analysis inside it."""

    payload: dict[str, object]


class _ModelEnvelope(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    declaration: dict[str, object]
    sources: dict[str, object] = msgspec.field(default_factory=dict)


def _encode(row: ModelDeclaration, sources: dict[str, object]) -> bytes:
    conv = codec.converter()
    conv.register_unstructure_hook(SemanticId, SemanticId.to_hex)
    conv.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
    return codec.encode_json(_ModelEnvelope(conv.unstructure(row), sources))


def _decode(data: bytes) -> ModelDeclaration:
    wire = codec.decode_json(data, _ModelEnvelope)
    return codec.converter().structure(wire.declaration, ModelDeclaration)


@attrs.define
class CaseBuilder:
    """Editable generated case row; numeric interpretation stays in Rust."""

    declaration: CaseDeclaration

    @classmethod
    def create(cls, case_id: SemanticId, name: str) -> Self:
        """Start an empty selected case."""
        return cls(
            CaseDeclaration(
                case_id=case_id,
                name=name,
                variables=(),
                parameters=(),
                instances=(),
                rows=(),
                objective=None,
                values=(),
            )
        )

    def variable(self, variable: Variable, value: float) -> Self:
        """Declare a variable and its initial or fixed value."""
        self.declaration = attrs.evolve(
            self.declaration,
            variables=(*self.declaration.variables, variable),
            values=(
                *self.declaration.values,
                Value(symbol_id=variable.port.symbol_id, value=value),
            ),
        )
        return self

    def parameter(self, parameter: Parameter, value: float) -> Self:
        """Declare one compile-sensitive physical parameter."""
        self.declaration = attrs.evolve(
            self.declaration,
            parameters=(*self.declaration.parameters, parameter),
            values=(
                *self.declaration.values,
                Value(symbol_id=parameter.symbol_id, value=value),
            ),
        )
        return self

    def instance(self, instance: Instance) -> Self:
        """Bind one reusable definition to semantic slots and output rows."""
        self.declaration = attrs.evolve(
            self.declaration, instances=(*self.declaration.instances, instance)
        )
        return self

    def row(self, row: Row) -> Self:
        """Declare the quantity and interval of one constraint value."""
        self.declaration = attrs.evolve(
            self.declaration, rows=(*self.declaration.rows, row)
        )
        return self

    def objective(self, objective: Objective) -> Self:
        """Declare the authored objective quantity and sense."""
        self.declaration = attrs.evolve(self.declaration, objective=objective)
        return self


@attrs.define
class ModelBuilder:
    """Editable draft; freeze atomically admits a new immutable native revision."""

    _runtime: _NativeRuntime
    declaration: ModelDeclaration
    _physical: _NativePhysicalContext
    _previous: _NativeModelRevision | None = None
    _sources: dict[str, object] = attrs.field(factory=dict)

    def definition(self, definition: Definition) -> Self:
        """Append a generated reusable source declaration."""
        self.declaration = attrs.evolve(
            self.declaration, definitions=(*self.declaration.definitions, definition)
        )
        return self

    def case(self, case: CaseBuilder | CaseDeclaration) -> Self:
        """Append a selected case; duplicates fail native admission."""
        row = case.declaration if isinstance(case, CaseBuilder) else case
        self.declaration = attrs.evolve(
            self.declaration, cases=(*self.declaration.cases, row)
        )
        return self

    def _source(self, family: str, row: object) -> Self:
        conv = codec.converter()
        conv.register_unstructure_hook(SemanticId, SemanticId.to_hex)
        conv.register_unstructure_hook(ContentHash, ContentHash.to_prefixed)
        previous = self._sources.get(family, [])
        if not isinstance(previous, list):
            message = "source inventory must be a list"
            raise TypeError(message)
        self._sources[family] = [*previous, conv.unstructure(row)]
        return self

    def numerical_requirement(self, row: a.AuthoredNumericalRequirementsRow) -> Self:
        """Declare an ID-keyed physical nominal or acceptance budget."""
        return self._source("numerics", row)

    def provider_scaling(self, row: a.AuthoredProviderScalingBindingsRow) -> Self:
        """Bind an exact native provider output to a property scaling default."""
        return self._source("scaling_bindings", row)

    def default_scaling(self, row: a.AuthoredDefaultScalingRow) -> Self:
        """Retain an authored default; only an explicit binding activates it."""
        return self._source("scaling_defaults", row)

    def dynamics(self, row: a.AuthoredDynamicCasesRow) -> Self:
        """Add the generated semi-explicit dynamic declaration."""
        return self._source("dynamics", row)

    def fit(self, row: a.AuthoredFitCasesRow) -> Self:
        """Bind shared parameters and authored observations to experiments."""
        return self._source("fits", row)

    def balance(self, row: a.AuthoredPhysicalBalancesRow) -> Self:
        """Declare signed physical sources and independent closure tolerances."""
        return self._source("balances", row)

    def directional_valve_law(self, row: a.AuthoredDirectionalValveLawsRow) -> Self:
        """Declare the C2 closed-to-forward valve law and its pressure width."""
        return self._source("valve_laws", row)

    def native_provider(self, row: a.AuthoredNativeProvidersRow) -> Self:
        """Select a native library factory without Python callbacks."""
        return self._source("providers", row)

    def observation(self, row: a.AuthoredObservationsRow) -> Self:
        """Append an original measurement and its physical units."""
        return self._source("observations", row)

    def dataset(self, row: a.AuthoredDatasetsRow) -> Self:
        """Append the measurement dataset provenance."""
        return self._source("datasets", row)

    def freeze(self) -> "ModelRevision":
        """Admit a revision while preserving the previous one on error."""
        wire = _encode(self.declaration, self._sources)
        native = (
            self._runtime.model(wire, self._physical)
            if self._previous is None
            else self._previous.revise(wire)
        )
        return ModelRevision(self._runtime, native, self._physical)


@attrs.frozen
class ModelRevision:
    """Immutable authored meaning and value assumptions, sharing a native compiler."""

    _runtime: _NativeRuntime
    _handle: _NativeModelRevision
    _physical: _NativePhysicalContext

    def prepare_flow(self, case_id: SemanticId, flow_id: SemanticId) -> "PreparedFlow":
        """Inspect physical ports and select tears from this exact revision."""
        return PreparedFlow(
            self._handle.prepare_flow(case_id.to_hex(), flow_id.to_hex())
        )

    @property
    def identity(self) -> ContentHash:
        """Complete native declaration, physical and provider identity."""
        return ContentHash.from_prefixed(self._handle.identity)

    @property
    def declaration(self) -> ModelDeclaration:
        """Original generated source declaration."""
        return _decode(self._handle.declaration())

    def edit(self) -> ModelBuilder:
        """Create an independent draft sharing only derived compiler state."""
        return ModelBuilder(
            self._runtime,
            self.declaration,
            self._physical,
            self._handle,
            codec.decode_json(self._handle.declaration(), _ModelEnvelope).sources,
        )

    def prepare(
        self,
        case_id: SemanticId,
        settings: SolveSettings,
    ) -> "PreparedCase":
        """Compile and admit an explicit representation without starting a solve."""
        return PreparedCase(self._handle.prepare(case_id.to_hex(), settings))

    def prepare_simulation(
        self, dynamic_id: SemanticId, settings: SimulationSettings
    ) -> "PreparedOperation":
        """Compile a finite native BDF simulation without starting it."""
        return PreparedOperation(
            self._handle.prepare_simulation(dynamic_id.to_hex(), settings)
        )

    def prepare_recycle(
        self, request: Mapping[str, object], settings: SolveSettings
    ) -> "PreparedStrategy":
        """Compile declared causal unit directions and a selected tear witness."""
        return PreparedStrategy(
            self._handle.prepare_recycle(
                codec.encode_json(_AnalysisDocument(dict(request))), settings
            )
        )

    def prepare_initialization(
        self,
        case_id: SemanticId,
        settings: SolveSettings,
        stages: Sequence[Mapping[SemanticId, float]],
    ) -> "PreparedStrategy":
        """Prepare finite overlays over this revision's original bindings."""
        return PreparedStrategy(
            self._handle.prepare_initialization(
                case_id.to_hex(),
                settings,
                [{k.to_hex(): v for k, v in stage.items()} for stage in stages],
            )
        )

    def prepare_fit(
        self,
        fit_id: SemanticId,
        settings: SolveSettings,
        simulations: Mapping[SemanticId, SimulationSettings] | None = None,
        *,
        rank_tolerance: float = 1e-8,
        max_cells: int = 1000000,
    ) -> "PreparedOperation":
        """Compile one native NLP over steady or smooth transient experiments."""
        profiles = [(key.to_hex(), value) for key, value in (simulations or {}).items()]
        return PreparedOperation(
            self._handle.prepare_fit(
                fit_id.to_hex(),
                settings,
                profiles,
                rank_tolerance=rank_tolerance,
                max_cells=max_cells,
            )
        )


@attrs.frozen
class PreparedOperation:
    """Prepared simulation or fitting request sharing the ordinary run lifecycle."""

    _handle: _NativePreparedOperation

    @property
    def identity(self) -> ContentHash:
        """Exact prepared source and profile identity."""
        return ContentHash.from_prefixed(self._handle.identity)

    def start(self) -> "RunHandle":
        """Start one admitted native operation."""
        return RunHandle(self._handle.start())


@attrs.frozen
class PreparedFlow:
    """Selected physical graph with explicit cost, grouping and tear policy."""

    _handle: _NativePreparedFlow

    def graph(self) -> dict[str, object]:
        """Return source-attributed nodes, scalar ports, connections and decisions."""
        return codec.decode_json(
            self._handle.graph_json().encode(), _AnalysisDocument
        ).payload

    def select_tears(self, method: str, settings: SolveSettings) -> "StrategyResult":
        """Run the explicitly selected native MILP or policy-respecting heuristic."""
        return StrategyResult(self._handle.select_tears(method, settings))


@attrs.frozen
class PreparedStrategy:
    """Prepared explicit cone, causal map or transactional initialization."""

    _handle: _NativePreparedStrategy

    @property
    def routes(self) -> tuple[str, ...]:
        """Routes selected before execution, without implicit failure fallback."""
        return tuple(self._handle.routes)

    def run(self) -> "StrategyResult":
        """Run one finite strategy and retain its joined reports."""
        return StrategyResult(self._handle.run())


@attrs.frozen
class StrategyResult:
    """Owned native attempts, including failures and stage overlays."""

    _handle: _NativeStrategyResult

    def tears(self) -> dict[str, object] | None:
        """Selected decisions, authored cost and independently checked acyclic order."""
        data = self._handle.tears_json()
        return (
            None
            if data is None
            else codec.decode_json(data.encode(), _AnalysisDocument).payload
        )

    def attempts(self) -> tuple[NativeAttempt, ...]:
        """Actual native attempts with termination, qualification and progress."""
        return tuple(self._handle.attempts())

    def failures(self) -> tuple[tuple[int, str], ...]:
        """Attributable failures before a native report became available."""
        return tuple(self._handle.failures())

    def initialization(self) -> dict[str, object] | None:
        """Original values, committed unknowns and temporary stage evidence."""
        data = self._handle.initialization_json()
        return (
            None
            if data is None
            else codec.decode_json(data.encode(), _AnalysisDocument).payload
        )


@attrs.frozen
class PreparedCase:
    """Class-specific native representation with a resolved, inspectable route."""

    _handle: _NativePreparedCase

    def with_primal_start(self, values: Mapping[SemanticId, float]) -> "PreparedCase":
        """Select a complete primal seed independently of allocation reuse."""
        return PreparedCase(
            self._handle.with_primal_start(
                {key.to_hex(): value for key, value in values.items()}
            )
        )

    @property
    def eligibility(self) -> tuple[tuple[str, tuple[str, ...]], ...]:
        """Backend alternatives and preparation refusals for this case."""
        return tuple(
            (name, tuple(reasons)) for name, reasons in self._handle.eligibility
        )

    def with_start(self, seed: _NativeStart) -> Self:
        """Select a compatible result seed independently of allocation reuse."""
        return type(self)(self._handle.with_start(seed))

    @property
    def route(self) -> str:
        """Admitted route; availability alone does not grant model eligibility."""
        return self._handle.route

    def start(self) -> "RunHandle":
        """Start one finite native attempt."""
        return RunHandle(self._handle.start())


@attrs.frozen
class PublicationTicket:
    """Serialized pre-effect recovery request; settlement validates native witnesses."""

    json: bytes


class PublicationRoot(msgspec.Struct, frozen=True):
    """Exact immutable control selection."""

    location: str
    version: int


class PublicationCommitted(
    msgspec.Struct, tag="committed", tag_field="status", frozen=True
):
    """Matching transaction and complete request witnesses establish this exact root."""

    root: PublicationRoot


class PublicationNoncommit(
    msgspec.Struct, tag="proved_noncommit", tag_field="status", frozen=True
):
    """Positive durable evidence excludes the conditional publication."""

    reason: str


class PublicationConflict(
    msgspec.Struct, tag="conflict", tag_field="status", frozen=True
):
    """An observed identity or exact-parent conflict."""

    reason: str


class PublicationUnresolved(
    msgspec.Struct, tag="unresolved", tag_field="status", frozen=True
):
    """Evidence is incomplete; preserve the ticket and unresolved members."""

    reason: str


PublicationSettlement: TypeAlias = (
    PublicationCommitted
    | PublicationNoncommit
    | PublicationConflict
    | PublicationUnresolved
)


@attrs.frozen
class PublicationRequest:
    """Caller-stable identities and exact parent for one intended publication."""

    base: str
    workspace_id: SemanticId
    publication_id: SemanticId
    attempt_id: SemanticId
    parent: SemanticId | None = None


@attrs.frozen
class PublicationAttempt:
    """Explicit single-use write command over immutable results."""

    _handle: _NativePublicationAttempt

    @property
    def ticket(self) -> PublicationTicket:
        """Save before commit; it remains available after the command is consumed."""
        return PublicationTicket(self._handle.ticket())

    @property
    def attempt_id(self) -> SemanticId:
        """Native settlement identity retained after a failed commit."""
        return SemanticId.from_hex(self._handle.attempt_id)

    @property
    def publication_id(self) -> SemanticId:
        """Identity of the proposed control publication."""
        return SemanticId.from_hex(self._handle.publication_id)

    def commit(self) -> tuple[str, int]:
        """Return exact control URI/version for pse.open; never retry implicitly."""
        return self._handle.commit()


@attrs.frozen
class RunCompletion:
    """Registry-owned completion records shared with the durable Arrow projection."""

    solves: tuple[result_contracts.RuntimeSolveRunsRow, ...]
    computation: result_contracts.RuntimeComputationRunsRow | None
    lineage: tuple[result_contracts.RuntimeRunLineageRow, ...]
    assessments: tuple[result_contracts.RuntimeCandidateAssessmentsRow, ...]


@attrs.frozen
class RunResult:
    """Joined immutable outcome; failed and unattempted steps remain inspectable."""

    _handle: _NativeRunResult

    def available_start(self, step: int = 0) -> _NativeStart | None:
        """Return available numerical seed data and its provenance."""
        return self._handle.available_start(step)

    @property
    def run_id(self) -> SemanticId:
        """Unique execution identity."""
        return SemanticId.from_hex(self._handle.run_id)

    @property
    def usable(self) -> bool:
        """Whether every requested candidate satisfies its final usability policy."""
        return self._handle.usable

    @property
    def completion(self) -> RunCompletion:
        """Read the immutable joined assessment without evaluating the model again."""
        wire = msgspec.json.decode(self._handle.completion(), type=dict[str, object])
        wire.pop("diagnostics")
        return codec.converter().structure(wire, RunCompletion)

    def diagnostics(self) -> tuple[DiagnosticReport, ...]:
        """Structured native admission and execution failures, preserving causes."""
        return tuple(self._handle.diagnostics())

    def tables(self) -> tuple[str, ...]:
        """Declared result and reconstruction-source relation names."""
        return tuple(self._handle.tables())

    def table(self, name: str) -> TableStream:
        """Return a one-consumption stream owning its final Arrow buffers."""
        return TableStream(self._handle.table(name))

    def prepare_publication(
        self, base: str, workspace_id: SemanticId, *, parent: SemanticId | None = None
    ) -> PublicationAttempt:
        """Prepare control-last publication; this call does not write."""
        return PublicationAttempt(
            self._handle.prepare_publication(
                base,
                workspace_id.to_hex(),
                parent=None if parent is None else parent.to_hex(),
            )
        )

    def prepare_publication_request(
        self, request: PublicationRequest
    ) -> PublicationAttempt:
        """Prepare a retained request identity without performing any writes."""
        return PublicationAttempt(
            self._handle.prepare_publication(
                request.base,
                request.workspace_id.to_hex(),
                parent=None if request.parent is None else request.parent.to_hex(),
                publication_id=request.publication_id.to_hex(),
                attempt_id=request.attempt_id.to_hex(),
            )
        )


@attrs.frozen
class RunHandle:
    """Blocking and asyncio access to the same supervised native job."""

    _handle: _NativeRunHandle

    def cancel(self) -> None:
        """Request stop; join and terminal report ownership remain native."""
        self._handle.cancel()

    def wait(self) -> RunResult:
        """Release Python while waiting; join cancellation before a signal escapes."""
        return RunResult(self._handle.wait())

    async def wait_async(self) -> RunResult:
        """Request native stop on cancellation; retain the report for later waiters."""
        return RunResult(await self._handle.wait_async())

    def result(self) -> RunResult | None:
        """Nonblocking terminal snapshot."""
        result = self._handle.result()
        return None if result is None else RunResult(result)

    def progress(self) -> tuple[tuple[ProgressEvent, ...], int]:
        """Observe bounded typed native events and actual dropped-event count."""
        events, dropped = self._handle.progress()
        return tuple(events), dropped

    @property
    def progress_count(self) -> tuple[int, int]:
        """Retained native events and dropped-event count."""
        return self._handle.progress_count


@attrs.frozen(init=False)
class Runtime:
    """One native deployment budget shared with publication inspection."""

    _handle: _NativeRuntime

    def __init__(self, settings: EngineSettings) -> None:
        object.__setattr__(self, "_handle", _NativeRuntime(settings))

    def physical_from_documents(
        self, documents: Mapping[str, str]
    ) -> "PhysicalContext":
        """Admit actual physical package rows using the native source loader."""
        return PhysicalContext(self._handle.physical_from_documents(dict(documents)))

    def capabilities(self) -> tuple[SolverCapability, ...]:
        """Discover linked native libraries without PATH or optional Python probes."""
        return tuple(
            codec.converter().structure(
                msgspec.json.decode(self._handle.capabilities()), list[SolverCapability]
            )
        )

    def clear_program_cache(self) -> None:
        """Release retained programs while keeping active prepared workers valid."""
        self._handle.clear_program_cache()

    def settle_publication(self, ticket: PublicationTicket) -> PublicationSettlement:
        """Observe saved native witnesses without repeating a solve or publication."""
        return msgspec.json.decode(
            self._handle.settle_publication(ticket.json), type=PublicationSettlement
        )

    def prepare_conic(
        self,
        request: Mapping[str, object],
        physical: "PhysicalContext",
        settings: SolveSettings,
    ) -> PreparedStrategy:
        """Admit explicit library cone geometry and an exact quadratic witness."""
        return PreparedStrategy(
            self._handle.prepare_conic(
                codec.encode_json(_AnalysisDocument(dict(request))),
                physical._handle,  # noqa: SLF001 - same native boundary
                settings,
            )
        )

    def model(
        self, model_id: SemanticId, name: str, physical: "PhysicalContext"
    ) -> ModelBuilder:
        """Start a typed draft using an explicitly admitted physical catalog."""
        return self.from_declaration(
            ModelDeclaration(
                model_id=model_id,
                name=name,
                definitions=(),
                domains=(),
                groups=(),
                cases=(),
            ),
            physical,
        )

    def from_declaration(
        self, declaration: ModelDeclaration, physical: "PhysicalContext"
    ) -> ModelBuilder:
        """Use the same generated contract as package documents."""
        return ModelBuilder(self._handle, declaration, physical._handle)  # noqa: SLF001 - same native boundary

    def models_from_documents(
        self, documents: Mapping[str, str], physical: "PhysicalContext"
    ) -> tuple[ModelRevision, ...]:
        """Load a complete package.toml and computation_models YAML source inventory."""
        return tuple(
            ModelRevision(self._handle, row, physical._handle)  # noqa: SLF001 - same native boundary
            for row in self._handle.models_from_documents(
                dict(documents),
                physical._handle,  # noqa: SLF001
            )
        )

    def start(
        self, cases: Sequence[PreparedCase], *, continue_independent: bool = False
    ) -> RunHandle:
        """Run a finite sequence on the existing completion-owned native pipeline."""
        return RunHandle(
            self._handle.start(
                [c._handle for c in cases],  # noqa: SLF001 - same native boundary
                continue_independent=continue_independent,
            )
        )


@attrs.frozen
class PhysicalContext:
    """Admitted quantities, units, prerequisites and original source rows."""

    _handle: _NativePhysicalContext

    @property
    def identity(self) -> ContentHash:
        """Complete native compiler context identity."""
        return ContentHash.from_prefixed(self._handle.identity)
