use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Background, Border, Element, Length};
use sparky_theme::Skin;

use crate::domain::delay::Delay;
use crate::domain::mode::Mode;
use crate::picker::Message;
use crate::picker::keys::{Command, hint};

const SPACING: f32 = 8.0;
const PADDING: f32 = 16.0;

pub fn view(skin: Skin, delay: Delay) -> Element<'static, Message> {
    let options = [
        (Mode::Region, "Região"),
        (Mode::Window, "Janela"),
        (Mode::Output, "Monitor"),
    ]
    .map(|(mode, label)| option(skin, hint(mode), label, Command::Pick(mode)));
    let content = column(options)
        .push(option(skin, "Tab", &format!("Atraso: {}", delay.label()), Command::NextDelay))
        .push(option(skin, "Esc", "Cancelar", Command::Cancel))
        .spacing(SPACING);
    container(column![text("Capturar a tela").color(skin.bright_foreground), content].spacing(SPACING * 1.5))
        .padding(PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| panel(skin))
        .into()
}

fn option(skin: Skin, key: &str, label: &str, command: Command) -> Element<'static, Message> {
    let line = row![
        text(key.to_owned()).color(skin.accent).width(Length::Fixed(40.0)),
        text(label.to_owned()).color(skin.foreground),
    ]
    .align_y(Alignment::Center);
    button(line)
        .width(Length::Fill)
        .on_press(Message::Key(command))
        .style(move |_, status| entry(skin, status))
        .into()
}

fn panel(skin: Skin) -> container::Style {
    container::Style {
        background: Some(Background::Color(skin.background)),
        border: Border { color: skin.accent, width: 2.0, ..Border::default() },
        ..container::Style::default()
    }
}

fn entry(skin: Skin, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => skin.lighter_background,
        _ => skin.background,
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: skin.foreground,
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::border::Radius;
    use iced::widget::button;
    use sparky_theme::Skin;

    use super::{entry, panel};

    #[test]
    fn o_seletor_tem_cantos_retos_como_as_janelas() {
        let skin = Skin::default();
        assert_eq!(panel(skin).border.radius, Radius::from(0.0));
        assert_eq!(entry(skin, button::Status::Hovered).border.radius, Radius::from(0.0));
    }
}
