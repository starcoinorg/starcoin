use std::sync::Arc;

use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_storage::Store;

pub fn has_dag_block(
    header_id: HashValue,
    storage: Arc<dyn Store>,
    dag: &BlockDAG,
) -> Result<bool> {
    let header = match storage.get_block_header_by_hash(header_id)? {
        Some(header) => header,
        None => return Ok(false),
    };

    if storage.get_block_info(header.id())?.is_none() {
        return Ok(false);
    }

    dag.has_block_connected(&header)
}
