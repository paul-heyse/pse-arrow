// PROBE bootstrap stub (CBO-040 P1): replaces the stale baked literal so the generator can build.
pub(crate) fn hydrate() -> Result<crate::Registry, crate::SchemaError> {
    Err(crate::checks::invalid(
        "standard registry products",
        "bootstrap stub: run just codegen",
    ))
}
