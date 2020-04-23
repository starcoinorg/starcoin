use prometheus::core::{Collector, Desc, Opts};
use prometheus::proto;
use prometheus::Gauge;
use std::sync::Mutex;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
#[derive(Debug)]
pub struct ProcessCollector {
    pid: Pid,
    descs: Vec<Desc>,
    system: Mutex<System>,
    cpu_usage: Gauge,
    vsize: Gauge,
    rss: Gauge,
}

impl ProcessCollector {
    pub fn for_self() -> Self {
        let pid = std::process::id() as i32;
        Self::new(pid, "".to_string())
    }
    pub fn new(pid: i32, namespace: String) -> ProcessCollector {
        let vsize = Gauge::with_opts(
            Opts::new(
                "process_virtual_memory_bytes",
                "Virtual memory size in bytes.",
            )
            .namespace(namespace.clone()),
        )
        .unwrap();
        let rss = Gauge::with_opts(
            Opts::new(
                "process_resident_memory_bytes",
                "Resident memory size in bytes.",
            )
            .namespace(namespace.clone()),
        )
        .unwrap();
        let cpu_usage = Gauge::with_opts(
            Opts::new("process_cpu_usage", "Total user and system CPU time")
                .namespace(namespace.clone()),
        )
        .unwrap();
        let mut descs = vec![];
        descs.extend(vsize.desc().into_iter().cloned());
        descs.extend(rss.desc().into_iter().cloned());
        descs.extend(cpu_usage.desc().into_iter().cloned());
        ProcessCollector {
            pid,
            descs,
            system: Mutex::new(System::new()),
            cpu_usage,
            vsize,
            rss,
        }
    }
}

impl Collector for ProcessCollector {
    fn desc(&self) -> Vec<&Desc> {
        self.descs.iter().collect()
    }

    fn collect(&self) -> Vec<proto::MetricFamily> {
        let mut system = self.system.lock().unwrap();
        system.refresh_process(self.pid);
        let process_info = system.get_process(self.pid);

        match process_info {
            None => vec![],
            Some(process_info) => {
                let memory_usage = process_info.memory() * 1024;
                let virtual_mem = process_info.virtual_memory() * 1024;
                let cpu_usage = process_info.cpu_usage();
                self.rss.set(memory_usage as f64);
                self.vsize.set(virtual_mem as f64);
                self.cpu_usage.set(cpu_usage as f64);
                let mut mfs = Vec::with_capacity(3);
                mfs.extend(self.rss.collect());
                mfs.extend(self.vsize.collect());
                mfs.extend(self.cpu_usage.collect());
                mfs
            }
        }
    }
}
