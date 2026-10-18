// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Once-encoded physical blocks, bounded in memory and otherwise managed spill.

use super::{WorkflowError, contract};
use bytes::Bytes;
use datafusion::error::DataFusionError;
use datafusion_execution::{
    SpillFile, SpillWriter,
    memory_pool::{MemoryConsumer, MemoryReservation},
    runtime_env::RuntimeEnv,
};
use futures_util::{StreamExt, stream::BoxStream};
use std::{io::Write, sync::Arc};

const PAYLOAD_LIMIT: usize = pse_operations::canonical::PAYLOAD_BYTES;
const READ_BUFFERS: usize = 2 * 128 * 1024;

fn engine(error: DataFusionError) -> WorkflowError {
    pse_engine::EngineError::from(error).into()
}
fn io(error: std::io::Error) -> WorkflowError {
    engine(DataFusionError::IoError(error))
}

enum Storage {
    Memory(Vec<u8>),
    Spill(Arc<dyn SpillFile>, Box<dyn SpillWriter>),
}

pub(super) struct PhysicalSpool {
    env: Arc<RuntimeEnv>,
    storage: Storage,
    memory_limit: usize,
    owner: MemoryReservation,
}
impl PhysicalSpool {
    pub(super) fn new(env: Arc<RuntimeEnv>, memory_limit: usize) -> Result<Self, WorkflowError> {
        let owner = MemoryConsumer::new("physical:encoding-spool").register(&env.memory_pool);
        let bytes = memory_limit.checked_mul(2)
            .and_then(|n| n.checked_add(PAYLOAD_LIMIT + READ_BUFFERS + 4096))
            .ok_or_else(|| contract("physical spool memory extent"))?;
        owner.try_grow(bytes).map_err(engine)?;
        Ok(Self { env, storage: Storage::Memory(Vec::new()), memory_limit, owner })
    }
    pub(super) fn push(&mut self, payload: &[u8]) -> Result<(), WorkflowError> {
        if payload.len() > PAYLOAD_LIMIT {
            return Err(contract("physical spool block exceeds canonical payload limit"));
        }
        let prefix = u64::try_from(payload.len()).map_err(|_| contract("physical spool length"))?.to_le_bytes();
        if let Storage::Memory(bytes) = &mut self.storage
            && bytes.len().saturating_add(prefix.len()).saturating_add(payload.len()) <= self.memory_limit
        {
            bytes.extend_from_slice(&prefix);
            bytes.extend_from_slice(payload);
            return Ok(());
        }
        if let Storage::Memory(bytes) = &self.storage {
            let file = self.env.disk_manager.create_tmp_file("physical source encoding").map_err(engine)?;
            let mut writer = file.open_writer().map_err(engine)?;
            writer.write_all(bytes).map_err(io)?;
            self.storage = Storage::Spill(file, writer);
        }
        if let Storage::Spill(_, writer) = &mut self.storage {
            writer.write_all(&prefix).map_err(io)?;
            writer.write_all(payload).map_err(io)?;
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<PhysicalSpoolReader, WorkflowError> {
        let (stream, file) = match self.storage {
            Storage::Memory(bytes) => {
                let stream: BoxStream<'static, Result<Bytes, DataFusionError>> =
                    Box::pin(futures_util::stream::once(async move { Ok(Bytes::from(bytes)) }));
                (stream, None)
            }
            Storage::Spill(file, mut writer) => {
                writer.flush().map_err(io)?;
                writer.finish().map_err(engine)?;
                drop(writer);
                (file.read_stream().map_err(engine)?, Some(file))
            }
        };
        Ok(PhysicalSpoolReader { stream, current: Bytes::new(), _file: file, _owner: self.owner })
    }
}

pub(super) struct PhysicalSpoolReader {
    stream: BoxStream<'static, Result<Bytes, DataFusionError>>,
    current: Bytes,
    _file: Option<Arc<dyn SpillFile>>,
    _owner: MemoryReservation,
}
impl PhysicalSpoolReader {
    async fn read_exact(&mut self, output: &mut [u8]) -> Result<bool, WorkflowError> {
        let mut filled = 0;
        while filled < output.len() {
            if self.current.is_empty() {
                let Some(chunk) = self.stream.next().await else {
                    if filled == 0 { return Ok(false); }
                    return Err(contract("physical spool ended within a block"));
                };
                self.current = chunk.map_err(engine)?;
                continue;
            }
            let count = (output.len() - filled).min(self.current.len());
            output[filled..filled + count].copy_from_slice(&self.current.split_to(count));
            filled += count;
        }
        Ok(true)
    }
    pub(super) async fn next(&mut self) -> Result<Option<Vec<u8>>, WorkflowError> {
        let mut prefix = [0; 8];
        if !self.read_exact(&mut prefix).await? { return Ok(None); }
        let length = usize::try_from(u64::from_le_bytes(prefix)).map_err(|_| contract("physical spool length"))?;
        if length > PAYLOAD_LIMIT { return Err(contract("physical spool block length exceeds bound")); }
        let mut bytes = vec![0; length];
        if !self.read_exact(&mut bytes).await? { return Err(contract("physical spool payload absent")); }
        Ok(Some(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn physical_spool_preserves_exact_blocks_and_releases_memory_and_spill() {
        for memory_limit in [8, 4096] {
            let runtime = crate::workflow::tests::runtime();
            let env = runtime.shared.runtime_env();
            let before = env.memory_pool.reserved();
            let mut spool = PhysicalSpool::new(env.clone(), memory_limit).unwrap();
            let expected = [vec![0, 255, 0, 2], vec![9; 100], Vec::new()];
            for bytes in &expected { spool.push(bytes).unwrap(); }
            let mut reader = spool.finish().unwrap();
            let path = reader._file.as_ref().and_then(|file| file.path()).map(std::path::Path::to_owned);
            assert_eq!(path.is_some(), memory_limit == 8);
            for bytes in expected { assert_eq!(reader.next().await.unwrap().unwrap(), bytes); }
            assert!(reader.next().await.unwrap().is_none());
            drop(reader);
            assert_eq!(env.memory_pool.reserved(), before);
            if let Some(path) = path { assert!(!path.exists()); }
        }
    }
    #[test]
    fn physical_spool_pressure_refuses_before_construction() {
        let env = datafusion_execution::runtime_env::RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(datafusion_execution::memory_pool::GreedyMemoryPool::new(1024)))
            .build_arc().unwrap();
        assert!(PhysicalSpool::new(env.clone(), 32).is_err());
        assert_eq!(env.memory_pool.reserved(), 0);
        assert!(PhysicalSpool::new(env, usize::MAX).is_err());
    }
    #[test]
    fn physical_spool_disk_refusal_and_unfinished_drop_release_owned_scratch() {
        let env = datafusion_execution::runtime_env::RuntimeEnvBuilder::new()
            .with_max_temp_directory_size(16).build_arc().unwrap();
        let mut refused = PhysicalSpool::new(env.clone(), 0).unwrap();
        assert!(refused.push(&[7; 32]).is_err());
        drop(refused);
        assert_eq!(env.memory_pool.reserved(), 0);
        // A refused write returns its disk charge, so another owned spill fits.
        let mut short = PhysicalSpool::new(env.clone(), 0).unwrap();
        short.push(&[1; 4]).unwrap();
        let path = match &short.storage {
            Storage::Spill(file, _) => file.path().unwrap().to_owned(),
            Storage::Memory(_) => panic!("expected managed spill"),
        };
        drop(short);
        assert!(!path.exists());
        assert_eq!(env.memory_pool.reserved(), 0);
    }
}
