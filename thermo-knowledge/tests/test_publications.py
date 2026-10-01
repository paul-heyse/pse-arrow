# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse

"""Publications resolve across carriers: one paper cited under different keys is one publication,
what a carrier says about it is per carrier, and the build accepts the carriers together."""

from __future__ import annotations

from pathlib import Path

import psycopg
import pytest

from build_support import inputs_of, write_source
from mapping_support import real_declaration
from thermo_knowledge.build import build_database
from thermo_knowledge.canonical.writer import (
    CanonicalWriter,
    CompetingAssertion,
    ValidationError,
)
from thermo_knowledge.testing import TestDatabase

DOI = "10.1000/Xyz.Abc-1"


def cites(carrier: str, key: str, **cited: object):  # noqa: ANN201
    def fill(w: CanonicalWriter) -> None:
        w.citation(carrier, key, **cited)  # type: ignore[arg-type]

    return fill


def rows(url: str, query: str) -> list[tuple[object, ...]]:
    with psycopg.connect(url) as conn:
        return conn.execute(query).fetchall()  # type: ignore[arg-type]


@pytest.fixture
def database() -> TestDatabase:
    with TestDatabase() as database:
        yield database  # type: ignore[misc]


def test_two_carriers_citing_one_doi_under_different_keys_yield_one_publication(
    tmp_path: Path, database: TestDatabase
) -> None:
    canonical = tmp_path / "canonical"
    write_source(
        canonical,
        "alpha",
        cites("alpha", "smith2020", doi=DOI, year=2020, citation="Smith, J. (2020). A paper."),
    )
    write_source(
        canonical,
        "beta",
        cites("beta", "Smith:20", doi=DOI.upper(), year=2021, citation="J. Smith, A Paper."),
    )
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert result.tables["prov.publication"] == 1, "one paper is one publication"
    assert result.tables["prov.citation"] == 2, "each carrier's key is its own row"
    ((key, doi),) = rows(
        database.url, "SELECT s.key, p.doi FROM prov.publication p JOIN prov.source s USING (id)"
    )
    assert key == "doi:10.1000/xyz.abc-1" and doi == "10.1000/xyz.abc-1"
    cited = rows(
        database.url,
        "SELECT c.manifest_id, r.local_key, r.year, r.citation, p.doi FROM prov.citation r "
        "JOIN prov.carrier c ON c.id = r.carrier "
        "JOIN prov.publication p ON p.id = r.publication ORDER BY c.manifest_id",
    )
    assert cited == [
        ("alpha", "smith2020", 2020, "Smith, J. (2020). A paper.", "10.1000/xyz.abc-1"),
        ("beta", "Smith:20", 2021, "J. Smith, A Paper.", "10.1000/xyz.abc-1"),
    ]


def test_differing_year_and_citation_text_between_carriers_do_not_refuse_the_build(
    tmp_path: Path, database: TestDatabase
) -> None:
    canonical = tmp_path / "canonical"
    write_source(canonical, "alpha", cites("alpha", "k", doi=DOI, year=1999, citation="one"))
    write_source(canonical, "beta", cites("beta", "k", doi=DOI, year=2001, citation="two"))
    write_source(canonical, "gamma", cites("gamma", "k", doi=DOI))
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert result.tables["prov.publication"] == 1 and result.tables["prov.citation"] == 3
    assert rows(
        database.url, "SELECT year, citation FROM prov.citation ORDER BY year NULLS LAST"
    ) == [
        (1999, "one"),
        (2001, "two"),
        (None, None),
    ]


def test_a_publication_without_a_doi_is_keyed_by_the_carrier_and_its_citation_key(
    tmp_path: Path, database: TestDatabase
) -> None:
    canonical = tmp_path / "canonical"
    write_source(canonical, "alpha", cites("alpha", "handbook", citation="A handbook"))
    write_source(canonical, "beta", cites("beta", "handbook", citation="A handbook"))
    result = build_database(database.url, real_declaration(), inputs_of(canonical))
    assert result.tables["prov.publication"] == 2, "no DOI, so no evidence that they are one work"
    assert rows(
        database.url,
        "SELECT s.key FROM prov.publication p JOIN prov.source s USING (id) ORDER BY 1",
    ) == [
        ("carrier:alpha:handbook",),
        ("carrier:beta:handbook",),
    ]


def test_the_identity_bound_attributes_are_the_key_and_the_doi_and_the_rest_is_per_carrier() -> (
    None
):
    decl = real_declaration()
    assert {a.name for a in decl.attributes_of("publication")} == {"key", "title", "doi", "isbn"}
    relation = decl.relations["citation"]
    assert [k.name for k in relation.keys] == ["carrier", "local_key"]
    assert {c.name for c in relation.values} == {"publication", "year", "citation"}
    (rule,) = decl.kinds["publication"].requires
    assert (rule.name, rule.enforced) == ("key_follows_doi", "load")


def test_the_writer_refuses_a_publication_whose_key_breaks_the_rule() -> None:
    from mapping_support import writer

    w = writer(real_declaration())
    for values, message in (
        ({"key": "smith2020", "title": "t"}, "`carrier:<manifest id>:<citation key>`"),
        ({"key": "doi:10.1/A", "title": "t", "doi": "10.1/A"}, "stated in lower case"),
        ({"key": "doi:10.1/other", "title": "t", "doi": "10.1/a"}, "has the key `doi:10.1/a`"),
        ({"key": "carrier::x", "title": "t"}, "carrier:<manifest id>:<citation key>"),
    ):
        with pytest.raises(ValidationError, match=message):
            w.kind("publication", values)
    w.kind("publication", {"key": "doi:10.1/a", "title": "t", "doi": "10.1/a"})
    w.kind("publication", {"key": "carrier:src:smith", "title": "t"})


def test_a_carrier_that_cites_one_key_twice_differently_is_competing_with_itself() -> None:
    from mapping_support import writer

    w = writer(real_declaration())
    w.citation("src", "k", doi=DOI, year=2000)
    w.citation("src", "k", doi=DOI, year=2000)  # the same statement again is one row
    with pytest.raises(CompetingAssertion):
        w.citation("src", "k", doi=DOI, year=2001)


def test_citations_refuse_what_cannot_be_written() -> None:
    from mapping_support import writer

    w = writer(real_declaration())
    with pytest.raises(ValidationError, match="a citation key is not empty"):
        w.citation("src", " ")
    with pytest.raises(ValidationError, match="a DOI, when stated, is not empty"):
        w.citation("src", "k", doi=" ")
    with pytest.raises(ValidationError, match="the carrier `nobody` is not known to the writer"):
        w.citation("nobody", "k")
    assert w.rows("prov.publication") == 0 and w.rows("prov.citation") == 0
