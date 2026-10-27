# msgspec tagged-Struct decoding probe

Evidence for the library-leverage applicability assessment of
[CBO-041](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-041)
(and the row half of [CBO-043](../../../library_leverage/codebase/registry-generation-and-python-boundary.md#cbo-043)),
2026-10-27, source baseline `dbc4c9436` (inspected at `c2f8b485a`). It characterizes msgspec in
isolation with synthetic types; it does not exercise any pse-arrow document type, codec function or
the Rust header check.

## Question

Can a msgspec tagged Struct (`tag_field="document_version"`, integer `tag`) carry the guarantees the
Python codec's `_check_version_headers` currently adds for versioned boundary documents — a
**required** header, checked **before** the body, **duplicate-refusing**, with unknown-field and
non-finite refusal — and how does `msgspec.convert(from_attributes=True)` behave from an attrs row?
Static reading of msgspec 0.21.1 `_core.c` (`json_ensure_tag_matches`, `json_decode_struct_union`)
left open the exact behavior of direct (non-union) decode, error ordering and duplicate keys.

## Environment

- msgspec 0.21.1, attrs 26.1.0 (the repository `.venv`, from `uv.lock`), CPython 3.14.8, x86_64 Linux.
- Script kept outside the repository (agent scratch); full text below.

## Command

```bash
scripts/pse-env -- .venv/bin/python -I <scratch>/probe/msgspec_tagged_probe.py
```

Run once from the repository root on 2026-10-27; exit status 0.

## Outcome

**Tested (msgspec 0.21.1, CPython 3.14.8, synthetic structs; one run):**

| Case | Behavior |
|---|---|
| Direct decode of a tagged Struct, tag absent (A2, S3) | **Accepted** — the tag is not required. |
| Direct decode, wrong int tag (A3), negative or > int64 (A15, A16) | ValidationError: Invalid value 2 - at `$.document_version` |
| Direct decode, `"1"`/`true`/`1.0` for int tag (A4–A6) | ValidationError: Expected `int` - at `$.document_version` (bool and float refused) |
| Direct decode, body error **before** wrong tag (A7) | Body error wins: Expected `int`, got `str` - at `$.count` (document order). |
| Direct decode, unknown field before wrong tag (A9) | Object contains unknown field `extra` wins. |
| Union `DocV1 \| DocV2`, tag absent (U2) | ValidationError: Object missing required field `document_version` |
| Union, tag last (U1) | Accepted — tag found by lookahead. |
| Union, body error before **unknown** tag (U5) | Tag error wins: Invalid value 3 - at `$.document_version` (header-first). |
| Union, body error before **valid** tag (U4) | Body error reported (tag dispatched first, then body decoded). |
| `DocV1 \| None` (a union with a single struct), tag absent (O1) | **Accepted** — behaves like direct decode. |
| Duplicate tag with differing values, either order (A10, A11, U6) | Refused via Invalid value 2 - at `$.document_version`. |
| Duplicate tag with equal values (A12) | Accepted. |
| Duplicate ordinary field (A13), duplicate untagged int header (H1) | Accepted, last value wins. |
| `forbid_unknown_fields=True` (A14, U8, C7) | Object contains unknown field `extra` (decode and convert). |
| Constructing with the tag as keyword (E2) | `TypeError: Unexpected keyword argument 'document_version'`; encode always writes the tag first (E1). |
| Constructor type checking (E3) | None — `DocV1(name=5, count='x')` succeeds. |
| JSON `NaN` / `Infinity` literals (N1, N2) | `DecodeError: JSON is malformed: invalid character (byte 5)` |
| JSON `null` into `float` (N3) / `float \| None` (N4) | Refused Expected `float`, got `null` / accepted as `None`. |
| JSON `1e999` (N5) | ValidationError: Number out of range - at `$.x` |
| JSON `"nan"` strict (N6) / lax (N7) | Refused / accepted as `nan`. |
| Encode `nan`/`inf` (N8, N9) | Written as `null`. |
| `convert` of `nan`/`inf` from dict or attrs (N10–N13) | **Accepted** into `float` and `float \| None`. |
| `convert` with `Meta(ge=0)` (N14, N15) | `nan` refused (Expected `float` >= 0.0); `inf` accepted. |
| `convert` `True` into `float`/`int` (N16, C8, C9) | Refused (Expected `float`, got `bool`; Expected `int`, got `bool`). |
| attrs → tagged Struct, `from_attributes=True` (C1) | Accepted (tag absent on the attrs row is not required). |
| attrs → tagged Struct, no `from_attributes` (C2) | ValidationError: Expected `object`, got `Row` |
| attrs → tagged **union**, `from_attributes=True` (C3); dict without tag → union (C6) | Object missing required field `document_version` |
| dict with wrong tag → Struct (C5) | Invalid value 2 - at `$.document_version` |
| `msgspec.json.encode` of a frozen attrs instance (C10) | `{"count":1,"name":"a"}` — keys not in attrs declaration order (`name`, `count`). |

Interpretation (Proposed): a tagged Struct moves the *value* check (including bool/float refusal and
differing duplicates) into msgspec, but on direct decode it does **not** make the header required or
header-first; only a union of at least two tagged structs gives both. Equal duplicates and duplicate
non-tag keys are not refused. Non-finite refusal holds on JSON decode (no literals; `null` refused for
non-optional floats) but not on `convert` from Python objects, where an explicit check remains.

## Limits

- Synthetic types only; no generated pse-arrow document, `dec_hook`, `Meta` constraint set, nested
  versioned document or TOML path was exercised.
- One msgspec version (0.21.1); 0.22.0 release notes list no change to these paths (not run).
- Error *types and messages* are as printed by this run; msgspec does not document them as stable.
- The attrs encode key order (C10) was observed, not explained.

<details><summary>Probe script</summary>

```python
"""P2 probe: msgspec tagged-Struct decoding for a version-header-like tag."""
import math
import sys

import attrs
import msgspec


class DocV1(msgspec.Struct, tag_field="document_version", tag=1, forbid_unknown_fields=True):
    name: str
    count: int


class DocV2(msgspec.Struct, tag_field="document_version", tag=2, forbid_unknown_fields=True):
    name: str
    count: int


class StrDoc(msgspec.Struct, tag_field="document_version", tag="1", forbid_unknown_fields=True):
    name: str
    count: int


class Untagged(msgspec.Struct, forbid_unknown_fields=True):
    document_version: int
    name: str


class Num(msgspec.Struct, forbid_unknown_fields=True):
    x: float


class NumOpt(msgspec.Struct, forbid_unknown_fields=True):
    x: float | None = None


@attrs.frozen(kw_only=True)
class Row:
    name: str
    count: int


@attrs.frozen(kw_only=True)
class FRow:
    x: float


def run(label, fn):
    try:
        out = fn()
        print(f"{label}: OK -> {out!r}")
    except Exception as e:  # noqa: BLE001
        print(f"{label}: {type(e).__module__}.{type(e).__qualname__}: {e}")


dec1 = msgspec.json.Decoder(DocV1)
dec_u = msgspec.json.Decoder(DocV1 | DocV2)
dec_s = msgspec.json.Decoder(StrDoc)

print("python", sys.version.split()[0], "msgspec", msgspec.__version__, "attrs", attrs.__version__)
print("== direct (non-union) decode, int tag 1 ==")
run("A1 valid tag first", lambda: dec1.decode(b'{"document_version":1,"name":"a","count":1}'))
run("A2 missing tag", lambda: dec1.decode(b'{"name":"a","count":1}'))
run("A3 wrong int tag 2", lambda: dec1.decode(b'{"document_version":2,"name":"a","count":1}'))
run("A4 str tag '1' for int tag", lambda: dec1.decode(b'{"document_version":"1","name":"a","count":1}'))
run("A5 bool tag true", lambda: dec1.decode(b'{"document_version":true,"name":"a","count":1}'))
run("A6 float tag 1.0", lambda: dec1.decode(b'{"document_version":1.0,"name":"a","count":1}'))
run("A7 body error before wrong tag", lambda: dec1.decode(b'{"count":"x","document_version":2,"name":"a"}'))
run("A8 wrong tag before body error", lambda: dec1.decode(b'{"document_version":2,"count":"x","name":"a"}'))
run("A9 unknown field before wrong tag", lambda: dec1.decode(b'{"extra":1,"document_version":2,"name":"a","count":1}'))
run("A10 duplicate tag 1 then 2", lambda: dec1.decode(b'{"document_version":1,"document_version":2,"name":"a","count":1}'))
run("A11 duplicate tag 2 then 1", lambda: dec1.decode(b'{"document_version":2,"document_version":1,"name":"a","count":1}'))
run("A12 duplicate tag 1 then 1", lambda: dec1.decode(b'{"document_version":1,"document_version":1,"name":"a","count":1}'))
run("A13 duplicate body field", lambda: dec1.decode(b'{"document_version":1,"name":"a","name":"b","count":1}'))
run("A14 unknown field (forbid)", lambda: dec1.decode(b'{"document_version":1,"name":"a","count":1,"extra":0}'))
run("A15 tag is -1 (int64 path)", lambda: dec1.decode(b'{"document_version":-1,"name":"a","count":1}'))
run("A16 tag huge uint", lambda: dec1.decode(b'{"document_version":18446744073709551615,"name":"a","count":1}'))
print("== direct decode, str tag '1' ==")
run("S1 valid str tag", lambda: dec_s.decode(b'{"document_version":"1","name":"a","count":1}'))
run("S2 int 1 for str tag", lambda: dec_s.decode(b'{"document_version":1,"name":"a","count":1}'))
run("S3 missing str tag", lambda: dec_s.decode(b'{"name":"a","count":1}'))
print("== union decode DocV1 | DocV2 ==")
run("U1 tag last", lambda: dec_u.decode(b'{"name":"a","count":1,"document_version":2}'))
run("U2 missing tag", lambda: dec_u.decode(b'{"name":"a","count":1}'))
run("U3 unknown tag 3", lambda: dec_u.decode(b'{"document_version":3,"name":"a","count":1}'))
run("U4 body error before tag (tag valid)", lambda: dec_u.decode(b'{"count":"x","document_version":1,"name":"a"}'))
run("U5 body error before unknown tag", lambda: dec_u.decode(b'{"count":"x","document_version":3,"name":"a"}'))
run("U6 duplicate tag 1 then 2", lambda: dec_u.decode(b'{"document_version":1,"document_version":2,"name":"a","count":1}'))
run("U7 str tag '1' in int union", lambda: dec_u.decode(b'{"document_version":"1","name":"a","count":1}'))
run("U8 unknown field", lambda: dec_u.decode(b'{"document_version":1,"name":"a","count":1,"extra":0}'))
print("== union containing a single tagged struct (Union[DocV1, None]) ==")
dec_opt = msgspec.json.Decoder(DocV1 | None)
run("O1 missing tag", lambda: dec_opt.decode(b'{"name":"a","count":1}'))
print("== untagged Literal-free int header, duplicate key ==")
run("H1 duplicate int field", lambda: msgspec.json.decode(b'{"document_version":1,"document_version":2,"name":"a"}', type=Untagged))
print("== encode ==")
run("E1 encode DocV1", lambda: msgspec.json.encode(DocV1(name="a", count=1)))
run("E2 construct with tag kwarg", lambda: DocV1(document_version=1, name="a", count=1))
run("E3 construct wrong types (no validation)", lambda: DocV1(name=5, count="x"))
print("== non-finite floats ==")
run("N1 json decode NaN literal", lambda: msgspec.json.decode(b'{"x":NaN}', type=Num))
run("N2 json decode Infinity literal", lambda: msgspec.json.decode(b'{"x":Infinity}', type=Num))
run("N3 json decode null into float", lambda: msgspec.json.decode(b'{"x":null}', type=Num))
run("N4 json decode null into float|None", lambda: msgspec.json.decode(b'{"x":null}', type=NumOpt))
run("N5 json decode 1e999 (overflow)", lambda: msgspec.json.decode(b'{"x":1e999}', type=Num))
run("N6 json decode '\"nan\"' strict", lambda: msgspec.json.decode(b'{"x":"nan"}', type=Num))
run("N7 json decode '\"nan\"' lax", lambda: msgspec.json.decode(b'{"x":"nan"}', type=Num, strict=False))
run("N8 json encode nan", lambda: msgspec.json.encode(Num(x=math.nan)))
run("N9 json encode inf", lambda: msgspec.json.encode(Num(x=math.inf)))
run("N10 convert dict nan -> float", lambda: msgspec.convert({"x": math.nan}, Num))
run("N11 convert dict inf -> float", lambda: msgspec.convert({"x": math.inf}, Num))
run("N12 convert dict nan -> float|None", lambda: msgspec.convert({"x": math.nan}, NumOpt))
run("N13 convert attrs nan from_attributes", lambda: msgspec.convert(FRow(x=math.nan), Num, from_attributes=True))
run("N14 convert nan with Meta(ge=0)", lambda: msgspec.convert(math.nan, msgspec.Meta(ge=0) and __import__("typing").Annotated[float, msgspec.Meta(ge=0)]))
run("N15 convert inf with Meta(ge=0)", lambda: msgspec.convert(math.inf, __import__("typing").Annotated[float, msgspec.Meta(ge=0)]))
run("N16 convert True -> float", lambda: msgspec.convert({"x": True}, Num))
print("== convert from attrs ==")
run("C1 attrs -> tagged Struct from_attributes", lambda: msgspec.convert(Row(name="a", count=1), DocV1, from_attributes=True))
run("C2 attrs -> tagged Struct no from_attributes", lambda: msgspec.convert(Row(name="a", count=1), DocV1))
run("C3 attrs -> union from_attributes", lambda: msgspec.convert(Row(name="a", count=1), DocV1 | DocV2, from_attributes=True))
run("C4 dict without tag -> DocV1", lambda: msgspec.convert({"name": "a", "count": 1}, DocV1))
run("C5 dict wrong tag -> DocV1", lambda: msgspec.convert({"document_version": 2, "name": "a", "count": 1}, DocV1))
run("C6 dict without tag -> union", lambda: msgspec.convert({"name": "a", "count": 1}, DocV1 | DocV2))
run("C7 dict extra field -> DocV1", lambda: msgspec.convert({"document_version": 1, "name": "a", "count": 1, "extra": 0}, DocV1))
run("C8 dict bool for int", lambda: msgspec.convert({"document_version": 1, "name": "a", "count": True}, DocV1))
run("C9 attrs bool for int from_attributes", lambda: msgspec.convert(Row(name="a", count=True), DocV1, from_attributes=True))
run("C10 json encode attrs instance", lambda: msgspec.json.encode(Row(name="a", count=1)))
```

</details>

<details><summary>Raw output</summary>

```text
python 3.14.8 msgspec 0.21.1 attrs 26.1.0
== direct (non-union) decode, int tag 1 ==
A1 valid tag first: OK -> DocV1(name='a', count=1)
A2 missing tag: OK -> DocV1(name='a', count=1)
A3 wrong int tag 2: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
A4 str tag '1' for int tag: msgspec.ValidationError: Expected `int` - at `$.document_version`
A5 bool tag true: msgspec.ValidationError: Expected `int` - at `$.document_version`
A6 float tag 1.0: msgspec.ValidationError: Expected `int` - at `$.document_version`
A7 body error before wrong tag: msgspec.ValidationError: Expected `int`, got `str` - at `$.count`
A8 wrong tag before body error: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
A9 unknown field before wrong tag: msgspec.ValidationError: Object contains unknown field `extra`
A10 duplicate tag 1 then 2: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
A11 duplicate tag 2 then 1: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
A12 duplicate tag 1 then 1: OK -> DocV1(name='a', count=1)
A13 duplicate body field: OK -> DocV1(name='b', count=1)
A14 unknown field (forbid): msgspec.ValidationError: Object contains unknown field `extra`
A15 tag is -1 (int64 path): msgspec.ValidationError: Invalid value -1 - at `$.document_version`
A16 tag huge uint: msgspec.ValidationError: Invalid value 18446744073709551615 - at `$.document_version`
== direct decode, str tag '1' ==
S1 valid str tag: OK -> StrDoc(name='a', count=1)
S2 int 1 for str tag: msgspec.ValidationError: Expected `str` - at `$.document_version`
S3 missing str tag: OK -> StrDoc(name='a', count=1)
== union decode DocV1 | DocV2 ==
U1 tag last: OK -> DocV2(name='a', count=1)
U2 missing tag: msgspec.ValidationError: Object missing required field `document_version`
U3 unknown tag 3: msgspec.ValidationError: Invalid value 3 - at `$.document_version`
U4 body error before tag (tag valid): msgspec.ValidationError: Expected `int`, got `str` - at `$.count`
U5 body error before unknown tag: msgspec.ValidationError: Invalid value 3 - at `$.document_version`
U6 duplicate tag 1 then 2: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
U7 str tag '1' in int union: msgspec.ValidationError: Expected `int` - at `$.document_version`
U8 unknown field: msgspec.ValidationError: Object contains unknown field `extra`
== union containing a single tagged struct (Union[DocV1, None]) ==
O1 missing tag: OK -> DocV1(name='a', count=1)
== untagged Literal-free int header, duplicate key ==
H1 duplicate int field: OK -> Untagged(document_version=2, name='a')
== encode ==
E1 encode DocV1: OK -> b'{"document_version":1,"name":"a","count":1}'
E2 construct with tag kwarg: builtins.TypeError: Unexpected keyword argument 'document_version'
E3 construct wrong types (no validation): OK -> DocV1(name=5, count='x')
== non-finite floats ==
N1 json decode NaN literal: msgspec.DecodeError: JSON is malformed: invalid character (byte 5)
N2 json decode Infinity literal: msgspec.DecodeError: JSON is malformed: invalid character (byte 5)
N3 json decode null into float: msgspec.ValidationError: Expected `float`, got `null` - at `$.x`
N4 json decode null into float|None: OK -> NumOpt(x=None)
N5 json decode 1e999 (overflow): msgspec.ValidationError: Number out of range - at `$.x`
N6 json decode '"nan"' strict: msgspec.ValidationError: Expected `float`, got `str` - at `$.x`
N7 json decode '"nan"' lax: OK -> Num(x=nan)
N8 json encode nan: OK -> b'{"x":null}'
N9 json encode inf: OK -> b'{"x":null}'
N10 convert dict nan -> float: OK -> Num(x=nan)
N11 convert dict inf -> float: OK -> Num(x=inf)
N12 convert dict nan -> float|None: OK -> NumOpt(x=nan)
N13 convert attrs nan from_attributes: OK -> Num(x=nan)
N14 convert nan with Meta(ge=0): msgspec.ValidationError: Expected `float` >= 0.0
N15 convert inf with Meta(ge=0): OK -> inf
N16 convert True -> float: msgspec.ValidationError: Expected `float`, got `bool` - at `$.x`
== convert from attrs ==
C1 attrs -> tagged Struct from_attributes: OK -> DocV1(name='a', count=1)
C2 attrs -> tagged Struct no from_attributes: msgspec.ValidationError: Expected `object`, got `Row`
C3 attrs -> union from_attributes: msgspec.ValidationError: Object missing required field `document_version`
C4 dict without tag -> DocV1: OK -> DocV1(name='a', count=1)
C5 dict wrong tag -> DocV1: msgspec.ValidationError: Invalid value 2 - at `$.document_version`
C6 dict without tag -> union: msgspec.ValidationError: Object missing required field `document_version`
C7 dict extra field -> DocV1: msgspec.ValidationError: Object contains unknown field `extra`
C8 dict bool for int: msgspec.ValidationError: Expected `int`, got `bool` - at `$.count`
C9 attrs bool for int from_attributes: msgspec.ValidationError: Expected `int`, got `bool` - at `$.count`
C10 json encode attrs instance: OK -> b'{"count":1,"name":"a"}'
```

</details>
