//! new-transaction
//! sender: genesis
script{
use 0x1::Version;
fun main(account: &signer) {
    Version::initialize(account);
}
}
// check: EXECUTION_FAILURE
// check: CANNOT_WRITE_EXISTING_RESOUR

//! new-transaction
script{
use 0x1::Version;
fun main(account: &signer) {
    Version::set(account, 0);
}
}

// check: ABORTED
// check: 1


//! new-transaction
//! sender: genesis
script{
use 0x1::Version;
fun main(_account: &signer) {
    let version = Version::get();
    let _ = version;
    assert(version == 1, 100);
}
}

// check: EXECUTED

