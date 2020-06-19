script {
use 0x1::STC;

fun stc_init(association: &signer) {
    STC::initialize(association);
}
}