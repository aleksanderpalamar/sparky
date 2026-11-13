use iced::alignment::Horizontal;
use iced::widget::{Space, button, column, container, mouse_area, row, stack, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style;

use crate::app::message::Message;
use crate::app::state::{Files, Mode};
use crate::i18n;
use crate::view::style::{danger, dialog};

pub fn layer<'a>(files: &'a Files, base: Element<'a, Message>) -> Element<'a, Message> {
    let Mode::ConfirmDelete(names) = &files.mode else {
        return base;
    };
    let backdrop = mouse_area(Space::new().width(Length::Fill).height(Length::Fill))
        .on_press(Message::DeleteCancel);
    let centered = container(pane(files.skin, names))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    stack![base, backdrop, centered]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn question(names: &[String]) -> String {
    match names {
        [name] => i18n::with("delete-question-one", "name", shown(name)),
        names => i18n::with("delete-question-many", "count", names.len().to_string()),
    }
}

pub fn shown(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).to_owned()
}

fn pane(skin: Skin, names: &[String]) -> Element<'static, Message> {
    let title = text(question(names))
        .size(15)
        .width(Length::Fill)
        .align_x(Horizontal::Center);
    let warning = text(i18n::text("delete-warning"))
        .size(13)
        .color(skin.dark_foreground)
        .width(Length::Fill)
        .align_x(Horizontal::Center);
    let cancel = choice(i18n::text("cancel"), Message::DeleteCancel)
        .style(move |_, status| style::entry(skin, style::Highlight::Selected, status));
    let delete = choice(i18n::text("delete"), Message::DeleteConfirm)
        .style(move |_, status| danger(skin, status));
    let content = column![title, warning, row![cancel, delete].spacing(12)].spacing(16);
    container(content)
        .width(400)
        .padding(20)
        .style(move |_| dialog(skin, skin.lighter_background))
        .into()
}

fn choice(label: String, message: Message) -> button::Button<'static, Message> {
    button(
        text(label)
            .size(13)
            .width(Length::Fill)
            .align_x(Horizontal::Center),
    )
    .width(Length::Fill)
    .padding([8, 12])
    .on_press(message)
}

#[cfg(test)]
mod tests {
    use super::question;

    #[test]
    fn a_pergunta_cita_o_nome_de_um_item_ou_a_contagem() {
        assert_eq!(
            question(&["Fotos/praia.jpg".to_owned()]),
            "Excluir permanentemente “praia.jpg”?"
        );
        assert_eq!(
            question(&["a".to_owned(), "b".to_owned()]),
            "Excluir permanentemente 2 itens selecionados?"
        );
    }
}
