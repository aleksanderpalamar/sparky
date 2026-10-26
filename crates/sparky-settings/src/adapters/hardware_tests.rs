use std::fs;
use std::path::Path;

use super::from_paths;
use crate::domain::hardware::Hardware;

fn device(root: &Path, name: &str, class: &str, vendor: &str, model: &str) {
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    for (name, contents) in [("class", class), ("vendor", vendor), ("device", model)] {
        fs::write(path.join(name), contents).unwrap();
    }
}

#[test]
fn consulta_apenas_arquivos_de_hardware_sem_precisar_de_comandos_ou_gpu() {
    let root = tempfile::tempdir().unwrap();
    let proc = root.path().join("proc");
    let pci = root.path().join("pci");
    fs::create_dir(&proc).unwrap();
    fs::write(proc.join("cpuinfo"), "model name : CPU de teste\n").unwrap();
    fs::write(proc.join("meminfo"), "MemTotal: 16777216 kB\n").unwrap();
    device(&pci, "0000:03:00.0", "0x030000", "0x1002", "0x73ff");
    let about = from_paths(&proc, &pci, "1002  AMD\n\t73ff  Radeon de teste\n");
    assert_eq!(
        about,
        Hardware {
            processor: Some("CPU de teste".into()),
            gpus: vec!["AMD Radeon de teste".into()],
            memory_kib: Some(16_777_216),
        }
    );
}

#[test]
fn mostra_varias_gpus_em_ordem_estavel_e_ignora_controladores_de_rede() {
    let root = tempfile::tempdir().unwrap();
    device(root.path(), "0000:03:00.0", "0x030200", "0x10de", "0x1234");
    device(root.path(), "0000:00:02.0", "0x030000", "0x8086", "0xabcd");
    device(root.path(), "0000:01:00.0", "0x020000", "0x8086", "0x9999");
    let about = from_paths(&root.path().join("sem-proc"), root.path(), "");
    assert_eq!(about.gpus, ["GPU PCI 8086:abcd", "GPU PCI 10de:1234"]);
}

#[test]
fn nome_fornecido_pelo_driver_tem_prioridade_sobre_a_base_pci() {
    let root = tempfile::tempdir().unwrap();
    device(root.path(), "gpu", "0x038000", "0x1002", "0x73ff");
    fs::write(
        root.path().join("gpu/product_name"),
        "  AMD Radeon RX 6600\n",
    )
    .unwrap();
    let about = from_paths(&root.path().join("sem-proc"), root.path(), "");
    assert_eq!(about.gpus, ["AMD Radeon RX 6600"]);
}

#[test]
fn atributos_invalidos_nao_fabricam_gpus() {
    let root = tempfile::tempdir().unwrap();
    device(root.path(), "sem-vendor", "0x030000", "inválido", "0x73ff");
    device(root.path(), "sem-classe", "inválido", "0x1002", "0x73ff");
    device(
        root.path(),
        "vendor-grande",
        "0x030000",
        "0x10002",
        "0x73ff",
    );
    let about = from_paths(root.path(), root.path(), "");
    assert_eq!(about, Hardware::default());
}

#[test]
fn ausencia_de_proc_e_sysfs_e_tratada_sem_falha() {
    let root = tempfile::tempdir().unwrap();
    let absent = root.path().join("ausente");
    assert_eq!(from_paths(&absent, &absent, ""), Hardware::default());
}
