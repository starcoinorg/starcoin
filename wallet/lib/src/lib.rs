// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod file_wallet_store;
pub mod keystore_wallet;

#[cfg(test)]

mod test {
    use crate::file_wallet_store::FileWalletStore;
    use starcoin_types::account_address::AccountAddress;
    use wallet_api::{WalletAccount, WalletStore};

    #[test]
    fn file_store_test() {
        let wallet = FileWalletStore::new("./www");
        let account = AccountAddress::random();
        let wallet_account = WalletAccount::new(account, true);
        wallet.save_account(wallet_account.clone()).unwrap();
        assert_eq!(
            wallet_account.clone(),
            wallet.get_account(&account).unwrap().unwrap()
        );
        let key = String::from("mytest");
        let value = "my first test";
        wallet
            .save_to_account(&account, key.clone(), value.as_bytes().to_vec())
            .unwrap();
        let value2 = wallet
            .get_from_account(&account, key.clone().as_str())
            .unwrap()
            .unwrap();
        assert_eq!(value2, value.as_bytes().to_vec());
    }
}
