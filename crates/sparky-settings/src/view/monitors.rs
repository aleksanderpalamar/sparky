use iced::Element;
use iced::widget::{Column, column};
use sparky_ipc::OutputInfo;

use crate::app::message::Message;
use crate::app::pickers::ScreenField;
use crate::app::state::Settings;
use crate::domain::screens::{rotation_label, scale_label};
use crate::view::kit::controls::reading;
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;
use crate::view::monitor_controls::{active, choice, high_dynamic_range, variable_rate, yes_no};
use crate::view::screens::screens;

const NOTE: &str =
    "As alterações valem na hora e ficam gravadas no sparky.toml, na tabela [[output]].";

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let outputs = match &settings.outputs {
        Ok(outputs) if outputs.is_empty() => {
            return hint(skin, "Nenhum monitor informado pelo sparky.");
        }
        Ok(outputs) => outputs,
        Err(error) => return hint(skin, error.to_string()),
    };
    let blocks = outputs.iter().enumerate().map(|(index, output)| {
        focusable(
            skin,
            index,
            settings.focus == index,
            block(settings, index, output, outputs),
        )
    });
    column![
        caption(skin, "Tela"),
        screens(skin, outputs),
        Column::with_children(blocks).spacing(space::XXL),
        hint(skin, NOTE),
    ]
    .spacing(space::MD)
    .into()
}

pub fn title(index: usize, output: &OutputInfo) -> String {
    let described = match output.description.is_empty() {
        true => output.name.clone(),
        false => format!("{} · {}", output.name, output.description),
    };
    match output.enabled {
        true => format!("{} · {described}", index + 1),
        false => format!("{} · {described} (desligada)", index + 1),
    }
}

fn block<'a>(
    settings: &'a Settings,
    index: usize,
    output: &OutputInfo,
    outputs: &[OutputInfo],
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let mode = output
        .mode
        .map_or_else(|| "—".to_owned(), |mode| mode.label());
    let mut body = column![
        caption(skin, &title(index, output)),
        Line::new(
            "Resolução",
            choice(settings, output, ScreenField::Mode, mode)
        )
        .view(skin),
        Line::new(
            "Escala",
            choice(
                settings,
                output,
                ScreenField::Scale,
                scale_label(output.scale)
            )
        )
        .view(skin),
        Line::new(
            "Giro",
            choice(
                settings,
                output,
                ScreenField::Rotation,
                rotation_label(output.rotation).to_owned()
            )
        )
        .view(skin),
        high_dynamic_range(settings, output),
    ]
    .spacing(space::XXS)
    .padding(space::SM);
    if outputs.len() > 1 {
        body = body.push(active(settings, output, outputs));
    }
    body.push(Line::new("Taxa variável", variable_rate(settings, output)).view(skin))
        .push(Line::new("Tela ligada", reading(skin, yes_no(output.enabled))).view(skin))
        .into()
}

#[cfg(test)]
mod tests {
    use super::title;
    use sparky_ipc::OutputInfo;

    #[test]
    fn o_titulo_traz_numero_conector_descricao_e_estado() {
        let output = OutputInfo {
            name: "HDMI-A-1".to_owned(),
            description: "XXX BEYOND TV".to_owned(),
            enabled: true,
            ..OutputInfo::default()
        };
        assert_eq!(title(0, &output), "1 · HDMI-A-1 · XXX BEYOND TV");
        let desligada = OutputInfo {
            enabled: false,
            description: String::new(),
            ..output
        };
        assert_eq!(title(1, &desligada), "2 · HDMI-A-1 (desligada)");
    }
}
