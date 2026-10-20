use iced::widget::{button, row, text};
use iced::{Alignment, Color, Element, Length};
use sparky_theme::Skin;

use crate::domain::slots::{Slot, SlotState};
use crate::ui::style::{EMPTY_ALPHA, FONT, HOVER_ALPHA, SLOT_WIDTH, TEXT_SIZE};

pub fn workspaces<'a, M: Clone + 'a>(
    skin: &Skin,
    slots: &[Slot],
    on_press: impl Fn(u32) -> M,
) -> Element<'a, M> {
    let buttons = slots.iter().map(|slot| slot_button(skin, slot, on_press(slot.id)));
    row(buttons).height(Length::Fill).align_y(Alignment::Center).into()
}

fn slot_button<'a, M: Clone + 'a>(skin: &Skin, slot: &Slot, message: M) -> Element<'a, M> {
    let label = text(slot.label())
        .font(FONT)
        .size(TEXT_SIZE)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

    button(label)
        .width(SLOT_WIDTH)
        .height(Length::Fill)
        .padding(0)
        .on_press(message)
        .style(slot_style(*skin, slot.state))
        .into()
}

fn slot_style(skin: Skin, state: SlotState) -> impl Fn(&iced::Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: hovered.then(|| skin.fill(HOVER_ALPHA).into()),
            text_color: color_of(&skin, state),
            ..button::Style::default()
        }
    }
}

fn color_of(skin: &Skin, state: SlotState) -> Color {
    match state {
        SlotState::Empty => Color {
            a: EMPTY_ALPHA,
            ..skin.foreground
        },
        SlotState::Active | SlotState::Occupied => skin.foreground,
    }
}
