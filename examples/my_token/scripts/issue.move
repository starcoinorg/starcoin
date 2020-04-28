use 0xeae6b71b9583150c1c32bc9500ee5d15::MyToken;

fun main(amount: u64) {
    MyToken::issue(amount);
}