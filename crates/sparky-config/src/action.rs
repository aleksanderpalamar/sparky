use serde::{Deserialize, Serialize};
use sparky_tiling::{Direction, Fullscreen};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Action {
    Spawn {
        command: String,
    },
    CloseWindow,
    ToggleFloating,
    Fullscreen {
        mode: Fullscreen,
    },
    ToggleSplit,
    Focus {
        direction: Direction,
    },
    Swap {
        direction: Direction,
    },
    Resize {
        width: i32,
        height: i32,
    },
    Workspace {
        workspace: u32,
    },
    MoveToWorkspace {
        workspace: u32,
        #[serde(default = "follows")]
        follow: bool,
    },
    RelativeWorkspace {
        step: i32,
    },
    PreviousWorkspace,
    ToggleScratchpad,
    MoveToScratchpad,
    CycleMonitor {
        step: i32,
    },
    SwitchTerminal {
        terminal: i32,
    },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bind {
    pub keys: String,
    #[serde(flatten)]
    pub action: Action,
}

impl Bind {
    pub fn new(keys: &str, action: Action) -> Self {
        Self {
            keys: keys.to_owned(),
            action,
        }
    }
}

fn follows() -> bool {
    true
}
