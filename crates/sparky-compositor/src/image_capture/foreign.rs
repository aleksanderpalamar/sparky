use crate::state::Sparky;
use crate::windows::traits::{title_of, traits_of};
use smithay::delegate_foreign_toplevel_list;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::wayland::foreign_toplevel_list::{
    ForeignToplevelListHandler, ForeignToplevelListState,
};
use std::time::Duration;

impl ForeignToplevelListHandler for Sparky {
    fn foreign_toplevel_list_state(&mut self) -> &mut ForeignToplevelListState {
        self.watch_capture_sources();
        self.refresh_capture_sources();
        &mut self.protocols.foreign_toplevels
    }
}
delegate_foreign_toplevel_list!(Sparky);

impl Sparky {
    pub fn refresh_capture_sources(&mut self) {
        let windows: Vec<_> = self
            .windows
            .iter()
            .filter(|(id, _)| self.desktop.workspace_of(*id).is_some())
            .map(|(id, window)| (id, title_of(window), traits_of(window).app_id))
            .collect();
        let removed: Vec<_> = self
            .image_capture
            .toplevels
            .keys()
            .filter(|id| !windows.iter().any(|(known, _, _)| known == *id))
            .copied()
            .collect();
        for id in removed {
            self.image_capture.forget(id);
        }
        for (id, title, app_id) in windows {
            match self.image_capture.toplevels.get(&id) {
                Some(handle) => {
                    handle.send_title(&title);
                    handle.send_app_id(&app_id);
                    handle.send_done();
                }
                None => {
                    let handle = self
                        .protocols
                        .foreign_toplevels
                        .new_toplevel::<Self>(title, app_id);
                    handle.user_data().insert_if_missing(|| id);
                    self.image_capture.toplevels.insert(id, handle);
                }
            }
        }
        self.protocols.foreign_toplevels.cleanup_closed_handles();
    }

    fn watch_capture_sources(&mut self) {
        if self.image_capture.source_timer.is_some() {
            return;
        }
        let timer = Timer::from_duration(Duration::from_secs(1));
        match self.loop_handle.insert_source(timer, |_, _, state| {
            state.refresh_capture_sources();
            TimeoutAction::ToDuration(Duration::from_secs(1))
        }) {
            Ok(token) => self.image_capture.source_timer = Some(token),
            Err(error) => tracing::warn!("sem atualização das fontes de captura: {error}"),
        }
    }
}
