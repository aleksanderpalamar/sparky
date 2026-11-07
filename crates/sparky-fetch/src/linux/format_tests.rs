use super::format::{bytes, duration, usage};

#[test]
fn bytes_escolhem_a_maior_unidade_binaria_com_uma_casa() {
    assert_eq!(bytes(0), "0.0 B");
    assert_eq!(bytes(1536), "1.5 KiB");
    assert_eq!(bytes(5_583_457_485), "5.2 GiB");
    assert_eq!(bytes(2 * 1024 * 1024 * 1024 * 1024 * 1024), "2048.0 TiB");
}

#[test]
fn uso_mostra_usado_total_e_percentual_arredondado() {
    let gib = 1024 * 1024 * 1024;
    assert_eq!(
        usage(gib / 2, 4 * gib).as_deref(),
        Some("512.0 MiB / 4.0 GiB (13%)")
    );
    assert_eq!(usage(0, 0), None);
}

#[test]
fn duracao_omite_unidades_maiores_que_estao_zeradas() {
    assert_eq!(duration(59), "0m");
    assert_eq!(duration(4 * 60 + 59), "4m");
    assert_eq!(duration(3 * 3600 + 4 * 60), "3h 4m");
    assert_eq!(duration(2 * 86_400 + 3 * 3600 + 4 * 60), "2d 3h 4m");
    assert_eq!(duration(86_400 + 5 * 60), "1d 0h 5m");
}
