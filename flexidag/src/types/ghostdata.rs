use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;
use starcoin_types::blockhash::{BlockHashMap, BlockHashes, BlueWorkType, HashKTypeMap, KType};

#[derive(Clone, Serialize, Deserialize, Default, Debug, JsonSchema)]
pub struct GhostdagData {
    pub blue_score: u64,
    #[schemars(with = "String")]
    pub blue_work: BlueWorkType,
    pub selected_parent: Hash,
    pub mergeset_blues: BlockHashes,
    pub mergeset_reds: BlockHashes,
    pub blues_anticone_sizes: HashKTypeMap,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Copy)]
pub struct CompactGhostdagData {
    pub blue_score: u64,
    pub blue_work: BlueWorkType,
    pub selected_parent: Hash,
}

impl GhostdagData {
    pub fn new(
        blue_score: u64,
        blue_work: BlueWorkType,
        selected_parent: Hash,
        mergeset_blues: BlockHashes,
        mergeset_reds: BlockHashes,
        blues_anticone_sizes: HashKTypeMap,
    ) -> Self {
        Self {
            blue_score,
            blue_work,
            selected_parent,
            mergeset_blues,
            mergeset_reds,
            blues_anticone_sizes,
        }
    }

    pub fn new_with_selected_parent(selected_parent: Hash, k: KType) -> Self {
        let mut mergeset_blues: Vec<Hash> = Vec::with_capacity(k.checked_add(1).unwrap() as usize);
        let mut blues_anticone_sizes: BlockHashMap<KType> = BlockHashMap::with_capacity(k as usize);
        mergeset_blues.push(selected_parent);
        blues_anticone_sizes.insert(selected_parent, 0);

        Self {
            blue_score: Default::default(),
            blue_work: Default::default(),
            selected_parent,
            mergeset_blues: BlockHashes::new(mergeset_blues),
            mergeset_reds: Default::default(),
            blues_anticone_sizes: HashKTypeMap::new(blues_anticone_sizes),
        }
    }

    pub fn mergeset_size(&self) -> usize {
        self.mergeset_blues
            .len()
            .checked_add(self.mergeset_reds.len())
            .unwrap()
    }

    /// Returns an iterator to the mergeset with no specified order (excluding the selected parent)
    pub fn unordered_mergeset_without_selected_parent(&self) -> impl Iterator<Item = Hash> + '_ {
        self.mergeset_blues
            .iter()
            .skip(1) // Skip the selected parent
            .cloned()
            .chain(self.mergeset_reds.iter().cloned())
    }

    /// Returns an iterator to the mergeset with no specified order (including the selected parent)
    pub fn unordered_mergeset(&self) -> impl Iterator<Item = Hash> + '_ {
        self.mergeset_blues
            .iter()
            .cloned()
            .chain(self.mergeset_reds.iter().cloned())
    }

    pub fn to_compact(&self) -> CompactGhostdagData {
        CompactGhostdagData {
            blue_score: self.blue_score,
            blue_work: self.blue_work,
            selected_parent: self.selected_parent,
        }
    }

    pub fn add_blue(
        &mut self,
        block: Hash,
        blue_anticone_size: KType,
        block_blues_anticone_sizes: &BlockHashMap<KType>,
    ) {
        // Add the new blue block to mergeset blues
        BlockHashes::make_mut(&mut self.mergeset_blues).push(block);

        // Get a mut ref to internal anticone size map
        let blues_anticone_sizes = HashKTypeMap::make_mut(&mut self.blues_anticone_sizes);

        // Insert the new blue block with its blue anticone size to the map
        blues_anticone_sizes.insert(block, blue_anticone_size);

        // Insert/update map entries for blocks affected by this insertion
        for (blue, size) in block_blues_anticone_sizes {
            blues_anticone_sizes.insert(*blue, size.checked_add(1).unwrap());
        }
    }

    pub fn add_red(&mut self, block: Hash) {
        // Add the new red block to mergeset reds
        BlockHashes::make_mut(&mut self.mergeset_reds).push(block);
    }

    pub fn finalize_score_and_work(&mut self, blue_score: u64, blue_work: BlueWorkType) {
        self.blue_score = blue_score;
        self.blue_work = blue_work;
    }
}
