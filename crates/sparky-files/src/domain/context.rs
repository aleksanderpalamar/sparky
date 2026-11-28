use crate::domain::tag_color::TagColor;

pub const PALETTE_HEIGHT: f32 = 56.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    File,
    Archive,
    Folder,
    Background,
    Trashed,
    TrashBackground,
    Bookmark(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Open,
    OpenWith,
    Cut,
    Copy,
    MoveTo,
    CopyTo,
    Rename,
    Compress,
    Extract,
    Email,
    Trash,
    Delete,
    Properties,
    NewFolder,
    NewFile,
    Paste,
    SelectAll,
    Restore,
    RestoreAll,
    Tag(Option<TagColor>),
    RemoveBookmark,
    Back,
    Forward,
    EditLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Item {
        action: Action,
        label: &'static str,
        shortcut: &'static str,
        enabled: bool,
    },
    Palette,
    Separator,
}

impl Row {
    pub fn action(self) -> Option<Action> {
        match self {
            Self::Item { action, .. } => Some(action),
            Self::Palette | Self::Separator => None,
        }
    }

    pub fn offers(self, sought: Action) -> bool {
        match self {
            Self::Item {
                action, enabled, ..
            } => enabled && action == sought,
            Self::Palette => matches!(sought, Action::Tag(Some(_))),
            Self::Separator => false,
        }
    }

    pub fn is_disabled(self, sought: Action) -> bool {
        matches!(self, Self::Item { action, enabled: false, .. } if action == sought)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Offer {
    pub clip: bool,
    pub entries: bool,
    pub back: bool,
    pub forward: bool,
}

pub fn entries(target: Target, offer: Offer) -> Vec<Row> {
    if let Target::Bookmark(_) = target {
        return vec![
            Row::Palette,
            item(Action::Tag(None), true),
            Row::Separator,
            item(Action::RemoveBookmark, true),
        ];
    }
    if target == Target::Trashed {
        return vec![item(Action::Restore, true), item(Action::Properties, true)];
    }
    if target == Target::TrashBackground {
        let mut rows = travel(offer);
        rows.push(item(Action::RestoreAll, offer.entries));
        return rows;
    }
    if target == Target::Background {
        let mut rows = travel(offer);
        rows.extend(background(offer));
        return rows;
    }
    let mut rows = vec![
        item(Action::Open, true),
        item(Action::OpenWith, true),
        Row::Separator,
    ];
    rows.extend(
        [Action::Cut, Action::Copy, Action::MoveTo, Action::CopyTo]
            .map(|action| item(action, true)),
    );
    rows.push(Row::Separator);
    rows.extend([Action::Rename, Action::Compress].map(|action| item(action, true)));
    if target == Target::Archive {
        rows.push(item(Action::Extract, true));
    }
    rows.push(item(
        Action::Email,
        matches!(target, Target::File | Target::Archive),
    ));
    rows.extend([Action::Trash, Action::Delete].map(|action| item(action, true)));
    rows.extend([Row::Separator, item(Action::Properties, true)]);
    rows
}

fn travel(offer: Offer) -> Vec<Row> {
    vec![
        item(Action::Back, offer.back),
        item(Action::Forward, offer.forward),
        item(Action::EditLocation, true),
        Row::Separator,
    ]
}

fn background(offer: Offer) -> Vec<Row> {
    vec![
        item(Action::NewFolder, true),
        item(Action::NewFile, true),
        item(Action::OpenWith, true),
        Row::Separator,
        item(Action::Paste, offer.clip),
        item(Action::SelectAll, offer.entries),
        Row::Separator,
        item(Action::Properties, true),
    ]
}

fn item(action: Action, enabled: bool) -> Row {
    let (label, shortcut) = match action {
        Action::Open => ("menu-open", "Enter"),
        Action::OpenWith => ("menu-open-with", ""),
        Action::Cut => ("menu-cut", "Ctrl+X"),
        Action::Copy => ("menu-copy", "Ctrl+C"),
        Action::MoveTo => ("menu-move-to", ""),
        Action::CopyTo => ("menu-copy-to", ""),
        Action::Rename => ("menu-rename", "F2"),
        Action::Compress => ("menu-compress", ""),
        Action::Extract => ("menu-extract", "e"),
        Action::Email => ("menu-email", ""),
        Action::Trash => ("menu-trash", "Delete"),
        Action::Delete => ("menu-delete", "Shift+Delete"),
        Action::Properties => ("menu-properties", "Alt+Enter"),
        Action::NewFolder => ("menu-new-folder", "Ctrl+Shift+N"),
        Action::NewFile => ("menu-new-file", ""),
        Action::Paste => ("menu-paste", "Ctrl+V"),
        Action::SelectAll => ("menu-select-all", "Ctrl+A"),
        Action::Restore => ("menu-restore", ""),
        Action::RestoreAll => ("menu-restore-all", ""),
        Action::Tag(Some(color)) => (color.label(), ""),
        Action::Tag(None) => ("menu-no-color", ""),
        Action::RemoveBookmark => ("menu-remove-bookmark", ""),
        Action::Back => ("menu-back", "Alt+←"),
        Action::Forward => ("menu-forward", "Alt+→"),
        Action::EditLocation => ("menu-edit-location", "Ctrl+L"),
    };
    Row::Item {
        action,
        label,
        shortcut,
        enabled,
    }
}

pub fn height(rows: &[Row]) -> f32 {
    12.0 + rows
        .iter()
        .map(|row| match row {
            Row::Item { .. } => 32.0,
            Row::Palette => PALETTE_HEIGHT,
            Row::Separator => 10.0,
        })
        .sum::<f32>()
}
