// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use move_vm2_core_types::move_resource::MoveStructType;
use starcoin_vm2_state_api::StateWithProof;
use starcoin_vm2_vm_types::{
    access_path::AccessPath, account_config::AccountResource, state_store::state_key::StateKey,
};

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder.then("state proof", |world: &mut MyWorld, _step| {
        let client = world
            .default_rpc_client
            .as_ref()
            .take()
            .expect("get rpc client failed");
        let account = world
            .default_account
            .as_ref()
            .take()
            .expect("get account failed");
        let state_key = StateKey::resource(account.address(), &AccountResource::struct_tag())
            .expect("should have state");
        let proof_view = client
            .clone()
            .state_get_with_proof2(state_key.clone())
            .expect("should have state");
        let state_root = client
            .clone()
            .state_get_state_root2()
            .expect("should have state root2");
        let proof: StateWithProof = proof_view.try_into().expect("should convert proof view");
        proof
            .verify(
                state_root,
                AccessPath::resource_access_path(*account.address(), AccountResource::struct_tag()),
            )
            .expect("verify proof failed");
    });
    builder.build()
}
