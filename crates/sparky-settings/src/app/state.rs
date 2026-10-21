use std::fmt::Display;
use std::path::PathBuf;

use sparky_config::{Bind, Config, Editor, binds, parse};
use sparky_ipc::OutputInfo;
use sparky_theme::Skin;
use sparky_ui::notice::Notice;
use sparky_ui::viewport::Viewport;

use crate::domain::about::About;
use crate::domain::field::Field;
use crate::domain::keymaps::Keymaps;
use crate::domain::section::Section;
use crate::domain::system::power::Power;
use crate::ports::commands::CommandRunner;
use crate::ports::{Compositor, CompositorError, Store};

#[derive(Debug, Clone)]
pub enum Source {
    Editable(Editor),
    Broken(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Existing(usize),
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Field(Field),
    Autostart(Slot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browse,
    Editing(Target),
}

pub struct Services {
    pub store: Box<dyn Store>,
    pub compositor: Box<dyn Compositor>,
    pub commands: Box<dyn CommandRunner>,
}

pub struct Context {
    pub home: PathBuf,
    pub about: About,
    pub keymaps: Keymaps,
    pub skin: Skin,
    pub rows: usize,
}

pub struct Settings {
    pub section: Section,
    pub focus: usize,
    pub config: Config,
    pub source: Source,
    pub mode: Mode,
    pub input: String,
    pub notice: Option<Notice>,
    pub outputs: Result<Vec<OutputInfo>, CompositorError>,
    pub power: Power,
    pub list: Viewport,
    pub context: Context,
    pub services: Services,
}

impl Settings {
    pub fn new(services: Services, context: Context) -> Self {
        let mut settings = Self {
            section: Section::default(),
            focus: 0,
            config: Config::default(),
            source: Source::Editable(Editor::default()),
            mode: Mode::Browse,
            input: String::new(),
            notice: None,
            outputs: Ok(Vec::new()),
            power: Power::default(),
            list: Viewport::new(context.rows),
            context,
            services,
        };
        settings.reload();
        settings
    }

    pub fn reload(&mut self) {
        self.outputs = self.services.compositor.outputs();
        self.reload_system();
        match self.services.store.load() {
            Ok(contents) => self.read(contents.as_deref().unwrap_or_default()),
            Err(error) => self.broken(format!("não deu para ler o sparky.toml: {error}")),
        }
    }

    pub fn binds(&self) -> Vec<Bind> {
        let terminal = self.config.apps.terminal.clone().unwrap_or_else(|| "o terminal".to_owned());
        binds::merged(&self.config, &terminal)
    }

    pub fn inform(&mut self, text: impl Into<String>) {
        self.notice = Some(Notice::Info(text.into()));
    }

    pub fn fail(&mut self, error: impl Display) {
        self.notice = Some(Notice::Error(error.to_string()));
    }

    fn read(&mut self, contents: &str) {
        let editor = match Editor::parse(contents) {
            Ok(editor) => editor,
            Err(error) => return self.broken(error.to_string()),
        };
        self.source = Source::Editable(editor);
        match parse(contents) {
            Ok(config) => self.config = config,
            Err(error) => {
                self.config = Config::default();
                self.fail(format!("o sparky.toml tem um erro: {error}"));
            }
        }
    }

    fn broken(&mut self, reason: String) {
        self.config = Config::default();
        self.fail(&reason);
        self.source = Source::Broken(reason);
    }
}
