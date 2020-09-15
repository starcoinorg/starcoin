use crate::account_storage::AccountStorage;
use crate::Account;
use crate::AccountManager;
use anyhow::Result;
use starcoin_account_api::error::AccountError;
use starcoin_crypto::SigningKey;
use starcoin_types::access_path::AccessPath;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::genesis_config::ChainId;
use starcoin_types::identifier::{IdentStr, Identifier};
use starcoin_types::language_storage::{StructTag, CORE_CODE_ADDRESS};
use starcoin_types::transaction::{
    RawUserTransaction, Script, SignedUserTransaction, TransactionPayload,
};
use std::time::Duration;

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

#[test]
pub fn test_libra_wallet() -> Result<()> {
    use core::convert::{From, TryFrom};
    use starcoin_canonical_serialization::SCSCodec;
    use starcoin_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};
    use starcoin_crypto::{
        hash::{CryptoHash, PlainCryptoHash},
        HashValue,
    };
    use starcoin_types::transaction::authenticator::AuthenticationKey;

    let bytes = hex::decode("2c78c6fd8829de80451cda02310250b27307360ddc972d614fa0c8462ae41b3e")?;
    let private_key = Ed25519PrivateKey::try_from(&bytes[..])?;
    let public_key = Ed25519PublicKey::from(&private_key);

    let message = [1, 2, 3, 4];
    let result = private_key.sign_arbitrary_message(&message);

    let address = starcoin_types::account_address::from_public_key(&public_key);
    let hash_value = HashValue::sha3_256_of(&public_key.to_bytes());
    let key = AuthenticationKey::new(*HashValue::sha3_256_of(&public_key.to_bytes()).as_ref());

    let sign_bytes = vec![
        227, 94, 250, 168, 43, 200, 137, 74, 61, 254, 197, 71, 245, 135, 201, 43, 222, 190, 56,
        235, 247, 254, 56, 247, 108, 36, 250, 192, 143, 236, 101, 153, 61, 241, 129, 47, 38, 146,
        213, 9, 79, 56, 90, 210, 179, 53, 73, 208, 248, 231, 22, 9, 55, 177, 154, 212, 248, 2, 66,
        221, 67, 101, 152, 6,
    ];
    let _sign = Ed25519Signature::try_from(&sign_bytes[..])?;

    let raw_txn_bytes = vec![
        125, 67, 213, 38, 157, 219, 137, 205, 183, 247, 184, 18, 104, 155, 241, 53, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 161, 1, 161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 17, 4, 19, 4, 5, 23,
        24, 7, 47, 42, 8, 89, 16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 2, 3, 0, 0, 3, 4, 1, 1, 1, 0, 6, 2,
        6, 2, 5, 10, 2, 0, 1, 5, 1, 1, 3, 6, 12, 5, 4, 4, 6, 12, 5, 10, 2, 4, 1, 9, 0, 7, 65, 99,
        99, 111, 117, 110, 116, 14, 99, 114, 101, 97, 116, 101, 95, 97, 99, 99, 111, 117, 110, 116,
        9, 101, 120, 105, 115, 116, 115, 95, 97, 116, 8, 112, 97, 121, 95, 102, 114, 111, 109, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 5, 1, 13, 10, 1, 17, 1, 32, 3, 5, 5, 8,
        10, 1, 10, 2, 56, 0, 11, 0, 10, 1, 10, 3, 56, 1, 2, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 13, 48, 120, 49, 58, 58, 83, 84, 67, 58, 58, 83, 84, 67, 13, 48, 120, 49,
        58, 58, 83, 84, 67, 58, 58, 83, 84, 67, 0, 3, 3, 112, 48, 56, 223, 253, 244, 219, 3, 173,
        17, 252, 117, 207, 222, 197, 149, 4, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0,
        0, 32, 78, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 13, 48, 120, 49, 58, 58, 83, 84, 67,
        58, 58, 83, 84, 67, 159, 150, 87, 95, 0, 0, 0, 0, 254,
    ];
    let raw_txn = RawUserTransaction::decode(&raw_txn_bytes)?;

    println!("raw txn hash is {:?}", raw_txn.hash());

    let sign_checked_txn = raw_txn.sign(&private_key, public_key.clone())?;
    println!(
        "sign bytes is {:?}",
        sign_checked_txn
            .into_inner()
            .authenticator()
            .signature_bytes(),
    );
    //println!("verify result is {:?}", sign.verify(&raw_txn, &public_key)?);
    println!("public key is {:?}", public_key.to_bytes().as_ref());
    println!("hash value is {:?}", hash_value.as_ref());
    println!("key is {:?}", key.derived_address());
    println!("address is {:?},result is {:?}", address, result);

    println!(
        "core code address is {}",
        AccountAddress::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
        ])
    );

    let path = StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::from(IdentStr::new("Account")?),
        name: Identifier::from(IdentStr::new("Account")?),
        type_params: vec![],
    };
    println!("path hash is {:?}", path.hash());
    let access_path = AccessPath::new(address, path.access_vector());
    println!("access path is {:?}", access_path);
    let stxn_bytes = vec![
        125, 67, 213, 38, 157, 219, 137, 205, 183, 247, 184, 18, 104, 155, 241, 53, 7, 0, 0, 0, 0,
        0, 0, 0, 0, 161, 1, 161, 28, 235, 11, 1, 0, 0, 0, 6, 1, 0, 2, 3, 2, 17, 4, 19, 4, 5, 23,
        24, 7, 47, 42, 8, 89, 16, 0, 0, 0, 1, 0, 1, 1, 1, 0, 2, 2, 3, 0, 0, 3, 4, 1, 1, 1, 0, 6, 2,
        6, 2, 5, 10, 2, 0, 1, 5, 1, 1, 3, 6, 12, 5, 4, 4, 6, 12, 5, 10, 2, 4, 1, 9, 0, 7, 65, 99,
        99, 111, 117, 110, 116, 14, 99, 114, 101, 97, 116, 101, 95, 97, 99, 99, 111, 117, 110, 116,
        9, 101, 120, 105, 115, 116, 115, 95, 97, 116, 8, 112, 97, 121, 95, 102, 114, 111, 109, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 5, 1, 13, 10, 1, 17, 1, 32, 3, 5, 5, 8,
        10, 1, 10, 2, 56, 0, 11, 0, 10, 1, 10, 3, 56, 1, 2, 1, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 1, 3, 83, 84, 67, 3, 83, 84, 67, 0, 3, 3, 170, 98, 21, 247, 38, 8, 228, 209,
        97, 153, 20, 39, 180, 155, 110, 103, 4, 16, 112, 48, 56, 223, 253, 244, 219, 3, 173, 17,
        252, 117, 207, 222, 197, 149, 2, 200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 32, 78,
        0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 13, 48, 120, 49, 58, 58, 83, 84, 67, 58, 58, 83,
        84, 67, 5, 226, 96, 95, 0, 0, 0, 0, 254, 0, 32, 130, 108, 242, 253, 81, 233, 250, 135, 55,
        141, 56, 92, 52, 117, 153, 246, 9, 69, 123, 70, 107, 203, 151, 216, 30, 34, 96, 130, 71,
        68, 12, 143, 64, 6, 102, 250, 227, 98, 221, 129, 136, 197, 243, 79, 206, 201, 57, 0, 57,
        163, 216, 146, 36, 227, 205, 214, 21, 85, 200, 71, 42, 155, 16, 207, 204, 134, 183, 87, 89,
        253, 28, 178, 254, 244, 28, 94, 129, 152, 49, 111, 118, 238, 236, 36, 49, 239, 179, 197,
        211, 150, 199, 7, 37, 161, 6, 202, 7,
    ];
    let stxn = SignedUserTransaction::decode(&stxn_bytes)?;
    println!("txn hash is {:?}", stxn.crypto_hash());
    Ok(())
}
