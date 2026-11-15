use std::path::Path;

use iced::widget::{Column, column, container, image, mouse_area, row, svg, text, text_input};
use iced::{Alignment, Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::prompt::PROMPT;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::Message;
use crate::app::state::Launcher;
use crate::domain::app::App;
use crate::domain::icon::Icon;
use crate::i18n;

const ROW_HEIGHT: f32 = 44.0;
const SEARCH_SIZE: f32 = 16.0;
const ICON_SIZE: f32 = 28.0;
const SYMBOLIC_SIZE: f32 = 20.0;

pub fn view(launcher: &Launcher) -> Element<'_, Message> {
    let skin = launcher.skin;
    let results = launcher.results();
    let range = launcher.list.range(results.len());
    let top = range.start;
    let rows = range.filter_map(|index| {
        results.get(index).map(|app| {
            line(
                skin,
                app,
                launcher.icons.get(&app.id).map(|path| path.as_path()),
                index - top,
                Highlight::when(index == launcher.cursor),
            )
        })
    });
    let list: Element<'_, Message> = match results.is_empty() {
        true => text(i18n::text("no-apps"))
            .size(TEXT_SIZE)
            .color(skin.dark_foreground)
            .into(),
        false => Column::with_children(rows).spacing(2).into(),
    };
    let body = match launcher.mode.search() {
        Some(placeholder) => column![search(&placeholder, &launcher.query), list],
        None => column![list],
    };
    container(body.spacing(SPACING))
        .padding(PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
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

fn line(
    skin: Skin,
    app: &App,
    icon: Option<&Path>,
    index: usize,
    highlight: Highlight,
) -> Element<'static, Message> {
    let mut lines = column![
        text(app.name.clone())
            .size(TEXT_SIZE)
            .color(skin.bright_foreground)
    ];
    if let Some(detail) = &app.detail {
        lines = lines.push(
            text(detail.clone())
                .size(SMALL_TEXT)
                .color(skin.dark_foreground),
        );
    }
    let content = row![picture(skin, app, icon), lines]
        .spacing(PADDING)
        .align_y(Alignment::Center);
    let cell = container(content)
        .height(ROW_HEIGHT)
        .width(Length::Fill)
        .padding([4.0, PADDING])
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Pick(index)).into()
}

fn picture(skin: Skin, app: &App, path: Option<&Path>) -> Element<'static, Message> {
    let symbolic = matches!(app.icon, Some(Icon::Symbolic(_)));
    let shown: Element<'static, Message> = match path {
        None => column![].into(),
        Some(path) if symbolic => svg(path.to_path_buf())
            .width(SYMBOLIC_SIZE)
            .height(SYMBOLIC_SIZE)
            .style(move |_, _| svg::Style {
                color: Some(skin.bright_foreground),
            })
            .into(),
        Some(path) if path.extension().is_some_and(|extension| extension == "svg") => {
            svg(path.to_path_buf())
                .width(ICON_SIZE)
                .height(ICON_SIZE)
                .into()
        }
        Some(path) => image(path.to_path_buf())
            .width(ICON_SIZE)
            .height(ICON_SIZE)
            .into(),
    };
    container(shown)
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .center(ICON_SIZE)
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

    use super::{ROW_HEIGHT, panel};
    use crate::domain::mode::Mode;
    use crate::domain::system::entries;
    use sparky_ui::style::PADDING;

    #[test]
    fn as_opcoes_do_menu_do_sistema_cabem_sem_sobra_na_caixa_sem_busca() {
        let opcoes = entries().len() as f32;
        let lista = opcoes * ROW_HEIGHT + (opcoes - 1.0) * 2.0;
        assert_eq!(Mode::System.size().1 as f32, lista + 2.0 * PADDING);
    }

    #[test]
    fn a_caixa_do_lancador_tem_cantos_retos_como_as_janelas() {
        assert_eq!(panel(Skin::default()).border.radius, Radius::from(0.0));
    }
}
