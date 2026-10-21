// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Disposable code bytes only; current scientific receiving remains outside this cache.
use super::*;
use pse_ids::{EncodingChecksum, EncodingHasher};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

const MAGIC: &[u8; 8] = b"PSECPP01";
const HEADER: usize = 80;
const MAX_ENTRIES: usize = 64;
const MAX_BYTES: u64 = 64 << 20;
const MAX_SCAN: usize = 1024;

pub(super) fn key(pending: &[Pending], compiler: &str) -> EncodingChecksum {
    let mut hash = EncodingHasher::new();
    let mut part = |bytes: &[u8]| {
        hash.update(&(bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    };
    part(MAGIC);
    part(
        library::Evaluator::CompiledScalar { module_bytes: 0 }
            .identity()
            .as_bytes(),
    );
    part(compiler.as_bytes());
    for flag in FLAGS {
        part(flag.as_bytes());
    }
    for item in pending {
        part(item.function.as_bytes());
        part(&item.inputs.to_le_bytes());
        part(&item.outputs.to_le_bytes());
        part(item.source.as_bytes());
    }
    hash.finish()
}
#[derive(Clone)]
pub(super) struct Cache {
    root: PathBuf,
}
impl Cache {
    pub(super) fn local() -> Self {
        Self {
            root: std::env::temp_dir().join(format!(
                "pse-arrow-scalar-{}-v1",
                rustix::process::geteuid().as_raw()
            )),
        }
    }
    #[cfg(test)]
    pub(super) fn at(root: PathBuf) -> Self {
        Self { root }
    }
    fn path(&self, key: &EncodingChecksum) -> PathBuf {
        self.root.join(format!("{}.bin", key.0.to_hex()))
    }
    fn check(scope: &pse_kernels::ExecutionScope) -> io::Result<()> {
        scope
            .check()
            .map_err(|_| io::Error::other("scalar cache operation stopped"))
    }
    pub(super) fn receive(
        &self,
        key: &EncodingChecksum,
        target: &Path,
        limit: usize,
        scope: &pse_kernels::ExecutionScope,
    ) -> bool {
        self.receive_inner(key, target, limit, scope)
            .unwrap_or(false)
    }
    fn receive_inner(
        &self,
        key: &EncodingChecksum,
        target: &Path,
        limit: usize,
        scope: &pse_kernels::ExecutionScope,
    ) -> io::Result<bool> {
        Self::check(scope)?;
        let path = self.path(key);
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_file()
            || metadata.len() < HEADER as u64
            || metadata.len() > limit as u64 + HEADER as u64
        {
            return Ok(false);
        }
        let mut input = File::open(path)?;
        let mut header = [0_u8; HEADER];
        input.read_exact(&mut header)?;
        if &header[..8] != MAGIC || &header[8..40] != key.0.as_bytes() {
            return Ok(false);
        }
        let extent = u64::from_le_bytes(header[72..80].try_into().map_err(io::Error::other)?);
        if extent == 0 || extent > limit as u64 || metadata.len() != extent + HEADER as u64 {
            return Ok(false);
        }
        let mut output = File::create(target)?;
        let mut hash = EncodingHasher::new();
        let mut bytes = [0_u8; 16 << 10];
        let mut remaining = extent;
        while remaining > 0 {
            Self::check(scope)?;
            let count = (remaining as usize).min(bytes.len());
            input.read_exact(&mut bytes[..count])?;
            hash.update(&bytes[..count]);
            output.write_all(&bytes[..count])?;
            remaining -= count as u64;
        }
        let mut extra = [0_u8; 1];
        if input.read(&mut extra)? != 0 || hash.finish().0.as_bytes() != &header[40..72] {
            return Ok(false);
        }
        output.flush()?;
        Self::check(scope)?;
        Ok(true)
    }
    pub(super) fn publish(
        &self,
        key: &EncodingChecksum,
        source: &Path,
        scope: &pse_kernels::ExecutionScope,
    ) {
        // A cache failure cannot invalidate already checked current code. The
        // caller still performs its authoritative late stop check afterwards.
        let _ = self.publish_inner(key, source, scope);
    }
    fn publish_inner(
        &self,
        key: &EncodingChecksum,
        source: &Path,
        scope: &pse_kernels::ExecutionScope,
    ) -> io::Result<()> {
        Self::check(scope)?;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.root)?;
        let root = std::fs::symlink_metadata(&self.root)?;
        if !root.file_type().is_dir()
            || root.uid() != rustix::process::geteuid().as_raw()
            || root.mode() & 0o077 != 0
        {
            return Ok(());
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join(".lock"))?;
        if lock.try_lock().is_err() {
            return Ok(());
        }
        let extent = std::fs::metadata(source)?.len();
        let required = extent
            .checked_add(HEADER as u64)
            .ok_or_else(|| io::Error::other("scalar cache extent"))?;
        if extent == 0 || required > MAX_BYTES {
            return Ok(());
        }
        struct Entry {
            name: String,
            bytes: u64,
            modified: SystemTime,
        }
        let name = format!("{}.bin", key.0.to_hex());
        let mut entries = Vec::with_capacity(MAX_ENTRIES + 1);
        let mut total = 0_u64;
        for (scanned, entry) in std::fs::read_dir(&self.root)?.enumerate() {
            Self::check(scope)?;
            if scanned >= MAX_SCAN {
                return Ok(());
            }
            let entry = entry?;
            let file = entry.file_name();
            let Some(file) = file.to_str() else {
                continue;
            };
            if file.len() != 68
                || !file.ends_with(".bin")
                || !file.as_bytes()[..64].iter().all(u8::is_ascii_hexdigit)
                || file == name
            {
                continue;
            }
            let metadata = entry.metadata()?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if entries.len() > MAX_ENTRIES {
                return Ok(());
            }
            total = total
                .checked_add(metadata.len())
                .ok_or_else(|| io::Error::other("scalar cache total"))?;
            entries.push(Entry {
                name: file.to_owned(),
                bytes: metadata.len(),
                modified: metadata.modified()?,
            });
        }
        entries.sort_by(|a, b| a.modified.cmp(&b.modified).then(a.name.cmp(&b.name)));
        let mut survivors = entries.len();
        for entry in entries {
            if survivors < MAX_ENTRIES && total <= MAX_BYTES - required {
                break;
            }
            Self::check(scope)?;
            std::fs::remove_file(self.root.join(entry.name))?;
            total -= entry.bytes;
            survivors -= 1;
        }
        let mut input = File::open(source)?;
        let mut hash = EncodingHasher::new();
        let mut bytes = [0_u8; 16 << 10];
        loop {
            Self::check(scope)?;
            let count = input.read(&mut bytes)?;
            if count == 0 {
                break;
            }
            hash.update(&bytes[..count]);
        }
        let checksum = hash.finish();
        let mut output = tempfile::NamedTempFile::new_in(&self.root)?;
        output.write_all(MAGIC)?;
        output.write_all(key.0.as_bytes())?;
        output.write_all(checksum.0.as_bytes())?;
        output.write_all(&extent.to_le_bytes())?;
        let mut input = File::open(source)?;
        let mut remaining = extent;
        while remaining > 0 {
            Self::check(scope)?;
            let count = (remaining as usize).min(bytes.len());
            input.read_exact(&mut bytes[..count])?;
            output.write_all(&bytes[..count])?;
            remaining -= count as u64;
        }
        output.flush()?;
        Self::check(scope)?;
        output
            .persist(self.path(key))
            .map_err(|error| error.error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    use std::time::Duration;
    use symbolica::{
        atom::{Atom, AtomCore},
        parse,
    };

    fn scope() -> pse_kernels::ExecutionScope {
        pse_kernels::ExecutionScope::new(Default::default(), None)
    }
    fn pending() -> Vec<Pending> {
        vec![Pending {
            source: "complete source".into(),
            function: "pse_value0",
            inputs: 1,
            outputs: 1,
            output_capacity: 1 << 20,
        }]
    }
    #[test]
    fn scalar_cache_receiving_rejects_incomplete_corrupt_changed_and_stopped_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let cache = Cache::at(directory.path().join("cache"));
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        std::fs::write(&source, b"complete code image").unwrap();
        let mut inputs = pending();
        let key = key(&inputs, COMPILER);
        cache.publish(&key, &source, &scope());
        assert!(cache.receive(&key, &target, 1024, &scope()));
        assert_eq!(std::fs::read(&target).unwrap(), b"complete code image");
        assert!(!cache.receive(&key, &target, 2, &scope()));
        inputs[0].source.push('!');
        assert!(!cache.receive(&super::key(&inputs, COMPILER), &target, 1024, &scope()));
        assert!(!cache.receive(
            &super::key(&pending(), "different compiler"),
            &target,
            1024,
            &scope()
        ));
        inputs = pending();
        inputs[0].outputs = 2;
        assert!(!cache.receive(&super::key(&inputs, COMPILER), &target, 1024, &scope()));
        let original = std::fs::read(cache.path(&key)).unwrap();
        let mut corrupt = original.clone();
        corrupt[HEADER] ^= 1;
        std::fs::write(cache.path(&key), &corrupt).unwrap();
        assert!(!cache.receive(&key, &target, 1024, &scope()));
        std::fs::write(cache.path(&key), &original[..HEADER + 1]).unwrap();
        assert!(!cache.receive(&key, &target, 1024, &scope()));
        std::fs::write(cache.path(&key), original).unwrap();
        std::fs::remove_file(&target).unwrap();
        let cancel = pse_diagnostics::Cancellation::default();
        cancel.cancel();
        let stopped = pse_kernels::ExecutionScope::new(cancel, None);
        assert!(!cache.receive(&key, &target, 1024, &stopped));
        assert!(!target.exists());
    }
    #[test]
    fn scalar_cache_publication_is_bounded_nonblocking_and_private_copies_survive_eviction() {
        let directory = tempfile::tempdir().unwrap();
        let cache = Cache::at(directory.path().join("cache"));
        let source = directory.path().join("source");
        let target = directory.path().join("private-copy");
        std::fs::write(&source, b"code").unwrap();
        let key0 = key(&pending(), COMPILER);
        cache.publish(&key0, &source, &scope());
        assert!(cache.receive(&key0, &target, 1024, &scope()));
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(cache.root.join(".lock"))
            .unwrap();
        lock.lock().unwrap();
        let key1 = key(&pending(), "different compiler");
        cache.publish(&key1, &source, &scope());
        assert!(!cache.path(&key1).exists());
        drop(lock);
        for i in 0..MAX_ENTRIES + 2 {
            let mut inputs = pending();
            inputs[0].source.push_str(&i.to_string());
            cache.publish(&key(&inputs, COMPILER), &source, &scope());
        }
        let records = std::fs::read_dir(&cache.root)
            .unwrap()
            .map(Result::unwrap)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "bin"))
            .count();
        assert_eq!(records, MAX_ENTRIES);
        assert_eq!(std::fs::read(&target).unwrap(), b"code");
        // Real concurrent publishers race eviction with receiving. A miss is
        // permitted; a received image must always be complete and exact.
        let barrier = std::sync::Barrier::new(3);
        std::thread::scope(|threads| {
            for writer in 0..2 {
                let cache = &cache;
                let source = &source;
                let barrier = &barrier;
                threads.spawn(move || {
                    barrier.wait();
                    for i in 0..32 {
                        let mut inputs = pending();
                        inputs[0].source = format!("writer{writer}/module{i}");
                        cache.publish(&key(&inputs, COMPILER), source, &scope());
                    }
                });
            }
            barrier.wait();
            for i in 0..64 {
                let mut inputs = pending();
                inputs[0].source = format!("writer{}/module{}", i % 2, i / 2);
                if cache.receive(&key(&inputs, COMPILER), &target, 1024, &scope()) {
                    assert_eq!(std::fs::read(&target).unwrap(), b"code");
                }
            }
        });
        let mut received = 0;
        for writer in 0..2 {
            for i in 0..32 {
                let mut inputs = pending();
                inputs[0].source = format!("writer{writer}/module{i}");
                if cache.receive(&key(&inputs, COMPILER), &target, 1024, &scope()) {
                    received += 1;
                    assert_eq!(std::fs::read(&target).unwrap(), b"code");
                }
            }
        }
        assert!(
            received > 0,
            "actual concurrent publishers must publish a complete image"
        );
        assert!(
            std::fs::read_dir(&cache.root)
                .unwrap()
                .map(Result::unwrap)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "bin"))
                .count()
                <= MAX_ENTRIES
        );
    }
    #[test]
    fn scalar_cache_reuses_code_across_fresh_processes() {
        const CHILD: &str = "PSE_SCALAR_CACHE_TEST_CHILD";
        const ROOT: &str = "PSE_SCALAR_CACHE_TEST_ROOT";
        if let Ok(mode) = std::env::var(CHILD) {
            crate::initialize().unwrap();
            let evaluator = Atom::evaluator_multiple(
                &[parse!("pse_cache_test::x^2+1")],
                &[parse!("pse_cache_test::x")],
            )
            .horner_iterations(0)
            .cpe_iterations(Some(0))
            .build()
            .unwrap()
            .map_coeff(&|coefficient| coefficient.to_real().unwrap().to_f64());
            let export = evaluator.export_instructions();
            let storage = library::EvaluatorFacts::inspect(
                &export,
                evaluator.count_operations(),
                "test.cache_inspection",
            )
            .unwrap()
            .storage;
            let hub = ObservationHub::new(Policy::default());
            let operation = hub.start("fresh process scalar cache");
            let context = operation.context();
            let _entered = context.enter();
            let mut batch = Batch {
                cache: Some(Cache::at(std::env::var_os(ROOT).unwrap().into())),
                ..Batch::default()
            };
            let mut current = batch
                .push(&evaluator, &storage, &export, 1 << 20, &scope())
                .unwrap();
            if mode == "invalid" {
                let bad =
                    PathBuf::from(std::env::var_os(ROOT).unwrap()).with_extension("not-a-module");
                std::fs::write(&bad, b"not loadable compiled code").unwrap();
                batch.cache.as_ref().unwrap().publish(
                    &key(&batch.pending, COMPILER),
                    &bad,
                    &scope(),
                );
            }
            batch.finish(&scope()).unwrap();
            current.instantiate().unwrap();
            let mut output = [f64::NAN];
            current.evaluate(&[3.], &mut output).unwrap();
            assert_eq!(output, [10.]);
            let snapshot = context.reader().snapshot(Duration::from_secs(60));
            let count = |name| {
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == name)
                    .map(|metric| metric.cumulative.clone())
                    .unwrap_or(Value::Count(0))
            };
            assert_eq!(
                count("math.native_scalar_compilations"),
                Value::Count(u64::from(mode != "hit"))
            );
            assert_eq!(
                count("math.native_scalar_cache_hits"),
                Value::Count(u64::from(mode == "hit"))
            );
            let path = current.template.get().unwrap()._directory.path().to_owned();
            drop(batch);
            assert!(path.exists());
            current.evaluate(&[4.], &mut output).unwrap();
            assert_eq!(output, [17.]);
            drop(current);
            assert!(!path.exists());
            std::fs::write(
                PathBuf::from(std::env::var_os(ROOT).unwrap()).with_extension(&mode),
                b"executed;original=10,17",
            )
            .unwrap();
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        for mode in ["cold", "hit", "invalid"] {
            let status=Command::new(std::env::current_exe().unwrap())
                .arg("--exact").arg("native_compiled::cache::tests::scalar_cache_reuses_code_across_fresh_processes")
                .arg("--nocapture").env(CHILD,mode).env(ROOT,directory.path().join("cache"))
                .status().unwrap();
            assert!(
                status.success(),
                "actual {mode} process must execute the scalar cache control"
            );
            assert_eq!(
                std::fs::read(directory.path().join("cache").with_extension(mode)).unwrap(),
                b"executed;original=10,17",
                "a zero-test child is not a passing observation"
            );
        }
    }
}
