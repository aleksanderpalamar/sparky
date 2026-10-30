use super::state::Settings;
use super::test_support::{Sparky, painel};
use super::text_probe::visible_texts;
use crate::domain::hardware::Hardware;

fn texts(settings: &Settings) -> Vec<String> {
    visible_texts(crate::view::about::view(settings))
}

#[test]
fn sobre_mostra_hardware_em_vez_do_caminho_da_configuracao() {
    let (settings, _) = painel(None, Sparky::default());
    let visible = texts(&settings);
    assert!(
        !visible
            .iter()
            .any(|text| text == "Configuração" || text.contains("sparky.toml"))
    );
    for label in ["Processador", "GPU", "RAM"] {
        assert!(visible.iter().any(|text| text == label), "{label}");
    }
}

#[test]
fn modelos_de_hardware_e_ram_total_aparecem_na_tela() {
    let (mut settings, _) = painel(None, Sparky::default());
    settings.context.about.hardware = Hardware {
        processor: Some("AMD Ryzen 5 5500".into()),
        gpus: vec!["AMD Radeon RX 6600".into(), "Intel UHD Graphics".into()],
        memory_kib: Some(16_150_580),
    };
    let visible = texts(&settings);
    for value in [
        "AMD Ryzen 5 5500",
        "AMD Radeon RX 6600\nIntel UHD Graphics",
        "15,4 GiB",
    ] {
        assert!(visible.iter().any(|text| text == value), "{value}");
    }
}

#[test]
fn informacoes_ausentes_ficam_explicitas_na_tela() {
    let (settings, _) = painel(None, Sparky::default());
    let visible = texts(&settings);
    for label in ["Processador", "GPU", "RAM"] {
        assert!(
            visible
                .windows(2)
                .any(|pair| pair == [label, "não informado"]),
            "{label}"
        );
    }
}
