//! Checked transfer of the existing straight-line binary64 interpreter program.
//!
//! This is execution correspondence, not symbolic/preparation-state serialization.
//! Inspection is allocation-free and passive. Restoration asks the caller to admit
//! actual requested backing before allocating validation scratch or evaluator storage.
use super::{
    ComplexPhase, ExpressionEvaluator, Instr, OperationCount, OptimizationSettings, Symbol,
};
use std::{collections::TryReserveError, error::Error, fmt};

const MAGIC: &[u8; 8] = b"SYMEVF64";
const VERSION: u16 = 1;
const HEADER: usize = 8 + 2 + 2 + 5 * 8;

/// Actual populations and requested Rust backing for one inspected program.
///
/// These are derived facts, never serialized allocation allowances. Requested
/// backing excludes allocator metadata/RSS, borrowed encoded bytes, and outer
/// native owners. Validation scratch is released before final backing is allocated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct F64ProgramExtents {
    /// Complete fixed-profile encoded byte length.
    pub encoded_bytes: usize,
    /// Parameter slots at the beginning of the stack.
    pub parameters: usize,
    /// End of the protected parameter/constant prefix.
    pub reserved: usize,
    /// Exact constant population; `reserved - parameters`.
    pub constants: usize,
    /// Complete numerical stack extent, including unused temporary slots.
    pub stack: usize,
    /// Ordered result indices, including aliases and repeated results.
    pub results: usize,
    /// Ordered supplier instructions.
    pub instructions: usize,
    /// All Add/Mul operand entries, in their original order and multiplicity.
    pub nary_operands: usize,
    /// Requested f64 stack backing.
    pub stack_bytes: usize,
    /// Requested instruction-vector and nested operand-vector backing.
    pub instruction_bytes: usize,
    /// Requested result-index backing.
    pub result_bytes: usize,
    /// Requested definition-bitset backing used before materialization.
    pub validation_bytes: usize,
    /// Requested final vector backing; the evaluator value itself is caller-owned.
    pub retained_bytes: usize,
    /// Maximum requested backing during this restoration, excluding borrowed input.
    pub peak_bytes: usize,
}

/// A structural, supported-profile or encoding refusal; no evaluator is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum F64TransferError {
    /// The fixed-profile header or version is unsupported.
    Header,
    /// A checked size/index calculation cannot be represented on this platform.
    Overflow(&'static str),
    /// The exact encoded extent is truncated, overlong or inconsistent.
    Encoding(&'static str),
    /// Unsupported supplier state or instruction.
    Unsupported(&'static str),
    /// An index or protected destination is invalid.
    Index(&'static str, usize),
    /// A temporary is consumed before any preceding instruction defines it.
    Undefined(usize),
    /// The caller supplied a differently sized encoding destination.
    Destination { expected: usize, actual: usize },
}
impl fmt::Display for F64TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "checked f64 program: {self:?}")
    }
}
impl Error for F64TransferError {}

/// Original admission, structural/dataflow, and allocation failures remain distinct.
#[derive(Debug)]
pub enum F64RestoreError<E> {
    /// Original supplier profile or dataflow refusal.
    Program(F64TransferError),
    /// The caller's unchanged admission/cancellation error payload.
    Admission(E),
    /// Original fallible reservation failure.
    Allocation(TryReserveError),
}
impl<E> From<F64TransferError> for F64RestoreError<E> {
    fn from(error: F64TransferError) -> Self {
        Self::Program(error)
    }
}
impl<E: fmt::Display> fmt::Display for F64RestoreError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(error) => error.fmt(f),
            Self::Admission(error) => error.fmt(f),
            Self::Allocation(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for F64RestoreError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Program(error) => error,
            Self::Admission(error) => error,
            Self::Allocation(error) => error,
        })
    }
}

fn add(a: usize, b: usize, what: &'static str) -> Result<usize, F64TransferError> {
    a.checked_add(b).ok_or(F64TransferError::Overflow(what))
}
fn backing<T>(entries: usize, what: &'static str) -> Result<usize, F64TransferError> {
    std::alloc::Layout::array::<T>(entries)
        .map(|layout| layout.size())
        .map_err(|_| F64TransferError::Overflow(what))
}
fn shape(
    parameters: usize,
    reserved: usize,
    stack: usize,
    instructions: usize,
    results: usize,
) -> Result<F64ProgramExtents, F64TransferError> {
    if parameters > reserved || reserved > stack {
        return Err(F64TransferError::Encoding(
            "parameter/constant/stack boundaries",
        ));
    }
    let constants = reserved - parameters;
    let stack_bytes = backing::<f64>(stack, "stack backing")?;
    let result_bytes = backing::<usize>(results, "result backing")?;
    let instruction_bytes = backing::<(Instr, ComplexPhase)>(instructions, "instruction backing")?;
    let validation_bytes = stack / 8 + usize::from(stack % 8 != 0);
    backing::<u8>(validation_bytes, "validation backing")?;
    let encoded_bytes = add(
        add(
            HEADER,
            backing::<u64>(constants, "constant encoding")?,
            "constant encoding",
        )?,
        backing::<u64>(results, "result encoding")?,
        "result encoding",
    )?;
    Ok(F64ProgramExtents {
        encoded_bytes,
        parameters,
        reserved,
        constants,
        stack,
        results,
        instructions,
        nary_operands: 0,
        stack_bytes,
        instruction_bytes,
        result_bytes,
        validation_bytes,
        retained_bytes: 0,
        peak_bytes: 0,
    })
}
impl F64ProgramExtents {
    fn finish(mut self) -> Result<Self, F64TransferError> {
        backing::<u8>(self.encoded_bytes, "encoded backing")?;
        self.instruction_bytes = add(
            self.instruction_bytes,
            backing::<usize>(self.nary_operands, "operand backing")?,
            "instruction backing",
        )?;
        self.retained_bytes = add(
            add(self.stack_bytes, self.instruction_bytes, "retained backing")?,
            self.result_bytes,
            "retained backing",
        )?;
        self.peak_bytes = self.retained_bytes.max(self.validation_bytes);
        Ok(self)
    }
}

/// Borrowed native program. Profile/indices/layout are checked without allocation;
/// complete definition-before-read validation is performed by the receiving restore.
pub struct F64ProgramRef<'a> {
    evaluator: &'a ExpressionEvaluator<f64>,
    extents: F64ProgramExtents,
}
impl fmt::Debug for F64ProgramRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("F64ProgramRef")
            .field("extents", &self.extents)
            .finish_non_exhaustive()
    }
}
impl ExpressionEvaluator<f64> {
    /// Borrow the supported fixed f64 execution profile, without exporting symbols,
    /// callbacks, optimization settings, hot starts or warmed stack values.
    pub fn f64_transfer(&self) -> Result<F64ProgramRef<'_>, F64TransferError> {
        if !self.external_fns.is_empty() {
            return Err(F64TransferError::Unsupported("external function state"));
        }
        let mut extents = shape(
            self.param_count,
            self.reserved_indices,
            self.stack.len(),
            self.instructions.len(),
            self.result_indices.len(),
        )?;
        for index in &self.result_indices {
            check_index(*index, extents.stack, "result")?;
        }
        for (instruction, phase) in &self.instructions {
            let wire = NativeInstruction::from_instr(instruction, *phase)?;
            validate_instruction(wire, &mut extents)?;
            extents.encoded_bytes = add(
                extents.encoded_bytes,
                wire.encoded_len()?,
                "instruction encoding",
            )?;
        }
        Ok(F64ProgramRef {
            evaluator: self,
            extents: extents.finish()?,
        })
    }
}
impl F64ProgramRef<'_> {
    /// Count the original supplier operations with checked accumulation, allocating nothing.
    pub fn operation_count(&self) -> Result<OperationCount, F64TransferError> {
        let mut count = OperationCount::default();
        for (instruction, phase) in &self.evaluator.instructions {
            NativeInstruction::from_instr(instruction, *phase)?.count_into(&mut count)?;
        }
        Ok(count)
    }
    /// Actual derived storage and encoded extents; this does not grant admission.
    pub fn extents(&self) -> F64ProgramExtents {
        self.extents
    }
    /// Exact caller-provisioned encoding extent.
    pub fn encoded_len(&self) -> usize {
        self.extents.encoded_bytes
    }
    /// Write into an exactly caller-provisioned destination, allocating nothing.
    pub fn write_into(&self, output: &mut [u8]) -> Result<(), F64TransferError> {
        if output.len() != self.extents.encoded_bytes {
            return Err(F64TransferError::Destination {
                expected: self.extents.encoded_bytes,
                actual: output.len(),
            });
        }
        let mut writer = Writer {
            bytes: output,
            position: 0,
        };
        writer.bytes(MAGIC);
        writer.bytes(&VERSION.to_le_bytes());
        writer.bytes(&0u16.to_le_bytes());
        for count in [
            self.extents.parameters,
            self.extents.reserved,
            self.extents.stack,
            self.extents.instructions,
            self.extents.results,
        ] {
            writer.index(count);
        }
        for constant in &self.evaluator.stack[self.extents.parameters..self.extents.reserved] {
            writer.bytes(&constant.to_bits().to_le_bytes());
        }
        for result in &self.evaluator.result_indices {
            writer.index(*result);
        }
        for (instruction, phase) in &self.evaluator.instructions {
            NativeInstruction::from_instr(instruction, *phase)?.write(&mut writer);
        }
        debug_assert_eq!(writer.position, self.extents.encoded_bytes);
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Operands<'a> {
    Native(&'a [usize]),
    Encoded(&'a [u8]),
}
impl Operands<'_> {
    fn len(self) -> usize {
        match self {
            Self::Native(values) => values.len(),
            Self::Encoded(bytes) => bytes.len() / 8,
        }
    }
    fn get(self, index: usize) -> usize {
        match self {
            Self::Native(values) => values[index],
            Self::Encoded(bytes) => {
                u64::from_le_bytes(bytes[index * 8..index * 8 + 8].try_into().unwrap()) as usize
            }
        }
    }
}
#[derive(Clone, Copy)]
struct NativeInstruction<'a> {
    opcode: u8,
    phase: ComplexPhase,
    destination: usize,
    first: usize,
    second: usize,
    power: i64,
    operands: Operands<'a>,
}
impl<'a> NativeInstruction<'a> {
    fn count_into(self, count: &mut OperationCount) -> Result<(), F64TransferError> {
        // The same scalar rules as ExpressionEvaluator::count_operations. Phases
        // do not change scalar operation populations; no execution is performed.
        match self.opcode {
            0 => {
                count.additions = add(count.additions, self.operands.len() - 1, "operation count")?
            }
            1 => {
                count.multiplications = add(
                    count.multiplications,
                    self.operands.len() - 1,
                    "operation count",
                )?
            }
            2 => {
                count.inversions = add(
                    count.inversions,
                    usize::from(self.power < 0),
                    "operation count",
                )?;
                let multiplications = usize::try_from(self.power.unsigned_abs().saturating_sub(1))
                    .map_err(|_| F64TransferError::Overflow("operation count"))?;
                count.multiplications =
                    add(count.multiplications, multiplications, "operation count")?;
            }
            _ => count.function_calls = add(count.function_calls, 1, "operation count")?,
        }
        Ok(())
    }
    fn from_instr(instruction: &'a Instr, phase: ComplexPhase) -> Result<Self, F64TransferError> {
        let mut result = Self {
            opcode: 0,
            phase,
            destination: 0,
            first: 0,
            second: 0,
            power: 0,
            operands: Operands::Native(&[]),
        };
        match instruction {
            Instr::Add(destination, operands) | Instr::Mul(destination, operands) => {
                result.opcode = u8::from(matches!(instruction, Instr::Mul(..)));
                result.destination = *destination;
                result.operands = Operands::Native(operands);
            }
            Instr::Pow(destination, base, power) => {
                result.opcode = 2;
                result.destination = *destination;
                result.first = *base;
                result.power = *power;
            }
            Instr::Powf(destination, base, power) => {
                result.opcode = 3;
                result.destination = *destination;
                result.first = *base;
                result.second = *power;
            }
            Instr::BuiltinFun(destination, symbol, argument) => {
                result.opcode = builtin_code(*symbol)?;
                result.destination = *destination;
                result.first = *argument;
            }
            _ => {
                return Err(F64TransferError::Unsupported(
                    "control flow or external instruction",
                ));
            }
        }
        Ok(result)
    }
    fn encoded_len(self) -> Result<usize, F64TransferError> {
        let phase = if matches!(self.phase, ComplexPhase::PartialReal(_)) {
            9
        } else {
            1
        };
        let payload = match self.opcode {
            0 | 1 => add(
                16,
                backing::<u64>(self.operands.len(), "operand encoding")?,
                "operand encoding",
            )?,
            2 | 3 => 24,
            _ => 16,
        };
        add(1 + phase, payload, "instruction encoding")
    }
    fn write(self, writer: &mut Writer<'_>) {
        writer.bytes(&[self.opcode]);
        match self.phase {
            ComplexPhase::Real => writer.bytes(&[0]),
            ComplexPhase::Imag => writer.bytes(&[1]),
            ComplexPhase::Any => writer.bytes(&[2]),
            ComplexPhase::PartialReal(count) => {
                writer.bytes(&[3]);
                writer.index(count);
            }
        }
        writer.index(self.destination);
        match self.opcode {
            0 | 1 => {
                writer.index(self.operands.len());
                for index in 0..self.operands.len() {
                    writer.index(self.operands.get(index));
                }
            }
            2 => {
                writer.index(self.first);
                writer.bytes(&self.power.to_le_bytes());
            }
            3 => {
                writer.index(self.first);
                writer.index(self.second);
            }
            _ => writer.index(self.first),
        }
    }
}
fn builtin_code(symbol: Symbol) -> Result<u8, F64TransferError> {
    [
        Symbol::EXP,
        Symbol::LOG,
        Symbol::SIN,
        Symbol::COS,
        Symbol::SQRT,
        Symbol::ABS,
        Symbol::CONJ,
    ]
    .iter()
    .position(|candidate| symbol == *candidate)
    .map(|index| index as u8 + 4)
    .ok_or(F64TransferError::Unsupported("builtin symbol"))
}
fn builtin_symbol(code: u8) -> Symbol {
    match code {
        4 => Symbol::EXP,
        5 => Symbol::LOG,
        6 => Symbol::SIN,
        7 => Symbol::COS,
        8 => Symbol::SQRT,
        9 => Symbol::ABS,
        10 => Symbol::CONJ,
        _ => unreachable!("inspected builtin"),
    }
}
fn check_index(index: usize, stack: usize, kind: &'static str) -> Result<(), F64TransferError> {
    if index >= stack {
        return Err(F64TransferError::Index(kind, index));
    }
    Ok(())
}
fn validate_instruction(
    instruction: NativeInstruction<'_>,
    extents: &mut F64ProgramExtents,
) -> Result<(), F64TransferError> {
    check_index(instruction.destination, extents.stack, "destination")?;
    if instruction.destination < extents.reserved {
        return Err(F64TransferError::Index(
            "protected destination",
            instruction.destination,
        ));
    }
    // Numerica 3.0.1's native f64 Real::pow takes u64 but debug-asserts this
    // range before an i32 cast. Symbolica supplies unsigned_abs for negatives.
    // A checked transfer must not turn this unsupported domain into a panic or
    // a release-build truncated exponent, nor replace it with different arithmetic.
    if instruction.opcode == 2 && instruction.power.unsigned_abs() > i32::MAX as u64 {
        return Err(F64TransferError::Unsupported(
            "integer power exceeds Numerica f64 range",
        ));
    }
    if let ComplexPhase::PartialReal(count) = instruction.phase {
        if instruction.opcode > 1 || count > instruction.operands.len() {
            return Err(F64TransferError::Unsupported(
                "PartialReal outside Add/Mul operand extent",
            ));
        }
    }
    match instruction.opcode {
        0 | 1 => {
            if instruction.operands.len() == 0 {
                return Err(F64TransferError::Encoding("empty Add/Mul"));
            }
            for index in 0..instruction.operands.len() {
                check_index(instruction.operands.get(index), extents.stack, "operand")?;
            }
            extents.nary_operands = add(
                extents.nary_operands,
                instruction.operands.len(),
                "operand population",
            )?;
        }
        2 | 4..=10 => check_index(instruction.first, extents.stack, "operand")?,
        3 => {
            check_index(instruction.first, extents.stack, "operand")?;
            check_index(instruction.second, extents.stack, "operand")?;
        }
        _ => return Err(F64TransferError::Unsupported("opcode")),
    }
    Ok(())
}

struct Writer<'a> {
    bytes: &'a mut [u8],
    position: usize,
}
impl Writer<'_> {
    fn bytes(&mut self, bytes: &[u8]) {
        let end = self.position + bytes.len();
        self.bytes[self.position..end].copy_from_slice(bytes);
        self.position = end;
    }
    fn index(&mut self, index: usize) {
        self.bytes(&(index as u64).to_le_bytes());
    }
}
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], F64TransferError> {
        let end = add(self.position, count, "cursor")?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or(F64TransferError::Encoding("truncated encoding"))?;
        self.position = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, F64TransferError> {
        Ok(self.take(1)?[0])
    }
    fn u64(&mut self) -> Result<u64, F64TransferError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn index(&mut self) -> Result<usize, F64TransferError> {
        usize::try_from(self.u64()?).map_err(|_| F64TransferError::Overflow("index/count"))
    }
    fn instruction(&mut self) -> Result<NativeInstruction<'a>, F64TransferError> {
        let opcode = self.byte()?;
        if opcode > 10 {
            return Err(F64TransferError::Unsupported("opcode"));
        }
        let phase = match self.byte()? {
            0 => ComplexPhase::Real,
            1 => ComplexPhase::Imag,
            2 => ComplexPhase::Any,
            3 => ComplexPhase::PartialReal(self.index()?),
            _ => return Err(F64TransferError::Unsupported("complex phase")),
        };
        let destination = self.index()?;
        let mut result = NativeInstruction {
            opcode,
            phase,
            destination,
            first: 0,
            second: 0,
            power: 0,
            operands: Operands::Encoded(&[]),
        };
        match opcode {
            0 | 1 => {
                let count = self.index()?;
                let bytes = self.take(backing::<u64>(count, "operand encoding")?)?;
                // Validate every encoded usize conversion before the borrowed fast accessor.
                for raw in bytes.chunks_exact(8) {
                    usize::try_from(u64::from_le_bytes(raw.try_into().unwrap()))
                        .map_err(|_| F64TransferError::Overflow("operand index"))?;
                }
                result.operands = Operands::Encoded(bytes);
            }
            2 => {
                result.first = self.index()?;
                result.power = i64::from_le_bytes(self.take(8)?.try_into().unwrap());
            }
            3 => {
                result.first = self.index()?;
                result.second = self.index()?;
            }
            _ => result.first = self.index()?,
        }
        Ok(result)
    }
}

/// Borrowed, allocation-free structural inspection. Dataflow validation and admission
/// remain separate; only `restore_with` can return the existing supplier evaluator.
#[derive(Clone, Copy, Debug)]
pub struct F64ProgramView<'a> {
    bytes: &'a [u8],
    extents: F64ProgramExtents,
    instruction_start: usize,
}
impl<'a> F64ProgramView<'a> {
    /// Count the inspected supplier instruction stream without restoring or allocating.
    /// Uses the same checked rules as the borrowed native transfer profile.
    pub fn operation_count(&self) -> Result<OperationCount, F64TransferError> {
        let mut count = OperationCount::default();
        let mut cursor = Cursor {
            bytes: self.bytes,
            position: self.instruction_start,
        };
        for _ in 0..self.extents.instructions {
            cursor.instruction()?.count_into(&mut count)?;
        }
        Ok(count)
    }
    /// Inspect the supported header, exact populations, indices and complete encoding.
    /// No symbolic state or allocation is restored, and the bytes grant no allowance.
    pub fn inspect(bytes: &'a [u8]) -> Result<Self, F64TransferError> {
        let mut cursor = Cursor { bytes, position: 0 };
        if cursor.take(8)? != MAGIC
            || cursor.take(2)? != VERSION.to_le_bytes()
            || cursor.take(2)? != [0, 0]
        {
            return Err(F64TransferError::Header);
        }
        let mut extents = shape(
            cursor.index()?,
            cursor.index()?,
            cursor.index()?,
            cursor.index()?,
            cursor.index()?,
        )?;
        cursor.take(backing::<u64>(extents.constants, "constant encoding")?)?;
        for _ in 0..extents.results {
            check_index(cursor.index()?, extents.stack, "result")?;
        }
        let instruction_start = cursor.position;
        for _ in 0..extents.instructions {
            let instruction = cursor.instruction()?;
            validate_instruction(instruction, &mut extents)?;
        }
        if cursor.position != bytes.len() {
            return Err(F64TransferError::Encoding("unconsumed encoding"));
        }
        extents.encoded_bytes = bytes.len();
        Ok(Self {
            bytes,
            extents: extents.finish()?,
            instruction_start,
        })
    }
    /// Actual derived populations and requested backing, never native admission.
    pub fn extents(&self) -> F64ProgramExtents {
        self.extents
    }
    /// Admit actual requested backing once, before any validation/materialization
    /// allocation, then check definition-before-read and restore fresh supplier storage.
    /// The unchanged generic admission error is retained with its original source chain.
    pub fn restore_with<E>(
        self,
        admit: impl FnOnce(&F64ProgramExtents) -> Result<(), E>,
    ) -> Result<ExpressionEvaluator<f64>, F64RestoreError<E>> {
        admit(&self.extents).map_err(F64RestoreError::Admission)?;
        let mut defined =
            reserve::<u8>(self.extents.validation_bytes).map_err(F64RestoreError::Allocation)?;
        defined.resize(self.extents.validation_bytes, 0);
        let mut cursor = Cursor {
            bytes: self.bytes,
            position: self.instruction_start,
        };
        for _ in 0..self.extents.instructions {
            let instruction = cursor.instruction()?;
            let check = |index: usize| -> Result<(), F64TransferError> {
                if index >= self.extents.reserved && defined[index / 8] & (1 << (index % 8)) == 0 {
                    return Err(F64TransferError::Undefined(index));
                }
                Ok(())
            };
            match instruction.opcode {
                0 | 1 => {
                    for index in 0..instruction.operands.len() {
                        check(instruction.operands.get(index))?;
                    }
                }
                2 | 4..=10 => check(instruction.first)?,
                3 => {
                    check(instruction.first)?;
                    check(instruction.second)?;
                }
                _ => unreachable!(),
            }
            defined[instruction.destination / 8] |= 1 << (instruction.destination % 8);
        }
        cursor.position = HEADER + 8 * self.extents.constants;
        for _ in 0..self.extents.results {
            let index = cursor.index()?;
            if index >= self.extents.reserved && defined[index / 8] & (1 << (index % 8)) == 0 {
                return Err(F64TransferError::Undefined(index).into());
            }
        }
        drop(defined);
        let mut stack = reserve(self.extents.stack).map_err(F64RestoreError::Allocation)?;
        stack.resize(self.extents.stack, f64::NAN);
        cursor.position = HEADER;
        for value in &mut stack[self.extents.parameters..self.extents.reserved] {
            *value = f64::from_bits(cursor.u64()?);
        }
        let mut result_indices =
            reserve(self.extents.results).map_err(F64RestoreError::Allocation)?;
        for _ in 0..self.extents.results {
            result_indices.push(cursor.index()?);
        }
        let mut instructions =
            reserve(self.extents.instructions).map_err(F64RestoreError::Allocation)?;
        for _ in 0..self.extents.instructions {
            let wire = cursor.instruction()?;
            let instruction = match wire.opcode {
                0 | 1 => {
                    let mut operands =
                        reserve(wire.operands.len()).map_err(F64RestoreError::Allocation)?;
                    for index in 0..wire.operands.len() {
                        operands.push(wire.operands.get(index));
                    }
                    if wire.opcode == 0 {
                        Instr::Add(wire.destination, operands)
                    } else {
                        Instr::Mul(wire.destination, operands)
                    }
                }
                2 => Instr::Pow(wire.destination, wire.first, wire.power),
                3 => Instr::Powf(wire.destination, wire.first, wire.second),
                _ => Instr::BuiltinFun(wire.destination, builtin_symbol(wire.opcode), wire.first),
            };
            instructions.push((instruction, wire.phase));
        }
        Ok(ExpressionEvaluator {
            stack,
            param_count: self.extents.parameters,
            reserved_indices: self.extents.reserved,
            instructions,
            result_indices,
            external_fns: Vec::new(),
            settings: OptimizationSettings::default(),
        })
    }
}

fn reserve<T>(entries: usize) -> Result<Vec<T>, TryReserveError> {
    #[cfg(test)]
    {
        let attempt = ALLOCATION_ATTEMPTS.with(|count| {
            count.set(count.get() + 1);
            count.get()
        });
        if FAIL_RESERVATION_AT.with(|count| count.get()) == Some(attempt) {
            return Err(Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err());
        }
    }
    let mut values = Vec::new();
    values.try_reserve_exact(entries)?;
    Ok(values)
}
#[cfg(test)]
thread_local! {
    static ALLOCATION_ATTEMPTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static FAIL_RESERVATION_AT: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
}
#[cfg(test)]
mod tests;
