use super::name;

const DATABASE: &str = "# PCI IDs\n1002  AMD\n\t73ff  Radeon RX 6600\n\t\t1234 5678  Variante\n\n8086  Intel\n\t73ff  Outro dispositivo\n\t1234  UHD Graphics\n";

#[test]
fn nome_da_gpu_combina_fabricante_e_modelo_do_mesmo_vendor() {
    assert_eq!(name(DATABASE, 0x1002, 0x73ff), "AMD Radeon RX 6600");
    assert_eq!(name(DATABASE, 0x8086, 0x1234), "Intel UHD Graphics");
}

#[test]
fn ids_desconhecidos_sao_mostrados_sem_inventar_modelos() {
    assert_eq!(name("", 0x1002, 0x73ff), "GPU PCI 1002:73ff");
    assert_eq!(name(DATABASE, 0xffff, 0x1234), "GPU PCI ffff:1234");
}

#[test]
fn modelo_nao_vaza_de_outro_vendor_ou_de_um_subsistema() {
    assert_eq!(name(DATABASE, 0x1002, 0x1234), "AMD [1002:1234]");
}

#[test]
fn entradas_malformadas_e_comentarios_nao_atrapalham_a_busca() {
    let database = "zzzz  inválido\n1002\n1002  AMD\n\tzzzz  inválido\n# comentário\n\tABCD\tModelo de teste\n";
    assert_eq!(name(database, 0x1002, 0xabcd), "AMD Modelo de teste");
}
