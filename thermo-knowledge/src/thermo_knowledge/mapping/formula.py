# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The one text a formula is asserted as, so that carriers that give the same composition give the
same formula assertion.

Resolution rule 4 (pipeline section 1) identifies a species of a formula-identified scope by its
formula text and charge, and does not parse the text. Two carriers therefore agree on a species
only if their mappings write one composition as one text. Each of them builds it with `hill`, the
Hill system: carbon first and hydrogen second where the composition has carbon, every other element
after in alphabetical order (all of them in alphabetical order where there is no carbon); an atom
count of one is left out, any other count follows its symbol as the shortest decimal that reads back
as the source's number.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence

from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.values import Quantity
from thermo_knowledge.mapping.claims import decimal_text
from thermo_knowledge.mapping.context import RowHeld, RunContext
from thermo_knowledge.mapping.spec import declared_entity
from thermo_knowledge.mapping.staged import SourceRow

CARBON = "C"
HYDROGEN = "H"
CHARGE = "charge"
"""The declared conserved quantity of electric charge."""


def hill(counts: Mapping[str, float]) -> str:
    """The Hill formula of a composition (element symbol to atoms, each more than zero): the
    empty text for no atoms, as for an electron."""
    present = {symbol: count for symbol, count in counts.items() if count != 0}
    if any(count < 0 for count in present.values()):
        raise ValueError(f"a formula has no negative atom counts: {dict(present)!r}")
    if CARBON in present:
        first = [s for s in (CARBON, HYDROGEN) if s in present]
        order = first + sorted(s for s in present if s not in first)
    else:
        order = sorted(present)
    return "".join(_term(symbol, present[symbol]) for symbol in order)


def _term(symbol: str, count: float) -> str:
    return symbol if count == 1 else f"{symbol}{decimal_text(count)}"


def composition(
    ctx: RunContext[object], rows: Sequence[SourceRow], *, symbol: str
) -> tuple[dict[str, float], int]:
    """The composition the rows of one species state, as `(atoms of each element by symbol,
    charge number)`: the rows are pairs whose rules target `composition.quantity` (the element
    `symbol` names, through the rule's `case`) and `composition.value`; a pair whose quantity
    the partition's constants make the charge adds to the charge. An element the declaration does
    not hold holds the block (`unknown_subject`)."""
    charge_entity = declared_entity(ctx.decl, "conserved_quantity", CHARGE)
    assert charge_entity is not None  # the declaration holds it
    counts: dict[str, float] = {}
    charge = 0
    for row in rows:
        found = ctx.attributes(row, pc.COMPOSITION.declared)
        stated = found.get(pc.COMPOSITION.value)
        if stated is None:
            raise RowHeld("not_a_number", f"{row.locator}: the pair states no count")
        amount = stated.value if isinstance(stated, Quantity) else float(stated)  # type: ignore[arg-type]
        if found.get(pc.COMPOSITION.quantity) == charge_entity.id:
            if amount != int(amount):
                raise RowHeld("not_an_integer", f"{row.locator}: a charge of {amount!r}")
            charge += int(amount)
        else:
            name = ctx.name(row, symbol)
            counts[name] = counts.get(name, 0.0) + amount
    return counts, charge
