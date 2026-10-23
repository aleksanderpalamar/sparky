use iced::alignment::Vertical;
use iced::widget::{Column, button, column, container, row, rule, scrollable, text, tooltip};
use iced::{Element, Length};
use sparky_storage::{Operation, Volume, VolumeKind};
use sparky_ui::style::{self, Highlight, PADDING, SIDEBAR_WIDTH, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::icons::IconKind;
use crate::view::icon::icon;

pub fn sidebar(files: &Files) -> Element<'_, Message> {
    let skin = files.skin;
    let places = files.places.iter().enumerate().map(|(index, place)| {
        let label = row![
            text((index + 1).to_string())
                .size(SMALL_TEXT)
                .color(skin.accent)
                .width(14),
            icon(files, IconKind::place(place.label), 20.0),
            text(place.label).size(13.0)
        ]
        .spacing(6)
        .align_y(Vertical::Center);
        let highlight = Highlight::when(place.path == files.location);
        button(label)
            .width(Length::Fill)
            .on_press(Message::Place(index))
            .style(move |_, status| style::entry(skin, highlight, status))
            .into()
    });
    let devices = files
        .volumes
        .iter()
        .enumerate()
        .map(|(index, volume)| device(files, volume, index));
    let mut content = column![
        text("Lugares")
            .size(SMALL_TEXT)
            .color(skin.bright_foreground),
        Column::with_children(places).spacing(3),
        rule::horizontal(1),
        text("Dispositivos")
            .size(SMALL_TEXT)
            .color(skin.bright_foreground),
        Column::with_children(devices).spacing(5)
    ]
    .spacing(SPACING);
    let progress = work_label(files.volume_work.as_ref());
    if let Some(label) = progress {
        content = content.push(text(label).size(SMALL_TEXT).color(skin.foreground));
    }
    if let Some(error) = &files.volume_error {
        content = content.push(text(error).size(SMALL_TEXT).color(skin.urgent));
    }
    if files.volumes.is_empty() && progress.is_none() && files.volume_error.is_none() {
        content = content.push(
            text("Nenhum volume disponível")
                .size(SMALL_TEXT)
                .color(skin.foreground),
        );
    }
    container(scrollable(content).height(Length::Fill))
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .padding(PADDING)
        .style(move |_| style::panel(skin))
        .into()
}

fn device<'a>(files: &'a Files, volume: &'a Volume, index: usize) -> Element<'a, Message> {
    let skin = files.skin;
    let kind = match volume.kind() {
        VolumeKind::Mtp => IconKind::Phone,
        VolumeKind::Block if volume.removable => IconKind::Removable,
        VolumeKind::Block => IconKind::Drive,
    };
    let name = match volume.label.is_empty() {
        true if !volume.size.is_empty() => format!("Volume de {}", volume.size),
        true => volume.path.clone(),
        false => volume.label.clone(),
    };
    let label = column![
        text(name).size(TEXT_SIZE),
        text(device_detail(volume))
            .size(SMALL_TEXT)
            .color(skin.foreground)
    ]
    .spacing(2);
    let selected = volume
        .mountpoint
        .as_ref()
        .is_some_and(|point| files.location.starts_with(point));
    let highlight = Highlight::when(selected);
    let body = row![icon(files, kind, 22.0), label]
        .spacing(6)
        .align_y(Vertical::Center);
    let busy = files.volume_work.is_some();
    let open = button(body)
        .width(Length::Fill)
        .on_press_maybe((!busy).then(|| Message::OpenVolume(volume.path.clone())))
        .style(move |_, status| style::entry(skin, highlight, status));
    let help = format!(
        "{} · {}\n{}\nAlt+{}: montar e abrir",
        volume.path,
        volume.filesystem,
        volume.mountpoint.as_deref().unwrap_or("Não montado"),
        index + 1
    );
    let open = tooltip(open, text(help).size(SMALL_TEXT), tooltip::Position::Right)
        .style(move |_| style::tooltip(skin));
    let mut line = row![open].align_y(Vertical::Center).spacing(2);
    if volume.mounted() {
        let eject = button(icon(files, IconKind::Eject, 18.0))
            .padding(5)
            .on_press_maybe((!busy).then(|| Message::UnmountVolume(volume.path.clone())))
            .style(move |_, status| style::entry(skin, Highlight::Plain, status));
        line = line.push(
            tooltip(
                eject,
                text("Desmontar · Alt+U para o volume aberto").size(SMALL_TEXT),
                tooltip::Position::Right,
            )
            .style(move |_| style::tooltip(skin)),
        );
    }
    line.width(Length::Fill).into()
}

fn device_detail(volume: &Volume) -> String {
    let state = match volume.mounted() {
        true => "montado",
        false => "não montado",
    };
    let summary = match volume.kind() {
        VolumeKind::Mtp => "MTP",
        VolumeKind::Block => &volume.size,
    };
    match summary.is_empty() {
        true => state.to_owned(),
        false => format!("{summary} · {state}"),
    }
}

fn work_label(operation: Option<&Operation>) -> Option<&'static str> {
    match operation {
        Some(Operation::Mount(_)) => Some("Montando…"),
        Some(Operation::Unmount(_)) => Some("Desmontando…"),
        Some(Operation::List) | None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{device_detail, work_label};
    use sparky_storage::{Operation, Volume};

    #[test]
    fn telefone_indica_mtp_sem_inventar_capacidade() {
        let mut volume = Volume {
            path: "mtp://Android_Test/".to_owned(),
            label: "Android".to_owned(),
            filesystem: "mtp".to_owned(),
            size: String::new(),
            mountpoint: None,
            removable: true,
        };
        assert_eq!(device_detail(&volume), "MTP · não montado");
        volume.mountpoint = Some("/run/user/1000/gvfs/mtp:host=Android_Test".to_owned());
        assert_eq!(device_detail(&volume), "MTP · montado");
    }

    #[test]
    fn consulta_periodica_nao_altera_a_indicacao_de_atividade() {
        assert_eq!(work_label(Some(&Operation::List)), work_label(None));
    }

    #[test]
    fn montar_e_desmontar_continuam_com_indicacao_de_progresso() {
        let device = "/dev/fixture".to_owned();
        assert_eq!(
            work_label(Some(&Operation::Mount(device.clone()))),
            Some("Montando…")
        );
        assert_eq!(
            work_label(Some(&Operation::Unmount(device))),
            Some("Desmontando…")
        );
    }
}
