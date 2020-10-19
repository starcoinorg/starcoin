#[macro_use]
extern crate bencher;
extern crate cryptonight;

use bencher::Bencher;
use cryptonight::cryptonight_r;

fn bench_slow4_32(b: &mut Bencher) {
    let bytes = [1u8; 32];
    b.iter(|| cryptonight_r(&bytes[..], bytes.len()))
}

fn bench_slow4_1024(b: &mut Bencher) {
    let bytes = [1u8; 1024];
    b.iter(|| cryptonight_r(&bytes[..], bytes.len()))
}

benchmark_group!(benches, bench_slow4_32, bench_slow4_1024);
benchmark_main!(benches);
