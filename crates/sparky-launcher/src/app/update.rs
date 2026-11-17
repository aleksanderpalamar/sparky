use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset};
use sparky_theme::Skin;
use sparky_ui::prompt;

use crate::adapters::launch::launch;
use crate::app::editing::Focus;
use crate::app::message::Message;
use crate::app::navigation::Scroll;
use crate::app::pictures;
use crate::app::state::Launcher;
use crate::domain::app::App;
use crate::domain::grid::Move;
use crate::domain::menu::Target;
use crate::domain::overlay::Overlay;

pub const NAME_INPUT: &str = "group-name";
pub const APPS: &str = "apps";

pub fn update(launcher: &mut Launcher, message: Message) -> Task<Message> {
    let closed = launcher.overlay == Overlay::Closed;
    match message {
        Message::Palette(palette) => launcher.skin = Skin::new(palette),
        Message::Icons(icons) => launcher.icons = pictures::shown(icons),
        Message::Typed(text) => return scroll(launcher.type_query(text)),
        Message::TypeAhead(text) if closed && launcher.mode.search().is_some() => {
            let query = format!("{}{text}", launcher.query);
            return Task::batch([scroll(launcher.type_query(query)), prompt::focus()]);
        }
        Message::Move(step) => return moved(launcher, step),
        Message::NextShelf(delta) if closed => return scroll(launcher.next_shelf(delta)),
        Message::ShowShelf(shelf) if closed => return scroll(launcher.show_shelf(shelf)),
        Message::Hover(index) if closed => launcher.hover(index),
        Message::Scrolled(window) => launcher.scrolled(window),
        Message::Pointer(point) => launcher.pointer = (point.x, point.y),
        Message::AppMenu(app) if closed => launcher.open_app_menu(&app),
        Message::GroupMenu(group) if closed => launcher.open_menu(Target::Group(group)),
        Message::MenuKey if closed => launcher.open_menu_on_chosen(),
        Message::Choose(index) => return edited(launcher, |launcher| launcher.choose(index)),
        Message::AddGroup if closed => return focus(launcher.add_group()),
        Message::Named(text) => launcher.type_name(text),
        Message::ConfirmName => return edited(launcher, Launcher::confirm_name),
        Message::Dismiss => return focus(launcher.dismiss()),
        Message::Submit => return submit(launcher),
        Message::Pick(index) if closed => return start(launcher.pick(index)),
        Message::Inside => {}
        Message::Outside => return iced::exit(),
        Message::Close if closed => return iced::exit(),
        Message::Close => return focus(launcher.dismiss()),
        _ => {}
    }
    Task::none()
}

fn moved(launcher: &mut Launcher, step: Move) -> Task<Message> {
    match (&launcher.overlay, step) {
        (Overlay::Closed, step) => return scroll(launcher.step(step)),
        (Overlay::Menu(_), Move::Up | Move::Previous) => launcher.menu_step(-1),
        (Overlay::Menu(_), Move::Down | Move::Next) => launcher.menu_step(1),
        (Overlay::Naming { .. }, _) => {}
    }
    Task::none()
}

fn submit(launcher: &mut Launcher) -> Task<Message> {
    match &launcher.overlay {
        Overlay::Closed => start(launcher.chosen()),
        Overlay::Menu(menu) => {
            let cursor = menu.cursor;
            edited(launcher, |launcher| launcher.choose(cursor))
        }
        Overlay::Naming { .. } => edited(launcher, Launcher::confirm_name),
    }
}

fn edited(launcher: &mut Launcher, change: impl FnOnce(&mut Launcher) -> Focus) -> Task<Message> {
    let target = change(launcher);
    Task::batch([focus(target), scroll(Scroll::To(launcher.window.offset))])
}

fn scroll(target: Scroll) -> Task<Message> {
    match target {
        Scroll::Keep => Task::none(),
        Scroll::To(y) => operation::scroll_to(APPS, AbsoluteOffset { x: 0.0, y }),
    }
}

fn focus(target: Focus) -> Task<Message> {
    match target {
        Focus::Name => {
            operation::focus(NAME_INPUT).chain(operation::move_cursor_to_end(NAME_INPUT))
        }
        Focus::Stay => Task::none(),
    }
}

fn start(app: Option<App>) -> Task<Message> {
    let Some(app) = app else {
        return Task::none();
    };
    if let Err(error) = launch(&app) {
        eprintln!("sparky-launcher: não deu para abrir {}: {error}", app.name);
    }
    iced::exit()
}
