// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::{Steps, StepsBuilder};
use move_core_types::move_resource::MoveStructType;
use starcoin_types::access_path::AccessPath;
use starcoin_vm_types::{account_config::AccountResource, state_store::state_key::StateKey};

pub fn steps() -> Steps<MyWorld> {
    let mut builder: StepsBuilder<MyWorld> = Default::default();
    builder.then("state proof", |world: &mut MyWorld, _step| {
        let client = world.default_rpc_client.as_ref().take().unwrap();
        let account = world.default_account.as_ref().take().unwrap();
        let access_path = AccessPath::resource_access_path(
            account.address.clone(),
            AccountResource::struct_tag(),
        );
        let key = StateKey::resource(account.address(), &AccountResource::struct_tag());
        let proof = client.clone().state_get_with_proof(key).unwrap();
        let state_root = client.clone().state_get_state_root().unwrap();
        proof
            .into_state_proof()
            .verify(state_root, access_path)
            .unwrap();
    });
    builder.build()
}
