// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reserve-first parsing, immutable source ownership and direct declaration binding.

use pse_authoring::ParseBudget;
use pse_runtime::authoring_driver::DriverError;
use pse_runtime::authoring_driver::document::{
    OwnedDocumentSet, load_package_sources_owned, load_package_texts, load_package_texts_owned,
};

use pse_columnar::{CancellationToken, CanonError, MemoryPool};
use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn header() -> String {
    include_str!("../../../tests/fixtures/packages/minimal_explicit/package.toml").to_owned()
}
fn sources() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("package.toml".to_owned(), header()),
        (
            "materials/species.yaml".to_owned(),
            include_str!(
                "../../../tests/fixtures/packages/minimal_explicit/materials/species.yaml"
            )
            .to_owned(),
        ),
    ])
}

#[derive(Debug)]
struct ObservedBudget {
    inner: Arc<dyn MemoryPool>,
    peak: Arc<AtomicUsize>,
    cancel_after_grow: Option<CancellationToken>,
}
impl ObservedBudget {
    fn new(bytes: usize) -> Self {
        Self {
            inner: Arc::new(pse_columnar::GreedyMemoryPool::new(bytes)),
            peak: Arc::new(AtomicUsize::new(0)),
            cancel_after_grow: None,
        }
    }
    fn pool(&self) -> Arc<dyn MemoryPool> {
        Arc::new(Self {
            inner: self.inner.clone(),
            peak: self.peak.clone(),
            cancel_after_grow: self.cancel_after_grow.clone(),
        })
    }
    fn peak(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
}
impl std::fmt::Display for ObservedBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObservedBudget")
    }
}
impl MemoryPool for ObservedBudget {
    fn name(&self) -> &'static str {
        "ObservedBudget"
    }
    fn register(&self, c: &pse_columnar::MemoryConsumer) {
        self.inner.register(c);
    }
    fn unregister(&self, c: &pse_columnar::MemoryConsumer) {
        self.inner.unregister(c);
    }
    fn grow(&self, r: &pse_columnar::MemoryReservation, n: usize) {
        self.inner.grow(r, n);
    }
    fn shrink(&self, r: &pse_columnar::MemoryReservation, n: usize) {
        self.inner.shrink(r, n);
    }
    fn try_grow(
        &self,
        r: &pse_columnar::MemoryReservation,
        n: usize,
    ) -> datafusion::common::Result<()> {
        self.inner.try_grow(r, n)?;
        self.peak
            .fetch_max(self.inner.reserved(), Ordering::Relaxed);
        if let Some(cancel) = &self.cancel_after_grow {
            cancel.cancel();
        }
        Ok(())
    }
    fn reserved(&self) -> usize {
        self.inner.reserved()
    }
    fn memory_limit(&self) -> datafusion::execution::memory_pool::MemoryLimit {
        self.inner.memory_limit()
    }
}

#[test]
fn owned_loader_matches_actual_rows_and_spans_and_retains_only_shared_owners() {
    let registry = pse_engine::validation::registry().unwrap();
    let texts = sources();
    let ordinary = load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
    let budget = Arc::new(ObservedBudget::new(512 * 1024 * 1024));
    let cancel = CancellationToken::new();
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget.pool(),
        &cancel,
    )
    .unwrap();
    assert_eq!(
        owned
            .bundle()
            .batches
            .iter()
            .map(|(id, b)| (*id, b.batch().clone()))
            .collect::<BTreeMap<_, _>>(),
        ordinary
            .batches
            .iter()
            .map(|(id, b)| (*id, b.batch().clone()))
            .collect::<BTreeMap<_, _>>()
    );
    assert_eq!(owned.bundle().package, ordinary.package);
    for (actual, expected) in owned.bundle().documents.iter().zip(&ordinary.documents) {
        assert_eq!(actual.text, expected.text);
        assert_eq!(actual.spans, expected.spans);
    }
    let retained = budget.inner.reserved();
    assert!(retained > 0);
    assert!(
        budget.peak() < 4 * 1024 * 1024,
        "minimal peak {}",
        budget.peak()
    );
    let shared = owned.clone();
    assert!(std::ptr::eq(owned.bundle(), shared.bundle()));
    assert_eq!(budget.inner.reserved(), retained);
    let second = OwnedDocumentSet::try_from_bundles(vec![shared], &budget.pool(), &cancel).unwrap();
    second.validate_registry(registry).unwrap();
    drop(second);
    assert_eq!(budget.inner.reserved(), retained);
    let set = OwnedDocumentSet::try_from_bundles(vec![owned], &budget.pool(), &cancel).unwrap();
    set.validate_registry(registry).unwrap();
    let shared = set.clone();
    assert!(std::ptr::eq(
        set.bundles().as_ptr(),
        shared.bundles().as_ptr()
    ));
    drop(set);
    assert!(budget.inner.reserved() >= retained);
    drop(shared);
    assert_eq!(budget.inner.reserved(), 0);
}

#[test]
fn tiny_budget_refuses_before_parsing_long_scalar_and_invalid_syntax() {
    let registry = pse_engine::validation::registry().unwrap();
    let texts = BTreeMap::from([(
        "package.toml".to_owned(),
        format!("not TOML {}", "x".repeat(512 * 1024)),
    )]);
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(1024));
    let error = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        "{error}"
    );
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn cancellation_after_reservation_releases_every_phase() {
    let registry = pse_engine::validation::registry().unwrap();
    let cancel = CancellationToken::new();
    let mut budget = ObservedBudget::new(512 * 1024 * 1024);
    budget.cancel_after_grow = Some(cancel.clone());
    let error = load_package_texts_owned(
        &sources(),
        registry,
        ParseBudget::default(),
        &budget.pool(),
        &cancel,
    )
    .unwrap_err();
    assert!(
        matches!(error, DriverError::Allocation(CanonError::Cancelled)),
        "{error}"
    );
    assert!(budget.peak() > 0);
    assert_eq!(budget.inner.reserved(), 0);
}

#[test]
fn parser_aliases_are_preserved_and_their_expanded_rows_are_validated() {
    let registry = pse_engine::validation::registry().unwrap();
    let mut texts = sources();
    let species = texts.get_mut("materials/species.yaml").unwrap();
    *species = species
        .replace("name: water", "name: &name water")
        .replace("doc: Water species.", "doc: *name");
    let expected = load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
    let budget = Arc::new(ObservedBudget::new(512 * 1024 * 1024));
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget.pool(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(
        owned
            .bundle()
            .batches
            .iter()
            .map(|(id, b)| (*id, b.batch().clone()))
            .collect::<BTreeMap<_, _>>(),
        expected
            .batches
            .iter()
            .map(|(id, b)| (*id, b.batch().clone()))
            .collect::<BTreeMap<_, _>>()
    );
    // Alias handling may reserve the parser node ceiling temporarily. Compare
    // retained storage with the same documents without aliases, including the
    // same native registry, so this measures alias overhead rather than registry size.
    let plain_budget = ObservedBudget::new(512 * 1024 * 1024);
    let plain = load_package_texts_owned(
        &sources(),
        registry,
        ParseBudget::default(),
        &plain_budget.pool(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert!(
        budget.inner.reserved() < plain_budget.inner.reserved() + 64 * 1024,
        "alias retained {}, plain retained {}",
        budget.inner.reserved(),
        plain_budget.inner.reserved()
    );
    drop(plain);
    assert_eq!(plain_budget.inner.reserved(), 0);
    drop(owned);
    assert_eq!(budget.inner.reserved(), 0);
}

#[test]
fn eight_kib_long_scalar_does_not_reserve_the_maximum_parser_node_budget() {
    let registry = pse_engine::validation::registry().unwrap();
    let text = header().replace("Minimal explicit identity fixture.", &"a".repeat(8192));
    let texts = BTreeMap::from([("package.toml".to_owned(), text)]);
    let budget = ObservedBudget::new(8 * 1024 * 1024);
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget.pool(),
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(owned.bundle().package.doc.len(), 8192);
    assert!(
        budget.peak() < 4 * 1024 * 1024,
        "8KiB peak {}",
        budget.peak()
    );
    drop(owned);
    assert_eq!(budget.inner.reserved(), 0);
}

#[test]
fn a_clone_iterator_cannot_understate_actual_source_allocation() {
    struct DifferentClone<'a> {
        text: &'a [u8],
        yielded: bool,
    }
    impl Clone for DifferentClone<'_> {
        fn clone(&self) -> Self {
            Self {
                text: b"",
                yielded: false,
            }
        }
    }
    impl<'a> Iterator for DifferentClone<'a> {
        type Item = (&'a str, &'a [u8]);
        fn next(&mut self) -> Option<Self::Item> {
            if self.yielded {
                None
            } else {
                self.yielded = true;
                Some(("package.toml", self.text))
            }
        }
    }
    let registry = pse_engine::validation::registry().unwrap();
    let text = "x".repeat(256 * 1024);
    let budget: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(2048));
    let error = load_package_sources_owned(
        DifferentClone {
            text: text.as_bytes(),
            yielded: false,
        },
        registry,
        ParseBudget::default(),
        &budget,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        "{error}"
    );
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn long_qualified_names_reserve_expansion_before_identity_hydration() {
    let registry = pse_engine::validation::registry().unwrap();
    let mut elements = String::from("elements:\n");
    for ordinal in 0..24 {
        assert!(
            write!(
                elements,
                "  - symbol: E{ordinal}{}\n    name: Element\n    atomic_mass: 1.0\n",
                "x".repeat(2048)
            )
            .is_ok()
        );
    }
    let texts = BTreeMap::from([
        (
            "package.toml".to_owned(),
            header().replace("id_policy = \"explicit\"", "id_policy = \"named\""),
        ),
        ("materials/elements.yaml".to_owned(), elements),
    ]);
    let small: Arc<dyn MemoryPool> = Arc::new(pse_columnar::GreedyMemoryPool::new(4 * 1024 * 1024));
    let error = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &small,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert_eq!(
        pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
        Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),
        "{error}"
    );
    assert_eq!(small.reserved(), 0);
    let budget: Arc<dyn MemoryPool> =
        Arc::new(pse_columnar::GreedyMemoryPool::new(64 * 1024 * 1024));
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget,
        &CancellationToken::new(),
    )
    .unwrap();
    let batch = &owned.bundle().batches[&pse_relations::generated::authored::entities::RELATION_ID];
    let entities = pse_relations::generated::authored::entities::View::from_checked(batch)
        .unwrap()
        .rows()
        .unwrap();
    assert_eq!(entities.len(), 24);
    for entity in entities {
        assert!(entity.qualified_name.starts_with("minimal.E"));
        assert!(entity.qualified_name.len() > 2048);
    }
    drop(owned);
    assert_eq!(budget.reserved(), 0);
}
