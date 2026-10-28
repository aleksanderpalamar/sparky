use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;

use iced::futures::channel::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::domain::{
    panel::PanelKind,
    prompt::{Answer, Prompt, PromptEvent, PromptId, PromptKind},
};
use crate::ports::prompts::Prompts;

type Pending = HashMap<PromptId, (PanelKind, oneshot::Sender<Answer>)>;

pub struct Broker {
    next: AtomicU64,
    output: UnboundedSender<PromptEvent>,
    pending: Mutex<Pending>,
}

impl std::fmt::Debug for Broker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PromptBroker")
    }
}

impl Broker {
    pub fn new() -> (Arc<Self>, mpsc::UnboundedReceiver<PromptEvent>) {
        let (output, input) = mpsc::unbounded();
        (
            Arc::new(Self {
                next: AtomicU64::new(1),
                output,
                pending: Mutex::default(),
            }),
            input,
        )
    }

    pub async fn request(&self, kind: PromptKind) -> Answer {
        let id = PromptId(self.next.fetch_add(1, Ordering::Relaxed));
        let (sender, receiver) = oneshot::channel();
        {
            let Ok(mut pending) = self.pending.lock() else {
                return Answer::Reject;
            };
            pending.insert(id, (kind.panel(), sender));
        }
        if self
            .output
            .unbounded_send(PromptEvent::Show(Prompt { id, kind }))
            .is_err()
        {
            self.answer(id, Answer::Reject);
            return Answer::Reject;
        }
        let answer = match tokio::time::timeout(Duration::from_secs(90), receiver).await {
            Ok(Ok(answer)) => answer,
            _ => Answer::Reject,
        };
        self.answer(id, Answer::Reject);
        let _ = self.output.unbounded_send(PromptEvent::Dismiss(id));
        answer
    }

    pub fn display(&self, kind: PromptKind) -> PromptId {
        let id = PromptId(self.next.fetch_add(1, Ordering::Relaxed));
        let _ = self
            .output
            .unbounded_send(PromptEvent::Show(Prompt { id, kind }));
        id
    }

    pub fn dismiss(&self, id: PromptId) {
        self.answer(id, Answer::Reject);
        let _ = self.output.unbounded_send(PromptEvent::Dismiss(id));
    }
}

impl Prompts for Broker {
    fn answer(&self, id: PromptId, answer: Answer) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        let Some((_, sender)) = pending.remove(&id) else {
            return;
        };
        let _ = sender.send(answer);
    }

    fn cancel(&self, panel: PanelKind) {
        let Ok(mut pending) = self.pending.lock() else {
            return;
        };
        let ids: Vec<_> = pending
            .iter()
            .filter_map(|(id, (kind, _))| (*kind == panel).then_some(*id))
            .collect();
        for id in ids {
            let Some((_, sender)) = pending.remove(&id) else {
                continue;
            };
            let _ = sender.send(Answer::Reject);
            let _ = self.output.unbounded_send(PromptEvent::Dismiss(id));
        }
    }
}
