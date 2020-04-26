use 0x2f1ea8901faef384366c615447e460da::MyToken;
use 0x0::LibraAccount;

fun main() {
    // Create 'Balance<Token>' resource under sender account to receive token
    LibraAccount::create_new_balance<MyToken::T>();
}