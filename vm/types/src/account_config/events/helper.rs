use crate::account_config::token_code::TokenCode;
use crate::account_config::{BlockRewardEvent, BurnEvent, DepositEvent, MintEvent, WithdrawEvent};
use crate::contract_event::ContractEvent;
use crate::event::EventKey;

pub enum BalanceEvent {
    Mint((EventKey, MintEvent)),
    Burn((EventKey, BurnEvent)),
    Deposit((EventKey, DepositEvent)),
    Withdraw((EventKey, WithdrawEvent)),
    BlockReward((EventKey, BlockRewardEvent)),
}

impl BalanceEvent {
    pub fn key(&self) -> &EventKey {
        match self {
            BalanceEvent::Mint((key, _)) => key,
            BalanceEvent::Burn((key, _)) => key,
            BalanceEvent::Deposit((key, _)) => key,
            BalanceEvent::Withdraw((key, _)) => key,
            BalanceEvent::BlockReward((key, _)) => key,
        }
    }

    pub fn amount(&self) -> u128 {
        match self {
            BalanceEvent::Mint((_, event)) => event.amount(),
            BalanceEvent::Burn((_, event)) => event.amount(),
            BalanceEvent::Deposit((_, event)) => event.amount(),
            BalanceEvent::Withdraw((_, event)) => event.amount(),
            BalanceEvent::BlockReward((_, event)) => event.amount(),
        }
    }

    pub fn token_code(&self) -> &TokenCode {
        match self {
            BalanceEvent::Mint((_, event)) => event.token_code(),
            BalanceEvent::Burn((_, event)) => event.token_code(),
            BalanceEvent::Deposit((_, event)) => event.token_code(),
            BalanceEvent::Withdraw((_, event)) => event.token_code(),
            BalanceEvent::BlockReward((_, event)) => event.token_code(),
        }
    }
}

impl std::fmt::Display for BalanceEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BalanceEvent::Mint((key, event)) => {
                write!(f, "MintEvent: key: {}, event: {}", key, event)
            }
            BalanceEvent::Burn((key, event)) => {
                write!(f, "BurnEvent: key: {}, event: {}", key, event)
            }
            BalanceEvent::Deposit((key, event)) => {
                write!(f, "DepositEvent: key: {}, event: {}", key, event)
            }
            BalanceEvent::Withdraw((key, event)) => {
                write!(f, "WithdrawEvent: key: {}, event: {}", key, event)
            }
            BalanceEvent::BlockReward((key, event)) => {
                write!(f, "BlockRewardEvent: key: {}, event: {}", key, event)
            }
        }
    }
}

impl TryFrom<&ContractEvent> for BalanceEvent {
    type Error = ();

    fn try_from(event: &ContractEvent) -> Result<BalanceEvent, Self::Error> {
        balance_event(&event).ok_or(())
    }
}

fn balance_event(event: &ContractEvent) -> Option<BalanceEvent> {
    if event.is::<DepositEvent>() {
        DepositEvent::try_from_bytes(event.event_data())
            .ok()
            .map(|e| BalanceEvent::Deposit((event.key().clone(), e)))
    } else if event.is::<WithdrawEvent>() {
        WithdrawEvent::try_from_bytes(event.event_data())
            .ok()
            .map(|e| BalanceEvent::Withdraw((event.key().clone(), e)))
    } else if event.is::<MintEvent>() {
        MintEvent::try_from_bytes(event.event_data())
            .ok()
            .map(|e| BalanceEvent::Mint((event.key().clone(), e)))
    } else if event.is::<BlockRewardEvent>() {
        BlockRewardEvent::try_from_bytes(event.event_data())
            .ok()
            .map(|e| BalanceEvent::BlockReward((event.key().clone(), e)))
    } else {
        None
    }
}
