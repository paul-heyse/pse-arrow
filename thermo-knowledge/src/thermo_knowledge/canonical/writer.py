# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The typed writer every mapping emits canonical rows through (pipeline section 2).

A mapping never computes an identifier, a unit conversion, an orientation or a provenance row;
this writer does, and it refuses what the declaration refuses, reporting each refusal with the
locator of the source row instead of leaving it to fail in the database:

* `kind` writes a kind instance: its identifier from the declared identity
  (`thermo_knowledge.identity`), the row of every table of its refinement chain, the members of
  its `Set<K>` attributes, its `prov.record` row when the kind's provenance is `own`, and its
  origin rows;
* `relation` writes a relation row the same way, with the keys of a transposable relation in
  the canonical orientation;
* `parameter_set` writes one parameter set of a slot group: `subject_key`, the subjects in the
  canonical orientation, the set row, the slot-group row and the family rows;
* `subform_choice` writes which form fills a sub-form slot for one subject;
* `validity_region` writes one region of the validity of a record with its clauses and the
  coverage row, and `validity_not_stated` records that the source states none.

The values are stored as the source asserted them. A transposable subject tuple is stored in the
canonical orientation (so uniqueness and `subject_key` do not depend on the order asserted), and
where the group's rule acts on values the row records the `arrangement` the values were asserted
for. The writer never transforms a value; the rule is applied on reading (`transposition.py`).

Validation covers types, required values, enum members, the domain checks of the generated DDL,
the invariants declared `ddl` (from their declared check) and `load` (`invariants.py`) and unit
conversion through `pint`. Nothing is written for a record that fails. `transaction` makes a
group of records atomic: the rows of a failing group are withdrawn.

A record of a kind with `provenance = "own"` needs at least one `Origin`; the writer creates the
`carrier`, `artifact`, `import_record`, `licence` and `rights_determination` rows it names and
the `record_origin` rows. Two records with one identifier and different content are a
`CompetingAssertion`: the mapping is wrong about identity.
"""

from __future__ import annotations

import math
import unicodedata
import uuid
from collections.abc import Iterator, Mapping, Sequence
from contextlib import contextmanager
from dataclasses import dataclass

import pyarrow as pa

from thermo_knowledge import identity, transposition
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical import invariants, tied_keys
from thermo_knowledge.canonical import values as v
from thermo_knowledge.canonical.provenance import Carriers, CarrierInfo, Origin, SourceRef
from thermo_knowledge.canonical.schemas import UUID, canonical_schemas, table_name
from thermo_knowledge.canonical.values import (
    Converted,
    NotApplicable,
    Problems,
    QuantityArray,
    Redirect,
    ValueRefused,
    Withheld,
)
from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.types import TypeRef
from thermo_knowledge.expression.units import describe, type_dimension

RECORD = "prov.record"


class WriteError(Exception):
    """A refusal of the writer."""


class ValidationError(WriteError):
    """A record failed validation; nothing of it was written."""

    def __init__(self, locator: str, problems: Sequence[str]) -> None:
        self.locator = locator
        self.problems = tuple(problems)
        super().__init__(f"{locator}: " + "; ".join(self.problems))


class MissingOrigin(WriteError):
    """A record of an `own` kind was emitted without an origin row."""

    def __init__(self, locator: str, what: str) -> None:
        self.locator = locator
        super().__init__(
            f"{locator}: {what} has no origin: a record needs at least one import record"
        )


class CompetingAssertion(WriteError):
    """Two records share an identifier and differ in content."""

    def __init__(self, table: str, first: str, second: str, columns: Sequence[str]) -> None:
        self.table = table
        self.first = first
        self.second = second
        super().__init__(
            f"competing assertions for one {table} identifier, at {first} and at {second}; "
            f"they differ in {', '.join(columns)}: the mapping is wrong about identity"
        )


@dataclass(frozen=True)
class FamilyRow:
    """One row of an indexed family: the index values by name and the slot values."""

    index: Mapping[str, object]
    values: Mapping[str, object]


@dataclass(frozen=True)
class NestedSet:
    """The value of a slot that accepts a contract: a parameter set of a slot group of a form
    implementing it.

    `subjects` are the nested group's subjects as the source asserted them (none for a global
    group). The set takes the parameterization and origins of the set that holds it."""

    slot_group: str
    slots: Mapping[str, object]
    families: Mapping[str, Sequence[FamilyRow]] | None = None
    subjects: Sequence[uuid.UUID] = ()


@dataclass(frozen=True)
class SetReference:
    """The value of a slot that references a set: an independently identified, top-level
    parameter set of a slot group of a form implementing the contract the slot names, which many
    sets may reference.

    The target is named as a set is identified: by its `parameterization`, its slot group, its
    `subjects` (in any order of a transposable group) and its `occurrence`."""

    parameterization: uuid.UUID
    slot_group: str
    subjects: Sequence[uuid.UUID] = ()
    occurrence: int = 1


@dataclass(frozen=True)
class TabulatedAxis:
    """One axis of a tabulated function: the name of its quantity type and its points, in
    ascending order, with their source unit."""

    quantity_type: str
    points: QuantityArray


@dataclass(frozen=True)
class TabulatedSeries:
    """One named series of a tabulated function: the name of its quantity type and its values in
    row-major order over the axes (the last axis varying fastest), with their source unit."""

    name: str
    quantity_type: str
    values: QuantityArray


@dataclass(frozen=True)
class TabulatedFunction:
    """The value of a slot that holds a tabulated function: its axes (numbered from one in the
    order given), its series on the full grid of the axes and how it is evaluated between points.

    The function's key is computed from the set that holds it, the slot and the family index, and
    it takes the origins of that set."""

    interpolation: str
    axes: Sequence[TabulatedAxis]
    series: Sequence[TabulatedSeries]
    distribution: uuid.UUID | None = None


@dataclass(frozen=True)
class _Holder:
    """The parameter set that holds a nested set, and what the nested set takes from it."""

    set_id: uuid.UUID
    parameterization: uuid.UUID
    origins: Sequence[Origin]
    locator: str


@dataclass
class _Stored:
    row: dict[str, object]
    locator: str


def _kinds_on_unit_paths(decl: m.Declaration) -> frozenset[str]:
    """The kinds a `unit_from` path passes through: the writer keeps the values of their
    instances so that a later record can read its unit from them."""
    found: set[str] = set()
    fields = [f for kind in decl.kinds.values() for f in kind.attributes]
    fields += [f for relation in decl.relations.values() for f in relation.values]
    for field in fields:
        if field.unit_from is None or "." not in field.unit_from:
            continue
        # the path starts at a reference of the record that owns the field; every reference but
        # the last names a kind whose instances the path goes through
        for kind in decl.kinds.values():
            owned = {a.name: a for a in decl.attributes_of(kind.name)}
            if field not in owned.values():
                continue
            current = owned
            for segment in field.unit_from.split(".")[:-1]:
                target = current[segment].type.element
                found.add(target)
                current = {a.name: a for a in decl.attributes_of(target)}
        for relation in decl.relations.values():
            if field in relation.values:
                current = {f.name: f for f in (*relation.keys, *relation.values)}
                for segment in field.unit_from.split(".")[:-1]:
                    target = current[segment].type.element
                    found.add(target)
                    current = {a.name: a for a in decl.attributes_of(target)}
    return frozenset(found)


def _is_real(attribute: m.Field) -> bool:
    return attribute.type.element_kind == "real"


def _sort_key(key: tuple[object, ...]) -> tuple[tuple[int, object], ...]:
    return tuple((0, part) if isinstance(part, int) else (1, str(part)) for part in key)


class CanonicalWriter:
    """Collects the canonical rows of one mapping run, table by table."""

    def __init__(self, decl: m.Declaration, carriers: Carriers | None = None) -> None:
        missing = invariants.missing_evaluators(decl)
        if missing:
            listed = ", ".join(f"{kind}.{name}" for kind, name in missing)
            raise WriteError(
                f"the declaration enforces these invariants at load and the writer has no "
                f"evaluator for them: {listed}"
            )
        self.decl = decl
        self.carriers = carriers if carriers is not None else Carriers()
        self.schemas = canonical_schemas(decl)
        self._meta_ids = decl.meta_ids()
        self._meta_names = {
            construct: {ident: name for name, ident in names.items()}
            for construct, names in self._meta_ids.items()
        }
        self._entities = {entity.id: entity for entity in decl.entities}
        self._slots: dict[str, m.Field] = {}
        for group in decl.slot_groups:
            for slot in group.slots:
                self._slots[f"{group.qualified}.{slot.name}"] = slot
            for family in group.families:
                for slot in family.slots:
                    self._slots[f"{family.qualified}.{slot.name}"] = slot
        self._unit_kinds = _kinds_on_unit_paths(decl)
        self._instances: dict[uuid.UUID, tuple[str, Mapping[str, Converted]]] = {}
        self._tables: dict[str, dict[tuple[object, ...], _Stored]] = {}
        self._seen: set[object] = set()
        self._journal: list[tuple[str, object, object]] | None = None

    # -- bookkeeping --------------------------------------------------------------------------

    @contextmanager
    def transaction(self) -> Iterator[None]:
        """Group the records written inside into one unit: if the block raises, every row it
        wrote is withdrawn."""
        if self._journal is not None:
            raise WriteError("transactions do not nest")
        self._journal = []
        try:
            yield
        except BaseException:
            for entry, table, key in reversed(self._journal):
                if entry == "row":
                    del self._tables[str(table)][key]  # type: ignore[index]
                elif entry == "instance":
                    self._instances.pop(key, None)  # type: ignore[arg-type]
                else:
                    self._seen.discard(key)
            raise
        finally:
            self._journal = None

    @contextmanager
    def _atomic(self) -> Iterator[None]:
        """A call's rows are all written or none: a transaction of its own unless one is open."""
        if self._journal is not None:
            yield
        else:
            with self.transaction():
                yield

    def _remember(self, key: object) -> bool:
        """Note `key`; whether it was new."""
        if key in self._seen:
            return False
        self._seen.add(key)
        if self._journal is not None:
            self._journal.append(("seen", None, key))
        return True

    def _put(
        self, table: str, key: tuple[object, ...], row: dict[str, object], locator: str
    ) -> None:
        names = self.schemas[table].names
        if set(row) != set(names):
            raise WriteError(
                f"internal error: the row for {table} has columns {sorted(row)}, "
                f"the table has {sorted(names)}"
            )
        rows = self._tables.setdefault(table, {})
        existing = rows.get(key)
        if existing is not None:
            if existing.row != row:
                differing = sorted(c for c in row if existing.row.get(c) != row[c])
                raise CompetingAssertion(table, existing.locator, locator, differing)
            return
        rows[key] = _Stored(row, locator)
        if self._journal is not None:
            self._journal.append(("row", table, key))

    def rows(self, table: str) -> int:
        """The number of rows written to `table` so far."""
        return len(self._tables.get(table, {}))

    def counts(self) -> dict[str, int]:
        """The number of rows in each table that has any (a table whose rows were all withdrawn
        by a failed group has none)."""
        return {name: len(rows) for name, rows in sorted(self._tables.items()) if rows}

    def tables(self) -> dict[str, pa.Table]:
        """Every table that has rows, as an Arrow table of its canonical schema, rows in
        identifier order (the same records in any order give the same tables)."""
        result: dict[str, pa.Table] = {}
        for name, schema in self.schemas.items():
            rows = self._tables.get(name)
            if not rows:
                continue
            ordered = [rows[key].row for key in sorted(rows, key=_sort_key)]
            arrays = [_array([row[field.name] for row in ordered], field.type) for field in schema]
            result[name] = pa.Table.from_arrays(arrays, schema=schema)
        return result

    # -- locators -----------------------------------------------------------------------------

    @staticmethod
    def _locator(origins: Sequence[Origin], at: str | None) -> str:
        if at is not None:
            return at
        if origins:
            return origins[0].ref.locator
        return "(no source row)"

    # -- values -------------------------------------------------------------------------------

    def _instance(self, reference: uuid.UUID) -> tuple[str, Mapping[str, Converted]] | None:
        """The kind and attribute values of an instance the writer can see: a declared entity or
        one it has written (of a kind some `unit_from` path goes through)."""
        entity = self._entities.get(reference)
        if entity is not None:
            return entity.kind, entity.values  # type: ignore[return-value]
        return self._instances.get(reference)

    def _real_unit(
        self,
        owner: Mapping[str, m.Field],
        attribute: m.Field,
        values: Mapping[str, Converted],
    ) -> str | None:
        """The storage unit of a `Real` value, from the `unit_from` its declaration states:
        `dimensionless`, or a path of references through `values` (the record's own converted
        values) to a quantity type, a slot or an observable."""
        path = attribute.unit_from
        if path is None:
            return None
        if path == m.DIMENSIONLESS:
            return path
        segments = path.split(".")
        fields: Mapping[str, m.Field] = owner
        current: Mapping[str, Converted] = values
        for position, segment in enumerate(segments):
            field = fields[segment]
            reference = current.get(segment)
            if not isinstance(reference, uuid.UUID):
                raise ValueRefused(
                    f"its unit comes from `{path}`, and `{segment}` has no value to take it from"
                )
            if position < len(segments) - 1:
                found = self._instance(reference)
                if found is None:
                    raise ValueRefused(
                        f"its unit comes from `{path}`, and the {field.type.element} {reference} "
                        "is neither declared nor written before this record"
                    )
                fields = {a.name: a for a in self.decl.attributes_of(found[0])}
                current = found[1]
                continue
            return self._terminal_unit(path, field, reference)
        raise AssertionError("unreachable: a unit path has a last segment")

    def _terminal_unit(self, path: str, field: m.Field, reference: uuid.UUID) -> str:
        decl = self.decl
        type_ = field.type
        if type_.element_kind == "meta" and type_.element == "quantity_type":
            return decl.quantity_types[self._meta_names["quantity_type"][reference]].unit
        if type_.element_kind == "meta":  # a slot
            qualified = self._meta_names["slot"][reference]
            slot = self._slots[qualified]
            unit = v.storage_unit(decl, slot.type)
            if unit is None:
                raise ValueRefused(
                    f"its unit comes from `{path}`, and the slot {qualified} is not a quantity"
                )
            return unit
        found = self._instance(reference)
        if found is None:
            raise ValueRefused(
                f"its unit comes from `{path}`, and the observable {reference} is not declared"
            )
        quantity = next(
            a
            for a in decl.attributes_of(found[0])
            if a.type.element_kind == "meta" and a.type.element == "quantity_type"
        )
        return decl.quantity_types[self._meta_names["quantity_type"][found[1][quantity.name]]].unit  # type: ignore[index]

    def _convert(
        self,
        owner: Mapping[str, m.Field],
        attribute: m.Field,
        raw: object,
        merged: Mapping[str, Converted],
    ) -> Converted:
        type_ = attribute.type
        hint = self._real_unit(owner, attribute, merged)
        if type_.container == "scalar":
            return v.scalar(self.decl, type_, raw, unit_hint=hint, meta_ids=self._meta_ids)
        if type_.container == "array":
            return v.array(self.decl, type_, raw, unit_hint=hint)
        if type_.container == "set":
            if not isinstance(raw, (list, tuple, set, frozenset)) or not all(
                isinstance(item, uuid.UUID) for item in raw
            ):
                raise ValueRefused(f"{type_.text} takes identifiers of {type_.element} instances")
            return sorted(set(raw), key=str)  # type: ignore[arg-type]
        raise ValueRefused(
            f"{type_.text} cannot be written: a PostgreSQL range has no binary form the bulk "
            "loader can ingest"
        )

    def _assemble(
        self,
        kind_name: str,
        given: Mapping[str, object],
        locator: str,
        *,
        extra: Mapping[str, Converted] | None = None,
    ) -> dict[str, Converted]:
        """Validate `given` against every attribute of `kind_name` and its refinement chain.

        Returns attribute name to converted value (a stateful slot also gets `<slot>__state` and
        `<slot>__redirect`). Raises `ValidationError` with every problem found."""
        decl = self.decl
        attributes = decl.attributes_of(kind_name)
        problems = Problems()
        owner = {a.name: a for a in attributes}
        for name in sorted(set(given) - set(owner)):
            problems.add(name, f"is not an attribute of kind `{kind_name}`")
        merged: dict[str, Converted] = dict(extra or {})
        ordered = sorted(attributes, key=_is_real)
        for attribute in ordered:
            name = attribute.name
            raw = given.get(name)
            if attribute.presence == "stateful":
                self._stateful(owner, attribute, raw, merged, problems)
                continue
            if isinstance(raw, (NotApplicable, Withheld, Redirect)):
                problems.add(name, "is not a stateful slot, so it takes a value")
                continue
            if raw is None:
                if attribute.default is not None:
                    merged[name] = attribute.default  # type: ignore[assignment]
                elif attribute.type.container == "set":
                    merged[name] = []
                elif attribute.optional:
                    merged[name] = None
                else:
                    problems.add(name, "a required value is missing")
                continue
            try:
                merged[name] = self._convert(owner, attribute, raw, merged)
            except ValueRefused as error:
                problems.add(name, str(error))
        if not problems:
            for kind in decl.chain(kind_name):
                for requirement in kind.requires:
                    found = invariants.violation(kind.name, requirement, merged)
                    if found is not None:
                        problems.add(kind.name, found)
        if problems:
            raise ValidationError(locator, problems.items)
        return merged

    def _stateful(
        self,
        owner: Mapping[str, m.Field],
        attribute: m.Field,
        raw: object,
        merged: dict[str, Converted],
        problems: Problems,
    ) -> None:
        name = attribute.name
        state_name, redirect_name = f"{name}__state", f"{name}__redirect"
        if raw is None:
            problems.add(name, "a stateful slot states a value or why it has none")
            return
        if isinstance(raw, NotApplicable):
            merged.update({name: None, state_name: "not_applicable", redirect_name: None})
        elif isinstance(raw, Withheld):
            merged.update({name: None, state_name: "withheld", redirect_name: None})
        elif isinstance(raw, Redirect):
            merged.update({name: None, state_name: "redirect", redirect_name: raw.parameter_set})
        else:
            try:
                merged[name] = self._convert(owner, attribute, raw, merged)
            except ValueRefused as error:
                problems.add(name, str(error))
                return
            merged.update({state_name: "known", redirect_name: None})

    # -- provenance ---------------------------------------------------------------------------

    def _ensure_carrier(self, manifest_id: str, locator: str) -> uuid.UUID:
        carrier = self.carriers.infos.get(manifest_id)
        if carrier is None:
            raise ValidationError(
                locator, [f"the carrier `{manifest_id}` is not known to the writer"]
            )
        carrier_id = identity.identifier(pc.SOURCE.declared, [carrier.key])
        if not self._remember((pc.CARRIER.declared, carrier.key)):
            return carrier_id
        if len(carrier.tree_hash) != 64:
            raise ValidationError(
                locator,
                [f"the carrier `{carrier.key}` has no SHA-256 tree hash (`{carrier.tree_hash}`)"],
            )
        self.kind(
            pc.CARRIER.declared,
            {
                pc.CARRIER.key: carrier.key,
                pc.CARRIER.title: carrier.title,
                pc.CARRIER.manifest_id: carrier.manifest_id,
                pc.CARRIER.resolved_pin: carrier.pin,
                pc.CARRIER.tree_hash: carrier.tree_hash,
                pc.CARRIER.retrieved: carrier.retrieved_at,
            },
            at=locator,
        )
        ordinals: dict[str, int] = {}
        for rights in carrier.rights:
            licence: uuid.UUID | None = None
            if rights.spdx is not None:
                licence = self.kind(
                    pc.LICENCE.declared,
                    {
                        pc.LICENCE.key: rights.spdx,
                        pc.LICENCE.spdx: rights.spdx,
                        pc.LICENCE.title: rights.spdx,
                    },
                    at=locator,
                )
            ordinals[rights.scope] = ordinals.get(rights.scope, 0) + 1
            rd = pc.RIGHTS_DETERMINATION
            self.kind(
                rd.declared,
                {
                    rd.carrier: carrier_id,
                    rd.scope: rights.scope,
                    rd.ordinal: ordinals[rights.scope],
                    rd.basis: rights.basis,
                    rd.licence: licence,
                    rd.statement: rights.statement,
                    rd.url: rights.url,
                    rd.store: rights.store,
                    rd.redistribute: rights.redistribute,
                    rd.commercial: rights.commercial,
                    rd.attribution_required: rights.attribution,
                    rd.share_alike: rights.share_alike,
                    rd.observed: rights.observed,
                },
                at=locator,
            )
        return carrier_id

    def citation(
        self,
        carrier: str,
        key: str,
        *,
        doi: str | None = None,
        year: int | None = None,
        citation: str | None = None,
        at: str | None = None,
    ) -> uuid.UUID:
        """The publication a carrier's citation `key` denotes, written under the key its identity
        rule gives, with the `citation` row that ties the carrier's key to it; returns the
        publication's identifier.

        A publication with a DOI is keyed `doi:` and the DOI in lower case, so two carriers that
        cite one paper under different keys name one publication; without a DOI it is keyed
        `carrier:<manifest id>:<key>`. What the carrier says about the work (its `key`, `year`
        and `citation` text) is per carrier and goes on the `citation` row, so carriers that
        differ in it still agree on the publication. `carrier` is the manifest id of a carrier
        the writer knows; its provenance rows are written with the first citation."""
        locator = at if at is not None else f"citation {carrier}:{key}"
        if not key.strip():
            raise ValidationError(locator, ["a citation key is not empty"])
        cleaned = None if doi is None else unicodedata.normalize("NFC", doi).strip().lower()
        if cleaned == "":
            raise ValidationError(locator, ["a DOI, when stated, is not empty"])
        publication = pc.PUBLICATION
        link = pc.CITATION
        with self._atomic():
            carrier_id = self._ensure_carrier(carrier, locator)
            text = f"doi:{cleaned}" if cleaned is not None else f"carrier:{carrier}:{key}"
            found = self.kind(
                publication.declared,
                {
                    pc.SOURCE.key: text,
                    pc.SOURCE.title: text,
                    publication.doi: cleaned,
                },
                at=locator,
            )
            self.relation(
                link.declared,
                {link.carrier: carrier_id, link.local_key: key},
                {link.publication: found, link.year: year, link.citation: citation},
                at=locator,
            )
        return found

    def import_record(self, ref: SourceRef) -> uuid.UUID:
        """The `import_record` of a source row, with its `artifact` and `carrier`."""
        carrier_id = self._ensure_carrier(ref.carrier, ref.locator)
        carrier = self.carriers.infos[ref.carrier]
        try:
            info = self.carriers.artifact(ref.carrier, ref.artifact)
        except (KeyError, OSError) as error:
            raise ValidationError(
                ref.locator, [f"the artifact {ref.artifact} cannot be hashed: {error}"]
            ) from error
        artifact = pc.ARTIFACT
        artifact_id = self.kind(
            artifact.declared,
            {
                artifact.carrier: carrier_id,
                artifact.path: ref.artifact,
                artifact.sha256: info.sha256,
                artifact.size: info.size,
            },
            at=ref.locator,
        )
        imported = pc.IMPORT_RECORD
        return self.kind(
            imported.declared,
            {
                imported.artifact: artifact_id,
                imported.locator: ref.locator,
                imported.reader: carrier.reader,
                imported.reader_version: carrier.reader_version,
            },
            at=ref.locator,
        )

    def _register(
        self, record_id: uuid.UUID, kind_name: str, origins: Sequence[Origin], locator: str
    ) -> None:
        """Register a record and link it to each origin."""
        if not origins:
            raise MissingOrigin(locator, f"the {kind_name} {record_id}")
        self._put(RECORD, (record_id,), {"id": record_id, "kind": kind_name}, locator)
        for origin in origins:
            link = pc.RECORD_ORIGIN
            self.relation(
                link.declared,
                {link.record: record_id, link.import_record: self.import_record(origin.ref)},
                {link.role: origin.role},
                at=origin.ref.locator,
            )

    # -- kinds --------------------------------------------------------------------------------

    def kind(
        self,
        name: str,
        values: Mapping[str, object],
        *,
        origins: Sequence[Origin] = (),
        at: str | None = None,
    ) -> uuid.UUID:
        """Write one instance of kind `name` and return its identifier.

        `values` are the attribute values by name (a quantity as `Quantity`, a reference as the
        referenced instance's identifier, a `Set<K>` as identifiers). A kind with
        `provenance = "own"` needs `origins`; any other kind takes none."""
        locator = self._locator(origins, at)
        kind = self.decl.kinds.get(name)
        if kind is None:
            raise ValidationError(locator, [f"`{name}` is not a kind"])
        if kind.abstract:
            raise ValidationError(
                locator, [f"kind `{name}` is abstract and has no direct instances"]
            )
        if kind.origin == "slot_group":
            raise ValidationError(
                locator, [f"`{name}` is a slot group: write its sets with `parameter_set`"]
            )
        with self._atomic():
            problems = Problems()
            completed = tied_keys.complete(self.decl, name, values, problems)
            if problems:
                raise ValidationError(locator, problems.items)
            merged = self._assemble(name, completed, locator)
            return self._emit_kind(name, merged, origins, locator)

    def _emit_kind(
        self,
        name: str,
        merged: Mapping[str, Converted],
        origins: Sequence[Origin],
        locator: str,
        *,
        slot_group: uuid.UUID | None = None,
    ) -> uuid.UUID:
        decl = self.decl
        chain = decl.chain(name)
        root = chain[0]
        record_id = identity.identifier(root.name, self._identity_values(root, merged))
        mode = root.provenance.mode
        if mode != "own" and origins:
            raise ValidationError(
                locator,
                [
                    f"kind `{name}` has provenance `{mode}`: it registers no record and takes no origin"
                ],
            )
        if mode == "own" and not origins:
            raise MissingOrigin(locator, f"the {name} {record_id}")
        for kind in chain:
            table = table_name(kind.schema, kind.name)
            row: dict[str, object] = {"id": record_id}
            if kind.origin == "slot_group":
                row["slot_group"] = slot_group
            for attribute in kind.attributes:
                column = attribute.name
                if attribute.type.container == "set":
                    child = table_name(kind.schema, f"{kind.name}__{column}")
                    for member in merged[column] or []:  # type: ignore[union-attr]
                        self._put(
                            child,
                            (record_id, member),
                            {"owner": record_id, "member": member},
                            locator,
                        )
                    continue
                row[column] = merged[column]
                if attribute.presence == "stateful":
                    row[f"{column}__state"] = merged[f"{column}__state"]
                    row[f"{column}__redirect"] = merged[f"{column}__redirect"]
            self._put(table, (record_id,), row, locator)
        if mode == "own":
            self._register(record_id, name, origins, locator)
        if any(link.name in self._unit_kinds for link in chain):
            self._instances[record_id] = (name, dict(merged))
            if self._journal is not None:
                self._journal.append(("instance", None, record_id))
        return record_id

    # -- relations ----------------------------------------------------------------------------

    def relation(
        self,
        name: str,
        keys: Mapping[str, object],
        values: Mapping[str, object] | None = None,
        *,
        origins: Sequence[Origin] = (),
        at: str | None = None,
    ) -> uuid.UUID:
        """Write one row of relation `name`: its keys by role name, its value columns by name.

        The keys of a transposable relation are stored in the canonical orientation and the values
        as asserted, with the `arrangement` recorded where the rule acts on values."""
        with self._atomic():
            return self._relation(name, keys, values, origins=origins, at=at)

    def _relation(
        self,
        name: str,
        keys: Mapping[str, object],
        values: Mapping[str, object] | None = None,
        *,
        origins: Sequence[Origin] = (),
        at: str | None = None,
    ) -> uuid.UUID:
        locator = self._locator(origins, at)
        relation = self.decl.relations.get(name)
        if relation is None:
            raise ValidationError(locator, [f"`{name}` is not a relation"])
        given = dict(values or {})
        problems = Problems()
        fields = {f.name: f for f in (*relation.keys, *relation.values)}
        for extra in sorted((set(keys) - {k.name for k in relation.keys})):
            problems.add(extra, f"is not a key of relation `{name}`")
        for extra in sorted(set(given) - {c.name for c in relation.values}):
            problems.add(extra, f"is not a value column of relation `{name}`")
        if m.ARRANGEMENT in given:
            problems.add(m.ARRANGEMENT, "is recorded by the writer from the order of the keys")
        converted: dict[str, Converted] = {}
        for key in relation.keys:
            raw = keys.get(key.name)
            if raw is None:
                problems.add(key.name, "a key is missing")
                continue
            try:
                converted[key.name] = self._convert(fields, key, raw, converted)
            except ValueRefused as error:
                problems.add(key.name, str(error))
        for column in relation.values:
            if column.name == m.ARRANGEMENT and transposition.stores_arrangement(relation):
                continue
            raw = given.get(column.name)
            if raw is None:
                if column.default is not None:
                    converted[column.name] = column.default  # type: ignore[assignment]
                elif column.optional:
                    converted[column.name] = None
                else:
                    problems.add(column.name, "a required value is missing")
                continue
            try:
                converted[column.name] = self._convert(fields, column, raw, converted)
            except ValueRefused as error:
                problems.add(column.name, str(error))
        if problems:
            raise ValidationError(locator, problems.items)
        self._orient_relation(relation, converted, locator)
        for requirement in relation.requires:
            found = invariants.violation(relation.name, requirement, converted)
            if found is not None:
                problems.add(relation.name, found)
        if problems:
            raise ValidationError(locator, problems.items)
        key_values = [converted[key.name] for key in relation.keys]
        record_id = identity.identifier(relation.name, key_values)  # type: ignore[arg-type]
        table = table_name(relation.schema, relation.name)
        row: dict[str, object] = {"id": record_id}
        for field in (*relation.keys, *relation.values):
            row[field.name] = converted[field.name]
        self._put(table, (record_id,), row, locator)
        mode = relation.provenance.mode
        if mode == "own":
            self._register(record_id, relation.name, origins, locator)
        elif origins:
            raise ValidationError(
                locator,
                [
                    f"relation `{name}` has provenance `{mode}`: it registers no record and takes no origin"
                ],
            )
        return record_id

    def _orient_relation(
        self, relation: m.Relation, converted: dict[str, Converted], locator: str
    ) -> None:
        """Store the keys of a transposable relation in the canonical orientation; where the rule
        acts on values, record the arrangement the values were asserted for. No value changes."""
        if relation.transposition is None:
            return
        names = [key.name for key in relation.keys]
        subjects = tuple(converted[name] for name in names)
        if transposition.is_diagonal(relation, subjects):
            raise ValidationError(
                locator,
                [
                    f"relation `{relation.name}` forbids the diagonal: its roles name one instance twice"
                ],
            )
        canonical, _ = transposition.canonical_orientation(relation, subjects)  # type: ignore[type-var]
        converted.update(zip(names, canonical, strict=True))
        if transposition.stores_arrangement(relation):
            converted[m.ARRANGEMENT] = transposition.arrangement_of(relation, subjects, canonical)

    def _check_output_observables(
        self, group: m.SlotGroup, merged: Mapping[str, Converted], locator: str
    ) -> None:
        """For each output of the form's contract whose observable this group's slot supplies,
        the observable the set names is a declared observable whose quantity type has the
        dimension of the output's declared type."""
        decl = self.decl
        form = decl.forms[group.form]
        problems = Problems()
        for supplied in form.output_observables:
            if supplied.slot_group != group.qualified:
                continue
            reference = merged.get(supplied.slot)
            found = self._instance(reference) if isinstance(reference, uuid.UUID) else None
            observable_kind = decl.framework.get(m.OBSERVABLE_ROLE)
            if found is None or observable_kind is None or not decl.is_a(found[0], observable_kind):
                problems.add(supplied.qualified, f"{reference!r} is not a declared observable")
                continue
            entity = self._entities[reference]  # type: ignore[index]
            quantity = next(
                a
                for a in decl.attributes_of(found[0])
                if a.type.element_kind == "meta" and a.type.element == "quantity_type"
            )
            type_name = self._meta_names["quantity_type"][found[1][quantity.name]]  # type: ignore[index]
            named = type_dimension(
                decl,
                TypeRef(
                    container="scalar", element_kind="quantity", element=type_name, text=type_name
                ),
            )
            contract = decl.contracts[form.implements]
            output = next(o for o in contract.outputs if o.name == supplied.output)
            wanted = type_dimension(decl, output.type)
            if named is None or wanted is None or named != wanted:
                problems.add(
                    supplied.qualified,
                    f"observable `{entity.name}` is a {type_name} quantity of dimension "
                    f"{describe(named) if named is not None else 'none'}, but output "
                    f"`{output.name}` of form `{form.name}` is {output.type.text}, of dimension "
                    f"{describe(wanted) if wanted is not None else 'none'}",
                )
        if problems:
            raise ValidationError(locator, problems.items)

    # -- sub-form choices ---------------------------------------------------------------------

    def subform_choice(
        self,
        *,
        parameterization: uuid.UUID,
        slot: str,
        subjects: Sequence[uuid.UUID],
        form: str,
        ordinal: int = 1,
        source_parameterization: uuid.UUID | None = None,
        at: str | None = None,
    ) -> uuid.UUID:
        """Write the choice of `form` for the sub-form slot `slot` (`form.subform`) for `subjects`,
        the subject of each role of the contract the slot accepts, in the parameterization. A slot
        with multiplicity `many` takes several choices, `ordinal` counting them from one.
        `source_parameterization` names the parameterization that holds the chosen form's sets when
        it is not `parameterization`."""
        sc = pc.SUBJECT_SUBFORM_CHOICE
        locator = self._locator((), at)
        declared = next(
            (
                sub
                for candidate in self.decl.forms.values()
                for sub in candidate.subforms
                if sub.qualified == slot
            ),
            None,
        )
        if declared is None:
            raise ValidationError(locator, [f"`{slot}` is not a sub-form slot (`form.subform`)"])
        problems = Problems()
        chosen = self.decl.forms.get(form)
        if chosen is None:
            problems.add(sc.form, f"`{form}` is not a declared form")
        elif chosen.implements != declared.accepts:
            problems.add(
                sc.form,
                f"slot `{slot}` accepts contract `{declared.accepts}`, and form `{form}` "
                f"implements `{chosen.implements}`",
            )
        if declared.per != "subject":
            problems.add(slot, "is chosen per model: a choice for a subject belongs to a slot chosen per subject")
        roles = self.decl.contracts[declared.accepts].roles
        if len(subjects) != len(roles):
            problems.add(
                sc.subject_key,
                f"`{slot}` is chosen for the {len(roles)} role(s) of contract "
                f"`{declared.accepts}`, {len(subjects)} subject(s) given",
            )
        if isinstance(ordinal, bool) or ordinal < 1:
            problems.add(sc.ordinal, f"{ordinal!r} is not a whole number from one")
        elif ordinal > 1 and declared.multiplicity != "many":
            problems.add(
                sc.ordinal, f"`{slot}` has multiplicity `{declared.multiplicity}`: it takes one choice"
            )
        if problems:
            raise ValidationError(locator, problems.items)
        with self._atomic():
            return self._relation(
                sc.declared,
                {
                    sc.parameterization: parameterization,
                    sc.slot: slot,
                    sc.subject_key: identity.canonical_encoding(list(subjects)),
                    sc.ordinal: ordinal,
                },
                {sc.form: form, sc.source_parameterization: source_parameterization},
                at=at,
            )

    # -- validity -----------------------------------------------------------------------------

    def validity_region(
        self,
        record: uuid.UUID,
        region: Mapping[str, object],
        clauses: Sequence[Mapping[str, object]],
        *,
        origins: Sequence[Origin],
        ordinal: int = 1,
        at: str | None = None,
    ) -> uuid.UUID:
        """Write one region of the validity of `record` (a parameter set, a parameterization or a
        model assembly) with its clauses and the coverage row that says the source states regions
        of that kind for it; returns the region's identifier.

        `region` holds the attributes of `validity_region` other than `record` and `ordinal` (its
        `kind`); each of `clauses` those of `region_clause` other than `region` and `ordinal`
        (`observable`, and as stated `component`, `aggregation`, `lower` and `upper`), numbered
        from one in the order given. Several regions of one kind on one record are alternatives and
        are written by separate calls with their own `ordinal`. A region has at least one
        clause."""
        locator = self._locator(origins, at)
        vr, rc = pc.VALIDITY_REGION, pc.REGION_CLAUSE
        problems = Problems()
        for name in (vr.record, vr.ordinal):
            if name in region:
                problems.add(name, "is given by the writer, not in `region`")
        for position, clause in enumerate(clauses, start=1):
            for name in (rc.region, rc.ordinal):
                if name in clause:
                    problems.add(f"clauses[{position}].{name}", "is given by the writer")
        if not clauses:
            problems.add("clauses", "a validity region has at least one clause")
        if problems:
            raise ValidationError(locator, problems.items)
        with self._atomic():
            found = self.kind(
                vr.declared,
                {**region, vr.record: record, vr.ordinal: ordinal},
                origins=origins,
                at=at,
            )
            for position, clause in enumerate(clauses, start=1):
                self.kind(
                    rc.declared,
                    {**clause, rc.region: found, rc.ordinal: position},
                    at=locator,
                )
            self._coverage(record, region[vr.kind], pc.VALIDITY_COVERAGE_STATE.member("stated"), locator)
            return found

    def validity_not_stated(
        self, record: uuid.UUID, kind: object, *, at: str | None = None
    ) -> uuid.UUID:
        """Record that the source gives no region of `kind` (an `envelope_kind` member) for
        `record`: its validity is unknown, not unbounded. Returns the coverage row's identifier."""
        with self._atomic():
            return self._coverage(
                record,
                kind,
                pc.VALIDITY_COVERAGE_STATE.member("not_stated"),
                self._locator((), at),
            )

    def _coverage(self, record: uuid.UUID, kind: object, state: str, locator: str) -> uuid.UUID:
        coverage = pc.VALIDITY_COVERAGE
        return self._relation(
            coverage.declared,
            {coverage.record: record, coverage.kind: kind},
            {coverage.value: state},
            at=locator,
        )

    # -- parameter sets -----------------------------------------------------------------------

    def parameter_set(
        self,
        *,
        parameterization: uuid.UUID,
        slot_group: str,
        subjects: Sequence[uuid.UUID],
        slots: Mapping[str, object],
        families: Mapping[str, Sequence[FamilyRow]] | None = None,
        origins: Sequence[Origin],
        occurrence: int | None = None,
        at: str | None = None,
    ) -> uuid.UUID:
        """Write one parameter set of the slot group `form.group` for the ordered `subjects`.

        The subjects are stored in the canonical orientation and the values as asserted, with the
        `arrangement` recorded where the group's rule acts on values (the rule is applied on
        reading); `subject_key` is the canonical encoding of the subjects, and the set,
        slot-group and family rows and the record are written. `occurrence` (from one, in source
        order) distinguishes repeated assertions for one subject within one parameterisation; it
        takes the declared default, one, when it is not given."""
        with self._atomic():
            return self._parameter_set(
                parameterization=parameterization,
                slot_group=slot_group,
                subjects=subjects,
                slots=slots,
                families=families,
                origins=origins,
                occurrence=occurrence,
                at=at,
            )

    def _parameter_set(
        self,
        *,
        parameterization: uuid.UUID,
        slot_group: str,
        subjects: Sequence[uuid.UUID],
        slots: Mapping[str, object],
        families: Mapping[str, Sequence[FamilyRow]] | None = None,
        origins: Sequence[Origin],
        occurrence: int | None = None,
        at: str | None = None,
        nested: tuple[_Holder, uuid.UUID, str] | None = None,
    ) -> uuid.UUID:
        """Write one set. `nested` is (holder, the slot's reference, the index key) when the set
        is the value of a slot of another set."""
        locator = self._locator(origins, at)
        decl = self.decl
        group = next((g for g in decl.slot_groups if g.qualified == slot_group), None)
        if group is None:
            raise ValidationError(locator, [f"`{slot_group}` is not a slot group (`form.group`)"])
        if len(subjects) != len(group.subjects):
            raise ValidationError(
                locator,
                [
                    f"`{slot_group}` has {len(group.subjects)} subject role(s), {len(subjects)} given"
                ],
            )
        for name in sorted(set(slots) & {s.name for s in group.subjects}):
            raise ValidationError(locator, [f"{name}: is a subject role, not a slot"])
        if m.ARRANGEMENT in slots:
            raise ValidationError(
                locator,
                [f"{m.ARRANGEMENT}: is recorded by the writer from the order of the subjects"],
            )
        ordered = tuple(subjects)
        if transposition.is_diagonal(group, ordered):
            raise ValidationError(
                locator,
                [f"`{slot_group}` forbids the diagonal: its subjects name one instance twice"],
            )
        canonical, _ = transposition.canonical_orientation(group, ordered)
        marker = self._meta_ids["slot_group"][group.qualified]
        if nested is None:
            subject_key = identity.canonical_encoding(list(canonical))
        else:
            holder, slot_reference, index_key = nested
            subject_key = identity.canonical_encoding([holder.set_id, slot_reference, index_key])
        if occurrence is not None and (
            isinstance(occurrence, bool) or not isinstance(occurrence, int) or occurrence < 1
        ):
            raise ValidationError(
                locator, [f"occurrence: {occurrence!r} is not a whole number from one"]
            )
        ps = pc.PARAMETER_SET
        given: dict[str, object] = {
            ps.parameterization: parameterization,
            ps.slot_group: marker,
            ps.subject_key: subject_key,
            ps.occurrence: occurrence,
            **slots,
        }
        root = decl.kinds[decl.kinds[group.id].root]
        set_id = identity.identifier(root.name, self._identity_values(root, given))
        holder_context = _Holder(set_id, parameterization, origins, locator)
        if nested is not None:
            given[ps.parent] = nested[0].set_id
            given[ps.parent_slot] = nested[1]
        problems = Problems()
        for slot in group.slots:
            if slot.shape not in ("nested_set", "set_reference", "tabulated_function"):
                continue
            raw = slots.get(slot.name)
            if raw is None:
                continue  # `_assemble` reports the missing value
            if slot.shape == "nested_set":
                if not isinstance(raw, NestedSet):
                    problems.add(slot.name, "a nested-set slot takes a NestedSet")
                    continue
                given[slot.name] = self._nested(
                    holder_context, f"{group.qualified}.{slot.name}", slot, "", raw
                )
            elif slot.shape == "set_reference":
                if not isinstance(raw, SetReference):
                    problems.add(slot.name, "a set-reference slot takes a SetReference")
                    continue
                given[slot.name] = self._reference(
                    holder_context.locator, f"{group.qualified}.{slot.name}", slot, raw
                )
            else:
                if not isinstance(raw, TabulatedFunction):
                    problems.add(slot.name, "a tabulated-function slot takes a TabulatedFunction")
                    continue
                given[slot.name] = self._tabulated(
                    holder_context, f"{group.qualified}.{slot.name}", slot, "", raw
                )
        if problems:
            raise ValidationError(locator, problems.items)
        for field, subject in zip(group.subjects, canonical, strict=True):
            given[field.name] = subject
        if transposition.stores_arrangement(group):
            given[m.ARRANGEMENT] = transposition.arrangement_of(group, ordered, canonical)
        merged = self._assemble(group.id, given, locator)
        self._check_output_observables(group, merged, locator)
        family_rows = self._family_rows(group, families or {}, locator, holder_context)
        written = self._emit_kind(group.id, merged, origins, locator, slot_group=marker)
        assert written == set_id, "a set's identifier does not depend on its values"
        for family, rows in family_rows:
            table = table_name(m.PARAM_SCHEMA, family.id)
            for row in rows:
                index_key_values = tuple(row[index.name] for index in family.indices)
                self._put(table, (set_id, *index_key_values), {"set_id": set_id, **row}, locator)
        return set_id

    def _identity_values(
        self, root: m.Kind, values: Mapping[str, object]
    ) -> list[identity.IdentityValue]:
        """The identity values of an instance of the refinement chain rooted at `root`, in
        declared order: an identity attribute not given takes its declared default, as it does
        in the row."""
        found: list[identity.IdentityValue] = []
        attributes = {attribute.name: attribute for attribute in root.attributes}
        for name in root.identity:
            value = values.get(name)
            if value is None:
                value = attributes[name].default
            if value is None:
                raise WriteError(
                    f"internal error: identity attribute `{name}` of `{root.name}` has no value"
                )
            found.append(value)  # type: ignore[arg-type]
        return found

    def _nested(
        self,
        holder: _Holder,
        slot_qualified: str,
        slot: m.Field,
        index_key: str,
        value: NestedSet,
    ) -> uuid.UUID:
        """Write the set that is the value of `slot` of the set `holder`, after checking that its
        form implements the contract the slot accepts."""
        decl = self.decl
        group = next((g for g in decl.slot_groups if g.qualified == value.slot_group), None)
        if group is None:
            raise ValidationError(
                holder.locator, [f"`{value.slot_group}` is not a slot group (`form.group`)"]
            )
        form = decl.forms[group.form]
        if form.implements != slot.accepts:
            raise ValidationError(
                holder.locator,
                [
                    f"{slot_qualified}: accepts contract `{slot.accepts}`, and form `{form.name}` "
                    f"of `{value.slot_group}` implements `{form.implements}`"
                ],
            )
        return self._parameter_set(
            parameterization=holder.parameterization,
            slot_group=value.slot_group,
            subjects=value.subjects,
            slots=value.slots,
            families=value.families,
            origins=holder.origins,
            at=holder.locator,
            nested=(holder, self._meta_ids["slot"][slot_qualified], index_key),
        )

    def _reference(
        self, locator: str, slot_qualified: str, slot: m.Field, value: SetReference
    ) -> uuid.UUID:
        """The identifier of the set a slot references, after checking that the form of the set's
        slot group implements the contract the slot names. A reference names a top-level set by
        the identity it is written under, so it can never name a nested set (whose identity is
        that of its holder); the verify check `referenced_set_implements_contract` holds the
        database to the same rule."""
        decl = self.decl
        group = next((g for g in decl.slot_groups if g.qualified == value.slot_group), None)
        if group is None:
            raise ValidationError(
                locator, [f"`{value.slot_group}` is not a slot group (`form.group`)"]
            )
        form = decl.forms[group.form]
        if form.implements != slot.references:
            raise ValidationError(
                locator,
                [
                    f"{slot_qualified}: references a set of a form implementing contract "
                    f"`{slot.references}`, and form `{form.name}` of `{value.slot_group}` "
                    f"implements `{form.implements}`"
                ],
            )
        if len(value.subjects) != len(group.subjects):
            raise ValidationError(
                locator,
                [
                    f"{slot_qualified}: `{value.slot_group}` has {len(group.subjects)} subject "
                    f"role(s), the reference gives {len(value.subjects)}"
                ],
            )
        if isinstance(value.occurrence, bool) or value.occurrence < 1:
            raise ValidationError(
                locator,
                [f"{slot_qualified}: occurrence {value.occurrence!r} is not a whole number from one"],
            )
        ordered = tuple(value.subjects)
        if transposition.is_diagonal(group, ordered):
            raise ValidationError(
                locator,
                [
                    f"{slot_qualified}: `{value.slot_group}` forbids the diagonal: the subjects "
                    "name one instance twice"
                ],
            )
        canonical, _ = transposition.canonical_orientation(group, ordered)
        ps = pc.PARAMETER_SET
        root = decl.kinds[decl.kinds[group.id].root]
        return identity.identifier(
            root.name,
            self._identity_values(
                root,
                {
                    ps.parameterization: value.parameterization,
                    ps.slot_group: self._meta_ids["slot_group"][group.qualified],
                    ps.subject_key: identity.canonical_encoding(list(canonical)),
                    ps.occurrence: value.occurrence,
                },
            ),
        )

    def _tabulated(
        self,
        holder: _Holder,
        slot_qualified: str,
        slot: m.Field,
        index_key: str,
        value: TabulatedFunction,
    ) -> uuid.UUID:
        """Write the tabulated function that is the value of `slot` of the set `holder`, with its
        axes and series, after checking the grid: every axis has points in strictly ascending
        order and every series has one value for each point of the product of the axes."""
        problems = Problems()
        if not value.axes:
            problems.add(slot_qualified, "a tabulated function has at least one axis")
        if not value.series:
            problems.add(slot_qualified, "a tabulated function has at least one series")
        lengths: list[int] = []
        for position, axis in enumerate(value.axes, start=1):
            where = f"{slot_qualified}.axes[{position}]"
            points = list(axis.points.values)
            lengths.append(len(points))
            if not points:
                problems.add(where, "an axis has at least one point")
            elif any(later <= earlier for earlier, later in zip(points, points[1:], strict=False)):
                problems.add(where, "the axis points are not strictly ascending")
        grid = math.prod(lengths)
        names: set[str] = set()
        for series in value.series:
            where = f"{slot_qualified}.series[{series.name}]"
            if series.name in names:
                problems.add(where, "the series name is repeated")
            names.add(series.name)
            found = len(series.values.values)
            if lengths and found != grid:
                shape = " x ".join(str(length) for length in lengths)
                problems.add(
                    where, f"has {found} values, the grid of the axes ({shape}) has {grid} points"
                )
        if problems:
            raise ValidationError(holder.locator, problems.items)
        tf, axis_kind, series_kind = pc.TABULATED_FUNCTION, pc.TABULATED_AXIS, pc.TABULATED_SERIES
        function = self.kind(
            slot.type.element,
            {
                tf.key: identity.canonical_encoding(
                    [holder.set_id, self._meta_ids["slot"][slot_qualified], index_key]
                ),
                tf.interpolation: value.interpolation,
                tf.distribution: value.distribution,
            },
            origins=holder.origins,
            at=holder.locator,
        )
        for position, axis in enumerate(value.axes, start=1):
            self.kind(
                axis_kind.declared,
                {
                    axis_kind.function: function,
                    axis_kind.ordinal: position,
                    axis_kind.axis_type: axis.quantity_type,
                    axis_kind.points: axis.points,
                },
                at=holder.locator,
            )
        for series in value.series:
            self.kind(
                series_kind.declared,
                {
                    series_kind.function: function,
                    series_kind.name: series.name,
                    series_kind.value_type: series.quantity_type,
                    series_kind.values: series.values,
                },
                at=holder.locator,
            )
        return function

    def _family_rows(
        self,
        group: m.SlotGroup,
        given: Mapping[str, Sequence[FamilyRow]],
        locator: str,
        holder: _Holder,
    ) -> list[tuple[m.Family, list[dict[str, object]]]]:
        problems = Problems()
        known = {family.name: family for family in group.families}
        for name in sorted(set(given) - set(known)):
            problems.add(name, f"is not a family of `{group.qualified}`")
        result: list[tuple[m.Family, list[dict[str, object]]]] = []
        for family in group.families:
            rows: list[dict[str, object]] = []
            seen: set[tuple[object, ...]] = set()
            for position, item in enumerate(given.get(family.name, ())):
                where = f"{family.qualified}[{position}]"
                row: dict[str, object] = {}
                for unknown in sorted(set(item.index) - {i.name for i in family.indices}):
                    problems.add(where, f"`{unknown}` is not an index of the family")
                for unknown in sorted(set(item.values) - {s.name for s in family.slots}):
                    problems.add(where, f"`{unknown}` is not a slot of the family")
                for index in family.indices:
                    raw = item.index.get(index.name)
                    if raw is None:
                        problems.add(where, f"index `{index.name}` is missing")
                        continue
                    try:
                        row[index.name] = v.scalar(self.decl, index.type, raw)
                    except ValueRefused as error:
                        problems.add(f"{where}.{index.name}", str(error))
                        continue
                    if index.minimum is not None and row[index.name] < index.minimum:  # type: ignore[operator]
                        problems.add(
                            f"{where}.{index.name}",
                            f"{row[index.name]!r} is below the minimum {index.minimum}",
                        )
                for slot in family.slots:
                    raw = item.values.get(slot.name)
                    if raw is None:
                        problems.add(where, f"slot `{slot.name}` is missing")
                        continue
                    if slot.shape == "nested_set":
                        if not isinstance(raw, NestedSet):
                            problems.add(
                                f"{where}.{slot.name}", "a nested-set slot takes a NestedSet"
                            )
                        elif all(index.name in row for index in family.indices):
                            row[slot.name] = self._nested(
                                holder,
                                f"{family.qualified}.{slot.name}",
                                slot,
                                identity.canonical_encoding([row[i.name] for i in family.indices]),  # type: ignore[misc]
                                raw,
                            )
                        continue
                    if slot.shape == "set_reference":
                        if not isinstance(raw, SetReference):
                            problems.add(
                                f"{where}.{slot.name}", "a set-reference slot takes a SetReference"
                            )
                        else:
                            row[slot.name] = self._reference(
                                holder.locator, f"{family.qualified}.{slot.name}", slot, raw
                            )
                        continue
                    if slot.shape == "tabulated_function":
                        if not isinstance(raw, TabulatedFunction):
                            problems.add(
                                f"{where}.{slot.name}",
                                "a tabulated-function slot takes a TabulatedFunction",
                            )
                        elif all(index.name in row for index in family.indices):
                            row[slot.name] = self._tabulated(
                                holder,
                                f"{family.qualified}.{slot.name}",
                                slot,
                                identity.canonical_encoding([row[i.name] for i in family.indices]),  # type: ignore[misc]
                                raw,
                            )
                        continue
                    try:
                        row[slot.name] = v.scalar(self.decl, slot.type, raw)
                    except ValueRefused as error:
                        problems.add(f"{where}.{slot.name}", str(error))
                if len(row) != len(family.indices) + len(family.slots):
                    continue
                key = tuple(row[index.name] for index in family.indices)
                if key in seen:
                    problems.add(where, f"the index {key} is repeated")
                seen.add(key)
                rows.append(row)
            if family.interval is not None:
                self._check_intervals(family, rows, problems)
            result.append((family, rows))
        if problems:
            raise ValidationError(locator, problems.items)
        return result

    @staticmethod
    def _check_intervals(
        family: m.Family, rows: list[dict[str, object]], problems: Problems
    ) -> None:
        assert family.interval is not None
        lower_name, upper_name = family.interval
        spans: list[tuple[float, float]] = []
        for row in rows:
            lower, upper = row[lower_name], row[upper_name]
            if not lower < upper:  # type: ignore[operator]
                problems.add(
                    family.qualified, f"{lower_name} {lower!r} is not below {upper_name} {upper!r}"
                )
                continue
            spans.append((lower, upper))  # type: ignore[arg-type]
        spans.sort()
        for (_, end), (start, _) in zip(spans, spans[1:], strict=False):
            if start < end:
                problems.add(family.qualified, "pieces overlap on the interval axis")
                break


def _array(values: list[object], type_: pa.DataType) -> pa.Array:
    """An Arrow array of `type_`; identifiers go through the canonical `uuid` extension type."""
    if type_ == UUID:
        storage = pa.array(
            [None if item is None else item.bytes for item in values],  # type: ignore[attr-defined]
            pa.binary(16),
        )
        return pa.ExtensionArray.from_storage(UUID, storage)
    return pa.array(values, type=type_)


__all__ = [
    "CanonicalWriter",
    "CarrierInfo",
    "CompetingAssertion",
    "FamilyRow",
    "MissingOrigin",
    "NestedSet",
    "Origin",
    "SetReference",
    "SourceRef",
    "TabulatedAxis",
    "TabulatedFunction",
    "TabulatedSeries",
    "ValidationError",
    "WriteError",
]
