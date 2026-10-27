"""Check whether convert validates already-constructed Struct instances."""

import json

import msgspec


class Fact(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    value: float


class Snapshot(msgspec.Struct, frozen=True, forbid_unknown_fields=True):
    facts: tuple[Fact, ...]


assert msgspec.__version__ == "0.21.1"
bad = Fact(value="bad")
whole = Snapshot(facts=(bad,))
same = msgspec.convert(whole, Snapshot, strict=True)
nested = msgspec.convert({"facts": [bad]}, Snapshot, strict=True)
assert same is whole
assert same.facts[0].value == "bad"
assert nested.facts[0] is bad
try:
    msgspec.convert({"facts": [{"value": "bad"}]}, Snapshot, strict=True)
except msgspec.ValidationError as error:
    mapping_error = str(error)
else:
    raise AssertionError("The raw mapping must fail validation")

print(
    json.dumps(
        {
            "msgspec_version": msgspec.__version__,
            "same_target_returns_original": same is whole,
            "invalid_same_target_field": same.facts[0].value,
            "mapping_retains_invalid_nested_struct": nested.facts[0] is bad,
            "raw_mapping_refused": mapping_error,
        },
        indent=2,
    )
)
