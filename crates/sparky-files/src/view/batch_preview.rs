use crate::app::batch::described;
use crate::app::state::Files;
use crate::domain::batch_name::{check, parse, plan};
use crate::i18n;

const SHOWN: usize = 2;

pub fn preview(files: &Files, names: &[String]) -> String {
    let Some(pattern) = parse(&files.input) else {
        return i18n::text("batch-hint");
    };
    let pairs = plan(names, &pattern);
    let disk = files.services.disk.as_ref();
    if let Err(problem) = check(&pairs, |name| disk.exists(&files.location.join(name))) {
        return described(&problem);
    }
    let changed: Vec<String> = pairs
        .iter()
        .filter(|(old, new)| old != new)
        .map(|(old, new)| format!("{old} → {new}"))
        .collect();
    let mut parts: Vec<String> = changed.iter().take(SHOWN).cloned().collect();
    if changed.len() > SHOWN {
        parts.push(format!("+{}", changed.len() - SHOWN));
    }
    parts.join(" · ")
}
