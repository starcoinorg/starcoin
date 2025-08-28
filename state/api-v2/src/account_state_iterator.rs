
pub struct AccountStateSetIterator {
    store: Arc<dyn StateNodeStore>,
    jmt_into_iter: JellyfishMerkleIntoIterator<AccountAddress, StorageTreeReader<AccountAddress>>,
}

impl AccountStateSetIterator {
    pub fn new(
        store: Arc<dyn StateNodeStore>,
        jmt_into_iter: JellyfishMerkleIntoIterator<
            AccountAddress,
            StorageTreeReader<AccountAddress>,
        >,
    ) -> Self {
        Self {
            store,
            jmt_into_iter,
        }
    }
}

impl Iterator for AccountStateSetIterator {
    type Item = (AccountAddress, AccountStateSet);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.jmt_into_iter.next();
        if let Some(item) = item {
            let item = item.ok()?;
            let account_address = item.0;
            let account_state = Vec::from(item.1);
            let account_state: AccountState = account_state.as_slice().try_into().ok()?;
            let mut state_sets = vec![];
            for (idx, storage_root) in account_state.storage_roots().iter().enumerate() {
                let state_set = match storage_root {
                    Some(storage_root) => {
                        let data_type = DataType::from_index(idx as u8).ok()?;
                        // TODO move support map resource have many elem, consider use iter
                        match data_type {
                            DataType::CODE => Some(
                                StateTree::<ModuleName>::new(
                                    self.store.clone(),
                                    Some(*storage_root),
                                )
                                .dump()
                                .ok()?,
                            ),
                            DataType::RESOURCE => Some(
                                StateTree::<StructTag>::new(
                                    self.store.clone(),
                                    Some(*storage_root),
                                )
                                .dump()
                                .ok()?,
                            ),
                            DataType::RESOURCEGROUP => Some(
                                StateTree::<StructTag>::new(
                                    self.store.clone(),
                                    Some(*storage_root),
                                )
                                .dump()
                                .ok()?,
                            ),
                        }
                    }
                    None => None,
                };
                state_sets.push(state_set);
            }
            return Some((account_address, AccountStateSet::new(state_sets)));
        }
        None
    }
}
