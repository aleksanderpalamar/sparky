use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, stack, text};
use iced::{Element, Length, Padding};
use sparky_ui::style::{self, STATUS_HEIGHT};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::toast::Toast;
use crate::i18n;
use crate::view::delete_dialog::shown;
use crate::view::style::dialog;

const GAP: f32 = 16.0;

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Some(toast) = files.toast.as_ref() else {
        return base;
    };
    let skin = files.skin;
    let undo = button(text(i18n::text("undo")).size(13))
        .padding([4, 12])
        .on_press(Message::ToastUndo)
        .style(move |_, status| style::entry(skin, style::Highlight::Selected, status));
    let close = button(text("×").size(13))
        .padding([4, 8])
        .on_press(Message::ToastDismiss)
        .style(move |_, status| style::entry(skin, style::Highlight::Plain, status));
    let bar = container(
        row![text(label(toast)).size(13), undo, close]
            .spacing(12)
            .align_y(Vertical::Center),
    )
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
    match toast.names.as_slice() {
        [name] => i18n::with("trashed-one", "name", shown(name)),
        names => i18n::with("trashed-many", "count", names.len().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::label;
    use crate::domain::toast::Toast;

    #[test]
    fn o_aviso_cita_o_nome_de_um_item_ou_a_contagem() {
        let um = Toast::trashed(vec!["Fotos/praia.jpg".to_owned()], Vec::new());
        assert_eq!(label(&um), "“praia.jpg” foi para a lixeira");
        let dois = Toast::trashed(vec!["a".to_owned(), "b".to_owned()], Vec::new());
        assert_eq!(label(&dois), "2 itens foram para a lixeira");
    }
}
