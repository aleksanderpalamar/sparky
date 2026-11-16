use iced::Task;
use iced::widget::operation;
use sparky_theme::Skin;
use sparky_ui::prompt;

use crate::adapters::launch::launch;
use crate::app::editing::Focus;
use crate::app::message::Message;
use crate::app::pictures;
use crate::app::state::Launcher;
use crate::app::view::ROW_STEP;
use crate::domain::app::App;
use crate::domain::grid::Move;
use crate::domain::menu::Target;
use crate::domain::overlay::Overlay;

pub const NAME_INPUT: &str = "group-name";

pub fn update(launcher: &mut Launcher, message: Message) -> Task<Message> {
    let closed = launcher.overlay == Overlay::Closed;
    match message {
        Message::Palette(palette) => launcher.skin = Skin::new(palette),
        Message::Icons(icons) => launcher.icons = pictures::shown(icons),
        Message::Typed(text) => launcher.type_query(text),
        Message::Move(step) => moved(launcher, step),
        Message::NextShelf(delta) if closed => launcher.next_shelf(delta),
        Message::ShowShelf(shelf) if closed => launcher.show_shelf(shelf),
        Message::Hover(index) if closed => launcher.hover(index),
        Message::Scrolled(delta) if closed => launcher.scroll(delta, ROW_STEP),
        Message::Pointer(point) => launcher.pointer = (point.x, point.y),
        Message::AppMenu(app) if closed => launcher.open_menu(Target::App(app)),
        Message::GroupMenu(group) if closed => launcher.open_menu(Target::Group(group)),
        Message::MenuKey if closed => launcher.open_menu_on_chosen(),
        Message::Choose(index) => return focus(launcher.choose(index)),
        Message::AddGroup if closed => return focus(launcher.add_group()),
        Message::Named(text) => launcher.type_name(text),
        Message::ConfirmName => return focus(launcher.confirm_name()),
        Message::Dismiss => return focus(launcher.dismiss()),
        Message::Submit => return submit(launcher),
        Message::Pick(index) if closed => return start(launcher.pick(index)),
        Message::Close if closed => return iced::exit(),
        Message::Close => return focus(launcher.dismiss()),
        _ => {}
    }
    Task::none()
}

fn moved(launcher: &mut Launcher, step: Move) {
    match (&launcher.overlay, step) {
        (Overlay::Closed, step) => launcher.step(step),
        (Overlay::Menu(_), Move::Up | Move::Previous) => launcher.menu_step(-1),
        (Overlay::Menu(_), Move::Down | Move::Next) => launcher.menu_step(1),
        (Overlay::Naming { .. }, _) => {}
    }
}

fn submit(launcher: &mut Launcher) -> Task<Message> {
    match &launcher.overlay {
        Overlay::Closed => start(launcher.chosen()),
        Overlay::Menu(menu) => {
            let cursor = menu.cursor;
            focus(launcher.choose(cursor))
        }
        Overlay::Naming { .. } => focus(launcher.confirm_name()),
    }
}

fn focus(target: Focus) -> Task<Message> {
    match target {
        Focus::Search => prompt::focus(),
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
