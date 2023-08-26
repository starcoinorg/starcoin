use super::ghostdag::protocol::{ColoringOutput, GhostdagManager};
use super::reachability::{inquirer, reachability_service::MTReachabilityService};
use super::types::ghostdata::GhostdagData;
use crate::consensusdb::prelude::StoreError;
use crate::consensusdb::{
    prelude::FlexiDagStorage,
    schemadb::{
        DbGhostdagStore, DbHeadersStore, DbReachabilityStore, DbRelationsStore, GhostdagStore,
        HeaderStore, ReachabilityStoreReader, RelationsStore, RelationsStoreReader,
    },
};
use anyhow::{bail, Ok};
use parking_lot::RwLock;
use starcoin_crypto::HashValue as Hash;
use starcoin_types::{
    blockhash::{BlockHashes, KType, ORIGIN},
    header::{ConsensusHeader, Header},
};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

pub type DbGhostdagManager = GhostdagManager<
    DbGhostdagStore,
    DbRelationsStore,
    MTReachabilityService<DbReachabilityStore>,
    DbHeadersStore,
>;
pub struct BlockDAG {
    genesis: Header,
    ghostdag_manager: DbGhostdagManager,
    relations_store: DbRelationsStore,
    reachability_store: DbReachabilityStore,
    ghostdag_store: DbGhostdagStore,
    header_store: DbHeadersStore,
    /// orphan blocks, parent hash -> orphan block
    missing_blocks: HashMap<Hash, HashSet<Header>>,
}

impl BlockDAG {
    pub fn new(genesis: Header, k: KType, db: FlexiDagStorage) -> Self {
        let ghostdag_store = db.ghost_dag_store.clone();
        let header_store = db.header_store.clone();
        let relations_store = db.relations_store.clone();
        let mut reachability_store = db.reachability_store;
        inquirer::init(&mut reachability_store).unwrap();
        let reachability_service =
            MTReachabilityService::new(Arc::new(RwLock::new(reachability_store.clone())));
        let ghostdag_manager = DbGhostdagManager::new(
            genesis.hash(),
            k,
            ghostdag_store.clone(),
            relations_store.clone(),
            header_store.clone(),
            reachability_service,
        );

        let mut dag = Self {
            genesis,
            ghostdag_manager,
            relations_store,
            reachability_store,
            ghostdag_store,
            header_store,
            missing_blocks: HashMap::new(),
        };
        dag.init_with_genesis();
        dag
    }

    pub fn clear_missing_block(&mut self) {
        self.missing_blocks.clear();
    }

    pub fn init_with_genesis(&mut self) -> anyhow::Result<()> {
        let exits = self.relations_store.has(Hash::new(ORIGIN))?;
        if exits {
            return Ok(());
        }
        self.relations_store
            .insert(Hash::new(ORIGIN), BlockHashes::new(vec![]))
            .unwrap();
        let _ = self.commit_header(&self.genesis.clone())?;
        Ok(())
    }

    pub fn commit_header_inner(
        &mut self,
        ghostdag_data: &GhostdagData,
        header: &Header,
    ) -> anyhow::Result<()> {
        // Generate ghostdag data
        let parents_hash = header.parents_hash();
        // Store ghostdata
        self.ghostdag_store
            .insert(header.hash(), Arc::new(ghostdag_data.clone()))?;

        // Update reachability store
        let mut reachability_store = self.reachability_store.clone();
        let mut merge_set = ghostdag_data
            .unordered_mergeset_without_selected_parent()
            .filter(|hash| self.reachability_store.has(*hash).unwrap());

        inquirer::add_block(
            &mut reachability_store,
            header.hash(),
            ghostdag_data.selected_parent,
            &mut merge_set,
        )?;

        // store relations
        self.relations_store
            .insert(header.hash(), BlockHashes::new(parents_hash.to_vec()))?;
        // Store header store
        self.header_store
            .insert(header.hash(), Arc::new(header.to_owned()), 0)?;

        Ok(())
    }

    pub fn commit_header(&mut self, header: &Header) -> anyhow::Result<ColoringOutput> {
        let ghostdag_data = if header.hash() != self.genesis.hash() {
            self.ghostdag_manager.ghostdag(header.parents_hash())
        } else {
            self.ghostdag_manager.genesis_ghostdag_data()
        };

        match self.commit_header_inner(&ghostdag_data, header) {
            anyhow::Result::Ok(()) => (),
            Err(error) => {
                let error_result = error.downcast::<StoreError>()?;
                match error_result {
                    StoreError::KeyAlreadyExists(_) => (), // if the header existed already, we check its color
                    _ => {
                        return anyhow::Result::Err(error_result.into());
                    }
                }
            }
        }
        Ok(self
            .ghostdag_manager
            .check_blue_candidate(&ghostdag_data, header.hash()))
    }
    fn is_in_dag(&self, _hash: Hash) -> anyhow::Result<bool> {
        return Ok(true);
    }
    pub fn verify_header(&self, _header: &Header) -> anyhow::Result<()> {
        //TODO: implemented it
        Ok(())
    }

    pub fn connect_block(&mut self, header: &Header) -> anyhow::Result<()> {
        let _ = self.verify_header(header)?;
        let is_orphan_block = self.update_orphans(header)?;
        if is_orphan_block {
            return Ok(());
        }
        self.commit_header(header);
        self.check_missing_block(header)?;
        Ok(())
    }

    pub fn check_missing_block(&mut self, header: &Header) -> anyhow::Result<()> {
        if let Some(orphans) = self.missing_blocks.remove(&header.hash()) {
            for orphan in orphans.iter() {
                let is_orphan = self.is_orphan(&orphan)?;
                if !is_orphan {
                    self.commit_header(header);
                }
            }
        }
        Ok(())
    }
    fn is_orphan(&self, header: &Header) -> anyhow::Result<bool> {
        for parent in header.parents_hash() {
            if !self.is_in_dag(parent.to_owned())? {
                return Ok(false);
            }
        }
        return Ok(true);
    }

    fn update_orphans(&mut self, block_header: &Header) -> anyhow::Result<bool> {
        let mut is_orphan = false;
        for parent in block_header.parents_hash() {
            if self.is_in_dag(parent.to_owned())? {
                continue;
            }
            if !self
                .missing_blocks
                .entry(parent.to_owned())
                .or_insert_with(HashSet::new)
                .insert(block_header.to_owned())
            {
                return Err(anyhow::anyhow!("Block already processed as a orphan"));
            }
            is_orphan = true;
        }
        Ok(is_orphan)
    }

    pub fn get_block_header(&self, hash: Hash) -> anyhow::Result<Header> {
        match self.header_store.get_header(hash) {
            anyhow::Result::Ok(header) => anyhow::Result::Ok(header),
            Err(error) => {
                println!("failed to get header by hash: {}", error.to_string());
                bail!("failed to get header by hash: {}", error.to_string());
            }
        }
    }

    pub fn get_parents(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.relations_store.get_parents(hash) {
            anyhow::Result::Ok(parents) => anyhow::Result::Ok((*parents).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }

    pub fn get_children(&self, hash: Hash) -> anyhow::Result<Vec<Hash>> {
        match self.relations_store.get_children(hash) {
            anyhow::Result::Ok(children) => anyhow::Result::Ok((*children).clone()),
            Err(error) => {
                println!("failed to get parents by hash: {}", error.to_string());
                bail!("failed to get parents by hash: {}", error.to_string());
            }
        }
    }

    // for testing
    pub fn push_parent_children(&mut self, child: Hash, parents: Arc<Vec<Hash>>) -> Result<(), StoreError> {
        self.relations_store.insert(child, parents)
    }

    pub fn get_genesis_hash(&self) -> Hash {
        self.genesis.hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensusdb::prelude::{FlexiDagStorage, FlexiDagStorageConfig};
    use starcoin_types::block::BlockHeader;
    use std::{env, fs};
    #[test]
    fn base_test() {
        let genesis = Header::new(BlockHeader::random(), vec![Hash::new(ORIGIN)]);
        let genesis_hash = genesis.hash();
        let k = 16;
        let db_path = env::temp_dir().join("smolstc");
        println!("db path:{}", db_path.to_string_lossy());
        if db_path
            .as_path()
            .try_exists()
            .unwrap_or_else(|_| panic!("Failed to check {db_path:?}"))
        {
            fs::remove_dir_all(db_path.as_path()).expect("Failed to delete temporary directory");
        }
        let config = FlexiDagStorageConfig::create_with_params(1, 0, 1024);
        let db = FlexiDagStorage::create_from_path(db_path, config)
            .expect("Failed to create flexidag storage");
        let mut dag = BlockDAG::new(genesis, k, db);

        let block = Header::new(BlockHeader::random(), vec![genesis_hash]);
        dag.commit_header(&block);
    }
}
