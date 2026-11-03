use tokio::sync::oneshot;

use crate::service::workers::terminate;

#[derive(Default)]
pub struct GreeterProcess {
    stop: Option<oneshot::Sender<()>>,
    pid: Option<u32>,
}

impl GreeterProcess {
    pub fn started(&mut self, stop: oneshot::Sender<()>, pid: Option<u32>) {
        self.stop = Some(stop);
        self.pid = pid;
    }

    pub fn ask_to_stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }

    pub fn gone(&mut self) {
        self.stop = None;
        self.pid = None;
    }

    pub fn terminate(&mut self) {
        terminate(self.pid.take());
    }
}
