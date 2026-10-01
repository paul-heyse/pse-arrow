# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Standard InChIKeys from structure identifiers, with RDKit (resolution rule 2).

A standard InChIKey is taken as given when it has the shape of one (14 letters, hyphen, 8
letters, the flag `S`, the version `A`, hyphen, one letter); from a standard InChI it is
computed with `rdkit.Chem.inchi.InchiToInchiKey`, from a SMILES by parsing the molecule and
computing its InChIKey. The charge is the molecule's formal charge where RDKit can build the
molecule. Nothing here guesses: a value RDKit cannot use yields a `problem`, not a key.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from functools import cache

from rdkit import Chem, rdBase
from rdkit.Chem import inchi as rd_inchi

STANDARD_INCHIKEY = re.compile(r"^[A-Z]{14}-[A-Z]{8}SA-[A-Z]$")
STANDARD_INCHI_PREFIX = "InChI=1S/"
STRUCTURAL_SCHEMES = ("inchikey", "inchi", "smiles")
"""The naming schemes this module can compute a key from."""

rdBase.DisableLog("rdApp.*")


@dataclass(frozen=True)
class StructureKey:
    """The InChIKey a structural identifier stands for, and its charge where known."""

    inchikey: str | None
    charge: int | None
    problem: str | None = None


@cache
def structure_key(scheme: str, value: str) -> StructureKey:
    """The standard InChIKey of a structural identifier of `scheme` (`inchikey`, `inchi` or
    `smiles`)."""
    if scheme == "inchikey":
        if STANDARD_INCHIKEY.match(value):
            return StructureKey(value, None)
        return StructureKey(None, None, "not a standard InChIKey")
    if scheme == "inchi":
        if not value.startswith(STANDARD_INCHI_PREFIX):
            return StructureKey(None, None, "not a standard InChI")
        key = rd_inchi.InchiToInchiKey(value)
        if not key:
            return StructureKey(None, None, "RDKit cannot compute an InChIKey from it")
        molecule = Chem.MolFromInchi(value)
        return StructureKey(key, None if molecule is None else Chem.GetFormalCharge(molecule))
    if scheme == "smiles":
        molecule = Chem.MolFromSmiles(value)
        if molecule is None:
            return StructureKey(None, None, "RDKit cannot parse the SMILES")
        key = Chem.MolToInchiKey(molecule)
        if not key:
            return StructureKey(None, None, "RDKit cannot compute an InChIKey from the molecule")
        return StructureKey(key, Chem.GetFormalCharge(molecule))
    raise ValueError(f"`{scheme}` is not a structural scheme this module handles")
