script{
use {{sender}}::MyToken;

fun main(account: &signer, amount: u128) {
    MyToken::mint(account, amount);
}
}