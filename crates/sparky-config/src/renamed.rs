use crate::edit::Editor;

const RENAMED: [(&str, &str); 2] = [
    ("/wallpapers/spark/", "/wallpapers/sparky/"),
    ("/wallpapers/spark-98/", "/wallpapers/sparky-98/"),
];

const FIELDS: [(&str, &str); 2] = [("general", "wallpaper"), ("login", "wallpaper")];

pub fn migrate(contents: &str) -> Option<String> {
    let mut editor = Editor::parse(contents).ok()?;
    let updates: Vec<_> = FIELDS
        .iter()
        .filter_map(|&(table, key)| {
            let old = editor.text(table, key)?;
            let new = wallpaper(old.to_owned());
            (new != old).then_some((table, key, new))
        })
        .collect();
    if updates.is_empty() {
        return None;
    }
    for (table, key, new) in updates {
        editor.set_text(table, key, Some(&new));
    }
    Some(editor.contents())
}

pub fn wallpaper(path: String) -> String {
    RENAMED
        .iter()
        .find_map(|(old, new)| {
            path.find(old)
                .map(|at| format!("{}{new}{}", &path[..at], &path[at + old.len()..]))
        })
        .unwrap_or(path)
}
