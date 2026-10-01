# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The resolution rules of pipeline section 1, as a pure function.

`resolve` takes every source entity of every carrier, the curated decisions and a function from
a structural identifier to its InChIKey, and returns which canonical species each entity
resolves to and with what status and rule. It reads no file and no clock, and its result does
not depend on the order of its inputs: entities are processed in key order and every
collection it builds is a set or sorted.

The rules depend on the class of the source entity (pipeline section 1), which the mapping
declares and resolution never infers:

- a `species` and a `species_form` go through the rules in order, the first that applies deciding
  (curated, structural, registry, formula-scoped, provisional); a form then takes the species
  its entity resolves to and the aggregation it states;
- a `defined_mixture` resolves to the mixture whose canonical key is built from its components'
  resolved keys, their fractions as decimal text and the basis, and is unique only when every
  component is;
- a `material` resolves by a registry identifier of a scheme declared as a registry;
- a `pseudo_component` has no rules yet (the mapping refuses the class) and a `polymer_type` is
  provisional unless a curated decision identifies it;
- an `undetermined` entity is provisional and unclassified unless a curated decision settles it.

No rule ever takes the first of several candidates. An entity with conflicting evidence is
`ambiguous`: it keeps its candidates, resolves to no canonical entity and gets a provisional one
of its own class. No provisional entity is a species unless its class is species.
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
COMPOSITION = pc.RESOLUTION_RULE.member("composition")
SPECIES = pc.ENTITY_CLASS.member("species")
SPECIES_FORM = pc.ENTITY_CLASS.member("species_form")
DEFINED_MIXTURE = pc.ENTITY_CLASS.member("defined_mixture")
MATERIAL = pc.ENTITY_CLASS.member("material")
POLYMER_TYPE = pc.ENTITY_CLASS.member("polymer_type")
UNDETERMINED = pc.ENTITY_CLASS.member("undetermined")
UNCLASSIFIED = pc.UNCLASSIFIED_ENTITY.declared
CHEMICAL = frozenset({SPECIES, SPECIES_FORM})
"""The classes whose entities go through the structural, registry and formula rules."""
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
    entity_class: str = SPECIES
    """The class the source entity's mapping declares."""
    mixture_definition: str | None = None
    mole_basis: bool | None = None
    components: tuple[tuple[EntityKey, str], ...] = ()
    """For a defined mixture: each component's source entity and its fraction as decimal text."""

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
class MixtureRec:
    canonical_key: str
    definition: str
    mole_basis: bool
    provisional: bool
    label: str
    origins: tuple[Origin, ...]
    components: tuple[tuple[str, str], ...]
    """(canonical key of the component's entity, fraction as decimal text), in key order; empty
    for a provisional mixture and for one no component is known of."""


@dataclass(frozen=True)
class MaterialRec:
    canonical_key: str
    registry_key: str | None
    provisional: bool
    label: str
    origins: tuple[Origin, ...]


@dataclass(frozen=True)
class PlainRec:
    """A provisional or curated entity with nothing but its key and label: a polymer type or an
    unclassified entity (`kind` names which)."""

    canonical_key: str
    kind: str
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
    entity_class: str = SPECIES
    """The class resolution applied: the source's, or a curated decision's for an undetermined
    entity."""
    target_kind: str = pc.SPECIES.declared
    """The kind of the entity the records attach to (`species`, `species_form`, `defined_mixture`,
    `material`, `polymer_type` or `unclassified_entity`)."""


@dataclass
class Resolution:
    species: dict[str, SpeciesRec] = field(default_factory=dict)
    forms: dict[str, FormRec] = field(default_factory=dict)
    mixtures: dict[str, MixtureRec] = field(default_factory=dict)
    materials: dict[str, MaterialRec] = field(default_factory=dict)
    others: dict[str, PlainRec] = field(default_factory=dict)
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


_DECISION_CLASSES: Mapping[str, frozenset[str]] = {
    SPECIES: frozenset({SPECIES}),
    SPECIES_FORM: frozenset({SPECIES}),
    DEFINED_MIXTURE: frozenset({DEFINED_MIXTURE}),
    MATERIAL: frozenset({MATERIAL}),
    POLYMER_TYPE: frozenset({POLYMER_TYPE}),
}
"""For a class the source states, the classes of canonical entity a curated decision may name."""


def _classes(ordered: Sequence[SourceEntityRec], decisions: Decisions) -> dict[EntityKey, str]:
    """The class resolution applies to each entity: the class its source states, or the class of
    the curated decision that settles an undetermined one. A decision whose class the source's
    class does not admit is an error."""
    classes: dict[EntityKey, str] = {}
    for entity in ordered:
        decision = decisions.identify.get(entity.key)
        if decision is None:
            classes[entity.key] = entity.entity_class
        elif entity.entity_class == UNDETERMINED:
            classes[entity.key] = decision.entity_class
        elif decision.entity_class in _DECISION_CLASSES.get(entity.entity_class, frozenset()):
            classes[entity.key] = entity.entity_class
        else:
            raise ResolveError(
                f"the decision that identifies {entity.key} names a `{decision.entity_class}`, "
                f"and its source states the class `{entity.entity_class}`"
            )
    return classes


def resolve(
    entities: Sequence[SourceEntityRec],
    decisions: Decisions,
    formula_scopes: FormulaScopes,
    structure: Structure,
    schemes: SchemeKinds,
) -> Resolution:
    """Resolve every source entity by the rules of its class."""
    result = Resolution()
    ordered = sorted(entities, key=lambda e: e.key)
    by_key = {e.key: e for e in ordered}
    result.unmatched_decisions = sorted(decisions.named() - set(by_key))
    classes = _classes(ordered, decisions)
    chemical = [e for e in ordered if classes[e.key] in CHEMICAL]
    _chemical(result, chemical, classes, decisions, formula_scopes, structure, schemes)
    _others(
        result,
        [e for e in ordered if classes[e.key] not in CHEMICAL],
        by_key,
        classes,
        decisions,
        schemes,
    )
    return result


def _chemical(
    result: Resolution,
    ordered: Sequence[SourceEntityRec],
    classes: Mapping[EntityKey, str],
    decisions: Decisions,
    formula_scopes: FormulaScopes,
    structure: Structure,
    schemes: SchemeKinds,
) -> None:
    """Species and species forms: rules 1 to 4 in order, else provisional (section 1)."""
    by_key = {e.key: e for e in ordered}

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
                key,
                UNIQUE,
                CURATED,
                decision.canonical_key,
                decision.canonical_key,
                (),
                None,
                classes[key],
            )
            continue
        if key in decisions.reject:
            results[key] = _provisional(
                entity, classes[key], REJECTED, CURATED, decisions.reject[key]
            )
            continue
        proposal = proposals.get(key)
        if proposal is None:
            results[key] = _provisional(entity, classes[key], UNRESOLVED, PROVISIONAL, None)
        elif len(proposal.keys) > 1 or key in merged_away:
            for candidate in proposal.keys:
                needed[candidate].append(entity)
            results[key] = _provisional(
                entity,
                classes[key],
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
                    classes[key],
                    AMBIGUOUS,
                    proposal.rule,
                    f"the stated charge {stated:+d} differs from the structure's {charges[target]:+d}",
                    (target,),
                )
                continue
            needed[target].append(entity)
            results[key] = EntityResult(
                key, UNIQUE, proposal.rule, target, target, (), None, classes[key]
            )

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
            outcome = replace(outcome, target_key=target, target_kind=pc.SPECIES_FORM.declared)
        result.entities[key] = outcome
    for form, held in form_origins.items():
        result.forms[form] = replace(result.forms[form], origins=_dedupe(held))


def mixture_key(mole_basis: bool, components: Sequence[tuple[str, str]]) -> str:
    """The canonical key of a defined mixture: `mixture:` and the canonical encoding of the basis
    (`mole` or `mass`) followed, for each component in key order, by the canonical key of its
    resolved entity and its fraction as decimal text."""
    return "mixture:" + identity.canonical_encoding(
        ["mole" if mole_basis else "mass", *(part for pair in sorted(components) for part in pair)]
    )


def material_key(scheme: str, value: str) -> str:
    """The canonical key of a material: `material:` and the canonical encoding of the registry
    scheme and the identifier."""
    return "material:" + identity.canonical_encoding([scheme, value])


def _registry_identifiers(entity: SourceEntityRec, schemes: SchemeKinds) -> list[tuple[str, str]]:
    return sorted(
        {
            (a.scheme, unicodedata.normalize("NFC", a.value))
            for a in entity.assertions
            if a.scheme in schemes.registry
        }
    )


def _components(
    entity: SourceEntityRec, resolved: Mapping[EntityKey, EntityResult]
) -> tuple[tuple[tuple[str, str], ...] | None, str | None]:
    """The components of a defined mixture as (canonical key, fraction text), or why it has no
    composition to build a key from: the source states none, or a component is not a unique
    species or species form of the corpus (or is listed twice)."""
    if not entity.components:
        return None, "the source states no composition"
    found: list[tuple[str, str]] = []
    for component, fraction in entity.components:
        outcome = resolved.get(component)
        if outcome is None or outcome.entity_class not in CHEMICAL:
            return None, f"the component {component} is not a species or species form"
        if outcome.status != UNIQUE:
            return None, f"the component {component} is not unique ({outcome.status})"
        found.append((outcome.target_key, fraction))
    if len({key for key, _ in found}) != len(found):
        return None, "a component is listed twice"
    return tuple(sorted(found)), None


def _others(
    result: Resolution,
    ordered: Sequence[SourceEntityRec],
    by_key: Mapping[EntityKey, SourceEntityRec],
    classes: Mapping[EntityKey, str],
    decisions: Decisions,
    schemes: SchemeKinds,
) -> None:
    """Defined mixtures, materials, polymer types and undetermined entities, by the rules of
    their class; they read the species and forms the chemical pass resolved."""
    members: dict[tuple[str, str], list[SourceEntityRec]] = defaultdict(list)
    """(kind, canonical key) to the entities resolved (or left candidates) to it."""
    chosen: dict[EntityKey, EntityResult] = {}
    parts: dict[str, tuple[tuple[str, str], ...]] = {}
    definitions: dict[str, set[str]] = defaultdict(set)
    for entity in ordered:
        key, klass = entity.key, classes[entity.key]
        kind = _TARGET_KINDS[klass]
        if key in decisions.reject:
            chosen[key] = _provisional(entity, klass, REJECTED, CURATED, decisions.reject[key])
            continue
        decided = decisions.identify.get(key)
        if klass == DEFINED_MIXTURE:
            if entity.mixture_definition is None or entity.mole_basis is None:
                raise ResolveError(
                    f"the defined mixture {key} states no definition and basis; its mapping "
                    "declares them for the scope"
                )
            found, why = _components(entity, result.entities)
            if decided is not None:
                target, status, rule = decided.canonical_key, UNIQUE, CURATED
            elif found is None:
                chosen[key] = _provisional(entity, klass, UNRESOLVED, PROVISIONAL, why)
                continue
            else:
                target, status, rule = mixture_key(entity.mole_basis, found), UNIQUE, COMPOSITION
            if found is not None:
                if parts.get(target, found) != found:
                    raise ResolveError(
                        f"the canonical key {target!r} is given different compositions by "
                        f"{key} and another entity"
                    )
                parts[target] = found
            definitions[target].add(entity.mixture_definition)
            members[(kind, target)].append(entity)
            chosen[key] = EntityResult(key, status, rule, target, target, (), None, klass, kind)
        elif klass == MATERIAL:
            registry = _registry_identifiers(entity, schemes)
            if decided is not None:
                target = decided.canonical_key
                members[(kind, target)].append(entity)
                chosen[key] = EntityResult(
                    key, UNIQUE, CURATED, target, target, (), None, klass, kind
                )
            elif len(registry) == 1:
                target = material_key(*registry[0])
                members[(kind, target)].append(entity)
                chosen[key] = EntityResult(
                    key, UNIQUE, REGISTRY, target, target, (), None, klass, kind
                )
            elif len(registry) > 1:
                candidates = tuple(sorted(material_key(*item) for item in registry))
                for candidate in candidates:
                    members[(kind, candidate)].append(entity)
                chosen[key] = _provisional(
                    entity,
                    klass,
                    AMBIGUOUS,
                    REGISTRY,
                    "several registry identifiers are asserted",
                    candidates,
                )
            else:
                chosen[key] = _provisional(entity, klass, UNRESOLVED, PROVISIONAL, None)
        elif decided is not None:  # a polymer type a curated decision identifies
            target = decided.canonical_key
            members[(kind, target)].append(entity)
            chosen[key] = EntityResult(key, UNIQUE, CURATED, target, target, (), None, klass, kind)
        else:  # a polymer type, or an entity whose class the source does not establish
            chosen[key] = _provisional(entity, klass, UNRESOLVED, PROVISIONAL, None)

    disagreeing = {target for target, found in definitions.items() if len(found) > 1}
    for key, outcome in list(chosen.items()):
        if outcome.target_key in disagreeing and outcome.status == UNIQUE:
            chosen[key] = _provisional(
                by_key[key],
                outcome.entity_class,
                AMBIGUOUS,
                outcome.rule,
                "the entities that give this composition differ on whether it is by definition "
                "or by measurement",
                (outcome.target_key,),
            )

    for (kind, canonical), entities in sorted(members.items()):
        origins = _dedupe([o for e in entities for o in e.origins])
        shown = label(entities)
        if kind == pc.DEFINED_MIXTURE.declared:
            first = entities[0]
            assert first.mole_basis is not None and first.mixture_definition is not None
            result.mixtures[canonical] = MixtureRec(
                canonical,
                sorted(definitions[canonical])[0],
                first.mole_basis,
                False,
                shown,
                origins,
                parts.get(canonical, ()),
            )
        elif kind == pc.MATERIAL.declared:
            registry = sorted(
                {item for e in entities for item in _registry_identifiers(e, schemes)}
            )
            own = next((item for item in registry if material_key(*item) == canonical), None)
            if own is None and len(registry) == 1:
                own = registry[0]
            result.materials[canonical] = MaterialRec(
                canonical, None if own is None else f"{own[0]}:{own[1]}", False, shown, origins
            )
        else:
            result.others[canonical] = PlainRec(canonical, kind, False, shown, origins)
    for key, outcome in sorted(chosen.items()):
        entity = by_key[key]
        if outcome.target_key.startswith(PROVISIONAL_PREFIX):
            kind = outcome.target_kind
            origins = _dedupe(entity.origins)
            if kind == pc.DEFINED_MIXTURE.declared:
                assert entity.mole_basis is not None and entity.mixture_definition is not None
                result.mixtures[outcome.target_key] = MixtureRec(
                    outcome.target_key,
                    entity.mixture_definition,
                    entity.mole_basis,
                    True,
                    entity.local_key,
                    origins,
                    (),
                )
            elif kind == pc.MATERIAL.declared:
                result.materials[outcome.target_key] = MaterialRec(
                    outcome.target_key, None, True, entity.local_key, origins
                )
            else:
                result.others[outcome.target_key] = PlainRec(
                    outcome.target_key, kind, True, entity.local_key, origins
                )
        result.entities[key] = outcome


def _provisional(
    entity: SourceEntityRec,
    entity_class: str,
    status: str,
    rule: str,
    reason: str | None,
    candidates: tuple[str, ...] = (),
) -> EntityResult:
    """The result of an entity that resolves to no canonical entity: its own provisional entity,
    of the kind its class says (an undetermined entity's is unclassified)."""
    key = provisional_key(entity.carrier_key, entity.scope, entity.local_key)
    return EntityResult(
        entity.key,
        status,
        rule,
        key,
        key,
        candidates,
        reason,
        entity_class,
        _TARGET_KINDS[entity_class],
    )


_TARGET_KINDS: Mapping[str, str] = {
    SPECIES: pc.SPECIES.declared,
    SPECIES_FORM: pc.SPECIES.declared,
    DEFINED_MIXTURE: pc.DEFINED_MIXTURE.declared,
    MATERIAL: pc.MATERIAL.declared,
    POLYMER_TYPE: pc.POLYMER_TYPE.declared,
    UNDETERMINED: UNCLASSIFIED,
}
"""The kind of the entity a class's provisional (or curated) target is."""


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
    "MaterialRec",
    "MixtureRec",
    "PlainRec",
    "Resolution",
    "ResolveError",
    "SchemeKinds",
    "SourceEntityRec",
    "SpeciesRec",
    "formula_key",
    "label",
    "material_key",
    "mixture_key",
    "provisional_key",
    "resolve",
    "scheme_kinds",
]
