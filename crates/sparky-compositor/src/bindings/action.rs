use sparky_tiling::{Direction, Fullscreen};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Spawn(String),
    CloseWindow,
    ToggleFloating,
    Fullscreen(Fullscreen),
    ToggleSplit,
    Focus(Direction),
    Swap(Direction),
    Resize(i32, i32),
    Workspace(u32),
    MoveToWorkspace { workspace: u32, follow: bool },
    RelativeWorkspace(i32),
    PreviousWorkspace,
    CycleMonitor(i32),
    Exit,
}
