use super::{memory_kib, memory_label, processor};

#[test]
fn processador_vem_do_modelo_sem_usar_o_indice_do_nucleo() {
    let input = "processor : 0\nmodel name : AMD Ryzen 5 5500\nprocessor : 1\nmodel name : AMD Ryzen 5 5500\n";
    assert_eq!(processor(input).as_deref(), Some("AMD Ryzen 5 5500"));
}

#[test]
fn aceita_os_campos_de_modelo_de_outras_arquiteturas() {
    assert_eq!(
        processor("Processor : ARMv8 Processor\nHardware : placa\n").as_deref(),
        Some("ARMv8 Processor")
    );
    assert_eq!(
        processor("Hardware : SoC de teste\n").as_deref(),
        Some("SoC de teste")
    );
    assert_eq!(
        processor("cpu model : modelo alternativo\n").as_deref(),
        Some("modelo alternativo")
    );
}

#[test]
fn processador_ausente_ou_vazio_nao_e_inventado() {
    assert_eq!(processor("processor : 0\nmodel name : \n"), None);
    assert_eq!(processor(""), None);
    assert_eq!(
        processor("model name : \nmodel name : CPU válida\n").as_deref(),
        Some("CPU válida")
    );
}

#[test]
fn ram_usa_memoria_total_em_vez_da_memoria_livre() {
    assert_eq!(
        memory_kib("MemFree: 1024 kB\nMemTotal: 16150580 kB\nMemAvailable: 2048 kB\n"),
        Some(16_150_580)
    );
}

#[test]
fn memoria_invalida_ou_com_unidade_desconhecida_fica_ausente() {
    for input in [
        "",
        "MemFree: 1 kB",
        "MemTotal: 0 kB",
        "MemTotal: -1 kB",
        "MemTotal: 12 MB",
        "MemTotal: 12",
        "MemTotal: 12 kB extra",
        "MemTotal: 18446744073709551616 kB",
    ] {
        assert_eq!(memory_kib(input), None, "{input}");
    }
}

#[test]
fn ram_e_formatada_em_gib_com_uma_casa_decimal() {
    assert_eq!(memory_label(16 * 1_048_576), "16,0 GiB");
    assert_eq!(memory_label(16_150_580), "15,4 GiB");
    assert_eq!(memory_label(524_288), "0,5 GiB");
}
