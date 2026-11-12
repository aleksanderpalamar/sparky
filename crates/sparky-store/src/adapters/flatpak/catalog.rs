use std::collections::{HashMap, HashSet};

use crate::domain::{Application, Installation, Source, valid_id};

pub(super) struct SearchRow {
    pub name: String,
    pub description: Option<String>,
    pub application_id: String,
    pub version: Option<String>,
    pub remotes: String,
    branch: String,
}

fn rows<const N: usize>(output: &str) -> Result<Vec<[&str; N]>, String> {
    if output.trim().is_empty() || output.trim() == "No matches found" {
        return Ok(Vec::new());
    }
    output
        .lines()
        .enumerate()
        .map(|(index, line)| {
            line.split('\t')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| format!("Flatpak row {} has invalid columns", index + 1))
        })
        .collect()
}

fn optional(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

pub(super) fn parse_search(output: &str) -> Result<Vec<SearchRow>, String> {
    let mut found: Vec<_> = rows::<6>(output)?
        .into_iter()
        .map(
            |[application_id, name, description, version, remotes, branch]| SearchRow {
                application_id: application_id.to_owned(),
                name: name.to_owned(),
                description: optional(description),
                version: optional(version),
                remotes: remotes.to_owned(),
                branch: branch.to_owned(),
            },
        )
        .collect();
    found.sort_by_key(|row| row.branch != "stable");
    Ok(found)
}

pub(super) fn parse_installed(
    output: &str,
    scope: Installation,
) -> Result<Vec<Application>, String> {
    Ok(rows::<5>(output)?
        .into_iter()
        .filter(|row| row[4] == "flathub" && valid_id(row[2]))
        .map(
            |[name, description, application_id, version, _]| Application {
                id: application_id.to_owned(),
                name: name.to_owned(),
                summary: description.to_owned(),
                version: optional(version),
                source: Source::Flathub(scope),
                installed: true,
            },
        )
        .collect())
}

pub fn parse(
    output: &str,
    scope: Installation,
    installed: &HashMap<String, Source>,
) -> Result<Vec<Application>, String> {
    let mut rows = rows::<5>(output)?;
    rows.sort_by(|left, right| {
        left[1]
            .to_lowercase()
            .cmp(&right[1].to_lowercase())
            .then_with(|| (right[4] == "stable").cmp(&(left[4] == "stable")))
    });
    let mut seen = HashSet::new();
    Ok(rows
        .into_iter()
        .filter(|row| valid_id(row[0]) && seen.insert(row[0].to_owned()))
        .map(
            |[application_id, name, description, version, _]| Application {
                installed: installed.contains_key(application_id),
                source: installed
                    .get(application_id)
                    .copied()
                    .unwrap_or(Source::Flathub(scope)),
                id: application_id.to_owned(),
                name: name.to_owned(),
                summary: description.to_owned(),
                version: optional(version),
            },
        )
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{parse, parse_installed, parse_search};
    use crate::domain::{Installation, Source};
    use std::collections::HashMap;

    #[test]
    fn catalogo_deduplica_e_mantem_escopo_instalado() {
        let rows = "org.gimp.GIMP\tGIMP\t\t2.10\t2.10\norg.gimp.GIMP\tGIMP\t\t3.0\tstable\n";
        let installed = HashMap::from([(
            "org.gimp.GIMP".to_owned(),
            Source::Flathub(Installation::User),
        )]);
        let apps = parse(rows, Installation::System, &installed).unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].version.as_deref(), Some("3.0"));
        assert_eq!(apps[0].source, Source::Flathub(Installation::User));
        assert!(apps[0].installed);
    }

    #[test]
    fn campos_vazios_e_linhas_invalidas_sao_tratados() {
        let rows = "GIMP\t\torg.gimp.GIMP\t\tflathub\n";
        let apps = parse_installed(rows, Installation::System).unwrap();
        assert_eq!(apps[0].summary, "");
        assert_eq!(apps[0].version, None);
        assert!(parse_search("invalid\trow\n").is_err());
    }
}
