pub mod actions;
pub mod events;
pub mod keys;
pub mod message;
pub mod page;
pub mod sampling;
pub mod selection;
pub mod state;
pub mod update;

#[cfg(test)]
mod keys_tests;
#[cfg(test)]
mod monitor_tests;
#[cfg(test)]
mod stop_tests;
#[cfg(test)]
pub(crate) mod test_support;

use std::sync::Arc;

use iced::Task;
use sparky_theme::Skin;

use crate::adapters::catalog::DesktopCatalog;
use crate::adapters::root_fs::RootFs;
use crate::adapters::signals::ProcessSignals;
use crate::adapters::statvfs::Statvfs;
use crate::app::message::Message;
use crate::app::state::{Monitor, Services};
use crate::sampler::Sampler;
use crate::view::{self, style};

pub fn run() -> iced::Result {
    iced::application(boot, update::update, view::view)
        .title(view::title)
        .subscription(events::subscription)
        .default_font(sparky_ui::style::FONT)
        .theme(|monitor: &Monitor| monitor.skin.theme())
        .window(sparky_ui::window::settings(
            style::APPLICATION_ID,
            style::WINDOW,
        ))
        .run()
}

fn boot() -> (Monitor, Task<Message>) {
    let page_size = rustix::param::page_size() as u64;
    let services = Services {
        sampler: Arc::new(Sampler::new(Arc::new(RootFs), Arc::new(Statvfs), page_size)),
        catalog: Arc::new(DesktopCatalog),
        signals: Arc::new(ProcessSignals),
        user: rustix::process::getuid().as_raw(),
    };
    let mut monitor = Monitor::new(
        services,
        Skin::new(sparky_theme::current_palette()),
        style::WINDOW,
    );
    let task = Task::batch([monitor.load_catalog(), monitor.tick()]);
    (monitor, task)
}
