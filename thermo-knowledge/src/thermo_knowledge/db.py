# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""A thin psycopg 3 layer over the configured server: connect, inspect, create, drop."""

from __future__ import annotations

import time
from dataclasses import dataclass

import psycopg
from psycopg import errors, sql

from thermo_knowledge import config


@dataclass(frozen=True)
class ServerInfo:
    """What `tk db status` reports."""

    version: str
    database: str
    user: str
    schemas: tuple[str, ...]


def connect(url: str | None = None, *, autocommit: bool = False) -> psycopg.Connection:
    """Open a connection to `url` (default: the configured database).

    Refuses a database named `pse` whatever the URL's origin.
    """
    target = url if url is not None else config.database_url()
    config.refuse_production(config.database_name(target))
    return psycopg.connect(target, autocommit=autocommit)


def connect_admin(url: str | None = None) -> psycopg.Connection:
    """An autocommit connection to the server's maintenance database, for CREATE and DROP
    DATABASE (neither may run inside a transaction)."""
    target = url if url is not None else config.database_url()
    return psycopg.connect(
        config.with_database(target, config.MAINTENANCE_DATABASE), autocommit=True
    )


def server_info(url: str | None = None) -> ServerInfo:
    """Server version, database, current user and the non-system schemas."""
    with connect(url) as conn:
        row = conn.execute(
            "SELECT current_setting('server_version'), current_database(), current_user"
        ).fetchone()
        if row is None:  # pragma: no cover - SELECT without FROM always returns a row
            raise RuntimeError("the server returned no row for a constant query")
        version, database, user = row
        schemas = conn.execute(
            "SELECT nspname FROM pg_namespace "
            "WHERE nspname NOT LIKE 'pg\\_%' AND nspname <> 'information_schema' "
            "ORDER BY nspname"
        ).fetchall()
    return ServerInfo(version, database, user, tuple(name for (name,) in schemas))


def database_exists(conn: psycopg.Connection, name: str) -> bool:
    """Whether the server has a database called `name`."""
    row = conn.execute("SELECT 1 FROM pg_database WHERE datname = %s", (name,)).fetchone()
    return row is not None


def create_database(url: str | None = None) -> bool:
    """Create the database `url` names, owned by the connecting role.

    Returns whether it was created; an existing database is left alone.
    """
    target = url if url is not None else config.database_url()
    name = config.refuse_production(config.database_name(target))
    with connect_admin(target) as conn:
        if database_exists(conn, name):
            return False
        try:
            # The creating role owns the new database.
            conn.execute(sql.SQL("CREATE DATABASE {}").format(sql.Identifier(name)))
        except errors.DuplicateDatabase:
            return False
        return True


def drop_database(url: str, *, attempts: int = 50, pause: float = 0.1) -> None:
    """Drop the database `url` names, terminating its sessions (`WITH (FORCE)`).

    FORCE refuses to terminate a server background worker (autovacuum) attached to the
    database; such a worker leaves within moments, so that refusal is retried.
    """
    name = config.refuse_production(config.database_name(url))
    statement = sql.SQL("DROP DATABASE IF EXISTS {} WITH (FORCE)").format(sql.Identifier(name))
    with connect_admin(url) as conn:
        for attempt in range(attempts + 1):
            try:
                conn.execute(statement)
                return
            except errors.InsufficientPrivilege:
                if attempt == attempts:
                    raise
                time.sleep(pause)
