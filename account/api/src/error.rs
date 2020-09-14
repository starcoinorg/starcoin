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
    OtherError(anyhow::Error),
}

impl From<AccountError> for AccountServiceError {
    fn from(err: AccountError) -> Self {
        match err {
            AccountError::AccountNotExist(a) => AccountServiceError::AccountNotExist(a),
            AccountError::AccountAlreadyExist(a) => AccountServiceError::AccountAlreadyExist(a),
            AccountError::AccountLocked(a) => AccountServiceError::AccountLocked(a),
            AccountError::RemoveDefaultAccountError(a) => {
                AccountServiceError::RemoveDefaultAccountError(a)
            }
            AccountError::InvalidPassword(a) => AccountServiceError::InvalidPassword(a),
            AccountError::InvalidPrivateKey => AccountServiceError::InvalidPrivateKey,

            AccountError::TransactionSignError(e) => AccountServiceError::AccountError(e),
            // WalletError::DecryptPrivateKeyError(e) => AccountServiceError::AccountError(e),
            AccountError::AccountPrivateKeyMissing(a) => AccountServiceError::AccountError(
                format_err!("no private key data associate with address {}", a),
            ),
            AccountError::StoreError(e) => AccountServiceError::AccountError(e),
        }
    }
}

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
