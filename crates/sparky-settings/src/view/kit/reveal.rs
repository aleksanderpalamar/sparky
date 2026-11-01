use iced::Task;
use iced::advanced::widget::operation::scrollable::{AbsoluteOffset, scroll_to};
use iced::advanced::widget::operation::{Outcome, Scrollable};
use iced::advanced::widget::{Id, Operation, operate};
use iced::{Rectangle, Vector};

pub const CONTENT: Id = Id::new("conteudo");
pub const FOCUSED: Id = Id::new("foco");

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Reveal {
    visible: Option<Span>,
    origin: f32,
    item: Option<Rectangle>,
}

pub fn task<T: Send + 'static>() -> Task<T> {
    operate(Reveal::default())
}

pub fn target(visible: Span, item: Span) -> Option<f32> {
    let height = visible.end - visible.start;
    if item.start < visible.start || item.end - item.start > height {
        return (item.start != visible.start).then_some(item.start);
    }
    (item.end > visible.end).then_some(item.end - height)
}

impl<T: 'static> Operation<T> for Reveal {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<T>)) {
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
            self.origin = content_bounds.y;
            self.visible = Some(Span {
                start: translation.y,
                end: translation.y + bounds.height,
            });
        }
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&FOCUSED) {
            self.item = Some(bounds);
        }
    }

    fn finish(&self) -> Outcome<T> {
        let (Some(visible), Some(item)) = (self.visible, self.item) else {
            return Outcome::None;
        };
        let item = Span {
            start: item.y - self.origin,
            end: item.y + item.height - self.origin,
        };
        let Some(y) = target(visible, item) else {
            return Outcome::None;
        };
        let offset = AbsoluteOffset {
            x: None,
            y: Some(y),
        };
        Outcome::Chain(Box::new(scroll_to(CONTENT, offset)))
    }
}

#[cfg(test)]
mod tests {
    use super::{Span, target};

    fn faixa(start: f32, end: f32) -> Span {
        Span { start, end }
    }

    #[test]
    fn item_visivel_nao_rola() {
        assert_eq!(target(faixa(100.0, 400.0), faixa(150.0, 300.0)), None);
    }

    #[test]
    fn item_abaixo_rola_ate_aparecer_inteiro_embaixo() {
        assert_eq!(target(faixa(0.0, 300.0), faixa(500.0, 650.0)), Some(350.0));
    }

    #[test]
    fn item_acima_rola_ate_o_topo_dele() {
        assert_eq!(
            target(faixa(400.0, 700.0), faixa(120.0, 260.0)),
            Some(120.0)
        );
    }

    #[test]
    fn item_maior_que_a_area_alinha_pelo_topo() {
        assert_eq!(target(faixa(0.0, 200.0), faixa(300.0, 800.0)), Some(300.0));
        assert_eq!(target(faixa(300.0, 500.0), faixa(300.0, 800.0)), None);
    }
}
