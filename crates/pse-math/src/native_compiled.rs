// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit scalar C++ preparation; shared code and independently loaded scratch.
use crate::{MathError, library};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex, OnceLock},
};
use symbolica::evaluate::{CompiledRealEvaluator, ExportSettings, ExpressionEvaluator, InlineASM};

mod cache;
#[cfg(test)]
use std::time::Instant;

#[cfg(test)]
const FUNCTION: &str = "pse_value";
const COMPILER: &str = "/usr/bin/g++";
const FLAGS: &[&str] = &[
    "-std=c++17",
    "-O2",
    "-pipe",
    "-fPIC",
    "-fno-fast-math",
    "-fno-unsafe-math-optimizations",
    "-ffp-contract=off",
    "-march=x86-64",
    "-mtune=generic",
];

pub(crate) struct Template {
    // Drop the loaded template before its source/executable directory. Workers
    // retain this owner as well as their library-owned executable reference.
    evaluator: Mutex<CompiledRealEvaluator>,
    _directory: tempfile::TempDir,
}
pub(crate) struct Evaluator {
    // Mutex supplies Sync for immutable program containers; evaluation obtains
    // exclusive mutable access, never a shared serialization lock across workers.
    worker: Option<Mutex<CompiledRealEvaluator>>,
    // Rust drops fields in declaration order: unload private native scratch
    // before the last template can remove its directory and shared ownership.
    template: Arc<OnceLock<Arc<Template>>>,
    function: &'static str,
}
impl Clone for Evaluator {
    fn clone(&self) -> Self {
        Self {
            template: self.template.clone(),
            function: self.function,
            worker: None,
        }
    }
}
impl Evaluator {
    #[cfg(test)]
    pub(crate) fn probe_module(&self) -> Result<(std::path::PathBuf, &'static str), MathError> {
        let template = self
            .template
            .get()
            .ok_or_else(|| MathError::Contract("compiled module not finalized".into()))?;
        Ok((template._directory.path().join("value.so"), self.function))
    }
    pub(crate) fn instantiate(&mut self) -> Result<(), MathError> {
        if self.worker.is_none() {
            let evaluator = self
                .template
                .get()
                .ok_or_else(|| MathError::Contract("compiled module not finalized".into()))?
                .evaluator
                .lock()
                .map_err(|_| MathError::Library("compiled template lock poisoned".into()))?
                .load_new_function(self.function)
                .map_err(MathError::Library)?;
            self.worker = Some(Mutex::new(evaluator));
        }
        Ok(())
    }
    pub(crate) fn reset(&mut self) {
        self.worker = None;
    }
    pub(crate) fn evaluate(&mut self, arguments: &[f64], values: &mut [f64]) -> Result<(), String> {
        let evaluator = self
            .worker
            .as_mut()
            .ok_or("compiled worker not instantiated")?
            .get_mut()
            .map_err(|_| "compiled worker lock poisoned")?;
        if evaluator.get_input_len() != arguments.len()
            || evaluator.get_output_len() != values.len()
        {
            return Err("compiled scalar input/output extent".into());
        }
        evaluator.evaluate(arguments, values);
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count("math.native_scalar_evaluations", "count", 1);
        }
        Ok(())
    }
}

const CAPTURE: usize = 4096;
const READER_STACK: usize = 64 << 10;

fn run_compiler(
    mut command: Command,
    scope: &pse_kernels::ExecutionScope,
) -> Result<(), MathError> {
    scope.check().map_err(crate::error::scope_error)?;
    command.stdin(Stdio::null()).stdout(Stdio::null());
    // The caller already prepaid READER_STACK and the capture/conversion overlap.
    // Park retains this stack, its directory and that admission on unknown drain.
    let mut compiler = pse_diagnostics::SupervisedChild::spawn(
        &mut command,
        pse_diagnostics::DrainFailurePolicy::Park,
        Some(pse_diagnostics::StderrCapture {
            bytes: CAPTURE,
            stack_bytes: READER_STACK,
        }),
    )
    .map_err(|error| MathError::Library(error.to_string()))?;
    let observed = compiler.wait_ready(Some(scope.cancellation_control()), scope.deadline());
    // Always drain and join stderr before releasing temporary outputs, even
    // when the driver failed or left a live group member behind.
    let status = compiler.drain();
    let diagnostics = compiler.take_stderr_prefix().unwrap_or_default();
    scope.check().map_err(crate::error::scope_error)?;
    observed.map_err(|error| MathError::Library(error.to_string()))?;
    let status = status.map_err(|error| MathError::Library(error.to_string()))?;
    if !status.success() {
        return Err(MathError::Library(format!(
            "compiled scalar compiler {status}: {}",
            String::from_utf8_lossy(&diagnostics)
        )));
    }
    Ok(())
}
fn compiler_command(path: &str, limit: usize, directory: &std::path::Path) -> Command {
    // prlimit installs the hard per-output file limit before exec, inherited by
    // cc1plus/as/ld. Separate compile/link phases own one named output each.
    let mut command = Command::new("/usr/bin/prlimit");
    command
        .arg(format!("--fsize={limit}:{limit}"))
        .arg("--core=0:0")
        .arg("--")
        .arg(path)
        .env("TMPDIR", directory);
    command
}

pub(crate) fn compile(
    evaluator: &ExpressionEvaluator<f64>,
    storage: &library::Storage,
    module_bytes: usize,
    scope: &pse_kernels::ExecutionScope,
) -> Result<Evaluator, MathError> {
    compile_prepared_with(evaluator, storage, module_bytes, scope, COMPILER)
}
/// Packing is a physical preparation bound, not an evaluation or arithmetic fusion.
/// Every member retains its individual preflight allowance and private worker scratch.
const FUNCTIONS: [&str; 32] = [
    "pse_value0",
    "pse_value1",
    "pse_value2",
    "pse_value3",
    "pse_value4",
    "pse_value5",
    "pse_value6",
    "pse_value7",
    "pse_value8",
    "pse_value9",
    "pse_value10",
    "pse_value11",
    "pse_value12",
    "pse_value13",
    "pse_value14",
    "pse_value15",
    "pse_value16",
    "pse_value17",
    "pse_value18",
    "pse_value19",
    "pse_value20",
    "pse_value21",
    "pse_value22",
    "pse_value23",
    "pse_value24",
    "pse_value25",
    "pse_value26",
    "pse_value27",
    "pse_value28",
    "pse_value29",
    "pse_value30",
    "pse_value31",
];
const FUNCTIONS_PER_MODULE: usize = FUNCTIONS.len();
pub(crate) struct Batch {
    pending: Vec<Pending>,
    template: Arc<OnceLock<Arc<Template>>>,
    cache: Option<cache::Cache>,
    detail: Option<pse_diagnostics::observation::Producer>,
}
impl Default for Batch {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            template: Arc::default(),
            // Unit controls explicitly choose private cache state. Production
            // always attempts the bounded disposable cache at this same owner.
            cache: if cfg!(test) {
                None
            } else {
                Some(cache::Cache::local())
            },
            detail: pse_diagnostics::observation::current().and_then(|context| {
                let selected = context.record_tap().is_some_and(|tap| {
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        tap.selected("native_scalar_compilation.v1")
                    }))
                    .unwrap_or(false)
                });
                selected.then(|| {
                    let producer = context.producer(
                        pse_diagnostics::observation::ProducerRole::Workflow,
                        "math.native_scalar_compilation",
                    );
                    producer.activate_record_family("native_scalar_compilation.v1");
                    producer
                })
            }),
        }
    }
}
struct Pending {
    source: String,
    function: &'static str,
    inputs: usize,
    outputs: usize,
    output_capacity: usize,
}
impl Batch {
    pub(crate) fn push(
        &mut self,
        evaluator: &ExpressionEvaluator<f64>,
        storage: &library::Storage,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<Evaluator, MathError> {
        if self.pending.len() == FUNCTIONS_PER_MODULE {
            self.finish(scope)?;
        }
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err(MathError::Contract(
                "compiled scalar profile supports Linux x86-64".into(),
            ));
        }
        scope.check().map_err(crate::error::scope_error)?;
        let scratch = storage
            .numeric_entries
            .checked_mul(size_of::<f64>())
            .ok_or(MathError::Limit("compiled template scratch"))?;
        let fixed = scratch
            .checked_add(size_of::<Template>() + size_of::<Evaluator>() + (64 << 10))
            .and_then(|n| n.checked_add(READER_STACK + 16 * CAPTURE))
            .ok_or(MathError::Limit("compiled module metadata extent"))?;
        let function = FUNCTIONS[self.pending.len()];
        let mut output_capacity = 0;
        // The enclosing construction owner already holds this module_bytes lease.
        // The supplier's exact count now admits source allocation and all overlap
        // inside that original allowance, before its fallible destination reserve.
        let source = evaluator
            .try_export_cpp_with(
                function,
                ExportSettings::new()
                    .inline_asm(InlineASM::None)
                    .include_header(self.pending.is_empty()),
                symbolica::evaluate::CppProfile::ScalarF64,
                crate::stable::cpp_heads()?,
                &mut |extent: symbolica::evaluate::OperationExtent| {
                    scope.check().map_err(crate::error::scope_error)?;
                    let peak = extent
                        .peak_bytes()
                        .ok_or(MathError::Limit("compiled source extent"))?;
                    output_capacity = module_bytes
                        .checked_sub(fixed)
                        .and_then(|n| n.checked_sub(peak))
                        .filter(|n| *n >= 4)
                        .ok_or(MathError::Limit(
                            "compiled source/output preflight allowance",
                        ))?;
                    Ok(())
                },
            )
            .map_err(|error| match error {
                symbolica::evaluate::OperationError::Control(error) => error,
                symbolica::evaluate::OperationError::Overflow => {
                    MathError::Limit("compiled source extent")
                }
                symbolica::evaluate::OperationError::Allocation => {
                    MathError::Limit("compiled source allocation")
                }
                symbolica::evaluate::OperationError::Unsupported(profile) => {
                    MathError::Contract(format!("unsupported checked C++ profile: {profile}"))
                }
                symbolica::evaluate::OperationError::InvalidProgram(message) => {
                    MathError::Library(message)
                }
                symbolica::evaluate::OperationError::Interrupted => MathError::Cancelled,
            })?
            .source;
        self.pending.push(Pending {
            source,
            function,
            inputs: evaluator.get_input_len(),
            outputs: evaluator.get_output_len(),
            output_capacity,
        });
        Ok(Evaluator {
            template: self.template.clone(),
            function,
            worker: None,
        })
    }
    pub(crate) fn finish(&mut self, scope: &pse_kernels::ExecutionScope) -> Result<(), MathError> {
        self.finish_with(scope, COMPILER)
    }
    fn finish_with(
        &mut self,
        scope: &pse_kernels::ExecutionScope,
        compiler_path: &str,
    ) -> Result<(), MathError> {
        if self.pending.is_empty() {
            return Ok(());
        }
        scope.check().map_err(crate::error::scope_error)?;
        let _phase = pse_diagnostics::observation::phase("math.native_scalar_preparation");
        let output_capacity = self
            .pending
            .iter()
            .try_fold(0usize, |n, item| n.checked_add(item.output_capacity))
            .ok_or(MathError::Limit("packed compiled module extent"))?;
        // Object, shared file, loaded image and atomic cache publication may
        // coexist. A cache hit retains the same conservative admission. Member
        // preflights also cover sources, export and bounded cache metadata.
        let object_limit = output_capacity / 4;
        let code_limit = output_capacity / 4;
        let directory = tempfile::Builder::new()
            .prefix("pse-math-native-")
            .tempdir()
            .map_err(|e| MathError::Library(e.to_string()))?;
        let source_path = directory.path().join("value.cpp");
        let library_path = directory.path().join("value.so");
        let object_path = directory.path().join("value.o");
        let key = cache::key(&self.pending, compiler_path);
        let received = self
            .cache
            .as_ref()
            .is_some_and(|cache| cache.receive(&key, &library_path, code_limit, scope));
        // Failed receiving, including a load/export mismatch, is an ordinary
        // miss. Nothing is published until every current demanded symbol agrees.
        let mut loaded = if received {
            self.load_module(&library_path, scope).ok()
        } else {
            None
        };
        let hit = loaded.is_some();
        if !hit {
            scope.check().map_err(crate::error::scope_error)?;
            let _compilation =
                pse_diagnostics::observation::phase("math.native_scalar_compilation");
            let mut source = std::fs::File::create(&source_path)
                .map_err(|e| MathError::Library(e.to_string()))?;
            for item in &self.pending {
                scope.check().map_err(crate::error::scope_error)?;
                source
                    .write_all(item.source.as_bytes())
                    .map_err(|e| MathError::Library(e.to_string()))?;
            }
            drop(source);
            let mut compile = compiler_command(compiler_path, object_limit, directory.path());
            compile
                .args(FLAGS)
                .arg("-c")
                .arg(&source_path)
                .arg("-o")
                .arg(&object_path);
            run_compiler(compile, scope)?;
            let mut link = compiler_command(compiler_path, code_limit, directory.path());
            link.arg("-shared")
                .arg("-Wl,-z,max-page-size=4096")
                .arg(&object_path)
                .arg("-o")
                .arg(&library_path);
            run_compiler(link, scope)?;
            loaded = Some(self.load_module(&library_path, scope)?);
        }
        let code_bytes = usize::try_from(
            std::fs::metadata(&library_path)
                .map_err(|e| MathError::Library(e.to_string()))?
                .len(),
        )
        .map_err(|_| MathError::Limit("compiled executable extent"))?;
        if code_bytes > code_limit {
            return Err(MathError::Contract(
                "compiler exceeded enforced output file limit".into(),
            ));
        }
        let loaded =
            loaded.ok_or_else(|| MathError::Contract("compiled scalar module missing".into()))?;
        if !hit && let Some(cache) = &self.cache {
            cache.publish(&key, &library_path, scope);
        }
        scope.check().map_err(crate::error::scope_error)?;
        self.template
            .set(Arc::new(Template {
                evaluator: Mutex::new(loaded),
                _directory: directory,
            }))
            .map_err(|_| MathError::Contract("compiled module already finalized".into()))?;
        if let Some(producer) = &self.detail {
            use pse_diagnostics::observation::{RecordAvailability, RecordField, Value};
            let count = |value: usize| u64::try_from(value).map_or(Value::Overflow, Value::Count);
            let source_bytes = self
                .pending
                .iter()
                .try_fold(0usize, |sum, item| sum.checked_add(item.source.len()));
            let object_bytes = if hit {
                None
            } else {
                std::fs::metadata(&object_path)
                    .ok()
                    .map(|metadata| metadata.len())
            };
            let field = |name: &str, unit: &str, value: Value| RecordField {
                name: name.into(),
                unit: unit.into(),
                value,
                availability: None,
            };
            let object = RecordField {
                name: "object_bytes".into(),
                unit: "bytes".into(),
                value: object_bytes.map_or_else(
                    || {
                        Value::Unavailable(
                            if hit {
                                "cache hit; no object produced"
                            } else {
                                "object extent unavailable"
                            }
                            .into(),
                        )
                    },
                    Value::Count,
                ),
                availability: object_bytes.is_none().then_some(if hit {
                    RecordAvailability::NotSupplied
                } else {
                    RecordAvailability::Unavailable
                }),
            };
            let record = producer.record(
                "native_scalar_compilation.v1",
                "finalized",
                None,
                None,
                vec![
                    field("cache_hit", "boolean", Value::Boolean(hit)),
                    field("functions", "functions", count(self.pending.len())),
                    field(
                        "generated_source_bytes",
                        "bytes",
                        source_bytes.map_or(Value::Overflow, count),
                    ),
                    object,
                    field("executable_bytes", "bytes", count(code_bytes)),
                    field("object_limit_bytes", "bytes", count(object_limit)),
                    field("executable_limit_bytes", "bytes", count(code_limit)),
                ],
            );
            producer.publish(&record);
        }
        if let Some(context) = pse_diagnostics::observation::current() {
            context.count(
                if hit {
                    "math.native_scalar_cache_hits"
                } else {
                    "math.native_scalar_compilations"
                },
                "count",
                1,
            );
            context.count(
                "math.native_scalar_module_functions",
                "count",
                self.pending.len() as u64,
            );
            context.gauge(
                "math.native_scalar_executable_bytes",
                "bytes",
                code_bytes as f64,
            );
        }
        self.pending.clear();
        self.template = Arc::default();
        Ok(())
    }
    fn load_module(
        &self,
        path: &std::path::Path,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<CompiledRealEvaluator, MathError> {
        scope.check().map_err(crate::error::scope_error)?;
        let loaded = CompiledRealEvaluator::load(path, self.pending[0].function)
            .map_err(MathError::Library)?;
        for item in &self.pending {
            scope.check().map_err(crate::error::scope_error)?;
            let checked = loaded
                .load_new_function(item.function)
                .map_err(MathError::Library)?;
            if checked.get_input_len() != item.inputs || checked.get_output_len() != item.outputs {
                return Err(MathError::Contract(
                    "compiled scalar library dimensions".into(),
                ));
            }
        }
        scope.check().map_err(crate::error::scope_error)?;
        Ok(loaded)
    }
}
fn compile_prepared_with(
    evaluator: &ExpressionEvaluator<f64>,
    storage: &library::Storage,
    module_bytes: usize,
    scope: &pse_kernels::ExecutionScope,
    compiler_path: &str,
) -> Result<Evaluator, MathError> {
    let mut batch = Batch::default();
    let evaluator = batch.push(evaluator, storage, module_bytes, scope)?;
    batch.finish_with(scope, compiler_path)?;
    Ok(evaluator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, time::Duration};
    use symbolica::{
        atom::{Atom, AtomCore},
        parse,
    };

    // Fixture construction/inspection belongs to the already constructed test
    // evaluator, matching the source-owned inspection in compile_stages.
    fn compile_with(
        evaluator: &ExpressionEvaluator<f64>,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
        compiler: &str,
    ) -> Result<Evaluator, MathError> {
        let export = evaluator.export_instructions();
        let storage = library::EvaluatorFacts::inspect(
            &export,
            evaluator.count_operations(),
            "math.test_representation_inspections",
        )?
        .storage;
        compile_prepared_with(evaluator, &storage, module_bytes, scope, compiler)
    }
    fn compile(
        evaluator: &ExpressionEvaluator<f64>,
        module_bytes: usize,
        scope: &pse_kernels::ExecutionScope,
    ) -> Result<Evaluator, MathError> {
        compile_with(evaluator, module_bytes, scope, COMPILER)
    }

    fn checked_source(evaluator: &ExpressionEvaluator<f64>) -> symbolica::evaluate::CheckedCpp {
        evaluator
            .try_export_cpp_with(
                FUNCTION,
                ExportSettings::new().inline_asm(InlineASM::None),
                symbolica::evaluate::CppProfile::ScalarF64,
                crate::stable::cpp_heads().unwrap(),
                &mut |_| Ok::<_, MathError>(()),
            )
            .unwrap()
    }

    fn arithmetic() -> ExpressionEvaluator<f64> {
        crate::initialize().unwrap();
        Atom::evaluator_multiple(
            &[parse!("pse_native_test::x^2+1")],
            &[parse!("pse_native_test::x")],
        )
        .horner_iterations(0)
        .cpe_iterations(Some(0))
        .build()
        .unwrap()
        .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64())
    }

    #[test]
    fn selected_compilation_record_retains_actual_finalized_module_extents() {
        use pse_diagnostics::observation::{
            CoherentRecord, ObservationHub, Policy, RecordTap, Value,
        };
        #[derive(Debug, Default)]
        struct Selected(Mutex<Vec<CoherentRecord>>);
        impl RecordTap for Selected {
            fn selected(&self, family: &str) -> bool {
                family == "native_scalar_compilation.v1"
            }
            fn observe(&self, record: &CoherentRecord) {
                self.0.lock().unwrap().push(record.clone());
            }
        }
        let evaluator = arithmetic();
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("selected native scalar compilation");
        let tap = Arc::new(Selected::default());
        let context = operation.context().with_record_tap(tap.clone());
        let _entered = context.enter();
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let compiled = compile(&evaluator, 1 << 20, &scope).unwrap();
        let module = compiled.template.get().unwrap()._directory.path();
        let expected_source = std::fs::metadata(module.join("value.cpp")).unwrap().len();
        let expected_object = std::fs::metadata(module.join("value.o")).unwrap().len();
        let expected_code = std::fs::metadata(module.join("value.so")).unwrap().len();
        let records = tap.0.lock().unwrap();
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.producer.operation, context.id());
        assert_eq!(record.family, "native_scalar_compilation.v1");
        assert_eq!(record.phase, "finalized");
        let value = |name: &str| {
            &record
                .values
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .value
        };
        assert_eq!(value("cache_hit"), &Value::Boolean(false));
        assert_eq!(value("functions"), &Value::Count(1));
        assert_eq!(
            value("generated_source_bytes"),
            &Value::Count(expected_source)
        );
        assert_eq!(value("object_bytes"), &Value::Count(expected_object));
        assert_eq!(value("executable_bytes"), &Value::Count(expected_code));
        assert!(
            matches!(value("object_limit_bytes"), Value::Count(limit) if *limit >= expected_object)
        );
        assert!(
            matches!(value("executable_limit_bytes"), Value::Count(limit) if *limit >= expected_code)
        );
    }

    #[test]
    fn named_modules_pack_bounded_symbols_and_keep_private_workers_and_live_files() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        crate::initialize().unwrap();
        let x = parse!("pse_native_test::x");
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("named module packing");
        let context = operation.context();
        let _entered = context.enter();
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let mut batch = Batch::default();
        let mut functions = Vec::new();
        for i in 0..=FUNCTIONS_PER_MODULE {
            // Each symbol has a different result, so a wrong same-arity dispatch
            // cannot pass merely because two packed functions share semantics.
            let expression = if i % 2 == 0 {
                x.clone().pow(2) + Atom::num(i as i64 + 1)
            } else {
                Atom::num(i as i64 + 7)
            };
            let arguments = if i % 2 == 0 {
                std::slice::from_ref(&x)
            } else {
                &[]
            };
            let selected = Atom::evaluator_multiple(&[expression], arguments)
                .horner_iterations(0)
                .cpe_iterations(Some(0))
                .build()
                .unwrap()
                .map_coeff(&|c| c.to_real().unwrap().to_f64());
            let export = selected.export_instructions();
            let storage = library::EvaluatorFacts::inspect(
                &export,
                selected.count_operations(),
                "test.named_inspection",
            )
            .unwrap()
            .storage;
            functions.push(batch.push(&selected, &storage, 1 << 20, &scope).unwrap());
        }
        assert!(Arc::ptr_eq(&functions[0].template, &functions[1].template));
        assert!(!Arc::ptr_eq(
            &functions[0].template,
            &functions[FUNCTIONS_PER_MODULE].template
        ));
        assert!(
            functions[FUNCTIONS_PER_MODULE].instantiate().is_err(),
            "no unfinalized function is executable"
        );
        batch.finish(&scope).unwrap();
        let path = functions[0]
            .template
            .get()
            .unwrap()
            ._directory
            .path()
            .to_path_buf();
        let mut private = functions[0].clone();
        for (i, function) in functions.iter_mut().enumerate() {
            function.instantiate().unwrap();
            let mut result = [f64::NAN];
            function
                .evaluate(if i % 2 == 0 { &[3.] } else { &[] }, &mut result)
                .unwrap();
            assert_eq!(
                result[0],
                if i % 2 == 0 {
                    10. + i as f64
                } else {
                    7. + i as f64
                }
            );
        }
        drop(functions);
        drop(batch);
        assert!(path.exists(), "live private worker keeps its exact module");
        private.instantiate().unwrap();
        let mut result = [f64::NAN];
        private.evaluate(&[4.], &mut result).unwrap();
        assert_eq!(result, [17.]);
        drop(private);
        assert!(!path.exists());
        let snapshot = context.reader().snapshot(Duration::from_secs(60));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "math.native_scalar_compilations")
                .unwrap()
                .cumulative,
            Value::Count(2)
        );
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "math.native_scalar_module_functions")
                .unwrap()
                .cumulative,
            Value::Count((FUNCTIONS_PER_MODULE + 1) as u64)
        );
    }

    #[test]
    fn named_module_failure_or_stop_never_publishes_pending_functions() {
        let evaluator = arithmetic();
        let export = evaluator.export_instructions();
        let storage = library::EvaluatorFacts::inspect(
            &export,
            evaluator.count_operations(),
            "test.named_inspection",
        )
        .unwrap()
        .storage;
        let cancel = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(cancel.clone(), None);
        let mut batch = Batch::default();
        let mut first = batch.push(&evaluator, &storage, 1 << 20, &scope).unwrap();
        let mut second = batch.push(&evaluator, &storage, 1 << 20, &scope).unwrap();
        assert!(batch.finish_with(&scope, "/bin/false").is_err());
        assert!(first.instantiate().is_err());
        assert!(second.instantiate().is_err());
        cancel.cancel();
        assert!(matches!(batch.finish(&scope), Err(MathError::Cancelled)));
        assert!(first.template.get().is_none());
    }

    #[test]
    fn native_scalar_failed_compiler_and_pre_cancel_never_publish() {
        let evaluator = arithmetic();
        let cancelled = pse_diagnostics::Cancellation::default();
        assert!(matches!(
            compile_with(
                &evaluator,
                1 << 20,
                &pse_kernels::ExecutionScope::new(cancelled.clone(), None),
                "/bin/false"
            ),
            Err(MathError::Library(_))
        ));
        assert!(matches!(
            compile(
                &evaluator,
                1,
                &pse_kernels::ExecutionScope::new(cancelled.clone(), None)
            ),
            Err(MathError::Limit(_))
        ));
        cancelled.cancel();
        assert!(matches!(
            compile(
                &evaluator,
                1 << 20,
                &pse_kernels::ExecutionScope::new(cancelled.clone(), None)
            ),
            Err(MathError::Cancelled)
        ));
    }

    #[test]
    fn native_scalar_original_deadline_wakes_and_drains_without_requesting_stop() {
        let fixture = tempfile::tempdir().unwrap();
        let marker = fixture.path().join("child");
        let quoted = marker.to_string_lossy().replace('\'', "'\"'\"'");
        let mut command = Command::new("/bin/sh");
        command.arg("-c").arg(format!(
            "sleep 60 &\nprintf '%s\\n' \"$!\" > '{quoted}'\nwait\n"
        ));
        let control = pse_diagnostics::Cancellation::default();
        let scope = pse_kernels::ExecutionScope::new(
            control.clone(),
            Some(Instant::now() + Duration::from_secs(1)),
        );
        let result = run_compiler(command, &scope);
        assert!(matches!(
            result,
            Err(MathError::Scope(pse_kernels::ProviderError::Deadline))
        ));
        assert!(!control.is_cancelled());
        if let Ok(child) = std::fs::read_to_string(marker) {
            assert_stopped_at_return(child.trim());
        }
    }

    #[test]
    fn native_scalar_driver_exit_drains_live_children_before_return() {
        for exit in [0, 7] {
            let fixture = tempfile::tempdir().unwrap();
            let marker = fixture.path().join("child");
            let quoted = marker.to_string_lossy().replace('\'', "'\"'\"'");
            let mut command = Command::new("/bin/sh");
            command.arg("-c").arg(format!(
                "sleep 60 &\nprintf '%s\\n' \"$!\" > '{quoted}'\nexit {exit}\n"
            ));
            let result = run_compiler(
                command,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            );
            assert_eq!(result.is_ok(), exit == 0);
            let child = std::fs::read_to_string(marker).unwrap();
            assert_stopped_at_return(child.trim());
        }
    }

    #[test]
    fn checked_native_scalar_export_covers_supported_scalar_channels() {
        crate::initialize().unwrap();
        let x = parse!("pse_native_bound::x");
        let y = parse!("pse_native_bound::y");
        let expressions = [
            &x + &y,
            &x * &y,
            x.pow(-1),
            x.pow(3),
            x.pow(&y),
            x.exp(),
            x.log(),
            x.sin(),
            x.cos(),
            x.sqrt(),
            x.abs(),
            x.conj(),
        ];
        let evaluator = Atom::evaluator_multiple(&expressions, &[x, y])
            .horner_iterations(0)
            .cpe_iterations(Some(0))
            .build()
            .unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64());
        let source = checked_source(&evaluator);
        assert_eq!(source.source.len(), source.extent.output_bytes);
        let mut compiled = compile(
            &evaluator,
            1 << 20,
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
        )
        .unwrap();
        compiled.instantiate().unwrap();
        let (x, y): (f64, f64) = (1.25, 2.5);
        let expected = [
            x + y,
            x * y,
            x.recip(),
            x.powi(3),
            x.powf(y),
            x.exp(),
            x.ln(),
            x.sin(),
            x.cos(),
            x.sqrt(),
            x.abs(),
            x,
        ];
        let mut actual = [0.; 12];
        compiled.evaluate(&[x, y], &mut actual).unwrap();
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                (actual - expected).abs() <= 32. * f64::EPSILON * expected.abs().max(1.),
                "compiled builtin {actual} differs from independent scalar {expected}"
            );
        }
    }

    #[test]
    fn checked_native_constants_and_repeated_outputs_keep_binary64_bits() {
        crate::initialize().unwrap();
        let ordinals = [1_i64, 2, 3, 4, 5, 1, 3, 5];
        let expressions = ordinals.map(Atom::num);
        let parameters: [Atom; 0] = [];
        let evaluator = Atom::evaluator_multiple(&expressions, &parameters)
            .horner_iterations(0)
            .build()
            .unwrap()
            .map_coeff(&|coefficient| {
                let value = coefficient.to_real().unwrap().to_f64();
                if value == 1. {
                    f64::MAX
                } else if value == 2. {
                    -f64::MAX
                } else if value == 3. {
                    f64::from_bits(1)
                } else if value == 4. {
                    f64::MIN_POSITIVE
                } else if value == 5. {
                    -0.0
                } else {
                    0.0
                }
            });
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let mut compiled = compile(&evaluator, 1 << 20, &scope).unwrap();
        compiled.instantiate().unwrap();
        let mut actual = [f64::NAN; 8];
        compiled.evaluate(&[], &mut actual).unwrap();
        let expected = [
            f64::MAX,
            -f64::MAX,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            -0.0,
            f64::MAX,
            f64::from_bits(1),
            -0.0,
        ];
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn checked_cpp_refusal_stop_and_unknown_head_leave_no_source_product() {
        use symbolica::evaluate::{CppProfile, OperationError, OperationExtent};
        let evaluator = arithmetic();
        let complete = checked_source(&evaluator);
        let required = complete.extent.peak_bytes().unwrap();
        let settings = || ExportSettings::new().inline_asm(InlineASM::None);
        let mut growth = 0;
        let result = evaluator.try_export_cpp_with(
            FUNCTION,
            settings(),
            CppProfile::ScalarF64,
            &[],
            &mut |extent: OperationExtent| {
                if extent.output_bytes > 0 {
                    growth += 1;
                }
                if extent.peak_bytes().unwrap() > required - 1 {
                    Err("original source refusal")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(
            result,
            Err(OperationError::Control("original source refusal"))
        ));
        assert_eq!(growth, 1, "refusal precedes source destination allocation");
        let mut stages = 0;
        let result = evaluator.try_export_cpp_with(
            FUNCTION,
            settings(),
            CppProfile::ScalarF64,
            &[],
            &mut |extent: OperationExtent| {
                if extent.output_bytes > 0 {
                    stages += 1;
                    if stages == 2 {
                        return Err("current emission stop");
                    }
                }
                Ok(())
            },
        );
        assert!(matches!(
            result,
            Err(OperationError::Control("current emission stop"))
        ));
        assert!(
            evaluator
                .try_export_cpp_with(
                    FUNCTION,
                    settings(),
                    CppProfile::ScalarF64,
                    &[],
                    &mut |_| Ok::<_, &'static str>(())
                )
                .is_ok()
        );
        let x = library::formal(0).unwrap();
        let known = crate::stable::call(crate::stable::Operation::Log1p, &[&x]).unwrap();
        let known = Atom::evaluator_multiple(&[known], &[x])
            .direct_translation(true)
            .horner_iterations(0)
            .build()
            .unwrap()
            .map_coeff(&|value| value.re.to_f64());
        assert!(matches!(
            known.try_export_cpp_with(
                FUNCTION,
                settings(),
                CppProfile::ScalarF64,
                &[],
                &mut |_| Ok::<_, &'static str>(())
            ),
            Err(OperationError::Unsupported(_))
        ));
    }

    #[test]
    fn checked_native_integer_power_keeps_parity_signed_zero_and_subnormal_order() {
        let source = checked_source(&arithmetic());
        // This independent executable harness calls the integer kernel actually
        // emitted by the checked producer, including its signed inversion order.
        let mut source = source.source;
        source.push_str(
            r#"
extern "C" unsigned long pse_integer_probe_realf64_get_buffer_len() { return 0; }
extern "C" std::size_t pse_integer_probe_realf64_get_input_len() { return 1; }
extern "C" std::size_t pse_integer_probe_realf64_get_output_len() { return 6; }
extern "C" void pse_integer_probe_realf64(double* p, double*, double* out) {
    out[0] = pse_value_realf64_heads::pse_integer_power(p[0],0u);
    out[1] = pse_value_realf64_heads::pse_integer_power(p[0],1u);
    out[2] = pse_value_realf64_heads::pse_integer_power(p[0],2u);
    out[3] = pse_value_realf64_heads::pse_integer_power(p[0],3u);
    out[4] = 1.0/pse_value_realf64_heads::pse_integer_power(p[0],3u);
    out[5] = pse_value_realf64_heads::pse_integer_power(p[0],2147483647u);
}
"#,
        );
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let mut batch = Batch::default();
        batch.pending.push(Pending {
            source,
            function: "pse_integer_probe",
            inputs: 1,
            outputs: 6,
            output_capacity: 1 << 20,
        });
        let mut evaluator = Evaluator {
            worker: None,
            template: batch.template.clone(),
            function: "pse_integer_probe",
        };
        batch.finish(&scope).unwrap();
        evaluator.instantiate().unwrap();
        for base in [
            -0.0_f64,
            0.0,
            -1.0,
            -2.0,
            2.0_f64.powi(-350),
            f64::from_bits(1),
        ] {
            let expected = [
                base.powi(0),
                base.powi(1),
                base.powi(2),
                base.powi(3),
                base.powi(3).recip(),
                base.powi(i32::MAX),
            ];
            let mut actual = [f64::NAN; 6];
            evaluator.evaluate(&[base], &mut actual).unwrap();
            for (actual, expected) in actual.into_iter().zip(expected) {
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "integer power of {base}"
                );
            }
        }
    }

    #[test]
    fn checked_native_stable_second_and_repeated_modules_match_original_coordinate_controls() {
        crate::initialize().unwrap();
        let x = library::formal(0).unwrap();
        let y = library::formal(1).unwrap();
        let log = crate::stable::call(crate::stable::Operation::Log1p, &[&x]).unwrap();
        let magnitude =
            crate::stable::call(crate::stable::Operation::Magnitude, &[&x, &y]).unwrap();
        let limits = crate::jets::EvaluationLimits::default();
        let layout =
            crate::jets::JetLayout::new(vec![0, 1], pse_kernels::DerivativeOrder::Second, limits)
                .unwrap();
        let floating = library::bounded_evaluator(
            pse_ids::SemanticId::NIL,
            &[log, magnitude],
            &[x, y],
            &layout,
            &[],
            library::Optimization::default(),
            &Arc::new(std::sync::atomic::AtomicBool::new(false)),
            limits,
            limits.operations,
            0,
            &mut library::ScalarCache::default(),
        )
        .unwrap();
        let scope = pse_kernels::ExecutionScope::new(Default::default(), None);
        let mut batch = Batch::default();
        let mut first = batch
            .push(
                &floating.evaluator,
                &floating.facts.storage,
                1 << 20,
                &scope,
            )
            .unwrap();
        let mut second = batch
            .push(
                &floating.evaluator,
                &floating.facts.storage,
                1 << 20,
                &scope,
            )
            .unwrap();
        batch.finish(&scope).unwrap();
        first.instantiate().unwrap();
        second.instantiate().unwrap();
        for (x, y) in [(1e-20_f64, 3e-20_f64), (3e200, 4e200), (3e-200, 4e-200)] {
            let h = x.hypot(y);
            let a = x / h;
            let b = y / h;
            let expected = [
                x.ln_1p(),
                1. / (1. + x),
                0.,
                -0.5 / (1. + x).powi(2),
                0.,
                0.,
                h,
                a,
                b,
                0.5 * b * b / h,
                -a * b / h,
                0.5 * a * a / h,
            ];
            for worker in [&mut first, &mut second] {
                let mut output = [0.; 12];
                worker
                    .evaluate(&[x, 1., 0., 0., 0., 0., y, 0., 1., 0., 0., 0.], &mut output)
                    .unwrap();
                for (actual, expected) in output.into_iter().zip(expected) {
                    assert!(actual.is_finite());
                    assert!(
                        (actual - expected).abs()
                            <= 512. * f64::EPSILON * expected.abs().max(1e-300),
                        "stable compiled Second {actual} != independent {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn native_scalar_export_preflight_covers_wide_argument_temporary_overlap() {
        crate::initialize().unwrap();
        let parameters = (0..256)
            .map(|i| library::formal(i).unwrap())
            .collect::<Vec<_>>();
        let sum = parameters
            .iter()
            .fold(Atom::num(0), |value, parameter| value + parameter);
        let product = parameters
            .iter()
            .fold(Atom::num(1), |value, parameter| value * parameter);
        let evaluator = Atom::evaluator_multiple(&[sum, product], &parameters)
            .horner_iterations(0)
            .cpe_iterations(Some(0))
            .build()
            .unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64());
        let source = checked_source(&evaluator);
        assert_eq!(source.source.len(), source.extent.output_bytes);
        let bound = source.extent.peak_bytes().unwrap();
        // A too-small module refuses before invoking even a deliberately invalid
        // compiler path; the effect boundary is behind export/output preflight.
        assert!(matches!(
            compile_with(
                &evaluator,
                bound,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
                "/no/compiler"
            ),
            Err(MathError::Limit(
                "compiled source/output preflight allowance"
            ))
        ));
    }

    #[test]
    fn native_scalar_compiler_diagnostics_and_file_output_are_bounded() {
        let fixture = tempfile::tempdir().unwrap();
        let noisy = fixture.path().join("noisy");
        std::fs::write(
            &noisy,
            "#!/bin/sh\nhead -c 1048576 /dev/zero | tr '\\000' x >&2\nexit 1\n",
        )
        .unwrap();
        std::fs::set_permissions(&noisy, std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = compile_with(
            &arithmetic(),
            1 << 20,
            &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
            noisy.to_str().unwrap(),
        );
        let Err(MathError::Library(detail)) = error else {
            panic!("noisy compiler must fail");
        };
        assert!(detail.len() < CAPTURE + 128);
        let oversized = fixture.path().join("oversized");
        std::fs::write(&oversized, "#!/bin/sh\nprev=\nfor arg; do if [ \"$prev\" = -o ]; then out=$arg; fi; prev=$arg; done\ndd if=/dev/zero of=\"$out\" bs=4096 count=1024 2>/dev/null\n").unwrap();
        std::fs::set_permissions(&oversized, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(matches!(
            compile_with(
                &arithmetic(),
                1 << 20,
                &pse_kernels::ExecutionScope::new(pse_diagnostics::Cancellation::default(), None),
                oversized.to_str().unwrap()
            ),
            Err(MathError::Library(_))
        ));
    }

    fn assert_stopped_at_return(child: &str) {
        let stat = std::fs::read_to_string(format!("/proc/{child}/stat"));
        assert!(
            stat.as_ref()
                .map_or(true, |stat| stat.rsplit_once(") ").is_some_and(
                    |(_, tail)| matches!(tail.as_bytes().first(), Some(b'Z' | b'X'))
                )),
            "owned compiler descendant still executing at return"
        );
    }

    #[test]
    fn native_scalar_cancelled_compiler_drains_owned_group_and_removes_partial_output() {
        let fixture = tempfile::tempdir().unwrap();
        let marker = fixture.path().join("compiler-started");
        let compiler = fixture.path().join("compiler");
        let quoted_marker = marker.to_string_lossy().replace('\'', "'\"'\"'");
        std::fs::write(&compiler, format!("#!/bin/sh\nfor arg; do case \"$arg\" in *.cpp) printf '%s\\n' \"$arg\" > '{quoted_marker}';; esac; done\nsleep 60 &\nprintf '%s\\n' \"$!\" >> '{quoted_marker}'\nwait\n")).unwrap();
        std::fs::set_permissions(&compiler, std::fs::Permissions::from_mode(0o700)).unwrap();
        let cancelled = pse_diagnostics::Cancellation::default();
        let flag = cancelled.clone();
        let started = marker.clone();
        let observer = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                if let Ok(text) = std::fs::read_to_string(&started)
                    && text.lines().count() == 2
                {
                    flag.cancel();
                    return text;
                }
                if Instant::now() >= deadline {
                    flag.cancel();
                    panic!("owned compiler fixture did not start");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let result = compile_with(
            &arithmetic(),
            1 << 20,
            &pse_kernels::ExecutionScope::new(cancelled.clone(), None),
            compiler.to_str().unwrap(),
        );
        assert!(matches!(result, Err(MathError::Cancelled)));
        let text = observer.join().unwrap();
        let mut lines = text.lines();
        let source = std::path::Path::new(lines.next().unwrap());
        assert!(
            !source.parent().unwrap().exists(),
            "cancelled preparation retained partial source/output"
        );
        let child = lines.next().unwrap();
        // Drain is part of compile_with: the descendant must already be dead
        // at return, not merely stop during a later grace-period poll.
        assert_stopped_at_return(child);
    }
}
