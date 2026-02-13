use anyhow::Result;
use starcoin_service_registry::{
    ActorService, ServiceContext, ServiceHandler, ServiceRef, ServiceRequest,
};
use std::collections::HashMap;

pub type ParallelWorkerId = u64;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkerSyncStat {
    pub worker_id: ParallelWorkerId,
    pub synced_block_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParallelSyncStat {
    pub worker_count: usize,
    pub total_synced_block_count: u64,
    pub workers: Vec<WorkerSyncStat>,
}

#[derive(Default)]
pub struct ParallelInfoService {
    worker_synced_blocks: HashMap<ParallelWorkerId, u64>,
}

impl ParallelInfoService {
    fn snapshot(&self) -> ParallelSyncStat {
        let mut workers: Vec<WorkerSyncStat> = self
            .worker_synced_blocks
            .iter()
            .map(|(worker_id, synced_block_count)| WorkerSyncStat {
                worker_id: *worker_id,
                synced_block_count: *synced_block_count,
            })
            .collect();
        workers.sort_unstable_by_key(|stat| stat.worker_id);

        let total_synced_block_count = workers.iter().fold(0_u64, |acc, stat| {
            acc.saturating_add(stat.synced_block_count)
        });

        ParallelSyncStat {
            worker_count: workers.len(),
            total_synced_block_count,
            workers,
        }
    }
}

impl ActorService for ParallelInfoService {}

#[derive(Clone, Debug)]
pub struct RegisterWorkerRequest {
    pub worker_id: ParallelWorkerId,
}

impl ServiceRequest for RegisterWorkerRequest {
    type Response = ();
}

impl ServiceHandler<Self, RegisterWorkerRequest> for ParallelInfoService {
    fn handle(&mut self, msg: RegisterWorkerRequest, _ctx: &mut ServiceContext<Self>) {
        self.worker_synced_blocks.entry(msg.worker_id).or_insert(0);
    }
}

#[derive(Clone, Debug)]
pub struct UnregisterWorkerRequest {
    pub worker_id: ParallelWorkerId,
}

impl ServiceRequest for UnregisterWorkerRequest {
    type Response = ();
}

impl ServiceHandler<Self, UnregisterWorkerRequest> for ParallelInfoService {
    fn handle(&mut self, msg: UnregisterWorkerRequest, _ctx: &mut ServiceContext<Self>) {
        self.worker_synced_blocks.remove(&msg.worker_id);
    }
}

#[derive(Clone, Debug)]
pub struct ReportWorkerSyncedBlockRequest {
    pub worker_id: ParallelWorkerId,
}

impl ServiceRequest for ReportWorkerSyncedBlockRequest {
    type Response = ();
}

impl ServiceHandler<Self, ReportWorkerSyncedBlockRequest> for ParallelInfoService {
    fn handle(&mut self, msg: ReportWorkerSyncedBlockRequest, _ctx: &mut ServiceContext<Self>) {
        let synced_block_count = self.worker_synced_blocks.entry(msg.worker_id).or_insert(0);
        *synced_block_count = synced_block_count.saturating_add(1);
    }
}

#[derive(Clone, Debug)]
pub struct ReportWorkerSyncedBlocksRequest {
    pub worker_id: ParallelWorkerId,
    pub block_count: u64,
}

impl ServiceRequest for ReportWorkerSyncedBlocksRequest {
    type Response = ();
}

impl ServiceHandler<Self, ReportWorkerSyncedBlocksRequest> for ParallelInfoService {
    fn handle(&mut self, msg: ReportWorkerSyncedBlocksRequest, _ctx: &mut ServiceContext<Self>) {
        let synced_block_count = self.worker_synced_blocks.entry(msg.worker_id).or_insert(0);
        *synced_block_count = synced_block_count.saturating_add(msg.block_count);
    }
}

#[derive(Clone, Debug)]
pub struct GetParallelSyncStatRequest;

impl ServiceRequest for GetParallelSyncStatRequest {
    type Response = ParallelSyncStat;
}

impl ServiceHandler<Self, GetParallelSyncStatRequest> for ParallelInfoService {
    fn handle(
        &mut self,
        _msg: GetParallelSyncStatRequest,
        _ctx: &mut ServiceContext<Self>,
    ) -> ParallelSyncStat {
        self.snapshot()
    }
}

#[derive(Clone, Debug)]
pub struct ResetParallelSyncStatRequest;

impl ServiceRequest for ResetParallelSyncStatRequest {
    type Response = ();
}

impl ServiceHandler<Self, ResetParallelSyncStatRequest> for ParallelInfoService {
    fn handle(&mut self, _msg: ResetParallelSyncStatRequest, _ctx: &mut ServiceContext<Self>) {
        self.worker_synced_blocks.clear();
    }
}

pub trait ParallelInfoAsyncService {
    fn register_worker(
        &self,
        worker_id: ParallelWorkerId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn unregister_worker(
        &self,
        worker_id: ParallelWorkerId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn report_worker_synced_block(
        &self,
        worker_id: ParallelWorkerId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn report_worker_synced_blocks(
        &self,
        worker_id: ParallelWorkerId,
        block_count: u64,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn get_parallel_sync_stat(
        &self,
    ) -> impl std::future::Future<Output = Result<ParallelSyncStat>> + Send;

    fn reset_parallel_sync_stat(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl ParallelInfoAsyncService for ServiceRef<ParallelInfoService> {
    async fn register_worker(&self, worker_id: ParallelWorkerId) -> Result<()> {
        self.send(RegisterWorkerRequest { worker_id }).await
    }

    async fn unregister_worker(&self, worker_id: ParallelWorkerId) -> Result<()> {
        self.send(UnregisterWorkerRequest { worker_id }).await
    }

    async fn report_worker_synced_block(&self, worker_id: ParallelWorkerId) -> Result<()> {
        self.send(ReportWorkerSyncedBlockRequest { worker_id }).await
    }

    async fn report_worker_synced_blocks(
        &self,
        worker_id: ParallelWorkerId,
        block_count: u64,
    ) -> Result<()> {
        self.send(ReportWorkerSyncedBlocksRequest {
            worker_id,
            block_count,
        })
        .await
    }

    async fn get_parallel_sync_stat(&self) -> Result<ParallelSyncStat> {
        self.send(GetParallelSyncStatRequest).await
    }

    async fn reset_parallel_sync_stat(&self) -> Result<()> {
        self.send(ResetParallelSyncStatRequest).await
    }
}
