use 0x2f1ea8901faef384366c615447e460da::MyToken;
use 0x0::LibraAccount;

fun main(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
    LibraAccount::pay_from_sender<MyToken::T>(payee, auth_key_prefix, amount);
}