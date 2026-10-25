use std::time::Duration;
use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay::reexports::wayland_protocols::ext::image_copy_capture::v1::server::ext_image_copy_capture_frame_v1::FailureReason;
use crate::state::Sparky;
use super::{queue::Job, rules::{Gate, FRAME_INTERVAL}, state::Wake, types::Target};

impl Sparky {
    pub fn process_captures(&mut self) {
        self.reap_captures();
        self.refresh_capture_sessions();
        let jobs = std::mem::take(&mut self.image_capture.pending);
        for mut job in jobs {
            if !job.notify.alive() {
                continue;
            }
            if !self.capture_current(&job) {
                continue;
            }
            let gate = self.capture_gate(&job);
            match gate {
                Gate::Later(delay) => {
                    self.wake_capture(delay);
                    self.image_capture.pending.push(job);
                    continue;
                }
                Gate::Unchanged => {
                    self.pump_capture(&job.target);
                    self.image_capture.pending.push(job);
                    continue;
                }
                Gate::Ready => {}
            }
            let outcome = self.submit_capture(&mut job);
            match outcome {
                Some(Ok(sync)) => {
                    self.mark_capture(&job);
                    self.finish_capture(job, sync);
                }
                Some(Err(error)) => {
                    tracing::warn!("captura DMA-BUF falhou: {error}");
                    job.notify.failed(FailureReason::Unknown);
                }
                None => job.notify.failed(FailureReason::Unknown),
            }
        }
    }

    fn submit_capture(
        &mut self,
        job: &mut Job,
    ) -> Option<Result<smithay::backend::renderer::sync::SyncPoint, String>> {
        if matches!(&self.backend, crate::backend::Backend::Drm(drm) if !drm.active) {
            return Some(Err("sessão gráfica inativa".into()));
        }
        let mut cursors = self
            .image_capture
            .cursors
            .take()
            .unwrap_or_else(crate::backend::drm::cursor::Cursors::load);
        let result = self.with_renderer(|state, renderer| {
            super::gpu::render(state, renderer, &mut cursors, job)
        });
        self.image_capture.cursors = Some(cursors);
        result
    }

    fn capture_current(&self, job: &Job) -> bool {
        let current = self.capture_dimensions(&job.target);
        if current.is_none() {
            job.notify.failed(FailureReason::Stopped);
            return false;
        }
        if job.region.is_some() || current == Some(job.size) {
            return true;
        }
        job.notify.failed(FailureReason::BufferConstraints);
        false
    }

    fn capture_gate(&self, job: &Job) -> Gate {
        let Some((_, data)) = &job.session else {
            return Gate::Ready;
        };
        match data.status.lock() {
            Ok(status) => status
                .pace
                .gate(self.start_time.elapsed(), self.image_capture.generation),
            Err(_) => Gate::Ready,
        }
    }

    fn mark_capture(&self, job: &Job) {
        let Some((_, data)) = &job.session else {
            return;
        };
        if let Ok(mut status) = data.status.lock() {
            status
                .pace
                .submitted(self.start_time.elapsed(), self.image_capture.generation);
        }
    }

    pub(super) fn pump_capture(&mut self, target: &Target) {
        let Target::Window(id) = target else { return };
        let Some(window) = self.windows.get(*id) else {
            return;
        };
        let Some(output) = self.space.outputs().next().cloned() else {
            return;
        };
        window.send_frame(
            &output,
            self.start_time.elapsed(),
            Some(FRAME_INTERVAL),
            |_, _| Some(output.clone()),
        );
        self.wake_capture(FRAME_INTERVAL);
    }

    pub(super) fn wake_capture(&mut self, delay: Duration) {
        let deadline = self.start_time.elapsed().saturating_add(delay);
        let current = self.image_capture.wake.as_ref().map(|wake| wake.deadline);
        if !super::rules::replaces_wake(current, deadline) {
            return;
        }
        if let Some(wake) = self.image_capture.wake.take() {
            self.loop_handle.remove(wake.token);
        }
        match self
            .loop_handle
            .insert_source(Timer::from_duration(delay), |_, _, state| {
                state.image_capture.wake = None;
                TimeoutAction::Drop
            }) {
            Ok(token) => self.image_capture.wake = Some(Wake { token, deadline }),
            Err(error) => tracing::warn!("sem agendamento de captura: {error}"),
        }
    }
}
