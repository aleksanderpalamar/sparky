use iced::widget::{image, svg};
use iced::{ContentFit, Element};

use crate::app::message::Message;
use crate::app::state::Files;
use crate::domain::entry::Entry;
use crate::domain::icons::IconKind;

pub fn entry_icon(
    files: &Files,
    entry: &Entry,
    place: Option<&str>,
    size: f32,
) -> Element<'static, Message> {
    let path = files.location.join(&entry.name);
    if let Some(handle) = files.previews.handle(&path) {
        return image(handle)
            .width(size)
            .height(size)
            .content_fit(ContentFit::Contain)
            .into();
    }
    icon(files, IconKind::for_entry(entry, place), size)
}

pub fn icon(files: &Files, kind: IconKind, size: f32) -> Element<'static, Message> {
    match files.icons.get(&kind) {
        Some(path) if path.extension().is_some_and(|extension| extension == "svg") => {
            let symbolic = path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with("-symbolic.svg"));
            let color = (symbolic || kind == IconKind::Eject).then_some(files.skin.foreground);
            svg(svg::Handle::from_path(path))
                .width(size)
                .height(size)
                .style(move |_, _| svg::Style { color })
                .into()
        }
        Some(path) => image(path.clone()).width(size).height(size).into(),
        None => fallback(files, kind, size),
    }
}

fn fallback(files: &Files, kind: IconKind, size: f32) -> Element<'static, Message> {
    let skin = files.skin;
    let [red, green, blue, _] = skin.accent.into_rgba8();
    let color = format!("#{red:02x}{green:02x}{blue:02x}");
    let drawing = match kind {
        IconKind::Folder
        | IconKind::Home
        | IconKind::Desktop
        | IconKind::Documents
        | IconKind::Downloads
        | IconKind::Pictures
        | IconKind::Music
        | IconKind::Videos => {
            "<path d='M6 17h20l6 7h26v30H6z'/><path d='M6 24h52' stroke='#ffffff' opacity='.35'/>"
        }
        IconKind::Drive | IconKind::Removable => {
            "<path d='M13 8h38v48H13z'/><circle cx='32' cy='28' r='12' fill='none' stroke='#ffffff' stroke-width='2'/><circle cx='32' cy='28' r='3' fill='#ffffff'/><path d='M21 47h22' stroke='#ffffff' stroke-width='3'/>"
        }
        IconKind::Eject => "<path d='M12 40l20-25 20 25zM12 47h40v7H12z'/>",
        IconKind::Phone => {
            "<path d='M18 5h28v54H18z'/><path d='M22 12h20v35H22z' fill='#ffffff' opacity='.35'/><circle cx='32' cy='53' r='2' fill='#ffffff'/>"
        }
        IconKind::Image => {
            "<path d='M7 10h50v44H7z'/><circle cx='43' cy='22' r='5' fill='#ffffff'/><path d='M12 48l14-20 10 12 6-7 10 15z' fill='#ffffff'/>"
        }
        IconKind::Audio => "<path d='M28 14v33a9 9 0 1 1-5-8V19l28-9v29a9 9 0 1 1-5-8V17z'/>",
        IconKind::Video => {
            "<path d='M7 11h50v43H7z'/><path d='M26 21l18 12-18 12z' fill='#ffffff'/>"
        }
        _ => {
            "<path d='M14 6h25l12 12v40H14z'/><path d='M39 6v13h12M22 31h21M22 39h21M22 47h16' fill='none' stroke='#ffffff' stroke-width='2'/>"
        }
    };
    let source = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><g fill='{color}'>{drawing}</g></svg>"
    );
    svg(svg::Handle::from_memory(source.into_bytes()))
        .width(size)
        .height(size)
        .into()
}
