use iced::advanced::clipboard::Null;
use iced::advanced::input_method::{self, InputMethod, Preedit, Purpose};
use iced::advanced::layout::{Layout, Node};
use iced::advanced::renderer::Headless;
use iced::advanced::widget::Tree;
use iced::advanced::{Shell, Widget};
use iced::futures::executor::block_on;
use iced::time::Instant;
use iced::{Event, Pixels, Point, Rectangle, Renderer, Size, Theme, mouse, window};
use sparky_ui::style::FONT;

use crate::app::composer::Composer;

const AREA: Rectangle = Rectangle {
    x: 42.0,
    y: 50.0,
    width: 8.0,
    height: 16.0,
};

struct Ensaio {
    composer: Composer<String>,
    arvore: Tree,
    renderer: Renderer,
}

impl Ensaio {
    fn novo() -> Self {
        let Some(renderer) = block_on(<Renderer as Headless>::new(
            FONT,
            Pixels(12.0),
            Some("tiny-skia"),
        )) else {
            panic!("o renderizador de CPU não abriu")
        };
        let composer = Composer::new(AREA, std::convert::identity);
        let arvore = Tree::new(&composer as &dyn Widget<String, Theme, Renderer>);
        Self {
            composer,
            arvore,
            renderer,
        }
    }

    fn evento(&mut self, evento: Event) -> (Vec<String>, InputMethod) {
        self.evento_em(evento, Point::ORIGIN)
    }

    fn evento_em(&mut self, evento: Event, position: Point) -> (Vec<String>, InputMethod) {
        let mut mensagens = Vec::new();
        let mut shell = Shell::new(&mut mensagens);
        let no = Node::new(Size::ZERO).move_to(position);
        Widget::<String, Theme, Renderer>::update(
            &mut self.composer,
            &mut self.arvore,
            &evento,
            Layout::new(&no),
            mouse::Cursor::Unavailable,
            &self.renderer,
            &mut Null,
            &mut shell,
            &Rectangle::default(),
        );
        let ime = shell.input_method().clone();
        (mensagens, ime)
    }
}

fn quadro() -> Event {
    Event::Window(window::Event::RedrawRequested(Instant::now()))
}

#[test]
fn o_texto_confirmado_pelo_ime_vira_mensagem() {
    let mut ensaio = Ensaio::novo();
    let (mensagens, _) = ensaio.evento(Event::InputMethod(input_method::Event::Commit(
        "ã".to_owned(),
    )));
    assert_eq!(mensagens, ["ã"]);
}

#[test]
fn a_cada_quadro_o_ime_e_pedido_na_celula_do_cursor() {
    let mut ensaio = Ensaio::novo();
    let (_, ime) = ensaio.evento(quadro());
    assert_eq!(
        ime,
        InputMethod::Enabled {
            cursor: AREA,
            purpose: Purpose::Terminal,
            preedit: None,
        }
    );
}

#[test]
fn a_composicao_aparece_ate_ser_confirmada() {
    let mut ensaio = Ensaio::novo();
    ensaio.evento(Event::InputMethod(input_method::Event::Preedit(
        "かな".to_owned(),
        None,
    )));
    let (_, compondo) = ensaio.evento(quadro());
    let InputMethod::Enabled { preedit, .. } = compondo else {
        panic!("o IME devia estar ligado")
    };
    assert_eq!(
        preedit.map(|preedit| preedit.content),
        Some("かな".to_owned())
    );
    ensaio.evento(Event::InputMethod(input_method::Event::Commit(
        "仮名".to_owned(),
    )));
    let (_, confirmado) = ensaio.evento(quadro());
    let InputMethod::Enabled { preedit, .. } = confirmado else {
        panic!("o IME devia estar ligado")
    };
    assert_eq!(preedit, None::<Preedit>);
}

#[test]
fn a_barra_de_abas_desloca_o_ime_junto_com_o_terminal() {
    let mut ensaio = Ensaio::novo();
    let (_, ime) = ensaio.evento_em(quadro(), Point::new(0.0, super::workspace::TAB_HEIGHT));
    let InputMethod::Enabled { cursor, .. } = ime else {
        panic!("sem IME")
    };
    assert_eq!(cursor.y, AREA.y + super::workspace::TAB_HEIGHT);
    assert_eq!(cursor.x, AREA.x);
}
