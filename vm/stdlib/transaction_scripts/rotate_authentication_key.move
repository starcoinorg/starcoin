script {
use 0x0::Account;
fun main(new_key: vector<u8>) {
  Account::rotate_authentication_key(new_key)
}
}
