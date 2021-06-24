use crate::{BlockData, BlockSimplified, BlockWithMetadata, EventData};
use anyhow::Result;
use elasticsearch::http::response::Response;
use elasticsearch::indices::{
    IndicesCreateParts, IndicesExistsParts, IndicesGetMappingParts, IndicesPutMappingParts,
};
use elasticsearch::{
    BulkOperation, BulkOperations, BulkParts, DeleteByQueryParts, DeleteParts, Elasticsearch,
    GetParts,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub struct IndexConfig {
    pub block_index: String,
    pub uncle_block_index: String,
    pub txn_info_index: String,
    pub pending_txn_index: String,
    pub txn_event_index: String,
}

impl IndexConfig {
    pub fn new_with_prefix(prefix: impl AsRef<str>) -> Self {
        Self {
            block_index: format!("{}.blocks", prefix.as_ref()),
            uncle_block_index: format!("{}.uncle_blocks", prefix.as_ref()),
            txn_info_index: format!("{}.txn_infos", prefix.as_ref()),
            pending_txn_index: format!("{}.pending_txns", prefix.as_ref()),
            txn_event_index: format!("{}.txn_events", prefix.as_ref()),
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            block_index: "blocks".to_string(),
            uncle_block_index: "uncle_blocks".to_string(),
            txn_info_index: "txn_infos".to_string(),
            pending_txn_index: "pending_txns".to_string(),
            txn_event_index: "txn_events".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct EsSinker {
    es: Elasticsearch,
    config: IndexConfig,
    state: RwLock<SinkState>,
}

#[derive(Clone, Debug, Default)]
struct SinkState {
    tip: Option<LocalTipInfo>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalTipInfo {
    pub block_hash: HashValue,
    pub block_number: u64,
}

impl EsSinker {
    pub fn new(es: Elasticsearch, config: IndexConfig) -> Self {
        Self {
            es,
            config,
            state: Default::default(),
        }
    }

    async fn create_index_if_not_exists(&self, index: &str) -> Result<()> {
        let exists = self
            .es
            .indices()
            .exists(IndicesExistsParts::Index(&[index]))
            .send()
            .await?
            .status_code()
            .is_success();
        if !exists {
            self.es
                .indices()
                .create(IndicesCreateParts::Index(index))
                .send()
                .await?
                .error_for_status_code()?;
        }
        Ok(())
    }
    /// init es indices
    pub async fn init_indices(&self) -> Result<()> {
        let block_index = self.config.block_index.as_str();
        let uncle_block_index = self.config.uncle_block_index.as_str();
        let txn_info_index = self.config.txn_info_index.as_str();
        let txn_event_index = self.config.txn_event_index.as_str();
        let pending_txn_index = self.config.pending_txn_index.as_str();
        self.create_index_if_not_exists(block_index).await?;
        self.create_index_if_not_exists(uncle_block_index).await?;
        self.create_index_if_not_exists(txn_info_index).await?;
        self.create_index_if_not_exists(txn_event_index).await?;
        self.create_index_if_not_exists(pending_txn_index).await?;
        let tip = self.get_remote_tip_header().await?;
        self.state.write().await.tip = tip.clone();
        if let Some(tip_info) = tip {
            info!(
                "remote tips: {}, {}",
                tip_info.block_hash, tip_info.block_number
            );
        }
        Ok(())
    }

    pub async fn update_remote_tip_header(
        &self,
        block_hash: HashValue,
        block_number: u64,
    ) -> Result<serde_json::Value> {
        let tip_info = LocalTipInfo {
            block_number,
            block_hash,
        };
        let body = serde_json::json!({
          "_meta": {
           "tip": tip_info
          }
        });

        let block_index = self.config.block_index.as_str();
        let resp = self
            .es
            .indices()
            .put_mapping(IndicesPutMappingParts::Index(&[block_index]))
            .body(body)
            .send()
            .await?;
        let resp = resp.error_for_status_code()?;
        let data = resp.json().await?;
        // self.state.write().await.tip = Some(tip_info);
        Ok(data)
    }

    pub async fn update_local_tip_header(
        &self,
        block_hash: HashValue,
        block_number: u64,
    ) -> Result<()> {
        let tip_info = LocalTipInfo {
            block_number,
            block_hash,
        };
        self.state.write().await.tip = Some(tip_info);
        Ok(())
    }

    pub async fn get_local_tip_header(&self) -> Result<Option<LocalTipInfo>> {
        let tip = self.state.read().await.tip.clone();
        Ok(tip)
    }

    async fn get_remote_tip_header(&self) -> Result<Option<LocalTipInfo>> {
        let block_index = self.config.block_index.as_str();
        let resp_data: Value = self
            .es
            .indices()
            .get_mapping(IndicesGetMappingParts::Index(&[block_index]))
            .send()
            .await?
            .error_for_status_code()?
            .json()
            .await?;
        let v = resp_data[block_index]["mappings"]["_meta"]["tip"].clone();
        let tip_info: Option<LocalTipInfo> = serde_json::from_value(v)?;
        Ok(tip_info)
    }

    pub async fn rollback_to_last_block(&self) -> Result<()> {
        let tip_header = self.get_local_tip_header().await?;
        if tip_header.is_none() {
            return Ok(());
        }
        let tip_header = tip_header.unwrap();
        let block_index = self.config.block_index.as_str();
        let block_id = tip_header.block_hash.to_string();
        let data: Value = self
            .es
            .get(GetParts::IndexId(block_index, block_id.as_str()))
            .send()
            .await?
            .error_for_status_code()?
            .json()
            .await?;

        let parent_hash: HashValue = if data["found"].as_bool().unwrap() {
            serde_json::from_value(data["_source"]["header"]["parent_hash"].clone())?
        } else {
            anyhow::bail!("cannot get block data with id {}", block_id);
        };

        // first, rollback tip header
        let rollback_to = (parent_hash, tip_header.block_number - 1);
        self.update_remote_tip_header(rollback_to.0, rollback_to.1)
            .await?;
        self.update_local_tip_header(rollback_to.0, rollback_to.1)
            .await?;

        // delete block
        {
            let resp = self
                .es
                .delete(DeleteParts::IndexId(block_index, block_id.as_str()))
                .send()
                .await?;
            let exception = resp.exception().await?;
            if let Some(ex) = exception {
                anyhow::bail!("{}", serde_json::to_string(&ex)?);
            }
        }
        // delete related txn infos.
        {
            let txn_info_index = self.config.txn_info_index.as_str();
            let search_condition = serde_json::json!({
                "query": {
                    "match": {
                        "block_hash": block_id,
                    }
                }
            });
            let resp = self
                .es
                .delete_by_query(DeleteByQueryParts::Index(&[txn_info_index]))
                .body(search_condition)
                .send()
                .await?;

            // check response
            if resp.status_code().is_client_error() || resp.status_code().is_server_error() {
                let exception = resp.exception().await?;
                if let Some(ex) = exception {
                    anyhow::bail!("{}", serde_json::to_string(&ex)?);
                }
            } else {
                let delete_response: serde_json::Value = resp.json().await?;

                let total_txn = delete_response["total"].as_u64();
                let deleted_txns = delete_response["deleted"].as_u64();
                info!(
                    "cleanup block {}, total txns: {:?}, delete {:?} txns",
                    block_id, total_txn, deleted_txns
                );
            }
        }

        info!(
            "Rollback to block: {}, height: {}",
            rollback_to.0, rollback_to.1
        );

        Ok(())
    }

    pub async fn repair_block(&self, block: BlockData) -> Result<()> {
        self.bulk(vec![block]).await?;
        Ok(())
    }

    // bulk insert data into es.
    pub async fn bulk(&self, blocks: Vec<BlockData>) -> anyhow::Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }
        let mut bulk_operations = BulkOperations::new();
        let block_index = self.config.block_index.as_str();
        let txn_info_index = self.config.txn_info_index.as_str();
        let pending_txn_index = self.config.pending_txn_index.as_str();
        let uncle_index = self.config.uncle_block_index.as_str();
        let event_index = self.config.txn_event_index.as_str();

        for blockdata in blocks {
            let BlockData { block, txns_data } = blockdata;
            bulk_operations.push(
                BulkOperation::index(BlockWithMetadata {
                    block: block.clone(),
                    metadata: txns_data[0].block_metadata.clone(),
                })
                .id(block.header.block_hash.to_string())
                .index(block_index),
            )?;
            for txn_data in txns_data {
                bulk_operations.push(
                    BulkOperation::index(txn_data.clone())
                        .id(txn_data.info.transaction_hash.to_string())
                        .index(txn_info_index),
                )?;
                bulk_operations.push(
                    BulkOperation::<()>::delete(txn_data.info.transaction_hash.to_string())
                        .index(pending_txn_index),
                )?;
                if !txn_data.events.is_empty() {
                    // event_vec.extend_from_slice(txn_data.events.as_slice());
                    for event in txn_data.events {
                        let mut event_data = EventData::from(event.clone());
                        event_data.timestamp = txn_data.timestamp;
                        bulk_operations
                            .push(BulkOperation::index(event_data).index(event_index))?;
                    }
                }
            }

            //add uncle
            if !block.uncles.is_empty() {
                for uncle in block.uncles {
                    bulk_operations.push(
                        BulkOperation::index(BlockSimplified {
                            header: uncle.clone(),
                            uncle_block_number: block.header.number,
                        })
                        .id(uncle.block_hash.to_string())
                        .index(uncle_index),
                    )?;
                }
            }
        }

        let resp = self
            .es
            .bulk(BulkParts::None)
            .body(vec![bulk_operations])
            .send()
            .await?;

        EsSinker::check_status_code(resp).await
    }

    // bulk insert data into es.
    pub async fn bulk_uncle(&self, uncle_blocks: Vec<BlockData>) -> anyhow::Result<()> {
        if uncle_blocks.is_empty() {
            return Ok(());
        }
        let mut bulk_operations = BulkOperations::new();
        let block_index = self.config.uncle_block_index.as_str();
        for blockdata in uncle_blocks {
            let BlockData { block, txns_data } = blockdata;
            bulk_operations.push(
                BulkOperation::index(BlockWithMetadata {
                    block: block.clone(),
                    metadata: txns_data[0].block_metadata.clone(),
                })
                .id(block.header.block_hash.to_string())
                .index(block_index),
            )?;
        }

        let resp = self
            .es
            .bulk(BulkParts::None)
            .body(vec![bulk_operations])
            .send()
            .await?;

        EsSinker::check_status_code(resp).await
    }

    async fn check_status_code(resp: Response) -> anyhow::Result<()> {
        // check response
        if resp.status_code().is_client_error() || resp.status_code().is_server_error() {
            let exception = resp.exception().await?;
            if let Some(ex) = exception {
                anyhow::bail!("{}", serde_json::to_string(&ex)?);
            }
        } else {
            let bulk_response: serde_json::Value = resp.json().await?;
            if let Some(true) = bulk_response["errors"].as_bool() {
                anyhow::bail!(
                    "[es] bulk error: {}",
                    serde_json::to_string(&bulk_response)?
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::LocalTipInfo;
    use crate::{EsSinker, IndexConfig};
    use elasticsearch::http::transport::SingleNodeConnectionPool;
    use elasticsearch::http::Url;
    use elasticsearch::Elasticsearch;
    use starcoin_crypto::HashValue;
    use std::str::FromStr;

    #[tokio::test(threaded_scheduler)]
    #[ignore]
    async fn test_update_tip_header() {
        let es_url = "http://localhost:9200";
        let transport = elasticsearch::http::transport::TransportBuilder::new(
            SingleNodeConnectionPool::new(Url::from_str(es_url).unwrap()),
        )
        .build()
        .unwrap();

        let es = Elasticsearch::new(transport);
        let sinker = EsSinker {
            es,
            config: IndexConfig::default(),
            state: Default::default(),
        };
        let v = sinker.get_local_tip_header().await.unwrap();
        assert!(v.is_none());

        let tip_info = LocalTipInfo {
            block_hash: HashValue::random(),
            block_number: 1,
        };

        let _ = sinker
            .update_remote_tip_header(tip_info.block_hash, tip_info.block_number)
            .await
            .unwrap();

        let v = sinker.get_local_tip_header().await.unwrap().unwrap();
        assert_eq!(v, tip_info);
    }
}
