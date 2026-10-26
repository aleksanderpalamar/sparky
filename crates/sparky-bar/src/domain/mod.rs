pub mod audio;
pub mod bluetooth;
pub mod clock;
pub mod indicator;
pub mod label;
pub mod network;
pub mod panel;
pub mod pixmap;
pub mod prompt;
pub mod reveal;
pub mod secret;
pub mod slots;
pub mod tray;
pub mod tray_layout;

#[cfg(test)]
mod indicator_tests;
#[cfg(test)]
mod reveal_tests;
#[cfg(test)]
mod tray_layout_tests;

#[cfg(test)]
mod panel_domain_tests;
