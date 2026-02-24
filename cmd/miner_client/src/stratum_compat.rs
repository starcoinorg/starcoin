use serde::{Deserialize, Serialize};
use starcoin_types::block::BlockHeaderExtra;
use starcoin_types::U256;
use std::{convert::TryInto, io, str};
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

const MAX_INBOUND_BYTES: usize = 256 * 1024;

pub fn target_hex_to_difficulty(target: &str) -> anyhow::Result<U256> {
    let mut temp = hex::decode(target)?;
    temp.reverse();
    let temp = hex::encode(temp);
    let temp = U256::from_str_radix(&temp, 16)?;
    Ok(U256::from(u64::MAX) / temp)
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct LoginRequest {
    pub login: String,
    pub pass: String,
    pub agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algo: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub id: String,
    pub job_id: String,
    pub nonce: String,
    pub result: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Status {
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StratumJobResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<LoginRequest>,
    pub id: String,
    pub status: String,
    pub job: StratumJob,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct StratumJob {
    pub height: u64,
    pub id: String,
    pub target: String,
    pub job_id: String,
    pub blob: String,
}

impl StratumJob {
    pub fn get_extra(&self) -> anyhow::Result<BlockHeaderExtra> {
        let blob = hex::decode(&self.blob)?;
        if blob.len() != 76 {
            return Err(anyhow::anyhow!("Invalid stratum job"));
        }
        let extra: [u8; 4] = blob[35..39].try_into()?;
        Ok(BlockHeaderExtra::new(extra))
    }
}

#[derive(Debug, Clone)]
pub enum Separator {
    Empty,
    Byte(u8),
}

impl Default for Separator {
    fn default() -> Self {
        Separator::Byte(b'\n')
    }
}

#[derive(Debug, Default)]
pub struct JsonStreamCodec {
    incoming_separator: Separator,
    outgoing_separator: Separator,
}

impl JsonStreamCodec {
    pub fn stream_incoming() -> Self {
        Self::new(Separator::Empty, Default::default())
    }

    pub fn new(incoming_separator: Separator, outgoing_separator: Separator) -> Self {
        Self {
            incoming_separator,
            outgoing_separator,
        }
    }
}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, 0x0D | 0x0A | 0x20 | 0x09)
}

impl Decoder for JsonStreamCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if buf.len() > MAX_INBOUND_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "jsonrpc message too large",
            ));
        }
        if let Separator::Byte(separator) = self.incoming_separator {
            if let Some(i) = buf.as_ref().iter().position(|&b| b == separator) {
                let line = buf.split_to(i);
                let _ = buf.split_to(1);

                match str::from_utf8(line.as_ref()) {
                    Ok(s) => Ok(Some(s.to_string())),
                    Err(_) => Err(io::Error::other("invalid UTF-8")),
                }
            } else {
                Ok(None)
            }
        } else {
            let mut depth = 0;
            let mut in_str = false;
            let mut is_escaped = false;
            let mut start_idx = 0;
            let mut whitespaces = 0;

            for idx in 0..buf.as_ref().len() {
                let byte = buf.as_ref()[idx];

                if (byte == b'{' || byte == b'[') && !in_str {
                    if depth == 0 {
                        start_idx = idx;
                    }
                    depth += 1;
                } else if (byte == b'}' || byte == b']') && !in_str {
                    depth -= 1;
                } else if byte == b'"' && !is_escaped {
                    in_str = !in_str;
                } else if is_whitespace(byte) {
                    whitespaces += 1;
                }
                is_escaped = byte == b'\\' && !is_escaped && in_str;

                if depth == 0 && idx != start_idx && idx - start_idx + 1 > whitespaces {
                    let bts = buf.split_to(idx + 1);
                    match String::from_utf8(bts.as_ref().to_vec()) {
                        Ok(val) => return Ok(Some(val)),
                        Err(_) => return Ok(None),
                    };
                }
            }
            Ok(None)
        }
    }
}

impl Encoder<String> for JsonStreamCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        let mut payload = msg.into_bytes();
        if let Separator::Byte(separator) = self.outgoing_separator {
            payload.push(separator);
        }
        buf.extend_from_slice(&payload);
        Ok(())
    }
}
