//! A SignedUserTransaction from a peer or `txpool.submit_hex_transaction`, through the stateless
//! pre-checks the pool runs before any state access, then the encodes a pooled txn goes through
//! when it is relayed or mined.
#![no_main]

use bcs_ext::BCSCodec;
use libfuzzer_sys::fuzz_target;
use network_api::messages::TransactionsMessage;
use starcoin_types::block::BlockBody;
use starcoin_types::transaction::SignedUserTransaction;
use starcoin_vm_types::parser::type_args_too_deep;

fuzz_target!(|data: &[u8]| {
    let Ok(txn) = SignedUserTransaction::decode(data) else {
        return;
    };
    txn.id();
    txn.raw_txn_bytes_len();
    txn.encode().expect("a decoded txn must encode again");
    // The txpool's first check, before signature and VM validation.
    if type_args_too_deep(txn.payload().ty_args()) {
        return;
    }
    // A txn admitted past the nesting check is relayed in a TransactionsMessage and hashed in a
    // block body; neither encode may fail for it.
    TransactionsMessage::new(vec![txn.clone()])
        .encode()
        .expect("an admitted txn must encode in a TransactionsMessage");
    BlockBody::new(vec![txn.clone()], None).hash();
    let _ = txn.check_signature();
});
