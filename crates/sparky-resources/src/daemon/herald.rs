use crate::application::notice::notice;
use crate::orchestration::Decision;
use crate::ports::Notifier;

pub struct Herald<N> {
    notifier: N,
    seen: u64,
}

impl<N: Notifier> Herald<N> {
    pub fn new(notifier: N) -> Self {
        Self { notifier, seen: 0 }
    }

    pub fn hear(&mut self, decisions: &[Decision], recorded: u64, enabled: bool) {
        let fresh = usize::try_from(recorded.saturating_sub(self.seen))
            .unwrap_or(usize::MAX)
            .min(decisions.len());
        self.seen = recorded;
        if !enabled {
            return;
        }
        for decision in &decisions[decisions.len() - fresh..] {
            if let Some(notice) = notice(decision) {
                self.notifier.notify(&notice.title, &notice.body);
            }
        }
    }
}
