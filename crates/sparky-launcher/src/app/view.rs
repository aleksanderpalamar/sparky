use iced::widget::{Column, column, container, mouse_area, text, text_input};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::Message;
use crate::app::state::Launcher;
use crate::domain::app::App;

const ROW_HEIGHT: f32 = 44.0;
const SEARCH_SIZE: f32 = 16.0;

pub fn view(launcher: &Launcher) -> Element<'_, Message> {
    let skin = launcher.skin;
    let search = text_input(launcher.mode.placeholder(), &launcher.query)
        .id(PROMPT)
        .on_input(Message::Typed)
        .on_submit(Message::Submit)
        .size(SEARCH_SIZE)
        .style(style::input)
        .padding(PADDING / 1.5);
    let results = launcher.results();
    let range = launcher.list.range(results.len());
    let top = range.start;
    let rows = range.filter_map(|index| results.get(index).map(|app| line(skin, app, index - top, Highlight::when(index == launcher.cursor))));
    let list: Element<'_, Message> = match results.is_empty() {
        true => text("Nenhum aplicativo encontrado").size(TEXT_SIZE).color(skin.dark_foreground).into(),
        false => Column::with_children(rows).spacing(2).into(),
    };
    container(column![search, list].spacing(SPACING))
        .padding(PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
        .into()
}

fn line(skin: Skin, app: &App, row: usize, highlight: Highlight) -> Element<'static, Message> {
    let mut lines = column![text(app.name.clone()).size(TEXT_SIZE).color(skin.bright_foreground)];
    if let Some(detail) = &app.detail {
        lines = lines.push(text(detail.clone()).size(SMALL_TEXT).color(skin.dark_foreground));
    }
    let cell = container(lines)
        .height(ROW_HEIGHT)
        .width(Length::Fill)
        .padding([4.0, PADDING])
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Pick(row)).into()
}

fn panel(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.background)),
        border: Border { color: skin.accent, width: 2.0, ..Border::default() },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::border::Radius;
    use sparky_theme::Skin;

    use super::panel;

    #[test]
    fn a_caixa_do_lancador_tem_cantos_retos_como_as_janelas() {
        assert_eq!(panel(Skin::default()).border.radius, Radius::from(0.0));
    }
}
