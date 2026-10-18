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
// grouped-v2 binds both limits. Changing either changes publication identity.
const GROUP_BLOCKS: usize = 4;
const GROUP_BYTES: usize = 2 * PAYLOAD_LIMIT;
// Four fixed-width physical headers, edit/plan descriptions and their encoded
// metadata copies. Encoder/SDK scratch remains independently charged upstream.
const GROUP_HEADER_BYTES: usize = 64 * 1024;

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

/// One immutable consecutive group. Its grant follows payloads through the
/// source edit and actual acknowledgement, independently of the spool's reader.
pub(super) struct PhysicalGroup {
    pub(super) first: u64,
    pub(super) end: u64,
    pub(super) blocks: Vec<Vec<u8>>,
    pub(super) owner: Arc<pse_columnar::AllocationLease>,
}
pub(super) struct PhysicalGroups {
    reader: PhysicalSpoolReader,
    pending: Option<Vec<u8>>,
    ordinal: u64,
    pool: Arc<dyn datafusion_execution::memory_pool::MemoryPool>,
    first_owner: Option<Arc<pse_columnar::AllocationLease>>,
    _lookahead: Arc<pse_columnar::AllocationLease>,
}
impl PhysicalGroups {
    pub(super) fn new(reader: PhysicalSpoolReader, pool: &Arc<dyn datafusion_execution::memory_pool::MemoryPool>) -> Result<Self, WorkflowError> {
        let lookahead = MemoryConsumer::new("physical:grouped-v2-lookahead").register(pool);
        lookahead.try_grow(PAYLOAD_LIMIT+4096).map_err(engine)?;
        let first_owner = Self::grant(pool)?;
        Ok(Self { reader, pending: None, ordinal: 0, pool: pool.clone(), first_owner: Some(first_owner),
            _lookahead: pse_columnar::AllocationLease::new(lookahead) })
    }
    fn grant(pool: &Arc<dyn datafusion_execution::memory_pool::MemoryPool>) -> Result<Arc<pse_columnar::AllocationLease>, WorkflowError> {
        let owner = MemoryConsumer::new("physical:grouped-v2-payload-and-plan").register(pool);
        owner.try_grow(GROUP_BYTES+GROUP_HEADER_BYTES).map_err(engine)?;
        Ok(pse_columnar::AllocationLease::new(owner))
    }
    pub(super) async fn next(&mut self) -> Result<Option<PhysicalGroup>, WorkflowError> {
        let Some(mut payload) = (match self.pending.take() {
            Some(payload) => Some(payload), None => self.reader.next().await?,
        }) else { self.first_owner = None; return Ok(None); };
        // The one read is prepaid by the reader/lookahead grant. Every escaped
        // group gets its own grant before retaining payloads or plan metadata.
        let owner = match self.first_owner.take() {
            Some(owner) => owner,
            None => match Self::grant(&self.pool) {
                Ok(owner) => owner,
                Err(error) => { self.pending = Some(payload); return Err(error); }
            },
        };
        let first = self.ordinal;
        let mut blocks = Vec::with_capacity(GROUP_BLOCKS);
        let mut bytes = 0;
        loop {
            if bytes + payload.len() > GROUP_BYTES {
                self.pending = Some(payload);
                break;
            }
            self.ordinal = self.ordinal.checked_add(1).ok_or_else(|| contract("physical group ordinal"))?;
            bytes += payload.len();
            blocks.push(payload);
            if blocks.len()==GROUP_BLOCKS { break; }
            payload = match self.reader.next().await? { Some(payload) => payload, None => break };
        }
        Ok(Some(PhysicalGroup { first, end: self.ordinal, blocks, owner }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn physical_groups_preserve_exact_bytes_count_and_length_boundaries_in_memory_and_spill() {
        for payloads in [vec![vec![1;17],vec![2;19],vec![3;23],vec![4;29],vec![5;31]],
            vec![vec![6;PAYLOAD_LIMIT],vec![7;PAYLOAD_LIMIT/2],vec![8;PAYLOAD_LIMIT],vec![9;1]]] {
            let runtime = crate::workflow::tests::runtime();
            let env = runtime.shared.runtime_env();
            let before = env.memory_pool.reserved();
            let mut observed = Vec::new();
            for memory_limit in [0, payloads.iter().map(|p|p.len()+8).sum()] {
                let mut spool = PhysicalSpool::new(env.clone(),memory_limit).unwrap();
                for payload in &payloads { spool.push(payload).unwrap(); }
                let mut groups = PhysicalGroups::new(spool.finish().unwrap(),&env.memory_pool).unwrap();
                let mut boundaries = Vec::new(); let mut original = Vec::new();
                while let Some(group) = groups.next().await.unwrap() {
                    assert!(group.blocks.len()<=GROUP_BLOCKS);
                    assert!(group.blocks.iter().map(Vec::len).sum::<usize>()<=GROUP_BYTES);
                    boundaries.push((group.first,group.end)); original.extend(group.blocks);
                }
                assert_eq!(original,payloads);
                observed.push(boundaries);
                drop(groups);
                assert_eq!(env.memory_pool.reserved(),before);
            }
            assert_eq!(observed[0],observed[1]);
            assert_eq!(observed[0],if payloads.len()==5 {vec![(0,4),(4,5)]} else {vec![(0,2),(2,4)]});
        }
    }
    #[tokio::test]
    async fn physical_groups_refuse_before_allocation_and_escaped_payloads_keep_their_grant() {
        let env = datafusion_execution::runtime_env::RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(datafusion_execution::memory_pool::GreedyMemoryPool::new(16<<20)))
            .build_arc().unwrap();
        let pressure = MemoryConsumer::new("test:physical-group-pressure").register(&env.memory_pool);
        pressure.try_grow(5<<20).unwrap();
        let mut spool = PhysicalSpool::new(env.clone(),0).unwrap(); spool.push(&[1;17]).unwrap();
        assert!(PhysicalGroups::new(spool.finish().unwrap(),&env.memory_pool).is_err());
        assert_eq!(env.memory_pool.reserved(),pressure.size());
        drop(pressure);
        let mut spool = PhysicalSpool::new(env.clone(),0).unwrap(); spool.push(&[1;17]).unwrap();
        let mut groups = PhysicalGroups::new(spool.finish().unwrap(),&env.memory_pool).unwrap();
        let group = groups.next().await.unwrap().unwrap();
        assert_eq!((group.first,group.end),(0,1));
        drop(groups);
        assert_eq!(env.memory_pool.reserved(),group.owner.size());
        assert_eq!(group.blocks,[vec![1;17]]);
        drop(group);
        assert_eq!(env.memory_pool.reserved(),0);
    }
    #[tokio::test]
    async fn physical_groups_preserve_next_membership_when_prior_payloads_hold_capacity() {
        let env = datafusion_execution::runtime_env::RuntimeEnvBuilder::new()
            .with_memory_pool(Arc::new(datafusion_execution::memory_pool::GreedyMemoryPool::new(16<<20)))
            .build_arc().unwrap();
        let mut spool = PhysicalSpool::new(env.clone(),0).unwrap();
        for byte in 0..5 { spool.push(&[byte;17]).unwrap(); }
        let mut groups = PhysicalGroups::new(spool.finish().unwrap(),&env.memory_pool).unwrap();
        let first = groups.next().await.unwrap().unwrap();
        assert_eq!((first.first,first.end),(0,4));
        assert!(groups.next().await.is_err(),"each outstanding group needs its own actual payload grant");
        drop(first);
        let second = groups.next().await.unwrap().unwrap();
        assert_eq!((second.first,second.end),(4,5)); assert_eq!(second.blocks,[vec![4;17]]);
        drop(second); drop(groups); assert_eq!(env.memory_pool.reserved(),0);
    }

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
