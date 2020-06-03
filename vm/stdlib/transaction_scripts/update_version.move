script {
use 0x0::Version;

fun main(major: u64) {
    Version::set(major)
}
}
