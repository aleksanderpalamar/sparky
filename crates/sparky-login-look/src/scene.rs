use iced::widget::{Column, container, image, space, stack};
use iced::{Background, Color, ContentFit, Element, Length, Rectangle, Size};

use crate::backdrop::Backdrop;
use crate::cover::{Crop, cover, left};
use crate::layout::{FORM_SHARE, Part, bands};
use crate::widgets::Look;

pub fn backdrop<'a, M: 'a>(
    backdrop: Option<&'a Backdrop>,
    background: Color,
    size: Size,
    look: Look,
) -> Element<'a, M> {
    let Some(backdrop) = backdrop else {
        return container(space())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(Background::Color(background)),
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

pub fn form<'a, M: 'a>(
    height: f32,
    look: Look,
    parts: &[Part],
    mut part: impl FnMut(Part) -> Element<'a, M>,
) -> Element<'a, M> {
    let mut column = Column::new().width(look.layout.form_width);
    let mut cursor = 0.0;
    for band in bands(height)
        .into_iter()
        .filter(|band| parts.contains(&band.part))
    {
        let gap = (band.top - cursor).max(0.0);
        let content = container(part(band.part))
            .center_x(Length::Fill)
            .center_y(band.height);
        column = column.push(space().height(gap)).push(content);
        cursor = band.top + band.height;
    }
    column.into()
}

fn rectangle(crop: Crop) -> Rectangle<u32> {
    Rectangle {
        x: crop.x,
        y: crop.y,
        width: crop.width,
        height: crop.height,
    }
}
