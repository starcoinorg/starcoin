//! account: alice

//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 1;
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED

//! new-transaction
//! sender: alice
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 1;
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        // failed with ENOT_GENESIS_ACCOUNT
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 2818"

//! new-transaction
//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 0; // if current_number == 0 then do_nothing
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 1; //failed with ECURRENT_NUMBER_IS_WRONG
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 26119"


//! new-transaction
//! sender: genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use 0x1::BlockReward;
    use 0x1::Authenticator;

    fun process_block_reward(account: signer) {
        let current_number = 2;
        let current_reward = 1000;

        let auth_key_vec = x"91e941f5bc09a285705c092dd654b94a7a8e385f898968d4ecfba49609a13461";
        let current_author = Authenticator::derived_address(copy auth_key_vec);

        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use 0x1::BlockReward;

    fun process_block_reward(account: signer) {
        let current_number = 3;
        let current_reward = 1000;

        let current_author = 0x2;
        let auth_key_vec = x""; // failed with EAUTHOR_AUTH_KEY_IS_EMPTY

        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 25863"

//! new-transaction
//! sender: genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use 0x1::BlockReward;
    use 0x1::Authenticator;

    fun process_block_reward(account: signer) {
        let current_number = 3;
        let current_reward = 1000;

        let auth_key_vec = x"5e99e2e5fe070777cbdcf576c29ee715bf81dda7705a276ae07e93781fc1a3c0";
        let _expected_address = Authenticator::derived_address(copy auth_key_vec); //0xbf81dda7705a276ae07e93781fc1a3c0
        let current_author = 0x2; //wrong address
        // EAUTHOR_ADDRESS_AND_AUTH_KEY_MISMATCH
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 26887"

//! new-transaction
//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 3;
        let current_reward = 0;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 4;
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
script {
    use 0x1::BlockReward;
    use 0x1::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 5;
        let current_reward = 1000;
        let current_author = {{alice}};
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, 0x1::Token::zero());
    }
}
// check: EXECUTED