use anyhow::{bail, Result};
use prometheus::core::{Collector, Desc, Opts};
use prometheus::proto;
use prometheus::Gauge;
use psutil::process;
use starcoin_logger::prelude::*;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ProcessCollector {
    pid: u32,
    descs: Vec<Desc>,
    process: Mutex<process::Process>,
    cpu_usage: Gauge,
    vsize: Gauge,
    rss: Gauge,
}

impl ProcessCollector {
    pub fn for_self() -> Result<Self> {
        let pid = std::process::id();
        Self::new(pid)
    }
    pub fn new(pid: u32) -> Result<ProcessCollector> {
        let vsize = Gauge::with_opts(Opts::new(
            "process_virtual_memory_bytes",
            "Virtual memory size in bytes.",
        ))?;
        let rss = Gauge::with_opts(Opts::new(
            "process_resident_memory_bytes",
            "Resident memory size in bytes.",
        ))?;
        let cpu_usage = Gauge::with_opts(Opts::new(
            "process_cpu_usage",
            "Total user and system CPU usage",
        ))?;
        let mut descs = vec![];
        descs.extend(vsize.desc().into_iter().cloned());
        descs.extend(rss.desc().into_iter().cloned());
        descs.extend(cpu_usage.desc().into_iter().cloned());
        let process = process::Process::new(pid);
        let process = match process {
            Err(e) => {
                bail!("fail to collect process info of pid {}, err: {:?}", pid, e);
            }
            Ok(p) => p,
        };

        Ok(ProcessCollector {
            pid,
            descs,
            process: Mutex::new(process),
            cpu_usage,
            vsize,
            rss,
        })
    }
}

impl Collector for ProcessCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let mut process = self.process.lock().expect("lock failed.");

        let mut mfs = Vec::with_capacity(3);

        // let process_info = system.get_process(self.pid);
        match process.memory_info() {
            Err(e) => {
                error!(
                    "fail to collect memory usage of pid {}, err: {:?}",
                    self.pid, e
                );
            }
            Ok(mem_info) => {
                self.rss.set(mem_info.rss() as f64);
                self.vsize.set(mem_info.vms() as f64);
                mfs.extend(self.rss.collect());
                mfs.extend(self.vsize.collect());
            }
        }
        match process.cpu_percent() {
            Err(e) => {
                error!(
                    "fail to collect cpu usage of pid {}, err: {:?}",
                    self.pid, e
                );
            }
            Ok(perent) => {
                self.cpu_usage.set(perent as f64);
                mfs.extend(self.cpu_usage.collect());
            }
        }

        mfs
    }
}
