use std::time::Instant;

use smithay::reexports::calloop::timer::{TimeoutAction, Timer};
use sparky_ipc::{Reply, SessionAction};

use crate::session::departure::{Closing, Farewell, Plan, Sleep, TICK, allowed, closing, plan, sleep, stuck_message};
use crate::spawn::spawn_program;
use crate::state::Sparky;
use crate::windows::surface::close;

const REFUSED: &str = "suspender, reiniciar e desligar só valem na sessão real do sparky";

enum Watch {
    Again,
    Finished,
}

impl Sparky {
    pub fn session_request(&mut self, action: SessionAction) -> Reply {
        let plan = plan(action);
        if !allowed(plan, self.is_real_session()) {
            return Reply::Error { message: REFUSED.to_owned() };
        }
        match plan {
            Plan::Lock => self.lock_screen(),
            Plan::Suspend => self.suspend(),
            Plan::Leave(farewell) => self.leave(farewell),
        }
        Reply::Done
    }

    fn lock_screen(&mut self) {
        if !self.lock.is_locked() {
            let locker = self.config.apps.locker.clone();
            self.spawn(&locker);
        }
    }

    fn suspend(&mut self) {
        self.lock_screen();
        let started = Instant::now();
        self.watch(move |state| match sleep(state.lock.is_locked(), started.elapsed()) {
            Sleep::Waiting => Watch::Again,
            Sleep::Now => {
                state.systemctl("suspend");
                Watch::Finished
            }
        });
    }

    fn leave(&mut self, farewell: Farewell) {
        if self.departure.is_some() {
            return;
        }
        self.departure = Some(farewell);
        for (_, window) in self.windows.iter() {
            close(window);
        }
        let started = Instant::now();
        self.watch(move |state| match closing(state.windows.iter().count(), started.elapsed()) {
            Closing::Waiting => Watch::Again,
            Closing::Done => {
                state.depart(farewell);
                Watch::Finished
            }
            Closing::Stuck(left) => {
                state.departure = None;
                state.notify(&stuck_message(left));
                Watch::Finished
            }
        });
    }

    fn depart(&mut self, farewell: Farewell) {
        self.departure = None;
        match farewell.power() {
            Some(command) => self.systemctl(command),
            None => self.loop_signal.stop(),
        }
    }

    fn systemctl(&self, command: &str) {
        tracing::info!("sessão: systemctl {command}");
        spawn_program("systemctl", &[command.to_owned()], &self.child_environment());
    }

    fn notify(&self, text: &str) {
        tracing::warn!("sessão: {text}");
        spawn_program("notify-send", &["Sparky".to_owned(), text.to_owned()], &self.child_environment());
    }

    fn watch(&mut self, mut check: impl FnMut(&mut Sparky) -> Watch + 'static) {
        let watching = self.loop_handle.insert_source(Timer::from_duration(TICK), move |_, _, state| match check(state) {
            Watch::Again => TimeoutAction::ToDuration(TICK),
            Watch::Finished => TimeoutAction::Drop,
        });
        if let Err(error) = watching {
            tracing::warn!("sessão: não deu para acompanhar o pedido: {error}");
        }
    }
}
