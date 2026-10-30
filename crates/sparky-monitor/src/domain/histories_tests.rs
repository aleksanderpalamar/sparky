use super::disk::DiskCounters;
use super::histories::Histories;
use super::measure::{DiskReading, NetReading, Reading};
use super::memory::Memory;
use super::snapshot::{CpuInfo, DiskSample, GpuDevice};

fn disco(name: &str, read: f64, write: f64) -> DiskReading {
    DiskReading {
        sample: DiskSample {
            counters: DiskCounters {
                name: name.to_owned(),
                read: 0,
                written: 0,
            },
            model: None,
            capacity: 0,
            used: 0,
            mounts: Vec::new(),
            temperature: None,
        },
        read,
        write,
    }
}

fn leitura() -> Reading {
    Reading {
        cpu: 12.5,
        cores: vec![10.0, 15.0],
        cpu_info: CpuInfo::default(),
        memory: Some(Memory {
            total: 100,
            available: 75,
            swap_total: 50,
            swap_free: 25,
        }),
        disks: vec![disco("sda", 100.0, 10.0), disco("nvme0n1", 300.0, 30.0)],
        networks: vec![NetReading {
            name: "eth0".to_owned(),
            received: 500.0,
            sent: 50.0,
        }],
        gpus: vec![GpuDevice {
            name: "AMD".to_owned(),
            busy: Some(40.0),
            vram_used: Some(2),
            vram_total: Some(8),
        }],
        processes: Vec::new(),
    }
}

#[test]
fn cada_leitura_alimenta_os_graficos_totais_e_por_dispositivo() {
    let mut histories = Histories::default();
    histories.record(&leitura());
    histories.record(&leitura());
    assert_eq!(histories.cpu.latest(), Some(12.5));
    assert_eq!(histories.cores.len(), 2);
    assert_eq!(histories.cores[1].latest(), Some(15.0));
    assert_eq!(histories.memory.latest(), Some(25.0));
    assert_eq!(histories.swap.latest(), Some(50.0));
    assert_eq!(histories.disk.first.latest(), Some(400.0));
    assert_eq!(histories.disk.second.latest(), Some(40.0));
    assert_eq!(histories.disks["nvme0n1"].first.values().count(), 2);
    assert_eq!(histories.network.first.latest(), Some(500.0));
    assert_eq!(histories.gpus[0].first.latest(), Some(40.0));
    assert_eq!(histories.gpus[0].second.latest(), Some(25.0));
}
