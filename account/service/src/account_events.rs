use crate::chain_notify_message::ContractEventNotification;
use anyhow::{Error, Result};
use starcoin_account::account_storage::AccountStorage;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_types::account_config::accept_token_payment::AcceptTokenEvent;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
use std::collections::HashSet;

#[derive(Clone)]
pub struct AccountEventService {
    storage: AccountStorage,
}

impl ActorService for AccountEventService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.subscribe::<ContractEventNotification>();
        Ok(())
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) -> Result<()> {
        ctx.unsubscribe::<ContractEventNotification>();
        Ok(())
    }
}

impl ServiceFactory<Self> for AccountEventService {
    fn create(ctx: &mut ServiceContext<Self>) -> Result<Self> {
        Ok(Self {
            storage: ctx.get_shared::<AccountStorage>()?,
        })
    }
}

impl EventHandler<Self, ContractEventNotification> for AccountEventService {
    fn handle_event(&mut self, item: ContractEventNotification, _ctx: &mut ServiceContext<Self>) {
        let addrs = match self.storage.list_addresses() {
            Ok(addresses) => addresses,
            Err(e) => {
                error!("Fail to get account list from storage, err: {}", e);
                return;
            }
        };
        let watched_keys: HashSet<_> = addrs
            .into_iter()
            .map(|addr| EventKey::new(2, addr))
            .collect();

        // short circuit
        if watched_keys.is_empty() {
            return;
        }

        for i in item.0 .1.as_ref() {
            let contract_event = &i.contract_event;
            if watched_keys.contains(&contract_event.event_key()) {
                if let Err(e) = self.handle_contract_event(contract_event) {
                    error!(
                        "fail to save accept token event: {:?}, err: {}",
                        &i.contract_event, e
                    );
                }
            }
        }
    }
}

impl AccountEventService {
    fn handle_contract_event(&self, contract_event: &ContractEvent) -> Result<(), Error> {
        if contract_event.is_typed::<AcceptTokenEvent>() {
            let evt = contract_event.decode_event::<AcceptTokenEvent>()?;
            let addr = contract_event.event_key().get_creator_address();
            let token_code = evt.token_code();
            self.storage.add_accepted_token(addr, token_code.clone())?;
            info!("addr {:#x} accept new token {}", addr, token_code);
        }

        Ok(())
    }
}
