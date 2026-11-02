use super::layout::{Layout, Part, bands};

fn perto(found: f32, expected: f32) -> bool {
    (found - expected).abs() < 0.5
}

fn centro(height: f32, part: Part) -> f32 {
    bands(height)
        .iter()
        .find(|band| band.part == part)
        .map_or(f32::NAN, |band| band.top + band.height / 2.0)
}

#[test]
fn o_formulario_ocupa_os_quarenta_por_cento_da_esquerda_e_os_campos_a_metade_dele() {
    let layout = Layout::new(1920.0, 1080.0);
    assert!(perto(layout.form_width, 768.0));
    assert!(perto(layout.field_width, 384.0));
}

#[test]
fn em_1080_as_letras_tem_o_tamanho_do_tema_de_13_pontos() {
    let layout = Layout::new(1920.0, 1080.0);
    assert!(perto(layout.time, 156.0));
    assert!(perto(layout.date, 34.67));
    assert!(perto(layout.text, 17.33));
    assert!(perto(layout.small, 13.87));
    assert!(perto(layout.field_height, 39.0));
    assert!(perto(layout.power_icon, 40.0));
}

#[test]
fn em_4k_tudo_dobra() {
    let full_hd = Layout::new(1920.0, 1080.0);
    let uhd = Layout::new(3840.0, 2160.0);
    assert!(perto(uhd.time, full_hd.time * 2.0));
    assert!(perto(uhd.field_height, full_hd.field_height * 2.0));
    assert!(perto(uhd.form_width, full_hd.form_width * 2.0));
}

#[test]
fn os_centros_ficam_onde_estao_na_foto() {
    let medidas = [
        (Part::Time, 0.247),
        (Part::Date, 0.362),
        (Part::Username, 0.552),
        (Part::Password, 0.605),
        (Part::Login, 0.687),
        (Part::PowerIcons, 0.838),
        (Part::PowerLabels, 0.872),
        (Part::Session, 0.936),
    ];
    for (part, fraction) in medidas {
        assert!(perto(centro(1080.0, part), 1080.0 * fraction), "{part:?}");
    }
}

#[test]
fn as_faixas_vem_de_cima_para_baixo_sem_se_cruzar_e_dentro_da_tela() {
    for height in [720.0, 1080.0, 1440.0, 2160.0] {
        let bands = bands(height);
        assert!(bands.first().is_some_and(|band| band.top >= 0.0));
        assert!(
            bands
                .last()
                .is_some_and(|band| band.top + band.height <= height)
        );
        for pair in bands.windows(2) {
            assert!(
                pair[0].top + pair[0].height <= pair[1].top,
                "{:?} cruza {:?}",
                pair[0].part,
                pair[1].part
            );
        }
    }
}

#[test]
fn o_aviso_fica_logo_acima_do_usuario() {
    let aviso = centro(1080.0, Part::Warning);
    let usuario = centro(1080.0, Part::Username);
    assert!(aviso < usuario);
    assert!(usuario - aviso < 50.0);
}
