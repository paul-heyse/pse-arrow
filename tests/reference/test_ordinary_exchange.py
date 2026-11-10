# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreign binary production and refusal controls using actual native contracts."""

from __future__ import annotations

import math
import os
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from google.protobuf.message import Message

from oracles import ordinary_exchange as exchange
from proto_exchange import (
    message_class,
    message_field,
    reference_type,
    repeated_field,
    scalar_field,
    set_scalar,
)

if TYPE_CHECKING:
    from collections.abc import Mapping

DESCRIPTORS = (
    exchange.ROOT / "crates/pse-runtime/src/generated/reference-exchange/descriptor.bin"
)


class OrdinaryExchangeUnitTests(unittest.TestCase):
    """Independent units and inputs need no candidate or fabricated physical DTO."""

    def test_original_fixed_roles_are_read_before_solving(self) -> None:
        """Original fixed roles are read before solving."""
        recycle = {
            "feed.state.flow",
            "feed.state.T",
            "feed.state.pressure",
            "feed.state.z[benzene]",
            "feed.state.z[toluene]",
        }
        for case in exchange.CASE_NAMES:
            selected = exchange._fixed_inputs(case)
            self.assertEqual(
                len(selected), 14 if case in exchange.CASE_NAMES[:3] else 5
            )
            self.assertTrue(
                all(coordinate.is_fixed() for coordinate in selected.values())
            )
            if case not in exchange.CASE_NAMES[:3]:
                self.assertEqual(set(selected), recycle)
            elif case == exchange.CASE_NAMES[1]:
                self.assertIn("exchanger.area", selected)
                self.assertNotIn("exchanger.cold.outlet.T", selected)
            else:
                self.assertIn("exchanger.cold.outlet.T", selected)
                self.assertNotIn("exchanger.area", selected)

    def test_ieee_binary_coordinates_preserve_signed_zero_and_nan_payload(self) -> None:
        """Ieee binary coordinates preserve signed zero and nan payload."""
        for bits in (0, 1 << 63, 0x7FF8000000001234, 0x3FF0000000000000):
            self.assertEqual(exchange.float_bits(exchange.bits_float(bits)), bits)

    def test_artifacts_are_exact_source_paths_with_no_caller_reads(self) -> None:
        """Artifacts are exact source paths with no caller reads."""
        paths = exchange.artifact_paths()
        self.assertEqual(len(paths), len(set(paths)))
        self.assertIn("packages/reference/conformance.toml", paths)
        self.assertIn("packages/reference/data/perry7/data/perry_liquid.parquet", paths)
        for invalid in (
            "/etc/passwd",
            "../candidate.bin",
            "packages/../candidate.bin",
            "packages//reference/conformance.toml",
            "packages\\reference\\conformance.toml",
        ):
            with self.subTest(path=invalid), self.assertRaises(ValueError):
                exchange._safe_path(invalid)

    def test_artifact_population_uses_the_physical_root_and_package_document_layout(
        self,
    ) -> None:
        """Include the selected physical package and exclude unrelated package files."""
        manifest = b"""
            [[runs]]
            name = "seed"
            package = "seed-data"
            physical = "distinct-physical"
        """
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for package in (
                "seed-data",
                "distinct-physical",
                "campaign",
                "ordinary-delivery",
                "data/ordinary-bt",
            ):
                package_root = root / "packages/reference" / package
                (package_root / "models").mkdir(parents=True)
                (package_root / "package.toml").touch()
                (package_root / "README.toml").touch()
                (package_root / "models/selected.pse").touch()
            with patch.object(exchange, "ROOT", root):
                paths = exchange.artifact_paths(manifest)
            self.assertIn(
                "packages/reference/distinct-physical/models/selected.pse", paths
            )
            self.assertFalse(any(path.endswith("README.toml") for path in paths))

    def test_absent_optional_identity_is_not_an_explicit_nil_identity(self) -> None:
        """Absent optional identity is not an explicit nil identity."""
        pool = reference_type(DESCRIPTORS.read_bytes()).DESCRIPTOR.file.pool
        key = message_class(
            pool.FindMessageTypeByName("pse.reference.v1.QuantityTypeKey")
        )()
        self.assertIsNone(exchange._optional_identity(key, "basis"))
        set_scalar(message_field(key, "basis"), "id", bytes(16))
        self.assertTrue(key.HasField("basis"))
        with self.assertRaisesRegex(ValueError, "nil"):
            exchange._optional_identity(key, "basis")
        set_scalar(message_field(key, "basis"), "id", bytes([0x31]) * 16)
        self.assertEqual(exchange._optional_identity(key, "basis"), bytes([0x31]) * 16)

    def test_scientific_input_refuses_oversized_bytes_before_parsing(self) -> None:
        """Enforce the physical artifact budget before admitting any content."""
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "property.tsv"
            artifact.write_bytes(b"12345")
            with (
                patch.object(exchange, "_safe_path", return_value=artifact),
                self.assertRaisesRegex(ValueError, "byte limit"),
            ):
                exchange._read_artifact("property.tsv", 4)

    def test_invalid_wire_contract_refuses_before_scientific_input_reads(self) -> None:
        """Invalid wire contract refuses before scientific input reads."""
        descriptors = DESCRIPTORS.read_bytes()
        pool = reference_type(descriptors).DESCRIPTOR.file.pool
        contract_type = message_class(
            pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
        )
        with patch.object(
            exchange,
            "_validate_artifacts",
            side_effect=AssertionError(
                "invalid headers cannot acquire scientific inputs"
            ),
        ):
            for version_number, case in ((2, exchange.CASE_NAMES[0]), (1, "unknown")):
                contract = contract_type(case_name=case)
                set_scalar(message_field(contract, "version"), "value", version_number)
                with (
                    self.subTest(version=version_number, case=case),
                    self.assertRaises(ValueError),
                ):
                    exchange.produce_reference(
                        descriptors, contract.SerializeToString()
                    )


class OrdinaryExchangeSolverTests(unittest.TestCase):
    """Independent exchange refusal controls that exercise a real solver."""

    def test_nonfinite_failed_witness_cannot_supply_a_reference_failure(self) -> None:
        """Nonfinite failed witness cannot supply a reference failure."""
        read_original = exchange.ordinary_recycle.overheated_observations

        def changed(
            failure: exchange.ordinary_recycle.OverheatedFailure,
        ) -> dict[str, float]:
            witness = read_original(failure)
            witness["energy transport residual"] = math.nan
            return witness

        with (
            patch.object(
                exchange.ordinary_recycle,
                "overheated_observations",
                side_effect=changed,
            ),
            self.assertRaisesRegex(RuntimeError, "physical contradiction"),
        ):
            exchange._reference_outputs(exchange.CASE_NAMES[-1])


class OrdinaryExchangeBinaryTests(unittest.TestCase):
    """Require the original native contract fixture for binary controls."""

    @classmethod
    def setUpClass(cls) -> None:
        """Load the five actual native contracts selected by the recipe."""
        directory = os.environ.get("PSE_ORDINARY_REFERENCE_CONTRACTS")
        if directory is None:
            raise RuntimeError(
                "PSE_ORDINARY_REFERENCE_CONTRACTS must select the five "
                "actual native contract binaries"
            )
        cls.contract_directory = Path(directory)
        cls.descriptors = DESCRIPTORS.read_bytes()
        cls.envelope_type = reference_type(cls.descriptors)
        pool = cls.envelope_type.DESCRIPTOR.file.pool
        cls.contract_type = message_class(
            pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
        )

    def contract(self, case: str) -> Message:
        """Decode one selected original contract in its private descriptor world."""
        return self.contract_type.FromString(
            (
                self.contract_directory
                / f"{case.rsplit('.', maxsplit=1)[-1]}.contract.bin"
            ).read_bytes()
        )

    def test_actual_physical_owners_and_typed_identity_wrappers_agree(self) -> None:
        """Actual physical owners and typed identity wrappers agree."""
        checked: set[str] = set()
        for case in exchange.CASE_NAMES:
            contract = self.contract(case)
            for item in (
                *repeated_field(contract, "specification", Message),
                *repeated_field(contract, "observables", Message),
            ):
                for field in (
                    "quantity",
                    "quantity_kind",
                    "canonical_unit",
                    "representation_unit",
                    "basis",
                    "reference_state",
                    "subject",
                ):
                    if not message_field(item, "physical").HasField(field):
                        continue
                    owner = exchange._message(message_field(item, "physical"), field)
                    identity = exchange._message(owner, "id")
                    self.assertEqual(
                        exchange._identity(owner), exchange._identity(identity)
                    )
                    checked.add(owner.DESCRIPTOR.full_name)
                    for invalid in (b"", bytes(16), bytes(15), bytes(17)):
                        changed = type(owner)()
                        changed.CopyFrom(owner)
                        set_scalar(message_field(changed, "id"), "id", invalid)
                        with (
                            self.subTest(owner=owner.DESCRIPTOR.full_name),
                            self.assertRaises(ValueError),
                        ):
                            exchange._identity(changed)
                    for missing in ("owner", "wrapper"):
                        changed = type(owner)()
                        changed.CopyFrom(owner)
                        if missing == "owner":
                            changed.ClearField("id")
                        else:
                            message_field(changed, "id").ClearField("id")
                        with (
                            self.subTest(
                                owner=owner.DESCRIPTOR.full_name, missing=missing
                            ),
                            self.assertRaises(ValueError),
                        ):
                            exchange._identity(changed)
        self.assertEqual(checked, set(exchange._IDENTITY_OWNERS))
        with self.assertRaisesRegex(ValueError, "unsupported"):
            exchange._identity(self.contract(exchange.CASE_NAMES[0]))

    def test_species_display_alias_keeps_the_original_subject_identity(self) -> None:
        """Species display alias keeps the original subject identity."""
        for case in exchange.CASE_NAMES:
            original = next(
                message_field(item, "physical")
                for item in repeated_field(
                    self.contract(case), "specification", Message
                )
                if ".z[" in scalar_field(item, "key", str)
            )
            for label in ("species", "chemistry.species", "display alias"):
                changed = type(original)()
                changed.CopyFrom(original)
                set_scalar(message_field(changed, "subject"), "name", label)
                exchange._validate_physical(changed, "MoleFraction")
                self.assertEqual(
                    exchange._identity(message_field(changed, "subject")),
                    exchange._SPECIES_KIND_ID,
                )

    def test_all_five_actual_contracts_produce_the_complete_ordered_population(
        self,
    ) -> None:
        """All five actual contracts produce the complete ordered population."""
        for case in exchange.CASE_NAMES:
            with self.subTest(case=case):
                contract = self.contract(case)
                actual = self.envelope_type.FromString(
                    exchange.produce_reference(
                        self.descriptors, contract.SerializeToString()
                    )
                )
                self.assertEqual(
                    message_field(actual, "meaning"), message_field(contract, "meaning")
                )
                self.assertEqual(
                    list(repeated_field(actual, "required", bytes)),
                    [
                        scalar_field(item, "output", bytes)
                        for item in repeated_field(contract, "observables", Message)
                    ],
                )
                if case == exchange.CASE_NAMES[-1]:
                    self.assertEqual(
                        message_field(actual, "outcome").WhichOneof("kind"), "failed"
                    )
                    self.assertEqual(
                        scalar_field(
                            message_field(actual, "outcome", "failed"), "code", int
                        ),
                        "trial_rejected",
                    )
                    self.assertEqual(
                        scalar_field(
                            message_field(actual, "outcome", "failed"), "stage", str
                        ),
                        "overheated_tear",
                    )
                    continue
                self.assertEqual(
                    message_field(actual, "outcome").WhichOneof("kind"), "completed"
                )
                self.assertFalse(
                    scalar_field(
                        message_field(actual, "outcome", "completed"), "blocks", int
                    )
                )
                outputs = repeated_field(
                    message_field(actual, "outcome", "completed"), "outputs", Message
                )
                self.assertEqual(
                    len(outputs), len(repeated_field(contract, "observables", Message))
                )
                for produced, expected in zip(
                    outputs,
                    repeated_field(contract, "observables", Message),
                    strict=True,
                ):
                    self.assertEqual(
                        scalar_field(produced, "id", bytes),
                        scalar_field(expected, "output", bytes),
                    )
                    self.assertTrue(
                        math.isfinite(
                            exchange.bits_float(scalar_field(produced, "value", int))
                        )
                    )
                    for output, source in (
                        ("basis", "basis"),
                        ("datum", "reference_state"),
                    ):
                        key = message_field(expected, "physical", "quantity", "key")
                        self.assertEqual(
                            produced.HasField(output), key.HasField(source)
                        )
                        if key.HasField(source):
                            self.assertEqual(
                                getattr(produced, output), getattr(key, source).id
                            )
                values = {
                    scalar_field(expected, "key", str): exchange.bits_float(
                        scalar_field(produced, "value", int)
                    )
                    for produced, expected in zip(
                        outputs,
                        repeated_field(contract, "observables", Message),
                        strict=True,
                    )
                }
                if case == exchange.CASE_NAMES[3]:
                    for species in ("benzene", "toluene"):
                        self.assertAlmostEqual(
                            values[f"fresh.{species}"],
                            values[f"vapor.{species}"] + values[f"purge.{species}"],
                            places=7,
                        )
                    self.assertAlmostEqual(
                        values["fresh.energy"] + values["heater.duty"],
                        values["vapor.energy"] + values["purge.energy"],
                        places=4,
                    )
                else:
                    self.assertAlmostEqual(
                        values["exchanger.hot.duty"] + values["exchanger.cold.duty"],
                        0.0,
                        places=4,
                    )
                    for side in ("hot", "cold"):
                        self.assertAlmostEqual(
                            values[f"exchanger.{side}.inlet.flow"],
                            values[f"exchanger.{side}.outlet.flow"],
                            places=7,
                        )

    def test_changed_correspondence_refuses_before_the_reference_calculation(
        self,
    ) -> None:
        """Changed correspondence refuses before the reference calculation."""
        original = self.contract(exchange.CASE_NAMES[0])

        def mutations() -> list[Message]:
            result = []
            for _ in range(19):
                copy = self.contract_type()
                copy.CopyFrom(original)
                result.append(copy)
            set_scalar(message_field(result[0], "version"), "value", 2)
            set_scalar(result[1], "case_name", "ordinary_delivery.unknown")
            repeated_field(result[2], "observables", Message).reverse()
            set_scalar(
                repeated_field(result[3], "observables", Message)[1],
                "output",
                scalar_field(
                    repeated_field(result[3], "observables", Message)[0],
                    "output",
                    bytes,
                ),
            )
            del repeated_field(result[4], "specification", Message)[-1]
            set_scalar(
                repeated_field(result[5], "specification", Message)[0],
                "value",
                exchange.float_bits(999.0),
            )
            set_scalar(
                repeated_field(result[6], "artifacts", Message)[0], "sha256", bytes(32)
            )
            set_scalar(
                repeated_field(result[7], "artifacts", Message)[0],
                "path",
                "../candidate.bin",
            )
            exponent = repeated_field(
                message_field(
                    repeated_field(result[8], "observables", Message)[0],
                    "physical",
                    "representation_unit",
                    "dimension",
                    "exponents",
                ),
                "values",
                Message,
            )[0]
            set_scalar(exponent, "num", scalar_field(exponent, "num", int) + 1)
            message_field(
                repeated_field(result[9], "observables", Message)[0], "physical"
            ).ClearField("basis")
            enthalpy = next(
                item
                for item in repeated_field(result[10], "observables", Message)
                if scalar_field(item, "key", str).endswith(".h")
            )
            set_scalar(
                message_field(enthalpy, "physical", "reference_state", "pressure"),
                "value",
                exchange.float_bits(101325.0),
            )
            scalar = next(
                item
                for item in repeated_field(result[11], "observables", Message)
                if scalar_field(item, "key", str) == "cost.area_count_ft2"
            )
            set_scalar(
                message_field(scalar, "physical", "quantity", "key", "basis"),
                "id",
                bytes(16),
            )
            money = next(
                item
                for item in repeated_field(result[12], "observables", Message)
                if scalar_field(item, "key", str) == "cost.capital"
            )
            set_scalar(
                message_field(money, "physical", "representation_unit"),
                "symbol",
                "USD_2018",
            )
            set_scalar(
                message_field(
                    repeated_field(result[13], "observables", Message)[0],
                    "physical",
                    "quantity_kind",
                ),
                "extensive",
                False,
            )
            pressure = next(
                item
                for item in repeated_field(result[14], "specification", Message)
                if scalar_field(item, "key", str).endswith(".pressure")
            )
            set_scalar(
                message_field(pressure, "physical", "quantity", "key", "scale_kind"),
                "value",
                "difference",
            )
            numerical_start = repeated_field(result[15], "specification", Message).add()
            numerical_start.CopyFrom(
                next(
                    item
                    for item in repeated_field(original, "specification", Message)
                    if scalar_field(item, "key", str).endswith(".T")
                )
            )
            set_scalar(numerical_start, "key", "exchanger.hot.outlet.T")
            set_scalar(numerical_start, "value", exchange.float_bits(360.0))
            species = next(
                item
                for item in repeated_field(result[16], "specification", Message)
                if ".z[" in scalar_field(item, "key", str)
            )
            set_scalar(
                message_field(species, "physical", "subject", "id"),
                "id",
                bytes([0x52]) * 16,
            )
            set_scalar(
                message_field(species, "physical", "quantity", "key", "subject_kind"),
                "id",
                bytes([0x52]) * 16,
            )
            duty = next(
                item
                for item in repeated_field(result[17], "observables", Message)
                if scalar_field(item, "key", str).endswith(".duty")
            )
            transported = next(
                item
                for item in repeated_field(original, "observables", Message)
                if scalar_field(item, "key", str).endswith(".energy_flow")
            )
            message_field(duty, "physical", "reference_state").CopyFrom(
                message_field(transported, "physical", "reference_state")
            )
            message_field(
                message_field(duty, "physical", "quantity", "key"), "reference_state"
            ).CopyFrom(
                message_field(
                    message_field(transported, "physical", "quantity", "key"),
                    "reference_state",
                )
            )
            duty_role = next(
                item
                for item in repeated_field(result[18], "observables", Message)
                if scalar_field(item, "key", str).endswith(".duty")
            )
            self.assertEqual(
                scalar_field(
                    message_field(duty_role, "physical", "representation_unit"),
                    "symbol",
                    str,
                ),
                scalar_field(
                    message_field(transported, "physical", "representation_unit"),
                    "symbol",
                    str,
                ),
            )
            message_field(duty_role, "physical").CopyFrom(
                message_field(transported, "physical")
            )
            return result

        with patch.object(
            exchange,
            "_reference_outputs",
            side_effect=AssertionError(
                "calculation must not run for a changed contract"
            ),
        ):
            for index, contract in enumerate(mutations()):
                with self.subTest(mutation=index), self.assertRaises(ValueError):
                    exchange.produce_reference(
                        self.descriptors, contract.SerializeToString()
                    )

    def test_reference_math_failure_cannot_become_a_passing_failed_outcome(
        self,
    ) -> None:
        """Reference math failure cannot become a passing failed outcome."""
        contract = self.contract(exchange.CASE_NAMES[-1])
        with (
            patch.object(
                exchange.ordinary_recycle,
                "overheated_failure",
                side_effect=RuntimeError("missing solver or setup failure"),
            ),
            self.assertRaisesRegex(RuntimeError, "missing solver"),
        ):
            exchange.produce_reference(self.descriptors, contract.SerializeToString())

    def test_solve_uses_verified_property_bytes_despite_substitute_then_restore(
        self,
    ) -> None:
        """Use verified input bytes while the checkout path holds a substitute."""
        contract = self.contract(exchange.CASE_NAMES[-1])
        path = exchange.PROPERTY_ARTIFACTS[0]
        original_path = exchange._safe_path
        verified = original_path(path).read_bytes()
        parse = exchange.parse_inputs
        calculate = exchange._reference_outputs
        with tempfile.TemporaryDirectory() as directory:
            replacement = Path(directory) / "property.tsv"
            replacement.write_bytes(verified)

            def parse_during_replacement(
                artifacts: Mapping[str, bytes],
            ) -> dict[str, exchange.SpeciesInput]:
                replacement.write_bytes(b"substituted invalid property data")
                self.assertEqual(artifacts[path], verified)
                return parse(artifacts)

            def solve_before_restoration(
                case_name: str,
                *,
                inputs: Mapping[str, exchange.SpeciesInput],
                environment: exchange.ReferenceEnvironment,
            ) -> dict[str, float] | None:
                self.assertNotEqual(replacement.read_bytes(), verified)
                try:
                    return calculate(case_name, inputs=inputs, environment=environment)
                finally:
                    replacement.write_bytes(verified)

            with (
                patch.object(
                    exchange,
                    "_safe_path",
                    side_effect=lambda relative: (
                        replacement if relative == path else original_path(relative)
                    ),
                ),
                patch.object(
                    exchange, "parse_inputs", side_effect=parse_during_replacement
                ) as parsed,
                patch.object(
                    exchange, "_reference_outputs", side_effect=solve_before_restoration
                ),
                patch.object(
                    exchange.ordinary_recycle,
                    "load_inputs",
                    side_effect=AssertionError(
                        "exchange reopened the property sources"
                    ),
                ),
            ):
                payload = exchange.produce_reference(
                    self.descriptors, contract.SerializeToString()
                )
            self.assertEqual(parsed.call_count, 1)
            self.assertEqual(
                scalar_field(
                    message_field(
                        self.envelope_type.FromString(payload), "outcome", "failed"
                    ),
                    "stage",
                    str,
                ),
                "overheated_tear",
            )


if __name__ == "__main__":
    unittest.main()
