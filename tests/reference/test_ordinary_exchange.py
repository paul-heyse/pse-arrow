# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreign binary production and refusal controls using actual native contracts."""

from __future__ import annotations

import math
import os
import tempfile
import unittest
from collections.abc import Mapping
from pathlib import Path
from unittest.mock import patch

from google.protobuf import message_factory
from google.protobuf.message import Message

from oracles import ordinary_exchange as exchange
from proto_exchange import reference_type

DESCRIPTORS = (
    exchange.ROOT / "crates/pse-runtime/src/generated/reference-exchange/descriptor.bin"
)


class OrdinaryExchangeUnitTests(unittest.TestCase):
    """Independent units and inputs need no candidate or fabricated physical DTO."""

    def test_original_fixed_roles_are_read_before_solving(self) -> None:
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
        for bits in (0, 1 << 63, 0x7FF8000000001234, 0x3FF0000000000000):
            self.assertEqual(exchange.float_bits(exchange.bits_float(bits)), bits)

    def test_artifacts_are_exact_source_paths_with_no_caller_reads(self) -> None:
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
        pool = reference_type(DESCRIPTORS.read_bytes()).DESCRIPTOR.file.pool
        key = message_factory.GetMessageClass(
            pool.FindMessageTypeByName("pse.reference.v1.QuantityTypeKey")
        )()
        self.assertIsNone(exchange._optional_identity(key, "basis"))
        key.basis.id = bytes(16)
        self.assertTrue(key.HasField("basis"))
        with self.assertRaisesRegex(ValueError, "nil"):
            exchange._optional_identity(key, "basis")
        key.basis.id = bytes([0x31]) * 16
        self.assertEqual(exchange._optional_identity(key, "basis"), bytes([0x31]) * 16)

    def test_scientific_input_refuses_oversized_bytes_before_parsing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "property.tsv"
            artifact.write_bytes(b"12345")
            with (
                patch.object(exchange, "_safe_path", return_value=artifact),
                self.assertRaisesRegex(ValueError, "byte limit"),
            ):
                exchange._read_artifact("property.tsv", 4)

    def test_invalid_wire_contract_refuses_before_scientific_input_reads(self) -> None:
        descriptors = DESCRIPTORS.read_bytes()
        pool = reference_type(descriptors).DESCRIPTOR.file.pool
        contract_type = message_factory.GetMessageClass(
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
                contract.version.value = version_number
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
    """A missing native contract fixture is an error, never passing reference evidence."""

    @classmethod
    def setUpClass(cls) -> None:
        directory = os.environ.get("PSE_ORDINARY_REFERENCE_CONTRACTS")
        if directory is None:
            raise RuntimeError(
                "PSE_ORDINARY_REFERENCE_CONTRACTS must select the five actual native contract binaries"
            )
        cls.contract_directory = Path(directory)
        cls.descriptors = DESCRIPTORS.read_bytes()
        cls.envelope_type = reference_type(cls.descriptors)
        pool = cls.envelope_type.DESCRIPTOR.file.pool
        cls.contract_type = message_factory.GetMessageClass(
            pool.FindMessageTypeByName("pse.reference.v1.ReferenceCaseContract")
        )

    def contract(self, case: str) -> Message:
        return self.contract_type.FromString(
            (
                self.contract_directory
                / f"{case.rsplit('.', maxsplit=1)[-1]}.contract.bin"
            ).read_bytes()
        )

    def test_actual_physical_owners_and_typed_identity_wrappers_agree(self) -> None:
        checked: set[str] = set()
        for case in exchange.CASE_NAMES:
            contract = self.contract(case)
            for item in (*contract.specification, *contract.observables):
                for field in (
                    "quantity",
                    "quantity_kind",
                    "canonical_unit",
                    "representation_unit",
                    "basis",
                    "reference_state",
                    "subject",
                ):
                    if not item.physical.HasField(field):
                        continue
                    owner = exchange._message(item.physical, field)
                    identity = exchange._message(owner, "id")
                    self.assertEqual(
                        exchange._identity(owner), exchange._identity(identity)
                    )
                    checked.add(owner.DESCRIPTOR.full_name)
                    for invalid in (b"", bytes(16), bytes(15), bytes(17)):
                        changed = type(owner)()
                        changed.CopyFrom(owner)
                        changed.id.id = invalid
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
                            changed.id.ClearField("id")
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
        for case in exchange.CASE_NAMES:
            original = next(
                item.physical
                for item in self.contract(case).specification
                if ".z[" in item.key
            )
            for label in ("species", "chemistry.species", "display alias"):
                changed = type(original)()
                changed.CopyFrom(original)
                changed.subject.name = label
                exchange._validate_physical(changed, "MoleFraction")
                self.assertEqual(
                    exchange._identity(changed.subject), exchange._SPECIES_KIND_ID
                )

    def test_all_five_actual_contracts_produce_the_complete_ordered_population(
        self,
    ) -> None:
        for case in exchange.CASE_NAMES:
            with self.subTest(case=case):
                contract = self.contract(case)
                actual = self.envelope_type.FromString(
                    exchange.produce_reference(
                        self.descriptors, contract.SerializeToString()
                    )
                )
                self.assertEqual(actual.meaning, contract.meaning)
                self.assertEqual(
                    list(actual.required),
                    [item.output for item in contract.observables],
                )
                if case == exchange.CASE_NAMES[-1]:
                    self.assertEqual(actual.outcome.WhichOneof("kind"), "failed")
                    self.assertEqual(actual.outcome.failed.code, "trial_rejected")
                    self.assertEqual(actual.outcome.failed.stage, "overheated_tear")
                    continue
                self.assertEqual(actual.outcome.WhichOneof("kind"), "completed")
                self.assertFalse(actual.outcome.completed.blocks)
                outputs = actual.outcome.completed.outputs
                self.assertEqual(len(outputs), len(contract.observables))
                for produced, expected in zip(
                    outputs, contract.observables, strict=True
                ):
                    self.assertEqual(produced.id, expected.output)
                    self.assertTrue(math.isfinite(exchange.bits_float(produced.value)))
                    for output, source in (
                        ("basis", "basis"),
                        ("datum", "reference_state"),
                    ):
                        key = expected.physical.quantity.key
                        self.assertEqual(
                            produced.HasField(output), key.HasField(source)
                        )
                        if key.HasField(source):
                            self.assertEqual(
                                getattr(produced, output), getattr(key, source).id
                            )
                values = {
                    expected.key: exchange.bits_float(produced.value)
                    for produced, expected in zip(
                        outputs, contract.observables, strict=True
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
        original = self.contract(exchange.CASE_NAMES[0])

        def mutations() -> list[Message]:
            result = []
            for _ in range(19):
                copy = self.contract_type()
                copy.CopyFrom(original)
                result.append(copy)
            result[0].version.value = 2
            result[1].case_name = "ordinary_delivery.unknown"
            result[2].observables.reverse()
            result[3].observables[1].output = result[3].observables[0].output
            del result[4].specification[-1]
            result[5].specification[0].value = exchange.float_bits(999.0)
            result[6].artifacts[0].sha256 = bytes(32)
            result[7].artifacts[0].path = "../candidate.bin"
            result[8].observables[
                0
            ].physical.representation_unit.dimension.exponents.values[0].num += 1
            result[9].observables[0].physical.ClearField("basis")
            enthalpy = next(
                item for item in result[10].observables if item.key.endswith(".h")
            )
            enthalpy.physical.reference_state.pressure.value = exchange.float_bits(
                101325.0
            )
            scalar = next(
                item
                for item in result[11].observables
                if item.key == "cost.area_count_ft2"
            )
            scalar.physical.quantity.key.basis.id = bytes(16)
            money = next(
                item for item in result[12].observables if item.key == "cost.capital"
            )
            money.physical.representation_unit.symbol = "USD_2018"
            result[13].observables[0].physical.quantity_kind.extensive = False
            pressure = next(
                item
                for item in result[14].specification
                if item.key.endswith(".pressure")
            )
            pressure.physical.quantity.key.scale_kind.value = "difference"
            numerical_start = result[15].specification.add()
            numerical_start.CopyFrom(
                next(item for item in original.specification if item.key.endswith(".T"))
            )
            numerical_start.key = "exchanger.hot.outlet.T"
            numerical_start.value = exchange.float_bits(360.0)
            species = next(
                item for item in result[16].specification if ".z[" in item.key
            )
            species.physical.subject.id.id = bytes([0x52]) * 16
            species.physical.quantity.key.subject_kind.id = bytes([0x52]) * 16
            duty = next(
                item for item in result[17].observables if item.key.endswith(".duty")
            )
            transported = next(
                item
                for item in original.observables
                if item.key.endswith(".energy_flow")
            )
            duty.physical.reference_state.CopyFrom(transported.physical.reference_state)
            duty.physical.quantity.key.reference_state.CopyFrom(
                transported.physical.quantity.key.reference_state
            )
            duty_role = next(
                item for item in result[18].observables if item.key.endswith(".duty")
            )
            self.assertEqual(
                duty_role.physical.representation_unit.symbol,
                transported.physical.representation_unit.symbol,
            )
            duty_role.physical.CopyFrom(transported.physical)
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
                self.envelope_type.FromString(payload).outcome.failed.stage,
                "overheated_tear",
            )


if __name__ == "__main__":
    unittest.main()
