use sparky_theme::{Palette, Rgb};

use crate::domain::system::themes::{Catalog, Theme, Themes};
use crate::domain::system::{Change, Control, RowId};
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
    for contents in ["", "mode = \"light\"", "accent = \"azul\"", "mode = \"sepia\"\naccent = \"#010203\""] {
        assert!(Theme::parse("invalido", contents).is_err());
    }
}

#[test]
fn o_catalogo_marca_a_paleta_em_uso_e_so_aplica_outra() {
    let theme = Theme::parse("azul", "accent = \"#010203\"");
    assert!(theme.is_ok());
    let themes = Themes::Ready(Catalog { themes: vec![Theme::builtin(), theme.unwrap_or_else(|_| Theme::builtin())], current: Palette::TOKYO_NIGHT });
    assert_eq!(themes.rows()[0].control, Control::Toggle(true));
    assert_eq!(themes.rows()[1].control, Control::Toggle(false));
    assert_eq!(themes.request(RowId::Theme(0), Change::Activate), None);
    assert!(themes.request(RowId::Theme(1), Change::Activate).is_some());
    assert_eq!(themes.request(RowId::Theme(99), Change::Activate), None);
    assert_eq!(themes.request(RowId::Theme(1), Change::Step(Step::More)), None);
}

#[test]
fn falhas_de_catalogo_sao_somente_leitura() {
    let themes = Themes::Unavailable("sem permissão".to_owned());
    assert_eq!(themes.rows()[0].control, Control::Reading);
    assert_eq!(themes.rows()[0].value, "sem permissão");
    assert_eq!(themes.request(RowId::Theme(0), Change::Activate), None);
}
