use starcoin_types::account_address::AccountAddress;
use thiserror::Error;

/// wallet error is used in wallet impl, to decouple from service.
#[derive(Error, Debug)]
pub enum AccountError {
    // param error
    #[error("account with address {0} not exists")]
    AccountNotExist(AccountAddress),
    #[error("account with address {0} already exists")]
    AccountAlreadyExist(AccountAddress),
    #[error("account {0} is locked")]
    AccountLocked(AccountAddress),

    #[error("cannot remove default account {0}")]
    RemoveDefaultAccountError(AccountAddress),

    #[error("invalid password, cannot decrypt account {0}")]
    InvalidPassword(AccountAddress),

    #[error("invalid private key: {0:?}")]
    InvalidPrivateKey(starcoin_crypto::CryptoMaterialError),

    #[error("invalid public key: {0:?}")]
    InvalidPublicKey(starcoin_crypto::CryptoMaterialError),
    // logic error
    #[error("transaction sign error, {0:?}")]
    TransactionSignError(anyhow::Error),
    #[error("message sign error, {0:?}")]
    MessageSignError(anyhow::Error),
    // #[error("decrypt private key error, {0:?}")]
    // DecryptPrivateKeyError(anyhow::Error),
    #[error("no private key data associate with address {0}")]
    AccountPrivateKeyMissing(AccountAddress),
    #[error("account vault store error, {0:?}")]
    StoreError(#[from] anyhow::Error),
}
