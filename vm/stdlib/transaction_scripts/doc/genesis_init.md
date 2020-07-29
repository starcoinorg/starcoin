
<a name="SCRIPT"></a>

# Script `genesis_init.move`

### Table of Contents

-  [Function `genesis_init`](#SCRIPT_genesis_init)



<a name="SCRIPT_genesis_init"></a>

## Function `genesis_init`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_genesis_init">genesis_init</a>(publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;, native_schedule: vector&lt;u8&gt;, reward_delay: u64, uncle_rate_target: u64, epoch_time_target: u64, reward_half_epoch: u64, init_block_time_target: u64, block_difficulty_window: u64, reward_per_uncle_percent: u64, min_time_target: u64, max_uncles_per_block: u64, total_supply: u128, pre_mine_percent: u64, parent_hash: vector&lt;u8&gt;, association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;, chain_id: u8, genesis_timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_genesis_init">genesis_init</a>(publishing_option: vector&lt;u8&gt;, instruction_schedule: vector&lt;u8&gt;,
                 native_schedule: vector&lt;u8&gt;, reward_delay: u64,
                 uncle_rate_target:u64,epoch_time_target: u64,
                 reward_half_epoch: u64, init_block_time_target: u64,
                 block_difficulty_window: u64, reward_per_uncle_percent: u64,
                 min_time_target:u64, max_uncles_per_block:u64,
                 total_supply: u128, pre_mine_percent:u64, parent_hash: vector&lt;u8&gt;,
                 association_auth_key: vector&lt;u8&gt;, genesis_auth_key: vector&lt;u8&gt;,
                 chain_id: u8,genesis_timestamp: u64,
                 ) {
    <a href="../../modules/doc/Genesis.md#0x1_Genesis_initialize">Genesis::initialize</a>(publishing_option, instruction_schedule,
                        native_schedule, reward_delay,
                        uncle_rate_target ,epoch_time_target,reward_half_epoch,
                        init_block_time_target, block_difficulty_window,
                        min_time_target, max_uncles_per_block,
                        reward_per_uncle_percent, total_supply,
                        pre_mine_percent, parent_hash ,
                        association_auth_key, genesis_auth_key, chain_id, genesis_timestamp);
}
</code></pre>



</details>
