use iced::Element;
use iced::widget::{Column, column};
use sparky_ipc::OutputInfo;
use sparky_theme::Skin;

use crate::app::message::Message;
use crate::app::state::Settings;
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;
use crate::view::screens::screens;

const NOTE: &str = "A escala de cada monitor vem do sparky.toml, na tabela [[output]].";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let outputs = match &settings.outputs {
        Ok(outputs) if outputs.is_empty() => {
            return hint(skin, "Nenhum monitor informado pelo sparky.");
        }
        Ok(outputs) => outputs,
        Err(error) => return hint(skin, error.to_string()),
    };
    let details = outputs.iter().enumerate().map(|(index, output)| {
        focusable(
            skin,
            index,
            settings.focus == index,
            details(skin, index, output),
        )
    });
    column![
        caption(skin, "Tela"),
        screens(skin, outputs),
        Column::with_children(details).spacing(space::XXL),
        hint(skin, NOTE),
    ]
    .spacing(space::MD)
    .into()
}

pub fn describe(output: &OutputInfo) -> String {
    format!("{} × {}", output.width, output.height)
}

fn details<'a>(skin: Skin, index: usize, output: &OutputInfo) -> Element<'a, Message> {
    let focus = match output.focused {
        true => "sim",
        false => "não",
    };
    column![
        caption(skin, &format!("{} · {}", index + 1, output.name)),
        Line::new("Tamanho", reading(skin, describe(output))).view(skin),
        Line::new(
            "Posição",
            reading(skin, format!("{}, {}", output.x, output.y))
        )
        .view(skin),
        Line::new("Em foco", reading(skin, focus)).view(skin),
    ]
    .spacing(space::XXS)
    .padding(space::SM)
    .into()
}

#[cfg(test)]
mod tests {
    use super::describe;
    use sparky_ipc::OutputInfo;

    #[test]
    fn descreve_o_tamanho_do_monitor() {
        let output = OutputInfo {
            name: "DP-1".to_owned(),
            x: 1920,
            y: 0,
            width: 2560,
            height: 1440,
            focused: true,
        };
        assert_eq!(describe(&output), "2560 × 1440");
    }
}
