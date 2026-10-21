use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Column, button, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, PADDING, SPACING, TEXT_SIZE};

use crate::app::message::Message;
use crate::app::state::{Mode, Settings, Target};
use crate::domain::system::{Control, Row};
use crate::domain::value::Step;
use crate::view::fields::{input, small};
use crate::view::style::{NUMBER_WIDTH, VALUE_WIDTH};

const ROW_PADDING: f32 = 10.0;
const CHOICE_WIDTH: f32 = 200.0;

pub fn view(settings: &Settings) -> Element<'_, Message> {
    let lines = settings
        .system_rows()
        .into_iter()
        .enumerate()
        .map(|(index, entry)| line(settings, index, entry));
    Column::with_children(lines).spacing(2).into()
}

fn line(settings: &Settings, index: usize, entry: Row) -> Element<'_, Message> {
    let skin = settings.context.skin;
    let highlight = Highlight::when(settings.focus == index);
    let label = text(entry.label).size(TEXT_SIZE).color(skin.bright_foreground).width(Length::Fill);
    let value = match settings.mode {
        Mode::Editing(Target::System(id)) if id == entry.id => input(settings),
        _ => control(skin, index, entry),
    };
    let content = row![label, value].spacing(SPACING).align_y(Vertical::Center);
    let cell = container(content)
        .padding([ROW_PADDING, PADDING])
        .width(Length::Fill)
        .style(move |_| style::row(skin, highlight));
    mouse_area(cell).on_press(Message::Point(index)).into()
}

fn control(skin: Skin, index: usize, entry: Row) -> Element<'static, Message> {
    let value = |width: f32| text(entry.value.clone()).size(TEXT_SIZE).width(width).align_x(Horizontal::Center);
    let control: Element<'static, Message> = match entry.control {
        Control::Choice => row![
            small(skin, "◂", Message::Nudge(index, Step::Less)),
            value(CHOICE_WIDTH),
            small(skin, "▸", Message::Nudge(index, Step::More)),
        ]
        .align_y(Vertical::Center)
        .into(),
        Control::Level => row![
            small(skin, "−", Message::Nudge(index, Step::Less)),
            value(NUMBER_WIDTH),
            small(skin, "+", Message::Nudge(index, Step::More)),
        ]
        .align_y(Vertical::Center)
        .into(),
        Control::Text | Control::Toggle(_) => {
            let color = match entry.control {
                Control::Toggle(true) => skin.accent,
                _ => skin.foreground,
            };
            button(text(entry.value.clone()).size(TEXT_SIZE).color(color))
                .on_press(Message::Activate(index))
                .style(move |_, status| style::entry(skin, Highlight::Plain, status))
                .into()
        }
        Control::Reading => text(entry.value.clone()).size(TEXT_SIZE).color(skin.dark_foreground).into(),
    };
    container(control).width(VALUE_WIDTH).align_x(Horizontal::Right).into()
}
