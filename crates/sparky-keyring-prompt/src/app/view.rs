use iced::widget::{Column, column, container, text, text_input};
use iced::{Background, Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use crate::app::{Message, Window, id_of};
use crate::domain::form::Field;
use crate::i18n;

pub fn view(window: &Window) -> Element<'_, Message> {
    let skin = window.skin;
    let form = &window.form;
    let mut content = column![
        text(i18n::text(form.mode.title()))
            .size(TITLE_SIZE)
            .color(skin.bright_foreground),
        text(i18n::text(form.mode.explanation()))
            .size(TEXT_SIZE)
            .color(skin.foreground),
    ]
    .spacing(SPACING);
    content = with_line(content, form.mode.warning().map(i18n::text), skin);
    content = content.push(field(
        Field::Password,
        i18n::text("password"),
        form.password(),
    ));
    if form.mode.confirms() {
        content = content.push(field(
            Field::Confirmation,
            i18n::text("repeat-password"),
            form.confirmation(),
        ));
    }
    content = with_line(
        content,
        form.problem.map(|problem| i18n::text(problem.message())),
        skin,
    );
    let hint = match form.mode.confirms() {
        true => "long-hint",
        false => "short-hint",
    };
    content = content.push(
        text(i18n::text(hint))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    );
    container(content)
        .padding(PADDING * 1.5)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
        .into()
}

fn field<'a>(field: Field, placeholder: String, value: &'a str) -> Element<'a, Message> {
    text_input(&placeholder, value)
        .id(id_of(field))
        .secure(true)
        .on_input(move |typed| Message::Typed(field, typed))
        .on_submit(Message::Enter(field))
        .size(TEXT_SIZE)
        .padding(PADDING / 1.5)
        .style(style::input)
        .into()
}

fn with_line<'a>(
    content: Column<'a, Message>,
    line: Option<String>,
    skin: Skin,
) -> Column<'a, Message> {
    match line {
        Some(line) => content.push(text(line).size(TEXT_SIZE).color(skin.urgent)),
        None => content,
    }
}

pub fn panel(skin: Skin) -> container::Style {
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

    use super::panel;

    #[test]
    fn a_janela_tem_cantos_retos_como_as_outras() {
        assert_eq!(panel(Skin::default()).border.radius, Radius::from(0.0));
    }
}
