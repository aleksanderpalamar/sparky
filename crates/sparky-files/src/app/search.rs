use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use iced::Task;
use sparky_ui::prompt;
use sparky_ui::viewport::Viewport;

use crate::app::background::background;
use crate::app::keys::Command;
use crate::app::message::Message;
use crate::app::sorting::detailed;
use crate::app::state::{Files, Mode};
use crate::domain::context::Action;
use crate::domain::listing::Listing;
use crate::domain::search::{Finish, Query, SearchEvent};

static NEXT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct Search {
    pub id: u64,
    pub query: String,
    pub cancel: Arc<AtomicBool>,
    pub finish: Option<Finish>,
}

impl Files {
    pub fn begin_search(&mut self) -> Task<Message> {
        if self.sidebar_page.is_some() || self.in_trash() {
            return Task::none();
        }
        self.input = self
            .search
            .as_ref()
            .map(|search| search.query.clone())
            .unwrap_or_default();
        self.mode = Mode::Search;
        prompt::focus()
    }

    pub(super) fn start_search(&mut self, input: &str) -> Task<Message> {
        let Some(query) = Query::new(input) else {
            return Task::none();
        };
        self.stop_search();
        let id = NEXT.fetch_add(1, Ordering::SeqCst);
        let cancel = Arc::new(AtomicBool::new(false));
        let hidden = self.listing.hidden();
        self.listing = Listing::arranged(Vec::new(), hidden, self.listing.order());
        self.viewport = Viewport::new(self.viewport.rows());
        self.notice = None;
        self.search = Some(Search {
            id,
            query: query.text().to_owned(),
            cancel: cancel.clone(),
            finish: None,
        });
        let finder = self.services.finder.clone();
        let root: PathBuf = self.location.clone();
        let stream = background(move |send: &dyn Fn(SearchEvent)| {
            let finish = finder.find(&root, &query, hidden, &cancel, &mut |batch| {
                send(SearchEvent::Found(batch))
            });
            send(SearchEvent::Finished(finish));
        });
        Task::run(stream, move |event| Message::Search(id, event))
    }

    pub fn search_event(&mut self, id: u64, event: SearchEvent) {
        let Some(search) = self.search.as_mut().filter(|search| search.id == id) else {
            self.stored_search_event(id, event);
            return;
        };
        match event {
            SearchEvent::Found(batch) => {
                let disk = self.services.disk.as_ref();
                let batch = detailed(disk, &self.location, self.listing.order(), batch);
                self.listing.extend(batch);
            }
            SearchEvent::Finished(finish) => search.finish = Some(finish),
        }
    }

    pub fn end_search(&mut self) {
        if self.stop_search() {
            self.go(self.location.clone());
        }
    }

    pub(super) fn stop_search(&mut self) -> bool {
        let Some(search) = self.search.take() else {
            return false;
        };
        search.cancel.store(true, Ordering::SeqCst);
        true
    }

    pub(super) fn prune_search(&mut self) {
        let disk = self.services.disk.as_ref();
        let location = &self.location;
        self.listing
            .retain(|entry| disk.exists(&location.join(&entry.name)));
    }

    pub fn blocks_in_search(&self, command: Command) -> bool {
        self.search.is_some()
            && matches!(
                command,
                Command::Create | Command::NewFolder | Command::Rename | Command::Paste
            )
    }

    pub fn blocks_action_in_search(&self, action: Action) -> bool {
        self.search.is_some()
            && matches!(
                action,
                Action::Rename | Action::NewFolder | Action::NewFile | Action::Paste
            )
    }
}
