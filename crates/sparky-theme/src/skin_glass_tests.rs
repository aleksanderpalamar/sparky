use crate::layers::WINDOW;
use crate::lightness::lightness;
use crate::palette::{Mode, Palette};
use crate::skin::Skin;

fn com_vidro(palette: Palette) -> Skin {
    Skin::new(Palette {
        glass: true,
        ..palette
    })
}

#[test]
fn com_vidro_a_janela_fica_translucida_e_os_cartoes_opacos() {
    let vidro = com_vidro(Palette::SPARKY);
    assert_eq!(vidro.background.a, WINDOW);
    assert_eq!(vidro.lighter_background.a, 1.0);
    assert_eq!(vidro.accent.a, 1.0);
}

#[test]
fn menus_e_dialogos_dentro_da_janela_sao_sempre_opacos() {
    let opaco = Skin::new(Palette::SPARKY);
    let vidro = com_vidro(Palette::SPARKY);
    for skin in [opaco, vidro] {
        assert_eq!(skin.raised.a, 1.0);
        assert_eq!(skin.raised.r, skin.lighter_background.r);
        assert_eq!(skin.raised.b, skin.lighter_background.b);
    }
}

#[test]
fn sem_vidro_os_textos_sao_os_do_tema() {
    let opaco = Skin::new(Palette::TOKYO_NIGHT);
    let tema = Skin::new(Palette {
        glass: false,
        ..Palette::TOKYO_NIGHT
    });
    assert_eq!(opaco.dark_foreground, tema.dark_foreground);
    assert_eq!(
        opaco.foreground,
        iced_core::Color::from_rgb8(0xa9, 0xb1, 0xd6)
    );
}

#[test]
fn com_vidro_os_textos_se_afastam_do_fundo_escuro() {
    let opaco = Skin::new(Palette::TOKYO_NIGHT);
    let vidro = com_vidro(Palette::TOKYO_NIGHT);
    for (antes, depois) in [
        (opaco.foreground, vidro.foreground),
        (opaco.dark_foreground, vidro.dark_foreground),
        (opaco.muted, vidro.muted),
    ] {
        assert!(lightness(depois) > lightness(antes) + 0.15);
    }
}

#[test]
fn com_vidro_os_textos_se_afastam_do_fundo_claro() {
    let claro = Palette {
        mode: Mode::Light,
        background: crate::Rgb::new(0xef, 0xf1, 0xf5),
        lighter_background: crate::Rgb::new(0xe6, 0xe9, 0xef),
        foreground: crate::Rgb::new(0x4c, 0x4f, 0x69),
        dark_foreground: crate::Rgb::new(0x8c, 0x8f, 0xa1),
        ..Palette::TOKYO_NIGHT
    };
    let opaco = Skin::new(claro);
    let vidro = com_vidro(claro);
    assert!(lightness(vidro.foreground) < lightness(opaco.foreground) - 0.15);
    assert!(lightness(vidro.dark_foreground) < lightness(opaco.dark_foreground) - 0.15);
}
