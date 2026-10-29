#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    File,
    Folder,
    Background,
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
    Email,
    Trash,
    Properties,
    NewFolder,
    NewFile,
    Paste,
    SelectAll,
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
    rows.push(item(Action::Email, target == Target::File));
    rows.push(item(Action::Trash, true));
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
        Action::Open => ("Abrir", "Enter"),
        Action::OpenWith => ("Abrir com...", ""),
        Action::Cut => ("Recortar", "Ctrl+X"),
        Action::Copy => ("Copiar", "Ctrl+C"),
        Action::MoveTo => ("Mover para...", ""),
        Action::CopyTo => ("Copiar para...", ""),
        Action::Rename => ("Renomear...", "F2"),
        Action::Compress => ("Comprimir...", ""),
        Action::Email => ("E-mail...", ""),
        Action::Trash => ("Mover para a lixeira", "Delete"),
        Action::Properties => ("Propriedades", "Alt+Enter"),
        Action::NewFolder => ("Nova pasta...", "Ctrl+Shift+N"),
        Action::NewFile => ("Novo documento...", ""),
        Action::Paste => ("Colar", "Ctrl+V"),
        Action::SelectAll => ("Selecionar tudo", "Ctrl+A"),
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
