use sparky_tiling::{Direction, Fullscreen};

use crate::action::{Action, Bind};
use crate::schema::Apps;

const DIRECTIONS: [(&str, Direction); 4] = [
    ("LEFT", Direction::Left),
    ("RIGHT", Direction::Right),
    ("UP", Direction::Up),
    ("DOWN", Direction::Down),
];

pub fn sparky(apps: &Apps, terminal: &str) -> Vec<Bind> {
    let mut binds = applications(apps, terminal);
    binds.extend(windows());
    binds.extend(directions());
    binds.extend(workspaces());
    binds.extend(groups());
    binds
}

fn applications(apps: &Apps, terminal: &str) -> Vec<Bind> {
    vec![
        bind("SUPER + RETURN", spawn(terminal)),
        bind("SUPER + SPACE", spawn(&apps.launcher)),
        bind("SUPER + SHIFT + F", spawn(&apps.files)),
        bind("SUPER + ALT + SPACE", spawn(&apps.settings)),
        bind("PRINT", spawn(&format!("{} region", apps.screenshot))),
        bind("SHIFT + PRINT", spawn(&format!("{} window", apps.screenshot))),
        bind("CTRL + PRINT", spawn(&format!("{} output", apps.screenshot))),
        bind("SUPER + PRINT", spawn(&apps.screenshot)),
        bind("SUPER + SHIFT + ESCAPE", Action::Exit),
    ]
}

fn windows() -> Vec<Bind> {
    vec![
        bind("SUPER + W", Action::CloseWindow),
        bind("SUPER + Q", Action::CloseWindow),
        bind("SUPER + J", Action::ToggleSplit),
        bind("SUPER + T", Action::ToggleFloating),
        bind("SUPER + F", Action::Fullscreen { mode: Fullscreen::Monitor }),
        bind("SUPER + ALT + F", Action::Fullscreen { mode: Fullscreen::Maximized }),
        bind("SUPER + code:20", Action::Resize { width: -100, height: 0 }),
        bind("SUPER + code:21", Action::Resize { width: 100, height: 0 }),
        bind("SUPER + SHIFT + code:20", Action::Resize { width: 0, height: -100 }),
        bind("SUPER + SHIFT + code:21", Action::Resize { width: 0, height: 100 }),
        bind("SUPER + TAB", Action::RelativeWorkspace { step: 1 }),
        bind("SUPER + SHIFT + TAB", Action::RelativeWorkspace { step: -1 }),
        bind("SUPER + CTRL + TAB", Action::PreviousWorkspace),
        bind("SUPER + S", Action::ToggleScratchpad),
        bind("SUPER + code:49", Action::ToggleScratchpad),
        bind("SUPER + ALT + S", Action::MoveToScratchpad),
        bind("SUPER + SHIFT + code:49", Action::MoveToScratchpad),
        bind("CTRL + ALT + TAB", Action::CycleMonitor { step: 1 }),
        bind("CTRL + ALT + SHIFT + TAB", Action::CycleMonitor { step: -1 }),
    ]
}

fn directions() -> Vec<Bind> {
    DIRECTIONS
        .iter()
        .flat_map(|(key, direction)| {
            [
                bind(&format!("SUPER + {key}"), Action::Focus { direction: *direction }),
                bind(&format!("SUPER + SHIFT + {key}"), Action::Swap { direction: *direction }),
            ]
        })
        .collect()
}

fn workspaces() -> Vec<Bind> {
    (1..=10u32)
        .flat_map(|workspace| {
            let key = format!("code:{}", workspace + 9);
            [
                bind(&format!("SUPER + {key}"), Action::Workspace { workspace }),
                bind(
                    &format!("SUPER + SHIFT + {key}"),
                    Action::MoveToWorkspace { workspace, follow: true },
                ),
                bind(
                    &format!("SUPER + SHIFT + ALT + {key}"),
                    Action::MoveToWorkspace { workspace, follow: false },
                ),
            ]
        })
        .collect()
}

fn groups() -> Vec<Bind> {
    let mut binds = vec![
        bind("SUPER + G", Action::ToggleGroup),
        bind("SUPER + ALT + G", Action::LeaveGroup),
        bind("SUPER + ALT + TAB", Action::CycleGroup { step: 1 }),
        bind("SUPER + ALT + SHIFT + TAB", Action::CycleGroup { step: -1 }),
        bind("SUPER + CTRL + RIGHT", Action::CycleGroup { step: 1 }),
        bind("SUPER + CTRL + LEFT", Action::CycleGroup { step: -1 }),
    ];
    binds.extend(
        DIRECTIONS
            .iter()
            .map(|(key, direction)| bind(&format!("SUPER + ALT + {key}"), Action::MoveIntoGroup { direction: *direction })),
    );
    binds.extend((1..=5).map(|index| bind(&format!("SUPER + ALT + code:{}", index + 9), Action::SelectInGroup { index })));
    binds
}

fn bind(keys: &str, action: Action) -> Bind {
    Bind::new(keys, action)
}

fn spawn(command: &str) -> Action {
    Action::Spawn {
        command: command.to_owned(),
    }
}
