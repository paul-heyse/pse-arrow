// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Selected authored syntax and concrete physical receipts. Native library values are
//! rebuilt using the ordinary lowering; no native atoms or workspace pointers persist.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const INTERPRETATION: &str = pse_ids::scientific_replay::ADMITTED_RECIPE_INTERPRETATION;
const MAX_PAYLOAD: usize = pse_ids::scientific_replay::MAX_ADMITTED_RECIPE_BYTES;
// Allocation-free serialization preflight. Overflow cannot become a small accepted extent.
#[derive(Default)]
struct ByteCount(usize);
impl std::io::Write for ByteCount {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("portable body encoded extent overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct Recipe {
    interpretation: String,
    pub(super) semantic_identity: Option<pse_ids::roles::SemanticBodyHash>,
    pub(super) checked_members: Option<BTreeSet<SemanticId>>,
    definition: SemanticId,
    expressions: Vec<Expr>,
    formals: Vec<Formal>,
    domains: Vec<DomainWire>,
    groups: Vec<GroupWire>,
    literals: Vec<((u32, u32), QuantityTypeId)>,
    physical: ContentHash,
    structure: ContentHash,
    slots: u64,
    occurrences: u64,
    outputs: Vec<QuantityTypeId>,
    local_quantities: BTreeMap<String, QuantityTypeId>,
    validity: BTreeMap<String, Validity>,
    functions: BTreeMap<String, pse_modeling::portable::FunctionRecord>,
    providers: BTreeMap<String, ProviderWire>,
    receipts: pse_quantity::resolved::receipts::ReceiptStream,
    proofs: pse_math::typed::receipts::ProofStream,
    spec: SpecWire,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DomainWire {
    name: String,
    id: SemanticId,
    members: Vec<SemanticId>,
    kind: pse_quantity::EntityKindId,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GroupWire {
    name: String,
    quantity: QuantityTypeId,
    axes: Vec<String>,
    slots: Vec<(Vec<SemanticId>, usize)>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ProviderWire {
    descriptor: pse_kernels::portable::ProviderRecord,
    output: usize,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SpecWire {
    definition: ContentHash,
    structure: ContentHash,
    physical: ContentHash,
    admissions: ContentHash,
    providers: Vec<ContentHash>,
    policy: ContentHash,
}
impl SpecWire {
    fn of(spec: &BodySpec) -> Self {
        Self {
            definition: spec.definition,
            structure: spec.structure,
            physical: spec.physical,
            admissions: spec.admissions,
            providers: spec.providers.clone(),
            policy: spec.policy,
        }
    }
    #[cfg(test)]
    fn matches(&self, spec: &BodySpec) -> bool {
        self == &Self::of(spec)
    }
    fn spec(&self) -> BodySpec {
        BodySpec {
            definition: self.definition,
            structure: self.structure,
            physical: self.physical,
            admissions: self.admissions,
            providers: self.providers.clone(),
            policy: self.policy,
        }
    }
}
impl Recipe {
    #[expect(
        clippy::too_many_arguments,
        reason = "one admitted body recipe captures its request, functions, outputs, local physical facts, quantity receipts, typed proofs and body specification from their owning scientific types"
    )]
    pub(super) fn capture(
        request: &Request<'_>,
        functions: BTreeMap<String, pse_modeling::portable::FunctionRecord>,
        outputs: &[QuantityTypeId],
        local_quantities: &BTreeMap<String, QuantityTypeId>,
        validity: &BTreeMap<String, Validity>,
        receipts: pse_quantity::resolved::receipts::ReceiptStream,
        proofs: pse_math::typed::receipts::ProofStream,
        spec: &BodySpec,
    ) -> Self {
        Self {
            interpretation: INTERPRETATION.into(),
            semantic_identity: None,
            checked_members: None,
            definition: request.definition,
            expressions: request.expressions.to_vec(),
            formals: request.formals.to_vec(),
            domains: request
                .domains
                .iter()
                .map(|(name, v)| DomainWire {
                    name: name.clone(),
                    id: v.members.id,
                    members: v.members.members().to_vec(),
                    kind: v.kind,
                })
                .collect(),
            groups: request
                .groups
                .iter()
                .map(|(name, v)| GroupWire {
                    name: name.clone(),
                    quantity: v.quantity,
                    axes: v.axes.clone(),
                    slots: v.slots.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                })
                .collect(),
            literals: request.literals.iter().map(|(k, v)| (*k, *v)).collect(),
            physical: request.physical,
            structure: request.structure,
            slots: request.limits.slots as u64,
            occurrences: request.limits.occurrences as u64,
            outputs: outputs.to_vec(),
            local_quantities: local_quantities.clone(),
            validity: validity.clone(),
            functions,
            providers: request
                .providers
                .iter()
                .map(|(n, v)| {
                    (
                        n.clone(),
                        ProviderWire {
                            descriptor: pse_kernels::portable::ProviderRecord::capture(
                                &v.descriptor,
                            ),
                            output: v.output,
                        },
                    )
                })
                .collect(),
            receipts,
            proofs,
            spec: SpecWire::of(spec),
        }
    }
    pub(super) fn encoded_bytes(&self) -> Result<usize, MathError> {
        let mut finite = true;
        for expression in self.expressions.iter().chain(
            self.validity
                .values()
                .flat_map(|range| [&range.lower, &range.upper]),
        ) {
            expression.walk(|node| {
                if let ExprKind::Number(number) = &node.kind {
                    finite &= number.value.is_finite();
                }
            });
        }
        if !finite {
            return Err(MathError::Contract(
                "portable scientific literal must be finite".into(),
            ));
        }
        // Count through serde's ordinary writer before allocating the description.
        // Scientific records and their existing wire schema remain authoritative.
        let mut count = ByteCount::default();
        serde_json::to_writer(&mut count, self)
            .map_err(|e| MathError::Contract(format!("portable body encoding: {e}")))?;
        if count.0 > MAX_PAYLOAD {
            return Err(MathError::ByteLimit {
                resource: "portable body payload",
                required: count.0,
                available: MAX_PAYLOAD,
            });
        }
        Ok(count.0)
    }
    pub(super) fn encode(&self) -> Result<Vec<u8>, MathError> {
        let extent = self.encoded_bytes()?;
        let mut bytes = Vec::with_capacity(extent);
        serde_json::to_writer(&mut bytes, self)
            .map_err(|e| MathError::Contract(format!("portable body encoding: {e}")))?;
        if bytes.len() != extent {
            return Err(MathError::Contract(
                "portable body encoded extent changed".into(),
            ));
        }
        Ok(bytes)
    }
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>()
            + self.interpretation.capacity()
            + self.expressions.capacity() * size_of::<Expr>()
            + self
                .expressions
                .iter()
                .map(pse_modeling::expression::retained_bytes)
                .sum::<usize>()
            + self.formals.capacity() * size_of::<Formal>()
            + self
                .formals
                .iter()
                .map(|f| f.path.capacity())
                .sum::<usize>()
            + self.domains.capacity() * size_of::<DomainWire>()
            + self
                .domains
                .iter()
                .map(|v| v.name.capacity() + v.members.capacity() * size_of::<SemanticId>())
                .sum::<usize>()
            + self.groups.capacity() * size_of::<GroupWire>()
            + self
                .groups
                .iter()
                .map(|v| {
                    v.name.capacity()
                        + v.axes.capacity() * size_of::<String>()
                        + v.axes.iter().map(String::capacity).sum::<usize>()
                        + v.slots.capacity() * size_of::<(Vec<SemanticId>, usize)>()
                        + v.slots
                            .iter()
                            .map(|(k, _)| k.capacity() * size_of::<SemanticId>())
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.literals.capacity() * size_of::<((u32, u32), QuantityTypeId)>()
            + self.outputs.capacity() * size_of::<QuantityTypeId>()
            + self.spec.providers.capacity() * size_of::<ContentHash>()
            + self.receipts.heap_bytes()
            + self
                .validity
                .iter()
                .map(|(name, v)| {
                    128 + size_of::<(String, Validity)>()
                        + name.capacity()
                        + pse_modeling::expression::retained_bytes(&v.lower)
                        + pse_modeling::expression::retained_bytes(&v.upper)
                })
                .sum::<usize>()
            + self
                .local_quantities
                .keys()
                .map(|name| 128 + size_of::<(String, QuantityTypeId)>() + name.capacity())
                .sum::<usize>()
            + self.proofs.retained_bytes()
            + self
                .checked_members
                .as_ref()
                .map_or(0, |v| v.len() * (size_of::<SemanticId>() + 96))
            + self
                .functions
                .iter()
                .map(|(name, v)| 128 + size_of::<String>() + name.capacity() + v.retained_bytes())
                .sum::<usize>()
            + self
                .providers
                .iter()
                .map(|(name, v)| {
                    128 + size_of::<String>()
                        + size_of::<usize>()
                        + name.capacity()
                        + v.descriptor.retained_bytes()
                })
                .sum::<usize>()
    }
}
/// Versioned payload identity. The canonical product owner stores this digest alongside
/// the admitted dependency and producer identities, then supplies it at reconstruction.
pub fn portable_payload_hash(payload: &[u8]) -> ContentHash {
    pse_ids::scientific_replay::admitted_recipe_payload_hash(payload)
}
/// Reconstruct only from the opaque authority issued at the controlled canonical
/// product boundary. Every scientific record must be present in those exact bytes.
/// Missing receipts are errors, with no admission or physical-inference fallback.
/// # Errors
/// Unsupported interpretation, incomplete receipts or numerical construction failure.
pub fn reconstruct_portable(
    permit: &pse_ids::scientific_replay::QualifiedScientificRecipe,
    registry: &QuantityRegistry,
    cancelled: &Arc<AtomicBool>,
) -> Result<AdmittedBody, MathError> {
    let payload = permit.payload();
    let recipe: Recipe = serde_json::from_slice(payload)
        .map_err(|e| MathError::Contract(format!("portable body decoding: {e}")))?;
    let expected_spec = recipe.spec.spec();
    if recipe.interpretation != INTERPRETATION
        || recipe.physical != expected_spec.physical
        || recipe.structure != expected_spec.structure
    {
        return Err(MathError::Contract(
            "portable body interpretation or admitted specification differs from qualified product"
                .into(),
        ));
    }
    let limits = BodyLimits {
        slots: usize::try_from(recipe.slots)
            .map_err(|_| MathError::Limit("portable body slots"))?,
        occurrences: usize::try_from(recipe.occurrences)
            .map_err(|_| MathError::Limit("portable body occurrences"))?,
    };
    let mut domains = BTreeMap::new();
    for v in &recipe.domains {
        if domains
            .insert(
                v.name.clone(),
                Domain {
                    members: pse_math::binding::FiniteDomain::new(
                        v.id,
                        v.members.clone(),
                        limits.occurrences,
                    )?,
                    kind: v.kind,
                },
            )
            .is_some()
        {
            return Err(MathError::Contract("duplicate portable domain".into()));
        }
    }
    let mut groups = BTreeMap::new();
    for v in &recipe.groups {
        let slots = v.slots.iter().cloned().collect::<BTreeMap<_, _>>();
        if slots.len() != v.slots.len()
            || groups
                .insert(
                    v.name.clone(),
                    Group {
                        quantity: v.quantity,
                        axes: v.axes.clone(),
                        slots,
                    },
                )
                .is_some()
        {
            return Err(MathError::Contract(
                "duplicate portable group membership".into(),
            ));
        }
    }
    let literals = recipe.literals.iter().copied().collect::<BTreeMap<_, _>>();
    if literals.len() != recipe.literals.len() {
        return Err(MathError::Contract(
            "duplicate portable literal occurrence".into(),
        ));
    }
    let mut body =
        pse_quantity::resolved::receipts::replay(&recipe.receipts, permit.records(), || {
            let functions = recipe
                .functions
                .iter()
                .map(|(n, v)| {
                    v.restore()
                        .map(|v| (n.clone(), v))
                        .map_err(|e| MathError::Contract(e.to_string()))
                })
                .collect::<Result<BTreeMap<_, _>, MathError>>()?;
            let providers = recipe
                .providers
                .iter()
                .map(|(n, v)| {
                    v.descriptor
                        .restore()
                        .map(|descriptor| {
                            (
                                n.clone(),
                                ProviderCall {
                                    descriptor,
                                    output: v.output,
                                },
                            )
                        })
                        .map_err(|e| MathError::Contract(e.to_string()))
                })
                .collect::<Result<BTreeMap<_, _>, MathError>>()?;
            let request = Request {
                definition: recipe.definition,
                expressions: &recipe.expressions,
                formals: &recipe.formals,
                domains: &domains,
                groups: &groups,
                providers: &providers,
                literals: &literals,
                physical: recipe.physical,
                structure: recipe.structure,
                limits,
            };
            pse_math::typed::receipts::replay(&recipe.proofs, permit.records(), || {
                request.admit_modeling_outputs_inner(
                    registry,
                    &pse_quantity::infer::NoInvariantFacts,
                    cancelled,
                    &functions,
                    &recipe.outputs,
                    &recipe.local_quantities,
                    &recipe.validity,
                )
            })
        })?;
    if body.spec != expected_spec {
        return Err(MathError::Contract(
            "reconstructed numerical body differs from admitted specification".into(),
        ));
    }
    if let Some(members) = &recipe.checked_members {
        body.math =
            Arc::new(Arc::unwrap_or_clone(body.math).with_checked_members(members.clone())?);
    }
    body.semantic_identity = recipe.semantic_identity;
    body.portable = Some(Arc::new(recipe));
    Ok(body)
}

#[cfg(test)]
#[allow(
    unsafe_code,
    reason = "controlled compiler fixtures exercise authentic admission and corrupted-store refusal"
)]
pub(super) fn reconstruct_fixture(
    payload: &[u8],
    digest: ContentHash,
    spec: &BodySpec,
    registry: &QuantityRegistry,
    cancelled: &Arc<AtomicBool>,
) -> Result<AdmittedBody, MathError> {
    // SAFETY: test-only trusted compiler/store fixture, including deliberate corrupt
    // payload cases which must be refused by strict decoding; never exported.
    let permit = unsafe {
        pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
            payload.to_vec(),
            digest,
            ContentHash::from_bytes([1; 32]),
            ContentHash::from_bytes([2; 32]),
        )
    }
    .map_err(|e| MathError::Contract(e.to_string()))?;
    let body = reconstruct_portable(&permit, registry, cancelled)?;
    if !SpecWire::of(body.spec()).matches(spec) {
        return Err(MathError::Contract("fixture specification differs".into()));
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_quantity::standard::{StandardInvariantChecker, standard_registry};
    #[derive(Debug, Default)]
    struct Capture(std::sync::Mutex<Vec<Arc<AdmittedBody>>>);
    impl crate::workspace::ModelingBodyRetention for Capture {
        fn generation(&self) -> u64 {
            0
        }
        fn get(
            &self,
            _: pse_ids::roles::SemanticBodyHash,
        ) -> Result<Option<Arc<AdmittedBody>>, MathError> {
            Ok(None)
        }
        fn retain(
            &self,
            _: u64,
            _: pse_ids::roles::SemanticBodyHash,
            body: Arc<AdmittedBody>,
        ) -> Result<Arc<AdmittedBody>, MathError> {
            self.0.lock().unwrap().push(body.clone());
            Ok(body)
        }
    }
    fn authored(text: &str) -> (QuantityRegistry, AdmittedBody) {
        use crate::workspace::{CompilerContext, CompilerWorkspace, WorkspaceLimits};
        let registry = standard_registry().unwrap();
        let context = CompilerContext {
            quantities: Arc::new(registry.clone()),
            preconditions: Arc::new(
                pse_quantity::PhysicalPreconditions::new(
                    pse_quantity::generated::standard_preconditions(),
                )
                .unwrap(),
            ),
            providers: BTreeMap::new(),
        };
        let declarations = pse_authoring::language::parse(
            text,
            SemanticId::from_bytes([79; 16]),
            pse_authoring::language::IdentityPolicy::Named,
            pse_authoring::ParseBudget::default(),
        )
        .unwrap();
        let root = declarations
            .iter()
            .find(|v| v.name == "Root")
            .unwrap()
            .declaration_id;
        let capture = Arc::new(Capture::default());
        let mut workspace = CompilerWorkspace::new(context, WorkspaceLimits::default()).unwrap();
        workspace.attach_body_retention(capture.clone()).unwrap();
        workspace
            .publish_modeling(declarations, pse_modeling::PhysicalScope::default())
            .unwrap();
        workspace
            .prepare_modeling_cancellable(
                root,
                pse_modeling::InstanceId::from_id(SemanticId::NIL),
                pse_modeling::Bindings::default(),
                pse_modeling::Limits::default(),
                Arc::new(AtomicBool::new(false)),
            )
            .unwrap();
        let body = capture.0.lock().unwrap().first().unwrap().as_ref().clone();
        drop(workspace);
        drop(capture);
        (registry, body)
    }
    #[test]
    fn portable_body_local_coordinates_roundtrip_and_refuse_corrupt_or_historical_inventory() {
        let (registry, body) = authored(
            "package p { fn f(x:Scalar)->Scalar=x*x; def Root {var x:Scalar;eq e:f(x)==1;} }",
        );
        let spec = body.spec.clone();
        let payload = body.portable_payload().unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(wire["interpretation"], "pse.admitted-body.v2");
        let key = wire["functions"]
            .as_object()
            .unwrap()
            .iter()
            .find(|(_, function)| !function["admissions"].as_array().unwrap().is_empty())
            .unwrap()
            .0
            .clone();
        assert_eq!(wire["functions"][&key]["version"], 2);
        let cancel = Arc::new(AtomicBool::new(false));
        reconstruct_fixture(
            &payload,
            portable_payload_hash(&payload),
            &spec,
            &registry,
            &cancel,
        )
        .unwrap();
        for (kind, expected) in [
            ("body", "outside retained inventory"),
            ("position", "outside retained inventory"),
            ("duplicate", "duplicate function admission"),
            ("missing", "missing its admission"),
            ("finite", "outside retained inventory"),
            ("version", "unsupported function record"),
            ("interpretation", "interpretation"),
        ] {
            let mut changed = wire.clone();
            let function = &mut changed["functions"][&key];
            match kind {
                "body" | "position" => {
                    function["admissions"][0][0]["Node"][kind] = serde_json::json!(usize::MAX)
                }
                "duplicate" => {
                    let entry = function["admissions"][0].clone();
                    function["admissions"].as_array_mut().unwrap().push(entry);
                }
                "missing" => {
                    function["admissions"].as_array_mut().unwrap().clear();
                }
                "finite" => function["admissions"][0][0] = serde_json::json!("FiniteReduction"),
                "version" => function["version"] = serde_json::json!(1),
                "interpretation" => {
                    changed["interpretation"] = serde_json::json!("pse.admitted-body.v1")
                }
                _ => panic!("fixture mutation"),
            }
            let bytes = serde_json::to_vec(&changed).unwrap();
            let error = reconstruct_fixture(
                &bytes,
                portable_payload_hash(&bytes),
                &spec,
                &registry,
                &cancel,
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains(expected), "{kind}: {error}");
        }
    }
    #[test]
    fn portable_admitted_body_restores_scientific_role_and_proof_receipts_without_proving() {
        let (registry, body) = authored(
            r#"package p {
            entity kind convention {} entity convention ideal {}
            coordinate map coords(x:Length) {slot x=x/1{m};}
            reconstruction family for coords(x:Length)->Length reference ideal=1{m};
            fn law(x:Coordinate<coords.x>)->Reduced<family>=0*x;
            fn potential(x:Length)->Length=reconstruct(family,law,x);
            response response_value from potential(x:Length,potential:Fn(x:Length)->Length)->Length=potential(x);
            def Root {var x:Length;eq residual:response_value(x,potential)==0{m};}
        }"#,
        );
        let payload = body.portable_payload().unwrap();
        let spec = body.spec.clone();
        drop(body);
        let wire: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        assert!(!wire["proofs"]["entries"].as_array().unwrap().is_empty());
        assert!(
            !wire["receipts"]["boundaries"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let cancel = Arc::new(AtomicBool::new(false));
        let rebuilt = reconstruct_fixture(
            &payload,
            portable_payload_hash(&payload),
            &spec,
            &registry,
            &cancel,
        )
        .unwrap();
        let compiled = rebuilt
            .math
            .compile(
                &[0],
                &[0],
                DerivativeOrder::First,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let jet = compiled
            .worker()
            .evaluate(
                &[2.0],
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(jet.values, vec![0.0]);
        assert_eq!(jet.jacobian, vec![0.0]);
        let mut missing = wire.clone();
        missing["proofs"]["entries"].as_array_mut().unwrap().clear();
        let bytes = serde_json::to_vec(&missing).unwrap();
        assert!(
            reconstruct_fixture(
                &bytes,
                portable_payload_hash(&bytes),
                &spec,
                &registry,
                &cancel
            )
            .unwrap_err()
            .to_string()
            .contains("missing mathematical proof receipt")
        );
        let mut missing = wire;
        missing["receipts"]["boundaries"]
            .as_array_mut()
            .unwrap()
            .clear();
        let bytes = serde_json::to_vec(&missing).unwrap();
        assert!(
            reconstruct_fixture(
                &bytes,
                portable_payload_hash(&bytes),
                &spec,
                &registry,
                &cancel
            )
            .unwrap_err()
            .to_string()
            .contains("missing physical role receipt")
        );
    }
    #[test]
    fn portable_admitted_body_restores_piecewise_proof_orders_without_proving() {
        let (registry, body) = authored(
            "package p {fn f(x:Scalar)->Scalar piecewise 2=if x<0 then 0 else x*x*x;def Root {var x:Scalar;eq residual:f(x)==0;}}",
        );
        let payload = body.portable_payload().unwrap();
        let spec = body.spec.clone();
        drop(body);
        let cancel = Arc::new(AtomicBool::new(false));
        let rebuilt = reconstruct_fixture(
            &payload,
            portable_payload_hash(&payload),
            &spec,
            &registry,
            &cancel,
        )
        .unwrap();
        let compiled = rebuilt
            .math
            .compile(
                &[0],
                &[0],
                DerivativeOrder::Second,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let mut worker = compiled.worker();
        for (x, value, first, second) in [
            (-1.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0, 0.0),
            (2.0, 8.0, 12.0, 12.0),
        ] {
            let jet = worker
                .evaluate(&[x], DerivativeOrder::Second, &mut BTreeMap::new(), &cancel)
                .unwrap();
            assert_eq!(jet.values, vec![value]);
            assert_eq!(jet.jacobian, vec![first]);
            assert_eq!(jet.hessians, vec![second]);
        }
        let mut changed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        changed["proofs"]["entries"][0]["request"] =
            serde_json::to_value(ContentHash::from_bytes([3; 32])).unwrap();
        let bytes = serde_json::to_vec(&changed).unwrap();
        assert!(
            reconstruct_fixture(
                &bytes,
                portable_payload_hash(&bytes),
                &spec,
                &registry,
                &cancel
            )
            .unwrap_err()
            .to_string()
            .contains("proof request differs")
        );
    }
    fn admitted() -> (QuantityRegistry, AdmittedBody) {
        admitted_with_unused_path(None)
    }
    fn admitted_with_unused_path(unused: Option<String>) -> (QuantityRegistry, AdmittedBody) {
        pse_math::initialize().unwrap();
        let registry = standard_registry().unwrap();
        let length = match registry.physical_name("Length").unwrap() {
            pse_quantity::PhysicalName::QuantityType(id) => id,
            other => panic!("{other:?}"),
        };
        let expressions = [dsl::parse_expr("x - 2{m}").unwrap()];
        let mut formals = vec![Formal {
            path: "x".into(),
            quantity: length,
        }];
        if let Some(path) = unused {
            formals.push(Formal {
                path,
                quantity: length,
            });
        }
        let identity = ContentHash::from_bytes([1; 32]);
        let body = Request {
            definition: SemanticId::from_bytes([7; 16]),
            expressions: &expressions,
            formals: &formals,
            domains: &BTreeMap::new(),
            groups: &BTreeMap::new(),
            providers: &BTreeMap::new(),
            literals: &BTreeMap::new(),
            physical: identity,
            structure: identity,
            limits: BodyLimits::default(),
        }
        .admit_modeling_outputs(
            &registry,
            &StandardInvariantChecker,
            &Arc::new(AtomicBool::new(false)),
            &BTreeMap::new(),
            &[length],
            &BTreeMap::new(),
            &BTreeMap::new(),
        )
        .unwrap();
        (registry, body)
    }
    #[test]
    fn portable_recipe_over_wire_block_restores_exact_body_and_enforces_logical_bound() {
        let (registry, body) = admitted_with_unused_path(Some("u".repeat(3 * 1024 * 1024 + 1)));
        let payload = body.portable_payload().unwrap();
        assert!(payload.len() > 3 * 1024 * 1024);
        assert_eq!(body.portable_payload_bytes().unwrap(), payload.len());
        let cancel = Arc::new(AtomicBool::new(false));
        let restored = reconstruct_fixture(
            &payload,
            portable_payload_hash(&payload),
            body.spec(),
            &registry,
            &cancel,
        )
        .unwrap();
        assert_eq!(restored.spec(), body.spec());
        let compiled = restored
            .math()
            .compile(
                &[0],
                &[0],
                DerivativeOrder::First,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let jet = compiled
            .worker()
            .evaluate(
                &[5.0],
                DerivativeOrder::First,
                &mut BTreeMap::new(),
                &cancel,
            )
            .unwrap();
        assert_eq!(jet.values, vec![3.0]);
        assert_eq!(jet.jacobian, vec![1.0]);
        let mut corrupted = payload;
        corrupted[0] ^= 1;
        assert!(
            reconstruct_fixture(
                &corrupted,
                portable_payload_hash(&body.portable_payload().unwrap()),
                body.spec(),
                &registry,
                &cancel
            )
            .is_err()
        );
        let mut oversized = body.portable.as_ref().unwrap().as_ref().clone();
        oversized.formals[1].path = "u".repeat(MAX_PAYLOAD);
        let error = oversized.encode().unwrap_err();
        assert!(
            matches!(error, MathError::ByteLimit { required, available: MAX_PAYLOAD, .. } if required > MAX_PAYLOAD)
        );
        use pse_diagnostics::TypedDiagnostic;
        assert_eq!(
            error.diagnostic_facts().rule,
            Some(pse_diagnostics::DiagnosticRule::MathLimit)
        );
        assert!(
            error
                .diagnostic_facts()
                .observations
                .contains_key("required_bytes")
        );
    }
    #[test]
    #[allow(
        unsafe_code,
        reason = "trusted fixture mints only actual compiler-admitted bytes; forged DTOs receive no mint"
    )]
    fn portable_replay_refuses_forged_receipts_even_with_legitimate_record_authority() {
        let (_, body) = admitted();
        let payload = body.portable_payload().unwrap();
        // SAFETY: this exact payload was emitted by successful compiler admission.
        let permit = unsafe {
            pse_ids::scientific_replay::QualifiedScientificRecipe::from_canonical_product(
                payload.clone(),
                portable_payload_hash(&payload),
                ContentHash::from_bytes([1; 32]),
                ContentHash::from_bytes([2; 32]),
            )
        }
        .unwrap();
        let wire: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        let stream: pse_quantity::resolved::receipts::ReceiptStream =
            serde_json::from_value(wire["receipts"].clone()).unwrap();
        let original = wire["receipts"]["entries"][0]["result"]["result"].clone();
        let record: pse_quantity::resolved::receipts::ContractRecord =
            serde_json::from_value(original.clone()).unwrap();
        assert!(record.restore().is_err());
        let mut forged = original;
        forged["scale_bits"] = serde_json::json!(2.0_f64.to_bits());
        let forged: pse_quantity::resolved::receipts::ContractRecord =
            serde_json::from_value(forged).unwrap();
        let result: Result<(), MathError> =
            pse_quantity::resolved::receipts::replay(&stream, permit.records(), || {
                assert!(record.restore().is_ok());
                assert!(
                    forged
                        .restore()
                        .unwrap_err()
                        .to_string()
                        .contains("absent from qualified")
                );
                Err(MathError::Cancelled)
            });
        assert!(matches!(result, Err(MathError::Cancelled)));
        let forged:pse_math::typed::receipts::ProofStream=serde_json::from_value(serde_json::json!({"version":1,"entries":[{"request":ContentHash::from_bytes([9;32]),"branches":[2]}]})).unwrap();
        let mut called = false;
        let result = pse_math::typed::receipts::replay(&forged, permit.records(), || {
            called = true;
            Ok(())
        });
        assert!(!called);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("absent from qualified")
        );
        let forged: pse_quantity::resolved::receipts::ReceiptStream =
            serde_json::from_value(serde_json::json!({"version":1,"entries":[],"boundaries":[]}))
                .unwrap();
        let mut called = false;
        let result: Result<(), MathError> =
            pse_quantity::resolved::receipts::replay(&forged, permit.records(), || {
                called = true;
                Ok(())
            });
        assert!(!called);
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("absent from qualified")
        );
    }
    #[test]
    fn portable_admitted_body_reconstructs_ordinary_scalar_solve_without_inference() {
        let (registry, body) = admitted();
        let identity = pse_ids::roles::SemanticBodyHash::from(ContentHash::from_bytes([22; 32]));
        let body = body.for_semantic_identity(identity);
        let spec = body.spec.clone();
        let payload = body.portable_payload().unwrap();
        let expected = portable_payload_hash(&payload);
        // Nothing from the original admitted body, its native atoms, or its backing
        // allocation survives into reconstruction.
        drop(body);
        let cancel = Arc::new(AtomicBool::new(false));
        let rebuilt = reconstruct_fixture(&payload, expected, &spec, &registry, &cancel).unwrap();
        assert_eq!(rebuilt.spec, spec);
        assert_eq!(rebuilt.semantic_identity(), Some(identity));
        let compiled = rebuilt
            .math
            .compile(
                &[0],
                &[0],
                DerivativeOrder::First,
                Optimization::default(),
                pse_math::jets::EvaluationLimits::default(),
                &cancel,
            )
            .unwrap();
        let mut worker = compiled.worker();
        let mut x = 9.0;
        for _ in 0..3 {
            let jet = worker
                .evaluate(&[x], DerivativeOrder::First, &mut BTreeMap::new(), &cancel)
                .unwrap();
            if jet.values[0].abs() < 1e-12 {
                break;
            }
            x -= jet.values[0] / jet.jacobian[0];
        }
        assert_eq!(x, 2.0);
        let jet = worker
            .evaluate(&[x], DerivativeOrder::First, &mut BTreeMap::new(), &cancel)
            .unwrap();
        assert_eq!(jet.values, vec![0.0]);
        assert_eq!(jet.jacobian, vec![1.0]);
    }
    #[test]
    fn portable_admitted_body_missing_or_changed_receipts_refuse_without_readmission() {
        let (registry, body) = admitted();
        let spec = body.spec.clone();
        let payload = body.portable_payload().unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        assert!(
            reconstruct_fixture(
                &payload,
                ContentHash::from_bytes([8; 32]),
                &spec,
                &registry,
                &cancel
            )
            .is_err()
        );
        let mut changed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        changed["receipts"]["entries"]
            .as_array_mut()
            .unwrap()
            .clear();
        let changed = serde_json::to_vec(&changed).unwrap();
        let error = reconstruct_fixture(
            &changed,
            portable_payload_hash(&changed),
            &spec,
            &registry,
            &cancel,
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("missing concrete physical receipt"),
            "{error}"
        );
        let mut changed: serde_json::Value = serde_json::from_slice(&payload).unwrap();
        changed["receipts"]["entries"][0]["request"] =
            serde_json::to_value(ContentHash::from_bytes([9; 32])).unwrap();
        let changed = serde_json::to_vec(&changed).unwrap();
        let error = reconstruct_fixture(
            &changed,
            portable_payload_hash(&changed),
            &spec,
            &registry,
            &cancel,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("differs from qualified receipt"), "{error}");
        let mut wrong = spec.clone();
        wrong.physical = ContentHash::from_bytes([3; 32]);
        assert!(
            reconstruct_fixture(
                &payload,
                portable_payload_hash(&payload),
                &wrong,
                &registry,
                &cancel
            )
            .is_err()
        );
    }
    #[test]
    fn portable_admitted_body_literal_wire_preserves_bits_and_rejects_nonfinite() {
        let (_, body) = admitted();
        let mut recipe = body.portable.as_ref().unwrap().as_ref().clone();
        for value in [-0.0, f64::MIN_POSITIVE, f64::from_bits(1), f64::MAX] {
            recipe.expressions[0] = Expr {
                kind: ExprKind::Number(dsl::Number {
                    value,
                    exact_integer: None,
                    unit: None,
                }),
                span: dsl::Span::default(),
            };
            let bytes = recipe.encode().unwrap();
            let decoded: Recipe = serde_json::from_slice(&bytes).unwrap();
            let ExprKind::Number(number) = &decoded.expressions[0].kind else {
                panic!("literal changed")
            };
            assert_eq!(number.value.to_bits(), value.to_bits());
        }
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            recipe.expressions[0] = Expr {
                kind: ExprKind::Number(dsl::Number {
                    value,
                    exact_integer: None,
                    unit: None,
                }),
                span: dsl::Span::default(),
            };
            assert!(recipe.encode().is_err());
        }
    }
}
