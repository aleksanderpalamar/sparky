use super::magnify::{PEAK, scale, targets};

#[test]
fn o_icone_sob_o_mouse_chega_ao_pico_e_os_vizinhos_caem_suavemente() {
    let pitch = 75.0;
    assert_eq!(scale(0.0, pitch), PEAK);
    let vizinho = scale(75.0, pitch);
    let segundo = scale(150.0, pitch);
    assert!(vizinho < PEAK && vizinho > segundo && segundo > 1.0);
    assert_eq!(scale(400.0, pitch), 1.0);
}

#[test]
fn sem_mouse_nenhum_icone_cresce() {
    assert_eq!(targets(None, &[10.0, 85.0], 75.0), vec![1.0, 1.0]);
    assert_eq!(targets(Some(10.0), &[10.0, 85.0], 75.0)[0], PEAK);
}
