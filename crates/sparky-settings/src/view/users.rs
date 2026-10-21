use iced::alignment::Vertical;
use iced::widget::{Column, button, column, row, text};
use iced::{Element, Length};
use sparky_ui::style::{SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::view::style::KEYS_WIDTH;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let Some(user) = settings.users.get(settings.focus) else {
        return text("Nenhuma conta de usuário encontrada.").into();
    };
    let skin = settings.context.skin;
    let previous = settings.focus.saturating_sub(1);
    let next = (settings.focus + 1).min(settings.users.len() - 1);
    let header = row![
        button("◂").on_press(Message::Point(previous)),
        text(format!("{} · {} de {}", user.title(), settings.focus + 1, settings.users.len()))
            .size(TEXT_SIZE).color(skin.accent).width(Length::Fill),
        button("▸").on_press(Message::Point(next)),
    ].spacing(SPACING).align_y(Vertical::Center);
    let facts = [
        ("Identificador", user.account.uid.to_string()),
        ("Tipo", user.role().to_owned()),
        ("Sessão", user.session().to_owned()),
        ("Pasta pessoal", user.account.home.clone()),
        ("Interpretador", user.account.shell.clone()),
        ("Grupos", user.groups.as_ref().map(|groups| groups.join(", ")).unwrap_or_else(|| "não disponível".to_owned())),
    ];
    let details = facts.into_iter().map(|(label, value)| row![
        text(label).size(TEXT_SIZE).color(skin.accent).width(KEYS_WIDTH),
        text(value).size(TEXT_SIZE).color(skin.foreground).width(Length::Fill),
    ].spacing(SPACING).into());
    column![
        header,
        Column::with_children(details).spacing(SPACING),
        text("Consulta de contas. Criação, senhas e grupos são administrados fora deste painel.")
            .size(SMALL_TEXT).color(skin.dark_foreground),
    ].spacing(SPACING).into()
}
