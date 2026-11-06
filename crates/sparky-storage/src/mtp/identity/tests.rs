use super::phone;

#[test]
fn identidade_serial_coincide_com_o_host_usado_pelo_gvfs() {
    let volume = phone(
        Some("Android_Test-1"),
        Some("001"),
        Some("002"),
        None,
        Some("Android"),
    )
    .unwrap();
    assert_eq!(volume.path, "mtp://Android_Test-1/");
    assert_eq!(volume.label, "Android");
    assert_eq!(volume.filesystem, "mtp");
    assert!(volume.removable);
    assert!(!volume.mounted());
}

#[test]
fn identidade_escapa_separadores_e_utf8_sem_criar_outro_caminho() {
    let volume = phone(Some("A/B #ç%"), None, None, None, None).unwrap();
    assert_eq!(volume.path, "mtp://A%2FB%20%23%C3%A7%25/");
    assert_eq!(volume.label, "Dispositivo móvel");
}

#[test]
fn sem_serial_usa_o_endereco_usb_compativel() {
    let volume = phone(None, Some("1"), Some("7"), None, Some(" ")).unwrap();
    assert_eq!(volume.path, "mtp://[usb:001,007]/");
    assert_eq!(volume.label, "Dispositivo móvel");
}

#[test]
fn endereco_usb_invalido_nao_cria_volume_inutilizavel() {
    for (bus, address) in [("0", "1"), ("1", "0"), ("a", "2"), ("1", "128")] {
        assert!(phone(None, Some(bus), Some(address), None, None).is_none());
    }
    assert!(phone(None, None, None, None, None).is_none());
}

#[test]
fn telefone_usa_marca_e_modelo_sem_repetir_a_marca() {
    let described = phone(
        Some("Samsung_S22"),
        None,
        None,
        Some("Samsung"),
        Some("Galaxy S22 Ultra"),
    )
    .unwrap();
    assert_eq!(described.label, "Samsung Galaxy S22 Ultra");
    let repeated = phone(
        Some("Samsung_S22"),
        None,
        None,
        Some("Samsung"),
        Some("Samsung Galaxy S22 Ultra"),
    )
    .unwrap();
    assert_eq!(repeated.label, "Samsung Galaxy S22 Ultra");
}
