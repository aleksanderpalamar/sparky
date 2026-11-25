use sparky_hardware::stat::CpuTimes;

use super::collector::Collector;
use super::fake_files::FakeFiles;
use super::sample::{GpuSample, MemorySample, Vram};
use crate::domain::Reading;

const PSI: &str = "some avg10=2.00 avg60=1.00 avg300=0.50 total=10\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=1\n";
const MEMINFO: &str = "MemTotal:       16000 kB\nMemAvailable:    8000 kB\nSwapTotal:        4000 kB\nSwapFree:         3000 kB\n";

fn machine() -> FakeFiles {
    FakeFiles::default()
        .with(
            "/proc/stat",
            "cpu  10 0 10 70 10 0 0 0\ncpu0 5 0 5 35 5 0 0 0\ncpu1 5 0 5 35 5 0 0 0\n",
        )
        .with("/proc/meminfo", MEMINFO)
        .with("/proc/pressure/cpu", PSI)
        .with("/proc/pressure/memory", PSI)
        .with("/proc/pressure/io", PSI)
        .with("/sys/class/drm/card0-DP-1/status", "connected")
        .with("/sys/class/drm/card1/device/gpu_busy_percent", "37\n")
        .with("/sys/class/drm/card1/device/mem_info_vram_used", "1024\n")
        .with("/sys/class/drm/card1/device/mem_info_vram_total", "8192\n")
}

#[test]
fn le_cpu_memoria_pressao_e_gpu_amd() {
    let sample = Collector::new(machine()).system();
    assert_eq!(
        sample.cpu.measured(),
        Some(&CpuTimes {
            busy: 20,
            total: 100
        })
    );
    assert_eq!(sample.cores.len(), 2);
    assert_eq!(
        sample.memory.measured(),
        Some(&MemorySample {
            total_kib: 16_000,
            available_kib: 8_000,
            swap_total_kib: 4_000,
            swap_free_kib: 3_000
        })
    );
    assert_eq!(
        sample.pressure.memory.measured().map(|psi| psi.some.avg10),
        Some(2.0)
    );
    assert_eq!(
        sample.gpu.measured(),
        Some(&GpuSample {
            busy_percent: Some(37),
            vram: Some(Vram {
                used: 1024,
                total: 8192
            })
        })
    );
}

#[test]
fn fonte_ausente_vira_dado_ausente_sem_derrubar_o_resto() {
    let files = FakeFiles::default().with("/proc/meminfo", MEMINFO);
    let sample = Collector::new(files).system();
    assert_eq!(sample.cpu, Reading::Missing);
    assert!(sample.cores.is_empty());
    assert_eq!(sample.pressure.io, Reading::Missing);
    assert_eq!(sample.gpu, Reading::Missing);
    assert!(sample.memory.measured().is_some());
}

#[test]
fn gpu_sem_metricas_de_uso_fica_ausente() {
    let files = FakeFiles::default().with("/sys/class/drm/card0/device/vendor", "0x10de\n");
    assert_eq!(Collector::new(files).system().gpu, Reading::Missing);
}

#[test]
fn texto_corrompido_nao_vira_medicao() {
    let files = machine()
        .with("/proc/pressure/cpu", "lixo")
        .with("/sys/class/drm/card1/device/gpu_busy_percent", "muito\n");
    let sample = Collector::new(files).system();
    assert_eq!(sample.pressure.cpu, Reading::Missing);
    assert_eq!(
        sample.gpu.measured().map(|gpu| gpu.busy_percent),
        Some(None)
    );
}
