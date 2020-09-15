use anyhow::{Error, Result};
use bstr::ByteSlice;
use starcoin_account_lib::account_storage::AccountStorage;
use starcoin_canonical_serialization::SCSCodec;
use starcoin_chain_notify::message::ContractEventNotification;
use starcoin_logger::prelude::*;
use starcoin_service_registry::{ActorService, EventHandler, ServiceContext, ServiceFactory};
use starcoin_types::account_address::AccountAddress;
use starcoin_types::account_config::accept_token_payment::AcceptTokenEvent;
use starcoin_types::account_config::token_code::TokenCode;
use starcoin_types::contract_event::ContractEvent;
use starcoin_types::event::EventKey;
use std::collections::HashSet;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct AccountEventService {
    storage: AccountStorage,
}

impl ActorService for AccountEventService {
    fn started(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.subscribe::<ContractEventNotification>();
    }

    fn stopped(&mut self, ctx: &mut ServiceContext<Self>) {
        ctx.unsubscribe::<ContractEventNotification>();
    }
}

impl ServiceFactory<AccountEventService> for AccountEventService {
    fn create(ctx: &mut ServiceContext<AccountEventService>) -> Result<AccountEventService> {
        Ok(Self {
            storage: ctx.get_shared::<AccountStorage>()?,
        })
    }
}

impl EventHandler<Self, ContractEventNotification> for AccountEventService {
    fn handle_event(
        &mut self,
        item: ContractEventNotification,
        _ctx: &mut ServiceContext<AccountEventService>,
    ) {
        let addrs = match self.storage.list_addresses() {
            Ok(addresses) => addresses,
            Err(e) => {
                error!("Fail to get account list from storage, err: {}", e);
                return;
            }
        };
        let watched_keys: HashSet<_> = addrs
            .into_iter()
            .map(|addr| EventKey::new_from_address(&addr, 2))
            .collect();

        // short circuit
        if watched_keys.is_empty() {
            return;
        }

        for i in item.0.as_ref() {
            if watched_keys.contains(i.contract_event.key()) {
                if let Err(e) = self.handle_contract_event(&i.contract_event) {
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
    fn handle_contract_event(&self, event: &ContractEvent) -> Result<(), Error> {
        let evt = AcceptTokenEvent::try_from(event)?;
        let addr = event.key().get_creator_address();
        let accepted_token = evt.currency_code();
        let parts: Vec<_> = accepted_token.split_str("::").collect();
        let token_addr = parts[0];
        // TODO: should move emit the fields directly?
        let token_code = AccountAddress::decode(token_addr).map(|addr| {
            TokenCode::new(
                addr,
                String::from_utf8_lossy(parts[1]).to_string(),
                String::from_utf8_lossy(parts[2]).to_string(),
            )
        })?;
        self.storage.add_accepted_token(addr, token_code.clone())?;
        info!("addr {:#x} accept new token {}", addr, token_code);
        Ok(())
    }
}
