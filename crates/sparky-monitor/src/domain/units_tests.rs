use super::units::{bytes, celsius, frequency, percent, rate};

#[test]
fn bytes_usam_unidades_binarias_com_virgula() {
    assert_eq!(bytes(0), "0 B");
    assert_eq!(bytes(999), "999 B");
    assert_eq!(bytes(1023), "1,00 KiB");
    assert_eq!(bytes(1_070_250_000), "1,00 GiB");
    assert_eq!(bytes(132_814_438), "126,66 MiB");
    assert_eq!(bytes(16_538_193_920), "15,40 GiB");
}

#[test]
fn taxas_usam_unidades_decimais_por_segundo() {
    assert_eq!(rate(0.0), "0 B/s");
    assert_eq!(rate(673.4), "673 B/s");
    assert_eq!(rate(413_700.0), "413,70 kB/s");
    assert_eq!(rate(33_610_000.0), "33,61 MB/s");
    assert_eq!(rate(1_220_000_000.0), "1,22 GB/s");
}

#[test]
fn percentual_frequencia_e_temperatura() {
    assert_eq!(percent(0.0), "0,0%");
    assert_eq!(percent(20.84), "20,8%");
    assert_eq!(frequency(998), "998 MHz");
    assert_eq!(frequency(3600), "3,60 GHz");
    assert_eq!(celsius(47.0), "47,0 °C");
}
