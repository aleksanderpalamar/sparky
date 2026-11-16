mod grid;
mod list;
mod overlay;
mod picture;
mod shelves;

pub use grid::ROW_STEP;

use iced::widget::{Column, container, mouse_area, text, text_input};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Launcher;
use crate::domain::shelf::Shelf;
use crate::i18n;

const SEARCH_SIZE: f32 = 16.0;

pub fn view(launcher: &Launcher) -> Element<'_, Message> {
    let skin = launcher.skin;
    let results = launcher.results();
    let found: Element<'_, Message> = match (results.is_empty(), launcher.mode.columns()) {
        (true, _) => empty(launcher),
        (false, 1) => list::list(launcher, &results),
        (false, _) => grid::grid(launcher, &results),
    };
    let mut body = Column::new().spacing(SPACING);
    if let Some(placeholder) = launcher.mode.search() {
        body = body.push(search(&placeholder, &launcher.query));
    }
    body = body.push(container(found).width(Length::Fill).height(Length::Fill));
    if launcher.mode.has_shelves() {
        body = body.push(shelves::bar(launcher));
    }
    let base = container(body)
        .padding(PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin));
    let tracked = mouse_area(base).on_move(Message::Pointer).into();
    overlay::layer(launcher, tracked)
}

fn empty(launcher: &Launcher) -> Element<'_, Message> {
    let key = match (launcher.shelf, launcher.query.trim().is_empty()) {
        (Shelf::Group(_), true) => "group-empty",
        _ => "no-apps",
    };
    text(i18n::text(key))
        .size(TEXT_SIZE)
        .color(launcher.skin.dark_foreground)
        .into()
}

fn search<'a>(placeholder: &str, query: &str) -> Element<'a, Message> {
    text_input(placeholder, query)
        .id(PROMPT)
        .on_input(Message::Typed)
        .on_submit(Message::Submit)
        .size(SEARCH_SIZE)
        .style(style::input)
        .padding(PADDING / 1.5)
        .into()
}

fn panel(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.background)),
        border: Border {
            color: skin.accent,
            width: 2.0,
            ..Border::default()
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::border::Radius;
    use sparky_theme::Skin;

    use super::list::{ROW_GAP, ROW_HEIGHT};
    use super::panel;
    use crate::domain::mode::Mode;
    use crate::domain::system::entries;
    use sparky_ui::style::PADDING;

    #[test]
    fn as_opcoes_do_menu_do_sistema_cabem_sem_sobra_na_caixa_sem_busca() {
        let opcoes = entries().len() as f32;
        let lista = opcoes * ROW_HEIGHT + (opcoes - 1.0) * ROW_GAP;
        assert_eq!(Mode::System.size().1 as f32, lista + 2.0 * PADDING);
    }

    #[test]
    fn a_caixa_do_lancador_tem_cantos_retos_como_as_janelas() {
        assert_eq!(panel(Skin::default()).border.radius, Radius::from(0.0));
    }
}
