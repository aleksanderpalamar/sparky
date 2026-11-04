use iced::widget::{Column, Space, column, row};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::system::volumes::Volume;
use crate::view::kit::controls::{action, reading};
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let volumes = settings.media.volumes();
    let problem = settings
        .media
        .error()
        .map(str::to_owned)
        .or_else(|| volumes.is_empty().then(|| crate::i18n::text("media-empty")));
    if let Some(problem) = problem {
        return column![
            caption(skin, &crate::i18n::text("group-volumes")),
            hint(skin, problem)
        ]
        .spacing(space::MD)
        .into();
    }
    let cards = volumes.iter().enumerate().map(|(index, volume)| {
        focusable(
            skin,
            index,
            settings.focus == index,
            card(skin, index, volume),
        )
    });
    Column::with_children(cards)
        .push(hint(skin, crate::i18n::text("media-safety")))
        .spacing(space::XXL)
        .into()
}

fn card<'a>(skin: Skin, index: usize, volume: &Volume) -> Element<'a, Message> {
    let label = match volume.mounted() {
        true => "media-unmount",
        false => "media-mount",
    };
    let filesystem = match volume.filesystem.is_empty() {
        true => crate::i18n::text("unknown"),
        false => volume.filesystem.clone(),
    };
    let place = volume
        .mountpoint
        .clone()
        .unwrap_or_else(|| crate::i18n::text("not-mounted"));
    let removable = match volume.removable {
        true => "yes",
        false => "no",
    };
    column![
        caption(skin, &format!("{} · {}", volume.name(), volume.size)),
        Line::new(
            crate::i18n::text("media-device"),
            reading(skin, volume.path.clone())
        )
        .view(skin),
        Line::new(crate::i18n::text("media-format"), reading(skin, filesystem)).view(skin),
        Line::new(crate::i18n::text("media-mounted-at"), reading(skin, place)).view(skin),
        Line::new(
            crate::i18n::text("media-removable"),
            reading(skin, crate::i18n::text(removable))
        )
        .view(skin),
        row![
            Space::new().width(Length::Fill),
            action(skin, crate::i18n::text(label), Message::Activate(index))
        ],
    ]
    .spacing(space::MD)
    .padding(space::SM)
    .into()
}
