use std::{cell::Cell, rc::Rc, time::Duration};
use smithay::backend::renderer::sync::SyncPoint;
use smithay::reexports::calloop::{generic::Generic, timer::{TimeoutAction, Timer}, Interest, Mode, PostAction};
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_frame_v1::FailureReason;
use crate::state::Sparky;
use super::{queue::Job, state::Flight};

impl Sparky {
    pub fn finish_capture(&mut self, job: Job, sync: SyncPoint) {
        if sync.is_reached() {
            job.complete(self.lock.is_locked());
            return;
        }
        let notify = job.notify.clone();
        let done = Rc::new(Cell::new(false));
        let finished = done.clone();
        let mut job = Some(job);
        let token = match sync.export() {
            Some(fd) => self
                .loop_handle
                .insert_source(
                    Generic::new(fd, Interest::READ, Mode::Level),
                    move |_, _, state| {
                        if let Some(job) = job.take() {
                            job.complete(state.lock.is_locked());
                        }
                        finished.set(true);
                        Ok(PostAction::Remove)
                    },
                )
                .map_err(|error| error.to_string()),
            None => self
                .loop_handle
                .insert_source(
                    Timer::from_duration(Duration::from_millis(2)),
                    move |_, _, state| {
                        if !sync.is_reached() {
                            return TimeoutAction::ToDuration(Duration::from_millis(2));
                        }
                        if let Some(job) = job.take() {
                            job.complete(state.lock.is_locked());
                        }
                        finished.set(true);
                        TimeoutAction::Drop
                    },
                )
                .map_err(|error| error.to_string()),
        };
        match token {
            Ok(token) => self.image_capture.flights.push(Flight {
                token,
                notify,
                done,
            }),
            Err(error) => {
                notify.failed(FailureReason::Unknown);
                tracing::warn!("falha ao acompanhar captura na GPU: {error}");
            }
        }
    }

    pub fn reap_captures(&mut self) {
        let flights = std::mem::take(&mut self.image_capture.flights);
        for flight in flights {
            if flight.done.get() {
                continue;
            }
            if !flight.notify.alive() {
                self.loop_handle.remove(flight.token);
                continue;
            }
            self.image_capture.flights.push(flight);
        }
    }
}
