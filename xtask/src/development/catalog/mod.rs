// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Independent development contributions to the existing registry and generator.
pub mod evidence;
pub mod knowledge;

/// Declare private development meanings; scientific registry declarations remain separate.
pub fn declare(builder: &mut pse_schema::RegistryBuilder) {
    evidence::declare(builder);
    knowledge::declare(builder);
}
/// Assemble only this responsibility's declarations.
/// # Errors
/// A malformed or unresolved private declaration.
pub fn assemble() -> Result<pse_schema::Registry, pse_schema::SchemaError> {
    let mut builder = pse_schema::RegistryBuilder::new();
    declare(&mut builder);
    builder.build()
}

#[cfg(test)]
mod tests {
    #[test]
    fn private_catalogue_never_enters_product_declarations() {
        let private = super::assemble().unwrap();
        let product = pse_schema::catalog::assemble().unwrap();
        for relation in private.relations() {
            assert!(product.relation(&relation.key.qualified_name()).is_none());
        }
        for vocabulary in private.enums() {
            assert!(product.enum_spec(vocabulary.name).is_none());
        }
    }

    #[test]
    fn private_vocabulary_change_is_confined_to_its_generated_owner() {
        use pse_codegen::codegen::{Language, family, generate};
        let product = pse_schema::catalog::assemble().unwrap();
        let before = generate(&product, Language::Rust).unwrap();
        let private = super::assemble().unwrap();
        let original = family::development_rust(&private).unwrap();
        let mut builder = pse_schema::RegistryBuilder::new();
        super::declare(&mut builder);
        pse_schema::declaration::enumeration(&mut builder, "PrivateOnlyMarker", ["first", "second"]);
        let changed = family::development_rust(&builder.build().unwrap()).unwrap();
        assert_ne!(original.files, changed.files);
        assert!(changed.files.keys().all(|path| path.starts_with("xtask/src/generated")));
        assert_eq!(before, generate(&product, Language::Rust).unwrap());
    }
}
