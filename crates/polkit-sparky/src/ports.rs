use crate::domain::{Event, Secret};

pub trait Responses: Send {
    fn respond(&self, response: Secret) -> Result<(), String>;
}

pub struct Conversation {
    pub responses: Box<dyn Responses>,
    pub events: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

pub trait Authenticator: Send + Sync {
    fn start(&self, uid: u32, cookie: &Secret) -> Result<Conversation, String>;
    fn shutdown(&self);
}
