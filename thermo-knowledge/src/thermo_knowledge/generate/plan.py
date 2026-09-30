# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Projection of a loaded declaration to PostgreSQL (meta-model section 7).

`build_plan` describes every object the DDL creates; `render_schema` writes it as one SQL
file ordered to apply to an empty database: schemas, the extension, types and domains, then
every table, then every foreign key (deferrable, added last so tables may refer to each other
in any order). `projection_diagnostics` reports identifiers over PostgreSQL's limit and names
that would collide.
"""

from __future__ import annotations

import dataclasses
from dataclasses import dataclass, field
from itertools import combinations

from thermo_knowledge import transposition as transposition_module
from thermo_knowledge.declaration import model as m
from thermo_knowledge.declaration.diagnostics import Code, Diagnostic
from thermo_knowledge.declaration.resolve import MANIFEST_DOCUMENT
from thermo_knowledge.declaration.types import TypeRef
from thermo_knowledge.generate import ir, naming
from thermo_knowledge.generate.meta_tables import META_TABLES
from thermo_knowledge.generate.sqltext import MAX_IDENTIFIER_BYTES, ident, literal, qname

SCHEMA_DOCS: dict[str, str] = {
    "meta": "The declaration reified as rows, and the types every table uses.",
    "prov": "Provenance: the record registry and pipeline bookkeeping.",
    "tk": "Canonical entities, systems, conventions, observables and forms.",
    "ev": "Datasets and data points.",
    "qual": "Qualification runs, residuals, equivalence assessments and the capability inventory.",
    "param": "One table per slot group, and one per family of slots.",
}
GENERATED_SCHEMAS: tuple[str, ...] = tuple(SCHEMA_DOCS)
"""The schemas the generated DDL creates, in creation order."""

META_ENTITY_COLUMNS: tuple[tuple[str, str, tuple[str, ...]], ...] = (
    (m.OBSERVABLE_ROLE, "observable", ("contract_argument", "contract_output", "slot")),
    (m.COMPOSITION_BASIS_ROLE, "basis", ("contract_argument",)),
)
"""Columns of `meta` tables that hold the identifier of a declared entity of a framework role's
kind: (role, column, tables)."""

_FINITE = (
    "NOT IN ('NaN'::double precision, 'Infinity'::double precision, '-Infinity'::double precision)"
)


@dataclass
class Plan:
    needs_btree_gist: bool = False
    types: list[ir.TypeDef] = field(default_factory=list)
    tables: list[ir.Table] = field(default_factory=list)


def _domain(
    name: str,
    base: str,
    comment: str,
    checks: list[tuple[str, str]],
    *,
    construct: str,
    module: str | None = None,
) -> ir.TypeDef:
    target = qname("meta", name)
    body = " ".join(
        f"CONSTRAINT {ident(f'{name}__{suffix}')} CHECK ({expression})"
        for suffix, expression in checks
    )
    return ir.TypeDef(
        object_kind="DOMAIN",
        schema="meta",
        name=name,
        create=f"CREATE DOMAIN {target} AS {base} {body};",
        comment=comment,
        construct=construct,
        module=module,
    )


def _fixed_types() -> list[ir.TypeDef]:
    bounds = " AND ".join(
        f"{f}(VALUE) {op} {literal(limit)}::double precision"
        for f in ("lower", "upper")
        for op, limit in ((">", "-Infinity"), ("<", "Infinity"))
    )
    return [
        ir.TypeDef(
            object_kind="TYPE",
            schema="meta",
            name="float8range",
            create=f"CREATE TYPE {qname('meta', 'float8range')} AS RANGE (subtype = double precision);",
            comment="A range over double precision.",
            construct="meta.float8range",
        ),
        _domain(
            "closed_interval",
            qname("meta", "float8range"),
            "A non-empty closed interval with finite bounds: Range<Q> columns.",
            [
                (
                    "closed",
                    "NOT isempty(VALUE) AND NOT lower_inf(VALUE) AND NOT upper_inf(VALUE) "
                    f"AND lower_inc(VALUE) AND upper_inc(VALUE) AND {bounds}",
                )
            ],
            construct="meta.closed_interval",
        ),
        _domain(
            "hash",
            "bytea",
            "A 32-byte content hash.",
            [("length", "octet_length(VALUE) = 32")],
            construct="meta.hash",
        ),
        _domain(
            "finite_real",
            "double precision",
            "A finite double precision number: Real and quantity expressions.",
            [("finite", f"VALUE {_FINITE}")],
            construct="meta.finite_real",
        ),
    ]


def sql_type(decl: m.Declaration, type_: TypeRef) -> str:
    """The PostgreSQL type of a column holding `type_` (arrays and ranges included)."""
    kind = type_.element_kind
    element: str
    if type_.container == "range":
        return qname("meta", "closed_interval")
    if kind == "primitive":
        element = {
            "Boolean": "boolean",
            "Integer": "bigint",
            "Text": "text",
            "Date": "date",
            "Timestamp": "timestamptz",
            "Hash": qname("meta", "hash"),
        }[type_.element]
    elif kind == "identifier":
        element = qname("meta", naming.scheme_domain(type_.element))
    elif kind == "quantity":
        element = qname("meta", naming.quantity_domain(type_.element))
    elif kind in ("expression", "real"):
        element = qname("meta", "finite_real")
    elif kind == "enum":
        element = qname("meta", type_.element)
    elif kind in ("kind", "record", "meta"):
        element = "uuid"
    else:  # source_text
        element = "text"
    return f"{element}[]" if type_.container == "array" else element


def _reference_target(decl: m.Declaration, type_: TypeRef) -> tuple[str, str] | None:
    """The table a reference-typed column points to, as (schema, table)."""
    if type_.container == "array" or type_.container == "range":
        return None
    if type_.element_kind == "kind":
        return decl.kinds[type_.element].schema, type_.element
    if type_.element_kind == "record":
        return "prov", "record"
    if type_.element_kind == "meta":
        return "meta", type_.element
    return None


def _literal(value: m.Scalar) -> str:
    if isinstance(value, bytes):
        return f"{literal(value)}"
    return literal(value)


def _transposition_constraints(
    table_name: str, transposition: m.Transposition, construct: str
) -> list[ir.Constraint]:
    roles = transposition.roles
    result: list[ir.Constraint] = []
    if transposition.rule == "ordered":
        pass
    elif transposition.rule == "permutation_group":
        images = transposition_module.arrangements(roles, transposition.permutations)[1:]
        if images:
            first = ", ".join(ident(r) for r in roles)
            expression = " AND ".join(
                f"ROW({first}) <= ROW({', '.join(ident(r) for r in image)})" for image in images
            )
            result.append(
                ir.Constraint(
                    name=f"{table_name}__ck__canonical",
                    body=ir.check(expression),
                    construct=f"{construct}.transposition",
                )
            )
    else:
        result.append(
            ir.Constraint(
                name=f"{table_name}__ck__canonical",
                body=ir.check(f"{ident(roles[0])} <= {ident(roles[1])}"),
                construct=f"{construct}.transposition",
            )
        )
    if transposition.diagonal == "forbidden":
        expression = " AND ".join(f"{ident(a)} <> {ident(b)}" for a, b in combinations(roles, 2))
        result.append(
            ir.Constraint(
                name=f"{table_name}__ck__diagonal",
                body=ir.check(expression),
                construct=f"{construct}.transposition",
            )
        )
    return result


def _requirement_constraint(table_name: str, requirement: m.Requirement) -> ir.Constraint:
    attrs = [ident(a) for a in requirement.attributes]
    rule = requirement.rule
    if rule == "nonempty":
        expression = f"{attrs[0]} <> ''"
    elif rule == "positive":
        expression = f"{attrs[0]} > 0"
    elif rule == "nonnegative":
        expression = f"{attrs[0]} >= 0"
    elif rule == "ordered":
        expression = f"{attrs[0]} <= {attrs[1]}"
    elif rule == "ordered_same_reference":
        expression = (
            f"{attrs[0]} IS NULL OR {attrs[1]} IS NULL OR {attrs[2]} IS DISTINCT FROM {attrs[3]} "
            f"OR {attrs[0]} <= {attrs[1]}"
        )
    elif rule == "present_iff":
        members = ", ".join(literal(member) for member in requirement.members)
        expression = f"({attrs[0]} IS NOT NULL) = ({attrs[1]}::text IN ({members}))"
    elif rule == "within":
        bounds = []
        if requirement.lower is not None:
            bounds.append(f"{attrs[0]} >= {literal(requirement.lower)}")
        if requirement.upper is not None:
            bounds.append(f"{attrs[0]} <= {literal(requirement.upper)}")
        expression = " AND ".join(bounds)
    else:  # one_of_present
        expression = f"num_nonnulls({', '.join(attrs)}) = 1"
    return ir.Constraint(
        name=f"{table_name}__ck__{requirement.name}",
        body=ir.check(expression),
        construct=requirement.construct,
    )


class _Builder:
    def __init__(self, decl: m.Declaration) -> None:
        self.decl = decl
        self.plan = Plan()
        self.group_of_kind = {group.id: group for group in decl.slot_groups}
        self.ids = decl.meta_ids()

    # -- columns ---------------------------------------------------------------------------

    def add_field(
        self,
        table: ir.Table,
        field_: m.Field,
        *,
        nullable: bool | None = None,
    ) -> None:
        """Add the column of `field_` with its foreign key, checks and uniqueness."""
        decl = self.decl
        type_ = field_.type
        name = field_.name
        not_null = not (field_.optional if nullable is None else nullable)
        table.columns.append(
            ir.Column(
                name=name,
                sql_type=sql_type(decl, type_),
                comment=field_.doc,
                construct=field_.construct,
                not_null=not_null,
                default=None if field_.default is None else _literal(field_.default),
            )
        )
        target = _reference_target(decl, type_)
        if target is not None:
            table.foreign_keys.append(
                ir.ForeignKey(
                    name=f"{table.name}__fk__{name}",
                    columns=(name,),
                    ref_schema=target[0],
                    ref_table=target[1],
                    ref_columns=("id",),
                    construct=field_.construct,
                )
            )
        if type_.container == "range" and decl.quantity_types[type_.element].scale == "absolute":
            table.constraints.append(
                ir.Constraint(
                    name=f"{table.name}__ck__{name}__nonnegative",
                    body=ir.check(f"lower({ident(name)}) >= 0"),
                    construct=field_.construct,
                )
            )
        if type_.container == "array":
            table.constraints.append(
                ir.Constraint(
                    name=f"{table.name}__ck__{name}__no_null",
                    body=ir.check(f"array_position({ident(name)}, NULL) IS NULL"),
                    construct=field_.construct,
                )
            )
        if field_.unique:
            table.constraints.append(
                ir.Constraint(
                    name=f"{table.name}__uq__{name}",
                    body=ir.unique([name]),
                    construct=field_.construct,
                    index_backed=True,
                )
            )

    def add_stateful(
        self, table: ir.Table, slot: m.Field, group_table: str, parameter_set: m.Kind
    ) -> None:
        """A stateful slot: value, state and redirect columns tied together by a check."""
        name = slot.name
        state, redirect = f"{name}__state", f"{name}__redirect"
        self.add_field(table, slot, nullable=True)
        table.columns.append(
            ir.Column(
                name=state,
                sql_type=qname("meta", "value_state"),
                comment=f"Whether `{name}` holds a value and, if not, why.",
                construct=slot.construct,
            )
        )
        table.columns.append(
            ir.Column(
                name=redirect,
                sql_type="uuid",
                comment=f"The parameter set `{name}` takes its value from, when its state is redirect.",
                construct=slot.construct,
                not_null=False,
            )
        )
        value, state_q, redirect_q = ident(name), ident(state), ident(redirect)
        table.constraints.append(
            ir.Constraint(
                name=f"{group_table}__ck__{name}__state",
                body=ir.check(
                    f"({state_q} = 'known') = ({value} IS NOT NULL) "
                    f"AND ({state_q} = 'redirect') = ({redirect_q} IS NOT NULL)"
                ),
                construct=slot.construct,
            )
        )
        table.constraints.append(
            ir.Constraint(
                name=f"{group_table}__ck__{name}__redirect",
                body=ir.check(f"{redirect_q} IS DISTINCT FROM {ident('id')}"),
                construct=slot.construct,
            )
        )
        table.foreign_keys.append(
            ir.ForeignKey(
                name=f"{group_table}__fk__{name}__redirect",
                columns=(redirect, "slot_group"),
                ref_schema=parameter_set.schema,
                ref_table=parameter_set.name,
                ref_columns=("id", "slot_group"),
                construct=slot.construct,
            )
        )

    # -- kinds -----------------------------------------------------------------------------

    def kind_table(self, kind: m.Kind) -> list[ir.Table]:
        decl = self.decl
        table = ir.Table(
            schema=kind.schema,
            name=kind.name,
            comment=kind.doc,
            construct=kind.construct,
            module=kind.module,
        )
        table.columns.append(
            ir.Column(
                name="id",
                sql_type="uuid",
                comment=f"Deterministic identifier of the {kind.name} instance.",
                construct=kind.construct,
            )
        )
        table.constraints.append(
            ir.Constraint(
                name=f"{kind.name}__pk",
                body=ir.primary_key(["id"]),
                construct=kind.construct,
                index_backed=True,
            )
        )
        extra: list[ir.Table] = []
        group = self.group_of_kind.get(kind.name)
        parameter_set = decl.kinds[decl.framework["parameter_set"]] if decl.framework else None
        if group is not None:
            assert parameter_set is not None
            marker = self.ids["slot_group"][group.qualified]
            table.columns.append(
                ir.Column(
                    name="slot_group",
                    sql_type="uuid",
                    comment="The slot group of this table; fixed, and tied to the parameter set row.",
                    construct=kind.construct,
                    default=f"{literal(str(marker))}::uuid",
                )
            )
            table.constraints.append(
                ir.Constraint(
                    name=f"{kind.name}__ck__slot_group",
                    body=ir.check(f"{ident('slot_group')} = {literal(str(marker))}::uuid"),
                    construct=kind.construct,
                )
            )
            table.foreign_keys.append(
                ir.ForeignKey(
                    name=f"{kind.name}__fk__id",
                    columns=("id", "slot_group"),
                    ref_schema=parameter_set.schema,
                    ref_table=parameter_set.name,
                    ref_columns=("id", "slot_group"),
                    construct=kind.construct,
                )
            )
        elif kind.extends is not None:
            parent = decl.kinds[kind.extends]
            table.foreign_keys.append(
                ir.ForeignKey(
                    name=f"{kind.name}__fk__id",
                    columns=("id",),
                    ref_schema=parent.schema,
                    ref_table=parent.name,
                    ref_columns=("id",),
                    construct=kind.construct,
                )
            )
        elif kind.provenance.mode == "own":
            table.foreign_keys.append(
                ir.ForeignKey(
                    name=f"{kind.name}__fk__id",
                    columns=("id",),
                    ref_schema="prov",
                    ref_table="record",
                    ref_columns=("id",),
                    construct=kind.construct,
                )
            )
        if kind.extends is None and kind.origin == "declared":
            table.constraints.append(
                ir.Constraint(
                    name=f"{kind.name}__identity",
                    body=ir.unique(list(kind.identity)),
                    construct=f"{kind.construct}.identity",
                    index_backed=True,
                )
            )
        if parameter_set is not None and kind.name == parameter_set.name:
            table.constraints.append(
                ir.Constraint(
                    name=f"{kind.name}__uq__id_slot_group",
                    body=ir.unique(["id", "slot_group"]),
                    construct=kind.construct,
                    index_backed=True,
                )
            )
        for attribute in kind.attributes:
            if attribute.type.container == "set":
                extra.append(self.set_table(kind, attribute))
            elif group is not None and attribute.presence == "stateful":
                assert parameter_set is not None
                self.add_stateful(table, attribute, kind.name, parameter_set)
            elif (
                kind.origin == "declared"
                and kind.extends is None
                and attribute.name in kind.identity
            ):
                self.add_field(table, attribute, nullable=False)
            else:
                self.add_field(table, attribute)
        for unique in kind.uniques:
            table.constraints.append(
                ir.Constraint(
                    name=f"{kind.name}__uq__{unique.name}",
                    body=ir.unique(list(unique.attributes)),
                    construct=unique.construct,
                    index_backed=True,
                )
            )
        for requirement in kind.requires:
            if requirement.enforced == "ddl":
                table.constraints.append(_requirement_constraint(kind.name, requirement))
        if group is not None and group.transposition is not None:
            table.constraints.extend(
                _transposition_constraints(kind.name, group.transposition, group.construct)
            )
        return [table, *extra]

    def set_table(self, kind: m.Kind, attribute: m.Field) -> ir.Table:
        name = f"{kind.name}__{attribute.name}"
        member_schema = self.decl.kinds[attribute.type.element].schema
        table = ir.Table(
            schema=kind.schema,
            name=name,
            comment=attribute.doc,
            construct=attribute.construct,
            module=kind.module,
            columns=[
                ir.Column(
                    name="owner",
                    sql_type="uuid",
                    comment=f"The {kind.name} that holds the set.",
                    construct=attribute.construct,
                ),
                ir.Column(
                    name="member",
                    sql_type="uuid",
                    comment=f"A {attribute.type.element} in the set.",
                    construct=attribute.construct,
                ),
            ],
            constraints=[
                ir.Constraint(
                    name=f"{name}__pk",
                    body=ir.primary_key(["owner", "member"]),
                    construct=attribute.construct,
                    index_backed=True,
                )
            ],
            foreign_keys=[
                ir.ForeignKey(
                    name=f"{name}__fk__owner",
                    columns=("owner",),
                    ref_schema=kind.schema,
                    ref_table=kind.name,
                    ref_columns=("id",),
                    construct=attribute.construct,
                ),
                ir.ForeignKey(
                    name=f"{name}__fk__member",
                    columns=("member",),
                    ref_schema=member_schema,
                    ref_table=attribute.type.element,
                    ref_columns=("id",),
                    construct=attribute.construct,
                ),
            ],
        )
        return table

    # -- relations -------------------------------------------------------------------------

    def relation_table(self, relation: m.Relation) -> ir.Table:
        table = ir.Table(
            schema=relation.schema,
            name=relation.name,
            comment=relation.doc,
            construct=relation.construct,
            module=relation.module,
        )
        table.columns.append(
            ir.Column(
                name="id",
                sql_type="uuid",
                comment="Deterministic identifier of the row, computed from its keys.",
                construct=relation.construct,
            )
        )
        for key in relation.keys:
            self.add_field(table, key, nullable=False)
        for value in relation.values:
            self.add_field(table, value)
        table.constraints.append(
            ir.Constraint(
                name=f"{relation.name}__pk",
                body=ir.primary_key([key.name for key in relation.keys]),
                construct=relation.construct,
                index_backed=True,
            )
        )
        table.constraints.append(
            ir.Constraint(
                name=f"{relation.name}__uq__id",
                body=ir.unique(["id"]),
                construct=relation.construct,
                index_backed=True,
            )
        )
        if relation.provenance.mode == "own":
            table.foreign_keys.append(
                ir.ForeignKey(
                    name=f"{relation.name}__fk__id",
                    columns=("id",),
                    ref_schema="prov",
                    ref_table="record",
                    ref_columns=("id",),
                    construct=relation.construct,
                )
            )
        for requirement in relation.requires:
            if requirement.enforced == "ddl":
                table.constraints.append(_requirement_constraint(relation.name, requirement))
        if relation.transposition is not None:
            table.constraints.extend(
                _transposition_constraints(
                    relation.name, relation.transposition, relation.construct
                )
            )
        return table

    # -- families --------------------------------------------------------------------------

    def family_table(self, group: m.SlotGroup, family: m.Family) -> ir.Table:
        table = ir.Table(
            schema="param",
            name=family.id,
            comment=family.doc,
            construct=family.construct,
            module=self.decl.forms[group.form].module,
        )
        table.columns.append(
            ir.Column(
                name="set_id",
                sql_type="uuid",
                comment="The parameter set this row belongs to.",
                construct=family.construct,
            )
        )
        table.foreign_keys.append(
            ir.ForeignKey(
                name=f"{family.id}__fk__set_id",
                columns=("set_id",),
                ref_schema="param",
                ref_table=group.id,
                ref_columns=("id",),
                construct=family.construct,
            )
        )
        for index in family.indices:
            self.add_field(table, index, nullable=False)
            if index.minimum is not None:
                table.constraints.append(
                    ir.Constraint(
                        name=f"{family.id}__ck__{index.name}__minimum",
                        body=ir.check(f"{ident(index.name)} >= {index.minimum}"),
                        construct=index.construct,
                    )
                )
        for slot in family.slots:
            self.add_field(table, slot, nullable=False)
        table.constraints.append(
            ir.Constraint(
                name=f"{family.id}__pk",
                body=ir.primary_key(["set_id", *(index.name for index in family.indices)]),
                construct=family.construct,
                index_backed=True,
            )
        )
        if family.interval is not None:
            lower, upper = family.interval
            table.columns.append(
                ir.Column(
                    name="interval",
                    sql_type=qname("meta", "float8range"),
                    comment="The half-open interval [lower, upper) the piece covers.",
                    construct=f"{family.construct}.interval",
                    not_null=False,
                    generated=f"{qname('meta', 'float8range')}({ident(lower)}, {ident(upper)}, '[)')",
                )
            )
            table.constraints.append(
                ir.Constraint(
                    name=f"{family.id}__ck__interval",
                    body=ir.check(f"{ident(lower)} < {ident(upper)}"),
                    construct=f"{family.construct}.interval",
                )
            )
            table.constraints.append(
                ir.Constraint(
                    name=f"{family.id}__xc__interval",
                    body=f"EXCLUDE USING gist ({ident('set_id')} WITH =, {ident('interval')} WITH &&)",
                    construct=f"{family.construct}.interval",
                    index_backed=True,
                )
            )
            self.plan.needs_btree_gist = True
        return table

    def meta_table(self, table: ir.Table) -> ir.Table:
        """A `meta` table; with the `observable` or `composition_basis` role bound, the columns
        that hold a declared entity of that role's kind also reference the kind's table."""
        references = []
        for role, column, tables in META_ENTITY_COLUMNS:
            kind_name = self.decl.framework.get(role)
            if kind_name is None or table.name not in tables:
                continue
            kind = self.decl.kinds[kind_name]
            references.append(
                ir.ForeignKey(
                    name=f"{table.name}__fk__{column}",
                    columns=(column,),
                    ref_schema=kind.schema,
                    ref_table=kind.name,
                    ref_columns=("id",),
                    construct=f"meta.{table.name}.{column}",
                )
            )
        if not references:
            return table
        return dataclasses.replace(table, foreign_keys=[*table.foreign_keys, *references])

    # -- whole plan ------------------------------------------------------------------------

    def build(self) -> Plan:
        decl = self.decl
        plan = self.plan
        plan.types.extend(_fixed_types())
        for enum in decl.enums.values():
            members = ", ".join(literal(member.name) for member in enum.members)
            plan.types.append(
                ir.TypeDef(
                    object_kind="TYPE",
                    schema="meta",
                    name=enum.name,
                    create=f"CREATE TYPE {qname('meta', enum.name)} AS ENUM ({members});",
                    comment=enum.doc,
                    construct=enum.construct,
                    module=enum.module,
                )
            )
        for scheme in decl.schemes.values():
            plan.types.append(
                _domain(
                    naming.scheme_domain(scheme.name),
                    "text",
                    scheme.doc,
                    [("nonempty", "VALUE <> ''")],
                    construct=scheme.construct,
                    module=scheme.module,
                )
            )
        for quantity in decl.quantity_types.values():
            domain = naming.quantity_domain(quantity.name)
            checks = [("finite", f"VALUE {_FINITE}")]
            if quantity.scale == "absolute":
                checks.append(("nonnegative", "VALUE >= 0"))
            plan.types.append(
                _domain(
                    domain,
                    "double precision",
                    f"{quantity.doc} Stored in {quantity.unit}.",
                    checks,
                    construct=quantity.construct,
                    module=quantity.module,
                )
            )
        plan.tables.extend(self.meta_table(table) for table in META_TABLES.values())
        plan.tables.append(
            ir.Table(
                schema="prov",
                name="record",
                comment="Registry of every record: what a source asserted. Provenance attaches here.",
                construct="prov.record",
                columns=[
                    ir.Column(
                        name="id",
                        sql_type="uuid",
                        comment="Identifier of the registered kind instance or relation row.",
                        construct="prov.record.id",
                    ),
                    ir.Column(
                        name="kind",
                        sql_type="text",
                        comment="The name of the kind or relation the record is an instance of.",
                        construct="prov.record.kind",
                    ),
                ],
                constraints=[
                    ir.Constraint(
                        name="record__pk",
                        body=ir.primary_key(["id"]),
                        construct="prov.record",
                        index_backed=True,
                    )
                ],
            )
        )
        for kind in decl.kinds.values():
            plan.tables.extend(self.kind_table(kind))
        for relation in decl.relations.values():
            plan.tables.append(self.relation_table(relation))
        for group in sorted(decl.slot_groups, key=lambda g: g.id):
            for family in group.families:
                plan.tables.append(self.family_table(group, family))
        return plan


def build_plan(decl: m.Declaration) -> Plan:
    """Describe every object the DDL of `decl` creates."""
    return _Builder(decl).build()


def render_schema(plan: Plan) -> str:
    """The plan as one SQL file that applies to an empty database."""
    blocks = [
        "-- @generated by thermo_knowledge.generate from the declaration; do not edit.\n"
        "-- Regenerate with `tk generate`; `tk generate --check` compares.",
        "SET standard_conforming_strings = on;",
    ]
    for schema, doc in SCHEMA_DOCS.items():
        blocks.append(
            f"CREATE SCHEMA {ident(schema)};\nCOMMENT ON SCHEMA {ident(schema)} IS {literal(doc)};"
        )
    if plan.needs_btree_gist:
        blocks.append(f"CREATE EXTENSION IF NOT EXISTS btree_gist WITH SCHEMA {ident('meta')};")
    blocks.extend(ir.render_type(type_def) for type_def in plan.types)
    blocks.extend(ir.render_table(table) for table in plan.tables)
    for table in plan.tables:
        blocks.extend(ir.render_foreign_key(table, fk) for fk in table.foreign_keys)
    return "\n\n".join(blocks) + "\n"


# -- identifier checks ----------------------------------------------------------------------


def projection_diagnostics(decl: m.Declaration) -> list[Diagnostic]:
    """Identifiers over PostgreSQL's 63-byte limit and names that would collide."""
    plan = build_plan(decl)
    found: list[Diagnostic] = []

    def report(module: str | None, construct: str, code: Code, message: str) -> None:
        module_info = decl.modules.get(module) if module else None
        document = module_info.document if module_info else MANIFEST_DOCUMENT
        found.append(Diagnostic(document=document, construct=construct, code=code, message=message))

    def length(name: str, module: str | None, construct: str, what: str) -> None:
        if len(name.encode()) > MAX_IDENTIFIER_BYTES:
            report(
                module,
                construct,
                Code.IDENTIFIER_TOO_LONG,
                f"{what} `{name}` is {len(name.encode())} bytes; PostgreSQL allows "
                f"{MAX_IDENTIFIER_BYTES}",
            )

    relations: dict[tuple[str, str], str] = {}

    def claim(schema: str, name: str, module: str | None, construct: str) -> None:
        other = relations.get((schema, name))
        if other is not None:
            report(
                module,
                construct,
                Code.IDENTIFIER_COLLISION,
                f"`{schema}.{name}` is also projected from {other}",
            )
        else:
            relations[(schema, name)] = construct

    for schema in GENERATED_SCHEMAS:
        length(schema, None, schema, "schema")
    # Built-in objects claim their names first, so a collision is reported on the declaration.

    def by_builtin(item: ir.Table | ir.TypeDef) -> bool:
        return item.module is not None

    for type_def in sorted(plan.types, key=by_builtin):
        length(type_def.name, type_def.module, type_def.construct, "type")
    for type_def in (t for t in plan.types if t.module is None):
        claim(type_def.schema, type_def.name, None, type_def.construct)
    for table in (t for t in plan.tables if t.module is None):
        claim(table.schema, table.name, None, table.construct)
    for type_def in (t for t in plan.types if t.module is not None):
        claim(type_def.schema, type_def.name, type_def.module, type_def.construct)
    for table in sorted(plan.tables, key=by_builtin):
        length(table.name, table.module, table.construct, "table")
        if table.module is not None:
            claim(table.schema, table.name, table.module, table.construct)
        column_names: set[str] = set()
        for column in table.columns:
            length(column.name, table.module, column.construct, "column")
            if column.name in column_names:
                report(
                    table.module,
                    column.construct,
                    Code.IDENTIFIER_COLLISION,
                    f"table `{table.schema}.{table.name}` would have two columns `{column.name}`",
                )
            column_names.add(column.name)
        constraint_names: set[str] = set()
        named = [(c.name, c.construct, c.index_backed) for c in table.constraints] + [
            (fk.name, fk.construct, False) for fk in table.foreign_keys
        ]
        for name, construct, index_backed in named:
            length(name, table.module, construct, "constraint")
            if name in constraint_names:
                report(
                    table.module,
                    construct,
                    Code.IDENTIFIER_COLLISION,
                    f"table `{table.schema}.{table.name}` would have two constraints `{name}`",
                )
            constraint_names.add(name)
            if index_backed:
                claim(table.schema, name, table.module, construct)
    return found


__all__ = [
    "GENERATED_SCHEMAS",
    "Plan",
    "build_plan",
    "projection_diagnostics",
    "render_schema",
    "sql_type",
]
