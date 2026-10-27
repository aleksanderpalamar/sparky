use crate::domain::Notification;
use iced::{
    Alignment, Element,
    widget::{container, image, svg, text},
};
use sparky_theme::Skin;
use std::path::Path;

pub fn icon<'a, M: 'a>(skin: &Skin, notification: &'a Notification) -> Element<'a, M> {
    let content = picture(notification).unwrap_or_else(|| fallback(skin, &notification.icon));
    container(content)
        .width(40)
        .height(40)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .into()
}

fn picture<'a, M: 'a>(notification: &'a Notification) -> Option<Element<'a, M>> {
    let picture = notification.picture.as_ref()?;
    let expected = picture.width.checked_mul(picture.height)?.checked_mul(4)? as usize;
    if picture.width == 0
        || picture.width > 64
        || picture.height == 0
        || picture.height > 64
        || picture.rgba.len() != expected
    {
        return None;
    }
    Some(
        image(image::Handle::from_rgba(
            picture.width,
            picture.height,
            picture.rgba.clone(),
        ))
        .width(36)
        .height(36)
        .into(),
    )
}

fn fallback<'a, M: 'a>(skin: &Skin, path: &str) -> Element<'a, M> {
    if path.is_empty() {
        return text("●").size(18).color(skin.accent).into();
    }
    if Path::new(path)
        .extension()
        .is_some_and(|extension| extension == "svg")
    {
        let symbolic = Path::new(path)
            .file_stem()
            .is_some_and(|name| name.to_string_lossy().ends_with("-symbolic"));
        let color = skin.foreground;
        return svg(path.to_owned())
            .width(36)
            .height(36)
            .style(move |_, _| svg::Style {
                color: symbolic.then_some(color),
            })
            .into();
    }
    image(path.to_owned()).width(36).height(36).into()
}
