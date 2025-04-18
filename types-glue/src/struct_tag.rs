use crate::{account_address, type_tag};
use starcoin_vm2_types::{
    identifier::Identifier as IdentifierVM2, language_storage::StructTag as StructTagVM2,
};
use starcoin_vm_types::{
    identifier::Identifier as IdentifierVM1, language_storage::StructTag as StructTagVM1,
};
use std::str::FromStr;

pub fn vm1_to_vm2(struct_tag: StructTagVM1) -> StructTagVM2 {
    StructTagVM2 {
        address: account_address::vm1_to_vm2(struct_tag.address),
        module: IdentifierVM2::from_str(struct_tag.module.as_str()).expect("invalid module name"),
        name: IdentifierVM2::from_str(struct_tag.module.as_str()).expect("invalid module name"),
        type_args: struct_tag
            .type_params
            .iter()
            .map(|tag| type_tag::vm1_to_vm2(tag.clone()))
            .collect(),
    }
}

pub fn vm2_to_vm1(struct_tag: StructTagVM2) -> StructTagVM1 {
    StructTagVM1 {
        address: account_address::vm2_to_vm1(struct_tag.address),
        module: IdentifierVM1::from_str(struct_tag.module.as_str()).expect("invalid module name"),
        name: IdentifierVM1::from_str(struct_tag.module.as_str()).expect("invalid module name"),
        type_params: struct_tag
            .type_args
            .iter()
            .map(|tag| type_tag::vm2_to_vm1(tag.clone()))
            .collect(),
    }
}

#[test]
fn test_parse_type_tag() {
    // TODO(BobOng): [dual-vm] to do this test
}
