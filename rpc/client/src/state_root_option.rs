use std::str::FromStr;
use starcoin_crypto::HashValue;
use starcoin_types::block::BlockNumber;

#[derive(Debug, Default, Clone, Copy)]
pub enum StateRootOption {
    #[default]
    Latest,
    BlockHash(HashValue),
    BlockNumber(BlockNumber),
}

impl FromStr for StateRootOption {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        if let Ok(number) = s.parse::<u64>() {
            Ok(StateRootOption::BlockNumber(number))
        } else {
            Ok(StateRootOption::BlockHash(HashValue::from_str(s)?))
        }
    }
}