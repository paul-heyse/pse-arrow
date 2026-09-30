# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Reading the phase-1 claims of every carrier, and writing a resolution as canonical rows and a
report."""

from __future__ import annotations

import uuid
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from thermo_knowledge import identity
from thermo_knowledge import pipeline_contract as pc
from thermo_knowledge.canonical.provenance import CarrierInfo, Carriers, Origin, SourceRef
from thermo_knowledge.canonical.store import CanonicalManifest
from thermo_knowledge.canonical.writer import CanonicalWriter
from thermo_knowledge.declaration import model as m
from thermo_knowledge.mapping import claims
from thermo_knowledge.resolve.engine import (
    AMBIGUOUS,
    Assertion,
    FormulaScopes,
    Resolution,
    ResolveError,
    SourceEntityRec,
)


@dataclass(frozen=True)
class Claims:
    """Everything phase 1 of every mapping claimed."""

    carriers: dict[str, CarrierInfo]
    entities: list[SourceEntityRec]
    formula_scopes: FormulaScopes


def read_claims(directories: dict[str, tuple[Path, CanonicalManifest]]) -> Claims:
    """The claims of each `_identity` directory (manifest id to directory and manifest),
    assembled into one `SourceEntityRec` per source entity."""
    carriers: dict[str, CarrierInfo] = {}
    scopes: dict[tuple[str, str], str | None] = {}
    entities: list[SourceEntityRec] = []
    for manifest_id, (directory, manifest) in sorted(directories.items()):
        carrier = manifest.carrier
        if carrier is None:
            raise ResolveError(f"{directory}: the phase-1 manifest records no carrier")
        carriers[manifest_id] = carrier
        for scope in manifest.formula_scopes:
            scopes[(manifest_id, scope.scope)] = scope.discriminator
        by_entity: dict[tuple[str, str, str], list[claims.EntityClaim]] = defaultdict(list)
        for claim in claims.read_entity_claims(directory):
            by_entity[claim.key].append(claim)
        found: dict[tuple[str, str, str], list[Assertion]] = defaultdict(list)
        seen: set[tuple[object, ...]] = set()
        for item in claims.read_assertion_claims(directory):
            marker = (item.entity, item.scheme, item.value)
            if marker not in seen:
                seen.add(marker)
                found[item.entity].append(Assertion(item.scheme, item.value, item.column))
        for key, rows in sorted(by_entity.items()):
            first = rows[0]
            for other in rows[1:]:
                for attribute in ("aggregation", "polymorph", "stated_charge"):
                    if getattr(first, attribute) != getattr(other, attribute):
                        raise ResolveError(
                            f"competing claims for the source entity {key}: {attribute} is "
                            f"{getattr(first, attribute)!r} at {first.locator} and "
                            f"{getattr(other, attribute)!r} at {other.locator}"
                        )
            entities.append(
                SourceEntityRec(
                    key=key,
                    carrier_key=carrier.key,
                    aggregation=first.aggregation,
                    polymorph=first.polymorph,
                    stated_charge=first.stated_charge,
                    origins=tuple(
                        sorted(
                            {
                                Origin(SourceRef(manifest_id, c.artifact, c.locator), c.origin_role)
                                for c in rows
                            },
                            key=lambda o: (o.ref.artifact, o.ref.locator, o.role),
                        )
                    ),
                    assertions=tuple(sorted(found[key], key=lambda a: (a.scheme, a.value))),
                )
            )
    return Claims(carriers, entities, FormulaScopes(scopes))


def _ids(decl: m.Declaration, kind: str) -> dict[str, uuid.UUID]:
    return {e.name: e.id for e in decl.entities if e.kind == kind}


def write_rows(
    decl: m.Declaration, read: Claims, resolution: Resolution, carriers: Carriers
) -> CanonicalWriter:
    """The canonical rows of a resolution: species, species forms, source entities, identity
    assertions and resolution candidates, with their records and origins."""
    writer = CanonicalWriter(decl, carriers)
    schemes = _ids(decl, pc.NAMING_SCHEME.declared)
    aggregations = _ids(decl, pc.AGGREGATION.declared)
    by_key = {e.key: e for e in read.entities}
    ids: dict[str, uuid.UUID] = {}
    for key, species in sorted(resolution.species.items()):
        ids[key] = writer.kind(
            pc.SPECIES.declared,
            {
                pc.SPECIES.canonical_key: species.canonical_key,
                pc.SPECIES.label: species.label,
                pc.SPECIES.provisional: species.provisional,
                pc.SPECIES.charge: species.charge,
                pc.SPECIES.inchikey: species.inchikey,
            },
            origins=species.origins,
            at=f"species {key}",
        )
    for key, form in sorted(resolution.forms.items()):
        aggregation = aggregations.get(form.aggregation)
        if aggregation is None:
            raise ResolveError(f"`{form.aggregation}` is not a declared aggregation")
        ids[key] = writer.kind(
            pc.SPECIES_FORM.declared,
            {
                pc.SPECIES_FORM.canonical_key: form.canonical_key,
                pc.SPECIES_FORM.label: form.label,
                pc.SPECIES_FORM.provisional: form.provisional,
                pc.SPECIES_FORM.species: ids[form.species_key],
                pc.SPECIES_FORM.aggregation: aggregation,
                pc.SPECIES_FORM.polymorph: form.polymorph,
            },
            origins=form.origins,
            at=f"species_form {key}",
        )
    for key, outcome in sorted(resolution.entities.items()):
        entity = by_key[key]
        carrier = read.carriers[entity.manifest_id]
        aggregation_id = None
        if entity.aggregation is not None:
            aggregation_id = aggregations.get(entity.aggregation)
            if aggregation_id is None:
                raise ResolveError(f"`{entity.aggregation}` is not a declared aggregation")
        source_entity = pc.SOURCE_ENTITY
        entity_id = writer.kind(
            source_entity.declared,
            {
                source_entity.carrier: uuid_of_carrier(carrier),
                source_entity.scope: entity.scope,
                source_entity.local_key: entity.local_key,
                source_entity.aggregation: aggregation_id,
                source_entity.polymorph: entity.polymorph,
                source_entity.stated_charge: entity.stated_charge,
                source_entity.status: outcome.status,
                source_entity.rule: outcome.rule,
                source_entity.target: ids[outcome.target_key],
            },
            origins=entity.origins,
            at=entity.origins[0].ref.locator,
        )
        for item in entity.assertions:
            scheme = schemes.get(item.scheme)
            if scheme is None:
                raise ResolveError(f"`{item.scheme}` is not a declared naming scheme")
            assertion = pc.IDENTITY_ASSERTION
            writer.kind(
                assertion.declared,
                {
                    assertion.source_entity: entity_id,
                    assertion.scheme: scheme,
                    assertion.value: item.value,
                },
                at=entity.origins[0].ref.locator,
            )
        for candidate in outcome.candidates:
            link = pc.RESOLUTION_CANDIDATE
            writer.relation(
                link.declared,
                {link.source_entity: entity_id, link.candidate: ids[candidate]},
                at=entity.origins[0].ref.locator,
            )
    return writer


def uuid_of_carrier(carrier: CarrierInfo) -> uuid.UUID:
    """The identifier of the `carrier` instance (a refinement of `source`, keyed by its key)."""
    return identity.identifier(pc.SOURCE.declared, [carrier.key])


def report(read: Claims, resolution: Resolution) -> dict[str, object]:
    """The resolution report: counts by carrier, status and rule, and every ambiguous entity
    with its candidates."""
    by_carrier: dict[str, dict[str, Counter[str]]] = {
        manifest_id: {"status": Counter(), "rule": Counter(), "status_rule": Counter()}
        for manifest_id in read.carriers
    }
    for key, outcome in resolution.entities.items():
        bucket = by_carrier[key[0]]
        bucket["status"][outcome.status] += 1
        bucket["rule"][outcome.rule] += 1
        bucket["status_rule"][f"{outcome.status}/{outcome.rule}"] += 1
    totals = {
        "status": Counter(o.status for o in resolution.entities.values()),
        "rule": Counter(o.rule for o in resolution.entities.values()),
    }
    return {
        "carriers": {
            manifest_id: {
                "entities": sum(buckets["status"].values()),
                "status": dict(sorted(buckets["status"].items())),
                "rule": dict(sorted(buckets["rule"].items())),
                "status_rule": dict(sorted(buckets["status_rule"].items())),
            }
            for manifest_id, buckets in sorted(by_carrier.items())
        },
        "totals": {
            "source_entities": len(resolution.entities),
            "species": sum(1 for s in resolution.species.values() if not s.provisional),
            "provisional_species": sum(1 for s in resolution.species.values() if s.provisional),
            "species_forms": len(resolution.forms),
            "status": dict(sorted(totals["status"].items())),
            "rule": dict(sorted(totals["rule"].items())),
        },
        "ambiguous": [
            {
                "entity": {"carrier": key[0], "scope": key[1], "key": key[2]},
                "rule": outcome.rule,
                "reason": outcome.reason,
                "candidates": list(outcome.candidates),
            }
            for key, outcome in sorted(resolution.entities.items())
            if outcome.status == AMBIGUOUS
        ],
        "registry_conflicts": [
            {"scheme": scheme, "value": value, "structures": list(structures)}
            for (scheme, value), structures in resolution.registry_conflicts.items()
        ],
        "unusable_structural_identifiers": [
            {
                "entity": {"carrier": key[0], "scope": key[1], "key": key[2]},
                "scheme": scheme,
                "value": value,
                "problem": problem,
            }
            for key, scheme, value, problem in sorted(resolution.unusable)
        ],
        "unmatched_decisions": [
            {"carrier": key[0], "scope": key[1], "key": key[2]}
            for key in resolution.unmatched_decisions
        ],
    }


def report_table(data: dict[str, object]) -> list[str]:
    """The report's counts as printable lines."""
    lines = ["carrier                    entities  status/rule"]
    carriers = data["carriers"]
    assert isinstance(carriers, dict)
    for manifest_id, info in carriers.items():
        detail = ", ".join(f"{k}: {v}" for k, v in info["status_rule"].items())
        lines.append(f"{manifest_id:<26} {info['entities']:>8}  {detail}")
    totals = data["totals"]
    assert isinstance(totals, dict)
    lines.append(
        f"{'all':<26} {totals['source_entities']:>8}  species {totals['species']}, "
        f"provisional {totals['provisional_species']}, forms {totals['species_forms']}"
    )
    ambiguous = data["ambiguous"]
    assert isinstance(ambiguous, list)
    for item in ambiguous:
        entity = item["entity"]
        lines.append(
            f"ambiguous  {entity['carrier']}/{entity['scope']}/{entity['key']}  "
            f"[{item['rule']}] candidates: {', '.join(item['candidates'])}"
            + (f"  ({item['reason']})" if item["reason"] else "")
        )
    return lines
