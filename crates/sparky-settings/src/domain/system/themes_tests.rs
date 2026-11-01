use sparky_theme::{Palette, Rgb};

use crate::domain::system::themes::{Catalog, Theme, Themes};
use crate::domain::system::{Change, Control, Request, RowId};
use crate::domain::value::Step;

#[test]
fn paletas_parciais_usam_os_padroes_nas_cores_ausentes() {
    let theme = Theme::parse("claro", "mode = \"light\"\naccent = \"#010203\"\n");
    assert!(theme.is_ok());
    let palette = theme.map(|theme| theme.palette()).unwrap_or_default();
    assert_eq!(palette.accent, Rgb::new(1, 2, 3));
    assert_eq!(palette.background, Palette::TOKYO_NIGHT.background);
}

#[test]
fn arquivos_sem_cores_ou_com_valores_invalidos_sao_recusados() {
    for contents in [
        "",
        "mode = \"light\"",
        "accent = \"azul\"",
        "mode = \"sepia\"\naccent = \"#010203\"",
    ] {
        assert!(Theme::parse("invalido", contents).is_err());
    }
}

fn catalogo() -> Themes {
    let tema =
        |nome: &str, cores: &str| Theme::parse(nome, cores).unwrap_or_else(|_| Theme::builtin());
    let azul = tema("azul", "accent = \"#010203\"");
    let claro = tema("claro", "mode = \"light\"\naccent = \"#040506\"");
    Themes::Ready(Catalog {
        themes: vec![Theme::builtin(), azul, claro],
        current: Palette::TOKYO_NIGHT,
    })
}

#[test]
fn o_catalogo_vira_uma_escolha_com_o_tema_em_uso() {
    let rows = catalogo().rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, RowId::Theme);
    assert_eq!(rows[0].label, "Tema atual");
    assert_eq!(rows[0].value, "tokyo-night (escuro)");
    let opcoes = ["tokyo-night (escuro)", "azul (escuro)", "claro (claro)"]
        .map(str::to_owned)
        .to_vec();
    assert_eq!(rows[0].control, Control::Choice(opcoes));
}

#[test]
fn escolher_pelo_nome_ou_pelas_setas_so_aplica_outro_tema() {
    let themes = catalogo();
    let escolha = |change| themes.request(RowId::Theme, change).map(|theme| theme.name);
    assert_eq!(
        escolha(Change::Text("azul (escuro)".to_owned())),
        Some("azul".to_owned())
    );
    assert_eq!(
        escolha(Change::Text("tokyo-night (escuro)".to_owned())),
        None
    );
    assert_eq!(escolha(Change::Text("roxo (escuro)".to_owned())), None);
    assert_eq!(escolha(Change::Step(Step::More)), Some("azul".to_owned()));
    assert_eq!(escolha(Change::Step(Step::Less)), Some("claro".to_owned()));
    assert_eq!(escolha(Change::Activate), None);
}

#[test]
fn falhas_de_catalogo_sao_somente_leitura() {
    let themes = Themes::Unavailable("sem permissão".to_owned());
    assert_eq!(themes.rows()[0].control, Control::Reading);
    assert_eq!(themes.rows()[0].value, "sem permissão");
    assert_eq!(themes.request(RowId::Theme, Change::Step(Step::More)), None);
}

#[test]
fn o_tema_leva_o_nome_dos_icones() {
    let tema = Theme::builtin().with_icons(Some("Yaru-blue".to_owned()));
    assert_eq!(tema.icons(), Some("Yaru-blue"));
    assert_eq!(Theme::builtin().icons(), None);
}

#[test]
fn trocar_os_icones_vai_pelo_gsettings() {
    let pedido = Request::SetIconTheme("Yaru-blue".to_owned());
    let argumentos = [
        "set",
        "org.gnome.desktop.interface",
        "icon-theme",
        "Yaru-blue",
    ]
    .map(str::to_owned)
    .to_vec();
    assert_eq!(pedido.command(), ("gsettings", argumentos));
}
