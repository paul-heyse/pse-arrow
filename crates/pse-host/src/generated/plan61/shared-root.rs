/// Crate-wide reflection descriptor pool, embedded once for the whole
/// generated module tree. Every package's `__buffa::reflect` surface
/// re-exports and delegates here, so a multi-package run carries one
/// copy of the `FileDescriptorSet` instead of one per package.
pub mod __buffa_fds {
    /// The serialized `FileDescriptorSet` for this codegen run,
    /// including transitive dependencies, with `source_code_info`
    /// stripped. The single embedded copy for the generated tree.
    pub const FILE_DESCRIPTOR_SET_BYTES: &[u8] = include_bytes!(
        "descriptor-runtime.bin"
    );
    #[allow(clippy::expect_used, reason = "Pinned generator asserts only its immutable embedded descriptor/type registration and self-encoding invariants; acquired input decoding remains fallible")]
    /// The one lazily-built descriptor pool for the whole tree, built
    /// from [`FILE_DESCRIPTOR_SET_BYTES`] on first access.
    ///
    /// The element-memory bound scales with the embedded descriptor
    /// length so large schema trees can exceed the untrusted-input
    /// default. The default remains a floor because smaller descriptor
    /// sets can still have an element-to-encoded ratio above the scale
    /// factor.
    ///
    /// # Panics
    ///
    /// Panics on first access if the embedded bytes are malformed —
    /// they're emitted by `buffa-codegen` from the same descriptors it
    /// generated this code from, so a panic indicates a codegen bug,
    /// not consumer input.
    pub fn descriptor_pool() -> &'static ::buffa::alloc::sync::Arc<
        ::buffa_descriptor::DescriptorPool,
    > {
        static POOL: ::std::sync::OnceLock<
            ::buffa::alloc::sync::Arc<::buffa_descriptor::DescriptorPool>,
        > = ::std::sync::OnceLock::new();
        POOL.get_or_init(|| {
            let options = ::buffa::DecodeOptions::new()
                .with_element_memory_limit(
                    FILE_DESCRIPTOR_SET_BYTES
                        .len()
                        .saturating_mul(64)
                        .max(::buffa::DEFAULT_ELEMENT_MEMORY_LIMIT),
                );
            ::buffa::alloc::sync::Arc::new(
                ::buffa_descriptor::DescriptorPool::decode_with_options(
                        FILE_DESCRIPTOR_SET_BYTES,
                        &options,
                    )
                    .expect("buffa-codegen emitted a decodable FileDescriptorSet"),
            )
        })
    }
}
