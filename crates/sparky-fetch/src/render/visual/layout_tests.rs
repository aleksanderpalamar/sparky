use crate::config::settings::{Layout, Logo};
use crate::render::view::View;
use crate::render::visual::fixture::view;
use crate::render::visual::layout::{self, Arrangement};
use crate::render::visual::segment::{Line, Segment, Tone};
use crate::render::visual::style::Paint;

fn linha(texto: &str, tom: Tone) -> Line {
    Line::new(vec![Segment::new(texto, tom)])
}

fn escolha(width: usize, layout: Layout, logo: Logo) -> Arrangement {
    let visao = View {
        layout,
        logo,
        ..view(width, false)
    };
    Arrangement::choose(&visao, 20)
}

fn texto(linhas: Vec<Line>) -> Vec<String> {
    linhas
        .iter()
        .map(|linha| Paint::Plain.line(linha))
        .collect()
}

#[test]
fn escolhe_o_arranjo_pela_largura_e_pelas_preferencias() {
    assert_eq!(
        escolha(80, Layout::Classic, Logo::Show),
        Arrangement::Beside
    );
    assert_eq!(
        escolha(47, Layout::Classic, Logo::Show),
        Arrangement::Beside
    );
    assert_eq!(escolha(46, Layout::Classic, Logo::Show), Arrangement::Above);
    assert_eq!(escolha(80, Layout::Stacked, Logo::Show), Arrangement::Above);
    assert_eq!(escolha(20, Layout::Stacked, Logo::Show), Arrangement::Above);
    assert_eq!(escolha(19, Layout::Classic, Logo::Show), Arrangement::Alone);
    assert_eq!(escolha(80, Layout::Classic, Logo::Hide), Arrangement::Alone);
}

#[test]
fn o_espaco_das_informacoes_desconta_o_logotipo_ao_lado() {
    assert_eq!(Arrangement::Beside.info_room(80, 20), 57);
    assert_eq!(Arrangement::Above.info_room(80, 20), 80);
    assert_eq!(Arrangement::Alone.info_room(80, 20), 80);
}

#[test]
fn ao_lado_completa_as_linhas_que_faltam_dos_dois_lados() {
    let logo = vec![linha("/\\", Tone::Logo), linha("\\/", Tone::Logo)];
    let curto = vec![linha("a", Tone::Value)];
    assert_eq!(
        texto(layout::compose(Arrangement::Beside, &logo, curto)),
        ["/\\   a", "\\/"]
    );
    let longo = vec![
        linha("a", Tone::Value),
        linha("b", Tone::Value),
        linha("c", Tone::Value),
    ];
    assert_eq!(
        texto(layout::compose(Arrangement::Beside, &logo, longo)),
        ["/\\   a", "\\/   b", "     c"]
    );
}

#[test]
fn acima_separa_com_linha_em_branco_e_sozinho_ignora_o_logotipo() {
    let logo = vec![linha("<>", Tone::Logo)];
    let info = vec![linha("a", Tone::Value)];
    assert_eq!(
        texto(layout::compose(Arrangement::Above, &logo, info.clone())),
        ["<>", "", "a"]
    );
    assert_eq!(
        texto(layout::compose(Arrangement::Above, &logo, Vec::new())),
        ["<>"]
    );
    assert_eq!(
        texto(layout::compose(Arrangement::Alone, &logo, info)),
        ["a"]
    );
}
