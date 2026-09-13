"""Probes 6, 7 — the typed contract layer asserted by §21.5."""
from typing import Any
import attrs, cattrs, msgspec

print("== PROBE 6: attrs + cattrs strictness (§21.5)")
@attrs.define(frozen=True)
class VarRow:
    ordinal: int
    semantic_id: bytes
    lower: float | None = None

c = cattrs.Converter()
ok = c.structure({"ordinal": 1, "semantic_id": b"\x00"*16, "lower": 0.0}, VarRow)
print("   structure happy path  :", ok)
# unknown key
try:
    r = c.structure({"ordinal": 1, "semantic_id": b"x", "lower": 0.0, "BOGUS": 9}, VarRow)
    print("   unknown key           : ACCEPTED (silently dropped) ->", r)
except Exception as e:
    print("   unknown key           : rejected", type(e).__name__)
# forbid_extra_keys
c2 = cattrs.Converter(forbid_extra_keys=True)
try:
    c2.structure({"ordinal": 1, "semantic_id": b"x", "lower": 0.0, "BOGUS": 9}, VarRow)
    print("   forbid_extra_keys     : ACCEPTED (unexpected)")
except Exception as e:
    print("   forbid_extra_keys     : rejected ->", type(e).__name__)
# wrong type
try:
    c.structure({"ordinal": "not-an-int", "semantic_id": b"x"}, VarRow)
    print("   wrong type            : ACCEPTED (unexpected)")
except Exception as e:
    print("   wrong type            : rejected ->", type(e).__name__)
# detailed validation
c3 = cattrs.Converter(detailed_validation=True)
try:
    c3.structure({"ordinal": "bad", "semantic_id": 5}, VarRow)
except Exception as e:
    print("   detailed_validation   :", type(e).__name__, "| sub-errors:",
          len(getattr(e, "exceptions", []) or []))
# Any field
@attrs.define
class Loose:
    payload: Any
try:
    r = c.structure({"payload": {"anything": 1}}, Loose)
    print("   Any field             : ACCEPTED ->", r, " (nothing in cattrs forbids it)")
except Exception as e:
    print("   Any field             : rejected", type(e).__name__)
print("   attrs introspection   : fields() ->", [f.name for f in attrs.fields(VarRow)])

print("== PROBE 7: msgspec constraints and schema (§20.2, §21.5)")
class Manifest(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    snapshot_id: str
    rows: msgspec.Meta | int = 0
class Rel(msgspec.Struct, forbid_unknown_fields=True):
    name: str
    rows: int
    ratio: float = 1.0
enc = msgspec.json.encode(Rel(name="stoichiometry", rows=12))
print("   encode                :", enc)
print("   decode round-trip     :", msgspec.json.decode(enc, type=Rel))
try:
    msgspec.json.decode(b'{"name":"x","rows":1,"BOGUS":2}', type=Rel)
    print("   unknown field         : ACCEPTED (unexpected)")
except Exception as e:
    print("   unknown field         : rejected ->", type(e).__name__, "|", str(e)[:70])
try:
    msgspec.json.decode(b'{"name":"x","rows":"twelve"}', type=Rel)
except Exception as e:
    print("   wrong type            : rejected ->", type(e).__name__, "|", str(e)[:70])
# Meta constraints
from typing import Annotated
class Scaled(msgspec.Struct):
    factor: Annotated[float, msgspec.Meta(gt=0.0)]
try:
    msgspec.json.decode(b'{"factor":-1.0}', type=Scaled)
except Exception as e:
    print("   Meta(gt=0) violation  : rejected ->", type(e).__name__, "|", str(e)[:70])
print("   json.schema(Rel)      :", msgspec.json.schema(Rel))
