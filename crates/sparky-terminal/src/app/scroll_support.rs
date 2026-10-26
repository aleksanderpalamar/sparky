use std::time::{Duration, Instant};

use iced::advanced::Shell;
use iced::advanced::clipboard::Null;
use iced::advanced::layout::{Layout, Limits};
use iced::advanced::renderer::Headless;
use iced::advanced::widget::Tree;
use iced::futures::{channel::mpsc, executor::block_on};
use iced::{Event, Pixels, Point, Rectangle, Renderer, Size, mouse};
use sparky_ui::style::FONT;

use crate::adapters::listener::Listener;
use crate::adapters::session::Session;
use crate::app::message::Message;
use crate::app::workspace::Workspace;
use crate::app::workspace_tests::workspace;
use crate::domain::launch::{Launch, Program};

pub fn text(app: &Workspace) -> String {
    app.tabs
        .active()
        .unwrap()
        .value
        .screen
        .rows
        .iter()
        .flatten()
        .map(|run| run.text.as_str())
        .collect()
}

pub fn scripted(script: &str) -> Workspace {
    let mut app = workspace();
    app.fit(Size::new(640.0, 256.0));
    let tab = app.tabs.active_mut().unwrap();
    let (sender, _receiver) = mpsc::unbounded();
    let launch = Launch {
        program: Program::Command {
            program: "sh".into(),
            args: vec!["-c".into(), script.into()],
        },
        directory: None,
    };
    tab.value.session = Some(Session::start(&launch, tab.value.size, Listener(sender)).unwrap());
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        app.tabs.active_mut().unwrap().value.refresh();
        if text(&app).contains("PRONTO") {
            return app;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("a saída do PTY privado não chegou");
}

pub fn wheel(app: &Workspace, delta: mouse::ScrollDelta, at: Point) -> Vec<Message> {
    let renderer = block_on(<Renderer as Headless>::new(
        FONT,
        Pixels(12.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut view = crate::app::tabs_view::view(app);
    let mut tree = Tree::new(&view);
    let viewport = Rectangle::with_size(app.size);
    let node =
        view.as_widget_mut()
            .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, app.size));
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    view.as_widget_mut().update(
        &mut tree,
        &Event::Mouse(mouse::Event::WheelScrolled { delta }),
        Layout::new(&node),
        mouse::Cursor::Available(at),
        &renderer,
        &mut Null,
        &mut shell,
        &viewport,
    );
    messages
}

pub fn populated() -> Workspace {
    scripted(
        "i=0; while [ \"$i\" -lt 120 ]; do printf 'linha-%03d\\n' \"$i\"; i=$((i+1)); done; printf 'PRONTO'; read -r fim",
    )
}

pub fn wait_for(app: &mut Workspace, expected: &str) {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(3) {
        app.tabs.active_mut().unwrap().value.refresh();
        if text(app).contains(expected) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("não recebeu {expected:?}: {}", text(app));
}
