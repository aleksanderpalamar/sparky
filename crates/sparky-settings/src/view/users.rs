use iced::Element;
use iced::widget::{Column, column};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::system::users::User;
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    if settings.users.is_empty() {
        return hint(skin, crate::i18n::text("users-empty"));
    }
    let accounts =
        settings.users.iter().enumerate().map(|(index, user)| {
            focusable(skin, index, settings.focus == index, account(skin, user))
        });
    Column::with_children(accounts)
        .push(hint(skin, crate::i18n::text("users-note")))
        .spacing(space::XXL)
        .into()
}

fn account<'a>(skin: Skin, user: &User) -> Element<'a, Message> {
    let groups = user
        .groups
        .as_ref()
        .map(|groups| groups.join(", "))
        .unwrap_or_else(|| crate::i18n::text("unavailable"));
    let facts = [
        ("users-id", user.account.uid.to_string()),
        ("users-role", user.role()),
        ("users-session", user.session()),
        ("users-home", user.account.home.clone()),
        ("users-shell", user.account.shell.clone()),
        ("users-groups", groups),
    ];
    let rows = facts
        .into_iter()
        .map(|(label, value)| Line::new(crate::i18n::text(label), reading(skin, value)).view(skin));
    column![
        caption(skin, &user.title()),
        Column::with_children(rows).spacing(space::XXS)
    ]
    .spacing(space::XXS)
    .padding(space::SM)
    .into()
}
