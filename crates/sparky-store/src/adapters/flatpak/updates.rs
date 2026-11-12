use std::collections::HashMap;

use super::Flatpak;
use crate::domain::Installation;
use crate::domain::updates::{UpdateItem, UpdateKind, UpdateSource, valid_flatpak_ref};

fn kind_flag(kind: UpdateKind) -> &'static str {
    match kind {
        UpdateKind::Application => "--app",
        UpdateKind::Runtime => "--runtime",
        UpdateKind::System => "--app",
    }
}

fn full_ref(reference: &str, kind: UpdateKind) -> Result<String, String> {
    let prefix = match kind {
        UpdateKind::Application => "app",
        UpdateKind::Runtime => "runtime",
        UpdateKind::System => return Err("Invalid Flatpak update kind".into()),
    };
    let reference = if reference.starts_with("app/") || reference.starts_with("runtime/") {
        reference.to_owned()
    } else {
        format!("{prefix}/{reference}")
    };
    if !reference.starts_with(&format!("{prefix}/")) || !valid_flatpak_ref(&reference) {
        return Err(format!("Invalid Flatpak ref: {reference}"));
    }
    Ok(reference)
}

fn fields(line: &str, expected: usize) -> Result<Vec<&str>, String> {
    let parts: Vec<_> = line.split('\t').collect();
    if parts.len() != expected {
        return Err(format!("Flatpak update row has {} columns", parts.len()));
    }
    Ok(parts)
}

fn installed_versions(
    output: &str,
    kind: UpdateKind,
) -> Result<HashMap<(String, String), Option<String>>, String> {
    output
        .lines()
        .map(|line| {
            let parts = fields(line, 3)?;
            Ok((
                (full_ref(parts[0], kind)?, parts[2].to_owned()),
                optional(parts[1]),
            ))
        })
        .collect()
}

fn optional(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_owned())
}

fn parse(
    output: &str,
    scope: Installation,
    kind: UpdateKind,
    installed: &HashMap<(String, String), Option<String>>,
) -> Result<Vec<UpdateItem>, String> {
    output
        .lines()
        .filter_map(|line| {
            let parts = match fields(line, 4) {
                Ok(parts) => parts,
                Err(error) => return Some(Err(error)),
            };
            let reference = match full_ref(parts[0], kind) {
                Ok(reference) => reference,
                Err(error) => return Some(Err(error)),
            };
            let current = installed.get(&(reference.clone(), parts[3].to_owned()))?;
            Some(Ok(UpdateItem {
                id: reference,
                name: parts[1].to_owned(),
                installed: current.clone(),
                available: optional(parts[2]),
                source: UpdateSource::Flatpak(scope),
                kind,
            }))
        })
        .collect()
}

impl Flatpak {
    pub fn updates(&self, scope: Installation) -> Result<Vec<UpdateItem>, String> {
        let mut updates = Vec::new();
        for kind in [UpdateKind::Application, UpdateKind::Runtime] {
            let installed = self.runner.run(
                "flatpak",
                &[
                    scope.flag(),
                    "list",
                    kind_flag(kind),
                    "--all",
                    "--columns=ref:full,version:full,origin:full",
                ],
            )?;
            let installed = installed_versions(&installed, kind)?;
            let available = self.runner.run(
                "flatpak",
                &[
                    scope.flag(),
                    "remote-ls",
                    kind_flag(kind),
                    "--all",
                    "--updates",
                    "--columns=ref:full,name:full,version:full,origin:full",
                ],
            )?;
            updates.extend(parse(&available, scope, kind, &installed)?);
        }
        Ok(updates)
    }

    pub fn apply_update(&self, scope: Installation, reference: Option<&str>) -> Result<(), String> {
        let mut args = vec![scope.flag(), "update", "-y", "--noninteractive"];
        if let Some(reference) = reference {
            if !valid_flatpak_ref(reference) {
                return Err(crate::i18n::text("invalid-package"));
            }
            args.extend(["--", reference]);
        }
        self.runner.run("flatpak", &args).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::{installed_versions, parse};
    use crate::domain::Installation;
    use crate::domain::updates::UpdateKind;

    #[test]
    fn separa_escopos_e_preserva_ref_completo() {
        let installed = installed_versions(
            "org.gimp.GIMP/x86_64/beta\t3.0\tflathub\n",
            UpdateKind::Application,
        )
        .unwrap();
        let found = parse(
            "app/org.gimp.GIMP/x86_64/beta\tGIMP\t3.1\tflathub\n",
            Installation::User,
            UpdateKind::Application,
            &installed,
        )
        .unwrap();
        assert_eq!(found[0].id, "app/org.gimp.GIMP/x86_64/beta");
        assert_eq!(found[0].installed.as_deref(), Some("3.0"));
        assert_eq!(found[0].available.as_deref(), Some("3.1"));
        assert_eq!(found[0].kind, UpdateKind::Application);
        assert!(
            parse(
                "app/org.gimp.GIMP/x86_64/stable\tGIMP\t3.1\tflathub\n",
                Installation::User,
                UpdateKind::Application,
                &installed
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn runtime_e_linha_invalida_sao_tratados() {
        let installed = installed_versions(
            "org.freedesktop.Platform/x86_64/24.08\t24.08\tflathub\n",
            UpdateKind::Runtime,
        )
        .unwrap();
        let found = parse(
            "runtime/org.freedesktop.Platform/x86_64/24.08\tPlatform\t24.08.1\tflathub\n",
            Installation::System,
            UpdateKind::Runtime,
            &installed,
        )
        .unwrap();
        assert_eq!(found[0].kind, UpdateKind::Runtime);
        assert!(
            parse(
                "broken\n",
                Installation::System,
                UpdateKind::Runtime,
                &installed
            )
            .is_err()
        );
    }
}
