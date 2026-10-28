use crate::domain::{
    panel::PanelKind,
    prompt::{Answer, PromptId},
};

pub trait Prompts: std::fmt::Debug + Send + Sync {
    fn answer(&self, id: PromptId, answer: Answer);
    fn cancel(&self, panel: PanelKind);
}
