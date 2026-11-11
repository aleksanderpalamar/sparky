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
    ShowUpdates,
    Installed(Result<Vec<Application>, String>),
    Select(usize),
    Back,
    FeatureNext,
    FeaturePrevious,
    Topic(String),
    Ask(Action),
    AskFor(usize, Action),
    Cancel,
    Confirm,
    Performed(Result<(), String>),
    EnableFlathub,
    FlathubEnabled(Result<(), String>),
}
