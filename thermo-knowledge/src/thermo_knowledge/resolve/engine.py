# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The resolution rules of pipeline section 1, as a pure function.

`resolve` takes every source entity of every carrier, the curated decisions and a function from
a structural identifier to its InChIKey, and returns which canonical species each entity
resolves to and with what status and rule. It reads no file and no clock, and its result does
not depend on the order of its inputs: entities are processed in key order and every
collection it builds is a set or sorted.

The rules apply in order and the first that applies decides (curated, structural, registry,
formula-scoped, provisional); no rule ever takes the first of several candidates. An entity
with conflicting evidence is `ambiguous`: it keeps its candidates, resolves to no canonical
species and gets a provisional one.
"""

from __future__ import annotations

import unicodedata
from collections import defaultdict
from collections.abc import Callable, Mapping, Sequence
from dataclasses import dataclass, field, replace

from thermo_knowledge import identity
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.provenance import Origin
from thermo_knowledge.declaration import model as m
from thermo_knowledge.resolve.decisions import Decisions, EntityKey
from thermo_knowledge.resolve.structure import STANDARD_INCHIKEY, StructureKey

UNIQUE = pc.RESOLUTION_STATUS.member("unique")
AMBIGUOUS = pc.RESOLUTION_STATUS.member("ambiguous")
UNRESOLVED = pc.RESOLUTION_STATUS.member("unresolved")
REJECTED = pc.RESOLUTION_STATUS.member("rejected")
CURATED = pc.RESOLUTION_RULE.member("curated")
STRUCTURAL = pc.RESOLUTION_RULE.member("structural")
REGISTRY = pc.RESOLUTION_RULE.member("registry")
FORMULA_SCOPE = pc.RESOLUTION_RULE.member("formula_scope")
PROVISIONAL = pc.RESOLUTION_RULE.member("provisional")
FORMULA_SCHEME = "formula"
NAME_SCHEME = "name"
PROVISIONAL_PREFIX = "provisional:"

type Structure = Callable[[str, str], StructureKey]


class ResolveError(Exception):
    """Phase-1 claims that contradict each other, or a decision that cannot be applied."""


@dataclass(frozen=True)
class SchemeKinds:
    """Which naming schemes encode structure and which are registries, as declared: the
    `structural` and `registry` attributes of the declared `naming_scheme` entities."""

    structural: frozenset[str]
    registry: frozenset[str]


def scheme_kinds(decl: m.Declaration) -> SchemeKinds:
    """The structural and registry schemes the declaration's naming schemes state."""
    scheme = pc.NAMING_SCHEME
    schemes = [e for e in decl.entities if e.kind == scheme.declared]
    return SchemeKinds(
        structural=frozenset(e.name for e in schemes if e.values.get(scheme.structural) is True),
        registry=frozenset(e.name for e in schemes if e.values.get(scheme.registry) is True),
    )


@dataclass(frozen=True)
class Assertion:
    scheme: str
    value: str
    column: str


@dataclass
class SourceEntityRec:
    """A source entity with every claim made about it."""

    key: EntityKey
    carrier_key: str
    aggregation: str | None
    polymorph: str | None
    stated_charge: int | None
    origins: tuple[Origin, ...]
    assertions: tuple[Assertion, ...]

    @property
    def manifest_id(self) -> str:
        return self.key[0]

    @property
    def scope(self) -> str:
        return self.key[1]

    @property
    def local_key(self) -> str:
        return self.key[2]


@dataclass(frozen=True)
class FormulaScopes:
    """The scopes each carrier declares formula-identified: (manifest id, scope) to discriminator."""

    scopes: Mapping[tuple[str, str], str | None]


@dataclass(frozen=True)
class SpeciesRec:
    canonical_key: str
    inchikey: str | None
    charge: int
    provisional: bool
    label: str
    origins: tuple[Origin, ...]


@dataclass(frozen=True)
class FormRec:
    canonical_key: str
    species_key: str
    aggregation: str
    polymorph: str | None
    provisional: bool
    label: str
    origins: tuple[Origin, ...]


@dataclass(frozen=True)
class EntityResult:
    key: EntityKey
    status: str
    rule: str
    target_key: str
    """The canonical key of the material entity its records attach to."""
    species_key: str
    candidates: tuple[str, ...]
    reason: str | None = None


@dataclass
class Resolution:
    species: dict[str, SpeciesRec] = field(default_factory=dict)
    forms: dict[str, FormRec] = field(default_factory=dict)
    entities: dict[EntityKey, EntityResult] = field(default_factory=dict)
    registry_conflicts: dict[tuple[str, str], tuple[str, ...]] = field(default_factory=dict)
    unusable: list[tuple[EntityKey, str, str, str]] = field(default_factory=list)
    """(entity, scheme, value, problem) of structural identifiers no key could be computed from."""
    unmatched_decisions: list[EntityKey] = field(default_factory=list)


@dataclass(frozen=True)
class _Proposal:
    rule: str
    keys: tuple[str, ...]
    reason: str | None = None


def provisional_key(carrier_key: str, scope: str, local_key: str) -> str:
    """The canonical key of the provisional entity of one source entity: its own identity."""
    return PROVISIONAL_PREFIX + identity.canonical_encoding([carrier_key, scope, local_key])


def formula_key(formula: str, charge: int, discriminator: str | None) -> str:
    text = f"formula:{formula}:{charge:+d}"
    return text if discriminator is None else f"{text}:{discriminator}"


def form_key(species_key: str, aggregation: str, polymorph: str | None) -> str:
    parts = [species_key, aggregation] + ([polymorph] if polymorph else [])
    return identity.canonical_encoding(parts)


def _dedupe(origins: Sequence[Origin]) -> tuple[Origin, ...]:
    return tuple(
        sorted(set(origins), key=lambda o: (o.ref.carrier, o.ref.artifact, o.ref.locator, o.role))
    )


def resolve(
    entities: Sequence[SourceEntityRec],
    decisions: Decisions,
    formula_scopes: FormulaScopes,
    structure: Structure,
    schemes: SchemeKinds,
) -> Resolution:
    """Resolve every source entity."""
    result = Resolution()
    ordered = sorted(entities, key=lambda e: e.key)
    by_key = {e.key: e for e in ordered}
    result.unmatched_decisions = sorted(decisions.named() - set(by_key))

    # Rule 1 names entities; rules 2 to 4 propose for the rest.
    proposals: dict[EntityKey, _Proposal] = {}
    structural: dict[EntityKey, list[StructureKey]] = {}
    for entity in ordered:
        if entity.key in decisions.identify or entity.key in decisions.reject:
            continue
        found: list[StructureKey] = []
        for item in entity.assertions:
            if item.scheme not in schemes.structural:
                continue
            key = structure(item.scheme, item.value)
            if key.inchikey is None:
                result.unusable.append((entity.key, item.scheme, item.value, key.problem or ""))
            else:
                found.append(key)
        structural[entity.key] = found
        keys = tuple(sorted({k.inchikey for k in found if k.inchikey}))
        if keys:
            reason = None
            if len(keys) > 1:
                reason = "structural identifiers give different InChIKeys"
            proposals[entity.key] = _Proposal(STRUCTURAL, keys, reason)

    # Rule 3: the corpus maps each registry identifier to the structures of the entities that
    # carry it with a structure.
    registry: dict[tuple[str, str], set[str]] = defaultdict(set)
    for entity in ordered:
        proposal = proposals.get(entity.key)
        if proposal is None or len(proposal.keys) != 1:
            continue
        for item in entity.assertions:
            if item.scheme in schemes.registry:
                registry[(item.scheme, item.value)].add(proposal.keys[0])
    result.registry_conflicts = {
        key: tuple(sorted(found)) for key, found in sorted(registry.items()) if len(found) > 1
    }
    for entity in ordered:
        if (
            entity.key in proposals
            or entity.key in decisions.identify
            or entity.key in decisions.reject
        ):
            continue
        found_keys: set[str] = set()
        for item in entity.assertions:
            if item.scheme in schemes.registry:
                found_keys |= registry.get((item.scheme, item.value), set())
        if found_keys:
            reason = (
                "a registry identifier maps to several structures" if len(found_keys) > 1 else None
            )
            proposals[entity.key] = _Proposal(REGISTRY, tuple(sorted(found_keys)), reason)

    # Rule 4: a formula and charge identify the entities of a declared scope.
    for entity in ordered:
        if (
            entity.key in proposals
            or entity.key in decisions.identify
            or entity.key in decisions.reject
        ):
            continue
        scope = (entity.manifest_id, entity.scope)
        if scope not in formula_scopes.scopes:
            continue
        formulas = sorted(
            {
                unicodedata.normalize("NFC", a.value).strip()
                for a in entity.assertions
                if a.scheme == FORMULA_SCHEME
            }
        )
        if formulas:
            charge = entity.stated_charge or 0
            discriminator = formula_scopes.scopes[scope]
            proposals[entity.key] = _Proposal(
                FORMULA_SCOPE,
                tuple(formula_key(f, charge, discriminator) for f in formulas),
                "several formulas are asserted" if len(formulas) > 1 else None,
            )

    # Curated `distinct`: two entities a rule would put in one species are neither resolved.
    merged_away: dict[EntityKey, str] = {}
    for first, second, reason in decisions.distinct:
        one, two = proposals.get(first), proposals.get(second)
        if one and two and len(one.keys) == 1 and one.keys == two.keys:
            for key in (first, second):
                merged_away[key] = (
                    f"a curated decision says {first} and {second} are distinct: {reason}"
                )

    # Species and their charges.
    charges = _charges(ordered, structural)
    results: dict[EntityKey, EntityResult] = {}
    needed: dict[str, list[SourceEntityRec]] = defaultdict(list)
    for entity in ordered:
        key = entity.key
        if key in decisions.identify:
            decision = decisions.identify[key]
            needed[decision.canonical_key].append(entity)
            charges.setdefault(decision.canonical_key, decision.charge)
            results[key] = EntityResult(
                key, UNIQUE, CURATED, decision.canonical_key, decision.canonical_key, ()
            )
            continue
        if key in decisions.reject:
            results[key] = _provisional(entity, REJECTED, CURATED, decisions.reject[key])
            continue
        proposal = proposals.get(key)
        if proposal is None:
            results[key] = _provisional(entity, UNRESOLVED, PROVISIONAL, None)
        elif len(proposal.keys) > 1 or key in merged_away:
            for candidate in proposal.keys:
                needed[candidate].append(entity)
            results[key] = _provisional(
                entity,
                AMBIGUOUS,
                proposal.rule,
                merged_away.get(key, proposal.reason),
                proposal.keys,
            )
        else:
            target = proposal.keys[0]
            stated = entity.stated_charge
            if stated is not None and target in charges and charges[target] != stated:
                needed[target].append(entity)
                results[key] = _provisional(
                    entity,
                    AMBIGUOUS,
                    proposal.rule,
                    f"the stated charge {stated:+d} differs from the structure's {charges[target]:+d}",
                    (target,),
                )
                continue
            needed[target].append(entity)
            results[key] = EntityResult(key, UNIQUE, proposal.rule, target, target, ())

    # Canonical species: members are the entities resolved to them; candidates add origins only.
    members: dict[str, list[SourceEntityRec]] = defaultdict(list)
    for key, outcome in results.items():
        if outcome.status == UNIQUE:
            members[outcome.species_key].append(by_key[key])
    for canonical_key in sorted(needed):
        carriers = members.get(canonical_key) or needed[canonical_key]
        is_structural = bool(STANDARD_INCHIKEY.match(canonical_key))
        result.species[canonical_key] = SpeciesRec(
            canonical_key=canonical_key,
            inchikey=canonical_key if is_structural else None,
            charge=charges.get(canonical_key, _stated_charge(carriers)),
            provisional=False,
            label=label(carriers),
            origins=_dedupe([o for e in needed[canonical_key] for o in e.origins]),
        )
    form_origins: dict[str, list[Origin]] = defaultdict(list)
    for key, outcome in sorted(results.items()):
        entity = by_key[key]
        if outcome.species_key.startswith(PROVISIONAL_PREFIX):
            result.species[outcome.species_key] = SpeciesRec(
                canonical_key=outcome.species_key,
                inchikey=None,
                charge=entity.stated_charge or 0,
                provisional=True,
                label=entity.local_key,
                origins=_dedupe(entity.origins),
            )
        if entity.aggregation is not None:
            target = form_key(outcome.species_key, entity.aggregation, entity.polymorph)
            form_origins[target].extend(entity.origins)
            if target not in result.forms:
                species = result.species[outcome.species_key]
                suffix = f", {entity.polymorph}" if entity.polymorph else ""
                result.forms[target] = FormRec(
                    canonical_key=target,
                    species_key=outcome.species_key,
                    aggregation=entity.aggregation,
                    polymorph=entity.polymorph,
                    provisional=species.provisional,
                    label=f"{species.label} ({entity.aggregation}{suffix})",
                    origins=(),
                )
            outcome = EntityResult(
                outcome.key,
                outcome.status,
                outcome.rule,
                target,
                outcome.species_key,
                outcome.candidates,
                outcome.reason,
            )
        result.entities[key] = outcome
    for target, found in form_origins.items():
        result.forms[target] = replace(result.forms[target], origins=_dedupe(found))
    return result


def _provisional(
    entity: SourceEntityRec,
    status: str,
    rule: str,
    reason: str | None,
    candidates: tuple[str, ...] = (),
) -> EntityResult:
    key = provisional_key(entity.carrier_key, entity.scope, entity.local_key)
    return EntityResult(entity.key, status, rule, key, key, candidates, reason)


def _charges(
    entities: Sequence[SourceEntityRec], structural: Mapping[EntityKey, list[StructureKey]]
) -> dict[str, int]:
    """The charge of each InChIKey the structures establish (an InChIKey encodes its charge, so
    the structures of one key agree)."""
    evidence: dict[str, set[int]] = defaultdict(set)
    for entity in entities:
        for item in structural.get(entity.key, ()):
            if item.inchikey and item.charge is not None:
                evidence[item.inchikey].add(item.charge)
    return {key: sorted(values)[0] for key, values in evidence.items()}


def _stated_charge(entities: Sequence[SourceEntityRec]) -> int:
    stated = sorted({e.stated_charge for e in entities if e.stated_charge is not None})
    return stated[0] if stated else 0


def label(entities: Sequence[SourceEntityRec]) -> str:
    """The display label of a species: never identity, chosen deterministically.

    Among the `name` assertions of the entities, the name asserted by the most distinct source
    entities; ties go to a name that is the local key of one of its entities, then to the
    smaller name in code-point order. Without a name, the smallest formula assertion; without
    that, the smallest local key."""
    by_name: dict[str, set[EntityKey]] = defaultdict(set)
    local_keys = {e.local_key for e in entities}
    for entity in entities:
        for item in entity.assertions:
            if item.scheme == NAME_SCHEME:
                by_name[unicodedata.normalize("NFC", item.value)].add(entity.key)
    if by_name:
        return min(
            by_name,
            key=lambda name: (-len(by_name[name]), 0 if name in local_keys else 1, name),
        )
    formulas = sorted(
        unicodedata.normalize("NFC", a.value)
        for e in entities
        for a in e.assertions
        if a.scheme == FORMULA_SCHEME
    )
    if formulas:
        return formulas[0]
    return min(local_keys)


__all__ = [
    "Assertion",
    "EntityResult",
    "FormRec",
    "FormulaScopes",
    "Resolution",
    "ResolveError",
    "SchemeKinds",
    "SourceEntityRec",
    "SpeciesRec",
    "formula_key",
    "label",
    "provisional_key",
    "resolve",
    "scheme_kinds",
]
