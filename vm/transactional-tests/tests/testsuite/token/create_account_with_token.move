//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::Account;
use StarcoinFramework::STC::STC;
use StarcoinFramework::DummyToken::DummyToken;
use StarcoinFramework::Authenticator;

fun main() {
    let dummy_auth_key = x"fb51f08c8e63ed9f4eac340b25d1b01d75995fa86f8ebc0b0819ebf80abc0ee6";
    let created_address = Authenticator::derived_address(dummy_auth_key);
    Account::create_account_with_address<STC>(created_address);
    assert!(Account::is_accepts_token<STC>(created_address), 1);
    assert!(Account::is_accepts_token<DummyToken>(created_address), 2);
}
}



//# run --signers alice
script {
use StarcoinFramework::Account;
use StarcoinFramework::STC::STC;
use StarcoinFramework::DummyToken::DummyToken;
use StarcoinFramework::Authenticator;

fun main() {
    let dummy_auth_key = x"9028e2757e3e57339af4b3df1d818cddac6e527363459f072d70484599956c8a";
    let created_address = Authenticator::derived_address(dummy_auth_key);
    Account::create_account_with_address<DummyToken>(created_address);
    assert!(Account::is_accepts_token<STC>(created_address), 1);
    assert!(Account::is_accepts_token<DummyToken>(created_address), 2);
}
}
