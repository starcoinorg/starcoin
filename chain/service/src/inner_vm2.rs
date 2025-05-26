use crate::chain_service::ChainReaderServiceInner;
use anyhow::Error;
use starcoin_chain_api::ReadableChainService;
use starcoin_crypto::HashValue;
use starcoin_types::contract_event::StcContractEventInfo;
use starcoin_vm2_types::contract_event::ContractEvent as ContractEvent2;

impl ChainReaderServiceInner {
    pub fn get_events_by_txn_info_hash2(
        &self,
        txn_info_id: HashValue,
    ) -> anyhow::Result<Option<Vec<ContractEvent2>>, Error> {
        let (_, storage2) = self.get_storages()?;
        storage2.get_contract_events(txn_info_id)
    }
    pub fn get_event_by_txn_hash2(
        &self,
        txn_hash: HashValue,
    ) -> anyhow::Result<Vec<StcContractEventInfo>> {
        let txn_info = self
            .get_transaction_info(txn_hash)?
            .ok_or_else(|| anyhow::anyhow!("cannot find txn info of txn {}", txn_hash))?;

        let events = self
            .get_events_by_txn_info_hash2(txn_info.id())?
            .unwrap_or_default();

        let event_infos = if events.is_empty() {
            vec![]
        } else {
            events
                .into_iter()
                .enumerate()
                .map(|(idx, evt)| StcContractEventInfo {
                    block_hash: txn_info.block_id(),
                    block_number: txn_info.block_number,
                    transaction_hash: txn_hash,
                    transaction_index: txn_info.transaction_index,
                    transaction_global_index: txn_info.transaction_global_index,
                    event_index: idx as u32,
                    event: evt.into(),
                })
                .collect()
        };
        Ok(event_infos)
    }
}
