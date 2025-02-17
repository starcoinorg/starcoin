// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, format_err, Result};
use futures::future::BoxFuture;
use futures::FutureExt;
use starcoin_chain_api::range_locate::{find_common_header_in_range, FindCommonHeader};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_network_rpc_api::RangeInLocation;
use starcoin_storage::Store;
use starcoin_types::block::BlockIdAndNumber;
use std::sync::Arc;
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
    type Item = BlockIdAndNumber;

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
                    RangeInLocation::NotInSelectedChain => {
                        let block_header = self
                            .storage
                            .get_block_header_by_hash(start_id)?
                            .ok_or_else(|| {
                                format_err!("Cannot find block header by hash1: {}", start_id)
                            })?;
                        end_id = Some(start_id);
                        start_id = block_header.pruning_point();
                        if start_id == HashValue::zero() {
                            start_id = self.storage.get_genesis()?.ok_or_else(|| {
                                format_err!("faild to get the genesis in find range locate task")
                            })?;
                        }
                    }
                    RangeInLocation::InSelectedChain(hash_value, hash_values) => {
                        if hash_values.is_empty() {
                            let header = self
                                .storage
                                .get_block_header_by_hash(hash_value)?
                                .ok_or_else(|| {
                                    format_err!("Cannot find block header by hash3: {}", hash_value)
                                })?;
                            return Ok(vec![BlockIdAndNumber {
                                id: header.id(),
                                number: header.number(),
                            }]);
                        } else {
                            if hash_values.len() == 1 {
                                let found_id = if let Some(found_id) = found_common_header {
                                    found_id
                                } else {
                                    hash_value
                                };
                                let header = self
                                    .storage
                                    .get_block_header_by_hash(found_id)?
                                    .ok_or_else(|| {
                                        format_err!(
                                            "In the last step, cannot find block header by hash2: {}",
                                            hash_values.last().unwrap()
                                        )
                                    })?;
                                return Ok(vec![BlockIdAndNumber {
                                    id: header.id(),
                                    number: header.number(),
                                }]);
                            }
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
                                        Some(*hash_values.last().expect("cannot be none!"));
                                    start_id = found_common_header.expect("cannot be none!");
                                    end_id = None;
                                }
                                FindCommonHeader::InRange(result_start_id, result_end_id) => {
                                    found_common_header = Some(result_start_id);
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

            let found_id = found_common_header
                .ok_or_else(|| format_err!("failed to find the dag common header2!"))?;
            let header = self
                .storage
                .get_block_header_by_hash(found_id)?
                .ok_or_else(|| format_err!("Cannot find block header by hash3: {}", found_id))?;
            Ok(vec![BlockIdAndNumber {
                id: header.id(),
                number: header.number(),
            }])
        }
        .boxed()
    }

    fn next(&self) -> Option<Self> {
        None
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

impl TaskResultCollector<BlockIdAndNumber> for DagAncestorCollector {
    type Output = BlockIdAndNumber;

    fn collect(&mut self, item: BlockIdAndNumber) -> Result<CollectorState> {
        let block_info = self.storage.get_block_info(item.id())?.ok_or_else(|| {
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
        if !self.dag.has_block_connected(&block_header)? {
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
