script{
use {{sender}}::MyToken;

fun main(account: &signer, amount: u64) {
    MyToken::mint(account, amount);
}
}