// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

pub mod file_wallet_store;
pub mod keystore_wallet;

#[cfg(test)]
mod test {
    use crate::file_wallet_store::FileWalletStore;
    // use starcoin_types::account_address::AccountAddress;
    use std::collections::HashMap;
    use wallet_api::{WalletAccount, WalletStore};

    #[test]
    fn test_file_store() {
        let tmpdir = tempfile::tempdir().unwrap();
        let wallet = FileWalletStore::new(tmpdir.path());
        let wallet_account = WalletAccount::random();
        let account = wallet_account.address().clone();
        wallet.save_account(wallet_account.clone()).unwrap();
        assert_eq!(
            wallet_account.clone().address,
            wallet.get_account(&account).unwrap().unwrap().address
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

    #[test]
    fn test_get_accounts() {
        let tmpdir = tempfile::tempdir().unwrap();
        let wallet = FileWalletStore::new(tmpdir.path());
        let mut account_map = HashMap::new();
        for _i in 0..10 {
            let wallet_account = WalletAccount::random();
            let account = wallet_account.address().clone();
            wallet.save_account(wallet_account.clone()).unwrap();
            account_map.insert(account, wallet_account);
        }
        let accounts = wallet.get_accounts().unwrap();
        assert_eq!(accounts.len(), 10);
        for account in accounts {
            let wac = account_map.get(&account.address);
            assert!(wac.is_some());
            wallet.remove_account(&account.address).unwrap();
        }
    }

    #[test]
    fn test_remove_account() {
        let tmpdir = tempfile::tempdir().unwrap();
        let wallet = FileWalletStore::new(tmpdir.path());
        let wallet_account = WalletAccount::random();
        let account = wallet_account.address().clone();
        wallet.save_account(wallet_account.clone()).unwrap();
        wallet
            .save_to_account(&account, String::from("key1"), "value1".as_bytes().to_vec())
            .unwrap();
        wallet.remove_account(&account).unwrap();
        let test_account = wallet.get_account(&account);
        assert!(test_account.is_err());
    }
}
