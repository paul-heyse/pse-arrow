use super::*;

/// A numerical function callback receiving, in order, evaluated arguments,
/// constant values, registered function evaluators, and a mutable cache of
/// previously evaluated expressions.
pub type EvalFnType<A, T> = Box<
    dyn Fn(
        &[T],
        &HashMap<A, T>,
        &HashMap<Symbol, EvaluationFn<A, T>>,
        &mut HashMap<AtomView<'_>, T>,
    ) -> T,
>;

/// A closure that can be called to evaluate a function called with arguments of type `T`.
pub struct EvaluationFn<A, T>(EvalFnType<A, T>);

impl<A, T> EvaluationFn<A, T> {
    /// Wrap a numerical callback for registration in an evaluation function map.
    pub fn new(f: EvalFnType<A, T>) -> EvaluationFn<A, T> {
        EvaluationFn(f)
    }

    /// Get a reference to the function that can be called to evaluate it.
    pub fn get(&self) -> &EvalFnType<A, T> {
        &self.0
    }
}

/// Policy controlling whether a registered function is expanded at its call sites.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InliningPolicy {
    /// Always inline the function body.
    #[default]
    Always,
    /// Keep the function body in a separate sub-evaluator.
    Never,
    /// Select an inlining strategy automatically.
    ///
    /// Currently this has the same behavior as [`Self::Always`].
    Auto,
}

/// Settings used when registering a function with an evaluator.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FunctionRegistrationOptions {
    inlining: InliningPolicy,
}

impl FunctionRegistrationOptions {
    /// Create function registration options with their default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the function inlining policy.
    pub fn inlining(mut self, inlining: InliningPolicy) -> Self {
        self.inlining = inlining;
        self
    }

    pub(crate) fn should_inline(&self) -> bool {
        !matches!(self.inlining, InliningPolicy::Never)
    }
}

/// A map of functions and constants used for evaluating expressions.
///
/// Examples
/// --------
/// ```rust
/// use symbolica::prelude::*;
/// let params = vec![parse!("x")];
/// let mut evaluator = parse!("f(x)")
///     .evaluator(&params)
///     .add_function(symbol!("f"), vec![symbol!("x")], parse!("x^2 + 1")).unwrap()
///     .build()
///     .unwrap()
///     .map_coeff(&|x| x.re.to_f64());
/// assert_eq!(evaluator.evaluate_single(&[2.0]), 5.0);
/// ```
#[cfg_attr(
    feature = "bincode",
    derive(bincode_trait_derive::Encode),
    derive(bincode_trait_derive::Decode),
    derive(bincode_trait_derive::BorrowDecodeFromDecode),
    trait_decode(trait = crate::state::HasStateMap)
)]
#[derive(Clone, Debug)]
pub struct FunctionMap {
    pub(super) tagged_fn_map: HashMap<(Symbol, Vec<Atom>), Expr>,
    pub(super) tag: HashMap<Symbol, usize>,
}

impl Default for FunctionMap {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionMap {
    /// Create a new, empty function map.
    pub fn new() -> Self {
        FunctionMap {
            tagged_fn_map: HashMap::default(),
            tag: HashMap::default(),
        }
    }

    /// Register a function. If `name` is a symbol, it will be treated as a regular function; if it is a function, its arguments will be treated as tags.
    /// Bodies see global evaluator inputs and their own arguments, which shadow matching
    /// global names, regardless of inlining policy. Use [`Self::add_aliases`] for definitions
    /// that expand in the caller's local scope.
    pub fn add_function<S: Into<Indeterminate>, A: Into<Indeterminate>>(
        &mut self,
        name: S,
        args: Vec<A>,
        body: Atom,
    ) -> Result<(), EvaluationError> {
        self.add_function_with_options(name, args, body, FunctionRegistrationOptions::default())
    }

    /// Register a function using explicit registration options.
    pub fn add_function_with_options<S: Into<Indeterminate>, A: Into<Indeterminate>>(
        &mut self,
        name: S,
        args: Vec<A>,
        body: Atom,
        options: FunctionRegistrationOptions,
    ) -> Result<(), EvaluationError> {
        let name = name.into();

        let (name, tags) = match name {
            Indeterminate::Symbol(name, _) => (name, vec![]),
            Indeterminate::Function(name, f) => {
                let tags = f
                    .as_fun_view()
                    .unwrap()
                    .iter()
                    .map(|x| x.to_owned())
                    .collect();
                (name, tags)
            }
        };

        self.add_tagged_function_with_options(name, tags, args, body, options)
    }

    /// Register argument-free aliases (e.g. `s1 -> x^2+y`).
    /// Aliases always inline in the scope where they are used, including local arguments.
    /// Function-shaped aliases match their full argument list as fixed tags.
    pub fn add_aliases(
        &mut self,
        aliases: impl IntoIterator<Item = (Atom, Atom)>,
    ) -> Result<(), EvaluationError> {
        for (from, to) in aliases {
            let (name, tags) = match &from {
                Atom::Var(v) => (v.get_symbol(), vec![]),
                Atom::Fun(f) => (
                    f.get_symbol(),
                    from.as_fun_view()
                        .unwrap()
                        .iter()
                        .map(|x| x.to_owned())
                        .collect(),
                ),
                _ => return Err(EvaluationError::NotIndeterminate { atom: from }),
            };
            self.add_tagged_function::<Indeterminate>(name, tags.clone(), vec![], to)?;
            self.tagged_fn_map.get_mut(&(name, tags)).unwrap().is_alias = true;
        }
        Ok(())
    }

    /// Register a function, where the first arguments are `tags` instead of arguments.
    pub fn add_tagged_function<A: Into<Indeterminate>>(
        &mut self,
        name: Symbol,
        tags: Vec<Atom>,
        args: Vec<A>,
        body: Atom,
    ) -> Result<(), EvaluationError> {
        self.add_tagged_function_with_options(
            name,
            tags,
            args,
            body,
            FunctionRegistrationOptions::default(),
        )
    }

    /// Register a tagged function using explicit registration options.
    pub fn add_tagged_function_with_options<A: Into<Indeterminate>>(
        &mut self,
        name: Symbol,
        tags: Vec<Atom>,
        args: Vec<A>,
        body: Atom,
        options: FunctionRegistrationOptions,
    ) -> Result<(), EvaluationError> {
        match self.tag.get(&name) {
            Some(&t) if t != tags.len() => {
                return Err(EvaluationError::InconsistentFunctionTagCount {
                    function: name,
                    expected: t,
                    actual: tags.len(),
                });
            }
            None => {
                self.tag.insert(name, tags.len());
            }
            _ => {}
        }

        let key = (name, tags.clone());
        if self.tagged_fn_map.contains_key(&key) {
            return Err(EvaluationError::FunctionRedefined {
                function: name.call(&tags),
            });
        }

        let id = self.tagged_fn_map.len();
        let tag_len = tags.len();
        self.tagged_fn_map
            .entry((name, tags.clone()))
            .or_insert_with(|| Expr {
                id,
                tag_len,
                is_alias: false,
                args: args.into_iter().map(|x| x.into()).collect(),
                body,
                options,
            });

        Ok(())
    }

    pub(super) fn get_tag_len(&self, symbol: &Symbol) -> usize {
        self.tag.get(symbol).cloned().unwrap_or(0)
    }

    pub(super) fn get(&self, a: AtomView) -> Option<&Expr> {
        if let AtomView::Var(s) = a {
            return self.tagged_fn_map.get(&(s.get_symbol(), vec![]));
        } else if let AtomView::Fun(aa) = a {
            let s = aa.get_symbol();
            let tag_len = self.get_tag_len(&s);

            if aa.get_nargs() >= tag_len {
                let tag = aa.iter().take(tag_len).map(|x| x.to_owned()).collect();
                return self.tagged_fn_map.get(&(s, tag));
            }
        }

        None
    }

    pub(super) fn has_non_inlined_functions(&self) -> bool {
        self.tagged_fn_map
            .values()
            .any(|expr| !expr.options.should_inline())
    }
}

/// Builder for creating an [`ExpressionEvaluator`]. Constructed using
/// [`AtomCore::evaluator_multiple`] or [`AtomCore::evaluator`].
///
/// Nested functions can be registerd using [`Self::add_function`] or [`Self::add_tagged_function`].
/// The [`Self::build`] method finalizes the builder and returns an [`ExpressionEvaluator`].
///
/// # Examples
///
/// ```
/// use symbolica::prelude::*;
/// let params = vec![parse!("x")];
/// let mut evaluator = parse!("f(x)")
///     .evaluator(&params)
///     .add_function(symbol!("f"), vec![symbol!("x")], parse!("x^2 + 1"))?
///     .build()?
///     .map_coeff(&|x| x.re.to_f64());
/// assert_eq!(evaluator.evaluate_single(&[2.0]), 5.0);
/// # Ok::<(), EvaluationError>(())
/// ```
#[derive(Clone, Debug)]
pub struct EvaluatorBuilder<'a> {
    exprs: Vec<AtomView<'a>>,
    fn_map: FunctionMap,
    params: Vec<Atom>,
    optimization_settings: OptimizationSettings,
}

impl<'a> EvaluatorBuilder<'a> {
    pub(crate) fn new<A: AtomCore>(expr: AtomView<'a>, params: &[A]) -> Self {
        Self {
            exprs: vec![expr],
            fn_map: FunctionMap::new(),
            params: params.iter().map(|p| p.as_atom_view().to_owned()).collect(),
            optimization_settings: OptimizationSettings::default(),
        }
    }

    pub(crate) fn new_multiple_views<A: AtomCore>(exprs: &[AtomView<'a>], params: &[A]) -> Self {
        Self {
            exprs: exprs.to_vec(),
            fn_map: FunctionMap::new(),
            params: params.iter().map(|p| p.as_atom_view().to_owned()).collect(),
            optimization_settings: OptimizationSettings::default(),
        }
    }

    pub(crate) fn new_multiple<E: AtomCore, A: AtomCore>(exprs: &'a [E], params: &[A]) -> Self {
        Self {
            exprs: exprs.iter().map(|e| e.as_atom_view()).collect(),
            fn_map: FunctionMap::new(),
            params: params.iter().map(|p| p.as_atom_view().to_owned()).collect(),
            optimization_settings: OptimizationSettings::default(),
        }
    }

    /// Set the function map.
    pub fn function_map(mut self, fn_map: FunctionMap) -> Self {
        self.fn_map = fn_map;
        self
    }

    /// Register a function. If `name` is a symbol, it will be treated as a regular function; if it is a function, its arguments will be treated as tags.
    pub fn add_function<S: Into<Indeterminate>, A: Into<Indeterminate>>(
        mut self,
        name: S,
        args: Vec<A>,
        body: Atom,
    ) -> Result<Self, EvaluationError> {
        self.fn_map.add_function(name, args, body)?;
        Ok(self)
    }

    /// Register a function using explicit registration options.
    pub fn add_function_with_options<S: Into<Indeterminate>, A: Into<Indeterminate>>(
        mut self,
        name: S,
        args: Vec<A>,
        body: Atom,
        options: FunctionRegistrationOptions,
    ) -> Result<Self, EvaluationError> {
        self.fn_map
            .add_function_with_options(name, args, body, options)?;
        Ok(self)
    }

    /// Register a function, where the first arguments are `tags` instead of arguments.
    pub fn add_tagged_function<A: Into<Indeterminate>>(
        mut self,
        name: Symbol,
        tags: Vec<Atom>,
        args: Vec<A>,
        body: Atom,
    ) -> Result<Self, EvaluationError> {
        self.fn_map.add_tagged_function(name, tags, args, body)?;
        Ok(self)
    }

    /// Register a tagged function using explicit registration options.
    pub fn add_tagged_function_with_options<A: Into<Indeterminate>>(
        mut self,
        name: Symbol,
        tags: Vec<Atom>,
        args: Vec<A>,
        body: Atom,
        options: FunctionRegistrationOptions,
    ) -> Result<Self, EvaluationError> {
        self.fn_map
            .add_tagged_function_with_options(name, tags, args, body, options)?;
        Ok(self)
    }

    /// Register argument-free aliases that always inline in the caller's local scope.
    /// See [`FunctionMap::add_aliases`].
    pub fn add_aliases(
        mut self,
        aliases: impl IntoIterator<Item = (Atom, Atom)>,
    ) -> Result<Self, EvaluationError> {
        self.fn_map.add_aliases(aliases)?;
        Ok(self)
    }

    /// Set all optimization settings at once.
    pub fn optimization_settings(mut self, optimization_settings: OptimizationSettings) -> Self {
        self.optimization_settings = optimization_settings;
        self
    }

    /// Set the number of Horner scheme optimization iterations.
    pub fn horner_iterations(mut self, horner_iterations: usize) -> Self {
        self.optimization_settings.horner_iterations = horner_iterations;
        self
    }

    /// Set the number of CPU cores to use during optimization.
    pub fn cores(mut self, n_cores: usize) -> Self {
        self.optimization_settings.n_cores = n_cores;
        self
    }

    /// Set the number of common pair elimination iterations.
    pub fn cpe_iterations(mut self, cpe_iterations: Option<usize>) -> Self {
        self.optimization_settings.cpe_iterations = cpe_iterations;
        self
    }

    /// Set a hot-start expression list for optimization.
    pub fn hot_start(mut self, hot_start: Option<Vec<Expression<Complex<Rational>>>>) -> Self {
        self.optimization_settings.hot_start = hot_start;
        self
    }

    /// Set the abort check used during optimization.
    pub fn abort_check(mut self, abort_check: Option<Box<dyn AbortCheck>>) -> Self {
        self.optimization_settings.abort_check = abort_check;
        self
    }

    /// Set the abort polling level.
    pub fn abort_level(mut self, abort_level: usize) -> Self {
        self.optimization_settings.abort_level = abort_level;
        self
    }

    /// Set the maximum number of variables considered for a Horner scheme.
    pub fn max_horner_scheme_variables(mut self, max_horner_scheme_variables: usize) -> Self {
        self.optimization_settings.max_horner_scheme_variables = max_horner_scheme_variables;
        self
    }

    /// Set the maximum number of common-pair cache entries.
    pub fn max_common_pair_cache_entries(mut self, max_common_pair_cache_entries: usize) -> Self {
        self.optimization_settings.max_common_pair_cache_entries = max_common_pair_cache_entries;
        self
    }

    /// Set the maximum distance considered for common-pair elimination.
    pub fn max_common_pair_distance(mut self, max_common_pair_distance: usize) -> Self {
        self.optimization_settings.max_common_pair_distance = max_common_pair_distance;
        self
    }

    /// Enable or disable verbose optimization output.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.optimization_settings.verbose = verbose;
        self
    }

    /// Enable or disable direct translation.
    pub fn direct_translation(mut self, direct_translation: bool) -> Self {
        self.optimization_settings.direct_translation = direct_translation;
        self
    }

    /// Copy borrowed source/parameters and build the native alias map only after
    /// its complete staging, table and tag backing has been admitted. The
    /// returned builder owns no caller control; finalize it with `try_build*`.
    pub fn try_new_aliases_with<C: OperationControl>(
        expressions: &'a [Atom],
        parameters: &[Atom],
        aliases: &[(Atom, Atom)],
        control: &mut C,
    ) -> Result<Self, OperationError<C::Error>> {
        use numerica::domains::backend::integer::IntegerAllocationContext;
        control
            .check(OperationExtent {
                stage: OperationStage::Construction,
                ..OperationExtent::default()
            })
            .map_err(OperationError::Control)?;
        let add = |a: usize, b: usize| a.checked_add(b).ok_or(OperationError::<C::Error>::Overflow);
        let mul = |a: usize, b: usize| a.checked_mul(b).ok_or(OperationError::<C::Error>::Overflow);
        fn signature(atom: &Atom) -> Option<(Symbol, usize)> {
            match atom {
                Atom::Var(v) => Some((v.get_symbol(), 0)),
                Atom::Fun(f) => Some((f.get_symbol(), f.get_nargs())),
                _ => None,
            }
        }
        struct AliasKey<'b> {
            symbol: Symbol,
            atom: AtomView<'b>,
        }
        impl PartialEq for AliasKey<'_> {
            fn eq(&self, other: &Self) -> bool {
                if self.symbol != other.symbol {
                    return false;
                }
                match (self.atom, other.atom) {
                    (AtomView::Fun(a), AtomView::Fun(b)) => {
                        a.get_nargs() == b.get_nargs()
                            && a.iter()
                                .zip(b.iter())
                                .all(|(a, b)| a.get_data() == b.get_data())
                    }
                    (AtomView::Fun(a), _) | (_, AtomView::Fun(a)) => a.get_nargs() == 0,
                    _ => true,
                }
            }
        }
        impl Eq for AliasKey<'_> {}
        impl std::hash::Hash for AliasKey<'_> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.symbol.hash(state);
                if let AtomView::Fun(f) = self.atom {
                    f.get_nargs().hash(state);
                    for arg in f {
                        arg.get_data().hash(state);
                    }
                } else {
                    0usize.hash(state);
                }
            }
        }
        fn table<K, V, E>(entries: usize) -> Result<usize, OperationError<E>> {
            if entries == 0 {
                return Ok(0);
            }
            // Pinned std/hashbrown load factor, four-bucket initial allocation;
            // include control groups and alignment rather than allocator RSS.
            let buckets = entries
                .checked_mul(8)
                .and_then(|v| v.checked_add(6))
                .map(|v| v / 7)
                .and_then(usize::checked_next_power_of_two)
                .ok_or(OperationError::Overflow)?
                .max(4);
            buckets
                .checked_mul(size_of::<(K, V)>() + 1)
                .and_then(|v| v.checked_add(16 + std::mem::align_of::<(K, V)>()))
                .ok_or(OperationError::Overflow)
        }
        let input_atoms = expressions
            .iter()
            .chain(parameters)
            .chain(aliases.iter().flat_map(|(from, to)| [from, to]));
        let mut input = add(
            mul(expressions.len(), size_of::<Atom>())?,
            add(
                mul(parameters.len(), size_of::<Atom>())?,
                mul(aliases.len(), size_of::<(Atom, Atom)>())?,
            )?,
        )?;
        let mut payload = 0;
        for atom in input_atoms {
            input = add(input, atom.get_capacity())?;
            payload = add(payload, atom.as_view().get_byte_size())?;
        }
        let mut tags = 0;
        for (from, _) in aliases {
            let (_, count) = signature(from).ok_or(OperationError::Unsupported(
                "alias key is not an indeterminate",
            ))?;
            tags = add(tags, count)?;
        }
        let tables = add(
            table::<(Symbol, Vec<Atom>), Expr, C::Error>(aliases.len())?,
            table::<Symbol, usize, C::Error>(aliases.len())?,
        )?;
        let output = add(
            add(payload, mul(parameters.len(), size_of::<Atom>())?)?,
            add(
                mul(expressions.len(), size_of::<AtomView>())?,
                add(tables, mul(tags, size_of::<Atom>())?)?,
            )?,
        )?;
        // Native add_aliases keeps the cloned source call plus four transient
        // tag copies while installing one map key. Table replacement retains
        // old/new buckets. Source atoms remain live throughout.
        let validation = add(
            table::<AliasKey<'_>, (), C::Error>(aliases.len())?,
            table::<Symbol, usize, C::Error>(aliases.len())?,
        )?;
        let temporary = add(
            mul(6, payload)?,
            add(
                mul(2, tables)?.max(validation),
                mul(mul(4, tags)?, size_of::<Atom>())?,
            )?,
        )?;
        let extent = OperationExtent {
            stage: OperationStage::Construction,
            work: add(aliases.len(), tags)?,
            input_bytes: input,
            temporary_bytes: temporary,
            output_bytes: output,
            output_entries: 0,
        };
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        control.check(extent).map_err(OperationError::Control)?;
        // Reject invalid duplicate/signature paths without invoking native error
        // construction, which would rebuild a symbolic call through callbacks.
        let mut seen = HashSet::default();
        seen.try_reserve(aliases.len())
            .map_err(|_| OperationError::Allocation)?;
        let mut arities = HashMap::default();
        arities
            .try_reserve(aliases.len())
            .map_err(|_| OperationError::Allocation)?;
        for (from, _) in aliases {
            let (symbol, arity) =
                signature(from).ok_or(OperationError::Unsupported("alias key"))?;
            if arities
                .insert(symbol, arity)
                .is_some_and(|previous| previous != arity)
                || !seen.insert(AliasKey {
                    symbol,
                    atom: from.as_view(),
                })
            {
                return Err(OperationError::Unsupported(
                    "duplicate or inconsistent alias signature",
                ));
            }
            control
                .check(OperationExtent { work: 0, ..extent })
                .map_err(OperationError::Control)?;
        }
        drop(seen);
        drop(arities);
        IntegerAllocationContext::fresh().run(|| {
            let mut map = FunctionMap::new();
            map.add_aliases(aliases.iter().cloned())
                .map_err(|_| OperationError::Unsupported("invalid native alias map"))?;
            let builder = Self::new_multiple(expressions, parameters).function_map(map);
            control
                .check(OperationExtent { work: 0, ..extent })
                .map_err(OperationError::Control)?;
            Ok(builder)
        })
    }

    /// Build the evaluator.
    /// Build the maintained direct profile with borrowed current-operation
    /// admission. Horner iterations, parallel workers and CPE settings are kept.
    /// Unknown callbacks, retained bodies and legacy EvalTree construction refuse
    /// before growth. The returned evaluator retains no caller control.
    pub fn try_build_with<C: OperationControl + Send>(
        self,
        control: &mut C,
    ) -> Result<ExpressionEvaluator<Complex<Rational>>, OperationError<C::Error>>
    where
        C::Error: Send,
    {
        self.try_build_known_with(&[], control)
    }

    /// Build with a closed table of qualified local numerical heads. This table
    /// does not authorize custom symbolic normalization or tagged generators.
    pub fn try_build_known_with<C: OperationControl + Send>(
        self,
        heads: &[KnownHead],
        control: &mut C,
    ) -> Result<ExpressionEvaluator<Complex<Rational>>, OperationError<C::Error>>
    where
        C::Error: Send,
    {
        super::checked_build::build(
            &self.exprs,
            &self.fn_map,
            &self.params,
            self.optimization_settings,
            heads,
            control,
        )
    }

    /// Build the evaluator using the native selected translation algorithm.
    pub fn build(self) -> Result<ExpressionEvaluator<Complex<Rational>>, EvaluationError> {
        // EvalTree::linearize expands every function call, so preserving an explicit call boundary
        // requires the direct translator even when the legacy tree optimizer was requested.
        if self.optimization_settings.direct_translation || self.fn_map.has_non_inlined_functions()
        {
            AtomView::to_evaluator(
                &self.exprs,
                &self.fn_map,
                &self.params,
                self.optimization_settings,
            )
        } else {
            let tree = AtomView::to_eval_tree_multiple(&self.exprs, &self.fn_map, &self.params)?;
            Ok(tree.optimize(&self.optimization_settings))
        }
    }
}

#[cfg_attr(
    feature = "bincode",
    derive(bincode_trait_derive::Encode),
    derive(bincode_trait_derive::Decode),
    derive(bincode_trait_derive::BorrowDecodeFromDecode),
    trait_decode(trait = crate::state::HasStateMap)
)]
#[derive(Clone, Debug)]
pub(super) struct Expr {
    pub(super) id: usize,
    pub(super) tag_len: usize,
    pub(super) is_alias: bool,
    pub(super) args: Vec<Indeterminate>,
    pub(super) body: Atom,
    pub(super) options: FunctionRegistrationOptions,
}

/// Settings for optimizing the evaluation of expressions.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct OptimizationSettings {
    pub(crate) horner_iterations: usize,
    pub(crate) n_cores: usize,
    pub(crate) cpe_iterations: Option<usize>,
    pub(crate) hot_start: Option<Vec<Expression<Complex<Rational>>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(crate) abort_check: Option<Box<dyn AbortCheck>>,
    pub(crate) abort_level: usize,
    pub(crate) max_horner_scheme_variables: usize,
    pub(crate) max_common_pair_cache_entries: usize,
    pub(crate) max_common_pair_distance: usize,
    pub(crate) verbose: bool,
    pub(crate) direct_translation: bool,
}

#[cfg(feature = "bincode")]
bincode::impl_borrow_decode!(OptimizationSettings);

#[cfg(feature = "bincode")]
impl bincode::Encode for OptimizationSettings {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> core::result::Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(&self.horner_iterations, encoder)?;
        bincode::Encode::encode(&self.n_cores, encoder)?;
        bincode::Encode::encode(&self.cpe_iterations, encoder)?;
        bincode::Encode::encode(&self.hot_start, encoder)?;
        bincode::Encode::encode(&self.max_horner_scheme_variables, encoder)?;
        bincode::Encode::encode(&self.max_common_pair_cache_entries, encoder)?;
        bincode::Encode::encode(&self.max_common_pair_distance, encoder)?;
        bincode::Encode::encode(&self.verbose, encoder)?;
        bincode::Encode::encode(&self.direct_translation, encoder)?;
        Ok(())
    }
}

#[cfg(feature = "bincode")]
impl<Context> bincode::Decode<Context> for OptimizationSettings {
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            horner_iterations: bincode::Decode::decode(decoder)?,
            n_cores: bincode::Decode::decode(decoder)?,
            cpe_iterations: bincode::Decode::decode(decoder)?,
            hot_start: bincode::Decode::decode(decoder)?,
            abort_check: None,
            abort_level: 0,
            max_horner_scheme_variables: bincode::Decode::decode(decoder)?,
            max_common_pair_cache_entries: bincode::Decode::decode(decoder)?,
            max_common_pair_distance: bincode::Decode::decode(decoder)?,
            verbose: bincode::Decode::decode(decoder)?,
            direct_translation: bincode::Decode::decode(decoder)?,
        })
    }
}

impl std::fmt::Debug for OptimizationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("OptimizationSettings")
            .field("horner_iterations", &self.horner_iterations)
            .field("n_cores", &self.n_cores)
            .field("cpe_iterations", &self.cpe_iterations)
            .field("hot_start", &self.hot_start)
            .field("abort_check", &self.abort_check.is_some())
            .field("abort_level", &self.abort_level)
            .field("verbose", &self.verbose)
            .finish()
    }
}

impl Default for OptimizationSettings {
    fn default() -> Self {
        OptimizationSettings {
            horner_iterations: 10,
            n_cores: 1,
            cpe_iterations: None,
            hot_start: None,
            abort_check: None,
            abort_level: 0,
            max_horner_scheme_variables: 500,
            max_common_pair_cache_entries: 1_000_000,
            max_common_pair_distance: 1000,
            verbose: false,
            direct_translation: true,
        }
    }
}

impl OptimizationSettings {
    /// Create optimization settings with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of Horner scheme optimization iterations.
    pub fn horner_iterations(mut self, horner_iterations: usize) -> Self {
        self.horner_iterations = horner_iterations;
        self
    }

    /// Set the number of CPU cores to use during optimization.
    pub fn cores(mut self, n_cores: usize) -> Self {
        self.n_cores = n_cores;
        self
    }

    /// Set the number of common pair elimination iterations.
    pub fn cpe_iterations(mut self, cpe_iterations: Option<usize>) -> Self {
        self.cpe_iterations = cpe_iterations;
        self
    }

    /// Set a hot-start expression list for optimization.
    pub fn hot_start(mut self, hot_start: Option<Vec<Expression<Complex<Rational>>>>) -> Self {
        self.hot_start = hot_start;
        self
    }

    /// Set the abort check used during optimization.
    pub fn abort_check(mut self, abort_check: Option<Box<dyn AbortCheck>>) -> Self {
        self.abort_check = abort_check;
        self
    }

    /// Set the abort polling level.
    pub fn abort_level(mut self, abort_level: usize) -> Self {
        self.abort_level = abort_level;
        self
    }

    /// Set the maximum number of variables considered for a Horner scheme.
    pub fn max_horner_scheme_variables(mut self, max_horner_scheme_variables: usize) -> Self {
        self.max_horner_scheme_variables = max_horner_scheme_variables;
        self
    }

    /// Set the maximum number of common-pair cache entries.
    pub fn max_common_pair_cache_entries(mut self, max_common_pair_cache_entries: usize) -> Self {
        self.max_common_pair_cache_entries = max_common_pair_cache_entries;
        self
    }

    /// Set the maximum distance considered for common-pair elimination.
    pub fn max_common_pair_distance(mut self, max_common_pair_distance: usize) -> Self {
        self.max_common_pair_distance = max_common_pair_distance;
        self
    }

    /// Enable or disable verbose optimization output.
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Enable or disable direct translation.
    pub fn direct_translation(mut self, direct_translation: bool) -> Self {
        self.direct_translation = direct_translation;
        self
    }
}
