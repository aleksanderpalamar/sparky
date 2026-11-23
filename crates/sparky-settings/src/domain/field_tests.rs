use crate::domain::field::{Blank, Field, Kind};
use crate::domain::section::Section;

#[test]
fn so_as_secoes_de_configuracao_tem_campos() {
    assert_eq!(Field::of(Section::Appearance).len(), 7);
    assert_eq!(Field::Wallpaper.place(), ("general", "wallpaper"));
    assert_eq!(Field::Wallpaper.kind(), Kind::Text(Blank::Allowed));
    assert_eq!(Field::RepeatRate.place(), ("input", "repeat_rate"));
    assert!(Field::of(Section::About).is_empty());
}

#[test]
fn apps_obrigatorios_nao_aceitam_vazio_e_o_terminal_aceita() {
    assert_eq!(Field::Launcher.kind(), Kind::Text(Blank::Refused));
    assert_eq!(Field::Terminal.kind(), Kind::Text(Blank::Allowed));
}

#[test]
fn o_menu_do_sistema_fica_nos_aplicativos() {
    assert!(Field::of(Section::Apps).contains(&Field::SystemMenu));
    assert_eq!(Field::SystemMenu.place(), ("apps", "system_menu"));
    assert_eq!(Field::SystemMenu.kind(), Kind::Text(Blank::Refused));
}

#[test]
fn a_secao_dock_liga_ajusta_o_tamanho_e_oculta() {
    assert_eq!(
        Field::of(Section::Dock),
        [
            Field::DockEnabled,
            Field::DockSize,
            Field::DockAutohide,
            Field::DockRounded,
            Field::DockRadius
        ]
    );
    assert_eq!(Field::DockRounded.place(), ("dock", "rounded"));
    assert_eq!(Field::DockRadius.place(), ("dock", "radius"));
    assert_eq!(Field::DockRounded.kind(), Kind::Flag);
    let Kind::Number(raio) = Field::DockRadius.kind() else {
        panic!("o raio é numérico");
    };
    assert_eq!((raio.min, raio.max, raio.step), (0, 16, 1));
    assert_eq!(Field::DockEnabled.place(), ("dock", "enabled"));
    assert_eq!(Field::DockSize.place(), ("dock", "size"));
    assert_eq!(Field::DockAutohide.place(), ("dock", "autohide"));
    assert_eq!(Field::DockEnabled.kind(), Kind::Flag);
    assert_eq!(Field::DockAutohide.kind(), Kind::Flag);
    let Kind::Number(bounds) = Field::DockSize.kind() else {
        panic!("o tamanho é numérico");
    };
    assert_eq!((bounds.min, bounds.max), (40, 96));
}

#[test]
fn o_raio_so_aparece_com_os_cantos_arredondados_ligados() {
    let mut config = sparky_config::Config::default();
    assert!(!Field::shown(Section::Dock, &config).contains(&Field::DockRadius));
    assert!(Field::shown(Section::Dock, &config).contains(&Field::DockRounded));
    config.dock.rounded = true;
    assert_eq!(
        Field::shown(Section::Dock, &config).last(),
        Some(&Field::DockRadius)
    );
    assert_eq!(
        Field::shown(Section::Apps, &config),
        Field::of(Section::Apps)
    );
}

#[test]
fn cada_campo_mora_numa_chave_do_sparky_toml() {
    assert_eq!(Field::RepeatDelay.place(), ("input", "repeat_delay"));
    assert_eq!(Field::DefaultBinds.place(), ("general", "default_binds"));
}
