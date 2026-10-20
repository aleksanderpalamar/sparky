use sparky_tiling::WorkspaceId;

use crate::bindings::action::Action;
use crate::spawn;
use crate::state::Sparky;

impl Sparky {
    pub fn run(&mut self, action: Action) {
        match action {
            Action::Spawn(command) => spawn::spawn(&command, &self.socket_name),
            Action::CloseWindow => self.close_focused(),
            Action::Exit => self.loop_signal.stop(),
            other => {
                self.change_layout(other);
                self.relayout();
            }
        }
    }

    fn change_layout(&mut self, action: Action) {
        let desktop = &mut self.desktop;
        match action {
            Action::ToggleFloating => desktop.toggle_floating(),
            Action::Fullscreen(mode) => desktop.toggle_fullscreen(mode),
            Action::ToggleSplit => desktop.toggle_split(),
            Action::Focus(direction) => {
                desktop.focus_direction(direction);
            }
            Action::Swap(direction) => desktop.swap_direction(direction),
            Action::Resize(width, height) => desktop.resize_focused((width, height)),
            Action::Workspace(workspace) => desktop.focus_workspace(WorkspaceId(workspace)),
            Action::MoveToWorkspace { workspace, follow } => {
                if let Some(window) = desktop.focused_window() {
                    desktop.move_to_workspace(window, WorkspaceId(workspace), follow);
                }
            }
            Action::RelativeWorkspace(step) => desktop.focus_relative(step),
            Action::PreviousWorkspace => desktop.focus_previous(),
            Action::CycleMonitor(step) => desktop.cycle_monitor(step),
            Action::Spawn(_) | Action::CloseWindow | Action::Exit => {}
        }
    }

    fn close_focused(&self) {
        let Some(id) = self.desktop.focused_window() else {
            return;
        };
        if let Some(toplevel) = self.windows.get(id).and_then(|window| window.toplevel()) {
            toplevel.send_close();
        }
    }
}
