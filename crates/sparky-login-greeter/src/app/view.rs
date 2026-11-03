use iced::widget::{Column, container, image, responsive, space, stack};
use iced::{Background, ContentFit, Element, Length, Rectangle, Size};

use crate::app::power;
use crate::app::style::Colors;
use crate::app::widgets::{self, Look};
use crate::app::{Greeter, Message};
use crate::domain::clock;
use crate::domain::cover::{Crop, cover, left};
use crate::domain::layout::{FORM_SHARE, Layout, Part, bands};

pub fn view(greeter: &Greeter) -> Element<'_, Message> {
    responsive(move |size| screen(greeter, size)).into()
}

fn screen(greeter: &Greeter, size: Size) -> Element<'_, Message> {
    let look = Look {
        layout: Layout::new(size.width, size.height),
        colors: Colors::new(greeter.skin),
    };
    stack![
        backdrop(greeter, size, look),
        form(greeter, size.height, look)
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn backdrop(greeter: &Greeter, size: Size, look: Look) -> Element<'_, Message> {
    let Some(backdrop) = &greeter.backdrop else {
        let color = greeter.skin.background;
        return container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(color)),
                ..container::Style::default()
            })
            .into();
    };
    let whole = cover(backdrop.size, (size.width, size.height));
    let strip = left(whole, FORM_SHARE).scaled(backdrop.reduction);
    stack![
        image(backdrop.sharp.clone())
            .crop(rectangle(whole))
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(ContentFit::Fill),
        image(backdrop.blurred.clone())
            .crop(rectangle(strip))
            .width(look.layout.form_width)
            .height(Length::Fill)
            .content_fit(ContentFit::Fill),
    ]
    .into()
}

fn rectangle(crop: Crop) -> Rectangle<u32> {
    Rectangle {
        x: crop.x,
        y: crop.y,
        width: crop.width,
        height: crop.height,
    }
}

fn form(greeter: &Greeter, height: f32, look: Look) -> Element<'_, Message> {
    let mut column = Column::new().width(look.layout.form_width);
    let mut cursor = 0.0;
    for band in bands(height) {
        let gap = (band.top - cursor).max(0.0);
        let content = container(part(greeter, band.part, look))
            .center_x(Length::Fill)
            .center_y(band.height);
        column = column.push(space().height(gap)).push(content);
        cursor = band.top + band.height;
    }
    column.into()
}

fn part(greeter: &Greeter, part: Part, look: Look) -> Element<'_, Message> {
    let layout = look.layout;
    match part {
        Part::Time => widgets::clock(clock::time(greeter.now.time()), layout.time, look),
        Part::Date => widgets::clock(clock::date(greeter.now.date_naive()), layout.date, look),
        Part::Warning => widgets::warning(
            greeter.form.warning().or_else(|| greeter.notice.clone()),
            look,
        ),
        Part::Username => widgets::username(greeter, look),
        Part::Password => widgets::password(greeter, look),
        Part::Login => widgets::login(greeter, look),
        Part::PowerIcons => power::icons(look),
        Part::PowerLabels => power::labels(greeter.focus, look),
        Part::Session => widgets::session(greeter, look),
    }
}
