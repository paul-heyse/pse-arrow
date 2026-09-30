# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Where the tree, its store and its database are: the one place that declares them.

Nothing here touches the file system or the network on import; helpers compute paths and
strings only.
"""

from __future__ import annotations

import os
from pathlib import Path
from urllib.parse import parse_qsl, quote, unquote, urlencode, urlsplit

TREE_NAME = "thermo-knowledge"
"""Directory name of the tree under the repository root."""

# src/thermo_knowledge/config.py -> src/thermo_knowledge -> src -> tree -> repository root
TREE_DIR = Path(__file__).resolve().parents[2]
REPO_ROOT = TREE_DIR.parent
TREE_RELATIVE = TREE_DIR.relative_to(REPO_ROOT)
"""The tree's location relative to the repository root."""

STORE_ENV = "PSE_THERMO_STORE"
DATABASE_URL_ENV = "PSE_THERMO_DATABASE_URL"
KEEP_FAILED_DATABASES_ENV = "PSE_THERMO_KEEP_FAILED_DATABASES"
"""When set to `1`, a test database whose test failed is kept for inspection."""

DEFAULT_STORE_RELATIVE = TREE_RELATIVE / ".store"
DEFAULT_DATABASE_URL = "postgres:///pse_thermo?host=/var/run/postgresql"
"""Peer authentication over the Unix socket of the server the operational store uses."""

PRODUCTION_DATABASE = "pse"
"""The operational store's database; this tree never operates on it."""

MAINTENANCE_DATABASE = "postgres"
"""The database an administrative connection uses to create and drop others."""

_URL_SCHEMES = frozenset({"postgres", "postgresql"})


class ConfigError(ValueError):
    """The configured store or database cannot be used."""


def store_root() -> Path:
    """The store root: `PSE_THERMO_STORE` (relative values resolve against the repository
    root), else `thermo-knowledge/.store`."""
    override = os.environ.get(STORE_ENV)
    if override:
        path = Path(override).expanduser()
        return path if path.is_absolute() else REPO_ROOT / path
    return REPO_ROOT / DEFAULT_STORE_RELATIVE


def raw_dir() -> Path:
    """Untouched acquired sources."""
    return store_root() / "raw"


def staged_dir() -> Path:
    """Source-faithful Parquet."""
    return store_root() / "staged"


def canonical_dir() -> Path:
    """Canonical Parquet, one directory per table."""
    return store_root() / "canonical"


def database_name(url: str) -> str:
    """The database a PostgreSQL URL names (path, else the `dbname` parameter)."""
    parts = urlsplit(url)
    if parts.scheme not in _URL_SCHEMES:
        raise ConfigError(f"{url!r} is not a postgres:// or postgresql:// URL")
    name = unquote(parts.path.lstrip("/"))
    if not name:
        name = dict(parse_qsl(parts.query)).get("dbname", "")
    if not name:
        raise ConfigError(f"{url!r} names no database")
    return name


def refuse_production(name: str) -> str:
    """Return `name`, or raise when it is the operational store's database."""
    if name == PRODUCTION_DATABASE:
        raise ConfigError(
            f"refusing to operate on database {PRODUCTION_DATABASE!r}: it is the "
            "operational store; the knowledge base lives in pse_thermo"
        )
    return name


def with_database(url: str, name: str) -> str:
    """`url` with its database replaced by `name` (a same-server URL for another database)."""
    database_name(url)
    refuse_production(name)
    parts = urlsplit(url)
    query = urlencode(
        [(key, value) for key, value in parse_qsl(parts.query) if key != "dbname"],
        safe="/",
    )
    # Assembled by hand: urlunsplit drops the empty authority of `postgres:///name`.
    url = f"{parts.scheme}://{parts.netloc}/{quote(name, safe='')}"
    return f"{url}?{query}" if query else url


def database_url() -> str:
    """The effective database URL: `PSE_THERMO_DATABASE_URL`, else the default.

    Raises `ConfigError` for a URL that names no database or names `pse`.
    """
    url = os.environ.get(DATABASE_URL_ENV) or DEFAULT_DATABASE_URL
    refuse_production(database_name(url))
    return url
