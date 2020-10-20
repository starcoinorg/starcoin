script {
use 0x1::VMConfig;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun propose_update_vm_config(account: &signer,
    instruction_schedule: vector<u8>,
    native_schedule: vector<u8>,
    global_memory_per_byte_cost: u64,
    global_memory_per_byte_write_cost: u64,
    min_transaction_gas_units: u64,
    large_transaction_cutoff: u64,
    instrinsic_gas_per_byte: u64,
    maximum_number_of_gas_units: u64,
    min_price_per_gas_unit: u64,
    max_price_per_gas_unit: u64,
    max_transaction_size_in_bytes: u64,
    gas_unit_scaling_factor: u64,
    default_account_size: u64,
    exec_delay: u64,) {
    let vm_config = VMConfig::new_vm_config(instruction_schedule,
                        native_schedule,
                        global_memory_per_byte_cost,
                        global_memory_per_byte_write_cost,
                        min_transaction_gas_units,
                        large_transaction_cutoff,
                        instrinsic_gas_per_byte,
                        maximum_number_of_gas_units,
                        min_price_per_gas_unit,
                        max_price_per_gas_unit,
                        max_transaction_size_in_bytes,
                        gas_unit_scaling_factor,
                        default_account_size);
    OnChainConfigDao::propose_update<STC::STC, VMConfig::VMConfig>(account, vm_config, exec_delay);
}

spec fun propose_update_vm_config {
    pragma verify = false;
}
}
