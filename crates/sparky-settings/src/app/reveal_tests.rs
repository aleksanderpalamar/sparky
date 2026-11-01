use std::rc::Rc;

use iced::advanced::Layout;
use iced::advanced::layout::{Limits, Node};
use iced::advanced::renderer::Headless;
use iced::advanced::widget::operation::{Outcome, Scrollable};
use iced::advanced::widget::{Id, Operation, Tree};
use iced::futures::executor::block_on;
use iced::keyboard::{Key, Modifiers};
use iced::{Element, Pixels, Rectangle, Renderer, Size, Vector};
use sparky_ui::style::FONT;

use crate::app::message::Message;
use crate::app::test_support::{Comandos, Sparky, painel_com, tecla};
use crate::app::update::update;
use crate::domain::section::Section;
use crate::ports::fake::FakeCommands;
use crate::view::kit::reveal::{self, CONTENT, FOCUSED};

struct Tela<'a> {
    view: Element<'a, Message>,
    tree: Tree,
    node: Node,
    renderer: Renderer,
}

impl<'a> Tela<'a> {
    fn new(mut view: Element<'a, Message>) -> Self {
        let renderer = block_on(<Renderer as Headless>::new(
            FONT,
            Pixels(12.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut tree = Tree::new(&view);
        let node = view.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &Limits::new(Size::ZERO, Size::new(700.0, 500.0)),
        );
        Self {
            view,
            tree,
            node,
            renderer,
        }
    }

    fn operar(&mut self, mut operation: Box<dyn Operation>) {
        loop {
            self.view.as_widget_mut().operate(
                &mut self.tree,
                Layout::new(&self.node),
                &self.renderer,
                operation.as_mut(),
            );
            let Outcome::Chain(next) = operation.finish() else {
                return;
            };
            operation = next;
        }
    }

    fn foco_visivel(&mut self) -> bool {
        let mut medida = Medida::default();
        self.view.as_widget_mut().operate(
            &mut self.tree,
            Layout::new(&self.node),
            &self.renderer,
            &mut medida,
        );
        let (Some((area, conteudo, deslocamento)), Some(item)) = (medida.area, medida.item) else {
            return false;
        };
        let topo = item.y - conteudo.y;
        topo >= deslocamento.y && topo + item.height <= deslocamento.y + area.height
    }
}

#[derive(Default)]
struct Medida {
    area: Option<(Rectangle, Rectangle, Vector)>,
    item: Option<Rectangle>,
}

impl Operation for Medida {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        translation: Vector,
        _: &mut dyn Scrollable,
    ) {
        if id == Some(&CONTENT) {
            self.area = Some((bounds, content_bounds, translation));
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&FOCUSED) {
            self.item = Some(bounds);
        }
    }
}

fn contas(total: u32) -> String {
    (0..total)
        .map(|n| format!("u{n}:x:{uid}:{uid}::/home/u{n}:/bin/sh\n", uid = 1000 + n))
        .collect()
}

#[test]
fn a_conta_escolhida_pelo_teclado_rola_para_a_area_visivel() {
    let commands = Comandos(Rc::new(
        FakeCommands::new().with("getent passwd", &contas(6)),
    ));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands);
    let _ = update(&mut settings, Message::Section(Section::Users));
    for _ in 0..5 {
        tecla(&mut settings, "j");
    }
    assert_eq!(settings.focus, 5);
    let mut tela = Tela::new(crate::view::view(&settings));
    assert!(!tela.foco_visivel());
    tela.operar(Box::new(reveal::Reveal::default()));
    assert!(tela.foco_visivel());
    tela.operar(Box::new(reveal::Reveal::default()));
    assert!(tela.foco_visivel());
}

#[test]
fn mover_o_foco_pelo_teclado_pede_a_rolagem() {
    let commands = Comandos(Rc::new(
        FakeCommands::new().with("getent passwd", &contas(3)),
    ));
    let (mut settings, _) = painel_com(None, Sparky::default(), commands);
    let _ = update(&mut settings, Message::Section(Section::Users));
    for tecla in ["j", "k"] {
        let tarefa = update(
            &mut settings,
            Message::Key(Key::Character(tecla.into()), Modifiers::empty()),
        );
        assert!(tarefa.units() > 0, "{tecla}");
    }
}
