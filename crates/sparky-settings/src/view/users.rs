use iced::Element;
use iced::widget::{Column, column};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::system::users::User;
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;

const NOTE: &str =
    "Criar conta, trocar senha e mudar grupos continuam no terminal: useradd, passwd e usermod.";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if settings.users.is_empty() {
        return hint(skin, "Nenhuma conta de usuário encontrada.");
    }
    let accounts =
        settings.users.iter().enumerate().map(|(index, user)| {
            focusable(skin, index, settings.focus == index, account(skin, user))
        });
    Column::with_children(accounts)
        .push(hint(skin, NOTE))
        .spacing(space::XXL)
        .into()
}

fn account<'a>(skin: Skin, user: &User) -> Element<'a, Message> {
    let groups = user
        .groups
        .as_ref()
        .map(|groups| groups.join(", "))
        .unwrap_or_else(|| "não disponível".to_owned());
    let facts = [
        ("Identificador", user.account.uid.to_string()),
        ("Tipo", user.role().to_owned()),
        ("Sessão", user.session().to_owned()),
        ("Pasta pessoal", user.account.home.clone()),
        ("Interpretador", user.account.shell.clone()),
        ("Grupos", groups),
    ];
    let rows = facts
        .into_iter()
        .map(|(label, value)| Line::new(label, reading(skin, value)).view(skin));
    column![
        caption(skin, &user.title()),
        Column::with_children(rows).spacing(space::XXS)
    ]
    .spacing(space::XXS)
    .padding(space::SM)
    .into()
}
