use fluent_bundle::FluentArgs;

use crate::{Catalog, Locale, locale_from_environment};

const PT: &str = "welcome = Bem-vindo\ncount = { $count ->\n    [one] Um arquivo\n   *[other] { $count } arquivos\n}\n";
const EN: &str =
    "welcome = Welcome\ncount = { $count ->\n    [one] One file\n   *[other] { $count } files\n}\n";

#[test]
fn escolhe_o_idioma_da_sessao_e_normaliza_a_regiao() {
    let locale = locale_from_environment(|key| match key {
        "LC_MESSAGES" => Some("en_US.UTF-8".to_owned()),
        "LANG" => Some("pt_BR.UTF-8".to_owned()),
        _ => None,
    });
    assert_eq!(locale.candidates(), vec!["en-US", "en"]);
    assert_eq!(Locale::parse("C"), Locale::parse("pt-BR"));
}

#[test]
fn ignora_variaveis_de_idioma_vazias() {
    let locale = locale_from_environment(|key| match key {
        "LC_ALL" | "LC_MESSAGES" => Some(String::new()),
        "LANG" => Some("en_US.UTF-8".to_owned()),
        _ => None,
    });
    assert_eq!(locale.candidates(), vec!["en-US", "en"]);
}

#[test]
fn traduz_e_formata_plurais() {
    let catalog = Catalog::new(&Locale::parse("en_US.UTF-8"), PT, EN).unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 2);
    assert_eq!(catalog.text("welcome").as_deref(), Some("Welcome"));
    assert_eq!(
        catalog.message("count", Some(&args)).as_deref(),
        Some("2 files")
    );
}

#[test]
fn volta_ao_portugues_quando_a_chave_falta() {
    let catalog = Catalog::new(&Locale::parse("en-US"), PT, "welcome = Welcome\n").unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 1);
    assert_eq!(
        catalog.message("count", Some(&args)).as_deref(),
        Some("Um arquivo")
    );
}

#[test]
fn pacote_instalado_pode_substituir_o_catalogo() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("sparky/i18n/en-US");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("core.ftl"), "welcome = Hello from package\n").unwrap();
    let catalog =
        Catalog::with_installed(&Locale::parse("en-US"), "core", &[folder.path()], PT, EN).unwrap();
    assert_eq!(
        catalog.text("welcome").as_deref(),
        Some("Hello from package")
    );
}

#[test]
fn pacote_incompleto_preserva_o_catalogo_embutido() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("sparky/i18n/en-US");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("core.ftl"), "welcome = Hello from package\n").unwrap();
    let catalog =
        Catalog::with_installed(&Locale::parse("en-US"), "core", &[folder.path()], PT, EN).unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 2);
    assert_eq!(
        catalog.message("count", Some(&args)).as_deref(),
        Some("2 files")
    );
    assert_eq!(
        catalog.text("welcome").as_deref(),
        Some("Hello from package")
    );
}

#[test]
fn pacote_invalido_nao_impede_catalogo_embutido() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("sparky/i18n/en-US");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("core.ftl"), "welcome = {\n").unwrap();
    let catalog =
        Catalog::with_installed(&Locale::parse("en-US"), "core", &[folder.path()], PT, EN).unwrap();
    assert_eq!(catalog.text("welcome").as_deref(), Some("Welcome"));
}
