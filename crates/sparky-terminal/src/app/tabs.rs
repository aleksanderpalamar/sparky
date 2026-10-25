use iced::Task;

use super::message::Message;
use super::state::Terminal;
use super::workspace::{TAB_HEIGHT, Workspace};
use crate::domain::input::Focus;
use crate::domain::launch::{Launch, Program};
use crate::domain::tabs::{TabAction, TabId};

impl Workspace {
    pub fn new_tab(&mut self) -> Option<TabId> {
        let launch = Launch {
            program: Program::Shell,
            directory: self.launch.directory.clone(),
        };
        let mut terminal = Terminal::new(launch, self.colors, self.palette, self.cell);
        terminal.fit(self.size.width, (self.size.height - TAB_HEIGHT).max(0.0));
        terminal.rescale(self.scale);
        let previous = self.tabs.active_id();
        let id = self.tabs.insert(terminal)?;
        self.changed_active(previous);
        Some(id)
    }

    pub fn select(&mut self, id: TabId) {
        let previous = self.tabs.active_id();
        if previous == Some(id) || !self.tabs.select(id) {
            return;
        }
        self.changed_active(previous);
    }

    pub fn cycle(&mut self, step: i8) {
        let previous = self.tabs.active_id();
        self.tabs.cycle(step);
        self.changed_active(previous);
    }

    pub fn close(&mut self, id: TabId) -> bool {
        let previous = self.tabs.active_id();
        if self.tabs.remove(id).is_none() {
            return false;
        }
        self.changed_active(previous);
        self.tabs.is_empty()
    }

    fn changed_active(&mut self, previous: Option<TabId>) {
        if previous == self.tabs.active_id() {
            return;
        }
        if self.focus == Focus::Gained {
            self.report_previous(previous);
            self.report_active(Focus::Gained);
        }
        if let Some(tab) = self.tabs.active_mut() {
            tab.value.refresh();
        }
    }

    fn report_previous(&mut self, previous: Option<TabId>) {
        let Some(tab) = previous.and_then(|id| self.tabs.get_mut(id)) else {
            return;
        };
        super::input::focus(&mut tab.value, Focus::Lost);
    }
}

pub fn action(app: &mut Workspace, action: TabAction) -> Task<Message> {
    match action {
        TabAction::New => {
            app.new_tab();
        }
        TabAction::Close => {
            let Some(id) = app.tabs.active_id() else {
                return Task::none();
            };
            return close(app, id);
        }
        TabAction::Cycle(step) => app.cycle(step),
    }
    super::tabs_view::reveal(app)
}

pub fn close(app: &mut Workspace, id: TabId) -> Task<Message> {
    match app.close(id) {
        true => iced::exit(),
        false => super::tabs_view::reveal(app),
    }
}
