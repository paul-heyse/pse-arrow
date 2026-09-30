# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""`identity/decisions.toml`: the only hand-maintained resolution authority (pipeline section 1).

```toml
[[decision]]
action = "identify"            # the entity is this canonical entity
entity = { carrier = "coolprop", scope = "fluids", key = "R1123" }
class = "species"              # the class of the canonical entity it names (see below)
canonical_key = "ZWQIPCQCAOKXDI-UHFFFAOYSA-N"   # an InChIKey, or another declared canonical key
charge = 0                     # optional, default 0, species only
reason = "why"

[[decision]]
action = "distinct"            # these entities are different identities, whatever their identifiers say
entities = [
  { carrier = "coolprop", scope = "fluids", key = "A" },
  { carrier = "thermo", scope = "chemicals", key = "B" },
]
reason = "why"

[[decision]]
action = "reject"              # the source's identification of the entity is wrong
entity = { carrier = "coolprop", scope = "fluids", key = "C" }
reason = "why"
```

An entity is named by the manifest id of its carrier, its scope and its local key. At most one
`identify` or `reject` decision names an entity.

An `identify` decision states the class of the canonical entity it names: `species`,
`defined_mixture`, `material` or `polymer_type`. It must agree with the class the source states
for the entity (a source entity of class `species_form` is identified through its species, so the
decision's class is `species`), unless the source states none (`undetermined`), when the decision
settles it.
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Literal

import msgspec
from msgspec import Struct

from thermo_knowledge import config

DECISIONS_NAME = "decisions.toml"

type EntityKey = tuple[str, str, str]


class EntityRef(Struct, forbid_unknown_fields=True):
    carrier: str
    scope: str
    key: str

    @property
    def tuple(self) -> EntityKey:
        return (self.carrier, self.scope, self.key)


class Decision(Struct, forbid_unknown_fields=True):
    action: Literal["identify", "distinct", "reject"]
    reason: str
    entity: EntityRef | None = None
    entities: list[EntityRef] = []
    entity_class: str | None = msgspec.field(name="class", default=None)
    canonical_key: str | None = None
    charge: int | None = None


class DecisionsFile(Struct, forbid_unknown_fields=True):
    decision: list[Decision] = []


class DecisionError(Exception):
    """A decisions file that is unreadable or inconsistent."""

    def __init__(self, problems: list[str]) -> None:
        self.problems = problems
        super().__init__("\n".join(problems))


IDENTIFIABLE_CLASSES = ("species", "defined_mixture", "material", "polymer_type")
"""The classes a curated decision can name an entity of: those with rows a canonical key alone
identifies. A pseudo-component also needs the `kind` that says why it has no formula."""


@dataclass(frozen=True)
class Identify:
    canonical_key: str
    entity_class: str
    charge: int
    reason: str


@dataclass(frozen=True)
class Decisions:
    """The decisions of a file, indexed for resolution."""

    identify: dict[EntityKey, Identify]
    reject: dict[EntityKey, str]
    distinct: tuple[tuple[EntityKey, EntityKey, str], ...]

    def named(self) -> set[EntityKey]:
        """Every entity any decision names."""
        names = set(self.identify) | set(self.reject)
        for first, second, _ in self.distinct:
            names.update((first, second))
        return names


EMPTY = Decisions({}, {}, ())


def default_path() -> Path:
    return config.TREE_DIR / "identity" / DECISIONS_NAME


def parse(text: str, *, file: str) -> Decisions:
    """Decode and check a decisions file; raises `DecisionError` naming each problem."""
    try:
        data = tomllib.loads(text)
        parsed = msgspec.convert(data, DecisionsFile, strict=True)
    except tomllib.TOMLDecodeError as error:
        raise DecisionError([f"{file}: invalid TOML: {error}"]) from error
    except msgspec.ValidationError as error:
        raise DecisionError([f"{file}: {error}"]) from error
    problems: list[str] = []
    identify: dict[EntityKey, Identify] = {}
    reject: dict[EntityKey, str] = {}
    distinct: list[tuple[EntityKey, EntityKey, str]] = []
    for index, item in enumerate(parsed.decision):
        where = f"{file}: decision[{index}] ({item.action})"
        if not item.reason.strip():
            problems.append(f"{where}: a decision carries a reason")
        if item.action == "distinct":
            if len(item.entities) != 2 or item.entity is not None:
                problems.append(f"{where}: `distinct` names exactly two `entities`")
                continue
            first, second = item.entities[0].tuple, item.entities[1].tuple
            if first == second:
                problems.append(f"{where}: the two entities are the same entity")
                continue
            if (
                item.canonical_key is not None
                or item.charge is not None
                or item.entity_class is not None
            ):
                problems.append(
                    f"{where}: `canonical_key`, `charge` and `class` belong to `identify`"
                )
            distinct.append((min(first, second), max(first, second), item.reason))
            continue
        if item.entity is None or item.entities:
            problems.append(f"{where}: names exactly one `entity`")
            continue
        key = item.entity.tuple
        if key in identify or key in reject:
            problems.append(f"{where}: another decision already identifies or rejects {key}")
            continue
        if item.action == "identify":
            if not item.canonical_key:
                problems.append(f"{where}: `identify` states the `canonical_key`")
                continue
            if item.entity_class not in IDENTIFIABLE_CLASSES:
                problems.append(
                    f"{where}: `identify` states the `class` of the entity it names, one of "
                    f"{', '.join(IDENTIFIABLE_CLASSES)}"
                )
                continue
            if item.charge is not None and item.entity_class != "species":
                problems.append(f"{where}: `charge` belongs to a species")
            identify[key] = Identify(
                item.canonical_key, item.entity_class, item.charge or 0, item.reason
            )
        else:
            if (
                item.canonical_key is not None
                or item.charge is not None
                or item.entity_class is not None
            ):
                problems.append(
                    f"{where}: `canonical_key`, `charge` and `class` belong to `identify`"
                )
            reject[key] = item.reason
    if problems:
        raise DecisionError(problems)
    return Decisions(identify, reject, tuple(sorted(distinct)))


def load(path: Path | None = None) -> tuple[Decisions, bytes]:
    """The decisions file (an absent file has none) and its bytes."""
    target = path if path is not None else default_path()
    if not target.is_file():
        return EMPTY, b""
    data = target.read_bytes()
    return parse(data.decode("utf-8"), file=str(target)), data
