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
    assert_eq!(Locale::parse("C"), Locale::parse("en"));
    assert_eq!(Locale::parse(""), Locale::parse("en"));
    assert_eq!(locale_from_environment(|_| None), Locale::parse("en"));
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
fn volta_ao_ingles_quando_a_chave_falta() {
    let catalog = Catalog::new(&Locale::parse("pt-BR"), "welcome = Bem-vindo\n", EN).unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 1);
    assert_eq!(
        catalog.message("count", Some(&args)).as_deref(),
        Some("One file")
    );
}

#[test]
fn idioma_sem_traducao_cai_no_ingles_e_o_portugues_vale_para_toda_regiao() {
    for (locale, esperado) in [
        ("de_DE.UTF-8", "Welcome"),
        ("C", "Welcome"),
        ("pt_PT.UTF-8", "Bem-vindo"),
        ("pt_BR.UTF-8", "Bem-vindo"),
    ] {
        let catalog = Catalog::new(&Locale::parse(locale), PT, EN).unwrap();
        assert_eq!(
            catalog.text("welcome").as_deref(),
            Some(esperado),
            "{locale}"
        );
    }
}

#[test]
fn pacote_de_outro_idioma_instalado_vence_e_completa_com_ingles() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("sparky/i18n/de");
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("core.ftl"), "welcome = Willkommen\n").unwrap();
    let catalog = Catalog::with_installed(
        &Locale::parse("de_DE.UTF-8"),
        "core",
        &[folder.path()],
        PT,
        EN,
    )
    .unwrap();
    let mut args = FluentArgs::new();
    args.set("count", 2);
    assert_eq!(catalog.text("welcome").as_deref(), Some("Willkommen"));
    assert_eq!(
        catalog.message("count", Some(&args)).as_deref(),
        Some("2 files")
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
