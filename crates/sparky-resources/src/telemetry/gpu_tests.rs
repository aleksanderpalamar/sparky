use std::cell::RefCell;
use std::io;
use std::path::Path;

use super::collector::Collector;
use super::fake_files::FakeFiles;
use super::sample::{GpuSample, Vram};
use crate::ports::Files;

const DEVICE: &str = "/sys/class/drm/card1/device";

struct Live(RefCell<FakeFiles>);

impl Files for &Live {
    fn read(&self, path: &Path) -> io::Result<String> {
        self.0.borrow().read(path)
    }

    fn entries(&self, directory: &Path) -> io::Result<Vec<String>> {
        self.0.borrow().entries(directory)
    }
}

fn card(busy: &str) -> FakeFiles {
    FakeFiles::default().with(&format!("{DEVICE}/gpu_busy_percent"), busy)
}

fn with_vram(files: FakeFiles) -> FakeFiles {
    files
        .with(&format!("{DEVICE}/mem_info_vram_used"), "1024\n")
        .with(&format!("{DEVICE}/mem_info_vram_total"), "8192\n")
}

#[test]
fn uso_sem_contadores_de_vram_continua_medido() {
    let gpu = Collector::new(card("42\n")).system().gpu;
    assert_eq!(
        gpu.measured(),
        Some(&GpuSample {
            busy_percent: Some(42),
            vram: None
        })
    );
}

#[test]
fn vram_ilegivel_nao_derruba_o_uso_e_volta_depois() {
    let live = Live(RefCell::new(
        card("42\n").with(&format!("{DEVICE}/mem_info_vram_used"), "lixo\n"),
    ));
    let collector = Collector::new(&live);
    let partial = collector.system().gpu;
    assert_eq!(
        partial.measured().map(|gpu| (gpu.busy_percent, gpu.vram)),
        Some((Some(42), None))
    );
    *live.0.borrow_mut() = with_vram(card("42\n"));
    assert_eq!(
        collector.system().gpu.measured().and_then(|gpu| gpu.vram),
        Some(Vram {
            used: 1024,
            total: 8192
        })
    );
}

#[test]
fn uso_zero_e_medicao_valida() {
    let gpu = Collector::new(with_vram(card("0\n"))).system().gpu;
    assert_eq!(gpu.measured().and_then(|gpu| gpu.busy_percent), Some(0));
}

#[test]
fn uso_ilegivel_com_vram_mantem_a_vram() {
    let gpu = Collector::new(with_vram(card("muito\n"))).system().gpu;
    assert_eq!(
        gpu.measured(),
        Some(&GpuSample {
            busy_percent: None,
            vram: Some(Vram {
                used: 1024,
                total: 8192
            })
        })
    );
}
