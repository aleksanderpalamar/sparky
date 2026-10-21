use crate::drag::{Edges, Horizontal, Vertical};
use crate::drag_session::{Change, Drag, Gesture, Release};
use crate::geometry::Rect;
use crate::ids::WindowId;
use crate::workspace::{Fullscreen, Layer};

const CAIXA: Rect = Rect::new(100, 100, 400, 300);

fn canto_inferior_direito() -> Gesture {
    Gesture::Resize(Edges { horizontal: Horizontal::Right, vertical: Vertical::Bottom })
}

fn arrasto(gesture: Gesture, layer: Layer) -> Drag {
    match Drag::start(WindowId(7), gesture, layer, CAIXA, (300, 250)) {
        Some(drag) => drag,
        None => panic!("o arrasto deveria começar"),
    }
}

#[test]
fn a_flutuante_acompanha_o_ponteiro_desde_o_ponto_de_partida() {
    let mut drag = arrasto(Gesture::Move, Layer::Floating);
    assert_eq!(drag.follow((310, 240)), Change::Frame(Rect::new(110, 90, 400, 300)));
    assert_eq!(drag.follow((350, 300)), Change::Frame(Rect::new(150, 150, 400, 300)));
    assert_eq!(drag.release(), Release::Nothing);
    assert_eq!(drag.window(), WindowId(7));
}

#[test]
fn a_flutuante_redimensiona_pelo_canto_escolhido() {
    let mut drag = arrasto(canto_inferior_direito(), Layer::Floating);
    assert_eq!(drag.follow((340, 260)), Change::Frame(Rect::new(100, 100, 440, 310)));
}

#[test]
fn no_mosaico_redimensionar_aplica_so_o_que_falta_a_cada_passo() {
    let mut drag = arrasto(canto_inferior_direito(), Layer::Tiled);
    assert_eq!(drag.follow((320, 250)), Change::Grow(20, 0));
    assert_eq!(drag.follow((330, 260)), Change::Grow(10, 10));
    assert_eq!(drag.follow((300, 250)), Change::Grow(-30, -10));
}

#[test]
fn no_mosaico_mover_so_troca_ao_soltar() {
    let mut drag = arrasto(Gesture::Move, Layer::Tiled);
    assert_eq!(drag.follow((900, 700)), Change::Nothing);
    assert_eq!(drag.release(), Release::SwapWithUnder);
}

#[test]
fn tela_cheia_nao_se_arrasta() {
    assert_eq!(
        Drag::start(WindowId(7), Gesture::Move, Layer::Fullscreen(Fullscreen::Monitor), CAIXA, (0, 0)),
        None
    );
}
