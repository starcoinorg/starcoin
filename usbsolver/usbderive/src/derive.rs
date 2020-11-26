use crate::constants::*;
use crate::proto::{DeriveResponse, Message, State};
use crate::read_until;
use anyhow::Result;
use serialport::{SerialPort, SerialPortInfo, SerialPortSettings, SerialPortType};
use std::io::BufReader;
use std::io::Write;
use std::time::Duration;
use starcoin_logger::prelude::*;

#[derive(Clone)]
pub struct Config {
    pub target_freq: u16,
    pub target_voltage: u16,
    pub read_timeout: Duration,
    baud_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_freq: 600,
            target_voltage: 750,
            read_timeout: Duration::from_secs(1),
            baud_rate: 115200,
        }
    }
}

pub struct UsbDerive {
    serial_port: Box<dyn SerialPort>,
    config: Config,
}

impl Clone for UsbDerive {
    fn clone(&self) -> Self {
        let serial_port = self
            .serial_port
            .try_clone()
            .expect("serial port should be cloned");
        let config = self.config.clone();
        Self {
            serial_port,
            config,
        }
    }
}

impl UsbDerive {
    pub fn detect(vid: u16, pid: u16) -> Result<Vec<SerialPortInfo>> {
        let ports = serialport::available_ports()?;
        let mut usb_ports = vec![];
        for port in ports {
            info!("detected port with name: {:?}", port.port_name);
            if let SerialPortType::UsbPort(usb_port) = &port.port_type {
                info!("detected usb port: {:?}, {:?}, {:?}", port.port_name, usb_port.vid, usb_port.pid);
                if usb_port.vid == vid && usb_port.pid == pid {
                    usb_ports.push(port);
                }
            }
        }
        Ok(usb_ports)
    }

    pub fn open(path: &str, config: Config) -> Result<Self> {
        let mut setting = SerialPortSettings::default();
        setting.baud_rate = config.baud_rate;
        setting.timeout = config.read_timeout;
        let serial_port = serialport::open_with_settings(path, &setting)?;
        Ok(Self {
            serial_port,
            config,
        })
    }

    pub fn read(&mut self) -> Result<DeriveResponse> {
        let mut raw_resp = vec![];
        let mut port_buf_reader = BufReader::new(&mut self.serial_port);
        read_until(&mut port_buf_reader, &PKT_ENDER, raw_resp.as_mut())?;
        DeriveResponse::new(raw_resp)
    }

    pub fn get_state(&mut self) -> Result<State> {
        let msg = Message::get_state_msg();
        let _ = self.serial_port.write(&msg)?;
        match self.read()? {
            DeriveResponse::State(state) => Ok(state),
            resp => {
                return Err(anyhow::anyhow!("Bad get state resp:{:?}", resp));
            }
        }
    }

    pub fn set_hw_params(&mut self) -> Result<()> {
        let msg = Message::set_hw_params_msg(self.config.target_freq, self.config.target_voltage);
        let _ = self.serial_port.write(&msg)?;
        let _ = self.read();
        Ok(())
    }

    pub fn set_job(&mut self, job_id: u8, target: u32, data: &[u8]) -> Result<()> {
        let msg = Message::write_job_msg(job_id, target, data);
        let _ = self.serial_port.write(&msg)?;
        let _ = self.read();
        Ok(())
    }

    pub fn set_opcode(&mut self) -> Result<()> {
        let msg = Message::opcode_msg();
        let _ = self.serial_port.write(&msg)?;
        let _ = self.read();
        Ok(())
    }

    pub fn reboot(&mut self) -> Result<()> {
        let msg = Message::reboot_msg();
        let _ = self.serial_port.write(&msg)?;
        Ok(())
    }

    pub fn can_open(&mut self) -> bool {
        return match self.get_state() {
            Ok(state) => state.goodcores == 0,
            Err(_) => false,
        };
    }
}
