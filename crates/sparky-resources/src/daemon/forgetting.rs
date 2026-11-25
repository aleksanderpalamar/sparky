use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::Thread;
use std::time::Duration;

use crate::protocol::{Refusal, Reply};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Stage {
    #[default]
    Idle,
    Requested,
    Working,
    Finished(bool),
}

#[derive(Debug, Default)]
pub struct Forgetting {
    stage: Mutex<Stage>,
    changed: Condvar,
    waker: Mutex<Option<Thread>>,
}

impl Forgetting {
    pub fn wake_with(&self, thread: Thread) {
        *lock(&self.waker) = Some(thread);
    }

    pub fn take(&self) -> bool {
        let mut stage = lock(&self.stage);
        if *stage != Stage::Requested {
            return false;
        }
        *stage = Stage::Working;
        true
    }

    pub fn finish(&self, removed: bool) {
        *lock(&self.stage) = Stage::Finished(removed);
        self.changed.notify_all();
    }

    fn request(&self) {
        {
            let mut stage = lock(&self.stage);
            if !matches!(*stage, Stage::Requested | Stage::Working) {
                *stage = Stage::Requested;
            }
        }
        if let Some(thread) = lock(&self.waker).as_ref() {
            thread.unpark();
        }
    }

    fn wait(&self, limit: Duration) -> Option<bool> {
        let stage = lock(&self.stage);
        let (stage, _) = self
            .changed
            .wait_timeout_while(stage, limit, |stage| {
                matches!(*stage, Stage::Requested | Stage::Working)
            })
            .unwrap_or_else(PoisonError::into_inner);
        match *stage {
            Stage::Finished(removed) => Some(removed),
            _ => None,
        }
    }
}

pub fn reply(forgetting: &Forgetting, limit: Duration) -> Reply {
    forgetting.request();
    match forgetting.wait(limit) {
        Some(true) => Reply::Done,
        _ => Reply::Refused {
            reason: Refusal::ForgetFailed,
        },
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
