# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""The `tk` command: one subcommand per pipeline stage, and the `db` group."""

from __future__ import annotations

from collections.abc import Callable

import psycopg
import typer

from thermo_knowledge import config, db
from thermo_knowledge.acquire.command import acquire_command
from thermo_knowledge.build.command import BUILD_HELP, build_command, status_lines
from thermo_knowledge.generate.command import generate_command
from thermo_knowledge.mapping.command import map_command
from thermo_knowledge.qualify.command import qualify_command
from thermo_knowledge.resolve.command import resolve_command
from thermo_knowledge.staging.command import load_src_command, read_command
from thermo_knowledge.verify.command import VERIFY_HELP, verify_command

app = typer.Typer(
    name="tk",
    help="Thermodynamic knowledge base pipeline (Plan 24).",
    no_args_is_help=True,
    add_completion=False,
)
db_app = typer.Typer(help="The pse_thermo database.", no_args_is_help=True)
app.add_typer(db_app, name="db")

STAGES: dict[str, str] = {
    "acquire": "Acquire declared sources into the raw store and record sources.lock.",
    "read": "Read raw sources into source-faithful staged Parquet.",
    "load-src": "Load staged Parquet into the src_<id> schemas.",
    "resolve": "Resolve identity assertions.",
    "map": "Map source-faithful tables to canonical Parquet.",
    "build": "Build the database from the generated DDL and canonical Parquet.",
    "verify": "Check constraints, invariants and cross-source agreement.",
    "qualify": "Qualify forms against oracle harnesses.",
    "project": "Project the database to .pse, snapshots and reports.",
    "generate": "Generate DDL, meta rows, loader types and docs from the declaration.",
}

_PASS_THROUGH = {"allow_extra_args": True, "ignore_unknown_options": True}


def _stub(stage: str) -> Callable[[], None]:
    def run() -> None:
        typer.echo(f"tk {stage}: not implemented in this packet (TK0a)", err=True)
        raise typer.Exit(code=1)

    run.__name__ = stage.replace("-", "_")
    return run


IMPLEMENTED: set[str] = {
    "acquire",
    "build",
    "generate",
    "read",
    "load-src",
    "resolve",
    "map",
    "verify",
    "qualify",
}
"""Stages with a real command, registered below; the stub loop skips them."""

for _stage, _help in STAGES.items():
    if _stage not in IMPLEMENTED:
        app.command(name=_stage, help=_help, context_settings=_PASS_THROUGH)(_stub(_stage))

app.command(name="acquire", help=STAGES["acquire"])(acquire_command)
app.command(name="read", help=STAGES["read"])(read_command)
app.command(name="load-src", help=STAGES["load-src"])(load_src_command)
app.command(name="resolve", help=STAGES["resolve"])(resolve_command)
app.command(name="map", help=STAGES["map"])(map_command)
app.command(name="build", help=BUILD_HELP)(build_command)
app.command(name="verify", help=VERIFY_HELP)(verify_command)
app.command(name="qualify", help=STAGES["qualify"])(qualify_command)
app.command(name="generate", help=STAGES["generate"])(generate_command)


def _fail(message: str) -> typer.Exit:
    typer.echo(f"error: {message}", err=True)
    return typer.Exit(code=1)


@db_app.command("url")
def db_url() -> None:
    """Print the effective database URL."""
    try:
        typer.echo(config.database_url())
    except config.ConfigError as error:
        raise _fail(str(error)) from error


@db_app.command("bootstrap")
def db_bootstrap() -> None:
    """Create the database if it does not exist, owned by the current role (idempotent)."""
    try:
        url = config.database_url()
        created = db.create_database(url)
    except config.ConfigError as error:
        raise _fail(str(error)) from error
    except psycopg.Error as error:
        raise _fail(f"cannot bootstrap the database: {error}") from error
    name = config.database_name(url)
    typer.echo(f"{'created' if created else 'exists'}: {name}")


@db_app.command("status")
def db_status() -> None:
    """Print server version, database, current user and non-system schemas."""
    try:
        info = db.server_info()
    except config.ConfigError as error:
        raise _fail(str(error)) from error
    except psycopg.Error as error:
        raise _fail(f"cannot reach the database: {error}") from error
    typer.echo(f"server:   PostgreSQL {info.version}")
    typer.echo(f"database: {info.database}")
    typer.echo(f"user:     {info.user}")
    typer.echo(f"schemas:  {', '.join(info.schemas) if info.schemas else '(none)'}")
    try:
        for line in status_lines():
            typer.echo(line)
    except psycopg.Error as error:
        raise _fail(f"cannot read the build state: {error}") from error


if __name__ == "__main__":
    app()
