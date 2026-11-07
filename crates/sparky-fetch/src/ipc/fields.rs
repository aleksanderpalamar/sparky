use sparky_ipc::{OutputInfo, SessionInfo, SessionKind};
use sparky_theme::Rgb;

use crate::domain::field::Field;
use crate::domain::status::{Reason, Status, Value};
use crate::i18n;
use crate::ipc::port::{Problem, Snapshot};

pub fn status(field: Field, snapshot: &Snapshot) -> Status {
    match field {
        Field::Compositor => from(&snapshot.info, compositor),
        Field::Session => from(&snapshot.info, session),
        Field::Theme => from(&snapshot.info, theme),
        Field::Colors => from(&snapshot.info, colors),
        Field::Monitors => from(&snapshot.outputs, |outputs| listed(outputs, monitor)),
        Field::Resolution => from(&snapshot.outputs, |outputs| listed(outputs, resolution)),
        _ => Status::unavailable(Reason::Uncollected),
    }
}

fn from<T>(result: &Result<T, Problem>, ready: impl Fn(&T) -> Status) -> Status {
    match result {
        Ok(value) => ready(value),
        Err(Problem::NoCompositor) => Status::unavailable(Reason::NoCompositor),
        Err(Problem::Unsupported) => Status::unavailable(Reason::Unsupported),
        Err(Problem::TimedOut) => Status::TimedOut,
        Err(Problem::Failed(error)) => Status::failed(error.clone()),
    }
}

fn compositor(info: &SessionInfo) -> Status {
    Status::text(i18n::with("compositor-version", "version", &info.version))
}

fn session(info: &SessionInfo) -> Status {
    let key = match info.session {
        SessionKind::Real => "session-real",
        SessionKind::Nested => "session-nested",
        SessionKind::Unknown => "session-unknown",
    };
    Status::text(i18n::with(key, "display", &info.display))
}

fn theme(info: &SessionInfo) -> Status {
    match &info.theme {
        Some(name) => Status::text(name.clone()),
        None => Status::text(i18n::text("theme-custom")),
    }
}

fn colors(info: &SessionInfo) -> Status {
    let Some(accent) = Rgb::parse(&info.accent) else {
        return Status::unavailable(Reason::Missing);
    };
    let swatches = info.swatches.iter().filter_map(|swatch| Rgb::parse(swatch));
    Status::ready(Value::Colors(
        std::iter::once(accent).chain(swatches).collect(),
    ))
}

fn listed(outputs: &[OutputInfo], line: fn(&OutputInfo) -> String) -> Status {
    let lines: Vec<String> = outputs
        .iter()
        .filter(|output| output.enabled)
        .map(line)
        .collect();
    match lines.is_empty() {
        true => Status::unavailable(Reason::Missing),
        false => Status::lines(lines),
    }
}

fn monitor(output: &OutputInfo) -> String {
    match output.description.trim() {
        "" => output.name.clone(),
        description => format!("{} · {description}", output.name),
    }
}

fn resolution(output: &OutputInfo) -> String {
    let size = match output.mode {
        Some(mode) => format!(
            "{}x{} @ {} Hz",
            mode.width,
            mode.height,
            (mode.refresh + 500) / 1000
        ),
        None => format!("{}x{}", output.width, output.height),
    };
    match scale(output.scale.factor()) {
        Some(scale) => format!("{size} (×{scale})"),
        None => size,
    }
}

fn scale(factor: f64) -> Option<String> {
    if (factor - 1.0).abs() < f64::EPSILON {
        return None;
    }
    let text = format!("{factor:.2}");
    Some(text.trim_end_matches('0').trim_end_matches('.').to_owned())
}
