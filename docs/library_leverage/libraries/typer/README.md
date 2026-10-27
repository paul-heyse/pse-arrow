# Typer

Realizations considered: installed `typer` 0.27.3 and `click` 8.5.0 in `.venv`, as
transitive dependencies. `uv tree --invert` shows `typer` comes through `pydocket` 0.27.0
in the `capability-inquiry` group. No upgrade route was considered.
Research date 2026-10-27. Sources: `.venv` dist-info presence. The Typer model
(`typer.Typer()`, `@app.command()`, `Annotated[T, typer.Option(...)]`, built on Click) is
documentation knowledge.

## Types

### typer/click-based-typed-cli
- **Need it serves:** The same need as other annotation-driven CLI frameworks: typed
  commands from function signatures.
- **Recognizable code shapes:** argparse proliferation.
- **Mechanisms and APIs:**
  - `typer.Typer()` and `@app.command()`.
  - `Annotated[T, typer.Option()/Argument()]`.
  - Click groups underneath, with completion and rich help.
- **Obligations discharged:** Parsing, conversion, help and completion.
- **Residual caller obligations:** As for any CLI framework: semantics, admission and
  positional compatibility.
- **Semantic conditions:** Click's parsing rules apply, including option interspersing and
  `--` handling.
- **Version / build conditions:** It is present only transitively, so relying on it would
  need a direct pin.
- **Limits and negative results:** No lazy import-path command registration was verified.
  Not stdlib.
- **Evidence:** Installed-version presence only. Model is documentation knowledge.

## Current applicability

### [CBO-052](../../../library_leverage/codebase/developer-tooling.md#cbo-052) — Several command surfaces without a shared declaration
- **Types:** `typer/click-based-typed-cli`.
- **Realization:** Installed transitively, at 0.27.3.
- **Conclusion:** Ruled out here in favour of the already directly pinned annotation-driven
  framework. Another framework already pinned as the target of the capability CLI generator
  occupies the same role, and adding a second CLI framework would fragment further
  (LOP-09). This is not a capability judgment against Typer.
- **What it would discharge / what remains local:** n/a.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** n/a.
- **Unsettled premises and what would settle them:** None.
- **Evidence:** Installed presence; application Proposed.
