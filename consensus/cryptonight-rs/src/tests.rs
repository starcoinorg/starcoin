use super::*;
use rustc_serialize as serialize;
use serialize::hex::FromHex;

struct TestCase {
    input: Vec<u8>,
    output: Vec<u8>,
}

#[test]
fn test_slow4() {
    let mut data = TestCase {
        input: "5468697320697320612074657374205468697320697320612074657374205468697320697320612074657374".from_hex().unwrap(),
        output: "56bbeaee6ff36e4cd22a3bef0458c57d1bce74f392b5dac62da1bc2c20fabe94".from_hex().unwrap(),
    };
    let hash = cryptonight_r(&data.input[..], data.input.len());
    data.output.reverse();
    assert_eq!(hash, data.output);
}

// add a test for amd ryzen cpu
// add -Ofast in build.rs, this test will failed on amd ryzen cpu.
#[test]
fn test_amd_ryzen() {
    let data = TestCase {
        input: "5f9f9874ec4def414b963687c4b17e00377150cc175f5c6d6e4182c6cb4378550000000000000079de00c0000000000000000000000000000000000000000000000000000000000000b1ec37".from_hex().unwrap(),
        output: "0000008e67af69c3c670dab325e8fc3a9dc045c6e339aa35f43b415604513742".from_hex().unwrap(),
    };
    let hash = cryptonight_r(&data.input[..], data.input.len());
    assert_eq!(hash, data.output);
}

#[test]
#[ignore]
fn test_with_spawn() {
    use super::*;
    let input = [0x1; 76];
    loop {
        std::thread::spawn(move || {
            cryptonight_r(&input, input.len());
        });
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
