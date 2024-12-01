/// Provides a common place for exporting `create_signer` across the Starcoin Framework.
///
/// To use create_signer, add the module below, such that:
/// `friend starcoin_framework::friend_wants_create_signer`
/// where `friend_wants_create_signer` is the module that needs `create_signer`.
///
/// Note, that this is only available within the Starcoin Framework.
///
/// This exists to make auditing straight forward and to limit the need to depend
/// on account to have access to this.
module starcoin_framework::create_signer {
    friend starcoin_framework::account;
    friend starcoin_framework::starcoin_account;
    friend starcoin_framework::coin;
    friend starcoin_framework::fungible_asset;
    friend starcoin_framework::genesis;
    // friend starcoin_framework::multisig_account;
    friend starcoin_framework::object;
    friend starcoin_framework::stc_transaction_validation;
    friend starcoin_framework::block_reward;
    friend starcoin_framework::transfer_scripts;

    public(friend) native fun create_signer(addr: address): signer;
}
