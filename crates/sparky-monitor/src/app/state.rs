use std::sync::Arc;

use iced::Size;
use sparky_theme::Skin;
use sparky_ui::notice::Notice;

use super::page::Page;
use crate::domain::apps::AppEntry;
use crate::domain::histories::Histories;
use crate::domain::measure::Reading;
use crate::domain::snapshot::Snapshot;
use crate::domain::table::Sort;
use crate::domain::usage::{AppRow, ProcessRow};
use crate::ports::{Catalog, Claim, Signals, Stop};
use crate::sampler::Sampler;

pub struct Services {
    pub sampler: Arc<Sampler>,
    pub catalog: Arc<dyn Catalog>,
    pub signals: Arc<dyn Signals>,
    pub user: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Applications,
    Processes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sampling {
    Idle,
    Busy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Process { pid: u32, name: String },
    App { name: String, pids: Vec<u32> },
}

pub struct Confirm {
    pub target: Target,
    pub stop: Stop,
    pub claims: Vec<Box<dyn Claim>>,
}

pub struct Monitor {
    pub page: Page,
    pub reading: Option<Reading>,
    pub processes: Vec<ProcessRow>,
    pub apps: Vec<AppRow>,
    pub histories: Histories,
    pub sort: Sort,
    pub focus: Focus,
    pub selected_process: Option<u32>,
    pub selected_app: Option<String>,
    pub confirm: Option<Confirm>,
    pub notice: Option<Notice>,
    pub skin: Skin,
    pub size: Size,
    pub sampling: Sampling,
    pub previous: Option<Snapshot>,
    pub catalog: Vec<AppEntry>,
    pub services: Services,
}

impl Monitor {
    pub fn new(services: Services, skin: Skin, size: Size) -> Self {
        Self {
            page: Page::Dashboard,
            reading: None,
            processes: Vec::new(),
            apps: Vec::new(),
            histories: Histories::default(),
            sort: Sort::default(),
            focus: Focus::Processes,
            selected_process: None,
            selected_app: None,
            confirm: None,
            notice: None,
            skin,
            size,
            sampling: Sampling::Idle,
            previous: None,
            catalog: Vec::new(),
            services,
        }
    }

    pub fn active_table(&self) -> Option<Focus> {
        match self.page {
            Page::Dashboard => Some(self.focus),
            Page::Applications => Some(Focus::Applications),
            Page::Network => None,
            Page::Processes | Page::Cpu | Page::Memory | Page::Disk | Page::Gpu => {
                Some(Focus::Processes)
            }
        }
    }
}
