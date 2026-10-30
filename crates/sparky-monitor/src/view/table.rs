use std::ops::Range;

use iced::alignment::{Horizontal, Vertical};
use iced::mouse::ScrollDelta;
use iced::widget::text::Wrapping;
use iced::widget::{Column, button, column, container, mouse_area, row, text};
use iced::{Element, Length};
use sparky_theme::Skin;
use sparky_ui::style::{self, Highlight, SMALL_TEXT, SPACING, TEXT_SIZE};
use sparky_ui::viewport::Viewport;

use super::icon::app_icon;
use super::style::{ICON_SIZE, ROW_HEIGHT, VALUE_WIDTH};
use crate::app::message::{Message, Selection};
use crate::app::state::{Focus, Monitor};
use crate::domain::table::{Column as Field, Order, Sort};
use crate::domain::units::{bytes, percent, rate};
use crate::domain::usage::Usage;

const FIELDS: [(Field, &str); 5] = [
    (Field::Cpu, "CPU"),
    (Field::Memory, "Memória"),
    (Field::Gpu, "GPU"),
    (Field::Vram, "VRAM"),
    (Field::Disk, "Disco"),
];

pub fn processes(monitor: &Monitor, rows: usize) -> Element<'_, Message> {
    let selected = monitor
        .selected_process
        .and_then(|pid| monitor.processes.iter().position(|row| row.pid == pid));
    let active = monitor.active_table() == Some(Focus::Processes);
    let lines = window(monitor.processes.len(), selected, rows).map(|index| {
        let row = &monitor.processes[index];
        let highlight = Highlight::when(active && Some(index) == selected);
        line(
            monitor.skin,
            None,
            &row.name,
            &row.usage,
            highlight,
            Selection::Process(row.pid),
        )
    });
    table(monitor, false, lines)
}

pub fn applications(monitor: &Monitor, rows: usize) -> Element<'_, Message> {
    let selected = monitor
        .selected_app
        .as_ref()
        .and_then(|id| monitor.apps.iter().position(|app| &app.id == id));
    let active = monitor.active_table() == Some(Focus::Applications);
    let lines = window(monitor.apps.len(), selected, rows).map(|index| {
        let app = &monitor.apps[index];
        let highlight = Highlight::when(active && Some(index) == selected);
        let icon = Some(app.icon.as_deref());
        line(
            monitor.skin,
            icon,
            &app.name,
            &app.usage,
            highlight,
            Selection::App(app.id.clone()),
        )
    });
    table(monitor, true, lines)
}

pub fn window(len: usize, selected: Option<usize>, rows: usize) -> Range<usize> {
    let mut viewport = Viewport::new(rows);
    viewport.follow(selected.unwrap_or(0), len);
    viewport.range(len)
}

fn table<'a>(
    monitor: &'a Monitor,
    icons: bool,
    lines: impl Iterator<Item = Element<'a, Message>>,
) -> Element<'a, Message> {
    let body = column![
        header(monitor.skin, monitor.sort, icons),
        Column::with_children(lines)
    ];
    mouse_area(body)
        .on_scroll(|delta| Message::Scrolled(lines_of(delta)))
        .into()
}

fn header<'a>(skin: Skin, sort: Sort, icons: bool) -> Element<'a, Message> {
    let mut cells = row![].spacing(SPACING).align_y(Vertical::Center);
    if icons {
        cells = cells.push(text("").width(ICON_SIZE));
    }
    cells = cells.push(title(skin, sort, Field::Name, "Nome", Length::Fill));
    for (field, label) in FIELDS {
        cells = cells.push(title(skin, sort, field, label, Length::Fixed(VALUE_WIDTH)));
    }
    cells.height(ROW_HEIGHT).into()
}

fn title<'a>(
    skin: Skin,
    sort: Sort,
    field: Field,
    label: &'a str,
    width: Length,
) -> Element<'a, Message> {
    let arrow = match (sort.column == field, sort.order) {
        (false, _) => "",
        (true, Order::Ascending) => " ▴",
        (true, Order::Descending) => " ▾",
    };
    let align = match field {
        Field::Name => Horizontal::Left,
        _ => Horizontal::Right,
    };
    button(
        text(format!("{label}{arrow}"))
            .size(SMALL_TEXT)
            .color(skin.dark_foreground)
            .width(Length::Fill)
            .align_x(align),
    )
    .padding([0, 4])
    .width(width)
    .on_press(Message::SortBy(field))
    .style(move |_, status| style::entry(skin, Highlight::Plain, status))
    .into()
}

fn line<'a>(
    skin: Skin,
    icon: Option<Option<&str>>,
    name: &str,
    usage: &Usage,
    highlight: Highlight,
    selection: Selection,
) -> Element<'a, Message> {
    let mut cells = row![].spacing(SPACING).align_y(Vertical::Center);
    if let Some(path) = icon {
        cells = cells.push(app_icon(path));
    }
    let name = text(name.to_owned())
        .size(TEXT_SIZE)
        .wrapping(Wrapping::None);
    cells = cells.push(container(name).width(Length::Fill).clip(true));
    for value in values(usage) {
        cells = cells.push(
            text(value)
                .size(TEXT_SIZE)
                .wrapping(Wrapping::None)
                .width(VALUE_WIDTH)
                .align_x(Horizontal::Right),
        );
    }
    button(cells.height(ROW_HEIGHT))
        .padding([0, 4])
        .width(Length::Fill)
        .on_press(Message::Select(selection))
        .style(move |_, status| style::entry(skin, highlight, status))
        .into()
}

fn values(usage: &Usage) -> [String; 5] {
    [
        percent(usage.cpu),
        bytes(usage.memory),
        usage.gpu.map(percent).unwrap_or_default(),
        usage.vram.map(bytes).unwrap_or_default(),
        rate(usage.disk),
    ]
}

fn lines_of(delta: ScrollDelta) -> isize {
    match delta {
        ScrollDelta::Lines { y, .. } => -(y.round() as isize),
        ScrollDelta::Pixels { y, .. } => -((y / ROW_HEIGHT).round() as isize),
    }
}
