use anyhow::Result;
use byteorder::{BigEndian, ReadBytesExt};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::executor::block_on;
use futures::SinkExt;
use starcoin_logger::prelude::*;
use starcoin_miner_client::{ConsensusStrategy, Solver, U256};
use std::io::Cursor;
use usbderive::{Config, DeriveResponse, UsbDerive};
use std::io;

#[derive(Clone)]
pub struct UsbSolver {
    derive: UsbDerive,
}

const VID: u16 = 1155;
const PID: u16 = 22336;

#[no_mangle]
impl UsbSolver {
    pub fn new() -> Result<Self> {
        let _ = starcoin_logger::init();
        let ports = UsbDerive::detect(VID, PID)?;
        let mut usb_derive: Option<UsbDerive> = None;
        for port in ports {
            match UsbDerive::open(&port.port_name, Config::default()) {
                Ok(derive) => {
                    usb_derive = Some(derive);
                    break;
                }
                Err(e) => {
                    warn!("Failed to open port:{:?}", e);
                    break;
                }
            }
        }
        let mut derive = match usb_derive {
            None => anyhow::bail!("No usb derive found"),
            Some(d) => d,
        };
        derive.set_hw_params()?;
        derive.set_opcode()?;
        info!("Usb solver inited");

        Ok(Self { derive })
    }

    fn difficulty_to_target_u32(difficulty: U256) -> u32 {
        let target = U256::max_value() / difficulty;
        let mut tb = [0u8; 32];
        target.to_big_endian(tb.as_mut());
        let mut data = Cursor::new(tb);
        data.read_u32::<BigEndian>().unwrap()
    }
}

impl Solver for UsbSolver {
    fn solve(
        &mut self,
        _strategy: ConsensusStrategy,
        minting_blob: &[u8],
        diff: U256,
        mut nonce_tx: UnboundedSender<(Vec<u8>, u32)>,
        mut stop_rx: UnboundedReceiver<bool>,
    ) {
        let target = UsbSolver::difficulty_to_target_u32(diff);
        if let Err(e) = self.derive.set_job(0x1, target, minting_blob) {
            error!("Set mint job to derive failed{:?}", e);
            return;
        }
        loop {
            if stop_rx.try_next().is_ok() {
                break;
            }
            // Blocking read since the poll has non-zero timeout
            match self.derive.read() {
                Ok(resp) => match resp {
                    DeriveResponse::SolvedJob(seal) => {
                        block_on(async {
                            let _ = nonce_tx.send((minting_blob.to_owned(), seal.nonce)).await;
                        });
                        break;
                    }
                    resp => {
                        debug!("get resp {:?}", resp);
                        continue;
                    }
                },
                Err(e) => {
                    if let Some(err) = e.downcast_ref::<std::io::Error>() {
                        if err.kind() == io::ErrorKind::TimedOut {
                            continue;
                        }
                    }
                    debug!("Failed to solve: {:?}", e);
                    break;
                }
            }
        }
    }
}
