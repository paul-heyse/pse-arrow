# FastMCP

Realizations considered: installed `fastmcp` 4.1.3, a direct pin in the `quality` group.
Research date 2026-10-27. Sources:
- Context7 `/prefecthq/fastmcp/v4.1.3`: `docs/cli/generate-cli.mdx` and
  `docs/python-sdk/fastmcp-cli-generate.mdx` (`generate_cli_command`).
- The local capability CLI generator's module docstring, which says its "narrow mapping
  follows FastMCP4.1.3's schema-to-Cyclopts approach. Presence, selection and complete
  results deliberately belong to the independent shared runtime."

## Types

### fastmcp/generate-cli-from-tool-schemas
- **Need it serves:** Give every tool of an MCP server a typed command-line subcommand
  derived from the tool's input schema, so the MCP and CLI surfaces share one declaration.
- **Recognizable code shapes:**
  - A hand-written or generated argv layer that mirrors an MCP server's tool parameters.
  - Separate CLI and MCP option vocabularies for the same operations.
- **Mechanisms and APIs:**
  - `fastmcp generate-cli <server-spec> [output.py] [-f] [--no-skill]` connects to a
    server (URL, Python file, MCPConfig or discovered name) and reads tools, resources and
    prompts.
  - It writes a standalone cyclopts script in which each tool is a subcommand with typed
    flags, help and completion.
  - It also writes an agent skill file unless `--no-skill` is given.
- **Obligations discharged:** The schema-to-flags mapping and the CLI scaffolding.
- **Residual caller obligations:** The generated CLI calls tools through an MCP client
  connection. Execution semantics are the server's.
- **Semantic conditions:**
  - Invocation goes through the MCP protocol. Behaviour equals the server's tool behaviour,
    including its transport and lifecycle.
  - The output is generated once, so regeneration is needed when tools change.
- **Version / build conditions:** Present at 4.1.3 per the versioned docs.
- **Limits and negative results:** No path that calls tool functions in-process without the
  MCP client. This is per the documentation description and was not verified in source.
- **Evidence:** Interface-checked (versioned Context7 docs).

## Current applicability

### [CBO-052](../../../library_leverage/codebase/developer-tooling.md#cbo-052) — Several command surfaces without a shared declaration
- **Types:** `fastmcp/generate-cli-from-tool-schemas`.
- **Realization:** Installed 4.1.3.
- **Conclusion:** Ruled out as a replacement for the local capability CLI generator. That
  generator deliberately keeps presence, selection and complete results in an independent
  shared runtime instead of routing through an MCP client. The upstream generator's
  MCP-client invocation contradicts that. The local generator already reuses the upstream
  mapping approach. The MCP-to-CLI pair is therefore already single-declaration, and the
  fragmentation sits elsewhere: justfile and argparse scripts.
- **What it would discharge / what remains local:** n/a.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** Invocation would bypass the
  shared runtime.
- **Unsettled premises and what would settle them:** None.
- **Evidence:** Interface-checked (docs; local docstring); application Proposed.
