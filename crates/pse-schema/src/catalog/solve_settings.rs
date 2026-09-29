// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The vocabularies of the backend and dynamics settings documents and of the solve
//! controls (ADR-0115 Outcome 3, ADR-0116 Outcome 2).
//!
//! Every enumeration a settings document carries is declared here, with exactly the
//! spelling its serde encoding already had, so the settings documents crossing the Python
//! boundary keep their bytes. A member's native option value, integer code or linked-library
//! mask is adapter knowledge and stays in `pse-backend-native`; the registry owns only the
//! vocabulary. Settings whose variants carry parameters (the Ipopt linear solver with its
//! orderings, KINSOL's linear solver and forcing term, the IDAS linear solver, the restart
//! barrier) are typed structures of their adapters, not vocabularies.

use crate::builder::RegistryBuilder;
use crate::model::{EnumDecl, EnumMember};

/// Declares every settings vocabulary.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_ipopt(builder);
    declare_controls(builder);
    declare_backend_methods(builder);
    declare_feral(builder);
    declare_presolve(builder);
    declare_dynamics(builder);
}

/// FERAL's fill-reducing orderings and matrix scalings in the POUNCE settings (Plan 22 B5).
/// FERAL's caller-supplied permutation and scaling vector are problem data of one KKT
/// dimension, not settings, so they are not members.
fn declare_feral(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "FeralOrdering",
        &[
            ("amd", "Approximate minimum degree."),
            ("amf", "Approximate minimum fill."),
            ("metis_nd", "METIS multilevel nested dissection."),
            ("scotch_nd", "SCOTCH nested dissection."),
            ("kahip_nd", "KaHIP nested dissection."),
            ("auto", "FERAL's size- and shape-based choice."),
            (
                "auto_race",
                "Race the candidate orderings and keep the least fill.",
            ),
        ],
    );
    vocabulary(
        builder,
        "FeralScaling",
        &[
            ("inf_norm", "Knight–Ruiz infinity-norm equilibration."),
            ("mc64_symmetric", "MC64-style symmetric matching scaling."),
            ("identity", "No scaling."),
            (
                "auto",
                "MC64 for arrow-KKT shapes, infinity-norm otherwise.",
            ),
        ],
    );
}

/// The library presolve passes and policy kinds of the solve settings, and the tear
/// selection method (Plan 22 B5): vocabularies that cross the Python boundary.
fn declare_presolve(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "PresolvePass",
        &[
            ("linear_bounds", "Propagation using proved affine rows."),
            ("redundant_rows", "Library redundancy analysis."),
            (
                "affine_elimination",
                "Library affine column elimination and recovery.",
            ),
            ("fbbt", "Native expression-tape interval propagation."),
            (
                "rank_diagnostics",
                "Equality-rank diagnostics, without objective-changing remedies.",
            ),
            ("auxiliary", "Explicit safe auxiliary nonlinear reduction."),
        ],
    );
    vocabulary(
        builder,
        "PresolvePolicyKind",
        &[
            (
                "off",
                "Identity transport: source coordinates are preserved.",
            ),
            ("auto", "Only qualified source-backed passes."),
            (
                "explicit",
                "Complete library controls; required ineligible passes fail admission.",
            ),
        ],
    );
    vocabulary(
        builder,
        "TearMethod",
        &[
            (
                "highs",
                "Exact weighted feedback-edge MILP with native incumbent, bound and gap reporting.",
            ),
            (
                "unweighted_heuristic",
                "Explicit unweighted greedy feedback arc set.",
            ),
        ],
    );
}

/// One vocabulary from its spellings and member documentation, in declaration order.
fn vocabulary(
    builder: &mut RegistryBuilder,
    name: &'static str,
    members: &[(&'static str, &'static str)],
) {
    builder.declare_enum(EnumDecl::platform(
        name,
        members
            .iter()
            .map(|(member, doc)| EnumMember::new(member, doc))
            .collect(),
    ));
}

/// The Ipopt linear solver, its orderings, scalings, pivoting and matchings, and the
/// barrier strategy (ADR-0108).
fn declare_ipopt(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "IpoptLinearSolver",
        &[
            ("mumps", "Sequential MUMPS."),
            (
                "spral",
                "SPRAL SSIDS on OpenMP threads; needs OMP_CANCELLATION=TRUE in the process.",
            ),
            ("pardisomkl", "oneMKL Pardiso on MKL threads."),
        ],
    );
    vocabulary(
        builder,
        "MumpsOrdering",
        &[
            ("amd", "Approximate minimum degree."),
            ("amf", "Approximate minimum fill."),
            ("pord", "PORD."),
            (
                "metis",
                "METIS nested dissection (the image's shared METIS).",
            ),
            (
                "qamd",
                "Approximate minimum degree with quasi-dense row detection.",
            ),
        ],
    );
    vocabulary(
        builder,
        "SpralOrdering",
        &[
            ("metis", "METIS with default settings."),
            ("matching", "Matching-based elimination ordering."),
        ],
    );
    vocabulary(
        builder,
        "SpralScaling",
        &[
            ("none", "No scaling."),
            ("mc64", "Weighted bipartite matching (MC64)."),
            ("auction", "Auction algorithm."),
            ("matching", "Matching-based ordering's scaling."),
            ("ruiz", "Ruiz norm equilibration."),
        ],
    );
    vocabulary(
        builder,
        "SpralPivot",
        &[
            ("aggressive", "Aggressive a posteriori pivoting."),
            ("block", "Block a posteriori pivoting."),
            (
                "threshold",
                "Threshold partial pivoting; SPRAL runs it serially.",
            ),
        ],
    );
    vocabulary(
        builder,
        "PardisoOrdering",
        &[
            ("amd", "Minimum degree."),
            ("metis", "METIS nested dissection."),
            ("parallel_metis", "OpenMP-parallel METIS nested dissection."),
        ],
    );
    vocabulary(
        builder,
        "PardisoMatching",
        &[
            ("complete", "Complete matching."),
            ("complete_plus2x2", "Complete matching with 2x2 pivots."),
            ("constraints", "Matching of the constraint block."),
        ],
    );
    vocabulary(
        builder,
        "MuStrategy",
        &[
            ("monotone", "Monotone Fiacco–McCormick decrease."),
            ("adaptive", "Adaptive (Nocedal–Wächter–Waltz) update."),
        ],
    );
}

/// The solve controls shared by the native adapters.
fn declare_controls(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "HessianMode",
        &[
            ("exact", "Exact weighted Lagrangian Hessian."),
            (
                "limited_memory",
                "Library-owned quasi-Newton approximation.",
            ),
            (
                "gauss_newton",
                "Gauss–Newton Hessian of a least-squares objective: the weighted response Gram JᵀWJ plus the constraint-multiplier Hessians, without residual curvature. Admitted for least-squares fits only.",
            ),
        ],
    );
    vocabulary(
        builder,
        "ReusePolicy",
        &[
            ("fresh", "Always construct a fresh native model."),
            (
                "allow_rebuild",
                "Rebuild explicitly when data updates are ineligible.",
            ),
            (
                "require_reuse",
                "Fail rather than rebuilding incompatible native state.",
            ),
        ],
    );
    vocabulary(
        builder,
        "Preconditioner",
        &[
            ("none", "Unpreconditioned Krylov iterations."),
            (
                "jacobi",
                "Diagonal (Jacobi) scaling by the compiled Newton-matrix diagonal; a zero diagonal entry leaves its row unscaled.",
            ),
        ],
    );
}

/// The method selections of POUNCE, HiGHS, KINSOL and Clarabel.
fn declare_backend_methods(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "PounceMethod",
        &[
            ("interior_point", "Native barrier/filter NLP method."),
            (
                "active_set_sqp",
                "Native active-set sequential quadratic programming.",
            ),
            (
                "l1_exact_penalty",
                "The Thierry–Biegler ℓ1 exact penalty-barrier method (ADR-0109); explicit only, never selected automatically and never a retry.",
            ),
        ],
    );
    vocabulary(
        builder,
        "HighsMethod",
        &[
            ("choose", "Native method selection."),
            ("simplex", "Simplex for LP."),
            ("ipm", "Interior point for LP."),
            ("pdlp", "First-order primal-dual LP method."),
        ],
    );
    vocabulary(
        builder,
        "KinsolStrategy",
        &[
            (
                "picard",
                "Declared constant linear splitting with native Anderson acceleration.",
            ),
            ("newton", "Full Newton step."),
            ("line_search", "Globalized Newton line search."),
            (
                "fixed_point",
                "Declared fixed-point map with native Anderson acceleration.",
            ),
        ],
    );
    vocabulary(
        builder,
        "KinsolOrthogonalization",
        &[
            (
                "modified_gram_schmidt",
                "Modified Gram-Schmidt, KINSOL's default.",
            ),
            (
                "inverse_compact_wy",
                "Inverse compact WY modified Gram-Schmidt.",
            ),
            (
                "classical_gram_schmidt2",
                "Classical Gram-Schmidt with reorthogonalization.",
            ),
            (
                "delayed_classical_gram_schmidt2",
                "Classical Gram-Schmidt with delayed reorthogonalization.",
            ),
        ],
    );
    vocabulary(
        builder,
        "ClarabelMode",
        &[
            (
                "single_solve",
                "Native presolve/chordal preprocessing may change the native layout.",
            ),
            (
                "reusable_data",
                "Preserve structure for Clarabel's data update API: native presolve, input zero-dropping and chordal decomposition are disabled.",
            ),
        ],
    );
    vocabulary(
        builder,
        "ClarabelMergeMethod",
        &[
            ("none", "No merging."),
            ("parent_child", "Merge a clique into its parent."),
            ("clique_graph", "Clique-graph merging."),
        ],
    );
    vocabulary(
        builder,
        "ClarabelDirect",
        &[
            (
                "qdldl",
                "Clarabel's built-in quasidefinite LDLᵀ; factorizes on one thread.",
            ),
            (
                "mkl_pardiso",
                "oneMKL Pardiso from the process's one linked oneMKL (LP64, GNU threading); admits more than one thread.",
            ),
        ],
    );
}

/// The dynamics profile, Diffsol and IDAS settings (ADR-0110).
fn declare_dynamics(builder: &mut RegistryBuilder) {
    vocabulary(
        builder,
        "DynamicsMethod",
        &[
            (
                "auto",
                "Diffsol normally; IDAS when native trial recovery is required.",
            ),
            (
                "diffsol",
                "Rust BDF with library-owned hybrid reset sensitivities.",
            ),
            ("idas", "Residual BDF with recoverable trial callbacks."),
        ],
    );
    vocabulary(
        builder,
        "TrialPolicy",
        &[
            ("terminal", "A trial failure terminates this attempt."),
            (
                "recoverable",
                "The native method must support rejecting and retrying a trial.",
            ),
        ],
    );
    vocabulary(
        builder,
        "DiffsolMethod",
        &[
            ("bdf", "Variable-order variable-step BDF."),
            ("tr_bdf2", "Two-stage SDIRK TR-BDF2."),
            ("esdirk34", "Four-stage ESDIRK 3(4)."),
            (
                "tsit45",
                "Explicit Tsitouras 4(5); only a mass-free ODE is admitted.",
            ),
        ],
    );
    vocabulary(
        builder,
        "DiffsolLinear",
        &[
            ("faer_lu", "faer sparse LU."),
            ("klu", "SuiteSparse KLU (Diffsol `suitesparse` backend)."),
        ],
    );
    vocabulary(
        builder,
        "DynamicSensitivity",
        &[
            ("none", "No parameter derivatives."),
            (
                "forward",
                "Forward sensitivities of every sampled state and output to every integration parameter.",
            ),
            (
                "adjoint",
                "Checkpointed adjoint gradients of one scalar functional of the sampled outputs; no response Jacobian.",
            ),
        ],
    );
    // ADR-0110 Outcome 5: shooting routes over the integrated experiment; the scheduled
    // inputs held free are the controls.
    vocabulary(
        builder,
        "ShootingMethod",
        &[
            (
                "single",
                "One window over the horizon: the controls are the only variables.",
            ),
            (
                "multiple",
                "One window per node interval: the differential states at the inner nodes are variables, closed by continuity rows.",
            ),
        ],
    );
    vocabulary(
        builder,
        "FitDerivatives",
        &[
            (
                "responses",
                "The response Jacobian from forward sensitivities; every Hessian mode is available.",
            ),
            (
                "gradient",
                "The objective gradient alone, from adjoint sensitivities of transient experiments; needs the limited-memory Hessian.",
            ),
        ],
    );
    vocabulary(
        builder,
        "SensitivityCorrector",
        &[
            (
                "simultaneous",
                "State and sensitivity corrections in one Newton iteration.",
            ),
            (
                "staggered",
                "Sensitivities corrected after each converged state step.",
            ),
        ],
    );
    vocabulary(
        builder,
        "IdasInitialization",
        &[
            (
                "algebraic_and_rates",
                "Keep the requested differential states; compute algebraic states and all rates.",
            ),
            (
                "steady_states",
                "Every rate is zero and every state is computed: a steady start.",
            ),
        ],
    );
    vocabulary(
        builder,
        "StateSign",
        &[
            ("free", "Unconstrained."),
            ("non_negative", "x >= 0."),
            ("positive", "x > 0."),
            ("non_positive", "x <= 0."),
            ("negative", "x < 0."),
        ],
    );
}
