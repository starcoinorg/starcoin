script {
use 0x1::Coin;
fun main<Currency>(account: &signer,allow_minting: bool) {
    Coin::update_minting_ability<Currency>(account, allow_minting)
}
}
