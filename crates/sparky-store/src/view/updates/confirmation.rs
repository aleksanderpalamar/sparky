use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::app::{Message, Store};
use crate::domain::Manager;
use crate::domain::updates::UpdateTarget;
use crate::i18n;

use super::super::{navigation, styles};
use super::groups::target_label;

pub fn view<'a>(store: &Store, targets: &[UpdateTarget]) -> Element<'a, Message> {
    let skin = store.skin;
    let mut body = column![text(label(store, targets)).size(14)].spacing(12);
    if targets.iter().any(|target| {
        matches!(
            target,
            UpdateTarget::Native {
                manager: Manager::Pacman,
                ..
            }
        )
    }) {
        body = body.push(text(i18n::text("pacman-system-update")).size(12));
    }
    body = body.push(
        row![
            navigation::control(store, &i18n::text("cancel"), Message::Cancel),
            navigation::control(store, &i18n::text("confirm"), Message::ConfirmUpdate),
        ]
        .spacing(8),
    );
    container(body)
        .padding(20)
        .width(Length::Fill)
        .style(move |_| styles::feature(skin))
        .into()
}

fn label(store: &Store, targets: &[UpdateTarget]) -> String {
    if targets.len() > 1 {
        let count = store.updates.as_ref().map_or(0, |report| report.count());
        return i18n::with("confirm-all-updates", "count", count.to_string());
    }
    let Some(target) = targets.first() else {
        return i18n::text("confirm-updates");
    };
    match target {
        UpdateTarget::Flatpak {
            reference: None, ..
        } => i18n::with("confirm-update-group", "name", target_label(target)),
        UpdateTarget::Native { package: None, .. } => {
            i18n::with("confirm-system-updates", "name", target_label(target))
        }
        UpdateTarget::Flatpak {
            reference: Some(_), ..
        }
        | UpdateTarget::Native {
            package: Some(_), ..
        } => {
            let name = store
                .updates
                .as_ref()
                .and_then(|report| {
                    report
                        .groups
                        .iter()
                        .flat_map(|group| group.items.iter())
                        .find(|item| item.target() == *target)
                })
                .map_or_else(|| target_label(target), |item| item.name.clone());
            i18n::with("confirm-update-item", "name", name)
        }
    }
}
