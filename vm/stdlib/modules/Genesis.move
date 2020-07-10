address 0x1 {

module Genesis {
   use 0x1::CoreAddresses;
   use 0x1::Account;
   use 0x1::Signer;
   use 0x1::TransactionTimeout;
   use 0x1::Coin;
   use 0x1::Timestamp;
   use 0x1::Association;
   use 0x1::STC::{Self,STC};
   use 0x1::PackageTxnManager;
   use 0x1::Consensus;
   use 0x1::Version;
   use 0x1::RewardConfig;
   use 0x1::VMConfig;
   use 0x1::Vector;
   use 0x1::Block;
   use 0x1::TransactionFee;
   use 0x1::BlockReward;

   //TODO refactor when move support ABI, and pass struct by argument
   public fun initialize(publishing_option: vector<u8>, instruction_schedule: vector<u8>,native_schedule: vector<u8>,
                         reward_halving_interval: u64, reward_base: u64, reward_delay: u64,
                         uncle_rate_target:u64,epoch_time_target: u64,reward_half_time_target: u64,
                         total_supply: u64, pre_mine_percent:u64,
                         parent_hash: vector<u8>,
                         association_auth_key: vector<u8>,
                         genesis_auth_key: vector<u8>,
   ){
        assert(Timestamp::is_genesis(), 1);

        let dummy_auth_key_prefix = x"00000000000000000000000000000000";

        // create genesis account
        let genesis_account = Account::create_genesis_account(CoreAddresses::GENESIS_ACCOUNT(),copy dummy_auth_key_prefix);

        Block::initialize(&genesis_account, parent_hash);

        Coin::initialize(&genesis_account);

        // init config
        VMConfig::initialize(&genesis_account, publishing_option, instruction_schedule, native_schedule);
        RewardConfig::initialize(&genesis_account, reward_halving_interval, reward_base, reward_delay);
        Version::initialize(&genesis_account);
        Consensus::initialize(&genesis_account,uncle_rate_target,epoch_time_target,reward_half_time_target);

        TransactionTimeout::initialize(&genesis_account);

        STC::initialize(&genesis_account);
        Account::add_currency<STC>(&genesis_account);

        let association = Account::create_genesis_account(CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), copy dummy_auth_key_prefix);
        Account::add_currency<STC>(&association);
        Association::initialize(&association);
        Association::grant_privilege<Coin::AddCurrency>(&association, &association);

        let association_balance = total_supply * pre_mine_percent / 100;
        if (association_balance > 0) {
             Account::mint_to_address<STC>(&genesis_account, Signer::address_of(&association), association_balance);
        };
        let miner_reward_balance = total_supply - association_balance;
        BlockReward::initialize(&genesis_account, miner_reward_balance);

        TransactionFee::initialize(&genesis_account);
        //Grant stdlib maintainer to association
        PackageTxnManager::grant_maintainer(&genesis_account, Signer::address_of(&association));
        //TODO set stdlib upgrade strategy.

        // only dev network set genesis auth key.
        if (!Vector::is_empty(&genesis_auth_key)){
            let genesis_rotate_key_cap = Account::extract_key_rotation_capability(&genesis_account);
            Account::rotate_authentication_key(&genesis_rotate_key_cap, genesis_auth_key);
            Account::restore_key_rotation_capability(genesis_rotate_key_cap);
        };

        let assoc_rotate_key_cap = Account::extract_key_rotation_capability(&association);
        Account::rotate_authentication_key(&assoc_rotate_key_cap, association_auth_key);
        Account::restore_key_rotation_capability(assoc_rotate_key_cap);

        //Set global time, and Timestamp::is_genesis() will return false.
        Timestamp::initialize(&genesis_account);

        Account::release_genesis_signer(genesis_account);
        Account::release_genesis_signer(association);
   }
}

}
