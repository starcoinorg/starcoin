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
