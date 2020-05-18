script {
use 0x0::Coin;

// Publish a newly created `Preburn<Token>` resource under the sender's account.
// This will fail if the sender already has a published `Preburn<Token>` resource.
fun main<Token>() {
    Coin::publish_preburn(Coin::new_preburn<Token>())
}
}
