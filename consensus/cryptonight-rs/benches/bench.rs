#[macro_use]
extern crate bencher;

use bencher::Bencher;
use cryptonight_rs::cryptonight_r;

fn bench_slow4_256(b: &mut Bencher) {
    let bytes = [1u8; 256];
    b.iter(|| cryptonight_r(&bytes[..], bytes.len()))
}

fn bench_slow4_1024(b: &mut Bencher) {
    let bytes = [1u8; 1024];
    b.iter(|| cryptonight_r(&bytes[..], bytes.len()))
}

benchmark_group!(benches, bench_slow4_256, bench_slow4_1024);
benchmark_main!(benches);
