use starcoin_types::block::BlockNumber;

#[derive(Clone, Debug)]
pub struct BlockDAGConfigMock {
    pub fork_number: BlockNumber,
}

#[derive(Clone, Debug)]
pub enum BlockDAGType {
    BlockDAGFormal,
    BlockDAGTestMock(BlockDAGConfigMock),
}
