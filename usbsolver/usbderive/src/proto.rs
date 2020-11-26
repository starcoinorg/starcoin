use crate::{constants::*, proto_msg};
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryInto;
use std::io::Cursor;
use std::time::{Duration, SystemTime};

pub struct Message;

impl Message {
    pub fn opcode_msg() -> Vec<u8> {
        let mut pktlen = vec![];
        pktlen
            .write_u32::<LittleEndian>(OPCODE_DD as u32 * 2 + 12)
            .unwrap();
        let mut height = vec![];
        height.write_u32::<LittleEndian>(OPCODE_HEIGHT).unwrap();
        proto_msg!(
            PKT_HEADER,
            [TYPE_SEND_OPCODE],
            [PV],
            pktlen,
            height,
            [OPCODE_RR],
            [OPCODE_DD],
            OPCODE,
            PKT_ENDER
        )
    }
    pub fn reboot_msg() -> Vec<u8> {
        proto_msg!(
            PKT_HEADER,
            [TYPE_REBOOT],
            [PV],
            [0x6, 0x0, 0x0, 0x0],
            PKT_ENDER
        )
    }
    pub fn write_job_msg(job_id: u8, target: u32, data: &[u8]) -> Vec<u8> {
        let mut target_b = vec![];
        target_b.write_u32::<LittleEndian>(target).unwrap();

        let mut pktlen = vec![];
        pktlen.write_u32::<LittleEndian>(104).unwrap();

        let start_nonce: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let end_nonce: [u8; 8] = [0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let job_num = [1];
        proto_msg!(
            PKT_HEADER,
            [TYPE_SEND_WORK],
            [PV],
            pktlen,
            target_b,
            start_nonce,
            end_nonce,
            job_num,
            [job_id],
            data,
            PKT_ENDER
        )
    }

    pub fn set_hw_params_msg(freq: u16, voltage: u16) -> Vec<u8> {
        let mut freq_b = vec![];
        let mut voltage_b = vec![];
        let mut varity_b = vec![];
        freq_b.write_u16::<LittleEndian>(freq).unwrap();
        voltage_b.write_u16::<LittleEndian>(voltage).unwrap();
        varity_b.write_u32::<LittleEndian>(ALGO_VARITY).unwrap();
        let pktlen: [u8; 4] = [0x10, 0x00, 0x00, 0x00];
        let flag: [u8; 1] = [0xA2];
        let target_temp: [u8; 1] = [80];
        proto_msg!(
            PKT_HEADER,
            [TYPE_SET_HWPARAMS],
            [PV],
            pktlen,
            flag,
            voltage_b,
            freq_b,
            varity_b,
            target_temp,
            PKT_ENDER
        )
    }

    pub fn get_state_msg() -> Vec<u8> {
        proto_msg!(
            PKT_HEADER,
            [TYPE_SET_HWPARAMS],
            [PV],
            [0x7, 0x0, 0x0, 0x0],
            [0x52],
            PKT_ENDER
        )
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub chips: u8,
    pub cores: u8,
    pub goodcores: u8,
    pub scanbits: u8,
    pub scantime: u16,
    pub voltage: u16,
    pub freq: u16,
    pub varity: u32,
    pub temp: u8,
    pub hwreboot: u8,
    pub tempwarn: u8,
    pub latest_updated: Duration,
}

impl State {
    pub fn new(raw_data: &[u8]) -> Result<Self> {
        if raw_data.len() < 25 {
            return Err(anyhow::anyhow!("Invalid raw data len less than 25"));
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System time is before the UNIX_EPOCH");
        let mut data = Cursor::new(&raw_data[13..]);

        Ok(Self {
            chips: raw_data[9],
            cores: raw_data[10],
            goodcores: raw_data[11],
            scanbits: raw_data[12],
            scantime: data.read_u16::<LittleEndian>()?,
            voltage: data.read_u16::<LittleEndian>()?,
            freq: data.read_u16::<LittleEndian>()?,
            varity: data.read_u32::<LittleEndian>()?,
            temp: raw_data[23],
            hwreboot: raw_data[24],
            tempwarn: raw_data[25],
            latest_updated: now,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Seal {
    pub job_id: u8,
    pub nonce: u32,
    pub hash: [u8; 32],
}

impl Seal {
    pub fn new(job_id: u8, nonce: u32, hash: [u8; 32]) -> Self {
        Self {
            job_id,
            nonce,
            hash,
        }
    }
}

#[derive(Debug)]
pub enum DeriveResponse {
    // job_id, nonce, hash
    SolvedJob(Seal),
    State(State),
    Others(Vec<u8>),
}

impl DeriveResponse {
    pub fn new(raw_data: Vec<u8>) -> Result<Self> {
        let location = raw_data
            .windows(PKT_HEADER.len())
            .position(|w| w == PKT_HEADER)
            .ok_or(anyhow::anyhow!("Receive Invalid PKT"))?;
        let type_location = location + PKT_HEADER.len() + TYPE_OFFSET;
        let data_type = &raw_data[type_location];

        let received = match data_type {
            &TYPE_RECV_STATE => {
                let state = State::new(&raw_data)?;
                DeriveResponse::State(state)
            }
            &TYPE_RECV_NONCE => {
                if raw_data.len() < 53 {
                    DeriveResponse::Others(raw_data.clone())
                } else {
                    let mut hash: [u8; 32] = raw_data[21..53].try_into()?;
                    hash.reverse();
                    let job_id = raw_data[9];
                    let nonce = Cursor::new(&raw_data[12..]).read_u32::<LittleEndian>()?;
                    DeriveResponse::SolvedJob(Seal::new(job_id, nonce, hash))
                }
            }
            _ => DeriveResponse::Others(raw_data),
        };
        Ok(received)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_hw_msg() {
        let msg = Message::set_hw_params_msg(600, 750);
        // freq:600, voltage:750, varity:4
        let expect_msg: [u8; 22] = [
            0xa5, 0x3c, 0x96, 0xa2, 0x10, 0x10, 0x00, 0x00, 0x00, 0xa2, 0xee, 0x02, 0x58, 0x02,
            0x04, 0x00, 0x00, 0x00, 0x50, 0x69, 0xc3, 0x5a,
        ];
        assert_eq!(msg, expect_msg);
    }

    #[test]
    fn test_get_state_msg() {
        let msg = Message::get_state_msg();
        let expect_msg: [u8; 13] = [
            0xa5, 0x3c, 0x96, 0xa2, 0x10, 0x07, 0x00, 0x00, 0x00, 0x52, 0x69, 0xc3, 0x5a,
        ];
        assert_eq!(expect_msg, msg.as_slice());
    }
}
