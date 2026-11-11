use iced::widget::{container, image, svg, text};
use iced::{ContentFit, Element};

use crate::app::{Message, Store};
use crate::domain::Application;

pub fn view<'a>(store: &Store, app: &Application, size: f32) -> Element<'a, Message> {
    if let Some(path) = store.icons.get(&app.id) {
        if path.extension().is_some_and(|extension| extension == "svg") {
            return svg(svg::Handle::from_path(path))
                .width(size)
                .height(size)
                .into();
        }
        return image(path.clone())
            .width(size)
            .height(size)
            .content_fit(ContentFit::Contain)
            .into();
    }
    let skin = store.skin;
    let initial = app.name.chars().next().unwrap_or('?').to_string();
    container(text(initial).size(size * 0.48).color(skin.accent))
        .width(size)
        .height(size)
        .center_x(size)
        .center_y(size)
        .style(move |_| super::styles::surface(skin))
        .into()
}
