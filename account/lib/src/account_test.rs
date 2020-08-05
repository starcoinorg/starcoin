use crate::account_storage::AccountStorage;
use crate::Account;
use crate::AccountManager;
use actix::clock::Duration;
use anyhow::Result;
use starcoin_account_api::error::AccountError;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::chain_config::ChainId;
use starcoin_types::transaction::{RawUserTransaction, Script, TransactionPayload};

#[test]
pub fn test_import_account() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let storage = AccountStorage::create_from_path(tempdir.path())?;
    let manager = AccountManager::new(storage)?;

    // should success
    let wallet = manager.create_account("hello")?;
    let kp = super::account_manager::gen_keypair();
    let result =
        manager.import_account(*wallet.address(), kp.private_key.to_bytes().to_vec(), "abc");
    assert!(result.is_err());

    assert!(
        matches!(result.err().unwrap(), AccountError::AccountAlreadyExist(addr) if addr == *wallet.address())
    );

    let normal_address = AccountAddress::random();
    let _account =
        manager.import_account(normal_address, kp.private_key.to_bytes().to_vec(), "abc")?;
    assert_eq!(manager.list_account_infos()?.len(), 2);
    Ok(())
}

#[test]
pub fn test_wallet() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let storage = AccountStorage::create_from_path(tempdir.path())?;
    let manager = AccountManager::new(storage.clone())?;

    // should success
    let wallet = manager.create_account("hello")?;

    let wallet_address = wallet.address();

    // test reload
    let loaded_wallet = Account::load(*wallet_address, "hello", storage)?;
    assert!(loaded_wallet.is_some());
    let reloaded_wallet = loaded_wallet.unwrap();
    assert_eq!(
        reloaded_wallet.private_key().to_bytes(),
        wallet.private_key().to_bytes()
    );

    // test default wallet
    let default_wallet_info = manager.default_account_info()?;
    assert!(default_wallet_info.is_some());
    let default_wallet_info = default_wallet_info.unwrap();
    assert_eq!(&default_wallet_info.address, wallet.address());

    // test wallet destroy

    wallet.destroy()?;

    Ok(())
}

#[test]
pub fn test_wallet_unlock() -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    let storage = AccountStorage::create_from_path(tempdir.path())?;
    let manager = AccountManager::new(storage)?;

    let wallet = manager.create_account("hello")?;

    let unlock_result = manager.unlock_account(*wallet.address(), "hell0", Duration::from_secs(1));
    assert!(unlock_result.is_err());
    manager.unlock_account(*wallet.address(), "hello", Duration::from_secs(1))?;
    let fake_txn = RawUserTransaction::new(
        *wallet.address(),
        1,
        TransactionPayload::Script(Script::new(vec![], vec![], vec![])),
        1000,
        1,
        100000,
        ChainId::new(1),
    );
    let _signed = manager.sign_txn(*wallet.address(), fake_txn)?;
    Ok(())
}
