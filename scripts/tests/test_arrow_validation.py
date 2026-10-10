# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent Cargo unit-graph controls for scoped correctness composition."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import unittest
from typing import TYPE_CHECKING

import msgspec

if TYPE_CHECKING:
    from collections.abc import Sequence

from scripts.arrow_validation import ValidationError, compose, graph_command, resolve


def package(name: str, *features: str) -> dict[str, object]:
    return {"id": name, "name": name, "features": {feature: [] for feature in features}}


def unit(
    name: str,
    dependencies: Sequence[int] = (),
    *,
    features: Sequence[str] = (),
    platform: str | None = None,
    target: str | None = None,
) -> dict[str, object]:
    return {
        "pkg_id": name,
        "target": {
            "name": target or name,
            "kind": ["lib"],
            "src_path": f"/{name}/lib.rs",
        },
        "dependencies": [{"index": i} for i in dependencies],
        "features": list(features),
        "platform": platform,
        "mode": "build",
    }


def graph(
    units: Sequence[dict[str, object]], roots: Sequence[int] = (0,)
) -> dict[str, object]:
    return {"version": 1, "units": list(units), "roots": list(roots)}


class Cargo:
    def __init__(
        self, packages: Sequence[dict[str, object]], *graphs: dict[str, object]
    ) -> None:
        self.responses = [
            msgspec.json.encode({"packages": list(packages)}),
            *(msgspec.json.encode(value) for value in graphs),
        ]
        self.commands: list[list[str]] = []

    def __call__(self, command: Sequence[str]) -> bytes:
        self.commands.append(list(command))
        return self.responses.pop(0)


class ScopedValidationTests(unittest.TestCase):
    def test_pure_normal_build_and_selected_dev_units_need_no_opt_in(self) -> None:
        cargo = Cargo(
            [package("modeling"), package("macro"), package("testkit")],
            graph([unit("modeling", [1, 2]), unit("macro"), unit("testkit")]),
        )
        command = ["cargo", "check", "-p", "modeling", "--all-targets", "--locked"]
        self.assertEqual(compose(command, run=cargo), command)
        self.assertEqual(len(cargo.commands), 2)

    def test_incidental_unifier_is_not_a_semantic_consumer(self) -> None:
        cargo = Cargo(
            [
                package("quantity"),
                package("pse-workspace-hack"),
                package("arrow-array", "force_validate"),
            ],
            graph(
                [
                    unit("quantity", [1]),
                    unit("pse-workspace-hack", [2]),
                    unit("arrow-array"),
                ]
            ),
        )
        command = ["cargo", "test", "-p", "quantity", "--lib"]
        self.assertEqual(compose(command, run=cargo), command)

    def test_direct_transitive_and_selected_dev_arrow_closures_activate_owner(
        self,
    ) -> None:
        # Cargo's unit graph already selects dev/build edges; the composer does not
        # infer them from an inventory containing other members' dev-dependencies.
        for edges in ([1], [2]):
            with self.subTest(edges=edges):
                packages = [
                    package("owner", "force-validate"),
                    package("arrow-array", "force_validate"),
                    package("bridge"),
                ]
                before = graph(
                    [unit("owner", edges), unit("arrow-array"), unit("bridge", [1])]
                )
                after = graph(
                    [
                        unit("owner", edges, features=["force-validate"]),
                        unit("arrow-array", features=["force_validate"]),
                        unit("bridge", [1]),
                    ]
                )
                cargo = Cargo(packages, before, after)
                result = compose(["cargo", "test", "-p", "owner"], run=cargo)
                self.assertEqual(
                    result,
                    [
                        "cargo",
                        "test",
                        "-p",
                        "owner",
                        "--features",
                        "owner/force-validate",
                    ],
                )
                self.assertEqual(len(cargo.commands), 3)

    def test_unselected_dev_inventory_does_not_activate_arrow(self) -> None:
        cargo = Cargo(
            [
                package("modeling"),
                package("other-dev-consumer", "force-validate"),
                package("arrow-array", "force_validate"),
            ],
            graph([unit("modeling")]),
        )
        self.assertNotIn(
            "--features",
            compose(["cargo", "check", "-p", "modeling", "--lib"], run=cargo),
        )

    def test_additional_selected_roots_each_receive_their_own_opt_in(self) -> None:
        packages = [
            package("pure"),
            package("owner", "force-validate"),
            package("second", "force-validate"),
            package("arrow-data", "force_validate"),
        ]
        before = graph(
            [unit("pure"), unit("owner", [3]), unit("second", [3]), unit("arrow-data")],
            [0, 1, 2],
        )
        after = graph(
            [
                unit("pure"),
                unit("owner", [3]),
                unit("second", [3]),
                unit("arrow-data", features=["force_validate"]),
            ],
            [0, 1, 2],
        )
        result = resolve(
            ["cargo", "test", "-p", "pure", "-powner", "-p", "second"],
            run=Cargo(packages, before, after),
        )
        self.assertEqual(result.packages, ["owner", "pure", "second"])
        self.assertEqual(
            result.command[-1], "owner/force-validate,second/force-validate"
        )
        self.assertEqual(result.command.count("-p"), 2)

    def test_unknown_consuming_root_refuses(self) -> None:
        cargo = Cargo(
            [package("new-owner"), package("arrow-data", "force_validate")],
            graph([unit("new-owner", [1]), unit("arrow-data")]),
        )
        with self.assertRaisesRegex(ValidationError, "no explicit force-validate"):
            compose(["cargo", "check", "-p", "new-owner"], run=cargo)

    def test_opt_in_name_without_actual_leaf_validation_refuses(self) -> None:
        packages = [
            package("owner", "force-validate"),
            package("arrow-cast", "force_validate"),
        ]
        unchanged = graph(
            [unit("owner", [1], features=["force-validate"]), unit("arrow-cast")]
        )
        with self.assertRaisesRegex(ValidationError, "arrow-cast lacks force_validate"):
            compose(
                ["cargo", "test", "-p", "owner"],
                run=Cargo(packages, unchanged, unchanged),
            )

    def test_host_unit_must_validate_independently_of_target_unit(self) -> None:
        packages = [
            package("owner", "force-validate"),
            package("arrow-array", "force_validate"),
        ]
        before = graph(
            [
                unit("owner", [1, 2]),
                unit("arrow-array"),
                unit("arrow-array", platform="target"),
            ]
        )
        after = graph(
            [
                unit("owner", [1, 2]),
                unit("arrow-array"),
                unit("arrow-array", platform="target", features=["force_validate"]),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "host build unit"):
            compose(
                ["cargo", "check", "-p", "owner", "--target", "target"],
                run=Cargo(packages, before, after),
            )

    def test_target_unit_must_validate_independently_of_host_unit(self) -> None:
        packages = [
            package("owner", "force-validate"),
            package("arrow-data", "force_validate"),
        ]
        before = graph(
            [
                unit("owner", [1, 2]),
                unit("arrow-data"),
                unit("arrow-data", platform="target"),
            ]
        )
        after = graph(
            [
                unit("owner", [1, 2]),
                unit("arrow-data", features=["force_validate"]),
                unit("arrow-data", platform="target"),
            ]
        )
        with self.assertRaisesRegex(ValidationError, "target build unit"):
            compose(
                ["cargo", "check", "-p", "owner", "--target", "target"],
                run=Cargo(packages, before, after),
            )

    def test_validation_must_not_add_a_selected_target(self) -> None:
        packages = [
            package("owner", "force-validate"),
            package("arrow-data", "force_validate"),
        ]
        before = graph([unit("owner", [1]), unit("arrow-data")])
        after = graph(
            [
                unit("owner", [1]),
                unit("arrow-data", features=["force_validate"]),
                unit("owner", target="new-bin"),
            ],
            [0, 2],
        )
        with self.assertRaisesRegex(
            ValidationError, "changed selected package/target roots"
        ):
            compose(
                ["cargo", "test", "-p", "owner"], run=Cargo(packages, before, after)
            )

    def test_user_features_target_defaults_and_runtime_arguments_are_preserved(
        self,
    ) -> None:
        packages = [
            package("owner", "force-validate", "custom"),
            package("arrow-data", "force_validate"),
        ]
        before = graph([unit("owner", [1]), unit("arrow-data")])
        after = graph(
            [unit("owner", [1]), unit("arrow-data", features=["force_validate"])]
        )
        command = [
            "cargo",
            "nextest",
            "run",
            "--package=owner",
            "--target=x86_64-unknown-linux-gnu",
            "--no-default-features",
            "--features",
            "owner/custom",
            "--cargo-profile",
            "release",
            "--profile",
            "ci",
            "--test",
            "specific",
            "-E",
            "test(exact)",
            "--",
            "--nocapture",
        ]
        cargo = Cargo(packages, before, after)
        result = compose(command, run=cargo)
        self.assertEqual(result[:-4], command[:-2])
        self.assertEqual(
            result[-4:], ["--features", "owner/force-validate", "--", "--nocapture"]
        )
        graph_args = cargo.commands[1]
        self.assertIn("x86_64-unknown-linux-gnu", graph_args)
        self.assertIn("--no-default-features", graph_args)
        self.assertIn("owner/custom", graph_args)
        self.assertIn("release", graph_args)
        self.assertNotIn("ci", graph_args)
        self.assertNotIn("test(exact)", graph_args)

    def test_unsupported_scope_option_refuses(self) -> None:
        with self.assertRaisesRegex(ValidationError, "unsupported scope option"):
            graph_command(["cargo", "test", "--unknown-compilation-mode"])

    def test_run_requires_an_exact_target_before_build_graph_substitution(self) -> None:
        with self.assertRaisesRegex(ValidationError, "explicit --bin or --example"):
            graph_command(["cargo", "run", "-p", "owner"])
        self.assertEqual(
            graph_command(
                ["cargo", "run", "-p", "owner", "--bin", "worker", "--", "argument"]
            ),
            [
                "cargo",
                "build",
                "--unit-graph",
                "-Z",
                "unstable-options",
                "-p",
                "owner",
                "--bin",
                "worker",
            ],
        )

    def test_exact_named_target_and_workspace_exclusions_reach_cargo(self) -> None:
        self.assertEqual(
            graph_command(
                [
                    "cargo",
                    "check",
                    "--workspace",
                    "--exclude=owner",
                    "--test",
                    "named",
                    "--all-features",
                ]
            ),
            [
                "cargo",
                "check",
                "--unit-graph",
                "-Z",
                "unstable-options",
                "--workspace",
                "--exclude",
                "owner",
                "--test",
                "named",
                "--all-features",
            ],
        )


if __name__ == "__main__":
    unittest.main()
