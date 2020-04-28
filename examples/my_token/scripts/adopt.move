use 0xeae6b71b9583150c1c32bc9500ee5d15::MyToken;
use 0x0::LibraAccount;

fun main() {
    // Create 'Balance<Token>' resource under sender account to receive token
    LibraAccount::create_new_balance<MyToken::T>();
}