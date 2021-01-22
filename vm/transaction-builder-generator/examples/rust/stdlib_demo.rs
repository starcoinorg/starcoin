// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use bcs_ext;
use starcoin_stdlib::encode_peer_to_peer_with_metadata_script;
use starcoin_types::{AccountAddress, Identifier, StructTag, TypeTag};

fn main() {
    let token = TypeTag::Struct(StructTag {
        address: AccountAddress([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
        module: Identifier("LBR".into()),
        name: Identifier("LBR".into()),
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
    let script = encode_peer_to_peer_with_metadata_script(
        token,
        payee,
        key_vec.to_vec(),
        amount,
        Vec::new(),
    );

    let output = bcs_ext::to_bytes(&script).unwrap();
    for o in output {
        print!("{} ", o);
    }
    println!();
}
