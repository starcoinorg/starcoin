// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{GenerateBlockEvent, TryMintBlockEvent};
use anyhow::{anyhow, Result};
use core::panic;
use starcoin_config::NodeConfig;
use starcoin_dag::{
    blockdag::BlockDAG,
    consensusdb::{consenses_state::DagState, schemadb::GhostdagStoreReader},
};
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_storage::{BlockStore, Storage};
use starcoin_txpool_api::PropagateTransactions;
use starcoin_types::{
    sync_status::SyncStatus,
    system_events::{NewDagBlockFromPeer, NewHeadBlock, SyncStatusChangeEvent, SystemStarted},
};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct GenerateBlockEventPacemaker {
    config: Arc<NodeConfig>,
    sync_status: Option<SyncStatus>,
    dag: BlockDAG,
    dag_state: DagState,
}

impl ServiceFactory<Self> for GenerateBlockEventPacemaker {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let dag = ctx
            .get_shared::<BlockDAG>()
            .unwrap_or_else(|e| panic!("BlockDAG should exist, error: {:?}", e));
        let storage = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap_or_else(|e| panic!("Storage should exist, error: {:?}", e));
        let startup_info = storage
            .get_startup_info()?
            .ok_or_else(|| anyhow!("StartupInfo not found, maybe the node is not started yet"))?;
        let head_block = storage
            .get_block_header_by_hash(startup_info.main)?
            .ok_or_else(|| anyhow!("BlockHeader should exist"))?;

        let dag_state = if head_block.is_genesis() {
            DagState {
                tips: vec![head_block.id()],
            }
        } else {
            match dag.get_dag_state(head_block.pruning_point()) {
                Ok(state) => state,
                Err(_) => DagState {
                    tips: vec![head_block.id()],
                },
            }
        };
        let self_ref = Self {
            config: ctx.get_shared::<Arc<NodeConfig>>()?,
            sync_status: None,
            dag,
            dag_state,
        };
        ctx.put_shared(self_ref.clone())?;
        Ok(self_ref)
    }
}

impl GenerateBlockEventPacemaker {
    pub fn send_event(&mut self, force: bool, ctx: &mut ServiceContext<Self>) {
        ctx.broadcast(GenerateBlockEvent::new_break(force));
    }

    pub fn is_synced(&self) -> bool {
        match self.sync_status.as_ref() {
            Some(sync_status) => sync_status.is_synced(),
            None => false,
        }
    }

    fn try_mint_later(
        &self,
        ctx: &mut ServiceContext<'_, Self>,
        current_blue_score: u64,
        try_count: u64,
        delay: u64,
    ) {
        ctx.run_later(Duration::from_millis(delay), move |ctx| {
            ctx.notify(TryMintBlockEvent {
                last_blue_score: current_blue_score,
                try_count,
            });
        });
    }

    fn current_blue_score(&self) -> Result<u64> {
        let current_dag_ghost_data = self
            .dag
            .ghost_dag_manager()
            .find_selected_parent(self.dag_state.tips.clone())
            .and_then(|id| {
                self.dag.storage.ghost_dag_store.get_data(id).map_err(|e| {
                    anyhow!(
                        "failed to get the ghost dag data when trying to mint for now: {:?}",
                        e
                    )
                })
            })?;
        Ok(current_dag_ghost_data.blue_score)
    }
}

impl ActorService for GenerateBlockEventPacemaker {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<SyncStatusChangeEvent>();
        ctx.subscribe::<NewHeadBlock>();
        ctx.subscribe::<NewDagBlockFromPeer>();
        ctx.subscribe::<SystemStarted>();
        ctx.subscribe::<TryMintBlockEvent>();
        //if mint empty block is disabled, trigger mint event for on demand mint (Dev)
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.subscribe::<PropagateTransactions>();
        }
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<SyncStatusChangeEvent>();
        ctx.unsubscribe::<NewHeadBlock>();
        ctx.unsubscribe::<NewDagBlockFromPeer>();
        ctx.unsubscribe::<SystemStarted>();
        ctx.unsubscribe::<TryMintBlockEvent>();
        if self.config.miner.is_disable_mint_empty_block() {
            ctx.unsubscribe::<PropagateTransactions>();
        }
        Ok(())
    }
}

impl EventHandler<Self, SystemStarted> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: SystemStarted, ctx: &mut ServiceContext<Self>) {
        self.try_mint_later(ctx, self.current_blue_score().unwrap_or(0), 5, 1000);
    }
}

impl EventHandler<Self, TryMintBlockEvent> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: TryMintBlockEvent, ctx: &mut ServiceContext<Self>) {
        // if !self.is_synced() {
        //     self.try_mint_later(ctx, 5, 1000);
        //     return;
        // }
        info!("jacktest: TryMintBlockEvent, msg: {:?}", msg);
        let current_blue_score = match self.current_blue_score() {
            Ok(score) => score,
            Err(e) => {
                warn!(
                    "failed to get the current score when trying to mint block event: {:?}",
                    e
                );
                return;
            }
        };
        let diff_blue_score = current_blue_score.saturating_sub(msg.last_blue_score);
        info!(
            "jacktest: TryMintBlockEvent, last state blue score: {:?}, current: {:?}, sub: {:?}",
            msg.last_blue_score, current_blue_score, diff_blue_score
        );

        if diff_blue_score < msg.try_count {
            info!("jacktest: TryMintBlockEvent, send mint event");
            self.send_event(true, ctx);
            self.try_mint_later(
                ctx,
                current_blue_score,
                msg.try_count.saturating_sub(diff_blue_score),
                25,
            );
        } else {
            info!("jacktest: TryMintBlockEvent, do not send mint event");
            self.try_mint_later(ctx, current_blue_score, 10, 1000);
        }
    }
}

impl EventHandler<Self, NewHeadBlock> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
        self.dag_state = match self
            .dag
            .get_dag_state(msg.executed_block.block().header().pruning_point())
        {
            Ok(state) => state,
            Err(_) => {
                warn!("cannot find the dag state in new head block!");
                DagState {
                    tips: vec![msg.executed_block.block().header().id()],
                }
            }
        };
        if self.is_synced() {
            self.send_event(true, ctx)
        } else {
            debug!("[pacemaker] Ignore NewHeadBlock event because the node has not been synchronized yet.")
        }
    }
}

impl EventHandler<Self, PropagateTransactions> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: PropagateTransactions, ctx: &mut ServiceContext<Self>) {
        if self.is_synced() {
            self.send_event(false, ctx)
        } else {
            debug!("[pacemaker] Ignore PropagateNewTransactions event because the node has not been synchronized yet.")
        }
    }
}

impl EventHandler<Self, SyncStatusChangeEvent> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: SyncStatusChangeEvent, ctx: &mut ServiceContext<Self>) {
        let is_synced = msg.0.is_synced();
        self.sync_status = Some(msg.0);
        if is_synced {
            self.send_event(false, ctx);
        }
    }
}

impl EventHandler<Self, NewDagBlockFromPeer> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewDagBlockFromPeer, ctx: &mut ServiceContext<Self>) {
        self.send_event(true, ctx);
    }
}
