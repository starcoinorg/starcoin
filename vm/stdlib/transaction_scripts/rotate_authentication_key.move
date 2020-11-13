script {
use 0x1::Account;

/// # Summary
/// Rotates the transaction sender's authentication key to the supplied new authentication key. May
/// be sent by any account.
///
/// # Technical Description
/// Rotate the `account`'s `Account::Account` `authentication_key` field to `new_key`.
/// `new_key` must be a valid ed25519 public key, and `account` must not have previously delegated
/// its `Account::KeyRotationCapability`.
///
/// # Parameters
/// | Name      | Type         | Description                                                 |
/// | ------    | ------       | -------------                                               |
/// | `account` | `&signer`    | Signer reference of the sending account of the transaction. |
/// | `new_key` | `vector<u8>` | New ed25519 public key to be used for `account`.            |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                               | Description                                                                              |
/// | ----------------           | --------------                                             | -------------                                                                            |
/// | `Errors::INVALID_STATE`    | `Account::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `Account::KeyRotationCapability`.     |
/// | `Errors::INVALID_ARGUMENT` | `Account::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                         |
///

fun rotate_authentication_key(account: &signer, new_key: vector<u8>) {
    let key_rotation_capability = Account::extract_key_rotation_capability(account);
    Account::rotate_authentication_key(&key_rotation_capability, new_key);
    Account::restore_key_rotation_capability(key_rotation_capability);
}

spec fun rotate_authentication_key {
    pragma verify = false;
}
}
