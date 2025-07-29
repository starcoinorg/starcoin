//# init -n test --public-keys SwapAdmin=0x5510ddb2f172834db92842b0b640db08c2bc3cd986def00229045d78cc528ac5

//# faucet --addr alice --amount 10000000000000000


//# run --signers alice
script {
    fun hello(_: signer) {
    }
}
// check: EXECUTED