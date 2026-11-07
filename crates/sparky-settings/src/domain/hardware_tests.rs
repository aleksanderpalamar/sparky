use super::memory_label;

#[test]
fn ram_e_formatada_em_gib_com_uma_casa_decimal() {
    assert_eq!(memory_label(16 * 1_048_576), "16,0 GiB");
    assert_eq!(memory_label(16_150_580), "15,4 GiB");
    assert_eq!(memory_label(524_288), "0,5 GiB");
}
