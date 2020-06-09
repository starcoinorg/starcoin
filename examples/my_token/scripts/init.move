script{
use {{sender}}::MyToken;

fun main(account: &signer) {
    MyToken::init(account);
}
}