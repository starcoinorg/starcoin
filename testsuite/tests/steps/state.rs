// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use starcoin_types::access_path::AccessPath;

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder.then("state proof", |world: &mut MyWorld, _step| {
        let client = world.rpc_client.as_ref().take().unwrap();
        let account = world.default_account.as_ref().take().unwrap();
        let access_path = AccessPath::new_for_account(account.clone().address);
        let proof = client.state_get_with_proof(access_path.clone()).unwrap();
        let state_root = client.state_get_state_root().unwrap();
        proof
            .proof
            .verify(state_root, access_path, proof.state.as_deref())
            .unwrap();
    });
    builder.build()
}
