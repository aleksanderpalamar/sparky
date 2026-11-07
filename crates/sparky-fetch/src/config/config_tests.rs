use std::path::PathBuf;
use std::time::Duration;

use sparky_theme::Rgb;

use crate::config::cli::{Command, ConfigSource, parse};
use crate::config::file::read;
use crate::config::path::config_path;
use crate::config::settings::{ColorMode, Format, Layout, Logo, Settings};
use crate::domain::field::Field;

fn argumentos(lista: &[&str]) -> Vec<String> {
    lista.iter().map(|item| (*item).to_owned()).collect()
}

fn rodar(lista: &[&str]) -> (crate::config::settings::Partial, ConfigSource) {
    match parse(&argumentos(lista)) {
        Ok(Command::Run { partial, config }) => (partial, config),
        other => panic!("esperava Run: {other:?}"),
    }
}

#[test]
fn o_arquivo_segue_o_xdg_config_home() {
    assert_eq!(
        config_path(Some("/cfg"), "/home/p"),
        PathBuf::from("/cfg/sparky-fetch/config.toml")
    );
    assert_eq!(
        config_path(Some("  "), "/home/p"),
        PathBuf::from("/home/p/.config/sparky-fetch/config.toml")
    );
    assert_eq!(
        config_path(None, "/home/p"),
        PathBuf::from("/home/p/.config/sparky-fetch/config.toml")
    );
}

#[test]
fn sem_nada_valem_os_padroes() {
    let settings = Settings::default();
    assert_eq!(settings.fields, Field::ALL.to_vec());
    assert_eq!(settings.format, None);
    assert_eq!(settings.color, ColorMode::Auto);
    assert_eq!(settings.logo, Logo::Show);
    assert_eq!(settings.layout, Layout::Classic);
    assert_eq!(settings.timeout, Duration::from_millis(400));
}

#[test]
fn o_toml_escolhe_campos_ordem_e_cores() {
    let texto = "fields = [\"kernel\", \"os\", \"gpu\"]\nhide = [\"gpu\"]\nformat = \"plain\"\ncolor = \"never\"\nlogo = false\nlayout = \"stacked\"\nwidth = 70\ntimeout_ms = 250\n\n[colors]\naccent = \"#ff0000\"\n";
    let partial = read(texto).unwrap_or_else(|error| panic!("{error}"));
    let settings = Settings::default().merged(&[partial]);
    assert_eq!(settings.fields, vec![Field::Kernel, Field::Os]);
    assert_eq!(settings.format, Some(Format::Plain));
    assert_eq!(settings.color, ColorMode::Never);
    assert_eq!(settings.logo, Logo::Hide);
    assert_eq!(settings.layout, Layout::Stacked);
    assert_eq!(settings.width, Some(70));
    assert_eq!(settings.timeout, Duration::from_millis(250));
    assert_eq!(settings.colors.accent, Some(Rgb::new(0xff, 0, 0)));
}

#[test]
fn o_toml_recusa_campo_cor_e_chave_desconhecidos() {
    assert!(read("fields = [\"bateria\"]\n").is_err());
    assert!(read("[colors]\naccent = \"azul\"\n").is_err());
    assert!(read("tamanho = 3\n").is_err());
    assert!(read("format = \"html\"\n").is_err());
}

#[test]
fn a_linha_de_comando_vence_o_toml() {
    let arquivo =
        read("format = \"plain\"\nfields = [\"os\", \"kernel\"]\n[colors]\naccent = \"#ff0000\"\n")
            .unwrap_or_else(|error| panic!("{error}"));
    let (cli, _) = rodar(&["--json", "--fields=cpu,os", "--accent", "#00ff00"]);
    let settings = Settings::default().merged(&[arquivo, cli]);
    assert_eq!(settings.format, Some(Format::Json));
    assert_eq!(settings.fields, vec![Field::Cpu, Field::Os]);
    assert_eq!(settings.colors.accent, Some(Rgb::new(0, 0xff, 0)));
}

#[test]
fn esconder_vale_somado_do_toml_e_da_linha_de_comando() {
    let arquivo = read("hide = [\"gpu\"]\n").unwrap_or_else(|error| panic!("{error}"));
    let (cli, _) = rodar(&["--hide", "colors,disk"]);
    let settings = Settings::default().merged(&[arquivo, cli]);
    assert!(!settings.fields.contains(&Field::Gpu));
    assert!(!settings.fields.contains(&Field::Colors));
    assert!(!settings.fields.contains(&Field::Disk));
    assert_eq!(settings.fields.len(), Field::ALL.len() - 3);
}

#[test]
fn le_as_opcoes_da_linha_de_comando() {
    let (partial, config) = rodar(&[
        "--format",
        "visual",
        "--no-color",
        "--no-logo",
        "--layout=stacked",
        "--width",
        "60",
        "--timeout",
        "900",
        "--config",
        "/tmp/f.toml",
    ]);
    let settings = Settings::default().merged(&[partial]);
    assert_eq!(settings.format, Some(Format::Visual));
    assert_eq!(settings.color, ColorMode::Never);
    assert_eq!(settings.logo, Logo::Hide);
    assert_eq!(settings.layout, Layout::Stacked);
    assert_eq!(settings.width, Some(60));
    assert_eq!(settings.timeout, Duration::from_millis(900));
    assert_eq!(config, ConfigSource::Path(PathBuf::from("/tmp/f.toml")));
    assert_eq!(rodar(&["--no-config"]).1, ConfigSource::Skip);
    assert_eq!(rodar(&[]).1, ConfigSource::Default);
}

#[test]
fn ajuda_versao_e_lista_de_campos() {
    assert!(matches!(parse(&argumentos(&["-h"])), Ok(Command::Help)));
    assert!(matches!(
        parse(&argumentos(&["--version"])),
        Ok(Command::Version)
    ));
    assert!(matches!(
        parse(&argumentos(&["--list-fields"])),
        Ok(Command::ListFields)
    ));
}

#[test]
fn erros_de_uso_sao_explicitos() {
    assert!(parse(&argumentos(&["--voar"])).is_err());
    assert!(parse(&argumentos(&["--width"])).is_err());
    assert!(parse(&argumentos(&["--width", "largo"])).is_err());
    assert!(parse(&argumentos(&["--fields", "cpu,bateria"])).is_err());
    assert!(parse(&argumentos(&["--color", "rosa"])).is_err());
    assert!(parse(&argumentos(&["--accent", "azul"])).is_err());
    assert!(parse(&argumentos(&["--timeout", "0"])).is_err());
}
