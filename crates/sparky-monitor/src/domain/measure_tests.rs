use std::time::{Duration, Instant};

use super::cpu::CpuTimes;
use super::disk::DiskCounters;
use super::gpu::GpuUse;
use super::measure::{measure, per_second};
use super::network::NetCounters;
use super::process::ProcStat;
use super::snapshot::{CpuInfo, DiskSample, ProcessSample, Snapshot};

fn processo(pid: u32, ticks: u64, io: u64, engine_ns: u64) -> ProcessSample {
    ProcessSample {
        stat: ProcStat {
            pid,
            comm: format!("p{pid}"),
            ppid: 1,
            ticks,
        },
        name: format!("p{pid}"),
        program: format!("p{pid}"),
        user: Some(1000),
        memory: 4096,
        io: Some((io, 0)),
        gpu: Some(GpuUse {
            engine_ns,
            vram: 1024,
        }),
    }
}

fn retrato(
    at: Instant,
    busy: u64,
    total: u64,
    bytes: u64,
    processos: Vec<ProcessSample>,
) -> Snapshot {
    Snapshot {
        at,
        cpu: CpuTimes { busy, total },
        cores: vec![CpuTimes { busy, total }],
        cpu_info: CpuInfo::default(),
        memory: None,
        disks: vec![DiskSample {
            counters: DiskCounters {
                name: "nvme0n1".to_owned(),
                read: bytes,
                written: bytes * 2,
            },
            model: None,
            capacity: 0,
            used: 0,
            mounts: Vec::new(),
            temperature: None,
        }],
        networks: vec![NetCounters {
            name: "enp7s0".to_owned(),
            received: bytes,
            sent: 0,
        }],
        gpus: Vec::new(),
        processes: processos,
    }
}

#[test]
fn taxas_do_sistema_saem_da_diferenca_entre_duas_leituras() {
    let inicio = Instant::now();
    let antes = retrato(inicio, 100, 1000, 0, Vec::new());
    let depois = retrato(inicio + Duration::from_secs(2), 150, 1200, 4000, Vec::new());
    let leitura = measure(&antes, &depois);
    assert_eq!(leitura.cpu, 25.0);
    assert_eq!(leitura.cores, [25.0]);
    assert_eq!(
        (leitura.disks[0].read, leitura.disks[0].write),
        (2000.0, 4000.0)
    );
    assert_eq!(leitura.networks[0].received, 2000.0);
}

#[test]
fn processos_usam_fatia_da_maquina_e_novos_comecam_em_zero() {
    let inicio = Instant::now();
    let antes = retrato(inicio, 0, 1000, 0, vec![processo(10, 100, 0, 0)]);
    let depois = retrato(
        inicio + Duration::from_secs(1),
        0,
        1200,
        0,
        vec![
            processo(10, 150, 3000, 250_000_000),
            processo(11, 900, 500, 0),
        ],
    );
    let leitura = measure(&antes, &depois);
    let p10 = &leitura.processes[0].usage;
    assert_eq!(p10.cpu, 25.0);
    assert_eq!(p10.disk, 3000.0);
    assert_eq!(p10.gpu, Some(25.0));
    assert_eq!(p10.vram, Some(1024));
    let p11 = &leitura.processes[1].usage;
    assert_eq!((p11.cpu, p11.disk, p11.gpu), (0.0, 0.0, Some(0.0)));
}

#[test]
fn contador_que_volta_ou_tempo_zero_nao_gera_taxa_negativa() {
    assert_eq!(per_second(500, 100, Duration::from_secs(1)), 0.0);
    assert_eq!(per_second(0, 100, Duration::ZERO), 0.0);
}
