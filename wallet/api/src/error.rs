use anyhow::format_err;
use starcoin_types::account_address::AccountAddress;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum AccountServiceError {
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
    #[error("invalid private key")]
    InvalidPrivateKey,

    // service error
    #[error("account error, {0:?}")]
    AccountError(anyhow::Error),
    #[error("other error: {0:?}")]
    OtherError(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl From<WalletError> for AccountServiceError {
    fn from(err: WalletError) -> Self {
        match err {
            WalletError::AccountNotExist(a) => AccountServiceError::AccountNotExist(a),
            WalletError::AccountAlreadyExist(a) => AccountServiceError::AccountAlreadyExist(a),
            WalletError::AccountLocked(a) => AccountServiceError::AccountLocked(a),
            WalletError::RemoveDefaultAccountError(a) => {
                AccountServiceError::RemoveDefaultAccountError(a)
            }
            WalletError::InvalidPassword(a) => AccountServiceError::InvalidPassword(a),
            WalletError::InvalidPrivateKey => AccountServiceError::InvalidPrivateKey,

            WalletError::TransactionSignError(e) => AccountServiceError::AccountError(e),
            // WalletError::DecryptPrivateKeyError(e) => AccountServiceError::AccountError(e),
            WalletError::AccountPrivateKeyMissing(a) => AccountServiceError::AccountError(
                format_err!("no private key data associate with address {}", a),
            ),
            WalletError::StoreError(e) => AccountServiceError::AccountError(e),
        }
    }
}

/// wallet error is used in wallet impl, to decouple from service.
#[derive(Error, Debug)]
pub enum WalletError {
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
    #[error("invalid private key")]
    InvalidPrivateKey,

    // logic error
    #[error("transaction sign error, {0:?}")]
    TransactionSignError(anyhow::Error),
    // #[error("decrypt private key error, {0:?}")]
    // DecryptPrivateKeyError(anyhow::Error),
    #[error("no private key data associate with address {0}")]
    AccountPrivateKeyMissing(AccountAddress),
    #[error("account vault store error, {0:?}")]
    StoreError(#[from] anyhow::Error),
}
