# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Disposable databases for tests, the counterpart of `pse_operations::testing::TestDatabase`.

Each `TestDatabase` is created on the server the configured URL names, under a name minted
here, so a test never touches `pse_thermo`. A failed test may leave its database for
inspection when `PSE_THERMO_KEEP_FAILED_DATABASES=1`.
"""

from __future__ import annotations

import os
import uuid
from types import TracebackType

from thermo_knowledge import config, db

NAME_PREFIX = "pse_thermo_test_"


class TestDatabase:
    """One uniquely named, empty database on the configured server."""

    __test__ = False  # not a pytest test class

    def __init__(self, server_url: str | None = None) -> None:
        server = server_url if server_url is not None else config.database_url()
        self.name = f"{NAME_PREFIX}{uuid.uuid4().hex}"
        self.url = config.with_database(server, self.name)
        self._created = False

    @classmethod
    def create(cls, server_url: str | None = None) -> TestDatabase:
        """Create the database and return its handle."""
        return cls(server_url).start()

    def start(self) -> TestDatabase:
        """Create the database if it does not exist yet."""
        if not self._created:
            # The name is minted here from a UUID; it carries no caller text.
            db.create_database(self.url)
            self._created = True
        return self

    def remove(self, *, failed: bool = False) -> bool:
        """Drop the database, unless `failed` and the environment asks to keep it.

        Returns whether the database was dropped.
        """
        if not self._created:
            return False
        if failed and os.environ.get(config.KEEP_FAILED_DATABASES_ENV) == "1":
            return False
        db.drop_database(self.url)
        self._created = False
        return True

    def __enter__(self) -> TestDatabase:
        return self.start()

    def __exit__(
        self,
        exc_type: type[BaseException] | None,
        exc: BaseException | None,
        traceback: TracebackType | None,
    ) -> None:
        self.remove(failed=exc_type is not None)
