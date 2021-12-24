//# init -n dev

//# faucet --addr alice --amount 50000000

//# run --signers alice --gas-budget 1000
script {
fun main() {
    while (true) {}
}
}

// check: OUT_OF_GAS
// check: gas_used
// check: 1000


//# run --signers alice --gas-budget 599

script {
    fun main() {
        while (true) {}
    }
}

// check: Discard
// check: MAX_GAS_UNITS_BELOW_MIN_TRANSACTION_GAS_UNITS


//# run --signers alice --gas-budget 400000001

script {
    fun main() {
        while (true) {}
    }
}

// check: Discard
// check: MAX_GAS_UNITS_EXCEEDS_MAX_GAS_UNITS_BOUND