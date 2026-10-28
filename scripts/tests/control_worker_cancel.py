# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Cancel one owned worker study through the native-independent public client."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import anyio

from scripts.tests.control_api_actual_check import NativeBlocker, actor_credentials


async def cancel(args: argparse.Namespace, blocker: NativeBlocker) -> bytes:
    import msgspec  # noqa: PLC0415

    from pse.control import ControlClient  # noqa: PLC0415
    from pse.contracts.documents import (  # noqa: PLC0415
        ControlCancelRequest,
        ControlReconcileRequest,
    )

    key, password = actor_credentials(args.actor)
    request = ControlCancelRequest(
        document_version=1,
        issuing_store=args.issuing_store,
        operation_id=args.operation,
        study=args.study,
    )
    async with ControlClient(
        args.endpoint, args.namespace, args.database, None,
        issuing_store=args.issuing_store, timeout=30,
        maximum_request_bytes=64 * 1024, maximum_response_bytes=2 * 1024 * 1024,
    ) as client:
        await client.sign_in(key, password)
        receipt = await client.cancel_study(request)
        assert receipt.status == "accepted" and receipt.cancelled
        assert not receipt.already_concluded
        assert receipt.operation_id == args.operation and receipt.study == args.study
        assert await client.cancel_study(request) == receipt
        reconciled = await client.reconcile(ControlReconcileRequest(
            document_version=1,
            issuing_store=args.issuing_store,
            operation_id=args.operation,
        ))
        assert reconciled.receipt == receipt
    assert not blocker.attempts, "Public control attempted a scientific import"
    assert not {"pse._native", "pse._build", "pse._scientific"}.intersection(sys.modules)
    return msgspec.json.encode(receipt)


async def bounded_cancel(args: argparse.Namespace, blocker: NativeBlocker) -> bytes:
    # Signin, command, exact replay, reconciliation and close share one outer clock.
    with anyio.fail_after(30):
        return await cancel(args, blocker)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("endpoint", "namespace", "database", "issuing-store", "operation", "study"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--actor", type=Path, required=True)
    args = parser.parse_args()
    blocker = NativeBlocker()
    sys.meta_path.insert(0, blocker)
    sys.stdout.buffer.write(anyio.run(bounded_cancel, args, blocker))


if __name__ == "__main__":
    main()
