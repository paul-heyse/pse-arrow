"""Probe msgspec identity hooks without importing the product extension."""

import sys
from typing import NewType

import msgspec


class BytesIdentity(bytes):
    """Model the existing identity's bytes-subclass representation."""

    def __new__(cls, raw: bytes):
        if len(raw) != 16:
            raise ValueError("identity width")
        return super().__new__(cls, raw)


class ValueIdentity:
    """Model a value representation whose wire codec is explicit."""

    def __init__(self, raw: bytes) -> None:
        if len(raw) != 16:
            raise ValueError("identity width")
        self.raw = raw


encoded: list[str] = []
decoded: list[str] = []


def encode_hook(value: object) -> str:
    encoded.append(type(value).__name__)
    if isinstance(value, BytesIdentity):
        return value.hex()
    if isinstance(value, ValueIdentity):
        return value.raw.hex()
    raise TypeError(type(value).__name__)


def decode_hook(target: type, value: object) -> object:
    decoded.append(target.__name__)
    if target not in (BytesIdentity, ValueIdentity) or not isinstance(value, str):
        raise TypeError("identity text expected")
    return target(bytes.fromhex(value))


print(f"Python {sys.version.split()[0]}; msgspec {msgspec.__version__}")
assert msgspec.__version__ == "0.21.1", "probe must use the repository pin"
raw = bytes(range(16))
for identity_type in (BytesIdentity, ValueIdentity):
    encoded.clear()
    decoded.clear()
    wire = msgspec.json.encode(identity_type(raw), enc_hook=encode_hook)
    recovered = msgspec.json.decode(wire, type=identity_type, dec_hook=decode_hook)
    recovered_raw = bytes(recovered) if isinstance(recovered, bytes) else recovered.raw
    print(f"{identity_type.__name__}: wire={wire!r}; enc_hook={encoded}; dec_hook={decoded}")
    assert recovered_raw == raw
    assert encoded == [identity_type.__name__]
    assert decoded == [identity_type.__name__]

RunIdentity = NewType("RunIdentity", ValueIdentity)
FitIdentity = NewType("FitIdentity", ValueIdentity)
identity = ValueIdentity(raw)
assert RunIdentity(identity) is identity and FitIdentity(identity) is identity
print("NewType role aliases preserve the same runtime object; runtime role checking is not supplied")


class IdentityDocument(msgspec.Struct, forbid_unknown_fields=True):
    existing: BytesIdentity
    role: RunIdentity


encoded.clear()
decoded.clear()
wire = msgspec.json.encode(IdentityDocument(BytesIdentity(raw), identity), enc_hook=encode_hook)
recovered_document = msgspec.json.decode(wire, type=IdentityDocument, dec_hook=decode_hook)
assert recovered_document.existing == raw and recovered_document.role.raw == raw
assert encoded == ["BytesIdentity", "ValueIdentity"]
assert decoded == ["BytesIdentity", "ValueIdentity"]
print("Nested Struct and NewType: both hexadecimal hooks round-trip")
for identity_type in (BytesIdentity, ValueIdentity):
    try:
        msgspec.json.decode(b'"00"', type=identity_type, dec_hook=decode_hook)
    except msgspec.ValidationError:
        print(f"{identity_type.__name__}: malformed width refused")
    else:
        raise AssertionError("short identity was accepted")
print("PASS: direct and nested hook round trips; invalid widths refused")
