//! Shared stream codec for Stratum's raw TCP JSON messages.

pub use starcoin_rpc_ipc::stream_codec::{Separator, StreamCodec as JsonStreamCodec};

#[cfg(test)]
mod tests {
    use super::JsonStreamCodec;
    use bytes::BytesMut;
    use tokio_util::codec::Decoder;

    #[test]
    fn decodes_leading_whitespace_and_short_frames() {
        let mut buf = BytesMut::from(&b"  {}\n\n[]"[..]);
        let mut codec = JsonStreamCodec::stream_incoming();

        let first = codec
            .decode(&mut buf)
            .expect("decode should succeed")
            .expect("first frame");
        let second = codec
            .decode(&mut buf)
            .expect("decode should succeed")
            .expect("second frame");

        assert_eq!(first, "{}");
        assert_eq!(second, "[]");
    }

    #[test]
    fn decodes_multiple_raw_json_messages() {
        let mut buf = BytesMut::from(&b"{\"id\":1}{\"id\":2}"[..]);
        let mut codec = JsonStreamCodec::stream_incoming();

        let first = codec
            .decode(&mut buf)
            .expect("decode should succeed")
            .expect("first frame");
        let second = codec
            .decode(&mut buf)
            .expect("decode should succeed")
            .expect("second frame");

        assert_eq!(first, "{\"id\":1}");
        assert_eq!(second, "{\"id\":2}");
    }
}
