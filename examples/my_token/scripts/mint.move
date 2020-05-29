script{
use {{sender}}::MyToken;

fun main(amount: u64) {
    MyToken::mint(amount);
}
}