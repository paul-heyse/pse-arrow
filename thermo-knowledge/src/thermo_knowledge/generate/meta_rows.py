# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Rows of the `meta` tables for a loaded declaration.

Each row is a mapping from column to value; `meta_rows` returns them as tuples in the column
order of `META_TABLES`, so a column added to a table cannot silently fall out of the rows.
"""

from __future__ import annotations

from collections.abc import Callable, Iterable
import uuid
from datetime import date, datetime

from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.types import TypeRef
from thermo_knowledge.expression.canonical import content_hash, evaluation_hash, residual_hash
from thermo_knowledge.generate import naming
from thermo_knowledge.generate.meta_tables import META_TABLES

type Row = dict[str, object]
type Add = Callable[..., None]
type Mark = Callable[[str, tuple[str, ...], str | None], None]
type Ids = dict[str, dict[str, uuid.UUID]]


def default_text(value: m.Scalar | None) -> str | None:
    """A default or absence value as canonical text."""
    if value is None:
        return None
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, bytes):
        return value.hex()
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, date):
        return value.isoformat()
    return str(value) if not isinstance(value, float) else repr(value)


def _type(type_: TypeRef) -> Row:
    return {
        "container": type_.container,
        "element_kind": type_.element_kind,
        "element": type_.element,
    }


def _observable_id(decl: m.Declaration, name: str | None) -> uuid.UUID | None:
    """The identifier of the observable entity `name` names (validated by the loader)."""
    if name is None:
        return None
    entity = decl.observable_entity(name)
    assert entity is not None, f"observable {name!r} is not a declared entity"
    return entity.id


def _basis_id(decl: m.Declaration, name: str | None) -> uuid.UUID | None:
    """The identifier of the composition basis entity `name` names (validated by the loader)."""
    if name is None:
        return None
    entity = decl.composition_basis_entity(name)
    assert entity is not None, f"composition basis {name!r} is not a declared entity"
    return entity.id


def _requirements(
    add: Add, mark: Mark, owner_type: str, owner: str, requires: tuple[m.Requirement, ...]
) -> None:
    for requirement in requires:
        add(
            "requirement",
            owner_type=owner_type,
            owner=owner,
            name=requirement.name,
            enforced=requirement.enforced,
            check_rule=requirement.rule,
            check_lower=requirement.lower,
            check_upper=requirement.upper,
            doc=requirement.doc,
        )
        mark(requirement.construct, requirement.traces, requirement.pse)
        for position, name in enumerate(requirement.attributes, start=1):
            add(
                "requirement_attribute",
                owner_type=owner_type,
                owner=owner,
                requirement=requirement.name,
                position=position,
                attribute=name,
            )
        for position, name in enumerate(requirement.members, start=1):
            add(
                "requirement_member",
                owner_type=owner_type,
                owner=owner,
                requirement=requirement.name,
                position=position,
                member=name,
            )


def meta_rows(decl: m.Declaration) -> dict[str, list[tuple[object, ...]]]:
    """Every `meta` row of `decl`, by table name, in column order."""
    rows: dict[str, list[Row]] = {name: [] for name in META_TABLES}

    def add(table: str, **row: object) -> None:
        rows[table].append(row)

    traced: list[tuple[str, tuple[str, ...], str | None]] = []

    def mark(construct: str, traces: tuple[str, ...], pse: str | None) -> None:
        traced.append((construct, traces, pse))

    for module in decl.modules.values():
        add(
            "module",
            name=module.name,
            schema_name=module.schema,
            doc=module.doc,
            document=module.document,
        )
    for module in decl.modules.values():
        for used in module.uses:
            add("module_use", module=module.name, used=used)
    for role, kind in sorted(decl.framework.items()):
        add("framework_role", role=role, kind=kind)
    for info in decl.units.values():
        add("unit", unit=info.unit)
        for dimension, exponent in info.dimensions:
            add("unit_dimension", unit=info.unit, dimension=dimension, exponent=exponent)
    for expression, unit in decl.expressions.items():
        add("quantity_expression", expression=expression, unit=unit)
    ids = decl.meta_ids()
    for quantity in decl.quantity_types.values():
        add(
            "quantity_type",
            id=ids["quantity_type"][quantity.name],
            name=quantity.name,
            module=quantity.module,
            unit=quantity.unit,
            scale=quantity.scale,
            production=quantity.production,
            pg_type=naming.quantity_domain(quantity.name),
            doc=quantity.doc,
        )
        mark(quantity.construct, quantity.traces, quantity.pse)
    for scheme in decl.schemes.values():
        add(
            "identifier_scheme",
            name=scheme.name,
            module=scheme.module,
            pg_type=naming.scheme_domain(scheme.name),
            doc=scheme.doc,
        )
        mark(scheme.construct, scheme.traces, scheme.pse)
    for enum in decl.enums.values():
        add("enum", name=enum.name, module=enum.module, pg_type=enum.name, doc=enum.doc)
        mark(enum.construct, enum.traces, enum.pse)
        for ordinal, facet in enumerate(enum.facets, start=1):
            add("enum_facet", enum=enum.name, name=facet.name, ordinal=ordinal, doc=facet.doc)
            mark(facet.construct, facet.traces, facet.pse)
        for ordinal, member in enumerate(enum.members, start=1):
            add("enum_member", enum=enum.name, name=member.name, ordinal=ordinal, doc=member.doc)
            mark(member.construct, member.traces, member.pse)
            for facet_name in member.facets:
                add("enum_member_facet", enum=enum.name, member=member.name, facet=facet_name)
    for kind in decl.kinds.values():
        add(
            "kind",
            id=ids["kind"][kind.name],
            name=kind.name,
            module=kind.module,
            origin=kind.origin,
            extends=kind.extends,
            root=kind.root,
            abstract=kind.abstract,
            provenance=kind.provenance.mode,
            provenance_attribute=kind.provenance.attribute,
            schema_name=kind.schema,
            table_name=kind.name,
            doc=kind.doc,
        )
        if kind.origin == "declared":
            mark(kind.construct, kind.traces, kind.pse)
    for kind in decl.kinds.values():
        for position, attribute in enumerate(kind.attributes, start=1):
            add(
                "attribute",
                kind=kind.name,
                name=attribute.name,
                position=position,
                **_type(attribute.type),
                optional=attribute.optional,
                is_unique=attribute.unique,
                default_value=default_text(attribute.default),
                unit_from=attribute.unit_from,
                doc=attribute.doc,
            )
            if kind.origin == "declared":
                mark(attribute.construct, attribute.traces, attribute.pse)
        for position, name in enumerate(kind.identity, start=1):
            add("kind_identity", kind=kind.name, position=position, attribute=name)
        for unique in kind.uniques:
            add("kind_unique", kind=kind.name, name=unique.name)
            for position, name in enumerate(unique.attributes, start=1):
                add(
                    "kind_unique_attribute",
                    kind=kind.name,
                    unique_name=unique.name,
                    position=position,
                    attribute=name,
                )
        _requirements(add, mark, "kind", kind.name, kind.requires)
    for relation in decl.relations.values():
        add(
            "relation",
            name=relation.name,
            module=relation.module,
            provenance=relation.provenance.mode,
            provenance_key=relation.provenance.attribute,
            absence=relation.absence,
            absence_default=default_text(relation.absence_default),
            schema_name=relation.schema,
            table_name=relation.name,
            doc=relation.doc,
        )
        mark(relation.construct, relation.traces, relation.pse)
        for position, key in enumerate(relation.keys, start=1):
            add(
                "relation_key",
                relation=relation.name,
                name=key.name,
                position=position,
                **_type(key.type),
                doc=key.doc,
            )
            mark(key.construct, key.traces, key.pse)
        for position, value in enumerate(relation.values, start=1):
            add(
                "relation_value",
                relation=relation.name,
                name=value.name,
                position=position,
                **_type(value.type),
                is_single=relation.single_value,
                optional=value.optional,
                default_value=default_text(value.default),
                unit_from=value.unit_from,
                doc=value.doc,
            )
            mark(value.construct, value.traces, value.pse)
        if relation.transposition is not None:
            _transposition(add, "relation", relation.name, relation.transposition)
        _requirements(add, mark, "relation", relation.name, relation.requires)
    for contract in decl.contracts.values():
        add(
            "contract",
            id=ids["contract"][contract.name],
            name=contract.name,
            module=contract.module,
            doc=contract.doc,
        )
        mark(contract.construct, contract.traces, contract.pse)
        for table, subjects in (("contract_role", contract.roles), ("contract_set", contract.sets)):
            for position, subject in enumerate(subjects, start=1):
                add(
                    table,
                    contract=contract.name,
                    name=subject.name,
                    position=position,
                    kind=subject.type.element,
                    doc=subject.doc,
                )
                mark(subject.construct, subject.traces, subject.pse)
        for position, argument in enumerate(contract.arguments, start=1):
            add(
                "contract_argument",
                contract=contract.name,
                name=argument.name,
                position=position,
                **_type(argument.type),
                basis=_basis_id(decl, argument.basis),
                observable=_observable_id(decl, argument.observable),
                doc=argument.doc,
            )
            mark(argument.construct, argument.traces, argument.pse)
            for index, set_name in enumerate(argument.over, start=1):
                add(
                    "contract_argument_set",
                    contract=contract.name,
                    argument=argument.name,
                    position=index,
                    set_name=set_name,
                )
        for position, output in enumerate(contract.outputs, start=1):
            add(
                "contract_output",
                contract=contract.name,
                name=output.name,
                position=position,
                **_type(output.type),
                observable=_observable_id(decl, output.observable),
                observable_from_set=output.observable_from_set,
                doc=output.doc,
            )
            mark(output.construct, output.traces, output.pse)
    for form in decl.forms.values():
        add(
            "form",
            id=ids["form"][form.name],
            name=form.name,
            module=form.module,
            implements=form.implements,
            completeness=form.completeness,
            status=form.status,
            doc=form.doc,
        )
        mark(form.construct, form.traces, form.pse)
        for position, citation in enumerate(form.citations, start=1):
            add("form_citation", form=form.name, position=position, citation=citation)
        for position, local in enumerate(form.locals, start=1):
            add(
                "form_local",
                form=form.name,
                name=local.name,
                position=position,
                expression=local.text,
                content_hash=content_hash(local.text),
            )
        for position, output in enumerate(form.outputs, start=1):
            add(
                "form_output",
                form=form.name,
                contract=form.implements,
                name=output.name,
                position=position,
                expression=output.text,
                content_hash=content_hash(output.text),
                evaluation_hash=evaluation_hash(form, output.name),
            )
        for supplied in form.output_observables:
            add(
                "form_output_observable",
                form=form.name,
                contract=form.implements,
                output=supplied.output,
                slot=supplied.qualified,
            )
        kind = decl.framework.get(m.CONVENTION_SET_ROLE)
        for position, convention in enumerate(form.conventions, start=1):
            assert kind is not None
            declaring = next(
                link.name
                for link in decl.chain(kind)
                if any(a.name == convention.name for a in link.attributes)
            )
            add(
                "form_convention",
                form=form.name,
                name=convention.name,
                position=position,
                kind=declaring,
            )
        for position, block in enumerate(form.implicit, start=1):
            _implicit_block(add, mark, form, position, block)
        for position, subform in enumerate(form.subforms, start=1):
            add(
                "subform_slot",
                id=ids["subform_slot"][subform.qualified],
                qualified_name=subform.qualified,
                form=form.name,
                name=subform.name,
                position=position,
                accepts=subform.accepts,
                multiplicity=subform.multiplicity,
                per=subform.per,
                doc=subform.doc,
            )
            mark(subform.construct, subform.traces, subform.pse)
        for group in form.slot_groups:
            _slot_group(add, mark, decl, ids, group)
    for entity in decl.entities:
        add("entity", kind=entity.kind, name=entity.name, id=entity.id, doc=entity.doc)
        mark(entity.construct, entity.traces, entity.pse)
    for construct, traces, pse in traced:
        for trace in traces:
            add("trace", construct=construct, trace=trace)
        if pse is not None:
            add("pse_mark", construct=construct, mark=pse)
    return {name: _tuples(name, table_rows) for name, table_rows in rows.items()}


def _implicit_block(
    add: Add, mark: Mark, form: m.Form, position: int, block: m.ImplicitBlock
) -> None:
    add(
        "form_implicit",
        form=form.name,
        name=block.name,
        position=position,
        select_rule=block.select,
        select_expression=block.select_by,
        select_hash=None if block.select_by is None else content_hash(block.select_by),
        doc=block.doc,
    )
    mark(block.construct, block.traces, block.pse)
    for index, unknown in enumerate(block.unknowns, start=1):
        add(
            "form_unknown",
            form=form.name,
            block=block.name,
            name=unknown.name,
            position=index,
            **_type(unknown.type),
        )
        mark(unknown.construct, unknown.traces, unknown.pse)
        for at, set_name in enumerate(unknown.over, start=1):
            add(
                "form_unknown_set",
                form=form.name,
                contract=form.implements,
                block=block.name,
                unknown=unknown.name,
                position=at,
                set_name=set_name,
            )
        for label in ("lower", "upper", "start"):
            bound = getattr(unknown, label)
            if bound is None:
                continue
            add(
                "form_unknown_bound",
                form=form.name,
                block=block.name,
                unknown=unknown.name,
                bound=label,
                number=bound.number,
                expression=bound.text,
                content_hash=None if bound.text is None else content_hash(bound.text),
            )
    for index, residual in enumerate(block.residuals, start=1):
        add(
            "form_residual",
            form=form.name,
            block=block.name,
            position=index,
            expression=residual.text,
            content_hash=residual_hash(residual.text),
        )


def _transposition(add: Add, owner_type: str, owner: str, transposition: m.Transposition) -> None:
    add(
        "transposition",
        owner_type=owner_type,
        owner=owner,
        rule=transposition.rule,
        diagonal=transposition.diagonal,
        by_index=transposition.by,
    )
    for position, role in enumerate(transposition.roles, start=1):
        add("transposition_role", owner_type=owner_type, owner=owner, position=position, role=role)
    for position, slot in enumerate(transposition.slots, start=1):
        add("transposition_slot", owner_type=owner_type, owner=owner, position=position, slot=slot)
    for row, entries in enumerate(transposition.matrix, start=1):
        for column, coefficient in enumerate(entries, start=1):
            add(
                "transposition_matrix",
                owner_type=owner_type,
                owner=owner,
                row_position=row,
                column_position=column,
                coefficient=float(coefficient),
            )
    for number, permutation in enumerate(transposition.permutations, start=1):
        for position, role in enumerate(permutation, start=1):
            add(
                "transposition_permutation",
                owner_type=owner_type,
                owner=owner,
                permutation=number,
                position=position,
                role=role,
            )


def _slot_group(add: Add, mark: Mark, decl: m.Declaration, ids: Ids, group: m.SlotGroup) -> None:
    add(
        "slot_group",
        id=ids["slot_group"][group.qualified],
        qualified_name=group.qualified,
        form=group.form,
        name=group.name,
        kind=group.id,
        table_name=group.id,
        doc=group.doc,
    )
    mark(group.construct, group.traces, group.pse)
    for position, (subject, binding) in enumerate(zip(group.subjects, group.bindings), start=1):
        add(
            "slot_group_subject",
            slot_group=group.qualified,
            name=subject.name,
            position=position,
            kind=subject.type.element,
            binds_to_type=binding.kind,
            binds_to=binding.target,
            doc=subject.doc,
        )
        mark(subject.construct, subject.traces, subject.pse)
    if group.transposition is not None:
        _transposition(add, "slot_group", group.qualified, group.transposition)
    for position, slot in enumerate(group.slots, start=1):
        _slot(
            add,
            mark,
            decl,
            ids,
            group.qualified,
            None,
            f"{group.qualified}.{slot.name}",
            position,
            slot,
        )
    for family in group.families:
        lower, upper = family.interval if family.interval else (None, None)
        add(
            "family",
            id=ids["family"][family.qualified],
            qualified_name=family.qualified,
            slot_group=group.qualified,
            name=family.name,
            table_name=family.id,
            interval_lower=lower,
            interval_upper=upper,
            doc=family.doc,
        )
        mark(family.construct, family.traces, family.pse)
        for position, index in enumerate(family.indices, start=1):
            add(
                "family_index",
                family=family.qualified,
                name=index.name,
                position=position,
                element_kind=index.type.element_kind,
                element=index.type.element,
                minimum=index.minimum,
                doc=index.doc,
            )
            mark(index.construct, index.traces, index.pse)
        for position, slot in enumerate(family.slots, start=1):
            _slot(
                add,
                mark,
                decl,
                ids,
                group.qualified,
                family.qualified,
                f"{family.qualified}.{slot.name}",
                position,
                slot,
            )


def _slot(
    add: Add,
    mark: Mark,
    decl: m.Declaration,
    ids: Ids,
    group: str,
    family: str | None,
    qualified: str,
    position: int,
    slot: m.Field,
) -> None:
    held_set = slot.shape in ("nested_set", "set_reference")
    add(
        "slot",
        id=ids["slot"][qualified],
        qualified_name=qualified,
        slot_group=group,
        family=family,
        name=slot.name,
        position=position,
        shape=slot.shape,
        element_kind=None if held_set else slot.type.element_kind,
        element=None if held_set else slot.type.element,
        accepts=slot.accepts,
        references_contract=slot.references,
        presence=slot.presence,
        observable=_observable_id(decl, slot.observable),
        doc=slot.doc,
    )
    mark(slot.construct, slot.traces, slot.pse)


def _tuples(table: str, rows: Iterable[Row]) -> list[tuple[object, ...]]:
    columns = [column.name for column in META_TABLES[table].columns]
    result: list[tuple[object, ...]] = []
    for row in rows:
        unknown = set(row) - set(columns)
        missing = set(columns) - set(row)
        if unknown or missing:
            raise AssertionError(
                f"meta.{table}: row columns disagree with the table "
                f"(unknown {sorted(unknown)}, missing {sorted(missing)})"
            )
        result.append(tuple(row[column] for column in columns))
    return result
