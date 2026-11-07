use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::names;

const DATABASE: &str = "1002  AMD\n\t73ff  Radeon RX 6600\n8086  Intel\n\tabcd  UHD Graphics\n";

fn files(entries: &[(&str, &str)]) -> HashMap<PathBuf, String> {
    entries
        .iter()
        .map(|(path, text)| (PathBuf::from(path), (*text).to_owned()))
        .collect()
}

fn found(devices: &[&str], entries: &[(&str, &str)]) -> Vec<String> {
    let files = files(entries);
    let devices: Vec<PathBuf> = devices.iter().map(PathBuf::from).collect();
    names(&devices, DATABASE, |path: &Path| files.get(path).cloned())
}

#[test]
fn gpus_seguem_a_ordem_dos_enderecos_e_ignoram_outras_classes() {
    let gpus = found(
        &["/d/03", "/d/00", "/d/01"],
        &[
            ("/d/03/class", "0x030200\n"),
            ("/d/03/vendor", "0x1002"),
            ("/d/03/device", "0x73ff"),
            ("/d/00/class", "0x030000"),
            ("/d/00/vendor", "0x8086"),
            ("/d/00/device", "0xabcd"),
            ("/d/01/class", "0x020000"),
            ("/d/01/vendor", "0x8086"),
            ("/d/01/device", "0x9999"),
        ],
    );
    assert_eq!(gpus, ["Intel UHD Graphics", "AMD Radeon RX 6600"]);
}

#[test]
fn nome_do_driver_vence_e_atributos_invalidos_sao_ignorados() {
    let gpus = found(
        &["/d/a", "/d/b", "/d/c"],
        &[
            ("/d/a/class", "0x038000"),
            ("/d/a/product_name", "  Placa do driver \n"),
            ("/d/b/class", "inválido"),
            ("/d/c/class", "0x030000"),
            ("/d/c/vendor", "0x10002"),
            ("/d/c/device", "0x73ff"),
        ],
    );
    assert_eq!(gpus, ["Placa do driver"]);
}
