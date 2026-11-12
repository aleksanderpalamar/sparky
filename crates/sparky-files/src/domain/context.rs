#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    File,
    Archive,
    Folder,
    Background,
    Trashed,
    TrashBackground,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    Item {
        action: Action,
        label: &'static str,
        shortcut: &'static str,
        enabled: bool,
    },
    Separator,
}

impl Row {
    pub fn action(self) -> Option<Action> {
        match self {
            Self::Item { action, .. } => Some(action),
            Self::Separator => None,
        }
    }

    pub fn is_disabled(self, sought: Action) -> bool {
        matches!(self, Self::Item { action, enabled: false, .. } if action == sought)
    }
}

pub fn entries(target: Target, has_clip: bool, has_entries: bool) -> Vec<Row> {
    if target == Target::Trashed {
        return vec![item(Action::Restore, true), item(Action::Properties, true)];
    }
    if target == Target::TrashBackground {
        return vec![item(Action::RestoreAll, has_entries)];
    }
    if target == Target::Background {
        return background(has_clip, has_entries);
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

fn background(has_clip: bool, has_entries: bool) -> Vec<Row> {
    vec![
        item(Action::NewFolder, true),
        item(Action::NewFile, true),
        item(Action::OpenWith, true),
        Row::Separator,
        item(Action::Paste, has_clip),
        item(Action::SelectAll, has_entries),
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
            Row::Separator => 10.0,
        })
        .sum::<f32>()
}

pub fn position(pointer: (f32, f32), window: (f32, f32), menu: (f32, f32)) -> (f32, f32) {
    (
        pointer.0.max(0.0).min((window.0 - menu.0).max(0.0)),
        pointer.1.max(0.0).min((window.1 - menu.1).max(0.0)),
    )
}
