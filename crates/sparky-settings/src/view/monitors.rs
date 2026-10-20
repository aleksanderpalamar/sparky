use iced::alignment::Vertical;
use iced::widget::{Column, container, row, text};
use iced::{Element, Length};
use sparky_ipc::OutputInfo;
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::view::style::{KEYS_WIDTH, LINE_HEIGHT};

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    match &settings.outputs {
        Ok(outputs) if outputs.is_empty() => note(skin, "Nenhum monitor informado pelo sparky."),
        Ok(outputs) => Column::with_children(
            outputs
                .iter()
                .enumerate()
                .map(|(index, output)| line(skin, output, Highlight::when(settings.focus == index))),
        )
        .spacing(2)
        .into(),
        Err(error) => note(skin, &error.to_string()),
    }
}

pub fn describe(output: &OutputInfo) -> String {
    let focus = match output.focused {
        true => " · em foco",
        false => "",
    };
    format!("{} × {} na posição ({}, {}){focus}", output.width, output.height, output.x, output.y)
}

fn line(skin: Skin, output: &OutputInfo, highlight: Highlight) -> Element<'static, Message> {
    let content = row![
        text(output.name.clone()).size(TEXT_SIZE).color(skin.accent).width(KEYS_WIDTH),
        text(describe(output)).size(TEXT_SIZE).width(Length::Fill),
    ]
    .spacing(SPACING)
    .align_y(Vertical::Center);
    container(content)
        .height(LINE_HEIGHT)
        .padding([0.0, PADDING])
        .width(Length::Fill)
        .align_y(Vertical::Center)
        .style(move |_| style::row(skin, highlight))
        .into()
}

fn note<'a>(skin: Skin, message: &str) -> Element<'a, Message> {
    container(text(message.to_owned()).size(SMALL_TEXT).color(skin.dark_foreground))
        .padding(PADDING)
        .into()
}

#[cfg(test)]
mod tests {
    use super::describe;
    use sparky_ipc::OutputInfo;

    #[test]
    fn descreve_tamanho_posicao_e_foco() {
        let output = OutputInfo { name: "DP-1".to_owned(), x: 1920, y: 0, width: 2560, height: 1440, focused: true };
        assert_eq!(describe(&output), "2560 × 1440 na posição (1920, 0) · em foco");
    }
}
