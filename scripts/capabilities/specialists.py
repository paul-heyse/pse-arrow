# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Closed specialist operations with endpoint-owned, demand-acquired transports.

No construction or catalogue operation starts or enumerates a specialist. Each
connection context lives in its own task; requests may come from different MCP
sessions without transferring ownership of an AnyIO cancel scope across tasks.
"""

from __future__ import annotations

import asyncio
import contextlib
import hashlib
import json
import os
import shutil
import time
from contextlib import asynccontextmanager
from pathlib import Path
from threading import RLock
from typing import TYPE_CHECKING, Annotated, Generic, Literal, TypeVar
from urllib.parse import unquote, urlsplit

from fastmcp import Client
from fastmcp.client.transports import StreamableHttpTransport
from fastmcp.exceptions import ToolError
from fastmcp.tools import Tool
from mcp.types import ToolAnnotations
from pydantic import Field, JsonValue

from scripts import pse_env, semantic_edits, semantic_navigation, surreal_server
from scripts.capabilities.model import Capability, Contract
from scripts.capabilities.responses import ReadResult, bounded

if TYPE_CHECKING:
    from collections.abc import AsyncGenerator, Awaitable, Callable, Coroutine
    from contextlib import AbstractAsyncContextManager

    from mcp.types import Tool as MCPTool

    from scripts.surreal_server import SelectedService

    ClientTools = dict[str, MCPTool]
    FileStamp = tuple[str, int, int, int, int]
    LanguageIdentity = tuple[
        tuple[str, int, int], FileStamp, FileStamp, str, FileStamp | None
    ]

T = TypeVar("T")

Language = Literal["python", "rust"]
Position = Annotated[int, Field(ge=1, le=4294967295)]
Digest = Annotated[str, Field(pattern=r"^[a-f0-9]{64}$")]
Owned = Annotated[list[str], Field(min_length=1, max_length=100)]
PreviewOffset = Annotated[int, Field(ge=0)]
PREVIEW_TEXT_CHARACTERS = 4096
PREVIEW_SUMMARY_FILES = 8
PreviewLimit = Annotated[int, Field(ge=1, le=PREVIEW_TEXT_CHARACTERS)]
MAX_PREVIEWS = 32
MAX_PREVIEW_BYTES = 64 << 20
PREVIEW_TEXT_BYTES = 12 << 10
Kind = Literal[
    "namespaces",
    "nodes",
    "databases",
    "tables",
    "functions",
    "analyzers",
    "params",
    "apis",
    "buckets",
    "models",
    "modules",
    "sequences",
    "configs",
    "users",
    "accesses",
    "fields",
    "indexes",
    "events",
]
ActionKind = Literal[
    "quickfix",
    "refactor",
    "refactor.extract",
    "refactor.inline",
    "refactor.move",
    "refactor.rewrite",
    "source",
    "source.organizeimports",
    "source.fixall",
]


class Coordinate(Contract):
    line: Position
    character: Position


class Range(Contract):
    start: Coordinate
    end: Coordinate


class HierarchyItem(Contract):
    name: str
    kind: Annotated[int, Field(ge=0, le=4294967295)]
    uri: str
    range: Range
    selection_range: Range = Field(alias="selectionRange")
    detail: str | None = None
    data: JsonValue = None
    out_of_workspace: bool = False


def refusal(operation: str, error: Exception) -> ToolError:
    remedy = (
        "Use an exact retained preview digest from this endpoint. A closed, foreign or invalid handle requires a new reviewed preview; no apply or replay is inferred."
        if operation.startswith("edit_preview_")
        else "Inspect the selected provider/source identity and operation schema. Correct input or explicitly prepare the provider; no retry or effects are inferred."
    )
    return ToolError(
        json.dumps(
            {
                "code": "pse.specialist.refused",
                "operation": operation,
                "cause": type(error).__name__,
                "stage": "specialist_selection",
                "message": "The selected specialist request was refused; supplied values are redacted.",
                "remedy": remedy,
            }
        )
    )


class PreviewCapacityError(ValueError):
    """Retention refused without evicting any previously issued handle."""


class Connection(Generic[T]):
    """One owned context task, shared safely by multiple sequential requests."""

    def __init__(self, factory: Callable[[], AbstractAsyncContextManager[Client]]):
        self.factory = factory
        self.queue: asyncio.Queue[
            tuple[
                Callable[[Client, ClientTools], Coroutine[object, object, T]],
                asyncio.Future[T],
            ]
        ] = asyncio.Queue()
        self.task: asyncio.Task[None] | None = None
        self.closed = False

    async def invoke(
        self, action: Callable[[Client, ClientTools], Coroutine[object, object, T]]
    ) -> T:
        if self.closed:
            raise ValueError("specialist connection owner is closed")
        future: asyncio.Future[T] = asyncio.get_running_loop().create_future()
        self.queue.put_nowait((action, future))
        if self.task is None:
            self.task = asyncio.create_task(self._serve(), name="pse-specialist-owner")
        return await future

    async def _serve(self) -> None:
        active: asyncio.Task[T] | None = None
        current: asyncio.Future[T] | None = None
        try:
            async with self.factory() as client:
                # Actual backend schemas are acquired only after an operation selects it.
                tools = {tool.name: tool for tool in await client.list_tools()}
                while True:
                    action, current = await self.queue.get()
                    if current.cancelled():
                        continue
                    active = asyncio.create_task(action(client, tools))

                    def cancel_request(
                        future: asyncio.Future[T], request: asyncio.Task[T] = active
                    ) -> None:
                        if future.cancelled():
                            request.cancel()

                    current.add_done_callback(cancel_request)
                    try:
                        value = await active
                    except asyncio.CancelledError:
                        if not current.cancelled():
                            raise
                    except Exception as error:
                        if not current.done():
                            current.set_exception(error)
                    else:
                        if not current.done():
                            current.set_result(value)
                    finally:
                        current.remove_done_callback(cancel_request)
                        active = None
        except asyncio.CancelledError:
            raise
        except Exception as error:
            if current is not None and not current.done():
                current.set_exception(error)
            while not self.queue.empty():
                _, future = self.queue.get_nowait()
                if not future.done():
                    future.set_exception(error)
        finally:
            if active is not None:
                active.cancel()
                with contextlib.suppress(asyncio.CancelledError):
                    await active
            if current is not None and not current.done():
                current.cancel()
            while not self.queue.empty():
                _, future = self.queue.get_nowait()
                if not future.done():
                    future.cancel()
            self.closed = True

    async def close(self) -> None:
        self.closed = True
        if self.task is not None:
            self.task.cancel()
            with contextlib.suppress(asyncio.CancelledError):
                await self.task


def _tool(tools: ClientTools, name: str, arguments: dict) -> None:
    """Fail before dispatch if the selected backend cannot accept this closed shape.

    The backend remains the final schema validator/decoder; the host never builds
    a generic name/method forwarding surface or substitutes SQL for another tool.
    """
    if name not in tools:
        raise ValueError("selected backend does not expose operation " + name)
    schema = tools[name].input_schema
    fields = schema.get("properties", {})
    if set(arguments) - set(fields) or set(schema.get("required", ())) - set(arguments):
        raise ValueError("selected backend operation schema changed; reload adapter")


def _stamp(path: Path) -> FileStamp:
    info = path.stat()
    return str(path.resolve()), info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns


class Specialists:
    """Lifetime owner for one canonical endpoint and its exact selected checkout."""

    def __init__(
        self,
        root: Path,
        *,
        fixture: bool = False,
        preview_capacity: int = MAX_PREVIEWS,
        preview_byte_capacity: int = MAX_PREVIEW_BYTES,
    ):
        self.root = root.resolve(strict=True)
        self.fixture = fixture
        self.languages: dict[str, Connection[dict]] = {}
        self.language_identities: dict[str, LanguageIdentity] = {}
        self.database: Connection[dict] | None = None
        self.selected: SelectedService | None = None
        self.closed = False
        if preview_capacity < 1 or preview_byte_capacity < 1:
            raise ValueError("preview retention capacities must be positive")
        self._preview_capacity = preview_capacity
        self._preview_byte_capacity = preview_byte_capacity
        self._previews: dict[str, bytes] = {}
        self._preview_bytes = 0
        self._preview_lock = RLock()

    def retain_preview(self, preview: dict) -> dict:
        """Keep exact immutable bytes until endpoint close; never evict issued handles."""
        if semantic_edits.validate(preview).resolve() != self.root:
            raise ValueError("preview belongs to another checkout")
        # Match semantic_edits.reconcile's preview_sha256, including its JSON encoding.
        original = json.dumps(preview, sort_keys=True).encode("utf-8")
        digest = hashlib.sha256(original).hexdigest()
        with self._preview_lock:
            if self.closed:
                raise ValueError(
                    "edit preview owner is closed; handles expire with the endpoint"
                )
            if digest not in self._previews:
                if (
                    len(self._previews) >= self._preview_capacity
                    or self._preview_bytes + len(original) > self._preview_byte_capacity
                ):
                    raise PreviewCapacityError(
                        "edit preview retention capacity exhausted; no existing handle was released: "
                        f"{len(self._previews)}/{self._preview_capacity} previews, "
                        f"{self._preview_bytes}/{self._preview_byte_capacity} bytes retained; "
                        f"new preview requires {len(original)} bytes"
                    )
                self._previews[digest] = original
                self._preview_bytes += len(original)
        return self.preview_summary(digest, preview)

    def retained_preview(self, preview_sha256: str) -> dict:
        """Resolve only this endpoint's exact handle, independently of response projection."""
        with self._preview_lock:
            if self.closed:
                raise ValueError(
                    "edit preview owner is closed; handles expire with the endpoint"
                )
            original = self._previews.get(preview_sha256)
            if original is None:
                raise ValueError(
                    "unknown edit preview digest for this endpoint; recreate and review it here"
                )
            if hashlib.sha256(original).hexdigest() != preview_sha256:
                raise ValueError("retained edit preview digest mismatch")
        preview = json.loads(original)
        if semantic_edits.validate(preview).resolve() != self.root:
            raise ValueError("preview belongs to another checkout")
        return preview

    @staticmethod
    def file_summary(file: dict, index: int) -> dict:
        return {
            "file_index": index,
            "path": file["path"],
            "before_sha256": file["before_sha256"],
            "after_sha256": file["after_sha256"],
            "before_characters": len(file["before"]),
            "after_characters": len(file["after"]),
            "edit_count": len(file["edits"]),
        }

    def preview_summary(self, preview_sha256: str, preview: dict) -> dict:
        return {
            "preview_sha256": preview_sha256,
            "root": str(self.root),
            "producer": preview["producer"],
            "position_encoding": preview["position_encoding"],
            "file_count": len(preview["files"]),
            "basis_count": len(preview["basis"]),
            "edit_count": sum(len(file["edits"]) for file in preview["files"]),
            "files": [
                self.file_summary(file, index)
                for index, file in enumerate(preview["files"][:PREVIEW_SUMMARY_FILES])
            ],
            "next_file_index": PREVIEW_SUMMARY_FILES
            if len(preview["files"]) > PREVIEW_SUMMARY_FILES
            else None,
            "detail_operation": "edit_preview_read",
            "lifetime": "Exact handle retained by this endpoint until it closes; reconnecting creates a new owner.",
        }

    def read_preview(
        self,
        preview_sha256: str,
        file_index: int | None,
        side: str,
        offset: int,
        limit: int,
    ) -> dict:
        preview = self.retained_preview(preview_sha256)
        if file_index is None:
            if offset:
                raise ValueError(
                    "a text offset requires selecting a preview file_index"
                )
            return self.preview_summary(preview_sha256, preview)
        if not 0 <= file_index < len(preview["files"]):
            raise ValueError("preview file_index is outside the retained file set")
        if (
            side not in {"before", "after"}
            or not 1 <= limit <= PREVIEW_TEXT_CHARACTERS
            or offset < 0
        ):
            raise ValueError("select a bounded before/after preview text window")
        file = preview["files"][file_index]
        text = file[side]
        if offset > len(text):
            raise ValueError("preview text offset is beyond the selected source")
        end = offset
        remaining = PREVIEW_TEXT_BYTES
        for character in text[offset : offset + limit]:
            size = len(json.dumps(character, ensure_ascii=True).encode("utf-8")) - 2
            if size > remaining:
                break
            remaining -= size
            end += 1
        return {
            "preview_sha256": preview_sha256,
            **self.file_summary(file, file_index),
            "side": side,
            "offset_encoding": "Unicode codepoints",
            "offset": offset,
            "end_offset": end,
            "total_characters": len(text),
            "text": text[offset:end],
            "next_offset": end if end < len(text) else None,
        }

    def _language_identity(self, language: Language) -> LanguageIdentity:
        if language not in os.environ.get(
            "PSE_CAPABILITIES_LANGUAGES", "python,rust"
        ).split(","):
            raise ValueError("language disabled by the selected session")
        bridge = semantic_navigation.diagnostic_trace.binary_path("mcpls")
        configured = semantic_navigation.configuration(
            self.root, language, fixture=self.fixture
        )
        server = (
            semantic_navigation.ROOT if self.fixture else self.root
        ) / ".venv/bin/pyrefly"
        if language == "rust":
            executable = shutil.which("rust-analyzer")
            if executable is None:
                raise ValueError("pinned rust-analyzer is unavailable")
            server = Path(executable)
        information = self.root.stat()
        return (
            (str(self.root), information.st_dev, information.st_ino),
            _stamp(bridge),
            _stamp(server),
            configured,
            _stamp(self.root / "rust-toolchain.toml") if language == "rust" else None,
        )

    async def language(
        self,
        language: Language,
        name: str,
        arguments: dict,
        *,
        expected: str | None = None,
    ) -> dict:
        if self.closed:
            raise ValueError("specialist lifetime owner is closed")
        identity = self._language_identity(language)
        if (
            language in self.language_identities
            and self.language_identities[language] != identity
        ):
            raise ValueError(
                "selected language server identity/configuration changed; restart endpoint"
            )
        self.language_identities.setdefault(language, identity)
        if language not in self.languages:
            self.languages[language] = Connection(
                lambda: semantic_navigation.connection(
                    self.root, language, fixture=self.fixture
                )
            )
        file_path = arguments.get("file_path")
        source = None
        before = None
        if file_path is not None:
            # Navigation can inspect generated source; edit protection stays in its owner.
            source = self.source(file_path)
            arguments = {**arguments, "file_path": str(source)}
            before = hashlib.sha256(source.read_bytes()).hexdigest()
            if expected is not None and before != expected:
                raise ValueError("source version differs from the requested source cut")

        async def action(client: Client, tools: ClientTools) -> dict:
            if self._language_identity(language) != identity:
                raise ValueError(
                    "selected language server identity changed before request"
                )
            if (
                source is not None
                and hashlib.sha256(source.read_bytes()).hexdigest() != before
            ):
                raise ValueError("source changed before request")
            _tool(tools, name, arguments)
            result = (
                await semantic_navigation.perform(
                    client, [{"name": name, "arguments": arguments}]
                )
            )[0]
            after = (
                None
                if source is None
                else hashlib.sha256(source.read_bytes()).hexdigest()
            )
            return {
                "language": language,
                "checkout": str(self.root),
                "position_encoding": "one-based UTF-16",
                "source_sha256": before,
                "source_unchanged": before == after,
                "complete": before == after,
                "native": result,
            }

        return await self.languages[language].invoke(action)

    def transport_failure(self, operation: str, error: Exception) -> ToolError:
        return ToolError(
            json.dumps(
                {
                    "code": "pse.specialist.transport_failed",
                    "operation": operation,
                    "cause": type(error).__name__,
                    "stage": "specialist_transport",
                    "message": "Specialist transport failed; supplied values and credentials are redacted.",
                    "remedy": "Inspect the selected provider's connection and original operation state before reconnecting or repeating work. Transport failure proves no rollback.",
                }
            )
        )

    def source(self, name: str) -> Path:
        uri = urlsplit(name)
        if uri.scheme:
            if (
                uri.scheme != "file"
                or uri.netloc not in {"", "localhost"}
                or uri.query
                or uri.fragment
            ):
                raise ValueError("navigation requires a local checkout source URI")
            path = Path(unquote(uri.path))
        else:
            path = Path(name)
        if not path.is_absolute():
            path = self.root / path
        if ".." in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
            raise ValueError("source path traverses a symlink or parent component")
        path.relative_to(self.root)
        if not path.is_file() or path.stat().st_size > 10 << 20:
            raise ValueError("source must be an existing bounded checkout file")
        return path

    async def select_database(self) -> dict:
        if self.closed:
            raise ValueError("specialist lifetime owner is closed")
        if os.environ.get("PSE_CAPABILITIES_DATABASE", "1") == "0":
            raise ValueError("database disabled by the selected session")
        if self.selected is None:
            state = os.environ.get("PSE_CAPABILITIES_DATABASE_STATE")
            if state is None:
                state = pse_env.base_environment(self.root, os.environ)[
                    "PSE_SURREAL_STATE"
                ]
            selected = await asyncio.to_thread(
                surreal_server.selected_service, Path(state), time.monotonic() + 8
            )
            # Concurrent explicit selections must not install competing generations.
            if self.selected is None:
                self.selected = selected
        await asyncio.to_thread(self.selected.recheck, time.monotonic() + 8)
        return {
            "identity": self.selected.identity,
            "selection": self.selected.selection_fence,
            "authority": "existing root VIEWER; canonical project operations remain native",
        }

    async def database_call(self, selection: str, name: str, arguments: dict) -> dict:
        if self.closed or self.selected is None:
            raise ValueError(
                "explicit database_select is required before database operations"
            )
        selected = self.selected
        if selection != selected.selection_fence:
            raise ValueError("selected database generation fence differs")

        async def recheck() -> None:
            await asyncio.to_thread(selected.recheck, time.monotonic() + 8)

        await recheck()
        if self.database is None:

            @asynccontextmanager
            async def connect() -> AsyncGenerator[Client]:
                await recheck()
                transport = StreamableHttpTransport(
                    f"http://127.0.0.1:{selected.port}/mcp",
                    headers=selected.authorization(),
                )
                try:
                    async with Client(
                        transport, timeout=100, init_timeout=30
                    ) as client:
                        await recheck()
                        yield client
                finally:
                    await transport.close()

            self.database = Connection(connect)

        async def action(client: Client, tools: ClientTools) -> dict:
            await recheck()
            _tool(tools, name, arguments)
            native = (
                await semantic_navigation.perform(
                    client, [{"name": name, "arguments": arguments}]
                )
            )[0]
            # Observing a changed generation cannot turn an executed call into a success.
            try:
                await recheck()
                unchanged = True
            except surreal_server.SupervisorError:
                unchanged = False
            return {
                "selection": selection,
                "identity": selected.identity,
                "selection_unchanged": unchanged,
                "complete": unchanged,
                "native": native,
            }

        return await self.database.invoke(action)

    async def close(self) -> None:
        with self._preview_lock:
            self.closed = True
            self._previews.clear()
            self._preview_bytes = 0
        for connection in (*self.languages.values(), self.database):
            if connection is not None:
                await connection.close()


def bindings(
    root: Path, *, owner: Specialists
) -> tuple[tuple[Capability, ...], dict[str, Tool]]:
    if root.resolve() != owner.root:
        raise ValueError("specialist owner belongs to another checkout")

    async def read(operation: str, request: Awaitable[object]) -> ReadResult:
        try:
            result = await request
        except (ValueError, OSError, surreal_server.SupervisorError) as error:
            raise refusal(operation, error) from error
        except Exception as error:
            raise owner.transport_failure(operation, error) from error
        return bounded(result, operation)

    async def navigation_hover(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Inspect real semantic hover at one-based UTF-16 coordinates; source versions and backend readiness/errors remain explicit."""
        return await read(
            "navigation_hover",
            owner.language(
                language,
                "get_hover",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_definition(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Resolve a definition through the selected language server at one-based UTF-16 coordinates."""
        return await read(
            "navigation_definition",
            owner.language(
                language,
                "get_definition",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_references(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        include_declaration: bool = False,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Find native references at one-based UTF-16 coordinates. Native index coverage and truncation remain explicit; an empty result is not proof of absence."""
        return await read(
            "navigation_references",
            owner.language(
                language,
                "get_references",
                {
                    "file_path": file_path,
                    "line": line,
                    "character": character,
                    "include_declaration": include_declaration,
                },
                expected=source_sha256,
            ),
        )

    async def navigation_implementations(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Resolve native implementations through the selected language server; retain unsupported-operation refusals."""
        return await read(
            "navigation_implementations",
            owner.language(
                language,
                "go_to_implementation",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_call_hierarchy(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Prepare actual call hierarchy items with native opaque data and one-based UTF-16 ranges."""
        return await read(
            "navigation_call_hierarchy",
            owner.language(
                language,
                "prepare_call_hierarchy",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_incoming_calls(
        language: Language, item: HierarchyItem
    ) -> ReadResult:
        """Read incoming calls from an exact prepared native hierarchy item; no arbitrary method dispatch."""
        owner.source(item.uri)
        return await read(
            "navigation_incoming_calls",
            owner.language(
                language,
                "get_incoming_calls",
                {
                    "item": item.model_dump(
                        mode="json", by_alias=True, exclude_unset=True
                    )
                },
            ),
        )

    async def navigation_outgoing_calls(
        language: Language, item: HierarchyItem
    ) -> ReadResult:
        """Read outgoing calls from an exact prepared native hierarchy item, retaining native errors and opaque server data."""
        owner.source(item.uri)
        return await read(
            "navigation_outgoing_calls",
            owner.language(
                language,
                "get_outgoing_calls",
                {
                    "item": item.model_dump(
                        mode="json", by_alias=True, exclude_unset=True
                    )
                },
            ),
        )

    async def navigation_type_definition(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Resolve the native type definition at one-based UTF-16 coordinates."""
        return await read(
            "navigation_type_definition",
            owner.language(
                language,
                "go_to_type_definition",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_declaration(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Resolve the native declaration; unsupported server operations retain their native refusal."""
        return await read(
            "navigation_declaration",
            owner.language(
                language,
                "go_to_declaration",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_completions(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        trigger: str | None = None,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Inspect real native completion candidates without applying their edits or executing commands."""
        arguments = {"file_path": file_path, "line": line, "character": character}
        if trigger is not None:
            arguments["trigger"] = trigger
        return await read(
            "navigation_completions",
            owner.language(
                language, "get_completions", arguments, expected=source_sha256
            ),
        )

    async def navigation_signature(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Read native signature help at one-based UTF-16 coordinates."""
        return await read(
            "navigation_signature",
            owner.language(
                language,
                "get_signature_help",
                {"file_path": file_path, "line": line, "character": character},
                expected=source_sha256,
            ),
        )

    async def navigation_tool_support(language: Language, file_path: str) -> ReadResult:
        """Explicitly acquire the selected language server and inspect its actual supported operations."""
        return await read(
            "navigation_tool_support",
            owner.language(language, "get_tool_support", {"file_path": file_path}),
        )

    async def navigation_workspace_symbols(
        language: Language, query: str, limit: Annotated[int, Field(ge=1, le=100)] = 100
    ) -> ReadResult:
        """Search actual native workspace symbols. Preserve native index coverage/truncation; an empty result is not proof of absence."""
        return await read(
            "navigation_workspace_symbols",
            owner.language(
                language, "workspace_symbol_search", {"query": query, "limit": limit}
            ),
        )

    async def navigation_code_actions(
        language: Language,
        file_path: str,
        start_line: Position,
        start_character: Position,
        end_line: Position,
        end_character: Position,
        kind_filter: ActionKind | None = None,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Inspect native code actions before selecting an action preview. Commands and edits are returned as data and never executed."""
        arguments = {
            "file_path": file_path,
            "start_line": start_line,
            "start_character": start_character,
            "end_line": end_line,
            "end_character": end_character,
        }
        if kind_filter is not None:
            arguments["kind_filter"] = kind_filter
        return await read(
            "navigation_code_actions",
            owner.language(
                language, "get_code_actions", arguments, expected=source_sha256
            ),
        )

    async def navigation_symbols(
        language: Language, file_path: str, source_sha256: Digest | None = None
    ) -> ReadResult:
        """Read actual document symbols; incomplete indexing and degraded positions stay native."""
        return await read(
            "navigation_symbols",
            owner.language(
                language,
                "get_document_symbols",
                {"file_path": file_path},
                expected=source_sha256,
            ),
        )

    async def navigation_diagnostics(
        language: Language, file_path: str, source_sha256: Digest | None = None
    ) -> ReadResult:
        """Request native diagnostics; an unavailable/indexing server is not a clean result."""
        return await read(
            "navigation_diagnostics",
            owner.language(
                language,
                "get_diagnostics",
                {"file_path": file_path},
                expected=source_sha256,
            ),
        )

    def preview_payload(result: dict, action_index: int | None) -> dict:
        if result["native"]["isError"]:
            raise ValueError("native edit request refused; inspect native error")
        if not result["source_unchanged"]:
            raise ValueError("source changed while edit request ran")
        payload = result["native"]["result"]
        semantic_edits.require_complete(payload)
        if action_index is not None:
            action = payload["actions"][action_index]
            if action.get("command"):
                raise ValueError(
                    "command-based code actions are not complete source edits"
                )
            return action["edit"]
        return payload

    async def preview_result(
        language: Language,
        operation: str,
        arguments: dict,
        owned: list[str],
        source_sha256: str | None,
        action_index: int | None = None,
    ) -> dict:
        cut = semantic_edits.capture(owner.root, owned)
        result = await owner.language(
            language, operation, arguments, expected=source_sha256
        )
        result["preview"] = None
        try:
            payload = preview_payload(result, action_index)
            preview = semantic_edits.preview(
                owner.root,
                cut,
                semantic_navigation.edit_result(payload),
                producer="mcpls:" + language,
            )
        except (ValueError, KeyError, IndexError, TypeError):
            result["preview_refusal"] = {
                "code": "pse.specialist.preview_unavailable",
                "stage": "preview_capture",
                "remedy": "Inspect the original server result and current source cut; only a complete attributable edit can produce an apply handle.",
            }
            result["complete"] = False
            return result
        # Retain before projection: omitted response strings can never alter this snapshot.
        # Capacity refusal is explicit and preserves every handle already issued.
        result["preview"] = owner.retain_preview(preview)
        native = dict(result["native"])
        native_payload = dict(native["result"])
        native_payload.pop("changes", None)
        native_payload.pop("actions", None)
        native["result"] = native_payload
        native.pop("content", None)
        native["source_edit_detail"] = (
            "Retained losslessly by preview_sha256; read before/after text through edit_preview_read."
        )
        result["native"] = native
        return result

    async def navigation_rename_preview(
        language: Language,
        file_path: str,
        line: Position,
        character: Position,
        new_name: str,
        owned: Owned,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Retain a protected exact rename preview and return its digest plus summary; never applies. Handles live only until this endpoint closes. Use edit_preview_read for lossless source text. Capture every owned consumer first; generated declarations and PyO3 strings remain outside language rename coverage."""
        return await read(
            "navigation_rename_preview",
            preview_result(
                language,
                "rename_symbol",
                {
                    "file_path": file_path,
                    "line": line,
                    "character": character,
                    "new_name": new_name,
                },
                owned,
                source_sha256,
            ),
        )

    async def navigation_action_preview(
        language: Language,
        file_path: str,
        start_line: Position,
        start_character: Position,
        end_line: Position,
        end_character: Position,
        owned: Owned,
        action_index: Annotated[int, Field(ge=0)],
        kind_filter: ActionKind | None = None,
        source_sha256: Digest | None = None,
    ) -> ReadResult:
        """Retain one complete code-action preview and return its endpoint-lifetime digest plus summary. Use edit_preview_read for lossless source text. Refuse command actions, partial edits, stale versions, protected paths and unknown coordinate encodings."""
        arguments = {
            "file_path": file_path,
            "start_line": start_line,
            "start_character": start_character,
            "end_line": end_line,
            "end_character": end_character,
        }
        if kind_filter is not None:
            arguments["kind_filter"] = kind_filter
        return await read(
            "navigation_action_preview",
            preview_result(
                language,
                "get_code_actions",
                arguments,
                owned,
                source_sha256,
                action_index,
            ),
        )

    def edit_preview_read(
        preview_sha256: Digest,
        file_index: PreviewOffset | None = None,
        side: Literal["before", "after"] = "before",
        offset: PreviewOffset = 0,
        limit: PreviewLimit = PREVIEW_TEXT_CHARACTERS,
    ) -> ReadResult:
        """Read an exact retained preview summary, or lossless before/after text for file_index. Offsets count Unicode codepoints; follow next_offset until null. JSON-byte bounds may shorten a chunk without losing text. Handles belong to this endpoint and expire when it closes; no source read, edit, server acquisition or handle renewal occurs."""
        try:
            return bounded(
                owner.read_preview(preview_sha256, file_index, side, offset, limit),
                "edit_preview_read",
            )
        except (ValueError, OSError) as error:
            raise refusal("edit_preview_read", error) from error

    def edit_preview_apply(preview_sha256: Digest, receipt: str) -> ReadResult:
        """Apply only this endpoint's explicitly reviewed exact retained preview digest. Handles expire when the endpoint closes. Native protection, all-input/stale-basis checks and truthful partial-write receipt reconciliation remain authoritative; no response dictionary is accepted as edit authority."""
        preview = None
        try:
            preview = owner.retained_preview(preview_sha256)
            destination = semantic_edits.owned_path(
                owner.root, receipt, allow_missing=True
            )
            return bounded(
                semantic_edits.apply(preview, destination), "edit_preview_apply"
            )
        except (ValueError, OSError) as error:
            if preview is not None:
                raise ToolError(
                    json.dumps(
                        {
                            "code": "pse.specialist.edit_apply_unsettled",
                            "stage": "edit_apply",
                            "cause": type(error).__name__,
                            "message": "Edit application did not return success. The error alone does not establish whether any source changed.",
                            "recovery": {"preview_sha256": preview_sha256},
                            "remedy": "Call edit_preview_reconcile with this retained preview digest and inspect original source/receipt before deciding further edits. Do not automatically repeat apply.",
                        }
                    )
                ) from error
            raise refusal("edit_preview_apply", error) from error

    def edit_preview_reconcile(preview_sha256: Digest) -> ReadResult:
        """Read actual unchanged/applied/diverged/missing state using this endpoint's exact retained preview digest after an interrupted edit. Handles expire when the endpoint closes. Never writes source or reacquires a server."""
        try:
            return bounded(
                semantic_edits.reconcile(owner.retained_preview(preview_sha256)),
                "edit_preview_reconcile",
            )
        except (ValueError, OSError) as error:
            raise refusal("edit_preview_reconcile", error) from error

    async def database_select() -> ReadResult:
        """Explicitly validate the already selected database generation and return its nonsecret fence. No service startup, credentials output or backend connection."""
        return await read("database_select", owner.select_database())

    async def database_info(
        selection: Digest,
        target: str | None = None,
        namespace: str | None = None,
        database: str | None = None,
    ) -> ReadResult:
        """Use the real selected database info tool. Target is root/ns/db or a table; credentials and generation stay fixed to database_select."""
        return await read(
            "database_info",
            owner.database_call(
                selection,
                "info",
                {"target": target, "namespace": namespace, "database": database},
            ),
        )

    async def database_list(
        selection: Digest,
        kind: Kind,
        table: str | None = None,
        scope: Literal["root", "ns", "db"] | None = None,
        namespace: str | None = None,
        database: str | None = None,
    ) -> ReadResult:
        """Use native database list semantics, including kind/table/scope refusals. No SQL fallback or arbitrary MCP operation forwarding."""
        return await read(
            "database_list",
            owner.database_call(
                selection,
                "list",
                {
                    "kind": kind,
                    "table": table,
                    "scope": scope,
                    "namespace": namespace,
                    "database": database,
                },
            ),
        )

    async def database_query(
        selection: Digest,
        query: str,
        parameters: dict[str, JsonValue] | None = None,
        namespace: str | None = None,
        database: str | None = None,
    ) -> ReadResult:
        """Execute the actual native SurrealQL query toolkit under existing selected VIEWER authority; preserve native parameters, typed values and errors. Canonical project writes require their native project APIs/admission. Query is never used to replace another capability."""
        return await read(
            "database_query",
            owner.database_call(
                selection,
                "query",
                {
                    "query": query,
                    "parameters": parameters,
                    "namespace": namespace,
                    "database": database,
                },
            ),
        )

    rows = [
        (navigation_hover, "editing"),
        (navigation_definition, "editing"),
        (navigation_symbols, "editing"),
        (navigation_diagnostics, "editing"),
        (navigation_references, "editing"),
        (navigation_implementations, "editing"),
        (navigation_call_hierarchy, "editing"),
        (navigation_incoming_calls, "editing"),
        (navigation_outgoing_calls, "editing"),
        (navigation_type_definition, "editing"),
        (navigation_declaration, "editing"),
        (navigation_completions, "editing"),
        (navigation_signature, "editing"),
        (navigation_tool_support, "editing"),
        (navigation_workspace_symbols, "editing"),
        (navigation_code_actions, "editing"),
        (navigation_rename_preview, "editing"),
        (navigation_action_preview, "editing"),
        (edit_preview_read, "editing"),
        (edit_preview_apply, "editing"),
        (edit_preview_reconcile, "editing"),
        (database_select, "integration"),
        (database_info, "integration"),
        (database_list, "integration"),
        (database_query, "integration"),
    ]
    tools: dict[str, Tool] = {}
    specs: list[Capability] = []
    for function, domain in rows:
        effectful = function is edit_preview_apply or function is database_query
        tool = Tool.from_function(
            function,
            annotations=ToolAnnotations(
                read_only_hint=not effectful,
                destructive_hint=effectful,
                idempotent_hint=not effectful,
                open_world_hint=False,
            ),
        )
        source = (
            "scripts/semantic_edits.py"
            if function in (edit_preview_apply, edit_preview_reconcile)
            else "scripts/capabilities/specialists.py"
        )
        spec = Capability(
            id=tool.name,
            title=tool.name.replace("_", " ").title(),
            description=tool.description or tool.name,
            domain=domain,
            family=(
                "controlled-edits"
                if function
                in (
                    edit_preview_read,
                    edit_preview_apply,
                    edit_preview_reconcile,
                    navigation_rename_preview,
                    navigation_action_preview,
                )
                else "language-navigation"
                if domain == "editing"
                else "database-tooling"
            ),
            guides=("root", "tooling"),
            owner=source,
            source=source,
            exposure="tool",
            effects="Selected specialist only; lazy owned transport, exact checkout/service, native authority and source protection.",
        )
        tool.tags.update(spec.tags)
        tools[spec.id] = tool
        specs.append(spec)
    return tuple(specs), tools
