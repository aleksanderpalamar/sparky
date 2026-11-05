use fluent_bundle::FluentArgs;
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Border, Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SMALL_TEXT, SPACING, TEXT_SIZE, TITLE_SIZE};

use super::Message;
use crate::domain::{Form, Phase, Secret};
use crate::i18n;

pub fn view(form: &Form, skin: Skin) -> Element<'_, Message> {
    let user: Element<'_, Message> = match form.request.identities.len() {
        1 => {
            let mut args = FluentArgs::new();
            args.set("name", form.identity.to_string());
            text(i18n::message("user", Some(&args)))
                .size(TEXT_SIZE)
                .into()
        }
        _ => pick_list(
            form.request.identities.as_slice(),
            Some(&form.identity),
            Message::Select,
        )
        .style(|theme, status| {
            let standard = pick_list::default(theme, status);
            pick_list::Style {
                border: Border {
                    radius: 0.0.into(),
                    ..standard.border
                },
                ..standard
            }
        })
        .into(),
    };
    let mut content = column![
        text(i18n::text("title"))
            .size(TITLE_SIZE)
            .color(skin.bright_foreground),
        text(&form.request.message).size(TEXT_SIZE),
        text(&form.request.action_id)
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
        user,
    ]
    .spacing(SPACING);
    let entry = matches!(form.phase, Phase::Input { .. });
    let echo = matches!(form.phase, Phase::Input { echo: true });
    let label = match form.prompt.is_empty() {
        true => i18n::text("password"),
        false => form.prompt.clone(),
    };
    content = content.push(text(label).size(TEXT_SIZE));
    content = content.push(
        text_input("", form.response())
            .id("polkit-response")
            .secure(!echo)
            .on_input_maybe(entry.then_some(|value| Message::Typed(Secret::new(value))))
            .on_submit_maybe(entry.then_some(Message::Submit))
            .padding(8)
            .size(TEXT_SIZE)
            .style(style::input),
    );
    if let Some(info) = &form.info {
        content = content.push(text(info).size(SMALL_TEXT));
    }
    if let Some(problem) = &form.problem {
        content = content.push(text(problem).size(SMALL_TEXT).color(skin.urgent));
    }
    let hint = match form.phase {
        Phase::Waiting => "waiting",
        Phase::Verifying => "verifying",
        _ => "hint",
    };
    content = content.push(
        text(i18n::text(hint))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground),
    );
    let submit = match form.phase {
        Phase::Retry => "retry",
        _ => "authenticate",
    };
    let confirm = button(text(i18n::text(submit)).size(TEXT_SIZE))
        .padding(8)
        .on_press_maybe((entry || form.phase == Phase::Retry).then_some(Message::Submit))
        .style(move |_, status| style::entry(skin, Highlight::Selected, status));
    let cancel = button(text(i18n::text("cancel")).size(TEXT_SIZE))
        .padding(8)
        .on_press(Message::Close)
        .style(move |_, status| style::entry(skin, Highlight::Plain, status));
    content = content.push(row![confirm, cancel].spacing(SPACING));
    container(scrollable(content))
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(PADDING * 1.5)
        .style(move |_| panel(skin))
        .into()
}

pub fn panel(skin: Skin) -> container::Style {
    container::Style {
        border: Border {
            color: skin.accent,
            width: 2.0,
            ..Border::default()
        },
        ..style::window(skin)
    }
}

#[cfg(test)]
mod tests {
    use super::panel;
    use iced::{Background, border::Radius};
    use sparky_theme::{Palette, Skin};

    #[test]
    fn janela_segue_as_cores_do_tema_e_os_cantos_retos() {
        for skin in [
            Skin::default(),
            Skin::new(Palette::from_colors(
                "background = \"#ffffff\"\naccent = \"#112233\"",
            )),
        ] {
            let style = panel(skin);
            assert_eq!(style.background, Some(Background::Color(skin.background)));
            assert_eq!(style.border.color, skin.accent);
            assert_eq!(style.border.radius, Radius::from(0.0));
        }
    }
}
