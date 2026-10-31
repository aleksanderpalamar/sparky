use iced::widget::{Column, Space, column, row};
use iced::{Element, Length};
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::domain::system::volumes::Volume;
use crate::view::kit::controls::{action, reading};
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;

const EMPTY: &str = "Nenhuma mídia removível conectada.";
const SAFETY: &str = "Desmonte antes de desconectar, para não perder o que ainda não foi gravado.";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let volumes = settings.media.volumes();
    let problem = settings
        .media
        .error()
        .or(volumes.is_empty().then_some(EMPTY));
    if let Some(problem) = problem {
        return column![caption(skin, "Volumes"), hint(skin, problem.to_owned())]
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
        .push(hint(skin, SAFETY))
        .spacing(space::XXL)
        .into()
}

fn card<'a>(skin: Skin, index: usize, volume: &Volume) -> Element<'a, Message> {
    let label = match volume.mounted() {
        true => "Desmontar",
        false => "Montar",
    };
    let filesystem = match volume.filesystem.is_empty() {
        true => "desconhecido".to_owned(),
        false => volume.filesystem.clone(),
    };
    let place = volume
        .mountpoint
        .clone()
        .unwrap_or_else(|| "não montado".to_owned());
    let removable = match volume.removable {
        true => "sim",
        false => "não",
    };
    column![
        caption(skin, &format!("{} · {}", volume.name(), volume.size)),
        Line::new("Dispositivo", reading(skin, volume.path.clone())).view(skin),
        Line::new("Formato", reading(skin, filesystem)).view(skin),
        Line::new("Montado em", reading(skin, place)).view(skin),
        Line::new("Removível", reading(skin, removable)).view(skin),
        row![
            Space::new().width(Length::Fill),
            action(skin, label, Message::Activate(index))
        ],
    ]
    .spacing(space::MD)
    .padding(space::SM)
    .into()
}
