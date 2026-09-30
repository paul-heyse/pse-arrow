# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `meta` tables: the declaration reified as rows (section 7).

The tables are defined here once, as data. `META_TABLES` yields the DDL; `meta_rows` (in
`meta_rows.py`) builds the rows for exactly these columns. Rows that `Meta<construct>` refers to
carry `id uuid`, derived from the construct's qualified name (`identity.meta_identifier`); every
other table is keyed by the natural names.
"""

from __future__ import annotations

from collections.abc import Sequence

from thermo_knowledge.generate import ir
from thermo_knowledge.generate.sqltext import ident, literal

META = "meta"

type Col = tuple[str, str, str]
"""A column: name, SQL type (a trailing `?` makes it nullable) and its comment."""

CONTAINERS = ("scalar", "range", "array", "set")
ELEMENT_KINDS = (
    "primitive",
    "identifier",
    "quantity",
    "expression",
    "enum",
    "kind",
    "record",
    "meta",
    "real",
    "source_text",
)
SHAPES = ("quantity", "enum", "reference", "tabulated_function", "nested_set", "set_reference")
PROVENANCE_MODES = ("own", "inherit", "declaration", "none")

_TYPE_COLUMNS: tuple[Col, ...] = (
    ("container", "text", "How the type holds values: scalar, range, array or set."),
    ("element_kind", "text", "What the element is: primitive, quantity, enum, kind and so on."),
    (
        "element",
        "text",
        "The element: a type, scheme, enum or kind name, an expression or a Meta construct.",
    ),
)


def _in(column: str, values: Sequence[str]) -> str:
    return f"{ident(column)} IN ({', '.join(literal(v) for v in values)})"


def _table(
    name: str,
    comment: str,
    cols: Sequence[Col],
    *,
    pk: Sequence[str],
    unique: Sequence[Sequence[str]] = (),
    fks: Sequence[tuple[Sequence[str], str, Sequence[str]]] = (),
    checks: Sequence[tuple[str, str]] = (),
    vocabulary: dict[str, Sequence[str]] | None = None,
    nulls_not_distinct: Sequence[Sequence[str]] = (),
) -> ir.Table:
    construct = f"meta.{name}"
    table = ir.Table(schema=META, name=name, comment=comment, construct=construct)
    for column, sql_type, doc in cols:
        nullable = sql_type.endswith("?")
        table.columns.append(
            ir.Column(
                name=column,
                sql_type=sql_type.rstrip("?"),
                comment=doc,
                construct=f"{construct}.{column}",
                not_null=not nullable,
            )
        )
    table.constraints.append(
        ir.Constraint(
            name=f"{name}__pk",
            body=ir.primary_key(list(pk)),
            construct=construct,
            index_backed=True,
        )
    )
    for group in unique:
        table.constraints.append(
            ir.Constraint(
                name=f"{name}__uq__{'_'.join(group)}",
                body=ir.unique(list(group)),
                construct=construct,
                index_backed=True,
            )
        )
    for group in nulls_not_distinct:
        table.constraints.append(
            ir.Constraint(
                name=f"{name}__uq__{'_'.join(group)}",
                body="UNIQUE NULLS NOT DISTINCT (" + ", ".join(ident(c) for c in group) + ")",
                construct=construct,
                index_backed=True,
            )
        )
    for column, values in (vocabulary or {}).items():
        table.constraints.append(
            ir.Constraint(
                name=f"{name}__ck__{column}",
                body=ir.check(_in(column, values)),
                construct=construct,
            )
        )
    for check_name, expression in checks:
        table.constraints.append(
            ir.Constraint(
                name=f"{name}__ck__{check_name}", body=ir.check(expression), construct=construct
            )
        )
    for columns, ref_table, ref_columns in fks:
        table.foreign_keys.append(
            ir.ForeignKey(
                name=f"{name}__fk__{'_'.join(columns)}",
                columns=tuple(columns),
                ref_schema=META,
                ref_table=ref_table,
                ref_columns=tuple(ref_columns),
                construct=construct,
            )
        )
    return table


def _build() -> dict[str, ir.Table]:
    t: list[ir.Table] = []
    add = t.append
    add(
        _table(
            "module",
            "A declaration module: one TOML document.",
            [
                ("name", "text", "The module's unique name."),
                ("schema_name", "text?", "The schema of the module's tables, where it has any."),
                ("doc", "text", "What the module covers."),
                ("document", "text", "The declaring document, relative to the tree."),
            ],
            pk=["name"],
            vocabulary={"schema_name": ("tk", "prov", "ev", "qual")},
        )
    )
    add(
        _table(
            "module_use",
            "A module's direct dependency on another module's names.",
            [
                ("module", "text", "The using module."),
                ("used", "text", "The module whose names it may reference."),
            ],
            pk=["module", "used"],
            fks=[(["module"], "module", ["name"]), (["used"], "module", ["name"])],
        )
    )
    add(
        _table(
            "framework_role",
            "The kind that fills each framework role named in the manifest.",
            [
                ("role", "text", "The role."),
                ("kind", "text", "The kind that fills it."),
            ],
            pk=["role"],
            fks=[(["kind"], "kind", ["name"])],
            vocabulary={
                "role": (
                    "parameter_set",
                    "parameterization",
                    "tabulated_function",
                    "slot_uncertainty",
                    "observable",
                    "composition_basis",
                    "convention_set",
                )
            },
        )
    )
    add(
        _table(
            "unit",
            "A canonical unit: a storage unit or the unit of a quantity expression.",
            [("unit", "text", "The canonical unit text (pint, abbreviated).")],
            pk=["unit"],
        )
    )
    add(
        _table(
            "unit_dimension",
            "The dimensionality of a unit as exponents of base dimensions.",
            [
                ("unit", "text", "The unit."),
                (
                    "dimension",
                    "text",
                    "A base dimension: mass, length, time, temperature, substance, ...",
                ),
                ("exponent", "integer", "Its exponent."),
            ],
            pk=["unit", "dimension"],
            fks=[(["unit"], "unit", ["unit"])],
        )
    )
    add(
        _table(
            "quantity_type",
            "A dimensioned number type with a storage unit.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<quantity_type> refers to."),
                ("name", "text", "The type's name."),
                ("module", "text", "The declaring module."),
                ("unit", "text", "The storage unit."),
                ("scale", "text", "absolute, difference, ratio, dimensionless or count."),
                ("production", "text?", "The corresponding production quantity type, if any."),
                (
                    "depends_on",
                    "text?",
                    "For a dependent type: the kind of the subject whose reaction-derived "
                    "dimension the type has (the storage unit is then that of a homogeneous "
                    "reaction of order zero).",
                ),
                (
                    "surface_unit",
                    "text?",
                    "For a dependent type: the rate unit when a participant is on a surface.",
                ),
                (
                    "concentration_unit",
                    "text?",
                    "For a dependent type: the unit of a species' concentration in a bulk phase.",
                ),
                (
                    "surface_concentration_unit",
                    "text?",
                    "For a dependent type: the unit of a species' concentration on a surface.",
                ),
                ("pg_type", "text", "The domain that projects the type."),
                ("doc", "text", "What the quantity means."),
            ],
            pk=["id"],
            unique=[["name"]],
            fks=[
                (["module"], "module", ["name"]),
                (["unit"], "unit", ["unit"]),
                (["depends_on"], "kind", ["name"]),
                (["surface_unit"], "unit", ["unit"]),
                (["concentration_unit"], "unit", ["unit"]),
                (["surface_concentration_unit"], "unit", ["unit"]),
            ],
            vocabulary={"scale": ("absolute", "difference", "ratio", "dimensionless", "count")},
            checks=[
                (
                    "dependent_complete",
                    '("depends_on" IS NULL) = ("surface_unit" IS NULL) '
                    'AND ("depends_on" IS NULL) = ("concentration_unit" IS NULL) '
                    'AND ("depends_on" IS NULL) = ("surface_concentration_unit" IS NULL)',
                )
            ],
        )
    )
    add(
        _table(
            "quantity_expression",
            "A quantity expression used as a type, with the unit it resolves to.",
            [
                ("expression", "text", "The normalised expression."),
                ("unit", "text", "The unit computed by unit algebra."),
            ],
            pk=["expression"],
            fks=[(["unit"], "unit", ["unit"])],
        )
    )
    add(
        _table(
            "identifier_scheme",
            "A declared scheme of opaque identifiers.",
            [
                ("name", "text", "The scheme's name."),
                ("module", "text", "The declaring module."),
                ("pg_type", "text", "The domain that projects the scheme."),
                ("doc", "text", "What the identifiers identify."),
            ],
            pk=["name"],
            fks=[(["module"], "module", ["name"])],
        )
    )
    add(
        _table(
            "enum",
            "A closed vocabulary pipeline code branches on.",
            [
                ("name", "text", "The enum's name."),
                ("module", "text", "The declaring module."),
                ("pg_type", "text", "The enum type that projects it."),
                ("doc", "text", "What the vocabulary distinguishes."),
            ],
            pk=["name"],
            fks=[(["module"], "module", ["name"])],
        )
    )
    add(
        _table(
            "enum_member",
            "A member of an enum.",
            [
                ("enum", "text", "The enum."),
                ("name", "text", "The member's name."),
                ("ordinal", "integer", "Position in the enum, from 1."),
                ("doc", "text", "What the member means."),
            ],
            pk=["enum", "name"],
            unique=[["enum", "ordinal"]],
            fks=[(["enum"], "enum", ["name"])],
        )
    )
    add(
        _table(
            "enum_facet",
            "A facet an enum declares: a property some of its members have.",
            [
                ("enum", "text", "The enum."),
                ("name", "text", "The facet's name."),
                ("ordinal", "integer", "Position among the enum's facets, from 1."),
                ("doc", "text", "What having the facet means."),
            ],
            pk=["enum", "name"],
            unique=[["enum", "ordinal"]],
            fks=[(["enum"], "enum", ["name"])],
        )
    )
    add(
        _table(
            "enum_member_facet",
            "A facet a member of an enum has.",
            [
                ("enum", "text", "The enum."),
                ("member", "text", "The member."),
                ("facet", "text", "A facet of the same enum the member has."),
            ],
            pk=["enum", "member", "facet"],
            fks=[
                (["enum", "member"], "enum_member", ["enum", "name"]),
                (["enum", "facet"], "enum_facet", ["enum", "name"]),
            ],
        )
    )
    add(
        _table(
            "kind",
            "A concept whose instances have identity; includes the kind each slot group expands to.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<kind> refers to."),
                ("name", "text", "The kind's name."),
                ("module", "text", "The declaring module (a slot group's form module)."),
                ("origin", "text", "declared, or slot_group for the kind a slot group expands to."),
                ("extends", "text?", "The kind this one refines."),
                ("root", "text", "The root of the refinement chain, which owns the identity."),
                ("abstract", "boolean", "Whether the kind has no direct instances."),
                ("provenance", "text", "own, inherit, declaration or none."),
                (
                    "provenance_attribute",
                    "text?",
                    "The reference whose record an inheriting instance shares.",
                ),
                ("schema_name", "text", "The schema of the kind's table."),
                ("table_name", "text", "The kind's table."),
                ("doc", "text", "What the kind is."),
            ],
            pk=["id"],
            unique=[["name"]],
            fks=[
                (["module"], "module", ["name"]),
                (["extends"], "kind", ["name"]),
                (["root"], "kind", ["name"]),
            ],
            vocabulary={
                "origin": ("declared", "slot_group"),
                "provenance": PROVENANCE_MODES,
            },
            checks=[
                (
                    "provenance_attribute",
                    '("provenance" = \'inherit\') = ("provenance_attribute" IS NOT NULL)',
                )
            ],
        )
    )
    add(
        _table(
            "attribute",
            "An attribute of a kind: a column of its table or a child table.",
            [
                ("kind", "text", "The kind."),
                ("name", "text", "The attribute's name."),
                ("position", "integer", "Declared position within the kind, from 1."),
                *_TYPE_COLUMNS,
                ("optional", "boolean", "Whether an instance may have no such attribute."),
                ("is_unique", "boolean", "Whether values are unique in addition to identity."),
                ("default_value", "text?", "The default, as canonical text."),
                (
                    "unit_from",
                    "text?",
                    "For a Real: where its unit comes from (a path of references or `dimensionless`).",
                ),
                ("doc", "text", "What the attribute means."),
            ],
            pk=["kind", "name"],
            unique=[["kind", "position"]],
            fks=[(["kind"], "kind", ["name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
        )
    )
    add(
        _table(
            "kind_identity",
            "An attribute of a root kind's identity, in encoding order.",
            [
                ("kind", "text", "The root kind."),
                ("position", "integer", "Position in the identity encoding, from 1."),
                ("attribute", "text", "The identity attribute."),
            ],
            pk=["kind", "position"],
            unique=[["kind", "attribute"]],
            fks=[(["kind", "attribute"], "attribute", ["kind", "name"])],
        )
    )
    add(
        _table(
            "kind_unique",
            "A composite uniqueness constraint of a kind.",
            [
                ("kind", "text", "The kind."),
                ("name", "text", "The constraint's name."),
            ],
            pk=["kind", "name"],
            fks=[(["kind"], "kind", ["name"])],
        )
    )
    add(
        _table(
            "kind_unique_attribute",
            "An attribute of a composite uniqueness constraint.",
            [
                ("kind", "text", "The kind."),
                ("unique_name", "text", "The constraint."),
                ("position", "integer", "Position in the constraint, from 1."),
                ("attribute", "text", "The attribute."),
            ],
            pk=["kind", "unique_name", "position"],
            fks=[
                (["kind", "unique_name"], "kind_unique", ["kind", "name"]),
                (["kind", "attribute"], "attribute", ["kind", "name"]),
            ],
        )
    )
    owner_of_requirement: list[Col] = [
        ("owner_type", "text", "kind or relation."),
        ("owner", "text", "The name of the kind or of the relation."),
    ]
    add(
        _table(
            "requirement",
            "An invariant of a kind or a relation and where it is enforced.",
            [
                *owner_of_requirement,
                ("name", "text", "The invariant's name."),
                ("enforced", "text", "Where it is enforced: ddl, load or verify."),
                (
                    "check_rule",
                    "text?",
                    "For ddl: nonempty, positive, nonnegative, ordered, one_of_present, "
                    "present_iff or within.",
                ),
                ("check_lower", "double precision?", "For within: the lower bound, when stated."),
                ("check_upper", "double precision?", "For within: the upper bound, when stated."),
                ("doc", "text", "The invariant."),
            ],
            pk=["owner_type", "owner", "name"],
            vocabulary={
                "owner_type": ("kind", "relation"),
                "enforced": ("ddl", "load", "verify"),
                "check_rule": (
                    "nonempty",
                    "positive",
                    "nonnegative",
                    "ordered",
                    "ordered_same_reference",
                    "one_of_present",
                    "present_iff",
                    "within",
                ),
            },
            checks=[
                ("rule_for_ddl", '("enforced" = \'ddl\') = ("check_rule" IS NOT NULL)'),
                (
                    "bounds_for_within",
                    '("check_lower" IS NULL AND "check_upper" IS NULL) '
                    'OR "check_rule" = \'within\'',
                ),
            ],
        )
    )
    add(
        _table(
            "requirement_attribute",
            "A column a ddl invariant's check is about: an attribute of a kind, a key or a value "
            "of a relation.",
            [
                *owner_of_requirement,
                ("requirement", "text", "The invariant."),
                ("position", "integer", "Position in the check, from 1."),
                ("attribute", "text", "The column."),
            ],
            pk=["owner_type", "owner", "requirement", "position"],
            fks=[
                (
                    ["owner_type", "owner", "requirement"],
                    "requirement",
                    ["owner_type", "owner", "name"],
                )
            ],
        )
    )
    add(
        _table(
            "requirement_member",
            "An enum member a present_iff invariant's `in` list names.",
            [
                *owner_of_requirement,
                ("requirement", "text", "The invariant."),
                ("position", "integer", "Position in the list, from 1."),
                ("member", "text", "The enum member."),
            ],
            pk=["owner_type", "owner", "requirement", "position"],
            fks=[
                (
                    ["owner_type", "owner", "requirement"],
                    "requirement",
                    ["owner_type", "owner", "name"],
                )
            ],
        )
    )
    add(
        _table(
            "relation",
            "An n-ary fact keyed by typed roles.",
            [
                ("name", "text", "The relation's name."),
                ("module", "text", "The declaring module."),
                ("provenance", "text", "own, inherit, declaration or none."),
                ("provenance_key", "text?", "The key whose record an inheriting row shares."),
                ("absence", "text", "What a missing row means: required, optional or default."),
                ("absence_default", "text?", "The default, as canonical text."),
                ("schema_name", "text", "The schema of the relation's table."),
                ("table_name", "text", "The relation's table."),
                ("doc", "text", "What the relation asserts."),
            ],
            pk=["name"],
            fks=[(["module"], "module", ["name"])],
            vocabulary={
                "provenance": PROVENANCE_MODES,
                "absence": ("required", "optional", "default"),
            },
            checks=[
                (
                    "provenance_key",
                    '("provenance" = \'inherit\') = ("provenance_key" IS NOT NULL)',
                ),
                ("absence_default", '("absence" = \'default\') = ("absence_default" IS NOT NULL)'),
            ],
        )
    )
    add(
        _table(
            "relation_key",
            "A key role of a relation.",
            [
                ("relation", "text", "The relation."),
                ("name", "text", "The role's name."),
                ("position", "integer", "Position in the key, from 1."),
                *_TYPE_COLUMNS,
                ("doc", "text", "What the role is."),
            ],
            pk=["relation", "name"],
            unique=[["relation", "position"]],
            fks=[(["relation"], "relation", ["name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
        )
    )
    add(
        _table(
            "relation_value",
            "A value column of a relation (a single value is named value).",
            [
                ("relation", "text", "The relation."),
                ("name", "text", "The column's name."),
                ("position", "integer", "Position among the values, from 1."),
                *_TYPE_COLUMNS,
                ("is_single", "boolean", "Whether this is the relation's single value."),
                ("optional", "boolean", "Whether a row may have no such value."),
                ("default_value", "text?", "The column default, as canonical text."),
                (
                    "unit_from",
                    "text?",
                    "For a Real: where its unit comes from (a path of references or `dimensionless`).",
                ),
                ("doc", "text", "What the value is."),
            ],
            pk=["relation", "name"],
            unique=[["relation", "position"]],
            fks=[(["relation"], "relation", ["name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
        )
    )
    owner_cols: list[Col] = [
        ("owner_type", "text", "relation or slot_group."),
        ("owner", "text", "The relation's name or the slot group's qualified name."),
    ]
    add(
        _table(
            "transposition",
            "What happens to a fact when two of its roles are swapped.",
            [
                *owner_cols,
                (
                    "rule",
                    "text",
                    "ordered, symmetric, parity, reciprocal, linear or permutation_group.",
                ),
                ("diagonal", "text", "forbidden or allowed: both roles may name one instance."),
                ("by_index", "text?", "For parity: the index whose parity sets the sign."),
            ],
            pk=["owner_type", "owner"],
            vocabulary={
                "owner_type": ("relation", "slot_group"),
                "rule": (
                    "ordered",
                    "symmetric",
                    "parity",
                    "reciprocal",
                    "linear",
                    "permutation_group",
                ),
                "diagonal": ("forbidden", "allowed"),
            },
            checks=[("by_for_parity", '("rule" = \'parity\') = ("by_index" IS NOT NULL)')],
        )
    )
    add(
        _table(
            "transposition_role",
            "A role a transposition swaps, in order.",
            [
                *owner_cols,
                ("position", "integer", "Position among the roles, from 1."),
                ("role", "text", "The key or subject role."),
            ],
            pk=["owner_type", "owner", "position"],
            fks=[(["owner_type", "owner"], "transposition", ["owner_type", "owner"])],
        )
    )
    add(
        _table(
            "transposition_slot",
            "A slot or value that changes when roles are swapped (reciprocal, linear), in the order of the rule.",
            [
                *owner_cols,
                (
                    "position",
                    "integer",
                    "Position among the slots, from 1; the row and column of a linear matrix.",
                ),
                ("slot", "text", "The slot or value column."),
            ],
            pk=["owner_type", "owner", "position"],
            unique=[["owner_type", "owner", "slot"]],
            fks=[(["owner_type", "owner"], "transposition", ["owner_type", "owner"])],
        )
    )
    add(
        _table(
            "transposition_matrix",
            "An entry of the matrix a linear transposition multiplies the values of its slots by.",
            [
                *owner_cols,
                ("row_position", "integer", "Row, from 1: the slot whose swapped value it gives."),
                (
                    "column_position",
                    "integer",
                    "Column, from 1: the slot whose value it multiplies.",
                ),
                ("coefficient", "double precision", "The entry."),
            ],
            pk=["owner_type", "owner", "row_position", "column_position"],
            fks=[(["owner_type", "owner"], "transposition", ["owner_type", "owner"])],
        )
    )
    add(
        _table(
            "transposition_permutation",
            "A role permutation that leaves a fact unchanged (permutation_group).",
            [
                *owner_cols,
                ("permutation", "integer", "Which permutation, from 1."),
                ("position", "integer", "Position in the arrangement, from 1."),
                ("role", "text", "The role placed there."),
            ],
            pk=["owner_type", "owner", "permutation", "position"],
            fks=[(["owner_type", "owner"], "transposition", ["owner_type", "owner"])],
        )
    )
    add(
        _table(
            "contract",
            "What a form consumes and produces.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<contract> refers to."),
                ("name", "text", "The contract's name."),
                ("module", "text", "The declaring module."),
                ("doc", "text", "The function the contract describes."),
            ],
            pk=["id"],
            unique=[["name"]],
            fks=[(["module"], "module", ["name"])],
        )
    )
    for label, what in (("role", "a single subject"), ("set", "an index set of entities")):
        add(
            _table(
                f"contract_{label}",
                f"A {label} of a contract: {what} an evaluation is about.",
                [
                    ("contract", "text", "The contract."),
                    ("name", "text", f"The {label}'s name."),
                    ("position", "integer", "Position, from 1."),
                    ("kind", "text", "The kind of the subject or of the set's elements."),
                    ("doc", "text", f"What the {label} is."),
                ],
                pk=["contract", "name"],
                unique=[["contract", "position"]],
                fks=[(["contract"], "contract", ["name"]), (["kind"], "kind", ["name"])],
            )
        )
    add(
        _table(
            "contract_argument",
            "An argument of a contract.",
            [
                ("contract", "text", "The contract."),
                ("name", "text", "The argument's name."),
                ("position", "integer", "Position, from 1."),
                *_TYPE_COLUMNS,
                (
                    "basis",
                    "uuid?",
                    "For a composition: the declared composition basis entity its values are "
                    "on (its identifier).",
                ),
                (
                    "observable",
                    "uuid?",
                    "The declared observable entity the argument is a value of (its identifier).",
                ),
                ("doc", "text", "What the argument is."),
            ],
            pk=["contract", "name"],
            unique=[["contract", "position"]],
            fks=[(["contract"], "contract", ["name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
        )
    )
    add(
        _table(
            "contract_argument_set",
            "A set an indexed argument of a contract ranges over, in index order.",
            [
                ("contract", "text", "The contract."),
                ("argument", "text", "The argument."),
                ("position", "integer", "Position among the argument's indices, from 1."),
                ("set_name", "text", "The contract set the index ranges over."),
            ],
            pk=["contract", "argument", "position"],
            fks=[
                (["contract", "argument"], "contract_argument", ["contract", "name"]),
                (["contract", "set_name"], "contract_set", ["contract", "name"]),
            ],
        )
    )
    add(
        _table(
            "contract_output",
            "An output of a contract.",
            [
                ("contract", "text", "The contract."),
                ("name", "text", "The output's name."),
                ("position", "integer", "Position, from 1."),
                *_TYPE_COLUMNS,
                (
                    "observable",
                    "uuid?",
                    "The declared observable entity the output denotes (its identifier).",
                ),
                (
                    "observable_from_set",
                    "boolean",
                    "Whether the observable is given by a slot of the implementing form (form_output_observable).",
                ),
                (
                    "extra_order",
                    "integer",
                    "For an output of a dependent quantity type: the extra concentration powers "
                    "of its dimension; zero otherwise.",
                ),
                ("doc", "text", "What the output is."),
            ],
            pk=["contract", "name"],
            unique=[["contract", "position"]],
            fks=[(["contract"], "contract", ["name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
            checks=[
                (
                    "observable_or_from_set",
                    'NOT ("observable_from_set" AND "observable" IS NOT NULL)',
                )
            ],
        )
    )
    add(
        _table(
            "form",
            "A model form: a catalogued family of equations.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<form> refers to."),
                ("name", "text", "The form's name."),
                ("module", "text", "The declaring module."),
                ("implements", "text", "The contract it implements."),
                ("completeness", "text", "How much of the form is declared."),
                (
                    "status",
                    "text",
                    "catalogued or expressed; whether a form is qualified is derived from recorded runs (qual.form_qualification).",
                ),
                ("doc", "text", "What the form is."),
            ],
            pk=["id"],
            unique=[["name"], ["name", "implements"]],
            fks=[(["module"], "module", ["name"]), (["implements"], "contract", ["name"])],
            vocabulary={
                "completeness": (
                    "fully_declared",
                    "structure_declared_equation_external",
                    "opaque_bundle",
                ),
                "status": ("catalogued", "expressed"),
            },
        )
    )
    add(
        _table(
            "form_citation",
            "A citation of a form, in declared order.",
            [
                ("form", "text", "The form."),
                ("position", "integer", "Position, from 1."),
                ("citation", "text", "The cited source key."),
            ],
            pk=["form", "position"],
            fks=[(["form"], "form", ["name"])],
        )
    )
    for label, what in (
        ("local", "a local binding, evaluated in declared order"),
        ("output", "the expression of one contract output"),
    ):
        add(
            _table(
                f"form_{label}",
                f"A form's {label}: {what}.",
                [
                    ("form", "text", "The form."),
                    *(
                        [("contract", "text", "The contract the form implements.")]
                        if label == "output"
                        else []
                    ),
                    ("name", "text", f"The {label}'s name."),
                    ("position", "integer", "Position in declared order, from 1."),
                    ("expression", "text", "The expression text as declared."),
                    (
                        "content_hash",
                        "text",
                        "SHA-256 (hex) of the canonical serialisation of the expression's typed tree.",
                    ),
                    *(
                        [
                            (
                                "evaluation_hash",
                                "text",
                                "SHA-256 (hex) of what evaluating this output reads from the form's text: its locals, this "
                                "output's expression and its implicit blocks, each in canonical serialisation; what a "
                                "qualification run records.",
                            )
                        ]
                        if label == "output"
                        else []
                    ),
                ],
                pk=["form", "name"],
                unique=[["form", "position"]],
                fks=(
                    [
                        (["form", "contract"], "form", ["name", "implements"]),
                        (["contract", "name"], "contract_output", ["contract", "name"]),
                    ]
                    if label == "output"
                    else [(["form"], "form", ["name"])]
                ),
                checks=[
                    ("content_hash", 'length("content_hash") = 64'),
                    *(
                        [("evaluation_hash", 'length("evaluation_hash") = 64')]
                        if label == "output"
                        else []
                    ),
                ],
            )
        )
    add(
        _table(
            "form_output_observable",
            "The slot that supplies the observable of an output a contract declares observable_from_set.",
            [
                ("form", "text", "The form."),
                ("contract", "text", "The contract the form implements."),
                ("output", "text", "The output."),
                (
                    "slot",
                    "text",
                    "The slot's qualified name (form.group.slot): a required observable reference.",
                ),
            ],
            pk=["form", "output"],
            fks=[
                (["form", "contract"], "form", ["name", "implements"]),
                (["contract", "output"], "contract_output", ["contract", "name"]),
                (["slot"], "slot", ["qualified_name"]),
            ],
        )
    )
    add(
        _table(
            "form_convention",
            "A convention fact a form reads (`convention.<name>` in its expressions, or "
            "`convention.<name>[i]` for one read per component): a quantity-typed attribute of "
            "the convention-set kind.",
            [
                ("form", "text", "The form."),
                ("name", "text", "The convention-set attribute the form reads."),
                ("position", "integer", "Position among the form's convention facts, from 1."),
                ("kind", "text", "The kind that declares the attribute (the kind bound to convention_set or one it extends)."),
                (
                    "slot_group",
                    "text?",
                    "For a fact read per component: the slot group of the form whose set for a component names the parameterization the component's fact is taken from.",
                ),
                (
                    "over",
                    "text?",
                    "For a fact read per component: the contract set the group's subject is bound to.",
                ),
            ],
            pk=["form", "name"],
            unique=[["form", "position"]],
            fks=[
                (["form"], "form", ["name"]),
                (["kind", "name"], "attribute", ["kind", "name"]),
                (["slot_group"], "slot_group", ["qualified_name"]),
            ],
            checks=[
                (
                    "component_fact_complete",
                    '("slot_group" IS NULL) = ("over" IS NULL)',
                )
            ],
        )
    )
    add(
        _table(
            "form_implicit",
            "An implicit block of a form: unknowns defined by residuals that vanish.",
            [
                ("form", "text", "The form."),
                ("name", "text", "The block's name within the form."),
                ("position", "integer", "Position among the form's blocks, from 1."),
                ("select_rule", "text", "How one root is chosen: unique, smallest, largest or by."),
                (
                    "select_expression",
                    "text?",
                    "For `by`: the expression minimised over the roots, as declared.",
                ),
                (
                    "select_hash",
                    "text?",
                    "For `by`: SHA-256 (hex) of the canonical serialisation of its typed tree.",
                ),
                ("doc", "text", "What the block determines."),
            ],
            pk=["form", "name"],
            unique=[["form", "position"]],
            fks=[(["form"], "form", ["name"])],
            vocabulary={"select_rule": ("unique", "smallest", "largest", "by")},
            checks=[
                (
                    "select",
                    '("select_rule" = \'by\') = ("select_expression" IS NOT NULL) '
                    'AND ("select_expression" IS NULL) = ("select_hash" IS NULL) '
                    'AND ("select_hash" IS NULL OR length("select_hash") = 64)',
                )
            ],
        )
    )
    add(
        _table(
            "form_unknown",
            "An unknown of an implicit block: a quantity, optionally over sets of the contract.",
            [
                ("form", "text", "The form."),
                ("block", "text", "The implicit block."),
                ("name", "text", "The unknown's name."),
                ("position", "integer", "Position among the block's unknowns, from 1."),
                *_TYPE_COLUMNS,
            ],
            pk=["form", "block", "name"],
            unique=[["form", "block", "position"]],
            fks=[(["form", "block"], "form_implicit", ["form", "name"])],
            vocabulary={"container": CONTAINERS, "element_kind": ELEMENT_KINDS},
        )
    )
    add(
        _table(
            "form_unknown_set",
            "A set an unknown of an implicit block ranges over, in index order.",
            [
                ("form", "text", "The form."),
                ("contract", "text", "The contract the form implements."),
                ("block", "text", "The implicit block."),
                ("unknown", "text", "The unknown."),
                ("position", "integer", "Position among the unknown's indices, from 1."),
                ("set_name", "text", "The contract set the index ranges over."),
            ],
            pk=["form", "block", "unknown", "position"],
            fks=[
                (["form", "block", "unknown"], "form_unknown", ["form", "block", "name"]),
                (["form", "contract"], "form", ["name", "implements"]),
                (["contract", "set_name"], "contract_set", ["contract", "name"]),
            ],
        )
    )
    add(
        _table(
            "form_unknown_bound",
            "A bound or the start of an unknown: a number in storage units, or an expression.",
            [
                ("form", "text", "The form."),
                ("block", "text", "The implicit block."),
                ("unknown", "text", "The unknown."),
                ("bound", "text", "lower, upper or start."),
                (
                    "number",
                    "double precision?",
                    "The value, in storage units, when a number was declared.",
                ),
                ("expression", "text?", "The expression text as declared, when one was."),
                (
                    "content_hash",
                    "text?",
                    "SHA-256 (hex) of the canonical serialisation of the expression's typed tree.",
                ),
            ],
            pk=["form", "block", "unknown", "bound"],
            fks=[(["form", "block", "unknown"], "form_unknown", ["form", "block", "name"])],
            vocabulary={"bound": ("lower", "upper", "start")},
            checks=[
                ("value", '("number" IS NULL) <> ("expression" IS NULL)'),
                (
                    "content_hash",
                    '("expression" IS NULL) = ("content_hash" IS NULL) '
                    'AND ("content_hash" IS NULL OR length("content_hash") = 64)',
                ),
            ],
        )
    )
    add(
        _table(
            "form_residual",
            "A residual of an implicit block: an expression that vanishes at the solution, or one "
            "with `for` clauses that states a residual for each element.",
            [
                ("form", "text", "The form."),
                ("block", "text", "The implicit block."),
                ("position", "integer", "Position among the block's residuals, from 1."),
                ("expression", "text", "The residual text as declared."),
                (
                    "content_hash",
                    "text",
                    "SHA-256 (hex) of the canonical serialisation of the residual's typed tree.",
                ),
            ],
            pk=["form", "block", "position"],
            fks=[(["form", "block"], "form_implicit", ["form", "name"])],
            checks=[("content_hash", 'length("content_hash") = 64')],
        )
    )
    add(
        _table(
            "slot_group",
            "The parameters one source fits together for one subject tuple.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<slot_group> refers to."),
                ("qualified_name", "text", "form.group."),
                ("form", "text", "The form."),
                ("name", "text", "The group's name within the form."),
                ("kind", "text", "The kind the group expands to."),
                ("table_name", "text", "The group's table in schema param."),
                ("doc", "text", "What the group fits."),
            ],
            pk=["id"],
            unique=[["qualified_name"]],
            fks=[(["form"], "form", ["name"]), (["kind"], "kind", ["name"])],
        )
    )
    add(
        _table(
            "slot_group_subject",
            "A subject role of a slot group.",
            [
                ("slot_group", "text", "The group's qualified name."),
                ("name", "text", "The role's name."),
                ("position", "integer", "Position in the subject tuple, from 1."),
                ("kind", "text", "The kind the role refers to."),
                (
                    "binds_to_type",
                    "text?",
                    "What the role is bound to in the form's contract: role or set.",
                ),
                ("binds_to", "text?", "The contract role or set the role is bound to."),
                ("doc", "text", "What the role is."),
            ],
            pk=["slot_group", "name"],
            unique=[["slot_group", "position"]],
            fks=[(["slot_group"], "slot_group", ["qualified_name"]), (["kind"], "kind", ["name"])],
            vocabulary={"binds_to_type": ("role", "set")},
            checks=[("binding", '("binds_to_type" IS NULL) = ("binds_to" IS NULL)')],
        )
    )
    add(
        _table(
            "family",
            "Slots that repeat over an index, projected to a child table of a slot group.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<family> refers to."),
                ("qualified_name", "text", "form.group.family."),
                ("slot_group", "text", "The group's qualified name."),
                ("name", "text", "The family's name within the group."),
                ("table_name", "text", "The family's table in schema param."),
                (
                    "interval_lower",
                    "text?",
                    "The slot bounding each piece below, if the family has an interval.",
                ),
                ("interval_upper", "text?", "The slot bounding each piece above."),
                ("doc", "text", "What the family holds."),
            ],
            pk=["id"],
            unique=[["qualified_name"]],
            fks=[(["slot_group"], "slot_group", ["qualified_name"])],
            checks=[
                (
                    "interval",
                    '("interval_lower" IS NULL) = ("interval_upper" IS NULL)',
                )
            ],
        )
    )
    add(
        _table(
            "family_index",
            "An index of a family.",
            [
                ("family", "text", "The family's qualified name."),
                ("name", "text", "The index's name."),
                ("position", "integer", "Position in the key, from 1."),
                ("element_kind", "text", "primitive, enum and so on."),
                ("element", "text", "The index type."),
                ("minimum", "bigint?", "The smallest index value, for an Integer index."),
                ("doc", "text", "What the index counts."),
            ],
            pk=["family", "name"],
            unique=[["family", "position"]],
            fks=[(["family"], "family", ["qualified_name"])],
            vocabulary={"element_kind": ELEMENT_KINDS},
        )
    )
    add(
        _table(
            "slot",
            "A slot of a slot group or of one of its families.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<slot> refers to."),
                ("qualified_name", "text", "form.group.slot, or form.group.family.slot."),
                ("slot_group", "text", "The group's qualified name."),
                ("family", "text?", "The family's qualified name, for a family slot."),
                ("name", "text", "The slot's name."),
                ("position", "integer", "Position among the owner's slots, from 1."),
                (
                    "shape",
                    "text",
                    "quantity, enum, reference, tabulated_function, nested_set or set_reference.",
                ),
                (
                    "element_kind",
                    "text?",
                    "The value's element kind; null for a nested set or a set reference.",
                ),
                ("element", "text?", "The value's type; null for a nested set or a set reference."),
                ("accepts", "text?", "For a nested set: the contract its form implements."),
                (
                    "references_contract",
                    "text?",
                    "For a set reference: the contract the form of the referenced set implements.",
                ),
                ("presence", "text", "required or stateful."),
                (
                    "observable",
                    "uuid?",
                    "The declared observable entity the slot denotes (its identifier).",
                ),
                (
                    "extra_order",
                    "integer",
                    "For a slot of a dependent quantity type: the extra concentration powers of "
                    "its dimension; zero otherwise.",
                ),
                ("doc", "text", "What the slot is."),
            ],
            pk=["id"],
            unique=[["qualified_name"]],
            nulls_not_distinct=[["slot_group", "family", "name"]],
            fks=[
                (["slot_group"], "slot_group", ["qualified_name"]),
                (["family"], "family", ["qualified_name"]),
                (["accepts"], "contract", ["name"]),
                (["references_contract"], "contract", ["name"]),
            ],
            vocabulary={
                "shape": SHAPES,
                "presence": ("required", "stateful"),
                "element_kind": ELEMENT_KINDS,
            },
            checks=[
                ("nested_set", '("shape" = \'nested_set\') = ("accepts" IS NOT NULL)'),
                (
                    "set_reference",
                    '("shape" = \'set_reference\') = ("references_contract" IS NOT NULL)',
                ),
            ],
        )
    )
    add(
        _table(
            "subform_slot",
            "A slot for a sub-form: a choice of form implementing a contract.",
            [
                ("id", "uuid", "Identifier of the row; what Meta<subform_slot> refers to."),
                ("qualified_name", "text", "form.subform."),
                ("form", "text", "The form."),
                ("name", "text", "The sub-form slot's name."),
                ("position", "integer", "Position among the form's sub-form slots, from 1."),
                ("accepts", "text", "The contract a chosen sub-form implements."),
                ("multiplicity", "text", "one, optional or many."),
                ("per", "text", "model or subject: what a choice is made for."),
                ("doc", "text", "What the sub-form is."),
            ],
            pk=["id"],
            unique=[["qualified_name"]],
            fks=[(["form"], "form", ["name"]), (["accepts"], "contract", ["name"])],
            vocabulary={"multiplicity": ("one", "optional", "many"), "per": ("model", "subject")},
        )
    )
    add(
        _table(
            "entity",
            "A declared entity: an instance the declaration itself asserts.",
            [
                ("kind", "text", "The entity's kind."),
                ("name", "text", "The declared name."),
                ("id", "uuid", "The deterministic identifier of the instance."),
                ("doc", "text", "What the entity is."),
            ],
            pk=["kind", "name"],
            unique=[["id"]],
            fks=[(["kind"], "kind", ["name"])],
        )
    )
    add(
        _table(
            "trace",
            "A handoff dictionary identifier a construct traces to.",
            [
                ("construct", "text", "The construct path, e.g. kinds.species.attributes.charge."),
                ("trace", "text", "The identifier, e.g. IC-13."),
            ],
            pk=["construct", "trace"],
        )
    )
    add(
        _table(
            "pse_mark",
            "A construct the .pse language cannot express today.",
            [
                ("construct", "text", "The construct path."),
                ("mark", "text", "gap:<short name>."),
            ],
            pk=["construct"],
        )
    )
    return {table.name: table for table in t}


META_TABLES: dict[str, ir.Table] = _build()
