use crate::domain::calendar::Calendar;
use sparky_notifications::{client::Client, domain::Snapshot};
use std::sync::Arc;

#[derive(Debug)]
pub struct CenterState {
    pub calendar: Calendar,
    pub client: Option<Arc<Client>>,
    pub snapshot: Snapshot,
    pub error: Option<String>,
}

impl Default for CenterState {
    fn default() -> Self {
        Self::new(chrono::Local::now().date_naive())
    }
}

impl CenterState {
    pub fn new(today: chrono::NaiveDate) -> Self {
        Self {
            calendar: Calendar::new(today),
            client: None,
            snapshot: Snapshot::default(),
            error: Some("Conectando ao serviço de notificações…".to_owned()),
        }
    }
}
