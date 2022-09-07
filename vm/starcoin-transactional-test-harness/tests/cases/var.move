//# init -n dev

//# faucet --addr creator --amount 12345000000

//# read-json tests/cases/content.json

//# var a=123 addr={{$.faucet[0].txn.raw_txn.decoded_payload.ScriptFunction.args[0]}}

//#run --signers creator --args {{$.var[0].a}}u64 --args {{$.var[0].addr}}
script {
     use StarcoinFramework::Debug;
     fun main(_sender: signer, number: u64, addr: address) {
          Debug::print(&number);
          Debug::print(&addr);
     }
}
