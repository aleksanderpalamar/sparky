use std::sync::Arc;
use std::time::Instant;

use super::Sampler;
use super::fake_system::{Arvore, Discos};
use crate::domain::gpu::DrmClient;

fn sistema() -> Arvore {
    Arvore::default()
        .com("/proc/stat", "cpu  10 0 10 80 0 0 0 0 0 0\ncpu0 10 0 10 80 0 0 0 0 0 0\n")
        .com("/proc/cpuinfo", "model name\t: AMD Ryzen 5 5500\n")
        .com("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq", "3600000\n")
        .com("/sys/devices/system/cpu/cpu1/cpufreq/scaling_cur_freq", "2400000\n")
        .com("/sys/devices/system/cpu/cpufreq/policy0", "x")
        .com("/sys/class/hwmon/hwmon0/name", "nvme\n")
        .com("/sys/class/hwmon/hwmon0/temp1_input", "30000\n")
        .com("/sys/class/hwmon/hwmon2/name", "k10temp\n")
        .com("/sys/class/hwmon/hwmon2/temp1_input", "47125\n")
        .com("/proc/meminfo", "MemTotal: 100 kB\nMemAvailable: 40 kB\n")
        .com("/proc/net/dev", "h1\nh2\neth0: 10 0 0 0 0 0 0 0 20 0 0 0 0 0 0 0\n")
        .com("/sys/block/nvme0n1/device/model", "KINGSTON SNV2S500G   \n")
        .com("/sys/block/nvme0n1/device/hwmon0/temp1_input", "34800\n")
        .com("/sys/block/sda/device/model", "SATA SSD\n")
        .com("/sys/block/zram0/size", "1")
        .com("/sys/block/dm-0/slaves/sda2", "")
        .com("/proc/diskstats", "259 0 nvme0n1 1 0 10 0 1 0 20 0\n8 0 sda 1 0 2 0 1 0 4 0\n253 0 zram0 1 0 9 0 1 0 9 0\n")
        .com("/proc/mounts", "/dev/nvme0n1p2 / btrfs rw 0 0\n/dev/nvme0n1p2 /home btrfs rw 0 0\n/dev/mapper/dados /dados ext4 rw 0 0\ntmpfs /tmp tmpfs rw 0 0\n")
        .link("/dev/mapper/dados", "../dm-0")
        .com("/sys/class/drm/card1/device/vendor", "0x1002\n")
        .com("/sys/class/drm/card1/device/gpu_busy_percent", "7\n")
        .com("/sys/class/drm/card1/device/mem_info_vram_used", "1024\n")
        .com("/sys/class/drm/card1-DP-1/status", "connected\n")
}

fn processos(arvore: Arvore) -> Arvore {
    let fdinfo = "drm-client-id:\t14\ndrm-engine-gfx:\t500 ns\ndrm-resident-vram:\t2 KiB\n";
    arvore
        .com(
            "/proc/2/stat",
            "2 (kthreadd) S 0 0 0 0 -1 0 0 0 0 0 0 0 0 0",
        )
        .com(
            "/proc/33/stat",
            "33 (kworker/0:1) I 2 0 0 0 -1 0 0 0 0 0 0 0 0 0",
        )
        .com(
            "/proc/953/stat",
            "953 (sparky) S 1 0 0 0 -1 0 0 0 0 0 30 20 0 0",
        )
        .com("/proc/953/cmdline", "/usr/bin/sparky\0")
        .com("/proc/953/statm", "100 25 0 0 0 0 0")
        .com("/proc/953/status", "Uid:\t1000\t1000\t1000\t1000\n")
        .com("/proc/953/io", "read_bytes: 7\nwrite_bytes: 3\n")
        .link("/proc/953/fd/19", "/dev/dri/card1")
        .link("/proc/953/fd/20", "/dev/dri/card1")
        .link("/proc/953/fd/3", "/home/palamar/.config/x")
        .com("/proc/953/fdinfo/19", fdinfo)
        .com("/proc/953/fdinfo/20", fdinfo)
        .com(
            "/proc/700/stat",
            "700 (avahi-daemon) S 1 0 0 0 -1 0 0 0 0 0 1 1 0 0",
        )
        .com("/proc/700/cmdline", "avahi-daemon: running\0")
        .com("/proc/self/stat", "lixo")
}

fn amostrar(arvore: Arvore) -> crate::domain::snapshot::Snapshot {
    Sampler::new(Arc::new(arvore), Arc::new(Discos), 4096).sample(Instant::now())
}

#[test]
fn cpu_memoria_e_rede_vem_dos_arquivos_do_kernel() {
    let retrato = amostrar(sistema());
    assert_eq!((retrato.cpu.busy, retrato.cpu.total), (20, 100));
    assert_eq!(retrato.cores.len(), 1);
    assert_eq!(retrato.cpu_info.model.as_deref(), Some("AMD Ryzen 5 5500"));
    assert_eq!(retrato.cpu_info.frequency, Some(3000));
    assert_eq!(retrato.cpu_info.temperature, Some(47.125));
    assert_eq!(retrato.memory.map(|memory| memory.used()), Some(60 * 1024));
    assert_eq!(retrato.networks[0].sent, 20);
}

#[test]
fn discos_somam_suas_montagens_inclusive_pelo_mapper() {
    let retrato = amostrar(sistema());
    let discos: Vec<_> = retrato
        .disks
        .iter()
        .map(|disk| {
            (
                disk.counters.name.as_str(),
                disk.model.as_deref(),
                disk.capacity,
                disk.used,
                disk.mounts.clone(),
                disk.temperature,
            )
        })
        .collect();
    assert_eq!(
        discos,
        [
            (
                "nvme0n1",
                Some("KINGSTON SNV2S500G"),
                1000,
                400,
                vec!["/".to_owned()],
                Some(34.8)
            ),
            (
                "sda",
                Some("SATA SSD"),
                500,
                100,
                vec!["/dados".to_owned()],
                None
            ),
        ]
    );
}

#[test]
fn gpu_e_processos_sem_threads_do_kernel() {
    let retrato = amostrar(processos(sistema()));
    assert_eq!(retrato.gpus.len(), 1);
    assert_eq!(retrato.gpus[0].name, "AMD");
    assert_eq!(retrato.gpus[0].busy, Some(7.0));
    let mut pids: Vec<_> = retrato
        .processes
        .iter()
        .map(|process| process.stat.pid)
        .collect();
    pids.sort();
    assert_eq!(pids, [700, 953]);
    let sparky = retrato
        .processes
        .iter()
        .find(|process| process.stat.pid == 953)
        .unwrap();
    assert_eq!(sparky.memory, 25 * 4096);
    assert_eq!(sparky.user, Some(1000));
    assert_eq!(sparky.io, Some((7, 3)));
    assert_eq!(
        sparky.gpu,
        [DrmClient {
            id: 14,
            engine_ns: 500,
            vram: 2048
        }]
    );
    let avahi = retrato
        .processes
        .iter()
        .find(|process| process.stat.pid == 700)
        .unwrap();
    assert_eq!((avahi.io, avahi.user), (None, None));
    assert!(avahi.gpu.is_empty());
    assert_eq!(avahi.program, "avahi-daemon");
}

#[test]
fn lvm_sobre_luks_chega_ao_disco_fisico_descendo_os_mapeamentos() {
    let arvore = Arvore::default()
        .com("/sys/block/nvme0n1/device/model", "KINGSTON\n")
        .com("/sys/block/dm-1/slaves/dm-0", "")
        .com("/sys/block/dm-0/slaves/nvme0n1p3", "")
        .com("/proc/diskstats", "259 0 nvme0n1 1 0 10 0 1 0 20 0\n")
        .com("/proc/mounts", "/dev/mapper/raiz / ext4 rw 0 0\n")
        .link("/dev/mapper/raiz", "../dm-1");
    let retrato = amostrar(arvore);
    assert_eq!(retrato.disks.len(), 1);
    assert_eq!(retrato.disks[0].mounts, ["/"]);
    assert_eq!(
        (retrato.disks[0].capacity, retrato.disks[0].used),
        (1000, 400)
    );
}
