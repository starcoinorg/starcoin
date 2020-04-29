use 0xeae6b71b9583150c1c32bc9500ee5d15::MyToken;
use 0x0::LibraAccount;

fun main(payee: address, auth_key_prefix: vector<u8>, amount: u64) {
    LibraAccount::pay_from_sender<MyToken::T>(payee, auth_key_prefix, amount);
}