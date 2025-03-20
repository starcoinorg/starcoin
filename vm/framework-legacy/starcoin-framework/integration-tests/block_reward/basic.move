//# init -n dev

//# faucet --addr alice

//# faucet --addr Genesis

//# run --signers Genesis

script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 2;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers alice


script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 1;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        // failed with ENOT_GENESIS_ACCOUNT
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 2818"


//# run --signers Genesis


script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 0; // if current_number == 0 then do_nothing
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers Genesis


script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 1; //failed with ECURRENT_NUMBER_IS_WRONG
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: "Keep(ABORTED { code: 26119"



//# run --signers Genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Authenticator;

    fun process_block_reward(account: signer) {
        let current_number = 3;
        let current_reward = 1000;

        let auth_key_vec = x"91e941f5bc09a285705c092dd654b94a7a8e385f898968d4ecfba49609a13461";
        let current_author = Authenticator::derived_address(copy auth_key_vec);

        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers Genesis
// author account doesn't exist, process_block_reward() will create the account
script {
    use StarcoinFramework::BlockReward;

    fun process_block_reward(account: signer) {
        let current_number = 4;
        let current_reward = 1000;

        let current_author = @0x2;
        // auth_key_vec argument is deprecated in StarcoinFrameworklib v5
        let auth_key_vec = x"";

        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers Genesis


script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 5;
        let current_reward = 0;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers Genesis

script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 6;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED


//# run --signers Genesis

script {
    use StarcoinFramework::BlockReward;
    use StarcoinFramework::Vector;

    fun process_block_reward(account: signer) {
        let current_number = 7;
        let current_reward = 1000;
        let current_author = @alice;
        let auth_key_vec = Vector::empty<u8>();
        BlockReward::process_block_reward(&account, current_number, current_reward, current_author, auth_key_vec, StarcoinFramework::Token::zero());
    }
}
// check: EXECUTED