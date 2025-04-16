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
    storage: Arc<Storage>,
}

impl ServiceFactory<Self> for GenerateBlockEventPacemaker {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        let dag = ctx
            .get_shared::<BlockDAG>()
            .unwrap_or_else(|e| panic!("BlockDAG should exist, error: {:?}", e));
        let storage = ctx
            .get_shared::<Arc<Storage>>()
            .unwrap_or_else(|e| panic!("Storage should exist, error: {:?}", e));
        let self_ref = Self {
            config: ctx.get_shared::<Arc<NodeConfig>>()?,
            sync_status: None,
            dag,
            storage,
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

    fn dag_current_state(&self) -> Result<DagState> {
        let startup_info = self
            .storage
            .get_startup_info()?
            .ok_or_else(|| anyhow!("StartupInfo not found, maybe the node is not started yet"))?;
        let head_block = self
            .storage
            .get_block_header_by_hash(startup_info.main)?
            .ok_or_else(|| anyhow!("BlockHeader should exist"))?;

        if head_block.is_genesis() {
            Ok(DagState {
                tips: vec![head_block.id()],
            })
        } else {
            self.dag.get_dag_state(head_block.pruning_point())
        }
    }

    fn try_mint_later(&self, ctx: &mut ServiceContext<'_, Self>, try_count: u64, delay: u64) {
        ctx.run_later(Duration::from_millis(delay), move |ctx| {
            let self_ref = match ctx.get_shared::<Self>() {
                Ok(self_ref) => self_ref,
                Err(e) => {
                    warn!(
                        "Failed to get self reference when trying to mint block event: {:?}",
                        e
                    );
                    return;
                }
            };
            let dag_state = match self_ref.dag_current_state() {
                Ok(state) => Arc::new(state),
                Err(e) => {
                    warn!(
                        "Failed to get dag state when trying to mint block event: {:?}",
                        e
                    );
                    return;
                }
            };
            ctx.notify(TryMintBlockEvent {
                dag_state,
                try_count,
            });
        });
    }

    fn diff_blue_score(&self, last_dag_state: &DagState) -> Result<u64> {
        let last_dag_ghost_data = self
            .dag
            .ghost_dag_manager()
            .find_selected_parent(last_dag_state.tips.clone())
            .and_then(|id| {
                self.dag.storage.ghost_dag_store.get_data(id).map_err(|e| {
                    anyhow!(
                        "failed to get the ghost dag data when trying to mint for now: {:?}",
                        e
                    )
                })
            })?;
        let tips = self.dag_current_state()?.tips;
        let current_dag_ghost_data = self
            .dag
            .ghost_dag_manager()
            .find_selected_parent(tips)
            .and_then(|id| {
                self.dag.storage.ghost_dag_store.get_data(id).map_err(|e| {
                    anyhow!(
                        "failed to get the ghost dag data when trying to mint for now: {:?}",
                        e
                    )
                })
            })?;

        Ok(current_dag_ghost_data
            .blue_score
            .saturating_sub(last_dag_ghost_data.blue_score))
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
        self.try_mint_later(ctx, 5, 1000);
    }
}

impl EventHandler<Self, TryMintBlockEvent> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, msg: TryMintBlockEvent, ctx: &mut ServiceContext<Self>) {
        // if !self.is_synced() {
        //     self.try_mint_later(ctx, 5, 1000);
        //     return;
        // }
        let blue_count = match self.diff_blue_score(&msg.dag_state) {
            Ok(count) => count,
            Err(e) => {
                warn!(
                    "Failed to get diff blue score when trying to mint block event: {:?}",
                    e
                );
                return;
            }
        };

        if blue_count < msg.try_count {
            self.send_event(true, ctx);
            self.try_mint_later(ctx, msg.try_count.saturating_sub(blue_count), 25);
        } else {
            self.try_mint_later(ctx, 5, 1000);
        }
    }
}

impl EventHandler<Self, NewHeadBlock> for GenerateBlockEventPacemaker {
    fn handle_event(&mut self, _msg: NewHeadBlock, ctx: &mut ServiceContext<Self>) {
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
