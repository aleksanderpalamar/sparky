use std::sync::Arc;
use smithay::backend::allocator::dmabuf::Dmabuf;
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_frame_v1::FailureReason;
use smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1;
use smithay::reexports::wayland_server::{Resource, protocol::wl_output::Transform};
use smithay::utils::{Clock, Monotonic};
use sparky_tiling::Rect;
use crate::state::Sparky;
use super::types::{Target, Frame, Session, SessionData};

pub struct Job {
    pub notify: Notify,
    pub buffer: Dmabuf,
    pub target: Target,
    pub size: (i32, i32),
    pub cursor: bool,
    pub session: Option<(Session, Arc<SessionData>)>,
    pub region: Option<Rect>,
    pub locked: bool,
}

#[derive(Clone)]
pub enum Notify {
    Image(Frame),
    Legacy(ZwlrScreencopyFrameV1, bool),
}

impl Notify {
    pub fn alive(&self) -> bool {
        match self {
            Self::Image(frame) => frame.is_alive(),
            Self::Legacy(frame, _) => frame.is_alive(),
        }
    }
    pub fn failed(&self, reason: FailureReason) {
        match self {
            Self::Image(frame) => frame.failed(reason),
            Self::Legacy(frame, _) => frame.failed(),
        }
    }
}

impl Job {
    pub fn render_transform(&self) -> smithay::utils::Transform {
        match (&self.notify, &self.target) {
            (Notify::Legacy(..), Target::Output(output)) => output
                .upgrade()
                .map_or(smithay::utils::Transform::Normal, |output| {
                    output.current_transform()
                }),
            _ => smithay::utils::Transform::Normal,
        }
    }

    pub fn complete(self, locked: bool) -> Option<Self> {
        if !self.notify.alive() {
            return None;
        }
        if !super::rules::can_publish(self.locked, locked) {
            self.reset_pace();
            return Some(self);
        }
        let now: std::time::Duration = Clock::<Monotonic>::new().now().into();
        let seconds = now.as_secs();
        let (hi, lo, ns) = ((seconds >> 32) as u32, seconds as u32, now.subsec_nanos());
        match self.notify {
            Notify::Image(frame) => {
                frame.transform(Transform::Normal);
                frame.damage(0, 0, self.size.0, self.size.1);
                frame.presentation_time(hi, lo, ns);
                frame.ready();
            }
            Notify::Legacy(frame, damage) => {
                frame.flags(smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_frame_v1::Flags::empty());
                if damage {
                    frame.damage(0, 0, self.size.0 as u32, self.size.1 as u32);
                }
                frame.ready(hi, lo, ns);
            }
        }
        None
    }

    fn reset_pace(&self) {
        let Some((_, data)) = &self.session else {
            return;
        };
        if let Ok(mut status) = data.status.lock() {
            status.pace = super::rules::Pace::default();
        }
    }
}

impl Sparky {
    pub fn queue_capture(&mut self, job: Job) {
        self.reap_captures();
        self.image_capture
            .pending
            .retain(|pending| pending.notify.alive());
        if !super::rules::admits(
            self.image_capture.pending.len(),
            self.image_capture.flights.len(),
        ) {
            job.notify.failed(FailureReason::Unknown);
            return;
        }
        self.image_capture.pending.push(job);
    }
}
