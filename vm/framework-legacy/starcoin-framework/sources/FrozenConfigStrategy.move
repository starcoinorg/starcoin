module StarcoinFramework::FrozenConfigStrategy {
    use StarcoinFramework::ACL;
    use StarcoinFramework::Account;
    use StarcoinFramework::Block;
    use StarcoinFramework::ChainId;
    use StarcoinFramework::Config;
    use StarcoinFramework::CoreAddresses::{Self, assert_association_root_address, assert_genesis_address};
    use StarcoinFramework::Errors;
    use StarcoinFramework::FrozenConfig::{Self, FrozenConfig};
    use StarcoinFramework::STC::{Self, STC};
    use StarcoinFramework::Signer;
    use StarcoinFramework::Vector;

    const ERR_ADD_ACCOUNT_FAILED: u64 = 101;
    const ERR_ADD_ACCOUNT_HAS_EXISTS: u64 = 102;
    const ERR_ADD_CANNOT_BE_CORE_ADDRESS: u64 = 103;
    const ERR_REMOVE_ACCOUNT_FAILED: u64 = 104;
    const ERR_REMOVE_ACCOUNT_NOT_EXISTS: u64 = 105;
    const ERR_BURN_NOT_YET_TIME: u64 = 106;
    const ERR_BURN_FROZEN_LIST_IS_EMPTY: u64 = 107;

    struct BurnBlockNumber has key {
        block_number: u64,
    }

    public fun initialize(
        framework_account: &signer,
        main_bnum: u64,
        barnard_bnum: u64,
        test_bnum: u64,
        other_bnum: u64
    ) {
        assert_genesis_address(framework_account);

        let association_account_address = CoreAddresses::ASSOCIATION_ROOT_ADDRESS();

        let block_number_by_chain = if (ChainId::is_main()) {
            main_bnum
        } else if (ChainId::is_barnard()) {
            barnard_bnum
        } else if (ChainId::is_test()) {
           test_bnum
        } else {
            other_bnum
        };
        if (!exists<BurnBlockNumber>(association_account_address)) {
            let association_account = Account::create_signer_friend(association_account_address);

            // Initialize config
            FrozenConfig::initialize(&association_account, Self::frozen_list_v1());

            // Initalize BurnBlockNumber
            move_to(&association_account, BurnBlockNumber {
                block_number: block_number_by_chain
            })
        }
    }

    public entry fun add_account(accocial_account: signer, account: address) {
        assert_config_address(&accocial_account);

        assert!(!CoreAddresses::is_core_address(account), Errors::invalid_state(ERR_ADD_CANNOT_BE_CORE_ADDRESS));

        let acl = FrozenConfig::get_frozen_account_list(config_address());
        assert!(!ACL::contains(&acl, account), Errors::invalid_state(ERR_ADD_ACCOUNT_HAS_EXISTS));
        ACL::add(&mut acl, account);
        FrozenConfig::set_account_list(&accocial_account, acl);

        assert!(
            ACL::contains(&FrozenConfig::get_frozen_account_list(config_address()), account),
            Errors::invalid_state(ERR_ADD_ACCOUNT_FAILED)
        );
    }

    public entry fun remove_account(associal_account: signer, account: address) {
        assert_config_address(&associal_account);

        let acl = FrozenConfig::get_frozen_account_list(config_address());
        assert!(ACL::contains(&acl, account), Errors::invalid_state(ERR_REMOVE_ACCOUNT_NOT_EXISTS));
        ACL::remove(&mut acl, account);
        FrozenConfig::set_account_list(&associal_account, acl);

        // Check has added
        assert!(
            !ACL::contains(&FrozenConfig::get_frozen_account_list(config_address()), account),
            Errors::invalid_state(ERR_REMOVE_ACCOUNT_FAILED)
        );
    }

    public entry fun set_global_frozen(sender: signer, frozen: bool) {
        assert_config_address(&sender);
        FrozenConfig::set_global_frozen(&sender, frozen);
    }

    public fun has_frozen_global(txn_sender: address): bool {
        if (CoreAddresses::is_core_address(txn_sender)) {
            return false
        };

        if (Config::config_exist_by_address<FrozenConfig>(config_address())) {
            FrozenConfig::get_frozen_global(config_address())
        } else {
            true
        }
    }

    public fun has_frozen_account(txn_sender: address): bool {
        if (CoreAddresses::is_core_address(txn_sender)) {
            return false
        };

        if (Config::config_exist_by_address<FrozenConfig>(config_address())) {
            let list = FrozenConfig::get_frozen_account_list(config_address());
            ACL::contains(&list, txn_sender)
        } else {
            true
        }
    }

    public entry fun update_burn_block_number(associal_account: signer, block_number: u64) acquires BurnBlockNumber {
        assert_association_root_address(&associal_account);

        let burn_block_number =
            borrow_global_mut<BurnBlockNumber>(Signer::address_of(&associal_account));
        burn_block_number.block_number = block_number;
    }

    /// Burn all frozen account balance
    /// First checks if the current block number is greater than the block number stored in the `BurnBlockNumber` resource.
    /// If the condition is met, it retrieves the list of frozen account addresses from the access - control list (ACL).
    /// Then it iterates through this list, withdraws the entire STC balance from each frozen account, and burns the withdrawn STC.
    public entry fun do_burn_frozen() acquires BurnBlockNumber {
        let current_block_number = Block::get_current_block_number();
        let burn_block_number =
            borrow_global_mut<BurnBlockNumber>(CoreAddresses::ASSOCIATION_ROOT_ADDRESS());

        assert!(current_block_number >= burn_block_number.block_number, Errors::invalid_state(ERR_BURN_NOT_YET_TIME));

        let acl = FrozenConfig::get_frozen_account_list(config_address());
        let addresses = ACL::get_vector(&acl);
        let len = Vector::length(&addresses);
        assert!(len > 0, Errors::invalid_state(ERR_BURN_FROZEN_LIST_IS_EMPTY));

        let i = 0;
        while (i < len) {
            let frozen_address = *Vector::borrow(&addresses, i);
            let balance = Account::balance<STC>(frozen_address);
            if (balance > 0) {
                let frozen_signer = Account::create_signer_friend(frozen_address);
                let stc = Account::withdraw<STC>(&frozen_signer, balance);
                STC::destroy(stc);
            };
            i = i + 1;
        }
    }

    public fun frozen_list_v1(): ACL::ACL {
        let acl = ACL::empty();

        // Add the initialize frozen account list
        if (ChainId::is_main()) {
            ACL::add(&mut acl, @0x11ed1fa4209b6f0f03e5385b8bc5d1ea);
            ACL::add(&mut acl, @0x329173fe798bfd77094a101c0adad3b4);
            ACL::add(&mut acl, @0xe845c1ae63507c3fbc2f31af0bcc18bb);
            ACL::add(&mut acl, @0x4a7198503af3c765030d4e43863f64a7);
            ACL::add(&mut acl, @0xbab1094a9ed5b2a2d3a10c143cded8a5);
            ACL::add(&mut acl, @0x7af065f4fa20ff2e1bc54be9b17184ed);
            ACL::add(&mut acl, @0x10ab9214c40102c524a12788849210f1);
            ACL::add(&mut acl, @0x2e890d015e7bd850e4ec99da86a952c0);
            ACL::add(&mut acl, @0xb9ad8b357eb59ec508db8e0f19515ae0);
            ACL::add(&mut acl, @0xe14270fab28624f05ff56472e3f1c2f7);
            ACL::add(&mut acl, @0xf9f1bfbbea129e6cbb6d0e11ece3e737);
            ACL::add(&mut acl, @0x0c44cf1168999b923b16d50c86934b56);
            ACL::add(&mut acl, @0x0ae121570b3fd6c9701fce43f06a3c27);
            ACL::add(&mut acl, @0x414f5c01ce1fe0020883020ef878f934);
            ACL::add(&mut acl, @0x7e969eb99f7d9c08cf71db20bc7323bf);
            ACL::add(&mut acl, @0xa62594faff9b19cecc3a511ca0dd3abd);
            ACL::add(&mut acl, @0x15c37f15045f448d26fd262d86d5619c);
            ACL::add(&mut acl, @0xf0d8331409d2da08b5d22c41772d7df7);
            ACL::add(&mut acl, @0xe0c0ce2df4f1e0b0f1b6dc10bbabfdb3);
            ACL::add(&mut acl, @0x4532c92d46cda2257fc9896b7bc0d031);
            ACL::add(&mut acl, @0xb78ff901ddc89744269f5b194fe124ec);
            ACL::add(&mut acl, @0xda9c2b5689b3c9ab8ecd3b0140505117);
            ACL::add(&mut acl, @0xa631d4daafec285ac92a81a9bd50f753);
            ACL::add(&mut acl, @0x58da94cd48805d9f98e80ae6734c0248);
            ACL::add(&mut acl, @0x3a126aee08f6c4cc905091943e9140b9);
            ACL::add(&mut acl, @0x79e5f6ce285211fe350369d0a52fee0d);
            ACL::add(&mut acl, @0x55d770233251c2973c09f8929610f12b);
            ACL::add(&mut acl, @0x375842560f651807d837b71ffd715458);
            ACL::add(&mut acl, @0x7b202199ec36e84b5fa89027690d2a6e);
            ACL::add(&mut acl, @0x8d9b5f9874a911297d39cdd931b6466b);
            ACL::add(&mut acl, @0x5918b0782056c9b698459ad37565d15b);
            ACL::add(&mut acl, @0x23b1620cf3b4f4528b09e31f109e732d);
            ACL::add(&mut acl, @0x614d3e65850a05365ed0556e483c9bae);
            ACL::add(&mut acl, @0xe8891c3775e9ce4e827b7a575e0731fa);
            ACL::add(&mut acl, @0x1e92f96b0d230e7b61b22b4d1d356b77);
            ACL::add(&mut acl, @0x114774968e64412c323605ceaf4fe8d5);
            ACL::add(&mut acl, @0x8f838a32dfaf44911466410ceed7e398);
            ACL::add(&mut acl, @0xdc9d382c448261ff4ba836fc5dbaba63);
            ACL::add(&mut acl, @0x1702e4f0df56482d09d233e4affbc0b3);
            ACL::add(&mut acl, @0x7b2ac05e6467aeb927cd6fa473badcf8);
            ACL::add(&mut acl, @0x8096295553fd54c584b8e961da18ab0c);
            ACL::add(&mut acl, @0x1eef2699f7ba8c79133c261bc54fce2c);
            ACL::add(&mut acl, @0xba73558ae7b59f6fdcff09c9ad1821cf);
            ACL::add(&mut acl, @0x211e0ae997fdd0da507713be1c160e8d);
            ACL::add(&mut acl, @0x03691f8d00b79502498f3b47faa8eafa);
            ACL::add(&mut acl, @0xa6caa5c2a2a4168c383be7f08b31087f);
            ACL::add(&mut acl, @0xca34c1afcbec6401b65642bdc9aa4e09);
            ACL::add(&mut acl, @0x8697aa50a5776d0ab22614fb9edf6675);
            ACL::add(&mut acl, @0xb6cda160a6433f7d648bd24a10a06a6a);
            ACL::add(&mut acl, @0x00b7563162ee94a57457ba08a5f80c3c);
            ACL::add(&mut acl, @0x12d95e1db2a54d15bc50927e5655af2d);
            ACL::add(&mut acl, @0x9c59015c60e0f262d3b6571bcd9c5b0b);
            ACL::add(&mut acl, @0x6820910808aba0dda29b486064ffc17f);
        } else {
            ACL::add(&mut acl, @0xd0c5a06ae6100ce115cad1600fe59e96);
            ACL::add(&mut acl, @0x1af80d10cb642adcd9f7fee1420104ec);
        };
        acl
    }

    fun config_address(): address {
        CoreAddresses::ASSOCIATION_ROOT_ADDRESS()
    }

    fun assert_config_address(sender: &signer) {
        CoreAddresses::assert_association_root_address(sender);
    }
}
