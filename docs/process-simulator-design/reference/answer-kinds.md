---
title: "Answer Kinds"
---

# Answer Kinds

<a id="source-40a141b58369810e8e69f617b4bda52f"></a>

## Analyze

Source: [Notion](https://app.notion.com/p/40a141b58369810e8e69f617b4bda52f).

**Meaning:** Inspect structural or local mathematical/scientific properties of a model, case, prepared form or qualified candidate. Structural diagnostics may operate before any successful Solve; local numerical analysis uses a suitable point and explicitly stated assumptions.

**Task classes:** [Model-based design of experiments](task-classes.md#source-40a141b58369819488a4eef536bc9e40); [Local sensitivity](task-classes.md#source-40a141b583698132a9a0dd565f3893f2); [Control analysis and tuning](task-classes.md#source-40a141b58369811dadd0d211c02af797); [Continuation, bifurcation, operability](task-classes.md#source-40a141b5836981819f82dccb72ec4c44)

**Gate checks:** Validate the particular analysis inputs and required products. Structural diagnostics must accept underdetermined, overdetermined or unsolved models when that is their subject. Local analysis checks point/branch/active-set conditions and derivative availability; numerical rank, DAE index methods, stability and identifiability each retain their own coverage and limitations. No universal proof or universal no-Decision/no-Uncertain rule is imposed.

**Parameters:** Requested analyses and their scope: counts/matching/block structure, local sensitivity/linearization, eigen/frequency response, local information/rank, controllability/observability or other registered diagnostics. A reference point is supplied only where the analysis needs one; a Solve is an explicit optional prerequisite.

**Lowers to:** Source/structural graph analysis, supported symbolic analysis, qualified local derivative products, sparse/dense linear algebra or eigenanalysis. Finite differences and other approximations state their method and error/regularity limitations.

**Solution:** Analysis findings, supported derivative/linear-model products and explicit unavailable results with source/point/formulation context. A singularity or rank deficiency may be the successful analysis finding; zero state-solution members is valid.

**Answer kind:** Analyze

This is an initial, extensible task-profile family. Eligibility is contextual; the table describes this profile and its explicit compositions, not a universal prohibition on reusing model data.

#### Contract column

| Concept | Analyze |
| --- | --- |
| Decision/estimand roles | May identify sensitivity, KKT or information coordinates; the requested analysis defines their treatment. |
| Uncertainty | May be part of the declared information/uncertainty analysis context; support is profile specific. |
| Dataset and observation model | Needed for relevant information/identifiability or residual diagnostics; optional otherwise. |
| Criterion and query constraints | May be inspected as part of the selected model/task/KKT analysis; they are not universally refused. |
| Structural choices | A selected configuration or explicit comparison set is required; analysis does not silently choose physics. |
| Reference point | Required only for local numerical analyses; structural counts/matching and other source diagnostics need no successful Solve. |
| Derivatives | Required only by analyses that consume or return them; not required for every structural diagnostic. |
| Duals | Consumed or returned where the selected KKT/optimization analysis supports them, with qualification. |
| Statistics | Available only under a declared statistical analysis profile and assumptions. |
| Design specification | May be the subject of closure, over/underspecification or rank diagnostics. |
| Regime | Part of the selected formulation context; not an implicit requirement to simulate first. |
| Finding strength | Separate structural observation, numerical/local evidence and proof; unresolved properties remain explicit. |
| Result cardinality | 0..n findings/products; a state solution member is optional. |

#### Assessment rule

Distinguish checkable contracts, structural diagnostics, local numerical evidence, assumptions and certificates. A failed or refused task may return zero numerical members with diagnostic and provenance evidence. Unsupported optional products are withheld with reasons; unsupported mandatory requirements are refused.


<a id="source-40a141b5836981f0a45ad165770120dd"></a>

## Expect

Source: [Notion](https://app.notion.com/p/40a141b5836981f0a45ad165770120dd).

**Meaning:** Compute declared statistics under a selected probability model, supplied by input distributions, a supported stochastic model or a posterior, with explicit conditioning, estimator and coverage semantics.

**Task classes:** [Stochastic kinetics, kinetic Monte Carlo](task-classes.md#source-40a141b5836981879f2fc62d227fda9a); [Molecular dynamics](task-classes.md#source-40a141b5836981fdba1fc800295b187b); [Molecular Monte Carlo (NVT, GCMC, Gibbs ensemble)](task-classes.md#source-40a141b583698131a15cd7921be183e0); [Uncertainty propagation, global sensitivity](task-classes.md#source-40a141b58369810a8c0ce5b89fe1dc4d); [Optimization under uncertainty (stochastic, robust, chance-constrained)](task-classes.md#source-40a141b583698195bf99e5db2364569e)

**Gate checks:** Validate measure definition/support and the selected evaluator/estimator contract. Properness and moment existence are proved only where the available method supports them, otherwise their assumptions/unresolved status are explicit. A sampled process evaluation can fail: retain failure coverage and follow the declared policy instead of presuming every sampled Solve is well posed. Confidence requirements depend on the requested statistical claim.

**Parameters:** Statistics and their scope; probability model and dependence; conditioning and failure treatment; requested uncertainty/confidence evidence. Statistical targets are semantic; quadrature, sampling, surrogate approximation and numerical discretization remain declared realization choices.

**Lowers to:** Qualified ensembles, quadrature or explicit approximation methods; supported posterior or stochastic-process estimators. Molecular dynamics, molecular Monte Carlo and general stochastic-physics implementations remain deferred specialized profiles rather than implicit process capabilities.

**Solution:** Available statistics and optional samples with coverage, approximation/estimator evidence and qualified intervals when justified. Missing moments, unresolved sampling convergence or failed scenarios may yield partial/unavailable products; they are not replaced by an unqualified complete distribution.

**Answer kind:** Expect

This is an initial, extensible task-profile family. Eligibility is contextual; the table describes this profile and its explicit compositions, not a universal prohibition on reusing model data.

#### Contract column

| Concept | Expect |
| --- | --- |
| Uncertainty source | A declared probability law, posterior or supported intrinsic stochastic model is required. |
| Decision role | Fixed decisions may be conditioned on; optimizing them belongs to an explicit outer/composed task. |
| Estimands | A posterior or fitted-parameter uncertainty may be consumed with its validity; inference itself remains a separate/composed task. |
| Distribution | Required for the actual probabilistic contract; a robust uncertainty set alone is not a probability law. |
| Dataset and observation model | May define an empirical measure or supply posterior/validation context; no blanket prohibition. |
| Criterion | Statistics or event functionals are explicit; an optimization objective is an outer/composed task. |
| Point/path conditions | May define event statistics or validity/conditioning; discarding infeasible samples must not silently change the measure. |
| Chance/robust constraints | Probability estimation is supported by this family; enforcing a quantified design constraint is an explicit composition. |
| Design specification | Optional within each evaluated process case. |
| Derivatives | Optional of statistics, under a justified differentiation/estimation method. |
| Duals | Not a default expectation product; constituent optimization evidence may be retained with its context. |
| Statistics | Required; requested moments need an explicit existence/coverage assessment. |
| Accuracy/confidence | Qualified by estimator and approximation assumptions; not every method yields a justified confidence interval. |
| Result cardinality | Zero or more available summaries/sample products with complete, partial or unavailable coverage. |

#### Assessment rule

Distinguish checkable contracts, structural diagnostics, local numerical evidence, assumptions and certificates. A failed or refused task may return zero numerical members with diagnostic and provenance evidence. Unsupported optional products are withheld with reasons; unsupported mandatory requirements are refused.


<a id="source-40a141b58369810e8c01df60ebe98800"></a>

## Extremize

Source: [Notion](https://app.notion.com/p/40a141b58369810e8c01df60ebe98800).

**Meaning:** Optimize explicitly declared criteria over admitted decision domains subject to the model and task requirements, while preserving scientific model choices, scenario/recourse semantics and the distinction between feasible, stationary, locally optimal and globally qualified results.

**Task classes:** [Geometry optimization, transition-state search](task-classes.md#source-40a141b58369818da84cc6177b7f2b1c); [Steady-state process optimization](task-classes.md#source-40a141b5836981fe8c56d3a459757706); [Process synthesis (superstructure)](task-classes.md#source-40a141b5836981c78eabee8eea737656); [Dynamic optimization, optimal control](task-classes.md#source-40a141b58369813380f0e2b75266eff0); [Model-based design of experiments](task-classes.md#source-40a141b58369819488a4eef536bc9e40); [Optimization under uncertainty (stochastic, robust, chance-constrained)](task-classes.md#source-40a141b583698195bf99e5db2364569e); [Control analysis and tuning](task-classes.md#source-40a141b58369811dadd0d211c02af797); [MPC and moving-horizon estimation](task-classes.md#source-40a141b583698107b3decc6ccd46c9f5); [Scheduling of batch plants](task-classes.md#source-40a141b5836981d09fd6f348e06ea597); [Techno-economics and life-cycle assessment](task-classes.md#source-40a141b5836981068a21df297df43afd)

**Gate checks:** Validate roles, domains, aggregation, physical constraints and selected realization capabilities. Finite bounds, coercivity, exact derivatives or global envelopes are required only by the chosen method/claim. Structural counts with decisions fixed are diagnostics, not a universal well-posedness theorem. Scenario-conditioned constraints need not all be chance/robust; their quantifiers must be explicit. Global or frontier-completeness claims require supporting evidence.

**Parameters:** Decision selections and admissible domains; scalar or explicitly treated multiple criteria; local/global target and tolerances; uncertainty/recourse or lexicographic/frontier policy when selected. Stationary-point or saddle search is a separately registered profile with its own index evidence, not ordinary process minimization.

**Lowers to:** NLP, LP/QP/conic forms when qualified, MINLP/GDP or dynamic optimization; supported scenario/robust formulations. A KKT or complementarity reformulation of an inner optimum retains its conditions and does not automatically preserve global semantics.

**Solution:** Zero or more candidates or qualified optima/frontier members with primal feasibility, applicable stationarity/curvature/gap evidence and coverage. Duals and optimum sensitivities are conditional products; a finite Pareto sample or best multi-start candidate is not an exhaustive/global certificate.

**Answer kind:** Extremize

This is an initial, extensible task-profile family. Eligibility is contextual; the table describes this profile and its explicit compositions, not a universal prohibition on reusing model data.

#### Contract column

| Concept | Extremize |
| --- | --- |
| Decision role | Required for an optimization task; each decision has a declared type/domain and case-sharing scope. |
| Structural Decision | Optional through an admitted structural alternative/disjunctive formulation. |
| Uncertainty | Optional with explicit probability/uncertainty-set, scenario, recourse and quantifier semantics. |
| Estimands | May be supplied from a separate Infer task; joint estimation/design needs an explicit composed profile. |
| Distribution or uncertainty set | Required only for the uncertainty semantics actually used; they are not interchangeable. |
| Dataset and observation model | May supply parameters, empirical objective terms or assessment context without silently converting the task to Infer. |
| Criterion | Required; explicit aggregation, units and multiobjective treatment. |
| Point/path/terminal constraints | Optional and form/profile dependent. |
| Chance/robust constraints | Optional under a supported quantified profile; ordinary scenario-conditioned physical constraints are also valid. |
| Design specification | Optional with explicit target/released-variable semantics. |
| Derivatives | Optional result products; algorithmic derivative requirements depend on the selected method. |
| Duals | Conditional on form, solver evidence and supported original-space reconstruction. |
| Statistics | May be requested for explicit uncertainty/risk outputs; no blanket prohibition. |
| Regime | Independent of optimization; steady, dynamic and periodic profiles have distinct eligibility. |
| Result cardinality | 0..n with per-member qualification and explicit frontier/search completeness. |

#### Assessment rule

Distinguish checkable contracts, structural diagnostics, local numerical evidence, assumptions and certificates. A failed or refused task may return zero numerical members with diagnostic and provenance evidence. Unsupported optional products are withheld with reasons; unsupported mandatory requirements are refused.


<a id="source-40a141b58369818f926dcf983d26cb3c"></a>

## Infer

Source: [Notion](https://app.notion.com/p/40a141b58369818f926dcf983d26cb3c).

**Meaning:** Estimate selected quantities or a posterior from observations and a declared observation/error model, with explicit prior, regularization and nuisance/uncertainty treatment. Identifiability and uncertainty qualification are assessed rather than assumed.

**Task classes:** [Parameter estimation](task-classes.md#source-40a141b5836981c7a20eef0a8cee92e6); [Data reconciliation, gross error detection](task-classes.md#source-40a141b5836981eeadabdb97d7a8aef0); [Bayesian calibration](task-classes.md#source-40a141b583698141af4cea4a66892d25); [MPC and moving-horizon estimation](task-classes.md#source-40a141b583698107b3decc6ccd46c9f5); [Surrogate and reduced-order model training](task-classes.md#source-40a141b58369811daacbf96d07b68090)

**Gate checks:** Validate estimand/data mappings, physical conventions, likelihood/loss and selected implementation capabilities. Local numerical rank is evidence for local identifiability under stated conditions, not proof of global/structural identifiability. Rank deficiency may yield identifiable subspaces or withheld covariance; it is not a universal ban on fitting. Priors and posterior propriety assumptions/checks are explicit; unresolved arbitrary properness is not presented as a proved gate.

**Parameters:** Point method (likelihood, weighted least squares, declared loss or MAP) or posterior method; selected datasets, observation/noise contracts, estimands and shared/case-local parameters; known/estimated noise, constraints and errors-in-variables treatment.

**Lowers to:** Likelihood/loss optimization, supported constrained fitting or explicit posterior approximation/sampling. Covariance or information analysis is a separately qualified product, and variational approximation is identified as such rather than silently equated with exact posterior sampling.

**Solution:** Zero or more fitted candidates or posterior products with residual/adequacy findings and provenance. Covariance, confidence/credible intervals and goodness-of-fit tests are produced only under their stated assumptions and coverage; unavailable products and prior/regularization influence are reported.

**Answer kind:** Infer

This is an initial, extensible task-profile family. Eligibility is contextual; the table describes this profile and its explicit compositions, not a universal prohibition on reusing model data.

#### Contract column

| Concept | Infer |
| --- | --- |
| Estimand role | Required; names quantities being inferred and their domains/shared-case meaning. |
| Decision role | Experimental design or controller decisions belong to an explicit outer/composed task. |
| Structural/model selection | Optional only through a declared supported model-comparison/selection profile. |
| Uncertainty/nuisance inputs | Allowed with explicit integration, conditioning, profiling or approximation treatment; not restricted universally to posterior tasks. |
| Prior | Required by a Bayesian posterior contract as applicable; optional for MAP/regularized point methods with its influence stated. |
| Dataset and observation model | Required for this data-conditioned profile, including alignment and noise/loss meaning. |
| Criterion | A declared statistical loss/likelihood belongs to Infer; arbitrary process-profit objectives require a composed task. |
| Point/path/parameter constraints | Allowed when the inference formulation supports them; not limited to scalar estimand bounds. |
| Chance/robust constraints | Only through an explicitly registered robust/quantified inference profile. |
| Design specification | Optional as physical/task constraints, with effective structural checks. |
| Derivatives and duals | Optional and capability/regularity dependent. |
| Statistics | Optional point/posterior summaries with qualified uncertainty evidence. |
| Identifiability | Report structural assumptions, local rank and unresolved directions separately; do not promise a universal proof. |
| Result cardinality | 0..n; covariance and intervals may be absent even when a point estimate is usable. |

#### Assessment rule

Distinguish checkable contracts, structural diagnostics, local numerical evidence, assumptions and certificates. A failed or refused task may return zero numerical members with diagnostic and provenance evidence. Unsupported optional products are withheld with reasons; unsupported mandatory requirements are refused.


<a id="source-40a141b5836981d28ffbc1a901cf1364"></a>

## Solve

Source: [Notion](https://app.notion.com/p/40a141b5836981d28ffbc1a901cf1364).

**Meaning:** Find or evaluate physically admissible values of the selected unknowns under the model, effective hard inputs and ordinary feasibility/design requirements, or construct a qualified trajectory/field under an admitted temporal or distributed profile.

**Task classes:** [Property evaluation](task-classes.md#source-40a141b5836981e0a8e2e47b40c50bf1); [Phase and chemical equilibrium (flash, Gibbs reactor)](task-classes.md#source-40a141b5836981c2ad25e466ff838ff5); [Steady-state flowsheet](task-classes.md#source-40a141b5836981e4b476efb26e9785a5); [Unit rating vs design](task-classes.md#source-40a141b5836981d5a486da2894ab6499); [Dynamic simulation (start-up, upset, batch)](task-classes.md#source-40a141b5836981aea83ce668148b9660); [Distributed unit models (PFR, packed bed, membrane, rate-based column)](task-classes.md#source-40a141b5836981bab161e184223889da); [CFD](task-classes.md#source-40a141b5836981fdb65ecee60bcb56d2); [Population balances (crystallization, polymerization)](task-classes.md#source-40a141b583698173aae3fc2178ee8ebd); [Reaction kinetics and microkinetics](task-classes.md#source-40a141b583698109a52ce2b399afdc04); [Electronic structure (HF, DFT, post-HF)](task-classes.md#source-40a141b58369816d9d32ecfc803e1cad); [Continuation, bifurcation, operability](task-classes.md#source-40a141b5836981819f82dccb72ec4c44); [Techno-economics and life-cycle assessment](task-classes.md#source-40a141b5836981068a21df297df43afd)

**Gate checks:** Check references, complete physical types, effective inputs, selected constraints and contextual kernel eligibility. Equation counts and structural matching are diagnostics or requirements of a selected square-root route, not proofs of existence, uniqueness or numerical rank. Initial/boundary consistency and DAE index are checked only by supported analyses; unresolved conditions are reported or refused when required. Design targets and released scalar/block variables must yield an admitted effective problem.

**Parameters:** Task profile (evaluation, steady feasibility, selected root, trajectory or distributed solve); required constraints and outputs; root/branch-selection policy where relevant. An exhaustive root request states a bounded scope and requires an appropriate completeness method; a starting guess alone is not a branch contract.

**Lowers to:** Direct evaluation; NLE or MCP; constrained feasibility formulations where supported; qualified ODE/DAE or hybrid integration; admitted distributed-unit discretization. The process profile does not imply a general CFD or electronic-structure solver.

**Solution:** Zero or more candidate/qualified members with explicit output coverage, residual/physical closure findings and provenance. Sensitivities, multipliers or certificates are optional products only when the selected route and regularity/reconstruction support them; failure or refusal may have no numerical member.

**Answer kind:** Solve

This is an initial, extensible task-profile family. Eligibility is contextual; the table describes this profile and its explicit compositions, not a universal prohibition on reusing model data.

#### Contract column

| Concept | Solve |
| --- | --- |
| Primary roles | Effective hard inputs and unknowns; a released design variable is an unknown in this task, rather than necessarily an optimization Decision. |
| Structural choices | Resolved selections or explicitly supported discrete feasibility decisions; no implicit scientific-model swap. |
| Uncertainty | An explicit conditioned case or composed uncertainty task is required for unresolved random inputs. |
| Estimands | Fitted in an Infer task; a fitted value can be supplied as a fixed input with provenance. |
| Dataset and observation model | May supply inputs or independent validation; do not automatically add a fitting objective. |
| Criterion | Absent for pure feasibility; any numerical merit function is implementation detail and does not become a physical objective. |
| Point/path/terminal query constraints | Allowed when the selected feasibility or dynamic profile supports them. |
| Chance/robust constraints | Only through an explicit admitted quantified/composed profile. |
| Design specification | Optional; declare target equations and released scalar/block quantities, then assess the effective structure/rank. |
| Derivatives | Optional, with exact/approximate and local-validity scope. |
| Duals/certificates | Only where meaningful and supported by the selected form and reconstruction; no automatic shadow-price claim. |
| Statistics | Belong to an explicit statistical composition or validation summary. |
| Regime | Explicit or governed by the documented task-profile default. |
| Result cardinality | 0..n; a successful selected task states its required output coverage and qualification. |

#### Assessment rule

Distinguish checkable contracts, structural diagnostics, local numerical evidence, assumptions and certificates. A failed or refused task may return zero numerical members with diagnostic and provenance evidence. Unsupported optional products are withheld with reasons; unsupported mandatory requirements are refused.
