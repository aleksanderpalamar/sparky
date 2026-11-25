use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle, Thread};
use std::time::Duration;

pub struct Worker<R> {
    stop: Arc<AtomicBool>,
    handle: JoinHandle<R>,
}

impl<R: Send + 'static> Worker<R> {
    pub fn spawn<T: Send + 'static>(
        mut state: T,
        mut step: impl FnMut(&mut T) -> Duration + Send + 'static,
        finish: impl FnOnce(T) -> R + Send + 'static,
    ) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let seen = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            while !seen.load(Ordering::SeqCst) {
                let pause = step(&mut state);
                if !seen.load(Ordering::SeqCst) {
                    thread::park_timeout(pause);
                }
            }
            finish(state)
        });
        Self { stop, handle }
    }

    pub fn thread(&self) -> &Thread {
        self.handle.thread()
    }

    pub fn finish(self) -> thread::Result<R> {
        self.stop.store(true, Ordering::SeqCst);
        self.handle.thread().unpark();
        self.handle.join()
    }
}
