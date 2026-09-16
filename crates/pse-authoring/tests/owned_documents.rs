// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Reserve-first parsing, immutable source ownership and direct declaration binding.

use pse_authoring::{
    AuthoringError, ParseBudget,
    document::{
        OwnedDocumentSet, load_package_sources_owned, load_package_texts, load_package_texts_owned,
    },
};
use pse_ids::{
    CancellationToken, CanonError, FixedBudget, MemoryReserver, Reservation, ReserveError,
};
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
    inner: Arc<FixedBudget>,
    peak: Arc<AtomicUsize>,
    cancel_after_grow: Option<CancellationToken>,
}
impl ObservedBudget {
    fn new(bytes: usize) -> Self {
        Self {
            inner: FixedBudget::new(bytes),
            peak: Arc::new(AtomicUsize::new(0)),
            cancel_after_grow: None,
        }
    }
    fn peak(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
}
impl MemoryReserver for ObservedBudget {
    fn open(&self, owner: &str) -> Box<dyn Reservation> {
        Box::new(ObservedReservation {
            inner: self.inner.open(owner),
            budget: Arc::clone(&self.inner),
            peak: Arc::clone(&self.peak),
            cancel: self.cancel_after_grow.clone(),
        })
    }
}
#[derive(Debug)]
struct ObservedReservation {
    inner: Box<dyn Reservation>,
    budget: Arc<FixedBudget>,
    peak: Arc<AtomicUsize>,
    cancel: Option<CancellationToken>,
}
impl Reservation for ObservedReservation {
    fn try_grow(&mut self, bytes: usize) -> Result<(), ReserveError> {
        self.inner.try_grow(bytes)?;
        self.peak
            .fetch_max(self.budget.reserved(), Ordering::Relaxed);
        if let Some(cancel) = &self.cancel {
            cancel.cancel();
        }
        Ok(())
    }
    fn shrink(&mut self, bytes: usize) {
        self.inner.shrink(bytes);
    }
    fn size(&self) -> usize {
        self.inner.size()
    }
    fn release(&mut self) {
        self.inner.release();
    }
}

#[test]
fn owned_loader_matches_actual_rows_and_spans_and_retains_only_shared_owners() {
    let registry = pse_schema::registry().unwrap();
    let texts = sources();
    let ordinary = load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
    let budget = ObservedBudget::new(512 * 1024 * 1024);
    let cancel = CancellationToken::new();
    let owned =
        load_package_texts_owned(&texts, registry, ParseBudget::default(), &budget, &cancel)
            .unwrap();
    assert_eq!(
        owned.bundle().decode_rows(registry).unwrap(),
        ordinary.decode_rows(registry).unwrap()
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
    let second = OwnedDocumentSet::try_from_bundles(vec![shared], &budget, &cancel).unwrap();
    second.validate_registry(registry).unwrap();
    drop(second);
    assert_eq!(budget.inner.reserved(), retained);
    let set = OwnedDocumentSet::try_from_bundles(vec![owned], &budget, &cancel).unwrap();
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
    let registry = pse_schema::registry().unwrap();
    let texts = BTreeMap::from([(
        "package.toml".to_owned(),
        format!("not TOML {}", "x".repeat(512 * 1024)),
    )]);
    let budget = FixedBudget::new(1024);
    let error = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        budget.as_ref(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(matches!(error, AuthoringError::Resource(_)), "{error}");
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn cancellation_after_reservation_releases_every_phase() {
    let registry = pse_schema::registry().unwrap();
    let cancel = CancellationToken::new();
    let mut budget = ObservedBudget::new(512 * 1024 * 1024);
    budget.cancel_after_grow = Some(cancel.clone());
    let error = load_package_texts_owned(
        &sources(),
        registry,
        ParseBudget::default(),
        &budget,
        &cancel,
    )
    .unwrap_err();
    assert!(
        matches!(error, AuthoringError::Allocation(CanonError::Cancelled)),
        "{error}"
    );
    assert!(budget.peak() > 0);
    assert_eq!(budget.inner.reserved(), 0);
}

#[test]
fn parser_aliases_are_preserved_and_their_expanded_rows_are_validated() {
    let registry = pse_schema::registry().unwrap();
    let mut texts = sources();
    let species = texts.get_mut("materials/species.yaml").unwrap();
    *species = species
        .replace("name: water", "name: &name water")
        .replace("doc: Water species.", "doc: *name");
    let expected = load_package_texts(texts.clone(), registry, ParseBudget::default()).unwrap();
    let budget = ObservedBudget::new(512 * 1024 * 1024);
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget,
        &CancellationToken::new(),
    )
    .unwrap();
    assert_eq!(
        owned.bundle().decode_rows(registry).unwrap(),
        expected.decode_rows(registry).unwrap()
    );
    // Alias handling may reserve the parser node ceiling temporarily. Compare
    // retained storage with the same documents without aliases, including the
    // same native registry, so this measures alias overhead rather than registry size.
    let plain_budget = ObservedBudget::new(512 * 1024 * 1024);
    let plain = load_package_texts_owned(
        &sources(),
        registry,
        ParseBudget::default(),
        &plain_budget,
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
    let registry = pse_schema::registry().unwrap();
    let text = header().replace("Minimal explicit identity fixture.", &"a".repeat(8192));
    let texts = BTreeMap::from([("package.toml".to_owned(), text)]);
    let budget = ObservedBudget::new(8 * 1024 * 1024);
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        &budget,
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
    let registry = pse_schema::registry().unwrap();
    let text = "x".repeat(256 * 1024);
    let budget = FixedBudget::new(2048);
    let error = load_package_sources_owned(
        DifferentClone {
            text: text.as_bytes(),
            yielded: false,
        },
        registry,
        ParseBudget::default(),
        budget.as_ref(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(matches!(error, AuthoringError::Resource(_)), "{error}");
    assert_eq!(budget.reserved(), 0);
}

#[test]
fn long_qualified_names_reserve_expansion_before_identity_hydration() {
    let registry = pse_schema::registry().unwrap();
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
    let small = FixedBudget::new(4 * 1024 * 1024);
    let error = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        small.as_ref(),
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(matches!(error, AuthoringError::Resource(_)), "{error}");
    assert_eq!(small.reserved(), 0);
    let budget = FixedBudget::new(64 * 1024 * 1024);
    let owned = load_package_texts_owned(
        &texts,
        registry,
        ParseBudget::default(),
        budget.as_ref(),
        &CancellationToken::new(),
    )
    .unwrap();
    let rows = owned.bundle().decode_rows(registry).unwrap();
    let entities = &rows[&pse_relations::generated::authored::entities::RELATION_ID];
    assert_eq!(entities.len(), 24);
    for row in entities {
        let entity =
            pse_relations::generated::authored::entities::Row::from_cells(row.clone()).unwrap();
        assert!(entity.qualified_name.starts_with("minimal.E"));
        assert!(entity.qualified_name.len() > 2048);
    }
    drop(owned);
    assert_eq!(budget.reserved(), 0);
}
