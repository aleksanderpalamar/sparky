use std::path::Path;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, stack, text};
use iced::{Element, Length, Padding};
use sparky_theme::Skin;
use sparky_ui::style::{self, STATUS_HEIGHT};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::toast::{Toast, ToastKind};
use crate::i18n;
use crate::view::delete_dialog::shown;
use crate::view::style::dialog;

const GAP: f32 = 16.0;

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Some(toast) = files.toast.as_ref() else {
        return base;
    };
    let skin = files.skin;
    let mut content = row![text(label(toast)).size(13)]
        .spacing(12)
        .align_y(Vertical::Center);
    if let Some((caption, message)) = action(toast) {
        content = content.push(
            button(text(i18n::text(caption)).size(13))
                .padding([4, 12])
                .on_press(message)
                .style(move |_, status| style::entry(skin, style::Highlight::Selected, status)),
        );
    }
    let bar = container(content.push(close(skin)))
        .padding([8, 12])
        .style(move |_| dialog(skin, skin.lighter_background));
    let placed = container(bar)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .padding(Padding::ZERO.bottom(STATUS_HEIGHT + GAP));
    stack![base, placed]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn label(toast: &Toast) -> String {
    match (&toast.kind, toast.folder()) {
        (ToastKind::Trashed { names, .. }, _) => match names.as_slice() {
            [name] => i18n::with("trashed-one", "name", shown(name)),
            names => i18n::with("trashed-many", "count", names.len().to_string()),
        },
        (ToastKind::Restored(paths), Some(folder)) => match paths.as_slice() {
            [path] => restored_one(path, folder),
            paths => restored_many(paths.len(), folder),
        },
        (ToastKind::Restored(paths), None) => {
            i18n::with("restored-many-places", "count", paths.len().to_string())
        }
    }
}

fn action(toast: &Toast) -> Option<(&'static str, Message)> {
    match &toast.kind {
        ToastKind::Trashed { .. } => Some(("undo", Message::ToastUndo)),
        ToastKind::Restored(_) => toast
            .folder()
            .map(|_| ("open-folder", Message::ToastOpenFolder)),
    }
}

fn close(skin: Skin) -> Element<'static, Message> {
    button(text("×").size(13))
        .padding([4, 8])
        .on_press(Message::ToastDismiss)
        .style(move |_, status| style::entry(skin, style::Highlight::Plain, status))
        .into()
}

fn restored_one(path: &Path, folder: &Path) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("name", name_of(path));
    args.set("folder", name_of(folder));
    i18n::message("restored-one", Some(&args))
}

fn restored_many(count: usize, folder: &Path) -> String {
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", count);
    args.set("folder", name_of(folder));
    i18n::message("restored-many", Some(&args))
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::label;
    use crate::domain::toast::Toast;

    #[test]
    fn o_aviso_da_lixeira_cita_o_nome_de_um_item_ou_a_contagem() {
        let um = Toast::trashed(vec!["Fotos/praia.jpg".to_owned()], Vec::new());
        assert_eq!(label(&um), "“praia.jpg” foi para a lixeira");
        let dois = Toast::trashed(vec!["a".to_owned(), "b".to_owned()], Vec::new());
        assert_eq!(label(&dois), "2 itens foram para a lixeira");
    }

    #[test]
    fn o_aviso_da_restauracao_cita_a_pasta_de_destino() {
        let um = Toast::restored(vec![PathBuf::from("/h/Capturas de tela/a.png")]);
        assert_eq!(label(&um), "“a.png” foi movido para “Capturas de tela”");
        let dois = Toast::restored(vec![PathBuf::from("/h/F/a"), PathBuf::from("/h/F/b")]);
        assert_eq!(label(&dois), "2 itens movidos para “F”");
        let soltos = Toast::restored(vec![PathBuf::from("/h/F/a"), PathBuf::from("/h/G/b")]);
        assert_eq!(label(&soltos), "2 itens restaurados");
    }
}
