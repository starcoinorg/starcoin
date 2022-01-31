//# init -n dev

//# faucet --addr alice --amount 0

//# faucet --addr Genesis

//# run --signers Genesis
// Minting by genesis
script {
use StarcoinFramework::STC::{STC};
use StarcoinFramework::Token;
use StarcoinFramework::Account;
fun main(account: signer) {
    // mint 100 coins and check that the market cap increases appropriately
    let old_market_cap = Token::market_cap<STC>();
    let coin = Token::mint<STC>(&account, 100);
    assert!(Token::value<STC>(&coin) == 100, 8000);
    assert!(Token::market_cap<STC>() == old_market_cap + 100, 8001);

    // get rid of the coin
    Account::deposit(@alice, coin);
}
}

// will fail with MISSING_DATA because STC mint capability has been destroyed
// check: MISSING_DATA
// check: Keep



//# run --signers StarcoinAssociation
script {
    use StarcoinFramework::STC::{Self, STC};
    use StarcoinFramework::Token;
    use StarcoinFramework::Account;
    fun test_burn(account: signer) {

        let market_cap = Token::market_cap<STC>();
        let coin = Account::withdraw<STC>(&account, 100);
        assert!(Token::value<STC>(&coin) == 100, 8000);

        // burn 100 coins and check that the market cap decreases appropriately
        // burn the coin
        STC::burn(coin);
        assert!(Token::market_cap<STC>() == market_cap - 100, 8002);
    }
}

// check: EXECUTED