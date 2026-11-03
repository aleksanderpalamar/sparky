use crate::app::state::Settings;
use crate::app::test_support::{painel_com_fundos, salvo, tecla};
use crate::domain::system::RowId;

fn linha(settings: &Settings, id: RowId) -> Option<usize> {
    settings.system_rows().iter().position(|row| row.id == id)
}

fn valor(settings: &Settings, id: RowId) -> Option<String> {
    settings
        .system_rows()
        .into_iter()
        .find(|row| row.id == id)
        .map(|row| row.value)
}

#[test]
fn aplicar_o_tema_leva_os_dois_papeis_de_parede_para_o_primeiro() {
    let (mut settings, arquivo, _) = painel_com_fundos();
    tecla(&mut settings, "l");
    assert_eq!(
        settings.config.general.wallpaper.as_deref(),
        Some("/w/azul/1-a.jpg")
    );
    assert_eq!(
        settings.config.login.wallpaper.as_deref(),
        Some("/w/azul/1-a.jpg")
    );
    assert!(salvo(&arquivo).contains("[login]\nwallpaper = \"/w/azul/1-a.jpg\""));
    assert_eq!(
        valor(&settings, RowId::LoginWallpaper).as_deref(),
        Some("1 de 2 · 1-a.jpg")
    );
}

#[test]
fn a_linha_da_tela_de_login_troca_so_o_papel_de_parede_do_login() {
    let (mut settings, arquivo, _) = painel_com_fundos();
    tecla(&mut settings, "l");
    let Some(index) = linha(&settings, RowId::LoginWallpaper) else {
        panic!("a seção Temas deveria ter a linha da tela de login");
    };
    settings.point(index);
    tecla(&mut settings, "l");
    assert_eq!(
        settings.config.login.wallpaper.as_deref(),
        Some("/w/azul/2-b.jpg")
    );
    assert_eq!(
        settings.config.general.wallpaper.as_deref(),
        Some("/w/azul/1-a.jpg")
    );
    assert!(salvo(&arquivo).contains("[login]\nwallpaper = \"/w/azul/2-b.jpg\""));
    assert_eq!(
        valor(&settings, RowId::LoginWallpaper).as_deref(),
        Some("2 de 2 · 2-b.jpg")
    );
    assert_eq!(
        valor(&settings, RowId::Wallpaper).as_deref(),
        Some("1 de 2 · 1-a.jpg")
    );
}

#[test]
fn trocar_o_da_area_de_trabalho_nao_mexe_no_do_login() {
    let (mut settings, _, _) = painel_com_fundos();
    tecla(&mut settings, "l");
    let Some(index) = linha(&settings, RowId::Wallpaper) else {
        panic!("a seção Temas deveria ter a linha do papel de parede");
    };
    settings.point(index);
    tecla(&mut settings, "l");
    assert_eq!(
        settings.config.general.wallpaper.as_deref(),
        Some("/w/azul/2-b.jpg")
    );
    assert_eq!(
        settings.config.login.wallpaper.as_deref(),
        Some("/w/azul/1-a.jpg")
    );
}
