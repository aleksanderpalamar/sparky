use crate::domain::groups::Group;

pub fn read(contents: &str) -> Vec<Group> {
    contents.lines().filter_map(parse).collect()
}

pub fn write(groups: &[Group]) -> String {
    groups.iter().map(render).collect()
}

fn parse(line: &str) -> Option<Group> {
    let (name, apps) = line.split_once('\t').unwrap_or((line, ""));
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    Some(Group {
        name: name.to_owned(),
        apps: apps
            .split(';')
            .map(str::trim)
            .filter(|app| !app.is_empty())
            .map(str::to_owned)
            .collect(),
    })
}

fn render(group: &Group) -> String {
    let name = group.name.replace(['\t', '\n'], " ");
    format!("{name}\t{}\n", group.apps.join(";"))
}
