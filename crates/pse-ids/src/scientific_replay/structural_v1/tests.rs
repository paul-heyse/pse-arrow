// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

fn limits() -> Limits { Limits { nodes: 100_000, key_bytes: 1 << 20, helper_bytes: 4096, depth: 256 } }
fn binding() -> ContentHash { ContentHash::from_bytes([0x65; 32]) }
fn counts<T: ?Sized + Serialize>(value: &T) -> Counts {
    let limits = limits();
    count(value, limits, limits.count_scratch_bytes().unwrap()).unwrap()
}
fn index<T: ?Sized + Serialize>(value: &T) -> Index {
    let counts = counts(value);
    build_index(value, binding(), counts, counts.index_scratch_bytes().unwrap()).unwrap()
}
fn query<T: ?Sized + Serialize>(value: &T) -> ContentHash {
    let counts = counts(value);
    query_hash(value, binding(), counts, counts.query_scratch_bytes().unwrap()).unwrap()
}
fn differential<T: ?Sized + Serialize>(value: &T) -> Index {
    let dom = serde_json::to_value(value).unwrap();
    let mut expected = BTreeSet::new();
    let root = super::super::structural_hash(&dom, binding(), Some(&mut expected));
    let index = index(value);
    assert_eq!(index.root, root);
    assert_eq!(index.nodes.iter().copied().collect::<BTreeSet<_>>(), expected);
    assert!(index.nodes.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(query(value), root);
    index
}

#[derive(Clone, Debug)]
enum KeyValue { Text(String), Integer(i64), Boolean(bool) }
impl Serialize for KeyValue {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self { Self::Text(v) => serializer.serialize_str(v), Self::Integer(v) => serializer.serialize_i64(*v), Self::Boolean(v) => serializer.serialize_bool(*v) }
    }
}
#[derive(Clone, Debug)]
enum Node { Null, Bool(bool), Integer(i64), Text(String), Array(Vec<Self>), Object(Vec<(KeyValue, Self)>) }
impl Serialize for Node {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Integer(v) => serializer.serialize_i64(*v),
            Self::Text(v) => serializer.serialize_str(v),
            Self::Array(v) => v.serialize(serializer),
            Self::Object(v) => {
                let mut map = serializer.serialize_map(None)?;
                for (key, value) in v { ser::SerializeMap::serialize_entry(&mut map, key, value)?; }
                ser::SerializeMap::end(map)
            }
        }
    }
}
fn field(name: &str, value: Node) -> (KeyValue, Node) { (KeyValue::Text(name.into()), value) }
fn secret() -> Node { Node::Object(vec![field("secret", Node::Integer(731))]) }

#[test]
fn membership_v1_overwritten_subtrees_and_rendered_aliases_preserve_only_survivors() {
    let overwritten = Node::Object(vec![field("k", secret()), field("k", Node::Integer(0))]);
    let result = differential(&overwritten);
    assert!(!result.nodes.contains(&query(&secret())));
    assert!(!result.nodes.contains(&query(&731_i64)));
    assert!(result.nodes.contains(&query(&0_i64)));
    let overlaps = Node::Object(vec![
        field("outer", overwritten),
        field("outer", Node::Bool(false)),
        field("surviving", secret()),
    ]);
    let result = differential(&overlaps);
    assert!(result.nodes.contains(&query(&secret())));
    assert!(result.nodes.contains(&query(&731_i64)), "an equal surviving digest must remain granted");
    assert!(!result.nodes.contains(&query(&0_i64)));

    let aliases = Node::Object(vec![
        (KeyValue::Integer(7), secret()),
        field("7", Node::Bool(true)),
        (KeyValue::Boolean(false), Node::Integer(888)),
        field("false", Node::Null),
    ]);
    let result = differential(&aliases);
    assert!(!result.nodes.contains(&query(&secret())));
    assert!(!result.nodes.contains(&query(&888_i64)));
}

#[test]
fn membership_v1_normalization_helpers_do_not_index_dummy_nodes_or_keys() {
    let result = differential(&Node::Object(vec![field("only-key", Node::Bool(true))]));
    assert!(!result.nodes.contains(&query(&())));
    assert!(!result.nodes.contains(&query("only-key")));
    assert!(result.nodes.contains(&query(&true)));
}

struct OrphanKeys;
impl Serialize for OrphanKeys {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        ser::SerializeMap::serialize_key(&mut map, "overwritten-before-value")?;
        ser::SerializeMap::serialize_key(&mut map, "real-key")?;
        ser::SerializeMap::serialize_value(&mut map, &true)?;
        ser::SerializeMap::serialize_key(&mut map, "unpaired-at-end")?;
        ser::SerializeMap::end(map)
    }
}
#[test]
fn membership_v1_orphan_keys_are_not_entries_values_or_members() {
    let actual = differential(&OrphanKeys);
    let expected = index(&BTreeMap::from([("real-key", true)]));
    assert_eq!(actual.root, expected.root);
    assert_eq!(actual.nodes, expected.nodes);
    assert!(!actual.nodes.contains(&query("overwritten-before-value")));
    assert!(!actual.nodes.contains(&query("unpaired-at-end")));
}

#[derive(Serialize)]
struct Unit;
#[derive(Serialize)]
struct Newtype<T>(T);
#[derive(Serialize)]
struct Tuple(u64, String);
#[derive(Serialize)]
enum Alternatives {
    Unit,
    Newtype(Option<u64>),
    Tuple(bool, Vec<u8>),
    Struct { source: String, values: Vec<Option<i64>> },
}
struct Bytes<'a>(&'a [u8]);
impl Serialize for Bytes<'_> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> { serializer.serialize_bytes(self.0) }
}
struct HumanReadable;
impl Serialize for HumanReadable {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let human = serializer.is_human_readable();
        serializer.serialize_str(if human { "human" } else { "binary" })
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Collected<'a>(&'a str);
impl fmt::Display for Collected<'_> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.0) } }
impl Serialize for Collected<'_> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> { serializer.collect_str(self) }
}

#[test]
fn membership_v1_wrappers_variants_bytes_unicode_and_human_readable_match_dom() {
    differential(&Unit);
    differential(&Newtype(Some(Newtype(None::<u64>))));
    differential(&Tuple(0, "nul\0and\nquote\"ユニコード".into()));
    differential(&Alternatives::Unit);
    let result = differential(&Alternatives::Newtype(None));
    assert!(result.nodes.contains(&query(&())));
    assert!(!result.nodes.contains(&query("Newtype")));
    let result = differential(&Alternatives::Tuple(true, vec![0, 255, 128]));
    assert!(result.nodes.contains(&query(&(true, vec![0_u8, 255, 128]))));
    differential(&Alternatives::Struct { source: "化学".into(), values: vec![None, Some(-2)] });
    differential(&Bytes(&[0, 255, 17]));
    differential(&HumanReadable);
    differential(&Collected("raw\0display\ntext"));
    differential(&BTreeMap::from([(Collected("display-key"), 17_u64)]));
    differential(&'\u{1f9ea}');
}

#[test]
fn membership_v1_numeric_profile_preserves_pinned_scalar_meaning_and_explicit_refusals() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0, 0.0, f64::MIN, f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1), 1.0 / 3.0] { differential(&value); }
    for value in [f32::NAN, f32::INFINITY, -0.0, 0.0, f32::MAX, f32::MIN_POSITIVE, f32::from_bits(1), 0.1_f32] { differential(&value); }
    for value in [i128::from(i64::MIN), -1, 0, i128::from(i64::MAX), i128::from(u64::MAX)] { differential(&value); }
    differential(&u128::from(u64::MAX));
    for value in [i128::MIN, i128::from(i64::MIN) - 1, i128::from(u64::MAX) + 1, i128::MAX] {
        assert!(matches!(count(&value, limits(), limits().count_scratch_bytes().unwrap()), Err(Error::Profile(_))));
    }
    assert!(matches!(count(&u128::MAX, limits(), limits().count_scratch_bytes().unwrap()), Err(Error::Profile(_))));
    assert!(serde_json::to_value(u128::MAX).is_err(), "this profile deliberately excludes arbitrary_precision");
    // Integer KEYS are rendered strings, not normalized number value nodes.
    differential(&BTreeMap::from([(u128::MAX, true)]));
    differential(&BTreeMap::from([(i128::MIN, true)]));
}

struct Sentinel(&'static str);
impl Serialize for Sentinel {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut value = serializer.serialize_struct(self.0, 1)?;
        ser::SerializeStruct::serialize_field(&mut value, self.0, "1")?;
        ser::SerializeStruct::end(value)
    }
}
#[test]
fn membership_v1_private_sentinels_and_unsupported_map_keys_refuse_without_fallback() {
    for sentinel in [RAW_SENTINEL, NUMBER_SENTINEL] {
        assert!(matches!(count(&Sentinel(sentinel), limits(), limits().count_scratch_bytes().unwrap()), Err(Error::Profile("private raw/number sentinel"))));
    }
    fn refused<T: Serialize>(key: &T) {
        let flag = Cell::new(false);
        let bounded = Key { value: key, limit: 4096, depth: 256, exhausted: &flag };
        assert!(serde_json::to_value(OneKey(&bounded)).is_err());
        assert!(normalize_key(key, 4096, 256).is_err());
    }
    refused(&None::<String>);
    refused(&Some("key"));
    refused(&());
    refused(&vec![1_u8, 2]);
    refused(&Alternatives::Newtype(Some(1)));
    refused(&f64::NAN);
    refused(&f64::INFINITY);
    assert_eq!(normalize_key(&Newtype("valid"), 4096, 256).unwrap(), "valid");
}

struct Hints { hint: Option<usize>, values: Vec<u64>, object: bool }
impl Serialize for Hints {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        if self.object {
            let mut map = serializer.serialize_map(self.hint)?;
            for value in &self.values { ser::SerializeMap::serialize_entry(&mut map, value, value)?; }
            ser::SerializeMap::end(map)
        } else {
            let mut seq = serializer.serialize_seq(self.hint)?;
            for value in &self.values { ser::SerializeSeq::serialize_element(&mut seq, value)?; }
            ser::SerializeSeq::end(seq)
        }
    }
}
#[test]
fn membership_v1_actual_lengths_ignore_unknown_false_and_malicious_hints() {
    // Never pass malicious hints to the old eager Value serializer: it reserves
    // them. The independent DOM oracle uses the same emissions with no hint.
    for object in [false, true] {
        let truthful = Hints { hint: None, values: vec![3, 2, 3, 1], object };
        let expected = differential(&truthful);
        for hint in [None, Some(0), Some(1), Some(8), Some(usize::MAX)] {
            let candidate = Hints { hint, values: truthful.values.clone(), object };
            let actual = index(&candidate);
            assert_eq!(actual.root, expected.root);
            assert_eq!(actual.nodes, expected.nodes);
            assert_eq!(counts(&candidate).actual, counts(&truthful).actual);
        }
    }
}

struct Observed<'a>(&'a Cell<bool>);
impl Serialize for Observed<'_> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> { self.0.set(true); serializer.serialize_bool(true) }
}
#[test]
fn membership_v1_admits_own_scratch_before_effects_and_checks_all_extents() {
    use pse_diagnostics::TypedDiagnostic as _;
    let observed = Cell::new(false);
    let value = Observed(&observed);
    let refused = count(&value, limits(), 0).unwrap_err();
    assert!(matches!(refused, Error::Limit(_)));
    assert_eq!(refused.diagnostic_code(), Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit));
    assert!(!observed.get());
    let counted = counts(&value);
    observed.set(false);
    assert!(matches!(build_index(&value, binding(), counted, counted.index_scratch_bytes().unwrap() - 1), Err(Error::Limit(_))));
    assert!(!observed.get());
    assert!(matches!(query_hash(&value, binding(), counted, counted.query_scratch_bytes().unwrap() - 1), Err(Error::Limit(_))));
    assert!(!observed.get());
    let mut overflow = counted;
    overflow.actual.nodes = usize::MAX;
    assert!(matches!(overflow.index_scratch_bytes(), Err(Error::Limit("extent overflow"))));
    overflow = counted;
    overflow.actual.edges = usize::MAX;
    assert!(matches!(overflow.query_scratch_bytes(), Err(Error::Limit("extent overflow"))));
    assert!(matches!(Limits { helper_bytes: usize::MAX, ..limits() }.count_scratch_bytes(), Err(Error::Limit("extent overflow"))));
    let limits = Limits { nodes: 0, ..limits() };
    assert!(matches!(count(&true, limits, limits.count_scratch_bytes().unwrap()), Err(Error::Limit("normalized nodes"))));
}

#[test]
fn membership_v1_key_format_depth_and_node_exhaustion_refuse_without_partial_index() {
    let small = Limits { helper_bytes: 4, ..limits() };
    assert!(matches!(count(&Node::Object(vec![field("too-long", Node::Null)]), small, small.count_scratch_bytes().unwrap()), Err(Error::Limit(_))));
    assert!(matches!(count(&BTreeMap::from([(Collected("too-long"), 0)]), small, small.count_scratch_bytes().unwrap()), Err(Error::Limit(_))));
    assert!(matches!(count(&Collected("too-long"), small, small.count_scratch_bytes().unwrap()), Err(Error::Limit(_))));
    assert!(matches!(normalize_key(&12345_u64, 4, 256), Err(Error::Limit(_))));
    assert!(matches!(normalize_key(&Newtype(Newtype("key")), 4096, 1), Err(Error::Limit(_))));
    let shallow = Limits { depth: 1, ..limits() };
    assert!(matches!(count(&Newtype(Newtype(7)), shallow, shallow.count_scratch_bytes().unwrap()), Err(Error::Limit("native serialization depth"))));
    let no_keys = Limits { key_bytes: 0, ..limits() };
    assert!(matches!(count(&Node::Object(vec![field("k", Node::Null)]), no_keys, no_keys.count_scratch_bytes().unwrap()), Err(Error::Limit("rendered key bytes"))));

    let original = vec![1_u64];
    let counted = counts(&original);
    assert!(matches!(build_index(&vec![1_u64, 2], binding(), counted, counted.index_scratch_bytes().unwrap()), Err(Error::Limit(_))));
    assert!(matches!(build_index(&Vec::<u64>::new(), binding(), counted, counted.index_scratch_bytes().unwrap()), Err(Error::ChangedShape)));
}

#[test]
fn membership_v1_real_nominal_ids_and_scientific_record_shapes_keep_scope_and_exact_bits() {
    #[derive(Serialize)]
    struct Proof { request: ContentHash, branches: Vec<u8> }
    #[derive(Serialize)]
    struct Description { identity: ContentHash, physical: ContentHash, checked: BTreeSet<crate::SemanticId>, proofs: Vec<Proof>, scale_bits: u64 }
    // Actual identity owners are legal at this crate boundary. Mathematical and
    // physical owner crates cannot become dev-dependencies of their identity crate.
    let description = Description {
        identity: ContentHash::from_bytes([1; 32]), physical: ContentHash::from_bytes([2; 32]),
        checked: BTreeSet::from([crate::SemanticId::from_bytes([0x22; 16]), crate::SemanticId::from_bytes([0x11; 16])]),
        proofs: vec![Proof { request: ContentHash::from_bytes([7; 32]), branches: vec![0, 2, 255] }],
        scale_bits: (-0.0_f64).to_bits(),
    };
    let result = differential(&description);
    assert!(result.nodes.contains(&query(&description.proofs[0])));
    let changed = Proof { request: description.proofs[0].request, branches: vec![0, 2, 254] };
    assert!(!result.nodes.contains(&query(&changed)));
    let counts = counts(&description);
    let foreign = build_index(&description, ContentHash::from_bytes([0x66; 32]), counts, counts.index_scratch_bytes().unwrap()).unwrap();
    assert_ne!(result.root, foreign.root);
    assert!(!foreign.nodes.contains(&query(&description.proofs[0])));
    differential(&description.identity);
    differential(&crate::roles::SemanticBodyHash::from(description.identity));
    assert_ne!(query(&(-0.0_f64).to_bits()), query(&0.0_f64.to_bits()));
}

fn node_strategy() -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![Just(Node::Null), any::<bool>().prop_map(Node::Bool), any::<i64>().prop_map(Node::Integer),
        proptest::collection::vec(any::<char>(), 0..12).prop_map(|chars| Node::Text(chars.into_iter().collect()))];
    leaf.prop_recursive(5, 128, 8, |inner| prop_oneof![
        proptest::collection::vec(inner.clone(), 0..6).prop_map(Node::Array),
        proptest::collection::vec((prop_oneof![Just(KeyValue::Integer(1)), Just(KeyValue::Text("1".into())),
            Just(KeyValue::Text("k".into())), Just(KeyValue::Boolean(false)), Just(KeyValue::Text("false".into()))], inner), 0..6).prop_map(Node::Object),
    ])
}
proptest! {
    #[test]
    fn membership_v1_differential_surviving_node_population(value in node_strategy()) { differential(&value); }
    #[test]
    fn membership_v1_differential_all_binary64_values(bits in any::<u64>()) { differential(&f64::from_bits(bits)); }
}
