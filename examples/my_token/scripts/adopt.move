use 0xeae6b71b9583150c1c32bc9500ee5d15::MyToken;
use 0x0::Account;

fun main() {
    // Create 'Balance<Token>' resource under sender account to receive token
    Account::create_new_balance<MyToken::T>();
}