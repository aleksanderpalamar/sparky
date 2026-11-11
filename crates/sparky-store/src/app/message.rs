use crate::domain::{Action, Application, SearchResult, SearchSource};

#[derive(Debug, Clone)]
pub enum Message {
    Palette(sparky_theme::Palette),
    Query(String),
    Source(SearchSource),
    Browse,
    Browsed(Result<SearchResult, String>),
    LoadMore,
    Search,
    Searched(Result<SearchResult, String>),
    ShowInstalled,
    Installed(Result<Vec<Application>, String>),
    Select(usize),
    Ask(Action),
    Cancel,
    Confirm,
    Performed(Result<(), String>),
    EnableFlathub,
    FlathubEnabled(Result<(), String>),
}
