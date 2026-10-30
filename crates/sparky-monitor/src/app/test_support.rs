use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use iced::Size;
use sparky_theme::Skin;

use super::state::{Monitor, Services};
use crate::domain::apps::AppEntry;
use crate::ports::{Catalog, SignalError, Signals, Stop};
use crate::sampler::Sampler;
use crate::sampler::fake_system::{Arvore, Discos};

pub const JANELA: Size = Size {
    width: 1280.0,
    height: 800.0,
};

#[derive(Default)]
pub struct Sinais {
    pub enviados: Mutex<Vec<(u32, Stop)>>,
    pub negados: Vec<u32>,
}

impl Signals for Sinais {
    fn send(&self, pid: u32, stop: Stop) -> Result<(), SignalError> {
        if self.negados.contains(&pid) {
            return Err(SignalError::Denied);
        }
        if let Ok(mut enviados) = self.enviados.lock() {
            enviados.push((pid, stop));
        }
        Ok(())
    }
}

pub struct Aplicativos;

impl Catalog for Aplicativos {
    fn apps(&self) -> Vec<AppEntry> {
        vec![AppEntry {
            id: "firefox".to_owned(),
            name: "Firefox".to_owned(),
            icon: None,
            program: "firefox".to_owned(),
        }]
    }
}

fn processo(arvore: Arvore, pid: u32, ppid: u32, comm: &str, ticks: u64, pages: u64) -> Arvore {
    arvore
        .com(
            &format!("/proc/{pid}/stat"),
            &format!("{pid} ({comm}) S {ppid} 0 0 0 -1 0 0 0 0 0 {ticks} 0 0 0"),
        )
        .com(
            &format!("/proc/{pid}/cmdline"),
            &format!("/usr/bin/{comm}\0"),
        )
        .com(
            &format!("/proc/{pid}/statm"),
            &format!("0 {pages} 0 0 0 0 0"),
        )
        .com(
            &format!("/proc/{pid}/status"),
            "Uid:\t1000\t1000\t1000\t1000\n",
        )
}

pub fn sistema(cpu_total: u64, ticks: u64) -> Arvore {
    let arvore = Arvore::default()
        .com(
            "/proc/stat",
            &format!("cpu  0 0 0 {cpu_total} 0 0 0 0 0 0\n"),
        )
        .com("/proc/meminfo", "MemTotal: 1000 kB\nMemAvailable: 400 kB\n");
    let arvore = processo(arvore, 1, 0, "systemd", 0, 10);
    let arvore = processo(arvore, 100, 1, "firefox", ticks, 300);
    let arvore = processo(arvore, 101, 100, "Web Content", ticks * 2, 200);
    processo(arvore, 200, 1, "zsh", 0, 50)
}

pub fn monitor(sinais: Arc<Sinais>) -> Monitor {
    let services = Services {
        sampler: Arc::new(Sampler::new(
            Arc::new(sistema(0, 0)),
            Arc::new(Discos),
            4096,
        )),
        catalog: Arc::new(Aplicativos),
        signals: sinais,
        user: 1000,
    };
    let mut monitor = Monitor::new(services, Skin::default(), JANELA);
    monitor.catalog = Aplicativos.apps();
    let inicio = Instant::now();
    let amostrar = |arvore, at| Sampler::new(Arc::new(arvore), Arc::new(Discos), 4096).sample(at);
    monitor.sampled(amostrar(sistema(1000, 0), inicio));
    monitor.sampled(amostrar(
        sistema(2000, 100),
        inicio + Duration::from_secs(1),
    ));
    monitor
}
