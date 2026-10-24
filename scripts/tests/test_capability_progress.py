# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Foreground wire observations retain native references and attached ownership."""
from __future__ import annotations
import asyncio
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from fastmcp import Client, FastMCP
from fastmcp.tools import ToolResult
from scripts.capabilities import process
from scripts.capabilities.worker_progress import OwnerReference, ProgressObserver, ProgressReporter, read
from scripts.capabilities.workers import WorkerCall
ROOT = Path(__file__).resolve().parents[2]

class ProgressTests(unittest.IsolatedAsyncioTestCase):
    async def test_latest_snapshot_retains_issued_reference_and_checks_request(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'progress.json'
            reporter = ProgressReporter(path, 'rust-test')
            reference = OwnerReference(kind='invocation', locator='/issued/original.json', registry='/original/registry', ownership='borrowed')
            reporter.publish('execution', 'started', reference=reference)
            reporter.publish('request', 'completed')
            actual = read(path, 'rust-test')
            self.assertIsNotNone(actual)
            assert actual is not None
            self.assertEqual(actual.sequence, 2)
            self.assertEqual(actual.references, (reference,))
            with self.assertRaisesRegex(ValueError, 'another operation'):
                read(path, 'assessment')
            path.write_bytes(b'x' * (16 * 1024 + 1))
            with self.assertRaisesRegex(ValueError, 'exceeds'):
                read(path, 'rust-test')

    async def test_no_context_fallback_and_bad_snapshot_do_not_invent_settlement(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'progress.json'
            observer = ProgressObserver(path, 'assessment')
            self.assertIsNone(observer.context)
            path.write_text('{bad')
            await observer.update()
            self.assertEqual(observer.omissions, ['worker progress unavailable'])
            self.assertEqual(observer.recovery()['references'], [])
            ProgressReporter(path, 'assessment').publish('assessment', 'started', reference=OwnerReference(kind='assessment', locator='/authentic/report'))
            await observer.update()
            self.assertEqual(observer.recovery()['references'][0]['locator'], '/authentic/report')
            self.assertIn('observation only', observer.recovery()['settlement'])

    async def test_actual_mcp_progress_precedes_return_and_survives_invoker(self) -> None:
        for via_invoker in (False, True):
            with self.subTest(via_invoker=via_invoker):
                observed = asyncio.Event()
                release = asyncio.Event()
                messages = []
                class Child:
                    returncode = None
                    async def wait(self) -> int:
                        await release.wait()
                        self.returncode = 0
                        return 0
                    def terminate(self) -> None:
                        release.set()
                async def spawn(*command: str, **_options: object) -> Child:
                    source, destination = Path(command[-2]), Path(command[-1])
                    reporter = ProgressReporter(source.parent / 'progress.json', 'assessment')
                    reporter.publish('assessment', 'started', reference=OwnerReference(kind='assessment', locator='/issued/native-report'))
                    destination.write_text(json.dumps({'result': {'owner': 'original', 'complete': False}}))
                    return Child()
                server = FastMCP('progress-transport-control')
                @server.tool
                async def selected_action() -> dict:
                    return await process.run_worker(ROOT, WorkerCall('qualification', 'assessment'), {})
                @server.tool
                async def invoke() -> ToolResult:
                    return await server.call_tool('selected_action', {})
                async def progress_handler(progress: float, total: float | None, message: str | None) -> None:
                    messages.append((progress, total, json.loads(message or '{}')))
                    observed.set()
                with patch.object(asyncio, 'create_subprocess_exec', side_effect=spawn):
                    async with Client(server) as client:
                        request = asyncio.create_task(client.call_tool('invoke' if via_invoker else 'selected_action', {}, progress_handler=progress_handler))
                        try:
                            await asyncio.wait_for(observed.wait(), 3)
                            self.assertFalse(request.done())
                            self.assertIsNone(messages[0][1])
                            self.assertEqual(messages[0][2]['references'][0]['locator'], '/issued/native-report')
                        finally:
                            release.set()
                        result = await request
                        self.assertFalse(result.is_error)

    async def test_notification_failure_keeps_native_response_and_final_snapshot(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'progress.json'
            reporter = ProgressReporter(path, 'assessment')
            class LostContext:
                async def report_progress(self, **_arguments: object) -> None:
                    raise OSError('closed observer')
            observer = ProgressObserver(path, 'assessment')
            observer.context = LostContext()
            reporter.publish('assessment', 'started', reference=OwnerReference(kind='assessment', locator='/issued/original'))
            await observer.update()
            reporter.publish('request', 'completed')
            task = asyncio.create_task(asyncio.sleep(0, result=0))
            await task
            await observer.until(task)
            self.assertEqual(observer.sequence, 2)
            self.assertEqual(observer.recovery()['references'][0]['locator'], '/issued/original')
            self.assertEqual(observer.omissions, ['progress notification unavailable'])
