//! Borrowed controls on the shared direct translator and its actual workers.
use super::*;
use crate::coefficient::{SerializedFloat, SerializedRational};
use crate::state::{Workspace, WorkspaceObserver};
use numerica::domains::backend::integer::IntegerAllocationContext;

fn add<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_add(b).ok_or(OperationError::Overflow)
}
fn mul<E>(a: usize, b: usize) -> Result<usize, OperationError<E>> {
    a.checked_mul(b).ok_or(OperationError::Overflow)
}
fn limbs<E>(bits: usize) -> Result<usize, OperationError<E>> {
    // Native fresh GMP destinations round to limbs; arithmetic destinations can
    // retain the carry limb. Old/new destinations coexist in the working bound.
    mul(add(add(bits, 63)? / 64, 2)?, size_of::<u64>())
}

/// Common-denominator complex-rational height, including unreduced intermediates.
#[derive(Clone, Copy)]
struct Height {
    a: usize,
    d: usize,
}
impl Height {
    const ONE: Self = Self { a: 1, d: 1 };
    fn sum<E>(self, other: Self) -> Result<Self, OperationError<E>> {
        Ok(Self {
            a: add(add(self.a, other.d)?.max(add(other.a, self.d)?), 1)?,
            d: add(self.d, other.d)?,
        })
    }
    fn product<E>(self, other: Self) -> Result<Self, OperationError<E>> {
        Ok(Self {
            a: add(add(self.a, other.a)?, 1)?,
            d: add(self.d, other.d)?,
        })
    }
    fn power<E>(mut self, exponent: i64) -> Result<Self, OperationError<E>> {
        if exponent < 0 {
            self = Self {
                a: add(self.a, self.d)?,
                d: add(mul(2, self.a)?, 1)?,
            };
        }
        let n = usize::try_from(exponent.unsigned_abs()).map_err(|_| OperationError::Overflow)?;
        Ok(Self {
            a: mul(n, add(self.a, 1)?)?,
            d: mul(n, self.d)?,
        })
    }
    fn bits<E>(self) -> Result<usize, OperationError<E>> {
        add(self.a, self.d)
    }
}

#[derive(Clone, Copy)]
struct Bound {
    height: Height,
    peak: usize,
    nodes: usize,
    wide_float: bool,
}

struct State<'a, C: OperationControl> {
    control: &'a mut C,
    refusal: Option<OperationError<C::Error>>,
    height: usize,
}

struct Session<'a, C: OperationControl> {
    state: Mutex<State<'a, C>>,
    input: usize,
    nodes: usize,
    source_bytes: usize,
    parameters: usize,
    outputs: usize,
    names: usize,
    workers: usize,
    permutations: usize,
}

impl<C: OperationControl> Session<'_, C> {
    fn fail(&self, error: OperationError<C::Error>) -> ! {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_none() {
            state.refusal = Some(error);
        }
        drop(state);
        panic::resume_unwind(Box::new(Stop));
    }

    fn value<T>(&self, value: Result<T, OperationError<C::Error>>) -> T {
        match value {
            Ok(value) => value,
            Err(error) => self.fail(error),
        }
    }

    fn extent(
        &self,
        height: usize,
        stage: OperationStage,
        work: usize,
    ) -> Result<OperationExtent, OperationError<C::Error>> {
        let n = self.nodes.max(1);
        let entries = add(self.parameters, mul(24, n)?)?;
        let instructions = mul(24, n)?;
        let integer = limbs(mul(4, add(height, 2)?)?)?;
        let coefficient = add(size_of::<Complex<Rational>>(), mul(4, integer)?)?;
        // Coefficient collection and integer-power distribution introduce at most
        // a Mul/Num or Pow wrapper per original edge. No linear user heads are
        // admitted. Three nodes per original node bound every canonical result.
        let atom = add(
            self.source_bytes,
            mul(mul(3, n)?, add(64, mul(4, integer)?)?)?,
        )?;
        // Each recursive collection/replacement frame can hold input, rebuilt and
        // normalized encodings. Vec growth includes old/new geometric overlap.
        let workspace = mul(mul(3, add(n, 1)?)?, atom)?;
        // At most eight live complex coefficient values (32 integers) occur in
        // native complex/rational normalization, including unreduced cross-products.
        // Private GMP/MPFR algorithm scratch is NOT inferred from these limbs:
        // it remains the caller's separately admitted opaque foreign allowance.
        let arithmetic = mul(add(n, 1)?, mul(8, coefficient)?)?;
        let output = add(
            add(
                mul(entries, coefficient)?,
                mul(
                    instructions,
                    add(size_of::<(Instr, ComplexPhase)>(), 4 * size_of::<usize>())?,
                )?,
            )?,
            add(
                mul(self.outputs, size_of::<usize>())?,
                add(
                    self.names,
                    mul(n, size_of::<ExternalFunctionContainer<Complex<Rational>>>())?,
                )?,
            )?,
        )?;
        // Lowering simultaneously retains public Instruction/Slot and private
        // Instr forms, constant/alias/subexpression tables and branch maps. A
        // pair belongs to at most one operand pair occurrence, so all CPE keys,
        // occurrence lists and heap records together have at most n^2 entries.
        let records = add(
            mul(n, 64 * size_of::<usize>())?,
            mul(mul(n, n)?, 32 * size_of::<usize>())?,
        )?;
        let permutation_bytes = mul(
            self.permutations,
            add(size_of::<Vec<usize>>(), 9 * size_of::<usize>())?,
        )?;
        let temporary = add(
            add(
                mul(add(self.workers, 2)?, add(workspace, arithmetic)?)?,
                mul(3, add(output, records)?)?,
            )?,
            permutation_bytes,
        )?;
        let extent = OperationExtent {
            stage,
            work,
            input_bytes: self.input,
            temporary_bytes: temporary,
            output_bytes: output,
            output_entries: entries,
        };
        extent.peak_bytes().ok_or(OperationError::Overflow)?;
        Ok(extent)
    }

    fn check(&self, height: usize, stage: OperationStage, work: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_some() {
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
        let height = height.max(state.height);
        let extent = self.extent(height, stage, work);
        let result = match extent {
            Ok(extent) => state.control.check(extent).map_err(OperationError::Control),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) => state.height = height,
            Err(error) => {
                state.refusal = Some(error);
                drop(state);
                panic::resume_unwind(Box::new(Stop));
            }
        }
    }

    fn finalize(&self, product: &ExpressionEvaluator<Complex<Rational>>) {
        let resident = self.value(super::checked_copy::extent(product, control::payload));
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.refusal.is_some() {
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
        let extent = self
            .extent(state.height, OperationStage::Finalization, 0)
            .map(|extent| OperationExtent {
                output_bytes: resident.input_bytes,
                output_entries: product.stack.len(),
                ..extent
            });
        let checked = match extent {
            Ok(extent) => state.control.check(extent).map_err(OperationError::Control),
            Err(error) => Err(error),
        };
        if let Err(error) = checked {
            state.refusal = Some(error);
            drop(state);
            panic::resume_unwind(Box::new(Stop));
        }
    }

    fn float(&self, value: SerializedFloat<'_>) -> (Height, bool) {
        let precision = value
            .0
            .get(..4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u32::from_le_bytes)
            .unwrap_or(0) as usize;
        if precision == 0 {
            self.fail(OperationError::Unsupported(
                "invalid serialized float precision",
            ));
        }
        // Stage one precedes MPFR parsing/decoding. Parsing text and precision
        // backing is admitted without materializing an integer ratio.
        self.check(
            self.value(add(precision, self.value(mul(value.0.len(), 8)))),
            OperationStage::Construction,
            0,
        );
        let decoded = value.to_float();
        let (numerator, denominator) = decoded.rational_bit_bounds().unwrap_or_else(|| {
            self.fail(OperationError::Unsupported(
                "nonfinite or unqualified float backend",
            ))
        });
        let a = usize::try_from(numerator).unwrap_or_else(|_| self.fail(OperationError::Overflow));
        let d =
            usize::try_from(denominator).unwrap_or_else(|_| self.fail(OperationError::Overflow));
        let wide = a.max(d) > i32::MAX as usize;
        (Height { a, d }, wide)
    }

    fn coefficient(&self, value: CoefficientView<'_>) -> (Height, bool) {
        fn rational<E>(value: SerializedRational<'_>) -> Result<(usize, usize), OperationError<E>> {
            match value {
                SerializedRational::Natural(n, d) => Ok((
                    (64 - n.unsigned_abs().leading_zeros()) as usize,
                    (64 - d.unsigned_abs().leading_zeros()) as usize,
                )),
                SerializedRational::Large(value) => Ok((
                    mul(value.num_digits.len(), 8)?,
                    mul(value.den_digits.len(), 8)?,
                )),
            }
        }
        let ((nr, dr), (ni, di), wide) = match value {
            CoefficientView::Natural(nr, dr, ni, di) => (
                self.value(rational(SerializedRational::Natural(nr, dr))),
                self.value(rational(SerializedRational::Natural(ni, di))),
                false,
            ),
            CoefficientView::Large(r, i) => {
                (self.value(rational(r)), self.value(rational(i)), false)
            }
            CoefficientView::Float(r, i) => {
                let (r, rw) = self.float(r);
                let (i, iw) = self.float(i);
                ((r.a, r.d), (i.a, i.d), rw || iw)
            }
            _ => self.fail(OperationError::Unsupported(
                "checked construction coefficient domain",
            )),
        };
        (
            Height {
                a: self.value(add(nr, di)).max(self.value(add(ni, dr))),
                d: self.value(add(dr.max(1), di.max(1))),
            },
            wide,
        )
    }

    fn bound(&self, atom: AtomView<'_>) -> Bound {
        let mut height = Height::ONE;
        let mut peak = 2;
        let mut nodes = 1;
        let mut wide_float = false;
        match atom {
            AtomView::Num(number) => {
                (height, wide_float) = self.coefficient(number.get_coeff_view());
            }
            AtomView::Var(_) => {}
            AtomView::Add(_) | AtomView::Mul(_) | AtomView::Fun(_) => {
                let children = match atom {
                    AtomView::Add(a) => a.iter(),
                    AtomView::Mul(m) => m.iter(),
                    AtomView::Fun(f) => f.iter(),
                    _ => unreachable!(),
                };
                for child in children {
                    let child = self.bound(child);
                    nodes = self.value(add(nodes, child.nodes));
                    peak = peak.max(child.peak);
                    wide_float |= child.wide_float;
                    height = if matches!(atom, AtomView::Add(_)) {
                        self.value(height.sum(child.height))
                    } else if matches!(atom, AtomView::Mul(_)) {
                        self.value(height.product(child.height))
                    } else {
                        Height::ONE
                    };
                }
            }
            AtomView::Pow(power) => {
                let (base, exponent) = power.get_base_exp();
                let base = self.bound(base);
                let exp_bound = self.bound(exponent);
                nodes = self.value(add(1, self.value(add(base.nodes, exp_bound.nodes))));
                peak = base.peak.max(exp_bound.peak);
                wide_float = base.wide_float || exp_bound.wide_float;
                height = if let AtomView::Num(exp) = exponent
                    && let CoefficientView::Natural(n, 1, 0, _) = exp.get_coeff_view()
                {
                    self.value(base.height.power(n))
                } else {
                    base.height
                };
            }
        }
        peak = peak.max(self.value(height.bits()));
        Bound {
            height,
            peak,
            nodes,
            wide_float,
        }
    }
}

impl<C: OperationControl + Send> WorkspaceObserver for Session<'_, C>
where
    C::Error: Send,
{
    fn enter(&self, atom: AtomView<'_>) -> usize {
        let bound = self.bound(atom);
        if bound.wide_float && (atom.needs_normalization() || matches!(atom, AtomView::Fun(_))) {
            self.fail(OperationError::Unsupported(
                "native float arithmetic precision/exponent range",
            ));
        }
        // sqrt/log/exp can raise MPFR precision. Their visible precision bound
        // is admitted from the actual current operands before normalization;
        // nested results are scanned again before the next native operation.
        self.check(
            self.value(add(self.value(mul(2, bound.peak)), 64)),
            OperationStage::Construction,
            1,
        );
        0
    }
    fn leave(&self, _: usize) {}
    fn power(&self, base: CoefficientView<'_>, exponent: CoefficientView<'_>) {
        let (height, wide) = self.coefficient(base);
        if wide {
            self.fail(OperationError::Unsupported(
                "native float arithmetic precision/exponent range",
            ));
        }
        let height = match exponent {
            CoefficientView::Natural(n, 1, 0, _) => self.value(height.power(n)),
            _ if matches!(base, CoefficientView::Float(..))
                || matches!(exponent, CoefficientView::Float(..)) =>
            {
                let (other, wide) = self.coefficient(exponent);
                if wide {
                    self.fail(OperationError::Unsupported(
                        "native float arithmetic precision/exponent range",
                    ));
                }
                Height {
                    a: self.value(add(self.value(height.bits()), self.value(other.bits()))),
                    d: 64,
                }
            }
            _ => self.fail(OperationError::Unsupported(
                "new exact numerical fractional-power normalization",
            )),
        };
        self.check(self.value(height.bits()), OperationStage::Construction, 1);
    }
}

impl<C: OperationControl + Send> BuildObserver for Session<'_, C>
where
    C::Error: Send,
{
    fn horner(&self, atom: AtomView<'_>) {
        let bound = self.bound(atom);
        self.check(bound.peak, OperationStage::Construction, 1);
    }
    fn linearization(&self, atom: AtomView<'_>) {
        let bound = self.bound(atom);
        self.check(bound.peak, OperationStage::Construction, 1);
    }
    fn float_rational(&self, value: &Float) {
        let (numerator, denominator) = value.rational_bit_bounds().unwrap_or_else(|| {
            self.fail(OperationError::Unsupported(
                "nonfinite or unqualified float backend",
            ))
        });
        let bits = numerator
            .checked_add(denominator)
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or_else(|| self.fail(OperationError::Overflow));
        // Stage two precedes the actual exponent-sized dyadic allocation.
        self.check(bits, OperationStage::Construction, 1);
    }
    fn stage(&self, stage: OperationStage) {
        self.check(1, stage, 0);
    }
}

fn alias<'a>(map: &'a FunctionMap, atom: AtomView<'_>) -> Option<&'a Expr> {
    map.tagged_fn_map.iter().find_map(|((symbol, tags), body)| {
        let matches = match atom {
            AtomView::Var(v) => v.get_symbol() == *symbol && tags.is_empty(),
            AtomView::Fun(f) => {
                f.get_symbol() == *symbol
                    && f.get_nargs() == tags.len()
                    && f.iter().zip(tags).all(|(a, b)| a == b.as_view())
            }
            _ => false,
        };
        matches.then_some(body)
    })
}

fn qualify<E>(
    atom: AtomView<'_>,
    params: &[Atom],
    map: &FunctionMap,
    heads: &[KnownHead],
    depth: usize,
    opaque: bool,
) -> Result<(usize, usize), OperationError<E>> {
    if depth
        > map
            .tagged_fn_map
            .len()
            .checked_add(1)
            .ok_or(OperationError::Overflow)?
    {
        return Err(OperationError::Unsupported("cyclic alias construction"));
    }
    let symbol = match atom {
        AtomView::Var(v) => Some(v.get_symbol()),
        AtomView::Fun(f) => Some(f.get_symbol()),
        _ => None,
    };
    let is_parameter = params.iter().any(|p| p.as_view() == atom);
    let mut names = 0;
    if let Some(symbol) = symbol {
        if symbol.get_normalization_function().is_some() || symbol.is_linear() {
            return Err(OperationError::Unsupported(
                "custom or linear-head normalization",
            ));
        }
        if let Some(info) = symbol.get_evaluation_info()
            && info.has_evaluator::<Complex<Float>>()
        {
            return Err(OperationError::Unsupported(
                "custom floating-point normalization callback",
            ));
        }
        names = symbol
            .get_aliases()
            .iter()
            .map(|a| a.len())
            .max()
            .unwrap_or(0)
            .max(symbol.get_name().len());
        if !opaque && !is_parameter {
            if let Some(body) = alias(map, atom) {
                let (nodes, body_names) =
                    qualify(body.body.as_view(), params, map, heads, depth + 1, false)?;
                let mut nodes = add(nodes, 1)?;
                names = add(names, body_names)?;
                // Lowering treats the whole fixed-tag alias as a definition;
                // Horner still visits its original arguments before lowering.
                if let AtomView::Fun(f) = atom {
                    for child in f {
                        let (n, child_names) = qualify(child, params, map, heads, depth, true)?;
                        nodes = add(nodes, n)?;
                        names = add(names, child_names)?;
                    }
                }
                return Ok((nodes, names));
            }
            let builtin = symbol.is_fixed_builtin();
            if !builtin {
                let head = KnownHead::find::<E>(heads, symbol)?;
                let arity = if let AtomView::Fun(f) = atom {
                    f.get_nargs()
                } else {
                    0
                };
                if arity != head.arity {
                    return Err(OperationError::Unsupported("qualified head arity"));
                }
            }
        }
    }
    let mut nodes = 1;
    match atom {
        AtomView::Add(a) => {
            for child in a {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Mul(m) => {
            for child in m {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Fun(f) => {
            for child in f {
                let (n, names_) =
                    qualify(child, params, map, heads, depth, opaque || is_parameter)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Pow(p) => {
            for child in [p.get_base(), p.get_exp()] {
                let (n, names_) = qualify(child, params, map, heads, depth, opaque)?;
                nodes = add(nodes, n)?;
                names = add(names, names_)?;
            }
        }
        AtomView::Num(n) => {
            if !matches!(
                n.get_coeff_view(),
                CoefficientView::Natural(..)
                    | CoefficientView::Large(..)
                    | CoefficientView::Float(..)
            ) {
                return Err(OperationError::Unsupported(
                    "checked construction coefficient domain",
                ));
            }
        }
        AtomView::Var(_) => {}
    }
    Ok((nodes, names))
}

pub(super) fn build<C: OperationControl + Send>(
    expressions: &[AtomView<'_>],
    map: &FunctionMap,
    params: &[Atom],
    settings: OptimizationSettings,
    heads: &[KnownHead],
    control: &mut C,
) -> Result<ExpressionEvaluator<Complex<Rational>>, OperationError<C::Error>>
where
    C::Error: Send,
{
    control
        .check(OperationExtent {
            stage: OperationStage::Construction,
            ..OperationExtent::default()
        })
        .map_err(OperationError::Control)?;
    if !settings.direct_translation {
        return Err(OperationError::Unsupported(
            "legacy evaluation-tree construction",
        ));
    }
    if settings.hot_start.is_some() || settings.abort_check.is_some() || settings.abort_level != 0 {
        return Err(OperationError::Unsupported(
            "retained hot-start or legacy abort callback",
        ));
    }
    if settings.n_cores == 0 {
        return Err(OperationError::Unsupported("zero Horner worker count"));
    }
    if map
        .tagged_fn_map
        .values()
        .any(|body| !body.is_alias || !body.args.is_empty() || !body.options.should_inline())
    {
        return Err(OperationError::Unsupported(
            "non-alias or retained function definitions",
        ));
    }
    let mut nodes = 0;
    let mut names = 0;
    let mut source_bytes = 0;
    for atom in expressions
        .iter()
        .copied()
        .chain(params.iter().map(Atom::as_view))
        .chain(map.tagged_fn_map.values().map(|body| body.body.as_view()))
    {
        let (n, names_) = qualify(atom, params, map, heads, 0, false)?;
        nodes = add(nodes, n)?;
        names = add(names, names_)?;
        source_bytes = add(source_bytes, atom.get_byte_size())?;
    }
    let mut input = add(source_bytes, mul(params.len(), size_of::<Atom>())?)?;
    input = add(input, mul(expressions.len(), size_of::<AtomView>())?)?;
    for atom in params {
        input = add(input, atom.get_capacity())?;
    }
    for ((_, tags), body) in &map.tagged_fn_map {
        input = add(input, body.body.get_capacity())?;
        input = add(input, mul(tags.capacity(), size_of::<Atom>())?)?;
        for tag in tags {
            input = add(input, tag.get_capacity())?;
        }
    }
    input = add(
        input,
        mul(
            map.tagged_fn_map
                .capacity()
                .checked_next_power_of_two()
                .ok_or(OperationError::Overflow)?,
            2 * (size_of::<((Symbol, Vec<Atom>), Expr)>() + 16),
        )?,
    )?;
    input = add(
        input,
        mul(
            map.tag
                .capacity()
                .checked_next_power_of_two()
                .ok_or(OperationError::Overflow)?,
            2 * (size_of::<(Symbol, usize)>() + 16),
        )?,
    )?;
    let mut permutations = 0;
    let mut factorial = 1usize;
    for count in 1..=9.min(nodes).min(settings.max_horner_scheme_variables) {
        factorial = mul(factorial, count)?;
        if factorial <= settings.horner_iterations {
            permutations = factorial;
        }
    }
    let workers = LicenseManager::max_threads(settings.n_cores)
        .min(settings.horner_iterations.saturating_sub(1));
    let session = Session {
        state: Mutex::new(State {
            control,
            refusal: None,
            height: 1,
        }),
        input,
        nodes,
        source_bytes,
        parameters: params.len(),
        outputs: expressions.len(),
        names,
        workers,
        permutations,
    };
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        IntegerAllocationContext::fresh().run(|| {
            session.check(1, OperationStage::Construction, 0);
            for expression in expressions {
                let bound = session.bound(*expression);
                session.check(bound.peak, OperationStage::Construction, 0);
            }
            for body in map.tagged_fn_map.values() {
                let bound = session.bound(body.body.as_view());
                session.check(bound.peak, OperationStage::Construction, 0);
            }
            let product =
                AtomView::to_evaluator_observed(expressions, map, params, settings, Some(&session));
            if let Ok(product) = &product {
                session.finalize(product);
            }
            product
        })
    }));
    let state = session
        .state
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match result {
        Err(payload) if payload.is::<Stop>() => {
            Err(state.refusal.unwrap_or(OperationError::Interrupted))
        }
        Err(payload) => panic::resume_unwind(payload),
        Ok(_) if state.refusal.is_some() => Err(state.refusal.unwrap()),
        Ok(result) => result.map_err(|error| OperationError::InvalidProgram(error.to_string())),
    }
}

/// Private unwind payload for an already-recorded original typed refusal.
pub(crate) struct Stop;

pub(crate) fn preserve_panic(
    current: &mut Option<Box<dyn std::any::Any + Send>>,
    incoming: Box<dyn std::any::Any + Send>,
) {
    if current
        .as_ref()
        .is_none_or(|previous| previous.is::<Stop>() && !incoming.is::<Stop>())
    {
        *current = Some(incoming);
    }
}

pub(crate) trait BuildObserver: WorkspaceObserver {
    fn horner(&self, atom: AtomView<'_>);
    fn linearization(&self, atom: AtomView<'_>);
    fn float_rational(&self, value: &Float);
    fn stage(&self, stage: OperationStage);
}

pub(crate) fn horner(
    atom: AtomView<'_>,
    scheme: &[Indeterminate],
    observer: Option<&dyn BuildObserver>,
) -> Atom {
    if let Some(observer) = observer {
        observer.horner(atom);
        let workspace = Workspace::controlled(observer);
        atom.horner_scheme_in(&workspace, Some(scheme), true, true)
    } else {
        atom.horner_scheme(Some(scheme), true, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domains::backend::integer::IntegerAllocationContext, parse};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn checked_builder_preserves_horner_products_and_reports_retained_backing() {
        let x = parse!("checked_builder_x");
        let expression = parse!("(2*checked_builder_x+2)^128 + checked_builder_x^2");
        let mut final_bytes = 0;
        let product = expression
            .evaluator(&[x.clone()])
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .cpe_iterations(Some(3))
            .try_build_with(&mut |extent: OperationExtent| {
                if extent.stage == OperationStage::Finalization {
                    final_bytes = extent.output_bytes;
                }
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        let resident = super::super::checked_copy::extent::<_, std::convert::Infallible>(
            &product,
            control::payload,
        )
        .unwrap();
        assert_eq!(final_bytes, resident.input_bytes);
        assert_eq!(product.settings.horner_iterations, 16);
        assert_eq!(product.settings.n_cores, 4);
        assert_eq!(product.settings.cpe_iterations, Some(3));
        assert!(product.settings.abort_check.is_none());
        let mut product = product.map_coeff(&|c| c.re.to_f64());
        let mut baseline = expression
            .evaluator(&[x])
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .cpe_iterations(Some(3))
            .build()
            .unwrap()
            .map_coeff(&|c| c.re.to_f64());
        for x in [0., 0.25, 1.] {
            assert_eq!(
                product.evaluate_single(&[x]),
                baseline.evaluate_single(&[x])
            );
        }
    }

    #[test]
    fn checked_builder_preserves_original_typed_refusal_and_sticky_worker_stop() {
        #[derive(Debug, PartialEq, Eq)]
        struct Refused(u64);
        let inputs = [
            parse!("checked_stop_a"),
            parse!("checked_stop_b"),
            parse!("checked_stop_c"),
        ];
        let expression = parse!(
            "(checked_stop_a+checked_stop_b+checked_stop_c)^2+checked_stop_a*checked_stop_b+checked_stop_b*checked_stop_c"
        );
        let mut calls = 0;
        let mut refused = false;
        let result = expression
            .evaluator(&inputs)
            .direct_translation(true)
            .horner_iterations(64)
            .cores(4)
            .try_build_with(&mut |_: OperationExtent| {
                assert!(!refused, "an original refusal is sticky across all workers");
                calls += 1;
                if calls == 40 {
                    refused = true;
                    Err(Refused(67))
                } else {
                    Ok(())
                }
            });
        assert_eq!(result.unwrap_err(), OperationError::Control(Refused(67)));
        assert!(refused);
        assert!(!IntegerAllocationContext::capture().is_fresh());
    }

    #[test]
    fn checked_builder_admits_float_precision_then_exponent_before_dyadic_growth() {
        #[derive(Debug, PartialEq, Eq)]
        struct Refused;
        let expression = Atom::num(Float::deserialize(b"1@100000", 53));
        let mut largest = 0;
        let result = expression
            .evaluator(&[] as &[Atom])
            .direct_translation(true)
            .horner_iterations(0)
            .try_build_with(&mut |extent: OperationExtent| {
                largest = largest.max(extent.peak_bytes().unwrap());
                if largest > 8_000_000 {
                    Err(Refused)
                } else {
                    Ok(())
                }
            });
        assert_eq!(result.unwrap_err(), OperationError::Control(Refused));
        assert!(largest > 8_000_000);
    }

    #[test]
    fn checked_builder_keeps_fixed_tag_aliases_and_function_shaped_inputs() {
        let head = crate::symbol!(
            "checked_builder_known_head",
            eval = EvaluationInfo::new().register(|args: &[f64]| args[0] + 3.)
        );
        let parameter = parse!("checked_builder_opaque(checked_builder_local)");
        let alias = parse!("checked_builder_alias(7)");
        let expression = head.call(&[parameter.clone()][..]) + &alias;
        let definitions = [(alias, &parameter * Atom::num(2))];
        let heads = [KnownHead {
            symbol: head,
            arity: 1,
            first: [None; 2],
            second: [None; 3],
        }];
        let product = expression
            .evaluator(&[parameter])
            .add_aliases(definitions)
            .unwrap()
            .direct_translation(true)
            .horner_iterations(16)
            .cores(4)
            .try_build_known_with(&heads, &mut |_: OperationExtent| {
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap();
        let mut numeric = product.map_coeff(&|c| c.re.to_f64());
        assert_eq!(numeric.evaluate_single(&[2.]), 9.);
    }

    #[test]
    fn checked_builder_refuses_custom_normalization_before_callback_or_growth() {
        static INVOCATIONS: AtomicUsize = AtomicUsize::new(0);
        let head = crate::symbol!(
            "checked_builder_custom_normalization",
            norm = |_, _| {
                INVOCATIONS.fetch_add(1, Ordering::SeqCst);
            }
        );
        let x = parse!("checked_builder_custom_x");
        let mut expression = Atom::new();
        expression.to_fun(head).add_arg(x.as_view());
        let mut reports = 0;
        let result = expression
            .evaluator(&[x])
            .direct_translation(true)
            .try_build_with(&mut |extent: OperationExtent| {
                reports += 1;
                assert_eq!(extent.output_bytes, 0);
                Ok::<_, std::convert::Infallible>(())
            });
        assert!(matches!(result, Err(OperationError::Unsupported(_))));
        assert_eq!(reports, 1);
        assert_eq!(INVOCATIONS.load(Ordering::SeqCst), 0);
    }

    #[derive(Default)]
    struct Probe {
        active: AtomicUsize,
        powers: AtomicUsize,
        threads: Mutex<std::collections::HashSet<std::thread::ThreadId>>,
        stop_power: bool,
    }
    impl WorkspaceObserver for Probe {
        fn enter(&self, _: AtomView<'_>) -> usize {
            assert!(IntegerAllocationContext::capture().is_fresh());
            self.active.fetch_add(1, Ordering::SeqCst);
            1
        }
        fn leave(&self, n: usize) {
            self.active.fetch_sub(n, Ordering::SeqCst);
        }
        fn power(&self, _: CoefficientView<'_>, exponent: CoefficientView<'_>) {
            self.powers.fetch_add(1, Ordering::SeqCst);
            if self.stop_power
                && matches!(exponent, CoefficientView::Natural(n, 1, 0, 1) if n > 128)
            {
                panic::resume_unwind(Box::new(Stop));
            }
        }
    }
    impl BuildObserver for Probe {
        fn horner(&self, _: AtomView<'_>) {
            assert!(IntegerAllocationContext::capture().is_fresh());
            self.threads
                .lock()
                .unwrap()
                .insert(std::thread::current().id());
        }
        fn linearization(&self, _: AtomView<'_>) {}
        fn float_rational(&self, _: &Float) {}
        fn stage(&self, _: OperationStage) {}
    }

    #[test]
    fn controlled_horner_normalizer_stops_before_factored_integer_power() {
        let expression = parse!("(2*controlled_horner_x+2)^1000000");
        let observer = Probe {
            stop_power: true,
            ..Probe::default()
        };
        let stopped = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            IntegerAllocationContext::fresh()
                .run(|| horner(expression.as_view(), &[], Some(&observer)))
        }));
        assert!(stopped.unwrap_err().is::<Stop>());
        assert!(observer.powers.load(Ordering::SeqCst) > 0);
        assert_eq!(
            observer.active.load(Ordering::SeqCst),
            0,
            "normalization leases unwind"
        );
        assert!(!IntegerAllocationContext::capture().is_fresh());
    }

    #[test]
    fn controlled_horner_workers_keep_native_parallel_search_and_values() {
        let parameters = [
            parse!("controlled_parallel_a"),
            parse!("controlled_parallel_b"),
            parse!("controlled_parallel_c"),
            parse!("controlled_parallel_d"),
        ];
        let expression = parse!(
            "(controlled_parallel_a+controlled_parallel_b+controlled_parallel_c+controlled_parallel_d)^2+controlled_parallel_a*controlled_parallel_b+controlled_parallel_b*controlled_parallel_c+controlled_parallel_c*controlled_parallel_d"
        );
        let settings = OptimizationSettings {
            direct_translation: true,
            horner_iterations: 32,
            n_cores: 4,
            ..OptimizationSettings::default()
        };
        let observer = Probe::default();
        let evaluator = IntegerAllocationContext::fresh()
            .run(|| {
                AtomView::to_evaluator_observed(
                    &[expression.as_view()],
                    &FunctionMap::new(),
                    &parameters,
                    settings.clone(),
                    Some(&observer),
                )
            })
            .unwrap();
        let baseline = AtomView::to_evaluator(
            &[expression.as_view()],
            &FunctionMap::new(),
            &parameters,
            settings,
        )
        .unwrap();
        assert_eq!(observer.active.load(Ordering::SeqCst), 0);
        assert_eq!(
            observer.threads.lock().unwrap().len(),
            LicenseManager::max_threads(4).min(31)
        );
        let mut actual = evaluator.map_coeff(&|c| c.re.to_f64());
        let mut expected = baseline.map_coeff(&|c| c.re.to_f64());
        for inputs in [[1., 2., 3., 4.], [0., -2., 0.5, 4.]] {
            assert_eq!(
                actual.evaluate_single(&inputs),
                expected.evaluate_single(&inputs)
            );
        }
    }
}
