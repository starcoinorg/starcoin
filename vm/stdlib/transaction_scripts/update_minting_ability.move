script {
use 0x0::Coin;
fun main<Currency>(allow_minting: bool) {
    Coin::update_minting_ability<Currency>(allow_minting)
}
}
