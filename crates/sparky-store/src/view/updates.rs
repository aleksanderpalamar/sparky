mod confirmation;
mod groups;

use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, Store};
use crate::domain::updates::UpdateTarget;
use crate::i18n;

use super::{navigation, styles};

pub fn view(store: &Store) -> Element<'_, Message> {
    let count = store.updates.as_ref().map_or(0, |report| report.count());
    let mut header = row![
        text(i18n::text("updates")).size(21).width(Length::Fill),
        navigation::control(store, &i18n::text("refresh"), Message::RefreshUpdates),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if count > 0 {
        header = header.push(navigation::control(
            store,
            &i18n::text("update-all"),
            Message::AskUpdateAll,
        ));
    }
    let mut fixed = column![header].spacing(12);
    if let Some(target) = &store.updating {
        fixed = fixed.push(status(store, target));
    } else if store.busy {
        fixed = fixed.push(text(i18n::text("checking-updates")).size(13));
    }
    if let Some(targets) = &store.update_pending {
        fixed = fixed.push(confirmation::view(store, targets));
    }
    let mut content = column![].spacing(18).padding([0, 24]);
    if let Some(report) = &store.updates {
        if let Some(warning) = &report.warning {
            content = content.push(text(warning).size(12).color(store.skin.urgent));
        }
        if count == 0
            && !store.busy
            && report.warning.is_none()
            && report.groups.iter().all(|group| group.error.is_none())
        {
            content = content.push(empty(store));
        }
        if count > 0 {
            content = content
                .push(text(i18n::with("updates-found", "count", count.to_string())).size(13));
        }
        for group in &report.groups {
            if !group.items.is_empty() || group.error.is_some() {
                content = content.push(groups::view(store, group));
            }
        }
    }
    column![
        container(fixed).padding([18, 24]).width(Length::Fill),
        scrollable(content).width(Length::Fill).height(Length::Fill),
    ]
    .height(Length::Fill)
    .into()
}

fn empty(store: &Store) -> Element<'_, Message> {
    let skin = store.skin;
    container(
        column![
            text("✓").size(56).color(skin.accent),
            text(i18n::text("up-to-date")).size(20),
            text(i18n::text("up-to-date-detail"))
                .size(12)
                .color(skin.dark_foreground),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .center_x(Length::Fill)
    .padding(48)
    .width(Length::Fill)
    .style(move |_| styles::surface(skin))
    .into()
}

fn status<'a>(store: &'a Store, target: &'a UpdateTarget) -> Element<'a, Message> {
    let skin = store.skin;
    container(
        text(i18n::with(
            "updating-source",
            "source",
            groups::target_label(target),
        ))
        .size(13),
    )
    .padding(14)
    .width(Length::Fill)
    .style(move |_| styles::feature(skin))
    .into()
}
