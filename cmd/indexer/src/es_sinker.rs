use crate::{BlockData, TransactionData};
use anyhow::Result;
use elasticsearch::indices::{
    IndicesCreateParts, IndicesExistsParts, IndicesGetMappingParts, IndicesPutMappingParts,
};
use elasticsearch::{BulkOperation, BulkOperations, BulkParts, Elasticsearch, GetParts};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starcoin_crypto::HashValue;
use starcoin_logger::prelude::*;
use starcoin_rpc_api::types::BlockView;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_types::transaction::Transaction;
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
#[derive(Clone, Debug, Serialize, Deserialize)]
struct BlockWithMetadata {
    #[serde(flatten)]
    block: BlockView,
    metadata: Option<BlockMetadata>,
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

        let BlockWithMetadata {
            block: block_view,
            metadata,
        } = if data["found"].as_bool().unwrap() {
            serde_json::from_value(data["_source"].clone())?
        } else {
            anyhow::bail!("cannot get block data with id {}", block_id);
        };
        // start deleting
        let mut bulk_operations = BulkOperations::new();
        bulk_operations.push(BulkOperation::<BlockView>::delete(block_id).index(block_index))?;

        // also remove metadata txn
        let txn_info_index = self.config.txn_info_index.as_str();

        // remove metadata txn if exists.
        if let Some(metadata) = metadata {
            bulk_operations.push(
                BulkOperation::<TransactionData>::delete(
                    Transaction::BlockMetadata(metadata).id().to_string(),
                )
                .index(txn_info_index),
            )?;
        }

        for txn_hash in block_view.body.txn_hashes() {
            bulk_operations.push(
                BulkOperation::<TransactionData>::delete(txn_hash.to_string())
                    .index(txn_info_index),
            )?;
        }
        let resp = self
            .es
            .bulk(BulkParts::None)
            .body(vec![bulk_operations])
            .send()
            .await?;

        let exception = resp.exception().await?;
        if let Some(ex) = exception {
            anyhow::bail!("{}", serde_json::to_string(&ex)?);
        }

        // rollback tip header
        let rollback_to = (block_view.header.parent_hash, block_view.header.number - 1);
        self.update_local_tip_header(rollback_to.0, rollback_to.1)
            .await?;
        info!(
            "Rollback to block: {}, height: {}",
            rollback_to.0, rollback_to.1
        );
        Ok(())
    }

    /// write new block into es.
    /// Caller need to make sure the block with right block number.
    pub async fn write_next_block(&self, block: BlockData) -> Result<()> {
        let BlockData { block, txns_data } = block;

        // TODO: check against old tip info
        let tip_info = LocalTipInfo {
            block_hash: block.header.block_hash,
            block_number: block.header.number,
        };

        let block_index = self.config.block_index.as_str();
        let txn_info_index = self.config.txn_info_index.as_str();
        let mut bulk_operations = BulkOperations::new();
        bulk_operations.push(
            BulkOperation::index(BlockWithMetadata {
                block: block.clone(),
                metadata: txns_data[0].block_metadata.clone().map(Into::into),
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

        let resp = self
            .es
            .bulk(BulkParts::None)
            .body(vec![bulk_operations])
            .send()
            .await?;
        let exception = resp.exception().await?;
        if let Some(ex) = exception {
            anyhow::bail!("{}", serde_json::to_string(&ex)?);
        }
        self.update_local_tip_header(tip_info.block_hash, tip_info.block_number)
            .await?;
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
