use iced::Element;
use iced::widget::{Column, column};
use sparky_ipc::{OutputInfo, Vrr};

use crate::app::message::Message;
use crate::app::pickers::ScreenField;
use crate::app::state::Settings;
use crate::domain::screens::{ScreenChange, last_enabled, rotation_label, scale_label};
use crate::view::kit::controls::reading;
use crate::view::kit::inputs::{locked_switch, picker, switch};
use crate::view::kit::rows::{Line, caption, focusable, hint};
use crate::view::kit::tokens::space;
use crate::view::screens::screens;

const NOTE: &str =
    "As alterações valem na hora e ficam gravadas no sparky.toml, na tabela [[output]].";
const HDR_NOTE: &str =
    "O compositor do Sparky ainda não envia HDR; o suporte próprio está planejado.";

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
        Line::new("HDR", locked_switch(skin, false))
            .note(HDR_NOTE)
            .view(skin),
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

fn choice<'a>(
    settings: &'a Settings,
    output: &OutputInfo,
    field: ScreenField,
    current: String,
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let Some(state) = settings.pickers.screen(&output.name, field) else {
        return reading(skin, current);
    };
    let name = output.name.clone();
    picker(skin, state, &current, move |value| {
        let change = match field {
            ScreenField::Mode => ScreenChange::Mode(value),
            ScreenField::Scale => ScreenChange::Scale(value),
            ScreenField::Rotation => ScreenChange::Rotation(value),
        };
        Message::Screen(name.clone(), change)
    })
}

fn active<'a>(
    settings: &Settings,
    output: &OutputInfo,
    outputs: &[OutputInfo],
) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let only = output.enabled && last_enabled(outputs, &output.name);
    let (control, note) = match only {
        true => (
            locked_switch(skin, true),
            "É a única tela ligada; desligá-la deixaria você sem imagem.",
        ),
        false => (
            switch(
                skin,
                output.enabled,
                Message::Screen(output.name.clone(), ScreenChange::Enabled(!output.enabled)),
            ),
            "Desligar remove a tela do espaço de trabalho.",
        ),
    };
    Line::new("Ativa", control).note(note).view(skin)
}

fn variable_rate<'a>(settings: &Settings, output: &OutputInfo) -> Element<'a, Message> {
    let skin = settings.context.skin;
    let on = match output.vrr {
        Vrr::Unsupported => return reading(skin, "não suportada"),
        Vrr::On => true,
        Vrr::Off => false,
    };
    switch(
        skin,
        on,
        Message::Screen(output.name.clone(), ScreenChange::Vrr(!on)),
    )
}

fn yes_no(value: bool) -> &'static str {
    match value {
        true => "sim",
        false => "não",
    }
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
