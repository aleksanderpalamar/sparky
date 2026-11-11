use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::domain::{Application, Installation, Source, valid_id};

#[derive(Debug, Deserialize)]
struct CatalogRow {
    name: String,
    description: Option<String>,
    application_id: String,
    version: Option<String>,
    branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct SearchRow {
    pub name: String,
    pub description: Option<String>,
    pub application_id: String,
    pub version: Option<String>,
    pub remotes: String,
    branch: Option<String>,
}

pub(super) fn parse_search(output: &str) -> Result<Vec<SearchRow>, String> {
    if output.trim() == "No matches found" {
        return Ok(Vec::new());
    }
    let mut rows: Vec<SearchRow> =
        serde_json::from_str(output).map_err(|error| error.to_string())?;
    rows.sort_by_key(|row| row.branch.as_deref() != Some("stable"));
    Ok(rows)
}

pub fn parse(
    output: &str,
    scope: Installation,
    installed: &HashMap<String, Source>,
) -> Result<Vec<Application>, String> {
    let mut rows: Vec<CatalogRow> =
        serde_json::from_str(output).map_err(|error| error.to_string())?;
    rows.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| {
                (right.branch.as_deref() == Some("stable"))
                    .cmp(&(left.branch.as_deref() == Some("stable")))
            })
    });
    let mut seen = HashSet::new();
    Ok(rows
        .into_iter()
        .filter(|row| valid_id(&row.application_id) && seen.insert(row.application_id.clone()))
        .map(|row| Application {
            installed: installed.contains_key(&row.application_id),
            source: installed
                .get(&row.application_id)
                .copied()
                .unwrap_or(Source::Flathub(scope)),
            id: row.application_id,
            name: row.name,
            summary: row.description.unwrap_or_default(),
            version: row.version,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::domain::{Installation, Source};
    use std::collections::HashMap;

    #[test]
    fn catalogo_deduplica_e_mantem_escopo_instalado() {
        let rows = r#"[{"name":"GIMP","application_id":"org.gimp.GIMP","version":"2.10","branch":"2.10"},{"name":"GIMP","application_id":"org.gimp.GIMP","version":"3.0","branch":"stable"}]"#;
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
}
