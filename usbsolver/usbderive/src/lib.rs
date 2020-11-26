#[allow(dead_code)]
mod constants;
pub mod derive;
mod proto;
mod tests;
pub use derive::{Config, UsbDerive};
pub use proto::DeriveResponse;
use std::io;
use std::io::BufRead;

#[macro_export]
macro_rules! proto_msg {
    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::<u8>::new();
            $(
                temp_vec.extend_from_slice($x.clone().as_ref());
            )*
            temp_vec
        }
    };
}

pub(crate) fn read_until(
    buf_reader: &mut dyn BufRead,
    delim: &[u8],
    buf: &mut Vec<u8>,
) -> io::Result<usize> {
    let mut total_n = 0;
    loop {
        let mut tmp_buf = vec![];
        let n = buf_reader.read_until(delim[delim.len() - 1], tmp_buf.as_mut())?;
        total_n += n;
        buf.extend_from_slice(tmp_buf.as_slice());
        if n <= delim.len() {
            break;
        }
        if &tmp_buf[n - delim.len()..] == delim {
            break;
        }
    }
    Ok(total_n)
}

#[test]
fn test_read_until() {
    let mut buf = vec![];
    let n = read_until(&mut io::Cursor::new(b"abcdef"), b"cd", buf.as_mut()).unwrap();
    assert_eq!(4, n);
    assert_eq!(b"abcd".to_vec(), buf);

    let mut buf = vec![];
    let n = read_until(&mut io::Cursor::new(b"abdef"), b"cd", buf.as_mut()).unwrap();
    assert_eq!(5, n);
    assert_eq!(b"abdef".to_vec(), buf);

    let mut buf = vec![];
    let n = read_until(&mut io::Cursor::new(b"a"), b"cd", buf.as_mut()).unwrap();
    assert_eq!(1, n);
    assert_eq!(b"a".to_vec(), buf);
}
