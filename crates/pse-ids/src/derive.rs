// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Derived identity: the keyed, framed hashing of blueprint §5.1.
//!
//! `blake3_128(context ‖ parts)` in the blueprint denotes exactly this:
//!
//! 1. `blake3::Hasher::new_derive_key(context)` — a *keyed* hash, not a plain one.
//!    `derive_key` and a truncated plain hash produce different bytes, so the choice is
//!    frozen together with the context spellings of the [`Frame`] catalog.
//! 2. Every part is framed as its `u64` little-endian byte length followed by its bytes.
//!    IDs, hashes and integers are parts too and carry a length like everything else, so
//!    `["ab", "c"]` and `["a", "bc"]` cannot collide.
//! 3. A 128-bit identity is the first 16 bytes of `finalize_xof()`: a defined digest of
//!    BLAKE3's extendable output, not a truncation of an unrelated hash. A 256-bit hash is
//!    `finalize()`.
//!
//! Consequence, and the reason any of this is worth the framing: re-running compilation on
//! an unchanged snapshot reproduces identical IDs, and changing mesh resolution changes
//! only the mesh-dependent ones.

use crate::id::{ContentHash, SemanticId};

/// Declares the frame catalog from a single table: one variant per `derive_key` context,
/// its exact spelling and its meaning (the variant's documentation), grouped by the area
/// and crates that derive under it. The generated reference lists the same table.
macro_rules! frames {
    ($(
        $area:literal ($owners:literal) {
            $( $(#[doc = $doc:literal])* $variant:ident => $spelling:literal, )*
        }
    )*) => {
        /// Every `derive_key` context in the workspace (blueprint §5.1, §5.3, ADR-0050,
        /// ADR-0115 Outcome 4).
        ///
        /// A context spelling is part of the identity contract: changing one changes every
        /// identity derived under it, so a new meaning or a new derivation version is a new
        /// variant, never a new spelling for an existing one. The version suffix of a
        /// spelling (and of its variant) versions the *derivation*, not a relation schema.
        ///
        /// [`FramedHasher::new`], [`derive_id`], [`derive_hash`] and the keyed
        /// [`crate::preimage`] entry points take a `Frame`, so every context is declared
        /// here and nowhere else. `docs/generated/frames.md` lists the catalog.
        ///
        /// ```
        /// use pse_ids::Frame;
        ///
        /// assert_eq!(Frame::NamedV1.as_str(), "pse:named:v1");
        /// assert!(Frame::ALL.contains(&Frame::RegistryV1));
        /// assert_eq!(Frame::FitCoordinateV1.area(), "runtime and workflows");
        /// ```
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Frame {
            $($( $(#[doc = $doc])* $variant, )*)*
        }

        impl Frame {
            /// Every frame, in declaration order.
            pub const ALL: &'static [Self] = &[$($(Self::$variant,)*)*];

            /// The exact `derive_key` context spelling of this frame.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $($(Self::$variant => $spelling,)*)*
                }
            }

            /// The area that derives under this frame, which groups the catalog.
            #[must_use]
            pub const fn area(self) -> &'static str {
                match self {
                    $($(Self::$variant => $area,)*)*
                }
            }

            /// The crates that derive under this frame's area.
            #[must_use]
            pub const fn owners(self) -> &'static str {
                match self {
                    $($(Self::$variant => $owners,)*)*
                }
            }

            /// What is derived under this frame: the variant's documentation on one line.
            #[must_use]
            pub const fn meaning(self) -> &'static str {
                match self {
                    $($(Self::$variant => concat!($($doc),*).trim_ascii(),)*)*
                }
            }
        }
    };
}

frames! {
    "identity, registry and platform" ("pse-ids, pse-schema, pse-columnar, pse-model") {
        /// Named-policy entity identity: `named_id(package_id, qualified_name)`.
        NamedV1 => "pse:named:v1",
        /// Registry identity and the registry fingerprint.
        RegistryV1 => "pse:registry:v1",
        /// An engine settings or profile digest (blueprint §14.3, §23.2).
        SettingsV1 => "pse:settings:v1",
        /// The digest of the build inputs recorded as build provenance.
        BuildInputsV1 => "pse:build-inputs:v1",
        /// A stored artifact descriptor's identity.
        ArtifactDescriptorV1 => "pse:artifact-descriptor:v1",
        /// A field-directed native value payload digest.
        NativeValuePayloadV1 => "pse:native-value-payload:v1",
        /// Durable semantic row tokens over native values; also the published row-key encoding.
        RowKeyNativeValuesV2 => "pse:row-key:native-values:v2",
        /// A generated typed fact batch, including relation and empty membership.
        TypedFactsV1 => "pse:typed-facts:v1",
        /// A generated typed fact row's primary-key lookup bucket.
        TypedRowKeyV1 => "pse:typed-row-key:v1",
        /// A relation's complete local semantic description.
        SchemaSemanticRelationV2 => "pse.schema.semantic-relation.v2",
        /// A semantic contract over a support closure.
        SchemaSemanticProductV2 => "pse.schema.semantic-product.v2",
        /// A profile's requirements and requested support closure.
        SchemaSemanticProfileV2 => "pse.schema.semantic-profile.v2",
        /// The exact observed native field layout.
        SchemaExecutionEncodingV1 => "pse.schema.execution-encoding.v1",
        /// An effective numerical policy request.
        NumericalPolicyV1 => "pse.numerical.policy.v1",
        /// A kernel provider's physical, algorithm and data identity.
        ProviderV4 => "pse.provider.v4",
        /// A kernel provider's configuration key framed with its checked output envelope.
        ProviderConfigurationV1 => "pse.provider.configuration.v1",
    }

    "physical typing" ("pse-quantity") {
        /// A unit product's identity over its canonical atomic factors and rational
        /// exponents, independent of how the product was spelled (ADR-0124).
        QuantityUnitProductV1 => "pse.quantity.unit-product.v1",
    }

    "structure" ("pse-structural") {
        /// A flowsheet projection over canonicalized inventories.
        FlowProjectionV1 => "pse.flow.projection.v1",
        /// A structural block's canonical membership.
        StructuralBlockV1 => "pse.structural.block.v1",
        /// A structural analysis scope, independent of traversal order.
        StructuralScopeV1 => "pse.structural.scope.v1",
    }

    "mathematics" ("pse-math") {
        /// An expression body key, never over printed atoms or process-global identifiers.
        MathBodyV1 => "pse.math.body.v1",
        /// A bound case's structure: bindings, units, inventories, class declarations, the
        /// requirements its lowerings place on the route, and its objectives with their
        /// lexicographic degradations.
        MathCaseStructureV5 => "pse.math.case-structure.v5",
        /// A bound case's values, separate from structure and prepared arithmetic.
        MathCaseValuesV1 => "pse.math.case-values.v1",
        /// The compiler-owned guarded-real interpretation policy.
        MathGuardedRealV1 => "pse.math.guarded-real.v1",
        /// Shared bound facts of a presolve analysis.
        MathBoundFactsV2 => "pse.math.bound-facts.v2",
        /// Coefficient extraction assumptions over shared bound facts.
        MathCoefficientAssumptionsV1 => "pse.math.coefficient-assumptions.v1",
        /// A quadratic matrix and orientation certified by an exact Gram representation.
        MathGramV2 => "pse.math.gram.v2",
        /// A numerical convexity assessment of an admitted snapshot.
        MathConvexityV1 => "pse.math.convexity.v1",
        /// The convexity fact of a prepared problem (ADR-0121).
        MathConvexityFactV1 => "pse.math.convexity-fact.v1",
        /// The mathematics environment projection.
        MathEnvironmentV1 => "pse.math.environment.v1",
        /// A factorable decomposition.
        MathFactorableV1 => "pse.math.factorable.v1",
        /// A normalization, separate from native algorithmic scaling.
        MathNormalizationV1 => "pse.math.normalization.v1",
        /// Value and guard assumptions projected through a normalization.
        MathNormalizedFactsV1 => "pse.math.normalized-facts.v1",
        /// A canonical sparse pattern, excluding values.
        SparsePatternV1 => "pse.sparse.pattern.v1",
        /// Projected numerical magnitudes.
        NumericalProjectionV1 => "pse.numerical.projection.v1",
        /// Projected magnitudes with characteristic scales in target unit coordinates.
        NumericalProjectionV2 => "pse.numerical.projection.v2",
        /// Resolved magnitudes projected to an admitted subtraction result.
        NumericalDifferenceProjectionV1 => "pse.numerical.difference-projection.v1",
        /// Resolved numerical targets.
        NumericalResolvedV1 => "pse.numerical.resolved.v1",
        /// An implicit block's configuration.
        ImplicitConfigurationV1 => "pse.implicit.configuration.v1",
        /// An implicit block's fixed configuration.
        ImplicitFixedConfigurationV1 => "pse.implicit.fixed-configuration.v1",
        /// An implicit block's regime configuration.
        ImplicitRegimeConfigurationV1 => "pse.implicit.regime-configuration.v1",
        /// An inner solver implementation's versioned capability name.
        InnerSolverV1 => "pse.inner-solver.v1",
    }

    "compilation" ("pse-compiler") {
        /// A compiled mathematics artifact request.
        MathArtifactV4 => "pse.math.artifact.v4",
        /// A local expression occurrence.
        MathLocalOccurrenceV2 => "pse.math.local-occurrence.v2",
        /// Checked expression occurrences and complete contextual admission products.
        MathLocalOccurrenceV3 => "pse.math.local-occurrence.v3",
        /// Retained physical inference selections, operand conversions and numerical scales.
        MathResolvedAdmissionsV1 => "pse.math.resolved-admissions.v1",
        /// Checked expression occurrences and complete contextual admission products.
        MathResolvedAdmissionsV2 => "pse.math.resolved-admissions.v2",
        /// A mathematical body including its retained physical admission product.
        MathBodyV2 => "pse.math.body.v2",
        /// Checked expression occurrences and complete contextual admission products.
        MathBodyV3 => "pse.math.body.v3",
        /// A physical inventory, including unit compositions and derived-kind definitions
        /// (ADR-0124), and the names and typed conditions its quantity types and reference
        /// states are addressed by (ADR-0123 Outcome 6).
        MathPhysicalInventoryV5 => "pse.math.physical-inventory.v5",
        /// Selected physical declaration inventory with explicit lengths for every
        /// nested collection, including unit factors and reduction domains (Plan 25a/I1).
        MathPhysicalInventoryV6 => "pse.math.physical-inventory.v6",
        /// Admitted owner-relative physical transformation specialized at an occurrence.
        ModelingPhysicalOperationV1 => "pse.modeling.physical-operation.v1",
        /// A physical reduction pass.
        MathPhysicalPassV1 => "pse.math.physical-pass.v1",
        /// A typed definition's admitted outputs; function validity predicates and the
        /// rejecting data-layer guards, each with its layer, the declaration stating it and
        /// its lineage (the parameter sets and arguments it reads), are framed in their
        /// canonical spelling, unit literals by their canonical products (ADR-0123 Outcomes 4
        /// and 8, Plan 23 H5).
        MathTypedDefinitionV5 => "pse.math.typed-definition.v5",
        /// A typed definition including the retained resolved physical admissions.
        MathTypedDefinitionV6 => "pse.math.typed-definition.v6",
        /// Checked expression occurrences and complete contextual admission products.
        MathTypedDefinitionV7 => "pse.math.typed-definition.v7",
        /// Scientific applicability effect stages, selected records and named permissions.
        MathTypedDefinitionV8 => "pse.math.typed-definition.v8",
        /// Scientific parameter source prerequisites retained through numeric input lowering.
        MathTypedDefinitionV9 => "pse.math.typed-definition.v9",
        /// A prepared view of a compiled modeling structure.
        CompilerModelingViewV2 => "pse.compiler.modeling-view.v2",
        /// The parametric projection of a prepared modeling view over requested parameters.
        CompilerModelingParametricV1 => "pse.compiler.modeling-parametric.v1",
        /// A grouped consumer body, its expressions and validity ranges in canonical spelling
        /// (ADR-0123 Outcome 8).
        ModelingConsumerBodyV2 => "pse.modeling.consumer-body.v2",
        /// An implicit residual, its terms, hints and guards in canonical spelling, function
        /// validity and data-layer guards with their envelopes, policies and lineages included
        /// (ADR-0123 Outcomes 4 and 8, Plan 23 H5).
        ModelingImplicitResidualV4 => "pse.modeling.implicit-residual.v4",
    }

    "modeling specialization" ("pse-modeling") {
        /// A continuity derivative, its body in canonical spelling (ADR-0123 Outcome 8).
        ModelingContinuityV2 => "pse.modeling.continuity.v2",
        /// A coordinate, preserving compound-key identity; an enumeration member is framed
        /// by its identity, not its name (ADR-0123 Outcome 2).
        ModelingCoordinateV2 => "pse.modeling.coordinate.v2",
        /// A definite integral, its integrand in canonical spelling (ADR-0123 Outcome 8).
        ModelingDefiniteIntegralV2 => "pse.modeling.definite-integral.v2",
        /// A dispatch group body, its expressions and equations in canonical spelling
        /// (ADR-0123 Outcome 8).
        ModelingDispatchBodyV2 => "pse.modeling.dispatch-body.v2",
        /// Dispatch bodies retaining full physical contracts and nominal contextual roles.
        ModelingDispatchBodyV3 => "pse.modeling.dispatch-body.v3",
        /// Checked expression occurrences and complete contextual admission products.
        ModelingDispatchBodyV4 => "pse.modeling.dispatch-body.v4",
        /// Checked scientific record identities and dependency closure products.
        ModelingDispatchBodyV5 => "pse.modeling.dispatch-body.v5",
        /// Dispatch bodies including generated parameter-read prerequisite contracts.
        ModelingDispatchBodyV6 => "pse.modeling.dispatch-body.v6",
        /// A finite function specialization, its validity, data-layer guards with their
        /// envelopes and selected policies, each with the parameter sets and arguments it
        /// reads, and body in canonical spelling, and its static arguments, enumeration
        /// members by identity (ADR-0123 Outcomes 2, 4 and 8, Plan 23 H5).
        ModelingFiniteFunctionV5 => "pse.modeling.finite-function.v5",
        /// Finite physical function with full anonymous contracts and contextual authority.
        ModelingFiniteFunctionV6 => "pse.modeling.finite-function.v6",
        /// Checked expression occurrences and complete contextual admission products.
        ModelingFiniteFunctionV7 => "pse.modeling.finite-function.v7",
        /// Checked scientific record identities and dependency closure products.
        ModelingFiniteFunctionV8 => "pse.modeling.finite-function.v8",
        /// Finite functions including generated parameter-read prerequisite contracts.
        ModelingFiniteFunctionV9 => "pse.modeling.finite-function.v9",
        /// A demanded numeric parameter read with its lexical source and prerequisites.
        ModelingParameterReadV1 => "pse.modeling.parameter-read.v1",
        /// Declared claim, form, actual selected identities and numerical arguments.
        ModelingApplicabilityCallV1 => "pse.modeling.applicability-call.v1",
        /// One source claim application at its actual instance and typed physical inputs.
        ModelingApplicabilityObservationV1 => "pse.modeling.applicability-observation.v1",
        /// One source claim application with its owner lineage, requirement and typed inputs.
        ModelingApplicabilityObservationV2 => "pse.modeling.applicability-observation.v2",
        /// Explicit whole-interval coverage of the same declared interval claim.
        ModelingApplicabilityCoverageV1 => "pse.modeling.applicability-coverage.v1",
        /// A finite reduction rewrite.
        ModelingFiniteReductionV1 => "pse.modeling.finite-reduction.v1",
        /// Checked expression occurrences and complete contextual admission products.
        ModelingFiniteReductionV2 => "pse.modeling.finite-reduction.v2",
        /// A keyed entity's identity: its key-declaring kind and its ordered, typed key
        /// values, defaults included. The concrete refinement is content, not identity
        /// (ADR-0123 Outcome 2).
        ModelingKeyedEntityV1 => "pse.modeling.keyed-entity.v1",
        /// A specialized member.
        ModelingMemberV1 => "pse.modeling.member.v1",
        /// A process state slot: its owner, declared role and canonical typed semantic indices.
        ModelingProcessSlotV1 => "pse.modeling.process-slot.v1",
        /// A realized mesh coordinate.
        ModelingMeshCoordinateV1 => "pse.modeling.mesh-coordinate.v1",
        /// A table row's identity: its table and its ordered, canonical typed key values
        /// (Plan 23 H5), framed as a keyed entity's are.
        ModelingTableRowV1 => "pse.modeling.table-row.v1",
        /// A typed constant.
        ModelingTypedConstantV1 => "pse.modeling.typed-constant.v1",
    }

    "native solvers" ("pse-backend-native") {
        /// Complete backend settings, derived from serde.
        BackendSettingsV4 => "pse.backend.settings.v4",
        /// A cone sequence layout in the pse encoding.
        ConeLayoutV3 => "pse.cone.layout.v3",
        /// The lower-bound cone row of a two-sided coefficient row lowered to cone form.
        ConeLoweredRowV1 => "pse.cone.lowered-row.v1",
        /// A recognized convex program's cone form: its auxiliary columns, atom rows and
        /// contract (ADR-0121).
        ConeRecognizedV1 => "pse.cone.recognized.v1",
        /// A factorable problem's variable domains.
        FactorableDomainV1 => "pse.factorable.domain.v1",
        /// The continuous problem of a fixed discrete assignment.
        FactorableFixedAssignmentV1 => "pse.factorable.fixed-assignment.v1",
        /// The profile of a fixed discrete assignment's continuous solve.
        FactorableFixedAssignmentProfileV1 => "pse.factorable.fixed-assignment.profile.v1",
        /// A HiGHS irreducible-infeasible-subsystem relaxation.
        HighsIisRelaxationV1 => "pse.highs.iis-relaxation.v1",
        /// A Jacobian diagnostic problem family's pattern and domains.
        JacobianDiagnosticLayoutV1 => "pse.jacobian-diagnostic.layout.v1",
        /// A Jacobian diagnostic problem.
        JacobianDiagnosticProblemV1 => "pse.jacobian-diagnostic.problem.v1",
        /// Resolved native accuracy budgets.
        NativeAccuracyV3 => "pse.native.accuracy.v3",
        /// Every linked adapter's native build.
        NativeBuildV1 => "pse.native.build.v1",
        /// Native solve controls.
        NativeControlsV2 => "pse.native.controls.v2",
        /// The linked Ipopt build.
        NativeIpoptBuildV1 => "pse.native.ipopt.build.v1",
        /// A seed's content, without its execution origin.
        NativeSeedV2 => "pse.native.seed.v2",
        /// A structural analysis scope over a residual Jacobian.
        NativeStructuralScopeV1 => "pse.native.structural-scope.v1",
        /// Presolve options.
        PresolvePolicyV1 => "pse.presolve.policy.v1",
        /// A presolve transformation.
        PresolveTransformationV3 => "pse.presolve.transformation.v3",
        /// The constraint system a SCIP reoptimization session was built for.
        ScipReoptimizationSystemV1 => "pse.scip.reoptimization.system.v1",
        /// A shooting window's oracle: its anchored starts and observed quadratures.
        ShootingWindowV1 => "pse.shooting.window.v1",
        /// A tear-selection decision column.
        TearDecisionV1 => "pse.tear.decision.v1",
        /// A tear-selection order column.
        TearOrderV1 => "pse.tear.order.v1",
    }

    "runtime and workflows" ("pse-runtime") {
        /// A conditional strategy's causal map.
        CausalMapV2 => "pse.causal-map.v2",
        /// The environment a completed step actually ran in.
        CompletedEnvironmentV1 => "pse.completed.environment.v1",
        /// A completed step's request lineage.
        CompletedRequestV2 => "pse.completed.request.v2",
        /// A durable rolling horizon's request: its plant, loop and stage templates (Plan 22
        /// Y5c).
        DurableHorizonRequestV1 => "pse.durable.horizon_request.v1",
        /// A durable job request.
        DurableJobRequestV2 => "pse.durable.job_request.v2",
        /// A durable modeling request.
        DurableModelingRequestV1 => "pse.durable.modeling_request.v1",
        /// A durable study's request: its definition (Plan 22 O7).
        DurableStudyRequestV1 => "pse.durable.study_request.v1",
        /// The value bindings of one durable study point (Plan 22 O7).
        DurableStudyPointBindingV1 => "pse.durable.study_point_binding.v1",
        /// A dynamic simulation profile, with its scheduled inputs and typed sensitivity; its
        /// IDAS settings carry no sign constraints.
        DynamicProfileV6 => "pse.dynamic.profile.v6",
        /// An explicit conic request; its quadratic is certified exactly, so it carries no
        /// Gram witness.
        ExplicitConicV4 => "pse.explicit-conic.v4",
        /// A fitting coordinate alias of an experiment and source.
        FitCoordinateV1 => "pse.fit.coordinate.v1",
        /// A prepared fit.
        FitPreparedV1 => "pse.fit.prepared.v1",
        /// A fit profile: its solver profile, rank tolerance, cell budget, derivative source,
        /// simulation profiles and requested intervals.
        FitProfileV3 => "pse.fit.profile.v3",
        /// A fit source.
        FitSourceV1 => "pse.fit.source.v1",
        /// Compiled simulation modes.
        ModelingDynamicModesV1 => "pse.modeling.dynamic-modes.v1",
        /// A modeling dynamic simulation, with the derivative order of its functions, its
        /// integration parameter values, event directions and state signs.
        ModelingDynamicV2 => "pse.modeling.dynamic.v2",
        /// A modeling fit's execution.
        ModelingFitExecutionV1 => "pse.modeling.fit-execution.v1",
        /// A modeling fit's source.
        /// Typed measured records belong to the admitted source revision; fitting selections are framed separately.
        ModelingFitSourceV2 => "pse.modeling.fit-source.v2",
        /// Historical fitting source with external observations; retained only as an immutable frame vocabulary.
        ModelingFitSourceV1 => "pse.modeling.fit-source.v1",
        /// Implicit trial hints.
        ModelingImplicitTrialHintsV1 => "pse.modeling.implicit-trial-hints.v1",
        /// A modeling source revision: the structured declaration rows, each package data
        /// document's identity and byte-level content hash, the physical inventory identity
        /// and the admitted physical name bindings (ADR-0123 Outcome 8, ADR-0125).
        ModelingSourceRevisionV3 => "pse.modeling.source-revision.v3",
        /// Source revision over temporal-composition declaration schema (ADR-0132).
        ModelingSourceRevisionV4 => "pse.modeling.source-revision.v4",
        /// A prepared numerical cone request.
        NumericalConeV1 => "pse.numerical.cone.v1",
        /// A publication request's algorithms.
        RunAlgorithmsV1 => "pse.run.algorithms.v1",
        /// A publication request's source.
        RunSourceV1 => "pse.run.source.v1",
        /// A publication request's target.
        RunTargetV1 => "pse.run.target.v1",
        /// A shooting NLP: its simulation, method, nodes, controls, rows and objective.
        ShootingProblemV1 => "pse.shooting.problem.v1",
        /// A solve's compilation and normalization, before a seed is attached.
        SolvePreparationV1 => "pse.solve.preparation.v1",
        /// A complete selected solve request.
        SolveRequestV1 => "pse.solve.request.v1",
        /// The preparation a stored seed is keyed by.
        SolveSeedPreparationV1 => "pse.solve.seed_preparation.v1",
        /// Conic solver data.
        SolverConicDataV1 => "pse.solver.conic-data.v1",
        /// A conic solver layout.
        SolverConicLayoutV3 => "pse.solver.conic-layout.v3",
        /// A conic solver session.
        SolverConicSessionV1 => "pse.solver.conic-session.v1",
        /// A solver session's coordinates.
        SolverCoordinatesV1 => "pse.solver.coordinates.v1",
        /// A solver session's data.
        SolverDataV1 => "pse.solver.data.v1",
        /// A complete effective solver request profile.
        SolverProfileV3 => "pse.solver.profile.v3",
        /// A native solver session's compatibility.
        SolverSessionV1 => "pse.solver.session.v1",
    }

    "operational store" ("pse-operations, pse-codegen") {
        /// The operational store's schema fingerprint over the generated `schema.sql` and
        /// `physical.sql` (ADR-0114 Outcome 23). Added after the catalog was captured.
        OpsSchemaV1 => "pse.ops.schema.v1",
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Frames one part into a hasher: `u64` little-endian length, then the bytes.
///
/// `usize` is at most 64 bits on every supported target, so the saturating fallback is
/// unreachable; it exists because the crate's panic policy has no room for an `expect`
/// that only a 128-bit address space could reach.
fn put_part(hasher: &mut blake3::Hasher, part: &[u8]) {
    crate::frame::FrameSink::put_len_prefixed(hasher, part);
}

/// Derives a 128-bit semantic ID under `frame` from framed `parts` (blueprint §5.1).
///
/// ```
/// use pse_ids::{derive_id, Frame, SemanticId};
///
/// // Framing is what keeps a split from being a collision.
/// assert_ne!(
///     derive_id(Frame::NamedV1, &[b"ab", b"c"]),
///     derive_id(Frame::NamedV1, &[b"a", b"bc"]),
/// );
/// ```
pub fn derive_id(frame: Frame, parts: &[&[u8]]) -> SemanticId {
    let mut hasher = blake3::Hasher::new_derive_key(frame.as_str());
    for part in parts {
        put_part(&mut hasher, part);
    }
    let mut bytes = [0_u8; SemanticId::WIDTH];
    hasher.finalize_xof().fill(&mut bytes);
    SemanticId::from_bytes(bytes)
}

/// Derives a 256-bit content hash under `frame` from framed `parts`.
///
/// Used for registry fingerprints, stage keys, settings digests and math IR node hashes —
/// every keyed digest that is not an entity identity.
pub fn derive_hash(frame: Frame, parts: &[&[u8]]) -> ContentHash {
    let mut hasher = blake3::Hasher::new_derive_key(frame.as_str());
    for part in parts {
        put_part(&mut hasher, part);
    }
    ContentHash::from_bytes(*hasher.finalize().as_bytes())
}

/// An incremental [`derive_id`] / [`derive_hash`] for callers that build a preimage from
/// heterogeneous components.
///
/// Every method contributes exactly one framed part, so a `FramedHasher` sequence and the
/// equivalent `parts` slice produce the same digest. Integers are contributed as their
/// little-endian bytes *inside* a length-prefixed part, which is what keeps this
/// equivalent to the slice form rather than a second, subtly different framing.
///
/// ```
/// use pse_ids::{derive_hash, Frame, FramedHasher};
///
/// let mut hasher = FramedHasher::new(Frame::SettingsV1);
/// hasher.str("target_partitions").u64(8);
///
/// let expected = derive_hash(Frame::SettingsV1, &[b"target_partitions", &8_u64.to_le_bytes()]);
/// assert_eq!(hasher.finish_hash(), expected);
/// ```
#[derive(Clone, Debug)]
pub struct FramedHasher {
    hasher: blake3::Hasher,
}

impl FramedHasher {
    /// Opens a hasher keyed with `frame`.
    pub fn new(frame: Frame) -> Self {
        Self {
            hasher: blake3::Hasher::new_derive_key(frame.as_str()),
        }
    }

    /// One part: the bytes, length-prefixed.
    pub fn part(&mut self, bytes: &[u8]) -> &mut Self {
        put_part(&mut self.hasher, bytes);
        self
    }

    /// One part: the UTF-8 bytes of `text`, length-prefixed.
    pub fn str(&mut self, text: &str) -> &mut Self {
        self.part(text.as_bytes())
    }

    /// One part: two little-endian bytes, length-prefixed.
    pub fn u16(&mut self, v: u16) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: four little-endian bytes, length-prefixed.
    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: eight little-endian bytes, length-prefixed.
    pub fn u64(&mut self, v: u64) -> &mut Self {
        self.part(&v.to_le_bytes())
    }

    /// One part: a single byte, `1` for true and `0` for false, length-prefixed.
    pub fn bool(&mut self, v: bool) -> &mut Self {
        self.part(&[u8::from(v)])
    }

    /// One part: the sixteen raw bytes of an identity, length-prefixed.
    pub fn id(&mut self, id: &SemanticId) -> &mut Self {
        self.part(id.as_bytes())
    }

    /// One part: the thirty-two raw bytes of a digest, length-prefixed.
    pub fn hash(&mut self, hash: &ContentHash) -> &mut Self {
        self.part(hash.as_bytes())
    }

    /// Finishes as a 256-bit content hash.
    pub fn finish_hash(self) -> ContentHash {
        ContentHash::from_bytes(*self.hasher.finalize().as_bytes())
    }

    /// Finishes as a 128-bit semantic ID: the first sixteen bytes of the extendable output.
    pub fn finish_id(self) -> SemanticId {
        let mut bytes = [0_u8; SemanticId::WIDTH];
        self.hasher.finalize_xof().fill(&mut bytes);
        SemanticId::from_bytes(bytes)
    }
}

/// The identity of a named-policy entity: `blake3_128("pse:named:v1" ‖ package_id ‖ name)`.
///
/// Named policy is for reference packages whose qualified names *are* the public contract
/// (units, elements, constants, property kinds, standard templates and methods). Under it
/// a rename is by definition a new entity, which is why `change_ops.op = rename` is
/// rejected for such a package and the old name becomes a `reference.aliases` row.
///
/// ```
/// use pse_ids::{named_id, SemanticId};
///
/// let package = named_id(SemanticId::NIL, "pse.schema");
/// // A rename under the named policy is a different entity, by construction.
/// assert_ne!(named_id(package, "unit:kelvin"), named_id(package, "unit:degK"));
/// ```
pub fn named_id(package_id: SemanticId, qualified_name: &str) -> SemanticId {
    derive_id(
        Frame::NamedV1,
        &[package_id.as_bytes(), qualified_name.as_bytes()],
    )
}

#[cfg(test)]
mod consolidation_unit {
    use super::*;

    const A: SemanticId = SemanticId::from_bytes([0x01; 16]);

    #[test]
    fn frame_spellings_unique() {
        let mut seen = std::collections::BTreeMap::new();
        for frame in Frame::ALL {
            if let Some(previous) = seen.insert(frame.as_str(), *frame) {
                panic!("{previous:?} and {frame:?} share the spelling {frame}");
            }
        }
        assert_eq!(seen.len(), Frame::ALL.len());
        // `ALL` lists each variant once, so its length is the variant count.
        let variants: std::collections::BTreeSet<_> = Frame::ALL.iter().collect();
        assert_eq!(variants.len(), Frame::ALL.len());
    }

    /// The generated catalog lists each frame with its area and meaning, so every
    /// variant is documented, and an area's frames are declared together.
    #[test]
    fn frame_catalog_documents_every_frame() {
        let mut areas = Vec::new();
        for frame in Frame::ALL {
            assert!(!frame.meaning().is_empty(), "{frame:?} has no meaning");
            assert_eq!(frame.meaning(), frame.meaning().trim(), "{frame:?}");
            assert!(!frame.owners().is_empty(), "{frame:?} has no owners");
            if areas.last() != Some(&frame.area()) {
                assert!(!areas.contains(&frame.area()), "{frame:?} splits its area");
                areas.push(frame.area());
            }
        }
        assert_eq!(
            Frame::RowKeyNativeValuesV2.meaning(),
            "Durable semantic row tokens over native values; also the published row-key encoding."
        );
        assert_eq!(Frame::OpsSchemaV1.owners(), "pse-operations, pse-codegen");
    }

    #[test]
    fn framing_distinguishes_a_split_that_concatenation_would_not() {
        assert_ne!(
            derive_id(Frame::RegistryV1, &[b"ab", b"c"]),
            derive_id(Frame::RegistryV1, &[b"a", b"bc"])
        );
        assert_ne!(
            derive_hash(Frame::RegistryV1, &[b"ab", b"c"]),
            derive_hash(Frame::RegistryV1, &[b"a", b"bc"])
        );
    }

    #[test]
    fn an_empty_part_is_not_no_part() {
        assert_ne!(
            derive_hash(Frame::RegistryV1, &[b"a", b""]),
            derive_hash(Frame::RegistryV1, &[b"a"])
        );
    }

    #[test]
    fn a_derived_id_is_the_first_sixteen_bytes_of_the_extendable_output() {
        let parts: &[&[u8]] = &[b"pse.schema", b"relation:authored.packages@1"];
        let mut hasher = blake3::Hasher::new_derive_key(Frame::NamedV1.as_str());
        for part in parts {
            put_part(&mut hasher, part);
        }
        let mut expected = [0_u8; 16];
        hasher.finalize_xof().fill(&mut expected);

        assert_eq!(derive_id(Frame::NamedV1, parts).as_bytes(), &expected);
    }

    #[test]
    fn a_changed_context_changes_every_derived_value() {
        let parts: &[&[u8]] = &[b"x"];
        assert_ne!(
            derive_id(Frame::TearOrderV1, parts),
            derive_id(Frame::TearDecisionV1, parts)
        );
        assert_ne!(
            derive_hash(Frame::RegistryV1, parts),
            derive_hash(Frame::SettingsV1, parts)
        );
    }

    #[test]
    fn a_keyed_digest_is_not_a_plain_one() {
        let mut plain = blake3::Hasher::new();
        put_part(&mut plain, b"x");
        assert_ne!(
            derive_hash(Frame::RegistryV1, &[b"x"]).as_bytes(),
            plain.finalize().as_bytes()
        );
    }

    #[test]
    fn the_framed_hasher_equals_the_slice_form_on_the_same_parts() {
        let mut framed = FramedHasher::new(Frame::RegistryV1);
        framed
            .str("P2")
            .u32(1)
            .id(&A)
            .hash(&ContentHash::from_bytes([0x09; 32]))
            .u64(64)
            .u16(7)
            .bool(true)
            .part(b"tail");

        let expected = derive_hash(
            Frame::RegistryV1,
            &[
                b"P2",
                &1_u32.to_le_bytes(),
                A.as_bytes(),
                &[0x09; 32],
                &64_u64.to_le_bytes(),
                &7_u16.to_le_bytes(),
                &[1_u8],
                b"tail",
            ],
        );
        assert_eq!(framed.finish_hash(), expected);
    }

    #[test]
    fn the_framed_hasher_and_derive_id_agree_on_identity_too() {
        let mut framed = FramedHasher::new(Frame::NamedV1);
        framed.id(&SemanticId::NIL).str("pse.schema");
        assert_eq!(framed.finish_id(), named_id(SemanticId::NIL, "pse.schema"));
    }

    #[test]
    fn a_derived_id_is_the_prefix_of_the_derived_hash_under_the_same_context() {
        let parts: &[&[u8]] = &[b"a", b"bc"];
        let id = derive_id(Frame::RegistryV1, parts);
        let hash = derive_hash(Frame::RegistryV1, parts);
        assert_eq!(id.as_bytes(), &hash.as_bytes()[..SemanticId::WIDTH]);
    }
}
