// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use starcoin_stdlib::{encode_peer_to_peer_with_metadata_script_function, ScriptFunctionCall};
use starcoin_types::{AccountAddress, Identifier, StructTag, TypeTag};
use serde_bytes::ByteBuf as Bytes;

fn main() {
    let token = TypeTag::Struct(StructTag {
        address: AccountAddress([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        module: Identifier("STC".into()),
        name: Identifier("STC".into()),
        type_params: Vec::new(),
    });
    let payee = AccountAddress([
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22,
    ]);
    let key_vec = [
        0x22u8, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x22,
        0x22, 0x22,
    ];
    let amount = 1234567;

    // Now encode and decode a peer to peer transaction script function.
    let payload = encode_peer_to_peer_with_metadata_script_function(
        token,
        payee.clone(),
        Bytes::from(key_vec.to_vec()),
        amount,
        Bytes::from(Vec::new()),
    );
    let function_call = ScriptFunctionCall::decode(&payload);
    match function_call {
        Some(ScriptFunctionCall::PeerToPeerWithMetadata {
                 amount: a,
                 payee: p,
                 ..
             }) => {
            assert_eq!(a, amount);
            assert_eq!(p, payee);
        }
        _ => panic!("unexpected type of script function"),
    };

    let output = bcs_ext::to_bytes(&payload).unwrap();
    for o in output {
        print!("{} ", o);
    }
    println!();
}
