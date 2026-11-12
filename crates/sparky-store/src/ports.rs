use crate::domain::updates::{UpdateReport, UpdateTarget};
use crate::domain::{Action, Application, SearchResult, SearchSource};

pub trait StoreBackend: Send + Sync + std::fmt::Debug {
    fn browse(&self) -> Result<SearchResult, String>;
    fn search(&self, query: &str, source: SearchSource) -> Result<SearchResult, String>;
    fn installed(&self) -> Result<Vec<Application>, String>;
    fn perform(&self, app: &Application, action: Action) -> Result<(), String>;
    fn enable_flathub(&self) -> Result<(), String>;
    fn check_updates(&self, refresh: bool) -> UpdateReport;
    fn apply_update(&self, target: &UpdateTarget) -> Result<(), String>;
}
