use crate::{BlockData, BlockWithMetadata};
use anyhow::Result;
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
    pub txn_info_index: String,
}

impl IndexConfig {
    pub fn new_with_prefix(prefix: impl AsRef<str>) -> Self {
        Self {
            block_index: format!("{}.blocks", prefix.as_ref()),
            txn_info_index: format!("{}.txn_infos", prefix.as_ref()),
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            block_index: "blocks".to_string(),
            txn_info_index: "txn_infos".to_string(),
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
        let txn_info_index = self.config.txn_info_index.as_str();
        self.create_index_if_not_exists(block_index).await?;
        self.create_index_if_not_exists(txn_info_index).await?;
        let tip = self._get_local_tip_header().await?;
        self.state.write().await.tip = tip;
        Ok(())
    }

    async fn update_local_tip_header(
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
        self.state.write().await.tip = Some(tip_info);
        Ok(data)
    }

    pub async fn get_local_tip_header(&self) -> Result<Option<LocalTipInfo>> {
        let tip = self.state.read().await.tip.clone();
        Ok(tip)
    }

    async fn _get_local_tip_header(&self) -> Result<Option<LocalTipInfo>> {
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
        let BlockData { block, txns_data } = block;

        let block_index = self.config.block_index.as_str();
        let txn_info_index = self.config.txn_info_index.as_str();
        let mut bulk_operations = BulkOperations::new();
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
        }

        self.bulk(bulk_operations).await?;
        Ok(())
    }

    /// write new block into es.
    /// Caller need to make sure the block with right block number.
    pub async fn write_next_block(&self, block: BlockData) -> Result<()> {
        let BlockData { block, txns_data } = block;

        // TODO: check against old tip info
        let tip_info = LocalTipInfo {
            block_hash: block.header.block_hash,
            block_number: block.header.number.0,
        };

        let block_index = self.config.block_index.as_str();
        let txn_info_index = self.config.txn_info_index.as_str();
        let mut bulk_operations = BulkOperations::new();
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
        }

        self.bulk(bulk_operations).await?;

        self.update_local_tip_header(tip_info.block_hash, tip_info.block_number)
            .await?;
        Ok(())
    }

    // bulk insert data into es.
    async fn bulk(&self, bulk_operations: BulkOperations) -> anyhow::Result<()> {
        let resp = self
            .es
            .bulk(BulkParts::None)
            .body(vec![bulk_operations])
            .send()
            .await?;

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
            .update_local_tip_header(tip_info.block_hash, tip_info.block_number)
            .await
            .unwrap();

        let v = sinker.get_local_tip_header().await.unwrap().unwrap();
        assert_eq!(v, tip_info);
    }
}
