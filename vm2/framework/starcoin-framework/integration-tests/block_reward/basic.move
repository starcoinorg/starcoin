//# init -n dev

//# faucet --addr alice

//# faucet --addr Genesis

//# run --signers Genesis

script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 2;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers alice


script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 1;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        // failed with ENOT_GENESIS_ACCOUNT
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: "Keep(ABORTED { code: 2818"


//# run --signers Genesis
script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 0; // if current_number == 0 then do_nothing
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers Genesis
script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 1; //failed with ECURRENT_NUMBER_IS_WRONG
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: "Keep(ABORTED { code: 26119"


//# run --signers Genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use std::vector;
    use starcoin_framework::block_reward;

    fun process_block_reward(account: signer) {
        let current_number = 3;
        let current_reward = 1000;

        // let auth_key_vec = vector::empty();
        // let current_author = create_object_address(
        //     &signer::address_of(&account),
        //     auth_key_vec
        // );// Authenticator::derived_address(copy auth_key_vec);

        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            @0x1,
            vector::empty(),
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers Genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use starcoin_framework::block_reward;

    fun process_block_reward(account: signer) {
        let current_number = 4;
        let current_reward = 1000;

        let current_author = @0x2;
        // auth_key_vec argument is deprecated in StarcoinFrameworklib v5
        let auth_key_vec = x"";

        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers Genesis
script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 5;
        let current_reward = 0;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers Genesis

script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 6;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED


//# run --signers Genesis

script {
    use starcoin_framework::block_reward;
    use starcoin_framework::vector;

    fun process_block_reward(account: signer) {
        let current_number = 7;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = vector::empty<u8>();
        block_reward::process_block_reward(
            &account,
            current_number,
            current_reward,
            current_author,
            auth_key_vec,
            starcoin_framework::coin::zero()
        );
    }
}
// check: EXECUTED