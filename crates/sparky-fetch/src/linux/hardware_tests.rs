use crate::domain::status::{Reason, Status, Value};
use crate::linux::fake::Fake;
use crate::linux::{cpu, disk, gpu, memory};

const GIB: u64 = 1024 * 1024 * 1024;

fn text(value: &str) -> Result<Value, Status> {
    Ok(Value::Text(value.to_owned()))
}

fn missing() -> Result<Value, Status> {
    Err(Status::unavailable(Reason::Missing))
}

#[test]
fn cpu_mostra_o_modelo_e_os_processadores_logicos() {
    let cpuinfo = "processor : 0\nmodel name : AMD Ryzen 7 5800X\nprocessor : 1\nmodel name : AMD Ryzen 7 5800X\n";
    assert_eq!(
        cpu::describe(cpuinfo).as_deref(),
        Some("AMD Ryzen 7 5800X (2)")
    );
    assert_eq!(cpu::describe("Hardware : SoC\n").as_deref(), Some("SoC"));
    let host = Fake::default().file("/proc/cpuinfo", "processor : 0\n");
    assert_eq!(cpu::collect(&host), missing());
    assert_eq!(cpu::collect(&Fake::default()), missing());
}

#[test]
fn gpus_vem_do_sysfs_com_nomes_da_base_pci() {
    let host = Fake::default()
        .dir("/sys/bus/pci/devices", &["0000:03:00.0", "0000:00:02.0"])
        .file("/sys/bus/pci/devices/0000:03:00.0/class", "0x030000\n")
        .file("/sys/bus/pci/devices/0000:03:00.0/vendor", "0x1002\n")
        .file("/sys/bus/pci/devices/0000:03:00.0/device", "0x73ff\n")
        .file("/sys/bus/pci/devices/0000:00:02.0/class", "0x030000\n")
        .file(
            "/sys/bus/pci/devices/0000:00:02.0/product_name",
            "Intel UHD\n",
        )
        .file(
            "/usr/share/misc/pci.ids",
            "1002  AMD\n\t73ff  Radeon RX 6600\n",
        );
    let gpus = vec!["Intel UHD".to_owned(), "AMD Radeon RX 6600".to_owned()];
    assert_eq!(gpu::collect(&host), Ok(Value::Lines(gpus)));
}

#[test]
fn sem_sysfs_ou_sem_gpu_o_campo_falta() {
    assert_eq!(gpu::collect(&Fake::default()), missing());
    let empty = Fake::default().dir("/sys/bus/pci/devices", &[]);
    assert_eq!(gpu::collect(&empty), missing());
}

#[test]
fn memoria_usada_e_o_total_menos_a_disponivel() {
    let meminfo = "MemTotal: 4194304 kB\nMemFree: 1 kB\nMemAvailable: 3145728 kB\n";
    assert_eq!(
        memory::describe(meminfo).as_deref(),
        Some("1.0 GiB / 4.0 GiB (25%)")
    );
    assert_eq!(memory::describe("MemTotal: 4194304 kB\n"), None);
    let broken = Fake::default().file("/proc/meminfo", "MemTotal: 0 kB\n");
    assert!(matches!(
        memory::collect(&broken),
        Err(Status::Failed { .. })
    ));
}

#[test]
fn disco_mostra_o_uso_da_raiz() {
    let host = Fake::default().disk(400 * GIB, 300 * GIB);
    assert_eq!(disk::collect(&host), text("100.0 GiB / 400.0 GiB (25%)"));
    assert_eq!(disk::collect(&Fake::default().disk(0, 0)), missing());
    assert_eq!(disk::collect(&Fake::default()), missing());
}
