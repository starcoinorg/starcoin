use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::file_format_common;
use move_binary_format::file_format_common::{IDENTIFIER_SIZE_MAX, LEGACY_IDENTIFIER_SIZE_MAX};
use move_bytecode_verifier::VerifierConfig;
use move_vm_runtime::config::VMConfig;
use move_vm_types::loaded_data::runtime_types::TypeBuilder;
use starcoin_vm2_vm_types::on_chain_config::{FeatureFlag, Features, TimedFeatureFlag, TimedFeatures};

pub fn get_max_binary_format_version(
    features: &Features,
    gas_feature_version_opt: Option<u64>,
) -> u32 {
    // For historical reasons, we support still < gas version 5, but if a new caller don't specify
    // the gas version, we default to 5, which was introduced in late '22.
    let gas_feature_version = gas_feature_version_opt.unwrap_or(5);
    if gas_feature_version < 5 {
        file_format_common::VERSION_5
    } else if features.is_enabled(FeatureFlag::VM_BINARY_FORMAT_V7) {
        file_format_common::VERSION_7
    } else if features.is_enabled(FeatureFlag::VM_BINARY_FORMAT_V6) {
        file_format_common::VERSION_6
    } else {
        file_format_common::VERSION_5
    }
}

pub fn starcoin_prod_deserializer_config(features: &Features) -> DeserializerConfig {
    DeserializerConfig::new(
        get_max_binary_format_version(features, None),
        get_max_identifier_size(features),
    )
}

pub fn get_max_identifier_size(features: &Features) -> u64 {
    if features.is_enabled(FeatureFlag::LIMIT_MAX_IDENTIFIER_LENGTH) {
        IDENTIFIER_SIZE_MAX
    } else {
        LEGACY_IDENTIFIER_SIZE_MAX
    }
}

pub fn verifier_config(features: &Features, _timed_features: &TimedFeatures) -> VerifierConfig {
    VerifierConfig {
        max_loop_depth: Some(5),
        max_generic_instantiation_length: Some(32),
        max_function_parameters: Some(128),
        max_basic_blocks: Some(1024),
        max_value_stack_size: 1024,
        max_type_nodes: Some(256),
        max_dependency_depth: Some(256),
        max_push_size: Some(10000),
        max_struct_definitions: None,
        max_fields_in_struct: None,
        max_function_definitions: None,
        max_back_edges_per_function: None,
        max_back_edges_per_module: None,
        max_basic_blocks_in_script: None,
        max_per_fun_meter_units: Some(1000 * 80000),
        max_per_mod_meter_units: Some(1000 * 80000),
        use_signature_checker_v2: features.is_enabled(FeatureFlag::SIGNATURE_CHECKER_V2),
        sig_checker_v2_fix_script_ty_param_count: features
            .is_enabled(FeatureFlag::SIGNATURE_CHECKER_V2_SCRIPT_FIX),
    }
}

pub fn starcoin_prod_vm_config(
    features: &Features,
    timed_features: &TimedFeatures,
    ty_builder: TypeBuilder,
) -> VMConfig {
    // Note: binary format v6 adds a few new integer types and their corresponding instructions.
    //       Therefore it depends on a new version of the gas schedule and cannot be allowed if
    //       the gas schedule hasn't been updated yet.
    // todo: select gas_feature_version properly. Currently it is set to 5 by default, but the
    //  default features has VM_BINARY_FORMAT_V6 enabled.
    let max_binary_format_version = get_max_binary_format_version(features, None);

    let max_identifier_size = get_max_identifier_size(features);

    let verifier_config = verifier_config(features, timed_features);

    let mut type_max_cost = 0;
    let mut type_base_cost = 0;
    let mut type_byte_cost = 0;
    if timed_features.is_enabled(TimedFeatureFlag::LimitTypeTagSize) {
        // 5000 limits type tag total size < 5000 bytes and < 50 nodes
        type_max_cost = 5000;
        type_base_cost = 100;
        type_byte_cost = 1;
    }

    VMConfig {
        verifier_config,
        deserializer_config: DeserializerConfig::new(max_binary_format_version, max_identifier_size),
        paranoid_type_checks: /*crate::StarcoinVM::get_paranoid_checks() */ false,
        max_value_nest_depth: Some(128),
        type_max_cost,
        type_base_cost,
        type_byte_cost,
        // todo: support aggregator_v2_type_tagging, set false as default now.
        aggregator_v2_type_tagging: false,
        check_invariant_in_swap_loc: false,
        ty_builder,
    }
}
