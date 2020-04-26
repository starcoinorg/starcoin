use 0x2f1ea8901faef384366c615447e460da::MyToken;

fun main(amount: u64) {
    MyToken::issue(amount);
}