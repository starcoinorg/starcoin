// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::tasks::BlockIdFetcher;
use anyhow::{bail, format_err, Result};
use futures::FutureExt;
use futures::{executor::block_on, future::BoxFuture};
use starcoin_accumulator::{Accumulator, MerkleAccumulator};
use starcoin_chain::{BlockChain, ChainReader};
use starcoin_chain_api::range_locate::{find_common_header_in_range, FindCommonHeader};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_logger::prelude::error;
use starcoin_network_rpc_api::RangeInPruningPoint;
use starcoin_storage::Store;
use starcoin_types::block::{Block, BlockIdAndNumber, BlockNumber};
use std::sync::Arc;
use stest::actix_export::Arbiter;
use stream_task::{CollectorState, TaskResultCollector, TaskState};

use super::BlockIdRangeFetcher;

#[derive(Clone)]
pub struct FindRangeLocateTask {
    start_id: HashValue,
    end_id: Option<HashValue>,
    fetcher: Arc<dyn BlockIdRangeFetcher>,
    storage: Arc<dyn Store>,
    dag: BlockDAG,
}

impl FindRangeLocateTask {
    pub fn new<F>(
        start_id: HashValue,
        end_id: Option<HashValue>,
        fetcher: F,
        storage: Arc<dyn Store>,
        dag: BlockDAG,
    ) -> Self
    where
        F: BlockIdRangeFetcher + 'static,
    {
        Self {
            start_id,
            end_id,
            fetcher: Arc::new(fetcher),
            storage,
            dag,
        }
    }
}

impl TaskState for FindRangeLocateTask {
    type Item = Option<HashValue>;

    fn new_sub_task(self) -> BoxFuture<'static, Result<Vec<Self::Item>>> {
        async move {
            let mut start_id = self.start_id;
            let mut end_id = self.end_id;
            let mut found_common_header = None;
            loop {
                match self
                    .fetcher
                    .fetch_range_locate(None, start_id, end_id)
                    .await?
                {
                    RangeInPruningPoint::NotInSelectedChain => {
                        let block_header = self
                            .storage
                            .get_block_header_by_hash(start_id)?
                            .ok_or_else(|| {
                                format_err!("Cannot find block header by hash: {}", start_id)
                            })?;
                        end_id = Some(start_id);
                        start_id = block_header.pruning_point();
                        if start_id == HashValue::zero() {
                            start_id = self.storage.get_genesis()?.ok_or_else(|| {
                                format_err!("faild to get the genesis in find range locate task")
                            })?;
                        }
                    }
                    RangeInPruningPoint::InSelectedChain(hash_value, hash_values) => {
                        if hash_values.len() == 0 {
                            return Ok(vec![Some(hash_value)]);
                        } else {
                            let find_result = find_common_header_in_range(&self.dag, &hash_values)
                                .map_err(|err| {
                                    format_err!(
                                        "failed to find_common_header_in_range, error: {:?}",
                                        err
                                    )
                                })?;

                            match find_result {
                                FindCommonHeader::AllInRange => {
                                    found_common_header =
                                        Some(hash_values.last().expect("cannot none!").clone());
                                    start_id = hash_values.last().unwrap().clone();
                                    end_id = None;
                                }
                                FindCommonHeader::InRange(result_start_id, result_end_id) => {
                                    found_common_header = Some(start_id);
                                    start_id = result_start_id;
                                    end_id = Some(result_end_id);
                                }
                                FindCommonHeader::Found(hash_value) => {
                                    found_common_header = Some(hash_value);
                                    break;
                                }
                                FindCommonHeader::NotInRange => break,
                            }
                        }
                    }
                }
            }

            Ok(vec![found_common_header])
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        //this should never happen, because all node's genesis block should same.
        let genesis_id = self
            .storage
            .get_genesis()
            .expect("failed to get genesis in find common ancestor task next function!")
            .expect("genesis should not be none");
        if self.start_id == genesis_id {
            error!("no common ancestor found!");
            return None;
        }

        let next_start_id = match self.storage.get_block_header_by_hash(self.start_id) {
            Ok(op_header) => {
                let header = if let Some(header) = op_header {
                    header
                } else {
                    error!(
                        "cannot find the block header by start id: {:?}",
                        self.start_id
                    );
                    return None;
                };
                header.pruning_point()
            }
            Err(e) => {
                error!(
                    "cannot find the block header by start id: {:?}, error: {:?}",
                    self.start_id, e
                );
                return None;
            }
        };
        Some(Self {
            start_id: next_start_id,
            end_id: Some(self.start_id),
            fetcher: self.fetcher.clone(),
            storage: self.storage.clone(),
            dag: self.dag.clone(),
        })
    }
}

pub struct DagAncestorCollector {
    dag: BlockDAG,
    storage: Arc<dyn Store>,
    ancestor: Option<BlockIdAndNumber>,
}

impl DagAncestorCollector {
    pub fn new(dag: BlockDAG, storage: Arc<dyn Store>) -> Self {
        Self {
            dag,
            storage,
            ancestor: None,
        }
    }
}

impl TaskResultCollector<HashValue> for DagAncestorCollector {
    type Output = BlockIdAndNumber;

    fn collect(&mut self, item: HashValue) -> Result<CollectorState> {
        let block_info = self.storage.get_block_info(item)?.ok_or_else(|| {
            format_err!(
                "failed to get the block info by found common ancestor id: {:?}",
                item
            )
        })?;
        let block_header = self
            .storage
            .get_block_header_by_hash(*block_info.block_id())?
            .ok_or_else(|| {
                format_err!(
                    "failed to get the block header by found common ancestor id: {:?}",
                    block_info.block_id()
                )
            })?;
        if self.dag.has_block_connected(&block_header)? {
            bail!(
                "failed to check the found common ancestor in dag, id: {:?}",
                item
            );
        }
        self.ancestor = Some(BlockIdAndNumber {
            id: block_header.id(),
            number: block_header.number(),
        });
        Ok(CollectorState::Enough)
    }

    fn finish(mut self) -> Result<Self::Output> {
        self.ancestor
            .take()
            .ok_or_else(|| format_err!("Unexpect state, collector finished by ancestor is None"))
    }
}
